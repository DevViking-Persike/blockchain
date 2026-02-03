use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub signature: Option<String>,
    pub public_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeployContractRequest {
    pub sender: String,
    pub source_code: String,
}

#[derive(Debug, Deserialize)]
pub struct CallContractRequest {
    pub sender: String,
    pub contract_address: String,
    pub call_data: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MineResponse {
    pub block_index: u64,
    pub block_hash: String,
    pub transactions_count: usize,
}

#[derive(Debug, Serialize)]
pub struct ContractDeployResponse {
    pub address: String,
}

#[derive(Debug, Serialize)]
pub struct ContractCallResponse {
    pub logs: Vec<i64>,
    pub result: Option<i64>,
    pub steps_used: u64,
}

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub address: String,
    pub balance: u64,
}

#[derive(Debug, Serialize)]
pub struct ChainValidResponse {
    pub valid: bool,
    pub length: u64,
}

#[derive(Debug, Serialize)]
pub struct NodeInfoResponse {
    pub chain_length: u64,
    pub difficulty: u32,
    pub mining_reward: u64,
    pub pending_transactions: usize,
    pub peer_count: usize,
}
