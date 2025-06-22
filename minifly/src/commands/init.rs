//! Initialize Minifly environment

use anyhow::Result;
use colored::*;
use minifly::Config;

/// Handle the init command
pub async fn handle(config: &Config) -> Result<()> {
    println!("{}", "🚀 Initializing Minifly environment...".blue().bold());
    
    // Initialize configuration
    Config::init()?;
    
    println!("{}", "✅ Minifly initialized successfully!".green().bold());
    println!();
    println!("Next steps:");
    println!("  1. Start the platform: {}", "minifly serve".cyan());
    println!("  2. Create an app: {}", "minifly apps create my-app".cyan());
    println!("  3. Deploy a machine: {}", "minifly machines create --app my-app --image nginx:latest".cyan());
    
    Ok(())
}