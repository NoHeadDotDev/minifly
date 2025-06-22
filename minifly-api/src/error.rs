use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use minifly_core::Error as CoreError;
use serde_json::json;

pub struct ApiError(pub CoreError);

impl From<CoreError> for ApiError {
    fn from(err: CoreError) -> Self {
        ApiError(err)
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError(CoreError::Internal(err.to_string()))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self.0 {
            CoreError::MachineNotFound(ref id) => (StatusCode::NOT_FOUND, format!("Machine not found: {}", id)),
            CoreError::AppNotFound(ref name) => (StatusCode::NOT_FOUND, format!("App not found: {}", name)),
            CoreError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
            CoreError::InvalidConfiguration(ref msg) => (StatusCode::BAD_REQUEST, format!("Invalid configuration: {}", msg)),
            CoreError::BadRequest(ref msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            CoreError::AuthenticationFailed => (StatusCode::UNAUTHORIZED, "Authentication failed".to_string()),
            CoreError::LeaseConflict => (StatusCode::CONFLICT, "Lease conflict".to_string()),
            CoreError::InvalidLeaseNonce => (StatusCode::BAD_REQUEST, "Invalid lease nonce".to_string()),
            CoreError::DockerError(ref msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Docker error: {}", msg)),
            CoreError::DatabaseError(ref msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", msg)),
            CoreError::NetworkError(ref msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Network error: {}", msg)),
            CoreError::Internal(ref msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal error: {}", msg)),
            CoreError::LiteFSError(ref msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("LiteFS error: {}", msg)),
            CoreError::Anyhow(ref err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal error: {}", err)),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, ApiError>;