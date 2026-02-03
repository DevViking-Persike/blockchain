use axum::extract::{Path, State};
use axum::Json;

use blockchain_core::transaction::Transaction;
use blockchain_core::wallet::Wallet;
use blockchain_vm::compiler;
use blockchain_vm::contract::ContractExecutor;

use crate::errors::ApiError;
use crate::models::*;
use crate::state::{AppState, NetworkCommand};

// --- Chain ---

pub async fn get_chain(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let bc = state.blockchain.lock().await;
    Ok(Json(serde_json::to_value(bc.chain()).map_err(|e| {
        ApiError::Internal(e.to_string())
    })?))
}

pub async fn validate_chain(
    State(state): State<AppState>,
) -> Json<ChainValidResponse> {
    let bc = state.blockchain.lock().await;
    Json(ChainValidResponse {
        valid: bc.is_chain_valid(),
        length: bc.height(),
    })
}

// --- Blocks ---

pub async fn mine_block(
    State(state): State<AppState>,
) -> Result<Json<MineResponse>, ApiError> {
    let mut bc = state.blockchain.lock().await;
    // Use a default miner address; in production this would come from config
    let block = bc
        .mine_pending("miner-node")
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let response = MineResponse {
        block_index: block.header.index,
        block_hash: block.hash.clone(),
        transactions_count: block.transactions.len(),
    };

    // Broadcast new block to network
    if let Some(tx) = &state.network_tx {
        let _ = tx.send(NetworkCommand::BroadcastBlock(block)).await;
    }

    Ok(Json(response))
}

pub async fn get_block(
    State(state): State<AppState>,
    Path(index): Path<u64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let bc = state.blockchain.lock().await;
    let block = bc
        .get_block(index)
        .ok_or_else(|| ApiError::NotFound(format!("Block {} not found", index)))?;
    Ok(Json(
        serde_json::to_value(block).map_err(|e| ApiError::Internal(e.to_string()))?,
    ))
}

// --- Transactions ---

pub async fn create_transaction(
    State(state): State<AppState>,
    Json(req): Json<CreateTransactionRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut tx = Transaction::new_transfer(req.sender, req.recipient, req.amount);

    // Apply signature if provided
    if let (Some(sig_hex), Some(pk_hex)) = (req.signature, req.public_key) {
        tx.signature = Some(
            hex::decode(&sig_hex)
                .map_err(|e| ApiError::BadRequest(format!("Invalid signature hex: {}", e)))?,
        );
        tx.public_key = Some(
            hex::decode(&pk_hex)
                .map_err(|e| ApiError::BadRequest(format!("Invalid public key hex: {}", e)))?,
        );
    }

    let mut bc = state.blockchain.lock().await;
    bc.add_transaction(tx.clone())?;

    // Broadcast to network
    if let Some(net_tx) = &state.network_tx {
        let _ = net_tx
            .send(NetworkCommand::BroadcastTransaction(tx.clone()))
            .await;
    }

    Ok(Json(
        serde_json::to_value(&tx).map_err(|e| ApiError::Internal(e.to_string()))?,
    ))
}

pub async fn get_pending_transactions(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let bc = state.blockchain.lock().await;
    Ok(Json(
        serde_json::to_value(bc.pending_transactions())
            .map_err(|e| ApiError::Internal(e.to_string()))?,
    ))
}

// --- Wallet ---

pub async fn create_wallet() -> Json<serde_json::Value> {
    let wallet = Wallet::new();
    Json(serde_json::to_value(wallet.info()).unwrap())
}

pub async fn get_balance(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Json<BalanceResponse> {
    let bc = state.blockchain.lock().await;
    let balance = bc.state().get_balance(&address);
    Json(BalanceResponse { address, balance })
}

// --- Contracts ---

pub async fn deploy_contract(
    State(state): State<AppState>,
    Json(req): Json<DeployContractRequest>,
) -> Result<Json<ContractDeployResponse>, ApiError> {
    let bytecode = compiler::compile(&req.source_code)?;
    let mut bc = state.blockchain.lock().await;
    let address = ContractExecutor::deploy(bc.state_mut(), &req.sender, bytecode)?;
    Ok(Json(ContractDeployResponse { address }))
}

pub async fn call_contract(
    State(state): State<AppState>,
    Json(req): Json<CallContractRequest>,
) -> Result<Json<ContractCallResponse>, ApiError> {
    let call_data = req.call_data.unwrap_or_default().into_bytes();
    let mut bc = state.blockchain.lock().await;
    let result =
        ContractExecutor::call(bc.state_mut(), &req.contract_address, &call_data)?;
    Ok(Json(ContractCallResponse {
        logs: result.logs,
        result: result.stack_top,
        steps_used: result.steps_used,
    }))
}

// --- Peers & Node ---

pub async fn get_peers(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let count = *state.peer_count.lock().await;
    Json(serde_json::json!({ "peer_count": count }))
}

pub async fn node_info(
    State(state): State<AppState>,
) -> Json<NodeInfoResponse> {
    let bc = state.blockchain.lock().await;
    let peer_count = *state.peer_count.lock().await;
    Json(NodeInfoResponse {
        chain_length: bc.height(),
        difficulty: bc.difficulty(),
        mining_reward: bc.mining_reward(),
        pending_transactions: bc.pending_transactions().len(),
        peer_count,
    })
}
