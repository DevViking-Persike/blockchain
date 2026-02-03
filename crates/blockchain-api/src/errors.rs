use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    NotFound(String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            Self::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({ "error": message }));
        (status, body).into_response()
    }
}

impl From<blockchain_core::errors::CoreError> for ApiError {
    fn from(err: blockchain_core::errors::CoreError) -> Self {
        match &err {
            blockchain_core::errors::CoreError::AccountNotFound(_)
            | blockchain_core::errors::CoreError::ContractNotFound(_) => {
                Self::NotFound(err.to_string())
            }
            blockchain_core::errors::CoreError::InsufficientBalance { .. }
            | blockchain_core::errors::CoreError::InvalidTransaction(_)
            | blockchain_core::errors::CoreError::InvalidSignature(_)
            | blockchain_core::errors::CoreError::DuplicateTransaction(_) => {
                Self::BadRequest(err.to_string())
            }
            _ => Self::Internal(err.to_string()),
        }
    }
}

impl From<blockchain_vm::errors::VmError> for ApiError {
    fn from(err: blockchain_vm::errors::VmError) -> Self {
        Self::BadRequest(err.to_string())
    }
}
