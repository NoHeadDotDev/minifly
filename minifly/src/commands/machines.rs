//! Machine management commands

use anyhow::Result;
use colored::*;
use tabled::{Table, Tabled};
use minifly::ApiClient;

/// List machines for an application
pub async fn list(client: &ApiClient, app_name: &str) -> Result<()> {
    if !client.health_check().await? {
        println!("{}", "❌ Minifly API server is not running".red());
        println!("Start it with: {}", "minifly serve".cyan());
        return Ok(());
    }
    
    println!("{}", format!("📋 Listing machines for app '{}'...", app_name).blue());
    
    match client.list_machines(app_name).await {
        Ok(machines) => {
            if machines.is_empty() {
                println!("{}", "No machines found".yellow());
                println!("Create one with: {}", format!("minifly machines create --app {} --image <image>", app_name).cyan());
            } else {
                let table_data: Vec<MachineRow> = machines.into_iter().map(|machine| MachineRow {
                    id: machine.id,
                    name: machine.name,
                    state: machine.state,
                    region: machine.region,
                    image: machine.config.image,
                    private_ip: machine.private_ip,
                }).collect();
                
                let table = Table::new(table_data);
                println!("{}", table);
            }
        }
        Err(e) => {
            println!("{}", format!("❌ Failed to list machines: {}", e).red());
        }
    }
    
    Ok(())
}

/// Create a new machine
pub async fn create(
    client: &ApiClient,
    app_name: &str,
    image: &str,
    name: Option<String>,
    region: Option<String>,
) -> Result<()> {
    if !client.health_check().await? {
        println!("{}", "❌ Minifly API server is not running".red());
        println!("Start it with: {}", "minifly serve".cyan());
        return Ok(());
    }
    
    println!("{}", format!("🚀 Creating machine for app '{}'...", app_name).blue());
    println!("  • Image: {}", image.cyan());
    if let Some(ref name) = name {
        println!("  • Name: {}", name.cyan());
    }
    if let Some(ref region) = region {
        println!("  • Region: {}", region.cyan());
    }
    
    match client.create_machine(app_name, image, name, region).await {
        Ok(machine) => {
            println!("{}", format!("✅ Machine '{}' created successfully!", machine.id).green());
            println!("  • ID: {}", machine.id.cyan());
            println!("  • State: {}", machine.state.yellow());
            println!("  • Region: {}", machine.region.cyan());
            println!("  • Private IP: {}", machine.private_ip.cyan());
        }
        Err(e) => {
            println!("{}", format!("❌ Failed to create machine: {}", e).red());
        }
    }
    
    Ok(())
}

/// Start a machine
pub async fn start(client: &ApiClient, machine_id: &str) -> Result<()> {
    if !client.health_check().await? {
        println!("{}", "❌ Minifly API server is not running".red());
        println!("Start it with: {}", "minifly serve".cyan());
        return Ok(());
    }
    
    println!("{}", format!("▶️  Starting machine '{}'...", machine_id).blue());
    
    match client.start_machine(machine_id).await {
        Ok(machine) => {
            println!("{}", format!("✅ Machine '{}' started successfully!", machine.id).green());
            println!("  • State: {}", machine.state.yellow());
        }
        Err(e) => {
            println!("{}", format!("❌ Failed to start machine: {}", e).red());
        }
    }
    
    Ok(())
}

/// Stop a machine
pub async fn stop(client: &ApiClient, machine_id: &str) -> Result<()> {
    if !client.health_check().await? {
        println!("{}", "❌ Minifly API server is not running".red());
        println!("Start it with: {}", "minifly serve".cyan());
        return Ok(());
    }
    
    println!("{}", format!("⏹️  Stopping machine '{}'...", machine_id).yellow());
    
    match client.stop_machine(machine_id).await {
        Ok(machine) => {
            println!("{}", format!("✅ Machine '{}' stopped successfully!", machine.id).green());
            println!("  • State: {}", machine.state.yellow());
        }
        Err(e) => {
            println!("{}", format!("❌ Failed to stop machine: {}", e).red());
        }
    }
    
    Ok(())
}

/// Delete a machine
pub async fn delete(client: &ApiClient, machine_id: &str, force: bool) -> Result<()> {
    if !client.health_check().await? {
        println!("{}", "❌ Minifly API server is not running".red());
        println!("Start it with: {}", "minifly serve".cyan());
        return Ok(());
    }
    
    let action = if force { "Force deleting" } else { "Deleting" };
    println!("{}", format!("🗑️  {} machine '{}'...", action, machine_id).yellow());
    
    match client.delete_machine(machine_id, force).await {
        Ok(_) => {
            println!("{}", format!("✅ Machine '{}' deleted successfully!", machine_id).green());
        }
        Err(e) => {
            println!("{}", format!("❌ Failed to delete machine: {}", e).red());
        }
    }
    
    Ok(())
}

#[derive(Tabled)]
struct MachineRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "State")]
    state: String,
    #[tabled(rename = "Region")]
    region: String,
    #[tabled(rename = "Image")]
    image: String,
    #[tabled(rename = "Private IP")]
    private_ip: String,
}