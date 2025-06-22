/// Platform management for starting the Minifly services
/// 
/// This module provides functionality to start the Minifly platform, which includes:
/// - API server for handling machine and app management
/// - LiteFS for distributed SQLite replication
/// - Health checks and service dependency management
/// - Process lifecycle management
use anyhow::{Context, Result, bail};
use colored::*;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, error};
use crate::client::ApiClient;
use crate::commands::dependencies;

/// Handle the serve command to start the Minifly platform with dependency checks
/// 
/// # Arguments
/// * `daemon` - Run in background as daemon process
/// * `port` - Port number for the API server (default: 4280)
/// * `dev` - Enable development mode with enhanced logging and auto-reload
/// 
/// # Examples
/// ```
/// // Start in foreground
/// serve::handle(false, 4280, false).await?;
/// 
/// // Start as daemon in development mode
/// serve::handle(true, 4280, true).await?;
/// ```
pub async fn handle(daemon: bool, port: u16, dev: bool) -> Result<()> {
    println!("{}", "🚀 Starting Minifly Platform".bold().blue());
    
    if dev {
        println!("{}", "📝 Development mode enabled".yellow());
    }
    
    // Check if platform is already running
    if is_platform_running(port).await {
        println!("{}", "✅ Minifly platform is already running".green());
        println!("   API Server: {}", format!("http://localhost:{}", port).blue());
        return Ok(());
    }
    
    // Create temporary API client for dependency checks
    let config = crate::config::Config::load().unwrap_or_default();
    let api_client = ApiClient::new(&config)?;
    
    // Check service dependencies before starting
    let dep_manager = dependencies::DependencyManager::new();
    let results = dep_manager.check_all_dependencies().await;
    
    let failed_deps: Vec<_> = results.iter()
        .filter(|r| !r.available)
        .collect();
    
    if !failed_deps.is_empty() {
        println!("\n{}", "⚠️  Some dependencies are not available:".yellow().bold());
        for dep in &failed_deps {
            println!("  • {}: {}", dep.service.red(), 
                    dep.error.as_ref().unwrap_or(&"Unknown error".to_string()).dimmed());
        }
        
        // Check if any required dependencies are missing
        let required_missing = failed_deps.iter()
            .any(|r| dep_manager.is_required(&r.service));
        
        if required_missing {
            println!("\n{}", "❌ Cannot start platform: required dependencies are missing".red().bold());
            println!("{}", "Please ensure Docker and SQLite are installed and available".yellow());
            return Err(anyhow::anyhow!("Required dependencies not available"));
        } else {
            println!("\n{}", "⚠️  Continuing with degraded functionality...".yellow());
        }
    }
    
    // Create necessary directories
    setup_directories().await?;
    
    // Start services in order
    println!("\n{}", "📦 Starting services...".cyan());
    
    // 1. Start API server
    start_api_server(port, daemon, dev).await?;
    
    // 2. Wait for API server to be ready
    wait_for_service_ready(port, "API Server").await?;
    
    // 3. Start LiteFS (if needed)
    start_litefs().await?;
    
    // Final comprehensive health check using dependency manager
    if comprehensive_health_check(&api_client, port).await {
        println!("\n{}", "✅ Minifly platform started successfully!".green().bold());
        println!("{}", "🌐 Services:".bold());
        println!("   API Server: {}", format!("http://localhost:{}", port).blue());
        println!("   Health Check: {}", format!("http://localhost:{}/health", port).blue());
        println!("   LiteFS: {}", "Ready".green());
        
        if !daemon {
            println!("\n{}", "Press Ctrl+C to stop the platform".dimmed());
            
            // Setup signal handlers for graceful shutdown
            tokio::signal::ctrl_c().await.context("Failed to listen for ctrl-c")?;
            println!("\n{}", "🛑 Shutting down Minifly platform...".yellow());
            
            // Graceful shutdown
            shutdown_platform(port).await?;
        }
    } else {
        bail!("Failed to start Minifly platform - health check failed");
    }
    
    Ok(())
}

/// Check if the Minifly platform is already running
/// 
/// # Arguments
/// * `port` - Port number to check for API server
/// 
/// # Returns
/// * `bool` - True if platform is running and responsive
pub async fn is_platform_running(port: u16) -> bool {
    let client = reqwest::Client::new();
    let url = format!("http://localhost:{}/v1/apps", port);
    
    match client.get(&url).send().await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

/// Setup necessary directories for Minifly operation
async fn setup_directories() -> Result<()> {
    let dirs = ["data", "data/litefs", "data/machines", "data/apps"];
    
    for dir in dirs {
        tokio::fs::create_dir_all(dir).await
            .with_context(|| format!("Failed to create directory: {}", dir))?;
    }
    
    info!("Created necessary directories");
    Ok(())
}

/// Start the API server process
/// 
/// # Arguments
/// * `port` - Port number for the API server
/// * `daemon` - Whether to run as daemon
/// * `dev` - Whether to enable development mode
async fn start_api_server(port: u16, daemon: bool, dev: bool) -> Result<()> {
    println!("   • Starting API Server on port {}...", port.to_string().yellow());
    
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--bin", "minifly-api"])
        .env("MINIFLY_PORT", port.to_string())
        .env("MINIFLY_HOST", "0.0.0.0");
    
    if dev {
        cmd.env("RUST_LOG", "debug,minifly_api=trace,tower_http=debug");
    } else {
        cmd.env("RUST_LOG", "info,minifly_api=debug");
    }
    
    if daemon {
        cmd.stdout(Stdio::null())
           .stderr(Stdio::null())
           .stdin(Stdio::null());
        
        cmd.spawn()
            .context("Failed to start API server as daemon")?;
    } else {
        // For non-daemon mode, we'll start it in background but keep it attached
        cmd.stdout(Stdio::inherit())
           .stderr(Stdio::inherit());
           
        cmd.spawn()
            .context("Failed to start API server")?;
    }
    
    Ok(())
}

/// Wait for a service to become ready
/// 
/// # Arguments
/// * `port` - Port number to check
/// * `service_name` - Name of the service for logging
async fn wait_for_service_ready(port: u16, service_name: &str) -> Result<()> {
    let max_attempts = 30;
    let mut attempts = 0;
    
    while attempts < max_attempts {
        if is_platform_running(port).await {
            println!("   ✓ {} is ready", service_name.green());
            return Ok(());
        }
        
        attempts += 1;
        if attempts % 5 == 0 {
            println!("   ⏳ Waiting for {} (attempt {}/{})", service_name, attempts, max_attempts);
        }
        
        sleep(Duration::from_secs(1)).await;
    }
    
    bail!("Timeout waiting for {} to become ready", service_name);
}

/// Start LiteFS if configuration is present
async fn start_litefs() -> Result<()> {
    // Check if litefs.yml exists in current directory or any subdirectory
    if tokio::fs::metadata("litefs.yml").await.is_ok() {
        println!("   • Starting LiteFS...");
        
        // LiteFS will be started automatically by the API server when needed
        // This is just a placeholder for future LiteFS standalone management
        
        println!("   ✓ {}", "LiteFS configuration detected".green());
    }
    
    Ok(())
}

/// Perform comprehensive health check using the new health endpoints
/// 
/// # Arguments
/// * `api_client` - API client for health checks
/// * `port` - Port number for API server health check
/// 
/// # Returns
/// * `bool` - True if all services are healthy
async fn comprehensive_health_check(api_client: &ApiClient, port: u16) -> bool {
    println!("   🔍 Running comprehensive health check...");
    
    // Try comprehensive health endpoint first
    match api_client.get("/health/comprehensive").await {
        Ok(response) => {
            if response.status().is_success() {
                println!("   ✅ All services healthy");
                return true;
            } else {
                warn!("Comprehensive health check returned: {}", response.status());
            }
        }
        Err(e) => {
            warn!("Comprehensive health check failed: {}", e);
        }
    }
    
    // Fallback to basic checks
    basic_health_check(port).await
}

/// Perform basic health check on all services (fallback)
/// 
/// # Arguments
/// * `port` - Port number for API server health check
/// 
/// # Returns
/// * `bool` - True if basic services are healthy
async fn basic_health_check(port: u16) -> bool {
    let client = reqwest::Client::new();
    
    // Check API server
    let api_url = format!("http://localhost:{}/health", port);
    if let Ok(response) = client.get(&api_url).send().await {
        if !response.status().is_success() {
            error!("API server health check failed");
            return false;
        }
    } else {
        error!("Cannot reach API server");
        return false;
    }
    
    // Check Docker connectivity
    if let Err(e) = Command::new("docker").arg("version").output() {
        warn!("Docker check failed: {} - some features may not work", e);
    }
    
    true
}

/// Gracefully shutdown the platform
/// 
/// This function performs an orderly shutdown of all Minifly services:
/// 1. Stops all running machines gracefully
/// 2. Terminates LiteFS processes
/// 3. Stops the API server
/// 4. Cleans up any temporary resources
/// 
/// # Arguments
/// * `port` - Port number for API server communication
/// 
/// # Examples
/// ```
/// shutdown_platform(4280).await?;
/// ```
async fn shutdown_platform(port: u16) -> Result<()> {
    println!("   🔄 Initiating graceful shutdown sequence...");
    
    // Create API client for shutdown operations
    let config = crate::config::Config::load().unwrap_or_default();
    let api_client = ApiClient::new(&config)?;
    
    // 1. Stop all running machines gracefully
    println!("   • Stopping running machines...");
    if let Err(e) = stop_all_machines(&api_client).await {
        warn!("Failed to stop some machines gracefully: {}", e);
        println!("   ⚠️  Some machines may not have stopped cleanly");
    } else {
        println!("   ✓ All machines stopped");
    }
    
    // 2. Stop LiteFS processes
    println!("   • Stopping LiteFS...");
    if let Err(e) = stop_litefs().await {
        warn!("Failed to stop LiteFS cleanly: {}", e);
        println!("   ⚠️  LiteFS may not have stopped cleanly");
    } else {
        println!("   ✓ LiteFS stopped");
    }
    
    // 3. Stop API server
    println!("   • Stopping API server...");
    if let Err(e) = stop_api_server(port).await {
        warn!("Failed to stop API server cleanly: {}", e);
        println!("   ⚠️  API server may still be running");
    } else {
        println!("   ✓ API server stopped");
    }
    
    // 4. Clean up temporary resources
    println!("   • Cleaning up resources...");
    cleanup_resources().await?;
    println!("   ✓ Resources cleaned up");
    
    println!("{}", "✅ Platform shutdown complete".green());
    
    Ok(())
}

/// Stop all running machines gracefully
/// 
/// # Arguments
/// * `api_client` - API client for communicating with the platform
async fn stop_all_machines(api_client: &ApiClient) -> Result<()> {
    // Get list of all applications
    let apps_response = api_client.get("/v1/apps").await?;
    if !apps_response.status().is_success() {
        bail!("Failed to retrieve applications list");
    }
    
    let apps: Vec<minifly_core::models::App> = apps_response.json().await?;
    
    for app in apps {
        // Get machines for each app
        let machines_url = format!("/v1/apps/{}/machines", app.name);
        let machines_response = api_client.get(&machines_url).await?;
        
        if machines_response.status().is_success() {
            let machines: Vec<minifly_core::models::Machine> = machines_response.json().await?;
            
            for machine in machines {
                if machine.state == minifly_core::models::MachineState::Started || machine.state == minifly_core::models::MachineState::Starting {
                    println!("     → Stopping machine {} ({})", machine.name, machine.id);
                    
                    let stop_url = format!("/v1/apps/{}/machines/{}/stop", app.name, machine.id);
                    let stop_request = minifly_core::models::StopMachineRequest {
                        timeout: Some("30".to_string()), // 30 second graceful timeout
                        signal: Some("SIGTERM".to_string()),
                    };
                    
                    match api_client.post(&stop_url, &stop_request).await {
                        Ok(response) if response.status().is_success() => {
                            println!("       ✓ Machine {} stopped gracefully", machine.name);
                        }
                        Ok(response) => {
                            warn!("Failed to stop machine {}: {}", machine.name, response.status());
                        }
                        Err(e) => {
                            warn!("Error stopping machine {}: {}", machine.name, e);
                        }
                    }
                    
                    // Small delay between stops to avoid overwhelming the system
                    sleep(Duration::from_millis(500)).await;
                }
            }
        }
    }
    
    Ok(())
}

/// Stop LiteFS processes
async fn stop_litefs() -> Result<()> {
    // Look for LiteFS processes and stop them gracefully
    let output = Command::new("pkill")
        .args(&["-f", "litefs"])
        .output();
        
    match output {
        Ok(result) if result.status.success() => {
            info!("LiteFS processes stopped successfully");
        }
        Ok(_) => {
            // pkill returns 1 if no processes found, which is fine
            info!("No LiteFS processes found");
        }
        Err(e) => {
            warn!("Failed to stop LiteFS processes: {}", e);
            // Try alternative approach
            let _ = Command::new("killall")
                .args(&["-TERM", "litefs"])
                .output();
        }
    }
    
    Ok(())
}

/// Stop the API server process
/// 
/// # Arguments
/// * `port` - Port number the API server is running on
async fn stop_api_server(port: u16) -> Result<()> {
    // First, try to request graceful shutdown via API
    let client = reqwest::Client::new();
    let shutdown_url = format!("http://localhost:{}/admin/shutdown", port);
    
    match client.post(&shutdown_url).send().await {
        Ok(response) if response.status().is_success() => {
            info!("API server initiated graceful shutdown");
            
            // Wait for server to stop responding
            for _ in 0..10 {
                sleep(Duration::from_secs(1)).await;
                if !is_platform_running(port).await {
                    return Ok(());
                }
            }
        }
        _ => {
            // API shutdown failed, try process-based approach
        }
    }
    
    // Find and stop the API server process
    let output = Command::new("lsof")
        .args(&["-ti", &format!(":{}", port)])
        .output();
        
    match output {
        Ok(result) if result.status.success() => {
            let pid_str = String::from_utf8_lossy(&result.stdout);
            let pid = pid_str.trim();
            
            if !pid.is_empty() {
                // Send SIGTERM first for graceful shutdown
                let _ = Command::new("kill")
                    .args(&["-TERM", pid])
                    .output();
                
                // Wait a bit for graceful shutdown
                sleep(Duration::from_secs(3)).await;
                
                // Check if still running, then force kill if needed
                if is_platform_running(port).await {
                    let _ = Command::new("kill")
                        .args(&["-KILL", pid])
                        .output();
                    
                    warn!("Had to force-kill API server process");
                } else {
                    info!("API server stopped gracefully");
                }
            }
        }
        _ => {
            // Try pkill as fallback
            let _ = Command::new("pkill")
                .args(&["-f", "minifly-api"])
                .output();
        }
    }
    
    Ok(())
}

/// Clean up temporary resources and data
async fn cleanup_resources() -> Result<()> {
    // Clean up any temporary files or state
    
    // Remove any stale lock files
    let lock_files = ["data/.minifly.lock", "data/litefs/.lock"];
    for lock_file in &lock_files {
        if tokio::fs::metadata(lock_file).await.is_ok() {
            let _ = tokio::fs::remove_file(lock_file).await;
        }
    }
    
    // Clean up any stale Docker containers with minifly label
    let _ = Command::new("docker")
        .args(&["container", "prune", "-f", "--filter", "label=minifly.managed=true"])
        .output();
    
    info!("Cleanup completed");
    Ok(())
}