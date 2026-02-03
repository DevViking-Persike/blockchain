mod config;

use clap::Parser;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;

use blockchain_api::routes::create_router;
use blockchain_api::state::AppState;
use blockchain_core::chain::Blockchain;
use blockchain_network::handler::{NetworkCommand, NetworkEvent};
use blockchain_network::node::NetworkNode;

use crate::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file (ignore if missing)
    dotenvy::dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Config::parse();

    tracing::info!(
        "Starting blockchain node (difficulty={}, reward={})",
        config.difficulty,
        config.mining_reward
    );

    // Create blockchain
    let blockchain = Blockchain::new(config.difficulty, config.mining_reward);

    // Create channels for network communication
    let (net_cmd_tx, net_cmd_rx) = mpsc::channel::<NetworkCommand>(256);
    let (net_event_tx, mut net_event_rx) = mpsc::channel::<NetworkEvent>(256);

    // Create app state with network sender
    // Map NetworkCommand from network crate to API crate's NetworkCommand
    let (api_cmd_tx, mut api_cmd_rx) =
        mpsc::channel::<blockchain_api::state::NetworkCommand>(256);
    let app_state = AppState::new(blockchain).with_network(api_cmd_tx);
    let shared_blockchain = app_state.blockchain.clone();
    let shared_peer_count = app_state.peer_count.clone();

    // Forward API commands to network commands
    let net_cmd_tx_clone = net_cmd_tx.clone();
    tokio::spawn(async move {
        while let Some(cmd) = api_cmd_rx.recv().await {
            let net_cmd = match cmd {
                blockchain_api::state::NetworkCommand::BroadcastTransaction(tx) => {
                    NetworkCommand::BroadcastTransaction(tx)
                }
                blockchain_api::state::NetworkCommand::BroadcastBlock(block) => {
                    NetworkCommand::BroadcastBlock(block)
                }
                blockchain_api::state::NetworkCommand::RequestChain => {
                    NetworkCommand::RequestChain
                }
            };
            let _ = net_cmd_tx_clone.send(net_cmd).await;
        }
    });

    // Start network node
    let network_node = NetworkNode::new(net_cmd_rx, net_event_tx, config.p2p_port);
    tokio::spawn(async move {
        if let Err(e) = network_node.run().await {
            tracing::error!("Network node error: {}", e);
        }
    });

    // Start API server
    let router = create_router(app_state);
    let api_addr = format!("0.0.0.0:{}", config.api_port);
    tracing::info!("API server starting on http://{}", api_addr);

    let listener = tokio::net::TcpListener::bind(&api_addr).await?;
    let api_handle = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, router).await {
            tracing::error!("API server error: {}", e);
        }
    });

    // Main event loop: process network events
    let event_loop = tokio::spawn(async move {
        while let Some(event) = net_event_rx.recv().await {
            match event {
                NetworkEvent::NewTransaction(tx) => {
                    tracing::info!("Received transaction from network: {}", tx.id);
                    let mut bc = shared_blockchain.lock().await;
                    if let Err(e) = bc.add_transaction(tx) {
                        tracing::warn!("Failed to add network transaction: {}", e);
                    }
                }
                NetworkEvent::NewBlock(block) => {
                    tracing::info!(
                        "Received block from network: index={}",
                        block.header.index
                    );
                    // For simplicity, request full chain sync
                    let _ = net_cmd_tx.send(NetworkCommand::RequestChain).await;
                }
                NetworkEvent::ChainRequest { peer } => {
                    tracing::info!("Chain requested by peer: {}", peer);
                    let bc = shared_blockchain.lock().await;
                    let chain = bc.chain().to_vec();
                    let msg = NetworkCommand::BroadcastBlock(
                        chain.last().unwrap().clone(),
                    );
                    let _ = net_cmd_tx.send(msg).await;
                }
                NetworkEvent::ChainResponse(chain) => {
                    tracing::info!(
                        "Received chain response: {} blocks",
                        chain.len()
                    );
                    let mut bc = shared_blockchain.lock().await;
                    if let Err(e) = bc.replace_chain(chain) {
                        tracing::debug!("Chain replacement rejected: {}", e);
                    }
                }
                NetworkEvent::PeerConnected(peer) => {
                    tracing::info!("Peer connected: {}", peer);
                    let mut count = shared_peer_count.lock().await;
                    *count += 1;
                }
                NetworkEvent::PeerDisconnected(peer) => {
                    tracing::info!("Peer disconnected: {}", peer);
                    let mut count = shared_peer_count.lock().await;
                    *count = count.saturating_sub(1);
                }
            }
        }
    });

    // Wait for API server (event loop runs indefinitely alongside it)
    tokio::select! {
        _ = api_handle => {
            tracing::info!("API server stopped");
        }
        _ = event_loop => {
            tracing::info!("Event loop stopped");
        }
    }

    Ok(())
}
