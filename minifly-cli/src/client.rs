use anyhow::{Context, Result};
use minifly_core::models::*;
use reqwest::{Client, header};
use crate::config::Config;

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    pub fn new(config: &Config) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        
        if let Some(token) = &config.token {
            headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&format!("Bearer {}", token))?,
            );
        }
        
        let client = Client::builder()
            .default_headers(headers)
            .build()?;
        
        Ok(Self {
            client,
            base_url: config.api_url.clone(),
        })
    }
    
    // Apps API
    pub async fn list_apps(&self) -> Result<Vec<AppResponse>> {
        let resp = self.client
            .get(format!("{}/v1/apps", self.base_url))
            .send()
            .await?;
        
        resp.json().await.context("Failed to parse response")
    }
    
    pub async fn create_app(&self, name: &str) -> Result<AppResponse> {
        let req = CreateAppRequest {
            app_name: name.to_string(),
            org_slug: "personal".to_string(),
        };
        
        let resp = self.client
            .post(format!("{}/v1/apps", self.base_url))
            .json(&req)
            .send()
            .await?;
        
        resp.json().await.context("Failed to parse response")
    }
    
    pub async fn delete_app(&self, name: &str) -> Result<()> {
        self.client
            .delete(format!("{}/v1/apps/{}", self.base_url, name))
            .send()
            .await?;
        
        Ok(())
    }
    
    // Machines API
    pub async fn list_machines(&self, app_name: &str) -> Result<Vec<Machine>> {
        let resp = self.client
            .get(format!("{}/v1/apps/{}/machines", self.base_url, app_name))
            .send()
            .await?;
        
        resp.json().await.context("Failed to parse response")
    }
    
    pub async fn create_machine(
        &self,
        app_name: &str,
        image: &str,
        name: Option<String>,
        region: Option<String>,
    ) -> Result<Machine> {
        let req = CreateMachineRequest {
            name,
            region,
            config: MachineConfig {
                image: image.to_string(),
                guest: GuestConfig {
                    cpu_kind: "shared".to_string(),
                    cpus: 1,
                    memory_mb: 256,
                    gpu_kind: None,
                    gpus: None,
                    kernel_args: None,
                },
                env: None,
                services: None,
                checks: None,
                restart: Some(RestartConfig {
                    policy: "on-failure".to_string(),
                    max_retries: Some(3),
                }),
                auto_destroy: None,
                dns: None,
                processes: None,
                files: None,
                init: None,
                mounts: None,
                containers: None,
            },
            skip_launch: None,
            skip_service_registration: None,
            lease_ttl: None,
        };
        
        let resp = self.client
            .post(format!("{}/v1/apps/{}/machines", self.base_url, app_name))
            .json(&req)
            .send()
            .await?;
        
        resp.json().await.context("Failed to parse response")
    }
    
    pub async fn start_machine(&self, app_name: &str, machine_id: &str) -> Result<StartMachineResponse> {
        let resp = self.client
            .post(format!("{}/v1/apps/{}/machines/{}/start", self.base_url, app_name, machine_id))
            .send()
            .await?;
        
        resp.json().await.context("Failed to parse response")
    }
    
    pub async fn stop_machine(&self, app_name: &str, machine_id: &str) -> Result<StopMachineResponse> {
        let resp = self.client
            .post(format!("{}/v1/apps/{}/machines/{}/stop", self.base_url, app_name, machine_id))
            .send()
            .await?;
        
        resp.json().await.context("Failed to parse response")
    }
    
    pub async fn delete_machine(&self, app_name: &str, machine_id: &str, force: bool) -> Result<()> {
        let url = if force {
            format!("{}/v1/apps/{}/machines/{}?force=true", self.base_url, app_name, machine_id)
        } else {
            format!("{}/v1/apps/{}/machines/{}", self.base_url, app_name, machine_id)
        };
        
        self.client.delete(url).send().await?;
        
        Ok(())
    }
    
    pub async fn get_machine_app(&self, _machine_id: &str) -> Result<String> {
        // This is a simplified implementation
        // In reality, we'd need to track machine -> app mapping
        Ok("default-app".to_string())
    }
    
    // Generic HTTP methods for deploy command
    pub async fn get(&self, path: &str) -> Result<reqwest::Response> {
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{}/v1{}", self.base_url, path)
        };
        
        self.client.get(url).send().await
            .context("Failed to send GET request")
    }
    
    pub async fn post<T: serde::Serialize>(&self, path: &str, body: &T) -> Result<reqwest::Response> {
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{}/v1{}", self.base_url, path)
        };
        
        self.client.post(url)
            .json(body)
            .send()
            .await
            .context("Failed to send POST request")
    }
    
    /// Check if the API server is healthy
    pub async fn health_check(&self) -> Result<bool> {
        let resp = self.client
            .get(format!("{}/health", self.base_url))
            .send()
            .await;
            
        match resp {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}