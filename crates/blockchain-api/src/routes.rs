use axum::routing::{get, post};
use axum::Router;

use crate::handlers;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Chain
        .route("/api/chain", get(handlers::get_chain))
        .route("/api/chain/valid", get(handlers::validate_chain))
        // Blocks
        .route("/api/blocks/mine", post(handlers::mine_block))
        .route("/api/blocks/:index", get(handlers::get_block))
        // Transactions
        .route("/api/transactions", post(handlers::create_transaction))
        .route(
            "/api/transactions/pending",
            get(handlers::get_pending_transactions),
        )
        // Wallet
        .route("/api/wallet/new", post(handlers::create_wallet))
        .route("/api/balance/:address", get(handlers::get_balance))
        // Contracts
        .route("/api/contracts/deploy", post(handlers::deploy_contract))
        .route("/api/contracts/call", post(handlers::call_contract))
        // Network
        .route("/api/peers", get(handlers::get_peers))
        .route("/api/node/info", get(handlers::node_info))
        .with_state(state)
}
