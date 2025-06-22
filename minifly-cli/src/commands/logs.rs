use anyhow::{Result, Context};
use colored::*;
use futures::StreamExt;
use serde::Deserialize;
use std::io::{self, Write};
use crate::client::ApiClient;

/// Log entry structure matching API server
#[derive(Debug, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    region: String,
    machine_id: String,
    app_name: String,
    message: String,
    stream: String,
    correlation_id: Option<String>,
}

/// Handle the logs command with real-time streaming
/// 
/// # Arguments
/// * `client` - API client for communicating with Minifly API
/// * `machine_id` - Machine ID to get logs from
/// * `follow` - Whether to follow log output (stream in real-time)
/// * `region` - Optional region filter for logs
/// 
/// # Examples
/// ```
/// // Get last 50 lines of logs
/// logs::handle(&client, "abc123", false, None).await?;
/// 
/// // Follow logs in real-time
/// logs::handle(&client, "abc123", true, None).await?;
/// 
/// // Filter by region
/// logs::handle(&client, "abc123", true, Some("sjc".to_string())).await?;
/// ```
pub async fn handle(client: &ApiClient, machine_id: &str, follow: bool, region: Option<String>) -> Result<()> {
    // First, get the app name for this machine
    let app_name = client.get_machine_app(machine_id).await
        .context("Failed to get app name for machine")?;
    
    if let Some(ref region_filter) = region {
        println!("Filtering logs for region: {}", region_filter.cyan());
    }
    
    if follow {
        println!("🔄 Streaming logs for machine {} (following)...", machine_id.yellow());
        println!("{}", "Press Ctrl+C to stop".dimmed());
    } else {
        println!("📄 Getting recent logs for machine {}...", machine_id.yellow());
    }
    
    // Get logs summary first
    if let Ok(summary) = get_logs_summary(client, &app_name, machine_id).await {
        println!("📊 Log Summary:");
        println!("   Region: {}", summary.region.blue());
        println!("   Recent logs: {} entries", summary.recent_log_count.to_string().green());
        if summary.has_errors {
            println!("   Status: {} (recent errors detected)", "⚠️  Warning".yellow());
        } else {
            println!("   Status: {}", "✅ Healthy".green());
        }
        println!();
    }
    
    // Start streaming logs
    stream_logs(client, &app_name, machine_id, follow, region).await
}

/// Stream logs from the API server
async fn stream_logs(
    client: &ApiClient, 
    app_name: &str,
    machine_id: &str, 
    follow: bool, 
    region: Option<String>
) -> Result<()> {
    let mut url = format!("/apps/{}/machines/{}/logs?timestamps=true&include_levels=true", app_name, machine_id);
    
    if follow {
        url.push_str("&follow=true");
    } else {
        url.push_str("&tail=100"); // Get last 100 lines
    }
    
    if let Some(region_filter) = region {
        url.push_str(&format!("&region={}", region_filter));
    }
    
    println!("🔗 Connecting to log stream...");
    
    // Create SSE client for streaming
    let response = client.get(&url).await
        .context("Failed to connect to log stream")?;
    
    if !response.status().is_success() {
        if response.status() == 404 {
            println!("{} Machine {} not found or no logs available", 
                "❌".red(), machine_id.yellow());
            return Ok(());
        } else {
            return Err(anyhow::anyhow!("Failed to get logs: HTTP {}", response.status()));
        }
    }
    
    println!("✅ Connected to log stream\n");
    
    // Process the SSE stream
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.context("Failed to read from log stream")?;
        let chunk_str = String::from_utf8_lossy(&chunk);
        buffer.push_str(&chunk_str);
        
        // Process complete lines
        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].trim().to_string();
            buffer.drain(..=line_end);
            
            if line.is_empty() || line.starts_with(':') {
                continue; // Skip empty lines and SSE comments
            }
            
            // Parse SSE data line
            if let Some(data) = line.strip_prefix("data: ") {
                if data == "heartbeat" {
                    continue; // Skip heartbeat messages
                }
                
                // Try to parse as log entry
                match serde_json::from_str::<LogEntry>(data) {
                    Ok(log_entry) => {
                        display_log_entry(&log_entry);
                    }
                    Err(_) => {
                        // Fallback for non-JSON data
                        println!("{}", data);
                    }
                }
            }
        }
        
        // Flush output
        io::stdout().flush().ok();
    }
    
    println!("\n📡 Log stream ended");
    Ok(())
}

/// Display a formatted log entry
fn display_log_entry(entry: &LogEntry) {
    let timestamp = parse_and_format_timestamp(&entry.timestamp);
    let region_badge = format!("[{}]", entry.region).blue().bold();
    let level_badge = format_log_level(&entry.level);
    let machine_id = entry.machine_id.get(..8).unwrap_or(&entry.machine_id).green();
    let stream_indicator = match entry.stream.as_str() {
        "stderr" => "⚠".red(),
        "stdout" => "→".blue(),
        _ => "•".white(),
    };
    
    // Show correlation ID for debugging if available
    let correlation = if let Some(ref id) = entry.correlation_id {
        format!(" [{}]", &id[..8]).dimmed()
    } else {
        "".normal()
    };
    
    println!("{} {} {} {} {} {}{}",
        region_badge,
        timestamp.dimmed(),
        level_badge,
        machine_id,
        stream_indicator,
        entry.message.white(),
        correlation
    );
}

/// Format log level with appropriate colors
fn format_log_level(level: &str) -> colored::ColoredString {
    match level.to_lowercase().as_str() {
        "error" => "ERROR".red().bold(),
        "warn" | "warning" => "WARN ".yellow().bold(),
        "debug" => "DEBUG".cyan(),
        "info" => "INFO ".green(),
        _ => level.normal(),
    }
}

/// Parse and format timestamp for display
fn parse_and_format_timestamp(timestamp: &str) -> String {
    // Try to parse ISO8601 timestamp
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
        dt.format("%H:%M:%S%.3f").to_string()
    } else {
        // Fallback to extracting time portion if already formatted
        if timestamp.len() >= 8 && timestamp.contains(':') {
            timestamp[..8].to_string()
        } else {
            chrono::Utc::now().format("%H:%M:%S").to_string()
        }
    }
}

/// Get logs summary for a machine
async fn get_logs_summary(
    client: &ApiClient, 
    app_name: &str, 
    machine_id: &str
) -> Result<LogsSummary> {
    let url = format!("/apps/{}/machines/{}/logs/summary", app_name, machine_id);
    let response = client.get(&url).await?;
    
    if response.status().is_success() {
        let summary: LogsSummary = response.json().await?;
        Ok(summary)
    } else {
        Err(anyhow::anyhow!("Failed to get logs summary: HTTP {}", response.status()))
    }
}

/// Log summary structure matching API server
#[derive(Debug, Deserialize)]
struct LogsSummary {
    machine_id: String,
    app_name: String,
    container_id: String,
    recent_log_count: usize,
    has_errors: bool,
    last_activity: String,
    region: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_format_timestamp() {
        let iso_timestamp = "2024-06-22T10:30:00.123Z";
        let formatted = parse_and_format_timestamp(iso_timestamp);
        assert!(formatted.contains("10:30:00"));
        
        let time_only = "10:30:00";
        let formatted2 = parse_and_format_timestamp(time_only);
        assert_eq!(formatted2, "10:30:00");
    }

    #[test]
    fn test_format_log_level() {
        assert!(format!("{}", format_log_level("error")).contains("ERROR"));
        assert!(format!("{}", format_log_level("warn")).contains("WARN"));
        assert!(format!("{}", format_log_level("info")).contains("INFO"));
        assert!(format!("{}", format_log_level("debug")).contains("DEBUG"));
    }
}