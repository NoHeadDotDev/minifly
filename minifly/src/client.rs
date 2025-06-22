//! HTTP client for communicating with the Minifly API

use anyhow::{Context, Result};
use reqwest::{Client, ClientBuilder};
use serde::de::DeserializeOwned;
use std::time::Duration;

use crate::{Config, App, Machine, CreateMachineRequest};

/// HTTP client for the Minifly API
#[derive(Debug, Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

impl ApiClient {
    /// Create a new API client
    pub fn new(config: &Config) -> Result<Self> {
        let mut builder = ClientBuilder::new()
            .timeout(Duration::from_secs(config.timeout))
            .user_agent(format!("minifly-cli/{}", env!("CARGO_PKG_VERSION")));
            
        if !config.verify_ssl {
            builder = builder.danger_accept_invalid_certs(true);
        }
        
        let client = builder.build()
            .context("Failed to create HTTP client")?;
            
        Ok(Self {
            client,
            base_url: config.api_url.trim_end_matches('/').to_string(),
            token: config.token.clone(),
        })
    }
    
    /// Check if the API server is running
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
    
    /// List all applications
    pub async fn list_apps(&self) -> Result<Vec<App>> {
        self.get("/v1/apps").await
    }
    
    /// Create a new application
    pub async fn create_app(&self, name: &str) -> Result<App> {
        let body = serde_json::json!({ "app_name": name });
        self.post("/v1/apps", &body).await
    }
    
    /// Delete an application
    pub async fn delete_app(&self, name: &str) -> Result<()> {
        let url = format!("/v1/apps/{}", name);
        self.delete(&url).await
    }
    
    /// List machines for an application
    pub async fn list_machines(&self, app_name: &str) -> Result<Vec<Machine>> {
        let url = format!("/v1/apps/{}/machines", app_name);
        self.get(&url).await
    }
    
    /// Create a new machine
    pub async fn create_machine(
        &self,
        app_name: &str,
        image: &str,
        name: Option<String>,
        region: Option<String>,
    ) -> Result<Machine> {
        use crate::types::{MachineConfig, Guest, RestartPolicy};
        use std::collections::HashMap;
        
        let config = MachineConfig {
            image: image.to_string(),
            env: HashMap::new(),
            services: vec![],
            guest: Guest::default(),
            restart: RestartPolicy::default(),
            auto_destroy: false,
            kill_timeout: Some(5),
        };
        
        let request = CreateMachineRequest {
            name,
            config,
            region,
            skip_launch: false,
            skip_service_registration: false,
        };
        
        let url = format!("/v1/apps/{}/machines", app_name);
        self.post(&url, &request).await
    }
    
    /// Start a machine
    pub async fn start_machine(&self, machine_id: &str) -> Result<Machine> {
        let url = format!("/v1/machines/{}/start", machine_id);
        self.post(&url, &serde_json::json!({})).await
    }
    
    /// Stop a machine
    pub async fn stop_machine(&self, machine_id: &str) -> Result<Machine> {
        let url = format!("/v1/machines/{}/stop", machine_id);
        self.post(&url, &serde_json::json!({})).await
    }
    
    /// Delete a machine
    pub async fn delete_machine(&self, machine_id: &str, force: bool) -> Result<()> {
        let url = if force {
            format!("/v1/machines/{}?force=true", machine_id)
        } else {
            format!("/v1/machines/{}", machine_id)
        };
        self.delete(&url).await
    }
    
    /// Get machine details
    pub async fn get_machine(&self, machine_id: &str) -> Result<Machine> {
        let url = format!("/v1/machines/{}", machine_id);
        self.get(&url).await
    }
    
    /// Generic GET request
    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.get(&url);
        
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }
        
        let response = request.send().await
            .with_context(|| format!("Failed to send GET request to {}", url))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, text);
        }
        
        let result = response.json().await
            .with_context(|| format!("Failed to parse JSON response from {}", url))?;
            
        Ok(result)
    }
    
    /// Generic POST request
    async fn post<T: DeserializeOwned>(&self, path: &str, body: &impl serde::Serialize) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.post(&url).json(body);
        
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }
        
        let response = request.send().await
            .with_context(|| format!("Failed to send POST request to {}", url))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, text);
        }
        
        let result = response.json().await
            .with_context(|| format!("Failed to parse JSON response from {}", url))?;
            
        Ok(result)
    }
    
    /// Generic DELETE request
    async fn delete(&self, path: &str) -> Result<()> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.delete(&url);
        
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }
        
        let response = request.send().await
            .with_context(|| format!("Failed to send DELETE request to {}", url))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, text);
        }
        
        Ok(())
    }
}