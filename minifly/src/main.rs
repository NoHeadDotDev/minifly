//! Minifly CLI - Local Fly.io development simulator
//!
//! This is the main CLI entry point for Minifly, providing a comprehensive set of commands
//! for local Fly.io development and testing.

use anyhow::Result;
use clap::{Parser, Subcommand};
use minifly::{Config, ApiClient};

mod commands;

use commands::{apps, deploy, dev, init, logs, machines, proxy, serve, status, stop};

#[derive(Parser)]
#[command(name = "minifly")]
#[command(about = "Local Fly.io development simulator with incredible DX", long_about = None)]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "Minifly Contributors")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(short, long, global = true, help = "API endpoint")]
    api_url: Option<String>,
    
    #[arg(short, long, global = true, help = "Authentication token")]
    token: Option<String>,
    
    #[arg(short, long, global = true, help = "Enable verbose logging")]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Minifly environment
    Init,
    
    /// Start the Minifly platform (API server + LiteFS)
    Serve {
        #[arg(short, long, help = "Run in background as daemon")]
        daemon: bool,
        
        #[arg(short, long, help = "Port for API server", default_value = "4280")]
        port: u16,
        
        #[arg(long, help = "Enable development mode with auto-reload")]
        dev: bool,
    },
    
    /// Development mode with auto-reload and log streaming
    Dev {
        #[arg(help = "Path to project directory", default_value = ".")]
        path: String,
        
        #[arg(short, long, help = "Port for API server", default_value = "4280")]
        port: u16,
    },
    
    /// Stop the Minifly platform
    Stop {
        #[arg(short, long, help = "Force stop all services")]
        force: bool,
    },
    
    /// Manage applications
    #[command(subcommand)]
    Apps(AppsCommands),
    
    /// Manage machines
    #[command(subcommand)]
    Machines(MachinesCommands),
    
    /// Deploy an application
    Deploy {
        #[arg(help = "Path to fly.toml")]
        path: Option<String>,
        
        #[arg(short, long, help = "Watch for changes and auto-redeploy")]
        watch: bool,
    },
    
    /// View logs from machines
    Logs {
        #[arg(help = "Machine ID")]
        machine_id: String,
        
        #[arg(short, long, help = "Follow log output")]
        follow: bool,
        
        #[arg(short, long, help = "Show logs from specific region")]
        region: Option<String>,
    },
    
    /// Proxy to a running service
    Proxy {
        #[arg(help = "Machine ID")]
        machine_id: String,
        
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
    
    /// Show Minifly status
    Status,
}

#[derive(Subcommand)]
enum AppsCommands {
    /// List all applications
    List,
    
    /// Create a new application
    Create {
        #[arg(help = "Application name")]
        name: String,
    },
    
    /// Delete an application
    Delete {
        #[arg(help = "Application name")]
        name: String,
    },
}

#[derive(Subcommand)]
enum MachinesCommands {
    /// List machines for an app
    List {
        #[arg(short, long, help = "Application name")]
        app: String,
    },
    
    /// Create a new machine
    Create {
        #[arg(short, long, help = "Application name")]
        app: String,
        
        #[arg(short, long, help = "Docker image")]
        image: String,
        
        #[arg(short, long, help = "Machine name")]
        name: Option<String>,
        
        #[arg(short, long, help = "Region")]
        region: Option<String>,
    },
    
    /// Start a machine
    Start {
        #[arg(help = "Machine ID")]
        machine_id: String,
    },
    
    /// Stop a machine
    Stop {
        #[arg(help = "Machine ID")]
        machine_id: String,
    },
    
    /// Delete a machine
    Delete {
        #[arg(help = "Machine ID")]
        machine_id: String,
        
        #[arg(short, long, help = "Force deletion")]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging if verbose mode or debug env var is set
    if cli.verbose || std::env::var("MINIFLY_DEBUG").is_ok() {
        minifly::logging::init_default_logging()?;
    }
    
    // Load configuration
    let mut config = Config::load()?;
    
    // Override with CLI arguments
    if let Some(api_url) = cli.api_url {
        config.api_url = api_url;
    }
    if let Some(token) = cli.token {
        config.token = Some(token);
    }
    
    // Create API client
    let client = ApiClient::new(&config)?;
    
    match cli.command {
        Commands::Init => {
            init::handle(&config).await?;
        }
        Commands::Serve { daemon, port, dev } => {
            serve::handle(daemon, port, dev).await?;
        }
        Commands::Dev { path, port } => {
            dev::handle(&path, port).await?;
        }
        Commands::Stop { force } => {
            stop::handle(force).await?;
        }
        Commands::Apps(cmd) => match cmd {
            AppsCommands::List => {
                apps::list(&client).await?;
            }
            AppsCommands::Create { name } => {
                apps::create(&client, &name).await?;
            }
            AppsCommands::Delete { name } => {
                apps::delete(&client, &name).await?;
            }
        },
        Commands::Machines(cmd) => match cmd {
            MachinesCommands::List { app } => {
                machines::list(&client, &app).await?;
            }
            MachinesCommands::Create { app, image, name, region } => {
                machines::create(&client, &app, &image, name, region).await?;
            }
            MachinesCommands::Start { machine_id } => {
                machines::start(&client, &machine_id).await?;
            }
            MachinesCommands::Stop { machine_id } => {
                machines::stop(&client, &machine_id).await?;
            }
            MachinesCommands::Delete { machine_id, force } => {
                machines::delete(&client, &machine_id, force).await?;
            }
        },
        Commands::Deploy { path, watch } => {
            deploy::handle(&client, path, watch).await?;
        }
        Commands::Logs { machine_id, follow, region } => {
            logs::handle(&client, &machine_id, follow, region).await?;
        }
        Commands::Proxy { machine_id, port } => {
            proxy::handle(&client, &machine_id, port).await?;
        }
        Commands::Status => {
            status::handle(&client).await?;
        }
    }
    
    Ok(())
}