use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use futures::StreamExt;
use libp2p::gossipsub::{self, IdentTopic, MessageAuthenticity};
use libp2p::identity::Keypair;
use libp2p::mdns;
use libp2p::swarm::SwarmEvent;
use libp2p::{noise, tcp, yamux, Multiaddr, SwarmBuilder};
use tokio::sync::mpsc;

use crate::behaviour::{BlockchainBehaviour, BlockchainBehaviourEvent};
use crate::handler::{NetworkCommand, NetworkEvent};
use crate::messages::NetworkMessage;

const BLOCKS_TOPIC: &str = "blockchain-blocks";
const TRANSACTIONS_TOPIC: &str = "blockchain-transactions";

pub struct NetworkNode {
    command_rx: mpsc::Receiver<NetworkCommand>,
    event_tx: mpsc::Sender<NetworkEvent>,
    listen_port: u16,
}

impl NetworkNode {
    pub fn new(
        command_rx: mpsc::Receiver<NetworkCommand>,
        event_tx: mpsc::Sender<NetworkEvent>,
        listen_port: u16,
    ) -> Self {
        Self {
            command_rx,
            event_tx,
            listen_port,
        }
    }

    pub async fn run(mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let local_key = Keypair::generate_ed25519();

        let mut swarm = SwarmBuilder::with_existing_identity(local_key)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|key| {
                let message_id_fn = |message: &gossipsub::Message| {
                    let mut s = DefaultHasher::new();
                    message.data.hash(&mut s);
                    gossipsub::MessageId::from(s.finish().to_string())
                };

                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(10))
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .message_id_fn(message_id_fn)
                    .build()
                    .expect("Valid gossipsub config");

                let gossipsub = gossipsub::Behaviour::new(
                    MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )
                .expect("Valid gossipsub behaviour");

                let mdns = mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                )
                .expect("Valid mDNS behaviour");

                BlockchainBehaviour { gossipsub, mdns }
            })?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        let blocks_topic = IdentTopic::new(BLOCKS_TOPIC);
        let transactions_topic = IdentTopic::new(TRANSACTIONS_TOPIC);

        swarm.behaviour_mut().gossipsub.subscribe(&blocks_topic)?;
        swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&transactions_topic)?;

        let listen_addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", self.listen_port).parse()?;
        swarm.listen_on(listen_addr)?;

        tracing::info!("Network node listening on port {}", self.listen_port);

        loop {
            tokio::select! {
                Some(cmd) = self.command_rx.recv() => {
                    self.handle_command(&mut swarm, &blocks_topic, &transactions_topic, cmd);
                }
                event = swarm.select_next_some() => {
                    self.handle_swarm_event(&mut swarm, event).await;
                }
            }
        }
    }

    fn handle_command(
        &self,
        swarm: &mut libp2p::Swarm<BlockchainBehaviour>,
        blocks_topic: &IdentTopic,
        transactions_topic: &IdentTopic,
        cmd: NetworkCommand,
    ) {
        match cmd {
            NetworkCommand::BroadcastTransaction(tx) => {
                if let Ok(data) = serde_json::to_vec(&NetworkMessage::NewTransaction(tx)) {
                    if let Err(e) = swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(transactions_topic.clone(), data)
                    {
                        tracing::warn!("Failed to publish transaction: {}", e);
                    }
                }
            }
            NetworkCommand::BroadcastBlock(block) => {
                if let Ok(data) = serde_json::to_vec(&NetworkMessage::NewBlock(block)) {
                    if let Err(e) = swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(blocks_topic.clone(), data)
                    {
                        tracing::warn!("Failed to publish block: {}", e);
                    }
                }
            }
            NetworkCommand::RequestChain => {
                if let Ok(data) = serde_json::to_vec(&NetworkMessage::ChainRequest) {
                    if let Err(e) = swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(blocks_topic.clone(), data)
                    {
                        tracing::warn!("Failed to request chain: {}", e);
                    }
                }
            }
        }
    }

    async fn handle_swarm_event(
        &self,
        swarm: &mut libp2p::Swarm<BlockchainBehaviour>,
        event: SwarmEvent<BlockchainBehaviourEvent>,
    ) {
        match event {
            SwarmEvent::Behaviour(BlockchainBehaviourEvent::Gossipsub(
                gossipsub::Event::Message {
                    message, ..
                },
            )) => {
                if let Ok(net_msg) = serde_json::from_slice::<NetworkMessage>(&message.data) {
                    let event = match net_msg {
                        NetworkMessage::NewTransaction(tx) => {
                            Some(NetworkEvent::NewTransaction(tx))
                        }
                        NetworkMessage::NewBlock(block) => {
                            Some(NetworkEvent::NewBlock(block))
                        }
                        NetworkMessage::ChainRequest => {
                            let peer = message
                                .source
                                .map(|p| p.to_string())
                                .unwrap_or_default();
                            Some(NetworkEvent::ChainRequest { peer })
                        }
                        NetworkMessage::ChainResponse(chain) => {
                            Some(NetworkEvent::ChainResponse(chain))
                        }
                    };
                    if let Some(ev) = event {
                        let _ = self.event_tx.send(ev).await;
                    }
                }
            }
            SwarmEvent::Behaviour(BlockchainBehaviourEvent::Mdns(
                mdns::Event::Discovered(peers),
            )) => {
                for (peer_id, _addr) in peers {
                    tracing::info!("mDNS discovered peer: {}", peer_id);
                    swarm
                        .behaviour_mut()
                        .gossipsub
                        .add_explicit_peer(&peer_id);
                    let _ = self
                        .event_tx
                        .send(NetworkEvent::PeerConnected(peer_id.to_string()))
                        .await;
                }
            }
            SwarmEvent::Behaviour(BlockchainBehaviourEvent::Mdns(
                mdns::Event::Expired(peers),
            )) => {
                for (peer_id, _addr) in peers {
                    tracing::info!("mDNS peer expired: {}", peer_id);
                    swarm
                        .behaviour_mut()
                        .gossipsub
                        .remove_explicit_peer(&peer_id);
                    let _ = self
                        .event_tx
                        .send(NetworkEvent::PeerDisconnected(peer_id.to_string()))
                        .await;
                }
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                tracing::info!("Listening on {}", address);
            }
            _ => {}
        }
    }
}
