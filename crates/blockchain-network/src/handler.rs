use blockchain_core::block::Block;
use blockchain_core::transaction::Transaction;

/// Commands sent from the application to the network layer.
#[derive(Debug, Clone)]
pub enum NetworkCommand {
    BroadcastTransaction(Transaction),
    BroadcastBlock(Block),
    RequestChain,
}

/// Events emitted from the network layer to the application.
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    NewTransaction(Transaction),
    NewBlock(Block),
    ChainRequest { peer: String },
    ChainResponse(Vec<Block>),
    PeerConnected(String),
    PeerDisconnected(String),
}
