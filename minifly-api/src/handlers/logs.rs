/// Log streaming handlers for machine logs
/// 
/// This module provides endpoints for streaming real-time logs from machines,
/// including Docker container logs with proper formatting, region context,
/// and correlation tracking.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::time::Duration;
use tracing::{error, info, warn};
use crate::state::AppState;

/// Query parameters for log streaming
#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    /// Follow logs in real-time
    #[serde(default)]
    pub follow: bool,
    /// Number of lines to show from the end
    pub tail: Option<String>,
    /// Include timestamps in log output
    #[serde(default)]
    pub timestamps: bool,
    /// Filter logs by region
    pub region: Option<String>,
    /// Include log levels
    #[serde(default)]
    pub include_levels: bool,
}

/// Log entry structure for streaming
#[derive(Debug, Serialize)]
pub struct LogEntry {
    /// Timestamp of the log entry
    pub timestamp: String,
    /// Log level (info, warn, error, debug)
    pub level: String,
    /// Region where the log originated
    pub region: String,
    /// Machine ID that generated the log
    pub machine_id: String,
    /// App name
    pub app_name: String,
    /// Raw log message
    pub message: String,
    /// Stream type (stdout, stderr)
    pub stream: String,
    /// Correlation ID for request tracking
    pub correlation_id: Option<String>,
}

/// Stream logs from a machine
/// 
/// # Endpoint
/// GET /v1/apps/{app_name}/machines/{machine_id}/logs
/// 
/// # Query Parameters
/// - follow: bool - Follow logs in real-time
/// - tail: String - Number of lines to show from end (e.g., "100")
/// - timestamps: bool - Include timestamps
/// - region: String - Filter by region
/// - include_levels: bool - Parse and include log levels
/// 
/// # Examples
/// ```bash
/// # Get last 50 lines
/// curl "http://localhost:4280/v1/apps/my-app/machines/abc123/logs?tail=50"
/// 
/// # Follow logs in real-time
/// curl "http://localhost:4280/v1/apps/my-app/machines/abc123/logs?follow=true"
/// 
/// # Get logs with timestamps and levels
/// curl "http://localhost:4280/v1/apps/my-app/machines/abc123/logs?timestamps=true&include_levels=true"
/// ```
pub async fn stream_machine_logs(
    Path((app_name, machine_id)): Path<(String, String)>,
    Query(params): Query<LogsQuery>,
    State(state): State<AppState>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    info!(
        app.name = %app_name,
        machine.id = %machine_id,
        follow = params.follow,
        tail = ?params.tail,
        region = ?params.region,
        "Starting log stream request"
    );

    // Get the container ID for this machine
    let container_id = match state.docker.get_container_id_by_machine(&machine_id).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            warn!(
                machine.id = %machine_id,
                "No container found for machine"
            );
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            error!(
                machine.id = %machine_id,
                error = %e,
                "Failed to get container for machine"
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Start streaming logs from Docker
    let log_stream = match state.docker.stream_logs(
        &container_id,
        params.follow,
        params.tail,
        params.timestamps,
    ).await {
        Ok(stream) => stream,
        Err(e) => {
            error!(
                container.id = %container_id,
                machine.id = %machine_id,
                error = %e,
                "Failed to start log stream"
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let region_filter = params.region.clone();
    let include_levels = params.include_levels;
    let app_name_clone = app_name.clone();
    let machine_id_clone = machine_id.clone();

    // Transform Docker log stream into SSE events
    let event_stream = log_stream.map(move |log_result| {
        match log_result {
            Ok(log_output) => {
                let log_entry = process_log_output(
                    log_output,
                    &app_name_clone,
                    &machine_id_clone,
                    &region_filter,
                    include_levels,
                );
                
                match serde_json::to_string(&log_entry) {
                    Ok(json) => Ok(Event::default().data(json)),
                    Err(e) => {
                        error!(error = %e, "Failed to serialize log entry");
                        Ok(Event::default().data(format!(r#"{{"error": "Failed to serialize log: {}"}}"#, e)))
                    }
                }
            }
            Err(e) => {
                error!(error = %e, "Docker log stream error");
                Ok(Event::default().data(format!(r#"{{"error": "Log stream error: {}"}}"#, e)))
            }
        }
    });

    Ok(Sse::new(event_stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(10))
            .text("heartbeat"),
    ))
}

/// Process Docker log output into structured log entry
fn process_log_output(
    log_output: bollard::container::LogOutput,
    app_name: &str,
    machine_id: &str,
    _region_filter: &Option<String>,
    include_levels: bool,
) -> LogEntry {
    use bollard::container::LogOutput;
    
    let (stream_type, message_bytes) = match log_output {
        LogOutput::StdOut { message } => ("stdout", message),
        LogOutput::StdErr { message } => ("stderr", message),
        LogOutput::StdIn { message } => ("stdin", message),
        LogOutput::Console { message } => ("console", message),
    };
    
    let raw_message = String::from_utf8_lossy(&message_bytes);
    let cleaned_message = raw_message.trim_end_matches('\n').trim_end_matches('\r');
    
    // Extract timestamp if present (Docker format: 2024-06-22T10:30:00.123456789Z)
    let (timestamp, message) = if cleaned_message.len() > 30 && cleaned_message.chars().nth(30) == Some(' ') {
        let (ts, msg) = cleaned_message.split_at(30);
        (ts.to_string(), msg.trim_start().to_string())
    } else {
        (chrono::Utc::now().to_rfc3339(), cleaned_message.to_string())
    };
    
    // Parse log level if requested
    let level = if include_levels {
        extract_log_level(&message)
    } else {
        "info".to_string()
    };
    
    // Generate correlation ID for this log entry
    let correlation_id = Some(uuid::Uuid::new_v4().to_string());
    
    LogEntry {
        timestamp,
        level,
        region: "local".to_string(), // In real Fly.io this would be the actual region
        machine_id: machine_id.to_string(),
        app_name: app_name.to_string(),
        message,
        stream: stream_type.to_string(),
        correlation_id,
    }
}

/// Extract log level from message content
fn extract_log_level(message: &str) -> String {
    let lower_msg = message.to_lowercase();
    
    // Check for common log level patterns
    if lower_msg.contains("error") || lower_msg.contains("err") || lower_msg.contains("fail") {
        "error".to_string()
    } else if lower_msg.contains("warn") || lower_msg.contains("warning") {
        "warn".to_string()
    } else if lower_msg.contains("debug") || lower_msg.contains("dbg") {
        "debug".to_string()
    } else if lower_msg.contains("info") || lower_msg.contains("starting") || lower_msg.contains("listening") {
        "info".to_string()
    } else {
        "info".to_string()
    }
}

/// Get log summary for a machine
/// 
/// # Endpoint
/// GET /v1/apps/{app_name}/machines/{machine_id}/logs/summary
/// 
/// # Returns
/// Summary information about logs including line counts, recent activity, etc.
pub async fn get_logs_summary(
    Path((app_name, machine_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<axum::Json<LogsSummary>, StatusCode> {
    info!(
        app.name = %app_name,
        machine.id = %machine_id,
        "Getting logs summary"
    );

    // Get container ID for this machine
    let container_id = match state.docker.get_container_id_by_machine(&machine_id).await {
        Ok(Some(id)) => id,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Get a small sample of recent logs to analyze
    let mut recent_logs = Vec::new();
    if let Ok(log_stream) = state.docker.stream_logs(&container_id, false, Some("10".to_string()), true).await {
        let mut stream = log_stream.take(10);
        while let Some(log_result) = stream.next().await {
            if let Ok(log_output) = log_result {
                recent_logs.push(log_output);
            }
        }
    }

    let summary = LogsSummary {
        machine_id: machine_id.clone(),
        app_name: app_name.clone(),
        container_id,
        recent_log_count: recent_logs.len(),
        has_errors: recent_logs.iter().any(|log| {
            match log {
                bollard::container::LogOutput::StdErr { .. } => true,
                bollard::container::LogOutput::StdOut { message } => {
                    String::from_utf8_lossy(message).to_lowercase().contains("error")
                }
                _ => false,
            }
        }),
        last_activity: chrono::Utc::now().to_rfc3339(),
        region: "local".to_string(),
    };

    Ok(axum::Json(summary))
}

/// Log summary information
#[derive(Debug, Serialize)]
pub struct LogsSummary {
    pub machine_id: String,
    pub app_name: String,
    pub container_id: String,
    pub recent_log_count: usize,
    pub has_errors: bool,
    pub last_activity: String,
    pub region: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_log_level() {
        assert_eq!(extract_log_level("This is an error message"), "error");
        assert_eq!(extract_log_level("WARNING: something happened"), "warn");
        assert_eq!(extract_log_level("Debug info here"), "debug");
        assert_eq!(extract_log_level("Server starting on port 8080"), "info");
        assert_eq!(extract_log_level("Normal message"), "info");
    }

    #[test]
    fn test_log_entry_serialization() {
        let entry = LogEntry {
            timestamp: "2024-06-22T10:30:00Z".to_string(),
            level: "info".to_string(),
            region: "local".to_string(),
            machine_id: "abc123".to_string(),
            app_name: "test-app".to_string(),
            message: "Hello world".to_string(),
            stream: "stdout".to_string(),
            correlation_id: Some("123-456-789".to_string()),
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("Hello world"));
        assert!(json.contains("abc123"));
    }
}