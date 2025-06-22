// Example of using Minifly to simulate Fly.io locally

use minifly_core::models::{CreateMachineRequest, MachineConfig, PortConfig, ServiceConfig};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Minifly Basic Usage Example ===\n");
    
    // This example shows how to:
    // 1. Create an app
    // 2. Create a machine with LiteFS volume
    // 3. Start/stop the machine
    
    // Note: Make sure the Minifly API server is running:
    // cargo run --bin minifly-api
    
    let client = reqwest::Client::new();
    let base_url = "http://localhost:4280/v1";
    
    // 1. Create an app
    println!("1. Creating app 'my-app'...");
    let app_request = serde_json::json!({
        "app_name": "my-app",
        "org_slug": "personal"
    });
    
    let response = client
        .post(format!("{}/apps", base_url))
        .json(&app_request)
        .send()
        .await?;
    
    if response.status().is_success() {
        println!("   ✓ App created successfully");
    } else {
        println!("   ✗ Failed to create app: {}", response.text().await?);
        return Ok(());
    }
    
    // 2. Create a machine with SQLite database and LiteFS
    println!("\n2. Creating machine with LiteFS volume...");
    
    let mut env = HashMap::new();
    env.insert("DATABASE_URL".to_string(), "/litefs/db.sqlite".to_string());
    env.insert("FLY_LITEFS_PRIMARY".to_string(), "true".to_string());
    
    let machine_request = CreateMachineRequest {
        name: Some("my-app-1".to_string()),
        region: Some("local".to_string()),
        config: MachineConfig {
            image: "alpine:latest".to_string(),
            size: Some("shared-cpu-1x".to_string()),
            env: Some(env),
            services: Some(vec![ServiceConfig {
                ports: vec![PortConfig {
                    port: 8080,
                    handlers: Some(vec!["http".to_string()]),
                    force_https: Some(false),
                }],
                protocol: "tcp".to_string(),
                internal_port: 8080,
            }]),
            mounts: Some(vec![minifly_core::models::MountConfig {
                volume: "sqlite_data".to_string(),
                path: "/litefs".to_string(),
            }]),
            ..Default::default()
        },
        skip_launch: Some(false),
        lease_ttl: None,
    };
    
    let response = client
        .post(format!("{}/apps/my-app/machines", base_url))
        .json(&machine_request)
        .send()
        .await?;
    
    if response.status().is_success() {
        let machine: serde_json::Value = response.json().await?;
        let machine_id = machine["id"].as_str().unwrap_or("unknown");
        println!("   ✓ Machine created with ID: {}", machine_id);
        println!("   ✓ LiteFS is running for SQLite replication");
        
        // 3. Check machine status
        println!("\n3. Checking machine status...");
        let response = client
            .get(format!("{}/apps/my-app/machines/{}", base_url, machine_id))
            .send()
            .await?;
        
        if response.status().is_success() {
            let machine: serde_json::Value = response.json().await?;
            println!("   Machine state: {}", machine["state"]);
            println!("   Private IP: {}", machine["private_ip"]);
        }
        
        // 4. Stop the machine
        println!("\n4. Stopping machine...");
        let response = client
            .post(format!("{}/apps/my-app/machines/{}/stop", base_url, machine_id))
            .send()
            .await?;
        
        if response.status().is_success() {
            println!("   ✓ Machine stopped successfully");
        }
        
        // 5. Delete the machine
        println!("\n5. Cleaning up - deleting machine...");
        let response = client
            .delete(format!("{}/apps/my-app/machines/{}", base_url, machine_id))
            .send()
            .await?;
        
        if response.status().is_success() {
            println!("   ✓ Machine deleted successfully");
        }
        
    } else {
        println!("   ✗ Failed to create machine: {}", response.text().await?);
    }
    
    // 6. Delete the app
    println!("\n6. Cleaning up - deleting app...");
    let response = client
        .delete(format!("{}/apps/my-app", base_url))
        .send()
        .await?;
    
    if response.status().is_success() {
        println!("   ✓ App deleted successfully");
    }
    
    println!("\n=== Example completed ===");
    
    Ok(())
}