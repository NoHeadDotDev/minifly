//! Application management commands

use anyhow::Result;
use colored::*;
use tabled::{Table, Tabled};
use minifly::ApiClient;

/// List all applications
pub async fn list(client: &ApiClient) -> Result<()> {
    // Check if API server is running
    if !client.health_check().await? {
        println!("{}", "❌ Minifly API server is not running".red());
        println!("Start it with: {}", "minifly serve".cyan());
        return Ok(());
    }
    
    println!("{}", "📋 Listing applications...".blue());
    
    match client.list_apps().await {
        Ok(apps) => {
            if apps.is_empty() {
                println!("{}", "No applications found".yellow());
                println!("Create one with: {}", "minifly apps create <name>".cyan());
            } else {
                let table_data: Vec<AppRow> = apps.into_iter().map(|app| AppRow {
                    name: app.name,
                    status: app.status,
                    hostname: app.hostname,
                    deployed: if app.deployed { "Yes".to_string() } else { "No".to_string() },
                }).collect();
                
                let table = Table::new(table_data);
                println!("{}", table);
            }
        }
        Err(e) => {
            println!("{}", format!("❌ Failed to list applications: {}", e).red());
        }
    }
    
    Ok(())
}

/// Create a new application
pub async fn create(client: &ApiClient, name: &str) -> Result<()> {
    if !client.health_check().await? {
        println!("{}", "❌ Minifly API server is not running".red());
        println!("Start it with: {}", "minifly serve".cyan());
        return Ok(());
    }
    
    println!("{}", format!("🚀 Creating application '{}'...", name).blue());
    
    match client.create_app(name).await {
        Ok(app) => {
            println!("{}", format!("✅ Application '{}' created successfully!", app.name).green());
            println!("  • Hostname: {}", app.hostname.cyan());
            println!("  • Status: {}", app.status.yellow());
        }
        Err(e) => {
            println!("{}", format!("❌ Failed to create application: {}", e).red());
        }
    }
    
    Ok(())
}

/// Delete an application
pub async fn delete(client: &ApiClient, name: &str) -> Result<()> {
    if !client.health_check().await? {
        println!("{}", "❌ Minifly API server is not running".red());
        println!("Start it with: {}", "minifly serve".cyan());
        return Ok(());
    }
    
    println!("{}", format!("🗑️  Deleting application '{}'...", name).yellow());
    
    match client.delete_app(name).await {
        Ok(_) => {
            println!("{}", format!("✅ Application '{}' deleted successfully!", name).green());
        }
        Err(e) => {
            println!("{}", format!("❌ Failed to delete application: {}", e).red());
        }
    }
    
    Ok(())
}

#[derive(Tabled)]
struct AppRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Hostname")]
    hostname: String,
    #[tabled(rename = "Deployed")]
    deployed: String,
}