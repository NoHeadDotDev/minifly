// Example of using Minifly to simulate a LiteFS cluster with primary/replica setup

use minifly_core::models::{CreateMachineRequest, MachineConfig};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Minifly LiteFS Cluster Example ===\n");
    
    // This example demonstrates:
    // 1. Creating a primary LiteFS node
    // 2. Creating replica LiteFS nodes
    // 3. Simulating database replication
    
    let client = reqwest::Client::new();
    let base_url = "http://localhost:4280/v1";
    
    // Create app
    println!("Creating app 'distributed-app'...");
    let app_request = serde_json::json!({
        "app_name": "distributed-app",
        "org_slug": "personal"
    });
    
    client
        .post(format!("{}/apps", base_url))
        .json(&app_request)
        .send()
        .await?;
    
    let mut machine_ids = Vec::new();
    
    // Create primary node
    println!("\nCreating PRIMARY node...");
    let mut env = HashMap::new();
    env.insert("DATABASE_URL".to_string(), "/litefs/app.db".to_string());
    env.insert("FLY_LITEFS_PRIMARY".to_string(), "true".to_string());
    env.insert("NODE_ROLE".to_string(), "primary".to_string());
    
    let primary_request = CreateMachineRequest {
        name: Some("distributed-app-primary".to_string()),
        region: Some("local".to_string()),
        config: MachineConfig {
            image: "alpine:latest".to_string(),
            env: Some(env),
            mounts: Some(vec![minifly_core::models::MountConfig {
                volume: "primary_data".to_string(),
                path: "/litefs".to_string(),
            }]),
            ..Default::default()
        },
        skip_launch: Some(false),
        lease_ttl: None,
    };
    
    let response = client
        .post(format!("{}/apps/distributed-app/machines", base_url))
        .json(&primary_request)
        .send()
        .await?;
    
    if response.status().is_success() {
        let machine: serde_json::Value = response.json().await?;
        let machine_id = machine["id"].as_str().unwrap_or("unknown").to_string();
        println!("✓ Primary node created: {}", machine_id);
        machine_ids.push(machine_id);
    }
    
    // Create replica nodes
    for i in 1..=2 {
        println!("\nCreating REPLICA node {}...", i);
        let mut env = HashMap::new();
        env.insert("DATABASE_URL".to_string(), "/litefs/app.db".to_string());
        env.insert("FLY_LITEFS_PRIMARY".to_string(), "false".to_string());
        env.insert("NODE_ROLE".to_string(), format!("replica-{}", i));
        
        let replica_request = CreateMachineRequest {
            name: Some(format!("distributed-app-replica-{}", i)),
            region: Some("local".to_string()),
            config: MachineConfig {
                image: "alpine:latest".to_string(),
                env: Some(env),
                mounts: Some(vec![minifly_core::models::MountConfig {
                    volume: format!("replica_data_{}", i),
                    path: "/litefs".to_string(),
                }]),
                ..Default::default()
            },
            skip_launch: Some(false),
            lease_ttl: None,
        };
        
        let response = client
            .post(format!("{}/apps/distributed-app/machines", base_url))
            .json(&replica_request)
            .send()
            .await?;
        
        if response.status().is_success() {
            let machine: serde_json::Value = response.json().await?;
            let machine_id = machine["id"].as_str().unwrap_or("unknown").to_string();
            println!("✓ Replica node {} created: {}", i, machine_id);
            machine_ids.push(machine_id);
        }
    }
    
    println!("\n=== LiteFS Cluster Setup Complete ===");
    println!("Primary: 1 node");
    println!("Replicas: 2 nodes");
    println!("\nIn a real setup, LiteFS would now:");
    println!("- Replicate SQLite changes from primary to replicas");
    println!("- Handle automatic failover if primary fails");
    println!("- Provide consistent reads across the cluster");
    
    // Cleanup
    println!("\nPress Enter to clean up resources...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    println!("Cleaning up...");
    for machine_id in machine_ids {
        client
            .delete(format!("{}/apps/distributed-app/machines/{}", base_url, machine_id))
            .send()
            .await?;
    }
    
    client
        .delete(format!("{}/apps/distributed-app", base_url))
        .send()
        .await?;
    
    println!("✓ Cleanup complete");
    
    Ok(())
}