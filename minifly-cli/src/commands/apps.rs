use anyhow::Result;
use colored::*;
use tabled::{Table, Tabled};
use crate::client::ApiClient;

#[derive(Tabled)]
struct AppRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "STATUS")]
    status: String,
    #[tabled(rename = "ORGANIZATION")]
    organization: String,
    #[tabled(rename = "CREATED")]
    created_at: String,
}

pub async fn list(client: &ApiClient) -> Result<()> {
    let apps = client.list_apps().await?;
    
    if apps.is_empty() {
        println!("No apps found. Create one with: minifly apps create <name>");
        return Ok(());
    }
    
    let rows: Vec<AppRow> = apps.into_iter()
        .map(|app| AppRow {
            name: app.name,
            status: app.status,
            organization: app.organization.slug,
            created_at: app.created_at,
        })
        .collect();
    
    let table = Table::new(rows);
    println!("{}", table);
    
    Ok(())
}

pub async fn create(client: &ApiClient, name: &str) -> Result<()> {
    println!("Creating app {}...", name.yellow());
    
    let app = client.create_app(name).await?;
    
    println!("{}", "App created successfully!".green());
    println!("ID: {}", app.id);
    println!("Name: {}", app.name);
    println!("Organization: {}", app.organization.name);
    
    Ok(())
}

pub async fn delete(client: &ApiClient, name: &str) -> Result<()> {
    use dialoguer::Confirm;
    
    let confirm = Confirm::new()
        .with_prompt(format!("Are you sure you want to delete app '{}'?", name))
        .interact()?;
    
    if !confirm {
        println!("Deletion cancelled.");
        return Ok(());
    }
    
    println!("Deleting app {}...", name.yellow());
    
    client.delete_app(name).await?;
    
    println!("{}", "App deleted successfully!".green());
    
    Ok(())
}