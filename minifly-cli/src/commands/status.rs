/// Enhanced status command with real-time information and region context
/// 
/// This module provides comprehensive status information including:
/// - Platform health and connectivity
/// - Service status with region information  
/// - Resource counts with breakdowns by region
/// - Docker and LiteFS status
/// - Recent activity and events
use anyhow::Result;
use colored::*;
use std::collections::HashMap;
use tabled::{Table, Tabled};
use serde::{Deserialize, Serialize};
use crate::client::ApiClient;

/// Status information for display
#[derive(Tabled)]
struct ServiceStatus {
    #[tabled(rename = "Service")]
    service: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Region")]
    region: String,
    #[tabled(rename = "Info")]
    info: String,
}

/// Machine summary for region display
#[derive(Tabled)]
struct RegionSummary {
    #[tabled(rename = "Region")]
    region: String,
    #[tabled(rename = "Apps")]
    apps: usize,
    #[tabled(rename = "Machines")]
    machines: usize,
    #[tabled(rename = "Running")]
    running: usize,
    #[tabled(rename = "Status")]
    status: String,
}

/// Health response structure matching API
#[derive(Debug, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
    uptime_seconds: u64,
    timestamp: String,
    services: HashMap<String, ServiceHealth>,
    summary: String,
}

#[derive(Debug, Deserialize)]
struct ServiceHealth {
    status: String,
    message: String,
    last_checked: String,
    response_time_ms: Option<u64>,
    details: HashMap<String, serde_json::Value>,
}

/// Handle the enhanced status command
/// 
/// Displays comprehensive platform status including:
/// - Service health with region information
/// - Resource counts by region
/// - Docker and dependency status
/// - Recent activity
pub async fn handle(client: &ApiClient) -> Result<()> {
    println!("{}", "🚀 Minifly Platform Status".bold().blue());
    println!("{}", "========================".blue());
    
    // 1. Check API connectivity and get region info
    let (api_status, api_region) = check_api_status(client).await;
    
    // 2. Get service status (try comprehensive health first, then fallback)
    let service_statuses = if api_status {
        match get_comprehensive_service_status(client, &api_region).await {
            Ok(statuses) => statuses,
            Err(_) => get_service_statuses(&api_region).await,
        }
    } else {
        get_service_statuses(&api_region).await
    };
    
    // 3. Show service status table
    println!("\n{}", "📊 Service Status".bold());
    let table = Table::new(service_statuses).to_string();
    println!("{}", table);
    
    // 4. Show resource summary by region
    if api_status {
        show_resource_summary(client).await?;
    }
    
    // 5. Show system information
    show_system_info().await;
    
    // 6. Show recent activity
    if api_status {
        show_recent_activity(client).await?;
    }
    
    Ok(())
}

/// Check API server status and get region information
async fn check_api_status(client: &ApiClient) -> (bool, String) {
    // Try the comprehensive health endpoint first
    match client.get("/health/comprehensive").await {
        Ok(response) => {
            let region = response
                .headers()
                .get("x-minifly-region")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("local")
                .to_string();
                
            (response.status().is_success(), region)
        }
        Err(_) => {
            // Fallback to simple health check
            match client.get("/health").await {
                Ok(response) => {
                    let region = response
                        .headers()
                        .get("x-minifly-region")
                        .and_then(|h| h.to_str().ok())
                        .unwrap_or("local")
                        .to_string();
                        
                    (response.status().is_success(), region)
                }
                Err(_) => (false, "unknown".to_string()),
            }
        }
    }
}

/// Get comprehensive service status from health endpoint
async fn get_comprehensive_service_status(client: &ApiClient, api_region: &str) -> Result<Vec<ServiceStatus>> {
    let response = client.get("/health/comprehensive").await?;
    let health: HealthResponse = response.json().await?;
    
    let mut statuses = Vec::new();
    
    // Add overall platform status
    let platform_status = match health.status.as_str() {
        "healthy" => "Healthy".green().to_string(),
        "degraded" => "Degraded".yellow().to_string(),
        "unhealthy" => "Unhealthy".red().to_string(),
        _ => health.status.yellow().to_string(),
    };
    
    statuses.push(ServiceStatus {
        service: "Platform".to_string(),
        status: platform_status,
        region: api_region.to_string(),
        info: format!("v{} ({}s uptime)", health.version, health.uptime_seconds),
    });
    
    // Add individual service statuses
    for (service_name, service_health) in health.services {
        let status_color = match service_health.status.as_str() {
            "healthy" => service_health.status.green().to_string(),
            "degraded" => service_health.status.yellow().to_string(),
            "unhealthy" => service_health.status.red().to_string(),
            _ => service_health.status.to_string(),
        };
        
        let info = if let Some(response_time) = service_health.response_time_ms {
            format!("{}ms - {}", response_time, service_health.message)
        } else {
            service_health.message
        };
        
        statuses.push(ServiceStatus {
            service: capitalize_service_name(&service_name),
            status: status_color,
            region: api_region.to_string(),
            info,
        });
    }
    
    Ok(statuses)
}

/// Capitalize service name for display
fn capitalize_service_name(name: &str) -> String {
    match name {
        "database" => "Database".to_string(),
        "docker" => "Docker".to_string(),
        "litefs" => "LiteFS".to_string(),
        "filesystem" => "File System".to_string(),
        _ => name.chars().enumerate().map(|(i, c)| {
            if i == 0 { c.to_uppercase().collect::<String>() } else { c.to_string() }
        }).collect::<String>(),
    }
}

/// Get status of all platform services
async fn get_service_statuses(api_region: &str) -> Vec<ServiceStatus> {
    let mut statuses = Vec::new();
    
    // API Server status
    let api_status = if api_region != "unknown" {
        "Running".green().to_string()
    } else {
        "Not reachable".red().to_string()
    };
    
    statuses.push(ServiceStatus {
        service: "API Server".to_string(),
        status: api_status,
        region: api_region.to_string(),
        info: "http://localhost:4280".to_string(),
    });
    
    // Docker status
    let docker_status = match std::process::Command::new("docker").arg("version").output() {
        Ok(output) if output.status.success() => "Running".green().to_string(),
        _ => "Not available".red().to_string(),
    };
    
    statuses.push(ServiceStatus {
        service: "Docker".to_string(),
        status: docker_status,
        region: api_region.to_string(),
        info: "Container runtime".to_string(),
    });
    
    // LiteFS status (check for running processes)
    let litefs_status = match std::process::Command::new("pgrep").args(&["-f", "litefs"]).output() {
        Ok(output) if !output.stdout.is_empty() => "Running".green().to_string(),
        _ => "Not running".yellow().to_string(),
    };
    
    statuses.push(ServiceStatus {
        service: "LiteFS".to_string(),
        status: litefs_status,
        region: api_region.to_string(),
        info: "Distributed SQLite".to_string(),
    });
    
    statuses
}

/// Show resource summary organized by region
async fn show_resource_summary(client: &ApiClient) -> Result<()> {
    println!("\n{}", "📦 Resources by Region".bold());
    
    let mut region_summaries: HashMap<String, RegionSummary> = HashMap::new();
    
    // Get apps and machines
    match client.list_apps().await {
        Ok(apps) => {
            for app in apps {
                match client.list_machines(&app.name).await {
                    Ok(machines) => {
                        for machine in machines {
                            let region = machine.region.clone();
                            let summary = region_summaries.entry(region.clone()).or_insert_with(|| {
                                RegionSummary {
                                    region: region.clone(),
                                    apps: 0,
                                    machines: 0,
                                    running: 0,
                                    status: "Active".green().to_string(),
                                }
                            });
                            
                            summary.apps = summary.apps.max(1); // Count this app
                            summary.machines += 1;
                            
                            // Check if machine is running (simplified check)
                            if format!("{:?}", machine.state).to_lowercase().contains("start") {
                                summary.running += 1;
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
            
            if region_summaries.is_empty() {
                println!("  No resources deployed yet");
            } else {
                let summaries: Vec<RegionSummary> = region_summaries.into_values().collect();
                let table = Table::new(summaries).to_string();
                println!("{}", table);
            }
        }
        Err(_) => {
            println!("  Unable to fetch resource information");
        }
    }
    
    Ok(())
}

/// Show system information
async fn show_system_info() {
    println!("\n{}", "⚙️  System Information".bold());
    println!("  Version: {}", env!("CARGO_PKG_VERSION").cyan());
    println!("  Platform: {}", std::env::consts::OS.cyan());
    println!("  Architecture: {}", std::env::consts::ARCH.cyan());
    
    // Check available disk space for data directory
    if let Ok(metadata) = std::fs::metadata("data") {
        if metadata.is_dir() {
            println!("  Data Directory: {}", "data/".cyan());
        }
    } else {
        println!("  Data Directory: {}", "Not created".yellow());
    }
}

/// Show recent activity from the platform
async fn show_recent_activity(client: &ApiClient) -> Result<()> {
    println!("\n{}", "📋 Recent Activity".bold());
    
    // Get recent machine events
    match client.list_apps().await {
        Ok(apps) => {
            let mut recent_events = Vec::new();
            
            for app in apps.iter().take(3) { // Limit to first 3 apps
                if let Ok(machines) = client.list_machines(&app.name).await {
                    for machine in machines.iter().take(2) { // Limit to 2 machines per app
                        if let Some(event) = machine.events.last() {
                            recent_events.push(format!(
                                "  {} {} {} {} in {}",
                                chrono::DateTime::from_timestamp(event.timestamp as i64 / 1000, 0)
                                    .unwrap_or_default()
                                    .format("%H:%M:%S")
                                    .to_string()
                                    .dimmed(),
                                event.event_type.cyan(),
                                machine.id.yellow(),
                                event.status.green(),
                                machine.region.blue()
                            ));
                        }
                    }
                }
            }
            
            if recent_events.is_empty() {
                println!("  No recent activity");
            } else {
                for event in recent_events.iter().take(5) {
                    println!("{}", event);
                }
            }
        }
        Err(_) => {
            println!("  Unable to fetch recent activity");
        }
    }
    
    Ok(())
}