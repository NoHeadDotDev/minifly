use anyhow::{Context, Result, bail};
use colored::*;
use std::fs;
use std::path::Path;
use std::process::Command;
use serde::{Deserialize, Deserializer};
use crate::client::ApiClient;
use crate::commands::secrets;
use minifly_core::models::{
    CreateMachineRequest, MachineConfig, GuestConfig, ServiceConfig, 
    PortConfig, MountConfig, CreateAppRequest, RestartConfig,
    AutostartConfig, AutostopConfig, TlsOptions,
};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct FlyToml {
    app: String,
    primary_region: Option<String>,
    build: Option<BuildConfig>,
    env: Option<std::collections::HashMap<String, String>>,
    #[serde(deserialize_with = "deserialize_mounts", default)]
    mounts: Option<Vec<MountToml>>,
    services: Option<Vec<ServiceToml>>,
    #[serde(rename = "http_service")]
    http_service: Option<HttpServiceToml>,
    #[serde(rename = "vm", default)]
    vm: Option<Vec<VmToml>>,
    statics: Option<Vec<StaticsToml>>,
    deploy: Option<DeployToml>,
    
    // Additional fields for validation
    #[serde(default)]
    experimental: Option<toml::Value>,
    #[serde(default)]
    processes: Option<toml::Value>,
    #[serde(default)]
    metrics: Option<toml::Value>,
    #[serde(default)]
    swap_size_mb: Option<u32>,
}

// Custom deserializer for mounts that handles both single object and array
fn deserialize_mounts<'de, D>(deserializer: D) -> Result<Option<Vec<MountToml>>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;
    
    struct MountsVisitor;
    
    impl<'de> Visitor<'de> for MountsVisitor {
        type Value = Option<Vec<MountToml>>;
        
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a mount object or array of mount objects")
        }
        
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
        
        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(MountsInnerVisitor)
        }
    }
    
    struct MountsInnerVisitor;
    
    impl<'de> Visitor<'de> for MountsInnerVisitor {
        type Value = Option<Vec<MountToml>>;
        
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a mount object or array of mount objects")
        }
        
        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            let mount = MountToml::deserialize(de::value::MapAccessDeserializer::new(map))?;
            Ok(Some(vec![mount]))
        }
        
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut mounts = Vec::new();
            while let Some(mount) = seq.next_element()? {
                mounts.push(mount);
            }
            Ok(Some(mounts))
        }
    }
    
    deserializer.deserialize_option(MountsVisitor)
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
    
    #[serde(default)]
    auto_stop_machines: Option<bool>,
    #[serde(default)]
    auto_start_machines: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct PortToml {
    port: u16,
    handlers: Vec<String>,
    #[serde(default)]
    force_https: Option<bool>,
    #[serde(default)]
    tls_options: Option<TlsOptionsToml>,
}

#[derive(Debug, Deserialize)]
struct TlsOptionsToml {
    #[serde(default)]
    alpn: Option<Vec<String>>,
    #[serde(default)]
    versions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct HttpServiceToml {
    internal_port: u16,
    #[serde(default)]
    force_https: Option<bool>,
    #[serde(default)]
    auto_stop_machines: Option<String>,
    #[serde(default)]
    auto_start_machines: Option<bool>,
    #[serde(default)]
    min_machines_running: Option<u32>,
    #[serde(default)]
    processes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct VmToml {
    #[serde(default)]
    size: Option<String>,
    #[serde(default)]
    cpu_kind: Option<String>,
    #[serde(default)]
    cpus: Option<u32>,
    #[serde(default)]
    memory_mb: Option<u32>,
    #[serde(default)]
    memory: Option<String>,
    #[serde(default)]
    processes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct StaticsToml {
    guest_path: String,
    url_prefix: String,
    #[serde(default)]
    tigris_bucket: Option<String>,
    #[serde(default)]
    index_document: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeployToml {
    #[serde(default)]
    strategy: Option<String>,
    #[serde(default)]
    release_command: Option<String>,
    #[serde(default)]
    wait_timeout: Option<String>,
}

/// Handles the deploy command with production config compatibility.
/// 
/// This function deploys applications using production Fly.io configurations
/// without requiring modifications. It automatically handles:
/// 
/// - Environment variable translation (FLY_* variables)
/// - Secrets loading from .fly.secrets files
/// - Volume mapping to local directories  
/// - LiteFS production configuration adaptation
/// - Dockerfile building with Fly.io compatibility
/// - Service discovery registration
/// - Fly.toml validation with compatibility warnings
/// 
/// # Arguments
/// 
/// * `client` - API client for communicating with Minifly API
/// * `path` - Optional path to fly.toml file (defaults to "fly.toml")
/// * `watch` - Enable watch mode for automatic redeployment on file changes
/// 
/// # Example
/// 
/// ```rust
/// # use minifly_cli::commands::deploy;
/// # use minifly_cli::client::ApiClient;
/// # tokio_test::block_on(async {
/// let client = ApiClient::new(&config)?;
/// 
/// // Deploy with production fly.toml
/// deploy::handle(&client, None, false).await?;
/// 
/// // Deploy with watch mode
/// deploy::handle(&client, Some("./app/fly.toml".to_string()), true).await?;
/// # Ok::<(), anyhow::Error>(())
/// # });
/// ```
pub async fn handle(client: &ApiClient, path: Option<String>, litefs_config: Option<String>, watch: bool) -> Result<()> {
    // Set litefs config path in environment if provided
    if let Some(litefs_path) = &litefs_config {
        std::env::set_var("LITEFS_CONFIG_PATH", litefs_path);
    }
    
    // Do the actual deployment
    let _url = deploy_without_watch(client, path, true).await?;
    
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

/// Handle deployment quietly (for auto-deployment from serve command)
pub async fn handle_quiet(client: &ApiClient, path: Option<String>) -> Result<String> {
    deploy_without_watch(client, path, false).await
}

/// Deploy without watch mode (internal function to avoid recursion)
async fn deploy_without_watch(client: &ApiClient, path: Option<String>, show_output: bool) -> Result<String> {
    // Determine which fly.toml to use
    let fly_toml_path = if let Some(explicit_path) = path {
        // Use explicitly specified path
        explicit_path
    } else {
        // Check for environment-specific config files
        let env = std::env::var("FLY_ENV").or_else(|_| std::env::var("MINIFLY_ENV")).ok();
        
        let config_path = if let Some(env_name) = env {
            // Try environment-specific config first
            let env_specific_path = format!("fly.{}.toml", env_name.to_lowercase());
            if Path::new(&env_specific_path).exists() {
                println!("📝 Using environment-specific config: {}", env_specific_path.yellow());
                env_specific_path
            } else {
                "fly.toml".to_string()
            }
        } else {
            "fly.toml".to_string()
        };
        
        config_path
    };
    
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
    
    // Validate fly.toml and show warnings
    let warnings = validate_fly_toml(&config);
    if !warnings.is_empty() {
        println!("\n⚠️  {} found:", "Compatibility warnings".yellow());
        for warning in warnings {
            println!("   • {}", warning);
        }
        println!();
    }
    
    // 1. Ensure app exists
    ensure_app_exists(client, &app_name).await?;
    
    // 2. Build or pull Docker image
    let image = build_or_get_image(&config).await?;
    
    // 3. Check for LiteFS configuration
    let litefs_config = {
        // Check for environment-specific litefs config
        let env = std::env::var("FLY_ENV").or_else(|_| std::env::var("MINIFLY_ENV")).ok();
        let litefs_path = std::env::var("LITEFS_CONFIG_PATH").ok();
        
        let config_path = if let Some(explicit_path) = litefs_path {
            // Use explicitly specified path
            if Path::new(&explicit_path).exists() {
                println!("📦 Using LiteFS config from LITEFS_CONFIG_PATH: {}", explicit_path.yellow());
                Some(explicit_path)
            } else {
                println!("⚠️  LITEFS_CONFIG_PATH specified but file not found: {}", explicit_path);
                None
            }
        } else if let Some(env_name) = env {
            // Try environment-specific config
            let env_specific_path = format!("litefs.{}.yml", env_name.to_lowercase());
            if Path::new(&env_specific_path).exists() {
                println!("📦 Using environment-specific LiteFS config: {}", env_specific_path.yellow());
                Some(env_specific_path)
            } else if Path::new("litefs.yml").exists() {
                println!("📦 Found litefs.yml, configuring LiteFS...");
                Some("litefs.yml".to_string())
            } else {
                None
            }
        } else if Path::new("litefs.yml").exists() {
            println!("📦 Found litefs.yml, configuring LiteFS...");
            Some("litefs.yml".to_string())
        } else {
            None
        };
        
        config_path.and_then(|path| fs::read_to_string(path).ok())
    };
    
    // 4. Load secrets for the app
    let app_secrets = secrets::load_secrets(&app_name).await
        .unwrap_or_else(|_| {
            println!("⚠️  No secrets found for app {}", app_name.yellow());
            std::collections::HashMap::new()
        });
    
    if !app_secrets.is_empty() {
        println!("🔐 Loaded {} secrets for app {}", app_secrets.len(), app_name.yellow());
    }
    
    // 5. Create machine configuration with secrets
    let machine_config = create_machine_config(&config, &image, litefs_config.is_some(), app_secrets)?;
    
    // 6. Deploy machine
    let machine_id = deploy_machine(client, &app_name, machine_config).await?;
    
    // Wait a bit for port assignment
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Get the actual port from Docker
    let actual_port = get_container_port(&app_name, &machine_id).await?;
    
    if show_output {
        println!("\n✅ {} deployed successfully!", "Application".green().bold());
        println!("🔗 Access your app at: {}", format!("http://localhost:{}", actual_port).blue());
        println!("\n📝 To check machine status:");
        println!("   minifly machines list {}", app_name);
        println!("\n📋 To view logs:");
        println!("   minifly logs {}", machine_id);
    }
    
    Ok(format!("http://localhost:{}", actual_port))
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
            
            let image_name = build_with_fly_compatibility(dockerfile, config).await?;
            
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

fn create_machine_config(
    config: &FlyToml, 
    image: &str, 
    has_litefs: bool,
    secrets: std::collections::HashMap<String, String>
) -> Result<MachineConfig> {
    let mut env = config.env.clone().unwrap_or_default();
    
    // Add secrets to environment
    for (key, value) in secrets {
        env.insert(key, value);
    }
    
    // Add LiteFS environment variables if LiteFS is configured
    if has_litefs {
        env.insert("FLY_LITEFS_PRIMARY".to_string(), "true".to_string());
        if !env.contains_key("DATABASE_PATH") {
            env.insert("DATABASE_PATH".to_string(), "/litefs".to_string());
        }
    }
    
    // Convert services - handle both [[services]] array and [http_service]
    let services = if let Some(services) = &config.services {
        // Traditional [[services]] format
        Some(services.iter().map(|s| ServiceConfig {
            ports: s.ports.iter().map(|p| PortConfig {
                port: p.port,
                handlers: p.handlers.clone(),
                force_https: p.force_https.or(Some(false)),
                tls_options: p.tls_options.as_ref().map(|tls| TlsOptions {
                    alpn: tls.alpn.clone().unwrap_or_else(|| vec!["h2".to_string(), "http/1.1".to_string()]),
                    versions: tls.versions.clone().unwrap_or_else(|| vec!["TLSv1.2".to_string(), "TLSv1.3".to_string()]),
                }),
            }).collect(),
            protocol: s.protocol.clone(),
            internal_port: s.internal_port,
            autostart: s.auto_start_machines.map(|enabled| AutostartConfig { enabled: Some(enabled) }),
            autostop: s.auto_stop_machines.map(|enabled| AutostopConfig {
                enabled: Some(enabled),
                seconds: None,
            }),
            force_instance_description: None,
        }).collect())
    } else if let Some(http_service) = &config.http_service {
        // New [http_service] format - convert to services
        Some(vec![ServiceConfig {
            ports: vec![
                PortConfig {
                    port: 80,
                    handlers: vec!["http".to_string()],
                    force_https: http_service.force_https,
                    tls_options: None,
                },
                PortConfig {
                    port: 443,
                    handlers: vec!["tls".to_string(), "http".to_string()],
                    force_https: None,
                    tls_options: None,
                },
            ],
            protocol: "tcp".to_string(),
            internal_port: http_service.internal_port,
            autostart: http_service.auto_start_machines.map(|enabled| AutostartConfig { enabled: Some(enabled) }),
            autostop: http_service.auto_stop_machines.as_ref().map(|stop| AutostopConfig {
                enabled: Some(stop != "off"),
                seconds: None,
            }),
            force_instance_description: None,
        }])
    } else {
        None
    };
    
    // Convert mounts
    let mounts = config.mounts.as_ref().map(|mounts| {
        mounts.iter().map(|m| MountConfig {
            volume: m.source.clone(),
            path: m.destination.clone(),
        }).collect()
    });
    
    // Extract VM configuration
    let guest = if let Some(vm_configs) = &config.vm {
        // Use the first VM config (or could match by process group)
        let vm = vm_configs.first();
        if let Some(vm) = vm {
            let memory_mb = vm.memory_mb.or_else(|| {
                // Parse memory string like "1gb" or "512mb"
                vm.memory.as_ref().and_then(|mem| {
                    let mem_lower = mem.to_lowercase();
                    if mem_lower.ends_with("gb") {
                        mem_lower.trim_end_matches("gb").parse::<u32>().ok().map(|gb| gb * 1024)
                    } else if mem_lower.ends_with("mb") {
                        mem_lower.trim_end_matches("mb").parse::<u32>().ok()
                    } else {
                        None
                    }
                })
            }).unwrap_or(1024);
            
            GuestConfig {
                cpu_kind: vm.cpu_kind.clone().unwrap_or_else(|| "shared".to_string()),
                cpus: vm.cpus.unwrap_or(1),
                memory_mb,
                gpu_kind: None,
                gpus: None,
                kernel_args: None,
            }
        } else {
            GuestConfig {
                cpu_kind: "shared".to_string(),
                cpus: 1,
                memory_mb: 1024,
                gpu_kind: None,
                gpus: None,
                kernel_args: None,
            }
        }
    } else {
        // Default guest config
        GuestConfig {
            cpu_kind: "shared".to_string(),
            cpus: 1,
            memory_mb: 1024,
            gpu_kind: None,
            gpus: None,
            kernel_args: None,
        }
    };
    
    Ok(MachineConfig {
        image: image.to_string(),
        guest,
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

async fn deploy_machine(client: &ApiClient, app_name: &str, config: MachineConfig) -> Result<String> {
    // Check if a machine already exists for this app
    let machines_response = client.get(&format!("/apps/{}/machines", app_name)).await?;
    
    if machines_response.status().is_success() {
        let machines: Vec<serde_json::Value> = machines_response.json().await?;
        
        if !machines.is_empty() {
            println!("🔄 Found existing machine(s), updating the first one...");
            
            // Get the first machine
            let machine_id = machines[0]["id"].as_str().unwrap_or("unknown");
            let machine_state = machines[0]["state"].as_str().unwrap_or("unknown");
            
            // If machine is stopped, start it
            if machine_state == "stopped" || machine_state == "created" {
                println!("   Starting stopped machine {}...", machine_id);
                let start_response = client.post(&format!("/apps/{}/machines/{}/start", app_name, machine_id), &serde_json::json!({})).await?;
                
                if !start_response.status().is_success() {
                    println!("   ⚠️  Failed to start existing machine, creating new one instead");
                } else {
                    println!("✓ Machine {} started", machine_id.green());
                    return Ok(machine_id.to_string());
                }
            } else if machine_state == "started" || machine_state == "starting" {
                println!("✓ Machine {} is already running", machine_id.green());
                return Ok(machine_id.to_string());
            }
        }
    }
    
    // No existing machines or failed to start, create a new one
    println!("🚀 Creating machine...");
    
    // Generate a unique name for the machine
    let machine_name = format!("{}-{}", app_name, &uuid::Uuid::new_v4().to_string()[..8]);
    
    let req = CreateMachineRequest {
        name: Some(machine_name),
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
        
        Ok(machine_id.to_string())
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
                                    match deploy_without_watch(&client_clone, Some(fly_toml_path_clone.to_string_lossy().to_string()), true).await {
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

/// Validates a fly.toml configuration and returns warnings for unsupported features.
/// 
/// This function analyzes production Fly.io configurations and identifies features
/// that may not work exactly the same in local development with Minifly. It helps
/// developers understand what functionality is simulated vs. fully supported.
/// 
/// # Validated Features
/// 
/// - `auto_stop_machines` / `auto_start_machines` - Simulated with container pause/unpause
/// - `experimental` features - May not be fully supported
/// - `processes` (multi-process apps) - Simulated as separate containers
/// - `metrics` endpoints - Not automatically configured locally
/// - `primary_region` - Ignored (all machines run in 'local' region)
/// 
/// # Arguments
/// 
/// * `config` - The parsed fly.toml configuration to validate
/// 
/// # Returns
/// 
/// A vector of warning messages describing compatibility issues or limitations.
/// 
/// # Example
/// 
/// ```rust
/// # use minifly_cli::commands::deploy::{FlyToml, validate_fly_toml};
/// let config = FlyToml {
///     app: "myapp".to_string(),
///     primary_region: Some("iad".to_string()),
///     // ... other fields
/// };
/// 
/// let warnings = validate_fly_toml(&config);
/// for warning in warnings {
///     println!("⚠️  {}", warning);
/// }
/// ```
fn validate_fly_toml(config: &FlyToml) -> Vec<String> {
    let mut warnings = vec![];
    
    // Check for auto stop/start machines
    if let Some(services) = &config.services {
        for service in services {
            if service.auto_stop_machines.unwrap_or(false) {
                warnings.push("auto_stop_machines is simulated with container pause/unpause".to_string());
            }
            if service.auto_start_machines.unwrap_or(false) {
                warnings.push("auto_start_machines is not fully supported - machines start manually".to_string());
            }
        }
    }
    
    // Check for experimental features
    if config.experimental.is_some() {
        warnings.push("Experimental features may not be fully supported in local development".to_string());
    }
    
    // Check for processes (multi-process apps)
    if config.processes.is_some() {
        warnings.push("Multi-process apps are simulated as separate containers".to_string());
    }
    
    // Check for metrics
    if config.metrics.is_some() {
        warnings.push("Metrics endpoints are not automatically configured locally".to_string());
    }
    
    // Check for primary region
    if config.primary_region.is_some() {
        warnings.push("Primary region is ignored - all machines run in 'local' region".to_string());
    }
    
    warnings
}

/// Get the actual port assigned to a container
async fn get_container_port(app_name: &str, machine_id: &str) -> Result<u16> {
    use std::process::Command;
    
    // Try to get port using container name with machine ID
    let container_name = format!("minifly-{}-{}", app_name, machine_id);
    
    // Method 1: Use docker port command
    let output = Command::new("docker")
        .args(&["port", &container_name])
        .output();
        
    if let Ok(output) = output {
        if output.status.success() {
            let ports = String::from_utf8_lossy(&output.stdout);
            // Parse output like "8080/tcp -> 0.0.0.0:32768"
            for line in ports.lines() {
                if let Some(mapping) = line.split(" -> ").nth(1) {
                    if let Some(port) = mapping.split(':').last() {
                        let port = port.trim();
                        if let Ok(port_num) = port.parse::<u16>() {
                            return Ok(port_num);
                        }
                    }
                }
            }
        }
    }
    
    // Method 2: Use docker ps with filters
    let output = Command::new("docker")
        .args(&["ps", "--filter", &format!("name={}", container_name), "--format", "{{.Ports}}"])
        .output();
        
    if let Ok(output) = output {
        if output.status.success() {
            let ports = String::from_utf8_lossy(&output.stdout);
            // Parse port mapping
            for port_mapping in ports.lines() {
                if let Some(host_part) = port_mapping.split("->").next() {
                    if let Some(port) = host_part.split(':').last() {
                        let port = port.trim();
                        if let Ok(port_num) = port.parse::<u16>() {
                            return Ok(port_num);
                        }
                    }
                }
            }
        }
    }
    
    // Method 3: Try with just app name if machine ID doesn't work
    let output = Command::new("docker")
        .args(&["ps", "--filter", &format!("name=minifly-{}", app_name), "--format", "{{.Ports}}", "--latest"])
        .output();
        
    if let Ok(output) = output {
        if output.status.success() {
            let ports = String::from_utf8_lossy(&output.stdout);
            for port_mapping in ports.lines() {
                if let Some(host_part) = port_mapping.split("->").next() {
                    if let Some(port) = host_part.split(':').last() {
                        let port = port.trim();
                        if let Ok(port_num) = port.parse::<u16>() {
                            return Ok(port_num);
                        }
                    }
                }
            }
        }
    }
    
    // If we can't determine the port, return a default
    Ok(8080)
}

/// Build Docker image with Fly.io compatibility
async fn build_with_fly_compatibility(dockerfile: &str, config: &FlyToml) -> Result<String> {
    let image_name = format!("{}-local:latest", config.app);
    
    // Read Dockerfile to check for Fly.io specific features
    let dockerfile_content = fs::read_to_string(dockerfile)
        .context("Failed to read Dockerfile")?;
    
    // Build arguments with proper ownership
    let fly_app_name_arg = format!("FLY_APP_NAME={}", config.app);
    let fly_region_arg = "FLY_REGION=local";
    let fly_build_id_arg = "FLY_BUILD_ID=local-build";
    
    let mut build_args = vec!["build", "-t", &image_name, "-f", dockerfile];
    
    // Add Fly.io build arguments
    build_args.extend(&[
        "--build-arg", &fly_app_name_arg,
        "--build-arg", fly_region_arg,
        "--build-arg", fly_build_id_arg,
    ]);
    
    // Warn about Fly.io base images
    if dockerfile_content.contains("FROM flyio/") {
        println!("⚠️  Dockerfile uses Fly.io base image - using closest equivalent");
    }
    
    // Add current directory
    build_args.push(".");
    
    let output = Command::new("docker")
        .args(&build_args)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .output()
        .context("Failed to execute docker build")?;
    
    if !output.status.success() {
        bail!("Docker build failed");
    }
    
    Ok(image_name)
}