use std::sync::Arc;

use blockchain_core::chain::Blockchain;
use tokio::sync::Mutex;

/// Shared application state passed to all API handlers.
#[derive(Clone)]
pub struct AppState {
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub peer_count: Arc<Mutex<usize>>,
    /// Channel to send commands to the network layer (if connected).
    pub network_tx: Option<tokio::sync::mpsc::Sender<NetworkCommand>>,
}

/// Commands sent from the API to the network layer.
#[derive(Debug, Clone)]
pub enum NetworkCommand {
    BroadcastTransaction(blockchain_core::transaction::Transaction),
    BroadcastBlock(blockchain_core::block::Block),
    RequestChain,
}

impl AppState {
    pub fn new(blockchain: Blockchain) -> Self {
        Self {
            blockchain: Arc::new(Mutex::new(blockchain)),
            peer_count: Arc::new(Mutex::new(0)),
            network_tx: None,
        }
    }

    pub fn with_network(
        mut self,
        tx: tokio::sync::mpsc::Sender<NetworkCommand>,
    ) -> Self {
        self.network_tx = Some(tx);
        self
    }
}
