use axum::{extract::State, Json, http::StatusCode};
use serde_json::json;
use crate::error::Result;
use crate::state::AppState;
use crate::health::{health_handler, liveness_handler, readiness_handler, HealthResponse};

/// Simple health check endpoint for backwards compatibility
pub async fn health_check() -> Result<Json<serde_json::Value>> {
    Ok(Json(json!({
        "status": "ok",
        "service": "minifly-api",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    })))
}

/// Comprehensive health check with service dependency validation
pub async fn comprehensive_health(state: State<AppState>) -> std::result::Result<Json<HealthResponse>, StatusCode> {
    health_handler(state).await
}

/// Kubernetes/Docker liveness probe endpoint
pub async fn liveness() -> StatusCode {
    liveness_handler().await
}

/// Kubernetes/Docker readiness probe endpoint  
pub async fn readiness(state: State<AppState>) -> StatusCode {
    readiness_handler(state).await
}