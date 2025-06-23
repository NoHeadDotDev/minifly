use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info};

use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize)]
struct CreateAppRequest {
    app_name: String,
    org_slug: String,
}

#[derive(Debug, Serialize)]
struct CreateMachineRequest {
    name: Option<String>,
    region: Option<String>,
    config: MachineConfig,
}

#[derive(Debug, Serialize)]
struct MachineConfig {
    image: String,
    guest: GuestConfig,
    env: HashMap<String, String>,
    services: Vec<ServiceConfig>,
    mounts: Vec<MountConfig>,
}

#[derive(Debug, Serialize)]
struct GuestConfig {
    cpu_kind: String,
    cpus: u32,
    memory_mb: u32,
}

#[derive(Debug, Serialize)]
struct ServiceConfig {
    ports: Vec<PortConfig>,
    protocol: String,
    internal_port: u16,
}

#[derive(Debug, Serialize)]
struct PortConfig {
    port: u16,
    handlers: Vec<String>,
}

#[derive(Debug, Serialize)]
struct MountConfig {
    volume: String,
    path: String,
}

#[derive(Debug, Deserialize)]
struct MachineResponse {
    id: String,
    state: String,
}

pub async fn provision_tenant_app(
    user_id: &str,
    email: &str,
    region: &str,
) -> AppResult<(String, String)> {
    let minifly_api_url = std::env::var("MINIFLY_API_URL")
        .unwrap_or_else(|_| "http://host.docker.internal:4280".to_string());
    
    let client = Client::new();
    
    // Generate unique app name for this user
    let app_name = format!("todo-user-{}", &user_id[..8]);
    
    info!("Provisioning tenant app {} for user {} in region {}", app_name, email, region);
    
    // Step 1: Create the app
    let create_app_url = format!("{}/v1/apps", minifly_api_url);
    let create_app_req = CreateAppRequest {
        app_name: app_name.clone(),
        org_slug: "personal".to_string(),
    };
    
    let response = client
        .post(&create_app_url)
        .json(&create_app_req)
        .send()
        .await
        .map_err(|e| AppError::TenantProvisioning(format!("Failed to create app: {}", e)))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        
        // If app already exists, that's okay
        if !status.as_u16() == 409 && !error_text.contains("already exists") {
            error!("Failed to create app: {} - {}", status, error_text);
            return Err(AppError::TenantProvisioning(format!("Failed to create app: {}", status)));
        }
    }
    
    // Step 2: Create a machine for the tenant
    let create_machine_url = format!("{}/v1/apps/{}/machines", minifly_api_url, app_name);
    
    let mut env = HashMap::new();
    env.insert("USER_ID".to_string(), user_id.to_string());
    env.insert("USER_EMAIL".to_string(), email.to_string());
    env.insert("TENANT_REGION".to_string(), region.to_string());
    env.insert("DATABASE_PATH".to_string(), "/data".to_string());
    
    let machine_config = MachineConfig {
        image: "ghcr.io/livebud/bud/sqlitedb:latest".to_string(), // Simple SQLite-based image
        guest: GuestConfig {
            cpu_kind: "shared".to_string(),
            cpus: 1,
            memory_mb: 256,
        },
        env,
        services: vec![ServiceConfig {
            ports: vec![PortConfig {
                port: 3000,
                handlers: vec!["http".to_string()],
            }],
            protocol: "tcp".to_string(),
            internal_port: 3000,
        }],
        mounts: vec![MountConfig {
            volume: format!("user_data_{}", user_id),
            path: "/data".to_string(),
        }],
    };
    
    let create_machine_req = CreateMachineRequest {
        name: Some(format!("{}-machine", app_name)),
        region: Some(region.to_string()),
        config: machine_config,
    };
    
    let response = client
        .post(&create_machine_url)
        .json(&create_machine_req)
        .send()
        .await
        .map_err(|e| AppError::TenantProvisioning(format!("Failed to create machine: {}", e)))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        error!("Failed to create machine: {} - {}", status, error_text);
        return Err(AppError::TenantProvisioning(format!("Failed to create machine: {}", status)));
    }
    
    let machine: MachineResponse = response
        .json()
        .await
        .map_err(|e| AppError::TenantProvisioning(format!("Failed to parse machine response: {}", e)))?;
    
    info!("Successfully provisioned tenant app {} with machine {}", app_name, machine.id);
    
    Ok((app_name, machine.id))
}

pub async fn get_tenant_app_url(app_name: &str) -> String {
    // In a real deployment, this would query the actual machine to get its URL
    // For local development, we'll construct a URL based on the app name
    format!("http://localhost/apps/{}", app_name)
}