//! Platform status command

use anyhow::Result;
use colored::*;
use minifly::ApiClient;

/// Handle the status command
pub async fn handle(client: &ApiClient) -> Result<()> {
    println!("{}", "📊 Minifly Platform Status".blue().bold());
    println!();
    
    // Check API server status
    println!("{}", "🔧 API Server".bold());
    if client.health_check().await? {
        println!("  Status: {}", "Running".green());
        println!("  URL: {}", "http://localhost:4280".cyan());
    } else {
        println!("  Status: {}", "Not Running".red());
        println!("  Start with: {}", "minifly serve".cyan());
    }
    
    println!();
    println!("{}", "📋 Quick Commands".bold());
    println!("  • List apps: {}", "minifly apps list".cyan());
    println!("  • Create app: {}", "minifly apps create <name>".cyan());
    println!("  • Create machine: {}", "minifly machines create --app <app> --image <image>".cyan());
    
    Ok(())
}