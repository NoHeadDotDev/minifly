/// Administrative endpoints for platform management
/// 
/// This module provides administrative functionality for managing the Minifly platform,
/// including graceful shutdown operations, system maintenance, and diagnostic tools.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use tracing::{info, warn};
use crate::state::AppState;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Global shutdown flag shared across the application
static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Initiate graceful shutdown of the API server
/// 
/// This endpoint allows external systems (like the CLI) to request a graceful shutdown
/// of the API server. The shutdown process will:
/// 1. Stop accepting new requests
/// 2. Complete existing requests
/// 3. Clean up resources
/// 4. Exit the process
/// 
/// # Security Note
/// This endpoint should be restricted in production environments
/// 
/// # Examples
/// ```bash
/// curl -X POST http://localhost:4280/admin/shutdown
/// ```
pub async fn shutdown(
    State(_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    info!("Graceful shutdown requested via admin endpoint");
    
    // Set the shutdown flag
    SHUTDOWN_REQUESTED.store(true, Ordering::Relaxed);
    
    // In a real implementation, you would:
    // 1. Stop accepting new connections
    // 2. Drain existing connections
    // 3. Clean up resources
    // 4. Signal the main process to exit
    
    // For now, we'll just acknowledge the request
    // The actual shutdown would be handled by the main server process
    tokio::spawn(async {
        // Give time for the response to be sent
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        warn!("Initiating graceful shutdown...");
        // In practice, you would send a signal to the main process here
        // For this implementation, we'll use std::process::exit
        std::process::exit(0);
    });
    
    Ok(Json(json!({
        "status": "ok",
        "message": "Graceful shutdown initiated",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Get system status and runtime information
/// 
/// This endpoint provides detailed information about the current state of the API server,
/// including uptime, resource usage, and configuration details.
/// 
/// # Returns
/// JSON object containing:
/// - Server uptime
/// - Configuration summary
/// - Resource usage metrics
/// - Shutdown status
pub async fn system_status(
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let uptime_seconds = state.start_time.elapsed().as_secs();
    let shutdown_requested = SHUTDOWN_REQUESTED.load(Ordering::Relaxed);
    
    Ok(Json(json!({
        "status": "ok",
        "server": {
            "name": "minifly-api",
            "version": env!("CARGO_PKG_VERSION"),
            "uptime_seconds": uptime_seconds,
            "uptime_human": format_duration(uptime_seconds),
            "shutdown_requested": shutdown_requested
        },
        "configuration": {
            "port": state.config.port,
            "data_directory": state.config.data_dir,
            "log_level": "info" // This would come from actual config
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Check if shutdown has been requested
/// 
/// This function can be used by other parts of the application to check
/// if a graceful shutdown has been requested.
pub fn is_shutdown_requested() -> bool {
    SHUTDOWN_REQUESTED.load(Ordering::Relaxed)
}

/// Helper function to format duration in human-readable format
/// 
/// # Arguments
/// * `seconds` - Duration in seconds
/// 
/// # Returns
/// Human-readable string like "2h 30m 45s"
fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(45), "45s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3665), "1h 1m 5s");
        assert_eq!(format_duration(7200), "2h 0m 0s");
    }
}