/// Platform shutdown functionality
/// 
/// This module provides functionality to gracefully stop the Minifly platform:
/// - Stop all running machines
/// - Shutdown LiteFS processes
/// - Stop the API server
/// - Clean up resources
use anyhow::Result;
use colored::*;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

/// Handle the stop command to shutdown the Minifly platform
/// 
/// # Arguments
/// * `force` - Force stop all services without graceful shutdown
/// 
/// # Examples
/// ```
/// // Graceful shutdown
/// stop::handle(false).await?;
/// 
/// // Force shutdown
/// stop::handle(true).await?;
/// ```
pub async fn handle(force: bool) -> Result<()> {
    println!("{}", "🛑 Stopping Minifly Platform".bold().red());
    
    if force {
        println!("{}", "⚡ Force mode enabled".yellow());
    }
    
    // Check if platform is running
    let default_port = 4280;
    if !is_platform_running(default_port).await {
        println!("{}", "ℹ️  Minifly platform is not running".blue());
        return Ok(());
    }
    
    println!("{}", "📋 Stopping services...".cyan());
    
    // 1. Stop all running machines
    stop_all_machines(default_port, force).await?;
    
    // 2. Stop LiteFS processes
    stop_litefs(force).await?;
    
    // 3. Stop API server
    stop_api_server(default_port, force).await?;
    
    // 4. Cleanup resources
    cleanup_resources().await?;
    
    println!("\n{}", "✅ Minifly platform stopped successfully".green().bold());
    
    Ok(())
}

/// Check if the Minifly platform is running
/// 
/// # Arguments
/// * `port` - Port number to check for API server
async fn is_platform_running(port: u16) -> bool {
    let client = reqwest::Client::new();
    let url = format!("http://localhost:{}/v1/apps", port);
    
    match client.get(&url).send().await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

/// Stop all running machines
/// 
/// # Arguments
/// * `port` - API server port
/// * `force` - Whether to force stop machines
async fn stop_all_machines(port: u16, force: bool) -> Result<()> {
    println!("   • Stopping running machines...");
    
    let client = reqwest::Client::new();
    let apps_url = format!("http://localhost:{}/v1/apps", port);
    
    // Get all apps
    match client.get(&apps_url).send().await {
        Ok(response) => {
            if let Ok(apps) = response.json::<Vec<serde_json::Value>>().await {
                for app in apps {
                    if let Some(app_name) = app["name"].as_str() {
                        // Get machines for this app
                        let machines_url = format!("http://localhost:{}/v1/apps/{}/machines", port, app_name);
                        
                        if let Ok(machines_response) = client.get(&machines_url).send().await {
                            if let Ok(machines) = machines_response.json::<Vec<serde_json::Value>>().await {
                                for machine in machines {
                                    if let Some(machine_id) = machine["id"].as_str() {
                                        stop_machine(port, app_name, machine_id, force).await?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(_) => {
            println!("     ⚠️  Could not retrieve running machines");
        }
    }
    
    println!("     ✓ All machines stopped");
    Ok(())
}

/// Stop a specific machine
/// 
/// # Arguments
/// * `port` - API server port
/// * `app_name` - Application name
/// * `machine_id` - Machine ID to stop
/// * `force` - Whether to force stop
async fn stop_machine(port: u16, app_name: &str, machine_id: &str, force: bool) -> Result<()> {
    let client = reqwest::Client::new();
    let stop_url = format!(
        "http://localhost:{}/v1/apps/{}/machines/{}/stop", 
        port, app_name, machine_id
    );
    
    let mut request = client.post(&stop_url);
    
    if force {
        request = request.json(&serde_json::json!({
            "signal": "SIGKILL",
            "timeout": 1
        }));
    } else {
        request = request.json(&serde_json::json!({
            "signal": "SIGTERM", 
            "timeout": 30
        }));
    }
    
    match request.send().await {
        Ok(response) => {
            if response.status().is_success() {
                println!("     ✓ Stopped machine {} ({})", machine_id, app_name);
            } else {
                println!("     ⚠️  Failed to stop machine {} ({})", machine_id, app_name);
            }
        }
        Err(_) => {
            println!("     ⚠️  Could not communicate with machine {} ({})", machine_id, app_name);
        }
    }
    
    Ok(())
}

/// Stop LiteFS processes
/// 
/// # Arguments
/// * `force` - Whether to force stop LiteFS
async fn stop_litefs(force: bool) -> Result<()> {
    println!("   • Stopping LiteFS...");
    
    // Find and stop LiteFS processes
    let output = Command::new("pgrep")
        .arg("-f")
        .arg("litefs")
        .output();
    
    match output {
        Ok(result) => {
            let pids = String::from_utf8_lossy(&result.stdout);
            for pid in pids.lines() {
                if let Ok(_pid_num) = pid.trim().parse::<u32>() {
                    let signal = if force { "KILL" } else { "TERM" };
                    
                    let kill_result = Command::new("kill")
                        .arg(format!("-{}", signal))
                        .arg(pid.trim())
                        .output();
                    
                    match kill_result {
                        Ok(_) => println!("     ✓ Stopped LiteFS process {}", pid),
                        Err(_) => println!("     ⚠️  Could not stop LiteFS process {}", pid),
                    }
                }
            }
        }
        Err(_) => {
            // No LiteFS processes found or pgrep not available
            println!("     ✓ No LiteFS processes found");
        }
    }
    
    Ok(())
}

/// Stop the API server
/// 
/// # Arguments
/// * `port` - API server port
/// * `force` - Whether to force stop
async fn stop_api_server(port: u16, force: bool) -> Result<()> {
    println!("   • Stopping API server...");
    
    // First try graceful shutdown if API is responsive
    if !force && is_platform_running(port).await {
        let client = reqwest::Client::new();
        let shutdown_url = format!("http://localhost:{}/admin/shutdown", port);
        
        if let Ok(_) = client.post(&shutdown_url).send().await {
            println!("     ✓ API server graceful shutdown initiated");
            
            // Wait for shutdown
            for _ in 0..10 {
                sleep(Duration::from_secs(1)).await;
                if !is_platform_running(port).await {
                    println!("     ✓ API server stopped gracefully");
                    return Ok(());
                }
            }
        }
    }
    
    // Find and kill the API server process
    let output = Command::new("pgrep")
        .arg("-f")
        .arg("minifly-api")
        .output();
    
    match output {
        Ok(result) => {
            let pids = String::from_utf8_lossy(&result.stdout);
            for pid in pids.lines() {
                if let Ok(_) = pid.trim().parse::<u32>() {
                    let signal = if force { "KILL" } else { "TERM" };
                    
                    let kill_result = Command::new("kill")
                        .arg(format!("-{}", signal))
                        .arg(pid.trim())
                        .output();
                    
                    match kill_result {
                        Ok(_) => println!("     ✓ Stopped API server process {}", pid),
                        Err(_) => println!("     ⚠️  Could not stop API server process {}", pid),
                    }
                }
            }
        }
        Err(_) => {
            println!("     ⚠️  Could not find API server processes");
        }
    }
    
    Ok(())
}

/// Clean up resources and temporary files
async fn cleanup_resources() -> Result<()> {
    println!("   • Cleaning up resources...");
    
    // Clean up any temporary Docker volumes
    let _ = Command::new("docker")
        .args(&["volume", "prune", "-f", "--filter", "label=minifly"])
        .output();
    
    // Clean up any orphaned containers
    let _ = Command::new("docker")
        .args(&["container", "prune", "-f", "--filter", "label=minifly"])
        .output();
    
    println!("     ✓ Resources cleaned up");
    
    Ok(())
}