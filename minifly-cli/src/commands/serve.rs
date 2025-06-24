/// Platform management for starting the Minifly services
/// 
/// This module provides functionality to start the Minifly platform, which includes:
/// - API server for handling machine and app management
/// - LiteFS for distributed SQLite replication
/// - Health checks and service dependency management
/// - Process lifecycle management
/// - Auto-deployment when run in project directories
/// - Automatic port allocation to avoid conflicts
/// - File watching for hot reloading in dev mode
use anyhow::{Context, Result, bail};
use colored::*;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;
use tokio::sync::Mutex;
use std::sync::Arc;
use tracing::{info, warn, error};
use crate::client::ApiClient;
use crate::commands::dependencies;

// Global deployment mutex to prevent concurrent deployments
lazy_static::lazy_static! {
    static ref DEPLOYMENT_MUTEX: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
}

/// Handle the serve command to start the Minifly platform with dependency checks
/// 
/// # Arguments
/// * `daemon` - Run in background as daemon process
/// * `port` - Port number for the API server (default: 4280)
/// * `dev` - Enable development mode with enhanced logging, auto-deployment, and file watching
/// 
/// # Features
/// - **Auto-deployment**: Automatically detects and deploys projects with fly.toml
/// - **Port allocation**: Docker automatically assigns available ports to avoid conflicts
/// - **File watching**: In dev mode, automatically redeploys on file changes
/// - **Graceful shutdown**: Properly handles Ctrl+C with full cleanup
/// 
/// # Examples
/// ```
/// // Start in foreground
/// serve::handle(false, 4280, false).await?;
/// 
/// // Start as daemon in development mode
/// serve::handle(true, 4280, true).await?;
/// 
/// // Start with auto-deployment in a project directory
/// // cd examples/basic-app && minifly serve --dev
/// ```
pub async fn handle(daemon: bool, port: u16, dev: bool, config_path: Option<String>) -> Result<()> {
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
        
        // Check if we're in a project directory and auto-deploy
        if let Some(project_info) = detect_project_config(dev, config_path).await? {
            println!("\n{}", "📦 Project detected, auto-deploying...".cyan().bold());
            
            // Acquire deployment lock to prevent concurrent deployments
            let _lock = DEPLOYMENT_MUTEX.lock().await;
            
            match auto_deploy_current_project(port, &project_info, dev).await {
                Ok(app_url) => {
                    println!("{}", "✅ Application deployed successfully!".green().bold());
                    println!("🔗 Access your app at: {}", app_url.blue().bold());
                    
                    if dev {
                        // Setup development mode features
                        setup_dev_mode_for_project(port, &project_info).await?;
                    }
                }
                Err(e) => {
                    println!("{}", format!("⚠️  Auto-deployment failed: {}", e).yellow());
                    println!("{}", "Platform is still available for manual deployment".dimmed());
                }
            }
        }
        
        if !daemon {
            if dev {
                println!("\n{}", "🎯 Development mode active".green().bold());
                println!("{}", "Features enabled:".bold());
                println!("   • Auto-deployment on startup");
                println!("   • File watching for changes");
                println!("   • Enhanced logging");
                println!();
            }
            
            println!("{}", "Press Ctrl+C to stop the platform".dimmed());
            
            // Setup signal handlers for graceful shutdown
            tokio::signal::ctrl_c().await.context("Failed to listen for ctrl-c")?;
            println!("\n{}", "🛑 Shutting down Minifly platform...".yellow());
            
            // Graceful shutdown
            shutdown_platform(port).await?;
            
            // Force exit to ensure background tasks don't prevent termination
            std::process::exit(0);
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
    let url = format!("http://localhost:{}/v1/health", port);
    
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

/// Setup the API server database
async fn setup_api_database() -> Result<()> {
    // Ensure data directory exists
    tokio::fs::create_dir_all("data").await
        .context("Failed to create data directory")?;
    
    // Create database file if it doesn't exist
    let db_path = "data/minifly.db";
    if !tokio::fs::metadata(db_path).await.is_ok() {
        // Create empty database file
        tokio::fs::File::create(db_path).await
            .context("Failed to create database file")?;
        info!("Created database file: {}", db_path);
    }
    
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
    
    // Ensure database directory exists and set up database
    setup_api_database().await?;
    
    let mut cmd = Command::new("minifly-api");
    
    // Use absolute path for data directory
    let current_dir = std::env::current_dir()
        .context("Failed to get current directory")?;
    let data_dir = current_dir.join("data");
    
    cmd.env("MINIFLY_API_PORT", port.to_string())
        .env("MINIFLY_DATABASE_URL", format!("sqlite:{}/minifly.db", data_dir.display()))
        .env("MINIFLY_DATA_DIR", data_dir.to_string_lossy().to_string());
    
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
    match api_client.get("/v1/health/comprehensive").await {
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
    let api_url = format!("http://localhost:{}/v1/health", port);
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
    // First, stop all Docker containers directly to ensure cleanup
    println!("     → Stopping Docker containers...");
    let output = std::process::Command::new("docker")
        .args(&["ps", "-a", "--filter", "name=minifly-", "--format", "{{.ID}}"])
        .output();
        
    if let Ok(output) = output {
        let container_ids = String::from_utf8_lossy(&output.stdout);
        for container_id in container_ids.lines().filter(|s| !s.is_empty()) {
            let _ = std::process::Command::new("docker")
                .args(&["stop", container_id])
                .output();
            let _ = std::process::Command::new("docker")
                .args(&["rm", container_id])
                .output();
        }
    }
    
    // Now stop machines via API
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

/// Project information detected in current directory
#[derive(Debug, Clone)]
struct ProjectInfo {
    /// Path to fly.toml file
    fly_toml_path: std::path::PathBuf,
    /// App name from fly.toml
    app_name: String,
    /// Project type (rust, docker, etc.)
    project_type: ProjectType,
}

#[derive(Debug, Clone)]
enum ProjectType {
    Rust,
    Docker,
    Generic,
}

/// Detect if current directory contains a deployable project
/// 
/// # Arguments
/// * `dev` - Whether we're in development mode (prefers fly.dev.toml)
/// 
/// # Returns
/// * `Ok(Some(ProjectInfo))` - Project detected and deployable
/// * `Ok(None)` - No project detected
/// * `Err(...)` - Error during detection
async fn detect_project_config(dev: bool, config_path: Option<String>) -> Result<Option<ProjectInfo>> {
    let current_dir = std::env::current_dir()
        .context("Failed to get current directory")?;
    
    // If config path is specified, use it
    let fly_toml_path = if let Some(config) = config_path {
        std::path::PathBuf::from(config)
    } else if dev {
        // In dev mode, prefer fly.dev.toml
        let dev_path = current_dir.join("fly.dev.toml");
        if dev_path.exists() {
            dev_path
        } else {
            current_dir.join("fly.toml")
        }
    } else {
        current_dir.join("fly.toml")
    };
    
    // Check if fly.toml exists
    if !fly_toml_path.exists() {
        return Ok(None);
    }
    
    // Read and parse fly.toml to get app name
    let fly_toml_content = tokio::fs::read_to_string(&fly_toml_path).await
        .context("Failed to read fly.toml")?;
    
    let toml_value: toml::Value = toml::from_str(&fly_toml_content)
        .context("Failed to parse fly.toml")?;
    
    let app_name = toml_value.get("app")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown-app")
        .to_string();
    
    // Detect project type
    let project_type = if current_dir.join("Cargo.toml").exists() {
        ProjectType::Rust
    } else if current_dir.join("Dockerfile").exists() {
        ProjectType::Docker
    } else {
        ProjectType::Generic
    };
    
    Ok(Some(ProjectInfo {
        fly_toml_path,
        app_name,
        project_type,
    }))
}

/// Get the URL for a deployed app by querying the actual assigned port
/// 
/// Since Docker automatically assigns available ports to avoid conflicts,
/// this function queries the Docker container to find the actual port mapping.
/// 
/// # Arguments
/// * `api_client` - API client for making requests (currently unused, kept for future API integration)
/// * `app_name` - Name of the app
/// 
/// # Returns
/// * `Ok(String)` - URL where the app is accessible with the actual port
/// * `Err(...)` - Failed to get app URL
/// 
/// # Example
/// ```
/// let url = get_deployed_app_url(&api_client, "example-app").await?;
/// // Returns: "http://localhost:32768"
/// ```
async fn get_deployed_app_url(api_client: &ApiClient, app_name: &str) -> Result<String> {
    use std::process::Command;
    
    // Wait a bit for container to be fully started and port to be assigned
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    // Try multiple methods to get the port
    
    
    // Method 2: Get the container ID first, then get port
    let container_id_output = Command::new("docker")
        .args(&["ps", "-q", "--filter", &format!("name=minifly-{}", app_name), "--latest"])
        .output();
        
    if let Ok(output) = container_id_output {
        if output.status.success() {
            let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !container_id.is_empty() {
                // Now get the port for this specific container
                let port_output = Command::new("docker")
                    .args(&["port", &container_id])
                    .output();
                    
                if let Ok(output) = port_output {
                    if output.status.success() {
                        let ports = String::from_utf8_lossy(&output.stdout);
                        // Parse output like "8080/tcp -> 0.0.0.0:32768"
                        for line in ports.lines() {
                            if let Some(mapping) = line.split(" -> ").nth(1) {
                                if let Some(port) = mapping.split(':').last() {
                                    let port = port.trim();
                                    if !port.is_empty() && port.chars().all(|c| c.is_numeric()) {
                                        return Ok(format!("http://localhost:{}", port));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Method 3: Original method with better parsing
    let output = Command::new("docker")
        .args(&["ps", "--filter", &format!("name=minifly-{}", app_name), "--format", "{{.Ports}}"])
        .output()
        .context("Failed to run docker ps command")?;
    
    if output.status.success() {
        let ports_output = String::from_utf8_lossy(&output.stdout);
        
        // Parse various port formats Docker might output
        for line in ports_output.lines() {
            // Handle "0.0.0.0:32769->8080/tcp" format
            if line.contains("->") {
                if let Some(host_part) = line.split("->").next() {
                    // Extract port from "0.0.0.0:32769" or ":::32769" format
                    let port = host_part
                        .split(':')
                        .last()
                        .unwrap_or("")
                        .trim()
                        .split(',')  // Handle multiple port mappings
                        .next()
                        .unwrap_or("")
                        .trim();
                    
                    if !port.is_empty() && port.chars().all(|c| c.is_numeric()) {
                        return Ok(format!("http://localhost:{}", port));
                    }
                }
            }
        }
    }
    
    // If we still couldn't get it, provide helpful instructions
    Ok("http://localhost (run 'docker ps' to find the port)".to_string())
}

/// Auto-deploy the current project using detected configuration
/// 
/// # Arguments
/// * `port` - API server port
/// * `project_info` - Project information from detection
/// * `dev` - Whether in development mode
/// 
/// # Returns
/// * `Ok(String)` - URL where the app is accessible
/// * `Err(...)` - Deployment failed
async fn auto_deploy_current_project(port: u16, project_info: &ProjectInfo, _dev: bool) -> Result<String> {
    let config_file = project_info.fly_toml_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("fly.toml");
    println!("   📁 Project: {} ({}) [{}]", 
        project_info.app_name.green(), 
        format!("{:?}", project_info.project_type).dimmed(),
        config_file.yellow());
    
    // Create API client
    let config = crate::config::Config {
        api_url: format!("http://localhost:{}", port),
        token: None,
    };
    let api_client = ApiClient::new(&config)?;
    
    // Use the existing deploy command (quietly, without showing duplicate output)
    let fly_toml_path = project_info.fly_toml_path.to_string_lossy().to_string();
    let app_url = crate::commands::deploy::handle_quiet(&api_client, Some(fly_toml_path)).await?;
    
    Ok(app_url)
}

/// Setup development mode features for the project
/// 
/// # Arguments
/// * `port` - API server port
/// * `project_info` - Project information
async fn setup_dev_mode_for_project(port: u16, project_info: &ProjectInfo) -> Result<()> {
    println!("   🔧 Setting up development mode...");
    
    // Setup file watcher
    setup_project_file_watcher(project_info, port).await?;
    
    // TODO: Setup log streaming
    println!("   ✓ File watcher enabled");
    println!("   ✓ Development mode ready");
    
    Ok(())
}

/// Setup file watcher for the project
async fn setup_project_file_watcher(project_info: &ProjectInfo, port: u16) -> Result<()> {
    use notify::{Watcher, RecursiveMode, watcher};
    use std::sync::mpsc::channel;
    
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1))
        .context("Failed to create file watcher")?;
    
    let project_dir = project_info.fly_toml_path.parent()
        .context("Failed to get project directory")?;
    
    // Watch the project directory
    watcher.watch(project_dir, RecursiveMode::Recursive)
        .context("Failed to start watching directory")?;
    
    // Spawn task to handle file change events
    let project_info_clone = project_info.clone();
    tokio::spawn(async move {
        use notify::DebouncedEvent;
        
        loop {
            match rx.recv() {
                Ok(event) => {
                    match event {
                        DebouncedEvent::Write(path) | DebouncedEvent::Create(path) => {
                            if should_trigger_redeploy(&path) {
                                println!("\n{}", "🔄 File change detected, redeploying...".yellow());
                                
                                // Acquire deployment lock to prevent concurrent deployments
                                if let Ok(_lock) = DEPLOYMENT_MUTEX.try_lock() {
                                    if let Err(e) = redeploy_project(&project_info_clone, port).await {
                                        eprintln!("{}", format!("❌ Redeploy failed: {}", e).red());
                                    } else {
                                        println!("{}", "✅ Redeploy completed".green());
                                    }
                                } else {
                                    println!("{}", "⏳ Another deployment is in progress, skipping...".yellow());
                                }
                                
                                println!("{}", "👀 Watching for changes...".dimmed());
                            }
                        }
                        _ => {}
                    }
                }
                Err(_) => break,
            }
        }
    });
    
    // Keep the watcher alive (this is a bit of a hack, but necessary for notify)
    std::mem::forget(watcher);
    
    Ok(())
}

/// Determine if a file change should trigger redeployment
fn should_trigger_redeploy(path: &std::path::Path) -> bool {
    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
        match filename {
            "fly.toml" | "Dockerfile" | "litefs.yml" => true,
            name if name.ends_with(".rs") => true,
            name if name.ends_with(".toml") => true,
            _ => false,
        }
    } else {
        false
    }
}

/// Redeploy the project after file changes
async fn redeploy_project(project_info: &ProjectInfo, port: u16) -> Result<()> {
    let config = crate::config::Config {
        api_url: format!("http://localhost:{}", port),
        token: None,
    };
    let api_client = ApiClient::new(&config)?;
    
    let fly_toml_path = project_info.fly_toml_path.to_string_lossy().to_string();
    crate::commands::deploy::handle(&api_client, Some(fly_toml_path), None, false).await?;
    
    Ok(())
}

/// Determine the app URL from fly.toml configuration
async fn determine_app_url(fly_toml_path: &std::path::Path) -> Result<String> {
    let fly_toml_content = tokio::fs::read_to_string(fly_toml_path).await
        .context("Failed to read fly.toml")?;
    
    let toml_value: toml::Value = toml::from_str(&fly_toml_content)
        .context("Failed to parse fly.toml")?;
    
    // Look for services configuration to determine port
    if let Some(services) = toml_value.get("services").and_then(|s| s.as_array()) {
        if let Some(service) = services.first() {
            if let Some(ports) = service.get("ports").and_then(|p| p.as_array()) {
                if let Some(port_config) = ports.first() {
                    if let Some(port) = port_config.get("port").and_then(|p| p.as_integer()) {
                        return Ok(format!("http://localhost:{}", port));
                    }
                }
            }
        }
    }
    
    // Default to port 80
    Ok("http://localhost:80".to_string())
}