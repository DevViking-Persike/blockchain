use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Invalid block: {0}")]
    InvalidBlock(String),

    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("Invalid chain: {0}")]
    InvalidChain(String),

    #[error("Insufficient balance: account {account} has {balance}, needs {required}")]
    InsufficientBalance {
        account: String,
        balance: u64,
        required: u64,
    },

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Duplicate transaction: {0}")]
    DuplicateTransaction(String),

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Contract not found: {0}")]
    ContractNotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Mining error: {0}")]
    MiningError(String),
}

pub type CoreResult<T> = Result<T, CoreError>;
