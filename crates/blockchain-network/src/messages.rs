use blockchain_core::block::Block;
use blockchain_core::transaction::Transaction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    NewTransaction(Transaction),
    NewBlock(Block),
    ChainRequest,
    ChainResponse(Vec<Block>),
}
