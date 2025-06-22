use anyhow::{Context, Result, bail};
use colored::*;
use std::fs;
use std::path::Path;
use std::process::Command;
use serde::Deserialize;
use crate::client::ApiClient;
use minifly_core::models::{
    CreateMachineRequest, MachineConfig, GuestConfig, ServiceConfig, 
    PortConfig, MountConfig, CreateAppRequest, RestartConfig,
};

#[derive(Debug, Deserialize)]
struct FlyToml {
    app: String,
    primary_region: Option<String>,
    build: Option<BuildConfig>,
    env: Option<std::collections::HashMap<String, String>>,
    mounts: Option<MountToml>,
    services: Option<Vec<ServiceToml>>,
}

#[derive(Debug, Deserialize)]
struct BuildConfig {
    dockerfile: Option<String>,
    image: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MountToml {
    source: String,
    destination: String,
}

#[derive(Debug, Deserialize)]
struct ServiceToml {
    internal_port: u16,
    protocol: String,
    ports: Vec<PortToml>,
}

#[derive(Debug, Deserialize)]
struct PortToml {
    port: u16,
    handlers: Vec<String>,
}

/// Handle the deploy command with optional watch mode
/// 
/// # Arguments
/// * `client` - API client for communicating with Minifly API
/// * `path` - Optional path to fly.toml file
/// * `watch` - Enable watch mode for automatic redeployment
pub async fn handle(client: &ApiClient, path: Option<String>, watch: bool) -> Result<()> {
    // Do the actual deployment
    deploy_without_watch(client, path).await?;
    
    // Enable watch mode if requested
    if watch {
        let fly_toml_path = "fly.toml".to_string();
        let abs_fly_toml_path = std::path::Path::new(&fly_toml_path).canonicalize()
            .context("Failed to get absolute path to fly.toml")?;
        
        println!("\n{}", "👀 Watch mode enabled - watching for changes...".yellow());
        start_watch_mode(client, &abs_fly_toml_path).await?;
    }
    
    Ok(())
}

/// Deploy without watch mode (internal function to avoid recursion)
async fn deploy_without_watch(client: &ApiClient, path: Option<String>) -> Result<()> {
    let fly_toml_path = path.unwrap_or_else(|| "fly.toml".to_string());
    
    // Get the absolute path before changing directories
    let abs_fly_toml_path = std::path::Path::new(&fly_toml_path).canonicalize()
        .context("Failed to get absolute path to fly.toml")?;
    
    // Change to the directory containing fly.toml
    let _original_dir = std::env::current_dir()?;
    if let Some(parent) = abs_fly_toml_path.parent() {
        std::env::set_current_dir(parent)
            .context("Failed to change to fly.toml directory")?;
    }
    
    let toml_filename = abs_fly_toml_path
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("fly.toml"))
        .to_string_lossy();
    
    println!("📖 Reading {}...", toml_filename.yellow());
    
    let content = fs::read_to_string(&abs_fly_toml_path)
        .context("Failed to read fly.toml")?;
    
    let config: FlyToml = toml::from_str(&content)
        .context("Failed to parse fly.toml")?;
    
    let app_name = config.app.clone();
    println!("🚀 Deploying app {}...", app_name.yellow());
    
    // 1. Ensure app exists
    ensure_app_exists(client, &app_name).await?;
    
    // 2. Build or pull Docker image
    let image = build_or_get_image(&config).await?;
    
    // 3. Check for LiteFS configuration
    let litefs_config = if Path::new("litefs.yml").exists() {
        println!("📦 Found litefs.yml, configuring LiteFS...");
        Some(fs::read_to_string("litefs.yml")?)
    } else {
        None
    };
    
    // 4. Create machine configuration
    let machine_config = create_machine_config(&config, &image, litefs_config.is_some())?;
    
    // 5. Deploy machine
    deploy_machine(client, &app_name, machine_config).await?;
    
    // Get the actual port mapping
    let port = config.services.as_ref()
        .and_then(|s| s.first())
        .and_then(|s| s.ports.first())
        .map(|p| p.port)
        .unwrap_or(80);
    
    println!("\n✅ {} deployed successfully!", "Application".green().bold());
    println!("🔗 Access your app at: {}", format!("http://localhost:{}", port).blue());
    println!("\n📝 To check machine status:");
    println!("   minifly machines list {}", app_name);
    println!("\n📋 To view logs:");
    println!("   minifly logs <machine-id>");
    
    Ok(())
}

async fn ensure_app_exists(client: &ApiClient, app_name: &str) -> Result<()> {
    // Try to get the app first
    match client.get(&format!("/apps/{}", app_name)).await {
        Ok(response) => {
            if response.status().is_success() {
                println!("✓ App {} already exists", app_name.green());
                return Ok(());
            }
        }
        Err(_) => {}
    }
    
    // Create the app
    println!("Creating app {}...", app_name);
    let create_req = CreateAppRequest {
        app_name: app_name.to_string(),
        org_slug: "personal".to_string(),
    };
    
    let response = client.post("/apps", &create_req).await?;
    if !response.status().is_success() {
        bail!("Failed to create app: {}", response.text().await?);
    }
    
    println!("✓ App {} created", app_name.green());
    Ok(())
}

async fn build_or_get_image(config: &FlyToml) -> Result<String> {
    if let Some(build) = &config.build {
        if let Some(image) = &build.image {
            println!("📦 Using image: {}", image.cyan());
            return Ok(image.clone());
        }
        
        let dockerfile = build.dockerfile.as_deref().unwrap_or("Dockerfile");
        if Path::new(dockerfile).exists() {
            println!("🔨 Building Docker image from {}...", dockerfile);
            
            let image_name = format!("{}-local:latest", config.app);
            let output = Command::new("docker")
                .args(&["build", "-t", &image_name, "-f", dockerfile, "."])
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .output()
                .context("Failed to execute docker build")?;
            
            if !output.status.success() {
                bail!("Docker build failed");
            }
            
            println!("✓ Docker image built: {}", image_name.green());
            return Ok(image_name);
        }
    }
    
    // Check if Dockerfile exists even without build config
    if Path::new("Dockerfile").exists() {
        println!("🔨 Found Dockerfile, building image...");
        
        let image_name = format!("{}-local:latest", config.app);
        let output = Command::new("docker")
            .args(&["build", "-t", &image_name, "."])
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .output()
            .context("Failed to execute docker build")?;
        
        if !output.status.success() {
            bail!("Docker build failed");
        }
        
        println!("✓ Docker image built: {}", image_name.green());
        return Ok(image_name);
    }
    
    // Default to a basic image
    println!("⚠️  No Dockerfile found, using default alpine image");
    Ok("alpine:latest".to_string())
}

fn create_machine_config(config: &FlyToml, image: &str, has_litefs: bool) -> Result<MachineConfig> {
    let mut env = config.env.clone().unwrap_or_default();
    
    // Add LiteFS environment variables if LiteFS is configured
    if has_litefs {
        env.insert("FLY_LITEFS_PRIMARY".to_string(), "true".to_string());
        if !env.contains_key("DATABASE_PATH") {
            env.insert("DATABASE_PATH".to_string(), "/litefs".to_string());
        }
    }
    
    // Convert services
    let services = config.services.as_ref().map(|services| {
        services.iter().map(|s| ServiceConfig {
            ports: s.ports.iter().map(|p| PortConfig {
                port: p.port,
                handlers: p.handlers.clone(),
                force_https: Some(false),
                tls_options: None,
            }).collect(),
            protocol: s.protocol.clone(),
            internal_port: s.internal_port,
            autostart: None,
            autostop: None,
            force_instance_description: None,
        }).collect()
    });
    
    // Convert mounts
    let mounts = config.mounts.as_ref().map(|m| {
        vec![MountConfig {
            volume: m.source.clone(),
            path: m.destination.clone(),
        }]
    });
    
    Ok(MachineConfig {
        image: image.to_string(),
        guest: GuestConfig {
            cpu_kind: "shared".to_string(),
            cpus: 1,
            memory_mb: 512,
            gpu_kind: None,
            gpus: None,
            kernel_args: None,
        },
        env: Some(env),
        services,
        mounts,
        restart: Some(RestartConfig {
            policy: "on-failure".to_string(),
            max_retries: Some(3),
        }),
        checks: None,
        auto_destroy: None,
        dns: None,
        processes: None,
        files: None,
        init: None,
        containers: None,
    })
}

async fn deploy_machine(client: &ApiClient, app_name: &str, config: MachineConfig) -> Result<()> {
    println!("🚀 Creating machine...");
    
    let req = CreateMachineRequest {
        name: Some(format!("{}-1", app_name)),
        region: Some("local".to_string()),
        config,
        skip_launch: Some(false),
        skip_service_registration: None,
        lease_ttl: None,
    };
    
    let response = client.post(&format!("/apps/{}/machines", app_name), &req).await?;
    
    if response.status().is_success() {
        let machine: serde_json::Value = response.json().await?;
        let machine_id = machine["id"].as_str().unwrap_or("unknown");
        println!("✓ Machine created: {}", machine_id.green());
        
        // Wait for machine to be ready
        println!("⏳ Waiting for machine to start...");
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        
        Ok(())
    } else {
        bail!("Failed to create machine: {}", response.text().await?);
    }
}

/// Start watch mode for automatic redeployment
/// 
/// # Arguments
/// * `client` - API client for deployments  
/// * `fly_toml_path` - Path to the fly.toml file to watch
async fn start_watch_mode(client: &ApiClient, fly_toml_path: &std::path::Path) -> Result<()> {
    use notify::{Watcher, RecursiveMode, watcher};
    use std::sync::mpsc::channel;
    use std::time::Duration;
    
    let (tx, rx) = channel();
    let mut _watcher = watcher(tx, Duration::from_secs(1))?;
    
    // Watch the directory containing the fly.toml
    if let Some(dir) = fly_toml_path.parent() {
        _watcher.watch(dir, RecursiveMode::Recursive)?;
        println!("   ✓ Watching {} for changes", dir.display());
    }
    
    println!("{}", "Press Ctrl+C to stop watching".dimmed());
    
    // Clone client and path for the spawned task
    let client_clone = client.clone();
    let fly_toml_path_clone = fly_toml_path.to_path_buf();
    
    // Spawn a task to handle file change events
    let handle = tokio::spawn(async move {
        use notify::DebouncedEvent;
        
        loop {
            match rx.recv() {
                Ok(event) => {
                    match event {
                        DebouncedEvent::Write(path) | DebouncedEvent::Create(path) => {
                            if let Some(filename) = path.file_name() {
                                if filename == "fly.toml" || 
                                   filename == "Dockerfile" ||
                                   filename == "litefs.yml" ||
                                   path.extension().map_or(false, |ext| ext == "rs" || ext == "js" || ext == "py") {
                                    
                                    println!("\n{}", "🔄 Change detected, redeploying...".yellow());
                                    
                                    // Redeploy without watch mode to avoid recursion
                                    match deploy_without_watch(&client_clone, Some(fly_toml_path_clone.to_string_lossy().to_string())).await {
                                        Ok(_) => println!("{}", "✅ Redeploy completed".green()),
                                        Err(e) => eprintln!("{}", format!("❌ Redeploy failed: {}", e).red()),
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
    
    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await.context("Failed to listen for ctrl-c")?;
    println!("\n{}", "🛑 Stopping watch mode...".yellow());
    
    // Cancel the watcher task
    handle.abort();
    
    Ok(())
}