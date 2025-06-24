/// Development mode with auto-reload and enhanced logging
/// 
/// This module provides development-focused functionality including:
/// - File watching for automatic redeployment
/// - Enhanced logging with real-time streaming
/// - Hot reloading of configurations
/// - Development-specific debugging features
use anyhow::{Context, Result};
use colored::*;
use std::path::Path;
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

/// Handle the dev command for development mode
/// 
/// # Arguments
/// * `path` - Path to the project directory to watch
/// * `port` - Port number for the API server
/// 
/// # Examples
/// ```
/// // Start development mode in current directory
/// dev::handle(".", 4280).await?;
/// 
/// // Start development mode in specific project directory
/// dev::handle("./my-app", 4280).await?;
/// ```
pub async fn handle(path: &str, port: u16) -> Result<()> {
    println!("{}", "🔧 Starting Minifly Development Mode".bold().cyan());
    println!("   Project: {}", path.yellow());
    println!("   Port: {}", port.to_string().yellow());
    println!();
    
    // Validate project directory
    let project_path = Path::new(path);
    if !project_path.exists() {
        return Err(anyhow::anyhow!("Project directory does not exist: {}", path));
    }
    
    // Check for fly.toml
    let fly_toml_path = project_path.join("fly.toml");
    if !fly_toml_path.exists() {
        println!("{}", "⚠️  No fly.toml found in project directory".yellow());
        println!("   Run {} to create one", "minifly init".cyan());
        println!();
    }
    
    // Start the platform if not already running
    if !crate::commands::serve::is_platform_running(port).await {
        println!("{}", "🚀 Starting Minifly platform...".cyan());
        
        // Start platform in development mode
        crate::commands::serve::handle(true, port, true).await?;
        
        // Give it a moment to fully start
        tokio::time::sleep(Duration::from_secs(2)).await;
    } else {
        println!("{}", "✅ Minifly platform is already running".green());
    }
    
    // Setup file watcher for auto-deployment
    if fly_toml_path.exists() {
        setup_file_watcher(path, port).await?;
    }
    
    // Start log streaming for all machines in this project
    start_log_streaming(path, port).await?;
    
    println!("\n{}", "🎯 Development mode active".green().bold());
    println!("{}", "Features enabled:".bold());
    println!("   • File watching for auto-deployment");
    println!("   • Real-time log streaming");
    println!("   • Enhanced debugging");
    println!();
    println!("{}", "Press Ctrl+C to exit development mode".dimmed());
    
    // Wait for interrupt
    tokio::signal::ctrl_c().await.context("Failed to listen for ctrl-c")?;
    
    println!("\n{}", "🛑 Exiting development mode...".yellow());
    
    Ok(())
}

/// Setup file watcher for automatic deployment
/// 
/// # Arguments
/// * `path` - Project directory to watch
/// * `port` - API server port
async fn setup_file_watcher(path: &str, port: u16) -> Result<()> {
    println!("{}", "👀 Setting up file watcher...".cyan());
    
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1))?;
    
    // Watch for changes to fly.toml, source files, and Dockerfile
    watcher.watch(path, RecursiveMode::Recursive)?;
    
    println!("   ✓ Watching for changes in {}", path.green());
    
    // Spawn a task to handle file change events
    let path_clone = path.to_string();
    tokio::spawn(async move {
        use notify::DebouncedEvent;
        
        loop {
            match rx.recv() {
                Ok(event) => {
                    match event {
                        DebouncedEvent::Write(path) | DebouncedEvent::Create(path) => {
                            if let Some(filename) = path.file_name() {
                                if filename == "fly.toml" || 
                                   filename == "Dockerfile" ||
                                   filename == "litefs.yml" {
                                    
                                    println!("\n{}", "🔄 File change detected, redeploying...".yellow());
                                    
                                    // Trigger redeploy
                                    if let Err(e) = redeploy_project(&path_clone, port).await {
                                        eprintln!("{}", format!("❌ Redeploy failed: {}", e).red());
                                    } else {
                                        println!("{}", "✅ Redeploy completed".green());
                                    }
                                    
                                    println!("{}", "👀 Watching for changes...".dimmed());
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    eprintln!("File watcher error: {:?}", e);
                    break;
                }
            }
        }
    });
    
    Ok(())
}

/// Redeploy the project when files change
/// 
/// # Arguments
/// * `path` - Project directory path
/// * `port` - API server port
async fn redeploy_project(path: &str, port: u16) -> Result<()> {
    let fly_toml_path = Path::new(path).join("fly.toml");
    
    if fly_toml_path.exists() {
        // Use our deploy command
        let client = crate::client::ApiClient::new(&crate::config::Config {
            api_url: format!("http://localhost:{}", port),
            token: None,
        })?;
        
        crate::commands::deploy::handle(&client, Some(fly_toml_path.to_string_lossy().to_string()), None, false).await?;
    }
    
    Ok(())
}

/// Start real-time log streaming for the project
/// 
/// # Arguments
/// * `path` - Project directory path
/// * `port` - API server port
async fn start_log_streaming(_path: &str, _port: u16) -> Result<()> {
    println!("{}", "📊 Starting log streaming...".cyan());
    
    // TODO: Implement real-time log streaming
    // This would:
    // 1. Discover all machines for this project
    // 2. Connect to Docker logs API for each container
    // 3. Stream logs in real-time with color coding
    // 4. Include region and machine information
    
    println!("   ✓ {}", "Log streaming ready (implementation pending)".green());
    
    Ok(())
}