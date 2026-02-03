use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Behaviour error: {0}")]
    Behaviour(String),

    #[error("Channel error: {0}")]
    Channel(String),
}

pub type NetworkResult<T> = Result<T, NetworkError>;
