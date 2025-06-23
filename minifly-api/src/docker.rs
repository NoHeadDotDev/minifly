use anyhow::{Context, Result};
use bollard::{
    Docker,
    container::{Config as ContainerConfig, CreateContainerOptions, StartContainerOptions},
    image::CreateImageOptions,
    service::{HostConfig, PortBinding, RestartPolicy, RestartPolicyNameEnum, Mount, MountTypeEnum},
};
use futures::StreamExt;
use minifly_core::models::{MachineConfig, GuestConfig, MountConfig};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, error, info};

#[derive(Clone)]
pub struct DockerClient {
    client: Docker,
}

impl DockerClient {
    pub fn new(docker_host: Option<&str>) -> Result<Self> {
        let client = if let Some(host) = docker_host {
            Docker::connect_with_socket(host, 120, bollard::API_DEFAULT_VERSION)?
        } else {
            Docker::connect_with_local_defaults()?
        };
        
        Ok(Self { client })
    }
    
    pub async fn create_container(
        &self,
        machine_id: &str,
        app_name: &str,
        config: &MachineConfig,
    ) -> Result<String> {
        info!("Creating container for machine {}", machine_id);
        
        // Pull image if needed
        self.pull_image(&config.image).await?;
        
        // Build container configuration
        let container_config = self.build_container_config(machine_id, app_name, config).await?;
        
        // Create container
        let options = CreateContainerOptions {
            name: format!("minifly-{}-{}", app_name, machine_id),
            ..Default::default()
        };
        
        let response = self.client
            .create_container(Some(options), container_config)
            .await
            .context("Failed to create container")?;
        
        Ok(response.id)
    }
    
    pub async fn start_container(&self, container_id: &str) -> Result<()> {
        info!("Starting container {}", container_id);
        
        self.client
            .start_container(container_id, None::<StartContainerOptions<String>>)
            .await
            .context("Failed to start container")?;
        
        Ok(())
    }
    
    pub async fn stop_container(&self, container_id: &str, timeout: Option<i64>) -> Result<()> {
        info!("Stopping container {}", container_id);
        
        let options = bollard::container::StopContainerOptions {
            t: timeout.unwrap_or(30),
        };
        
        self.client
            .stop_container(container_id, Some(options))
            .await
            .context("Failed to stop container")?;
        
        Ok(())
    }
    
    pub async fn remove_container(&self, container_id: &str) -> Result<()> {
        info!("Removing container {}", container_id);
        
        let options = bollard::container::RemoveContainerOptions {
            force: true,
            ..Default::default()
        };
        
        self.client
            .remove_container(container_id, Some(options))
            .await
            .context("Failed to remove container")?;
        
        Ok(())
    }
    
    pub async fn inspect_container(&self, container_id: &str) -> Result<bollard::models::ContainerInspectResponse> {
        self.client
            .inspect_container(container_id, None)
            .await
            .context("Failed to inspect container")
    }
    
    /// Get the assigned host ports for a container
    /// 
    /// Since we use automatic port allocation (port 0), Docker assigns ephemeral ports.
    /// This function retrieves the actual assigned ports after container creation.
    /// 
    /// # Arguments
    /// * `container_id` - The Docker container ID
    /// 
    /// # Returns
    /// * `Ok(Vec<u16>)` - List of assigned host ports
    /// * `Err(...)` - Failed to inspect container
    pub async fn get_container_ports(&self, container_id: &str) -> Result<Vec<u16>> {
        let container_info = self.inspect_container(container_id).await?;
        let mut ports = Vec::new();
        
        if let Some(network_settings) = &container_info.network_settings {
            if let Some(port_bindings) = &network_settings.ports {
                for (_, bindings) in port_bindings {
                    if let Some(bindings) = bindings {
                        for binding in bindings {
                            if let Some(host_port) = &binding.host_port {
                                if let Ok(port) = host_port.parse::<u16>() {
                                    ports.push(port);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        ports.sort();
        Ok(ports)
    }
    
    /// Get Docker daemon version information
    pub async fn version(&self) -> Result<bollard::system::Version> {
        self.client
            .version()
            .await
            .context("Failed to get Docker version")
    }
    
    /// List containers with optional filters
    pub async fn list_containers(&self, filters: Option<HashMap<String, Vec<String>>>) -> Result<Vec<bollard::models::ContainerSummary>> {
        let options = bollard::container::ListContainersOptions {
            all: true,
            filters: filters.unwrap_or_default(),
            ..Default::default()
        };
        
        self.client
            .list_containers(Some(options))
            .await
            .context("Failed to list containers")
    }
    
    /// Stream logs from a container
    pub async fn stream_logs(
        &self, 
        container_id: &str,
        follow: bool,
        tail: Option<String>,
        timestamps: bool,
    ) -> Result<impl futures::Stream<Item = Result<bollard::container::LogOutput, bollard::errors::Error>>> {
        use bollard::container::LogsOptions;
        
        let options = LogsOptions::<String> {
            follow,
            stdout: true,
            stderr: true,
            timestamps,
            tail: tail.unwrap_or_default(),
            ..Default::default()
        };
        
        Ok(self.client.logs(container_id, Some(options)))
    }
    
    /// Get container ID by machine ID
    pub async fn get_container_id_by_machine(&self, machine_id: &str) -> Result<Option<String>> {
        let mut filters = HashMap::new();
        filters.insert("label".to_string(), vec![format!("minifly.machine_id={}", machine_id)]);
        
        let containers = self.list_containers(Some(filters)).await?;
        
        Ok(containers.into_iter()
            .next()
            .and_then(|c| c.id))
    }
    
    async fn pull_image(&self, image: &str) -> Result<()> {
        // Skip pulling for local images (those ending with :latest and containing 'local')
        if image.contains("-local:") || image.ends_with("-local:latest") {
            info!("Skipping pull for local image: {}", image);
            return Ok(());
        }
        
        info!("Pulling image: {}", image);
        
        let options = CreateImageOptions {
            from_image: image,
            ..Default::default()
        };
        
        let mut stream = self.client.create_image(Some(options), None, None);
        
        while let Some(result) = stream.next().await {
            match result {
                Ok(info) => debug!("Pull progress: {:?}", info),
                Err(e) => {
                    error!("Error pulling image: {}", e);
                    return Err(e.into());
                }
            }
        }
        
        Ok(())
    }
    
    async fn build_container_config(
        &self,
        machine_id: &str,
        app_name: &str,
        config: &MachineConfig,
    ) -> Result<ContainerConfig<String>> {
        let mut labels = HashMap::new();
        labels.insert("minifly.managed".to_string(), "true".to_string());
        labels.insert("minifly.machine_id".to_string(), machine_id.to_string());
        labels.insert("minifly.app_name".to_string(), app_name.to_string());
        labels.insert("minifly.region".to_string(), "local".to_string());
        
        let mut container_config = ContainerConfig::<String> {
            image: Some(config.image.clone()),
            hostname: Some(format!("{}.vm.{}.internal", machine_id, app_name)),
            labels: Some(labels),
            ..Default::default()
        };
        
        // Set environment variables with Fly.io translations
        let mut env_vars = config.env.clone().unwrap_or_default();
        self.translate_fly_env_vars(&mut env_vars, app_name, machine_id);
        
        // Load and inject secrets
        if let Ok(secrets) = self.load_secrets(app_name).await {
            for (key, value) in secrets {
                env_vars.insert(key, value);
            }
        }
        
        let env_vec: Vec<String> = env_vars.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        container_config.env = Some(env_vec);
        
        // Set command
        if let Some(init) = &config.init {
            if let Some(exec) = &init.exec {
                container_config.cmd = Some(exec.clone());
            } else if let Some(entrypoint) = &init.entrypoint {
                container_config.entrypoint = Some(entrypoint.clone());
                if let Some(cmd) = &init.cmd {
                    container_config.cmd = Some(cmd.clone());
                }
            }
        }
        
        // Build host configuration
        let mut host_config = HostConfig::default();
        
        // Set resource limits
        self.set_resource_limits(&mut host_config, &config.guest);
        
        // Set restart policy
        if let Some(restart) = &config.restart {
            host_config.restart_policy = Some(RestartPolicy {
                name: Some(match restart.policy.as_str() {
                    "always" => RestartPolicyNameEnum::ALWAYS,
                    "on-failure" => RestartPolicyNameEnum::ON_FAILURE,
                    "unless-stopped" => RestartPolicyNameEnum::UNLESS_STOPPED,
                    _ => RestartPolicyNameEnum::NO,
                }),
                maximum_retry_count: restart.max_retries.map(|n| n as i64),
            });
        }
        
        // Set port bindings with automatic port allocation for local development
        // This prevents port conflicts when running multiple apps or when ports are already in use
        if let Some(services) = &config.services {
            let mut port_bindings = HashMap::new();
            
            for service in services {
                let internal_port = format!("{}/tcp", service.internal_port);
                
                // For local development, use automatic port allocation (port 0)
                // Docker will assign an available ephemeral port (typically 32768-65535)
                // This avoids conflicts with other services like web servers on port 80/443
                let binding = PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some("0".to_string()), // Let Docker assign an available port
                };
                
                port_bindings.insert(internal_port, Some(vec![binding]));
            }
            
            host_config.port_bindings = Some(port_bindings);
        }
        
        // Set volume mounts
        if let Some(mounts) = &config.mounts {
            host_config.mounts = Some(self.map_fly_volumes(mounts, app_name)?);
        }
        
        container_config.host_config = Some(host_config);
        
        Ok(container_config)
    }
    
    fn set_resource_limits(&self, host_config: &mut HostConfig, guest: &GuestConfig) {
        // Set CPU limits
        match guest.cpu_kind.as_str() {
            "shared" => {
                // For shared CPUs, use CPU shares (relative weight)
                host_config.cpu_shares = Some((guest.cpus * 1024) as i64);
            }
            "performance" => {
                // For performance CPUs, use CPU quota
                host_config.cpu_period = Some(100000);
                host_config.cpu_quota = Some((guest.cpus as i64) * 100000);
            }
            _ => {}
        }
        
        // Set memory limit
        host_config.memory = Some((guest.memory_mb as i64) * 1024 * 1024);
    }
    
    /// Translate Fly.io-specific environment variables to minifly equivalents
    fn translate_fly_env_vars(&self, env: &mut HashMap<String, String>, app_name: &str, machine_id: &str) {
        // Core Fly.io environment variables
        env.insert("FLY_APP_NAME".to_string(), app_name.to_string());
        env.insert("FLY_MACHINE_ID".to_string(), machine_id.to_string());
        env.insert("FLY_REGION".to_string(), "local".to_string());
        env.insert("FLY_PUBLIC_IP".to_string(), "127.0.0.1".to_string());
        
        // Generate a consistent private IP based on machine ID
        let machine_suffix = machine_id.chars()
            .filter(|c| c.is_numeric())
            .take(3)
            .collect::<String>()
            .parse::<u8>()
            .unwrap_or(2);
        env.insert("FLY_PRIVATE_IP".to_string(), format!("172.19.0.{}", machine_suffix));
        
        // Simulate Fly's internal DNS and services
        env.insert("FLY_CONSUL_URL".to_string(), "http://localhost:8500".to_string());
        env.insert("PRIMARY_REGION".to_string(), "local".to_string());
        
        // If using Tigris/S3, point to local MinIO (if configured)
        if env.contains_key("TIGRIS_ENDPOINT") || env.contains_key("AWS_ENDPOINT_URL") {
            env.insert("TIGRIS_ENDPOINT".to_string(), "http://localhost:9000".to_string());
            env.insert("AWS_ENDPOINT_URL".to_string(), "http://localhost:9000".to_string());
            env.insert("AWS_ENDPOINT_URL_S3".to_string(), "http://localhost:9000".to_string());
        }
        
        // Add helpful development overrides
        if !env.contains_key("NODE_ENV") && !env.contains_key("RAILS_ENV") {
            env.insert("NODE_ENV".to_string(), "development".to_string());
        }
    }
    
    /// Load secrets from .fly.secrets files for the specified application.
    /// 
    /// This function implements a hierarchical secrets loading system:
    /// 1. First loads app-specific secrets from `.fly.secrets.<app_name>`
    /// 2. Then loads default secrets from `.fly.secrets`
    /// 3. App-specific secrets take precedence over default secrets
    async fn load_secrets(&self, app_name: &str) -> Result<HashMap<String, String>> {
        use std::path::Path;
        use tokio::fs;
        
        let mut secrets = HashMap::new();
        
        // Try app-specific secrets file first
        let app_secrets_file = format!(".fly.secrets.{}", app_name);
        if Path::new(&app_secrets_file).exists() {
            let contents = fs::read_to_string(&app_secrets_file).await
                .context(format!("Failed to read {}", app_secrets_file))?;
            self.parse_secrets(&contents, &mut secrets)?;
        }
        
        // Then load default secrets file
        let default_secrets_file = ".fly.secrets";
        if Path::new(default_secrets_file).exists() {
            let contents = fs::read_to_string(default_secrets_file).await
                .context("Failed to read .fly.secrets")?;
            // Don't overwrite app-specific secrets
            let mut default_secrets = HashMap::new();
            self.parse_secrets(&contents, &mut default_secrets)?;
            for (k, v) in default_secrets {
                secrets.entry(k).or_insert(v);
            }
        }
        
        Ok(secrets)
    }

    /// Parse secrets from file contents in KEY=VALUE format
    fn parse_secrets(&self, contents: &str, secrets: &mut HashMap<String, String>) -> Result<()> {
        use anyhow::bail;
        
        for (line_num, line) in contents.lines().enumerate() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Parse KEY=VALUE
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim();
                let value = line[pos + 1..].trim();
                
                if key.is_empty() {
                    bail!("Empty key at line {}", line_num + 1);
                }
                
                // Remove quotes if present
                let value = if (value.starts_with('"') && value.ends_with('"')) ||
                            (value.starts_with('\'') && value.ends_with('\'')) {
                    &value[1..value.len() - 1]
                } else {
                    value
                };
                
                secrets.insert(key.to_string(), value.to_string());
            } else {
                bail!("Invalid format at line {} - expected KEY=VALUE", line_num + 1);
            }
        }
        
        Ok(())
    }

    /// Map Fly volumes to local directories
    fn map_fly_volumes(&self, mounts: &[MountConfig], app_name: &str) -> Result<Vec<Mount>> {
        mounts.iter().map(|mount| {
            // Use absolute path based on data directory (reuse existing minifly-data structure)
            let base_path = if let Ok(data_dir) = std::env::var("MINIFLY_DATA_DIR") {
                PathBuf::from(data_dir)
            } else {
                // Default to /tmp for volumes if no data dir specified
                PathBuf::from("/tmp")
            };
            
            // Keep the existing minifly-data structure for compatibility
            let local_path = base_path.join("minifly-data").join(app_name).join("volumes").join(&mount.volume);
            
            // Ensure directory exists
            std::fs::create_dir_all(&local_path)
                .context(format!("Failed to create volume directory: {:?}", local_path))?;
            
            // Create database file if it's a SQLite database path
            if mount.path == "/litefs" || mount.path.contains("data") {
                let db_file = local_path.join("app.db");
                if !db_file.exists() {
                    std::fs::File::create(&db_file)
                        .context(format!("Failed to create database file: {:?}", db_file))?;
                    info!("Created database file: {:?}", db_file);
                }
            }
            
            Ok(Mount {
                target: Some(mount.path.clone()),
                source: Some(local_path.to_string_lossy().to_string()),
                typ: Some(MountTypeEnum::BIND),
                read_only: Some(false),
                consistency: Some("consistent".to_string()),
                ..Default::default()
            })
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use minifly_core::models::{ServiceConfig, PortConfig};
    
    #[test]
    fn test_translate_fly_env_vars() {
        let client = DockerClient { client: Docker::connect_with_local_defaults().unwrap() };
        let mut env = HashMap::new();
        
        client.translate_fly_env_vars(&mut env, "test-app", "d123456789");
        
        assert_eq!(env.get("FLY_APP_NAME").unwrap(), "test-app");
        assert_eq!(env.get("FLY_MACHINE_ID").unwrap(), "d123456789");
        assert_eq!(env.get("FLY_REGION").unwrap(), "local");
        assert_eq!(env.get("FLY_PUBLIC_IP").unwrap(), "127.0.0.1");
        assert!(env.contains_key("FLY_PRIVATE_IP"));
        assert_eq!(env.get("FLY_CONSUL_URL").unwrap(), "http://localhost:8500");
        assert_eq!(env.get("PRIMARY_REGION").unwrap(), "local");
        assert_eq!(env.get("NODE_ENV").unwrap(), "development");
    }
    
    #[test]
    fn test_translate_fly_env_vars_with_tigris() {
        let client = DockerClient { client: Docker::connect_with_local_defaults().unwrap() };
        let mut env = HashMap::new();
        env.insert("TIGRIS_ENDPOINT".to_string(), "https://fly.storage.tigris.dev".to_string());
        
        client.translate_fly_env_vars(&mut env, "test-app", "d123456789");
        
        assert_eq!(env.get("TIGRIS_ENDPOINT").unwrap(), "http://localhost:9000");
        assert_eq!(env.get("AWS_ENDPOINT_URL").unwrap(), "http://localhost:9000");
        assert_eq!(env.get("AWS_ENDPOINT_URL_S3").unwrap(), "http://localhost:9000");
    }
    
    #[test]
    fn test_translate_fly_env_vars_preserves_existing_node_env() {
        let client = DockerClient { client: Docker::connect_with_local_defaults().unwrap() };
        let mut env = HashMap::new();
        env.insert("NODE_ENV".to_string(), "production".to_string());
        
        client.translate_fly_env_vars(&mut env, "test-app", "d123456789");
        
        assert_eq!(env.get("NODE_ENV").unwrap(), "production");
    }
    
    #[tokio::test]
    async fn test_build_container_config_uses_automatic_port_allocation() {
        let client = DockerClient { client: Docker::connect_with_local_defaults().unwrap() };
        
        let config = MachineConfig {
            image: "nginx:alpine".to_string(),
            guest: GuestConfig {
                cpu_kind: "shared".to_string(),
                cpus: 1,
                memory_mb: 256,
                gpu_kind: None,
                gpus: None,
                kernel_args: None,
            },
            env: None,
            services: Some(vec![ServiceConfig {
                ports: vec![
                    PortConfig {
                        port: 80,
                        handlers: vec!["http".to_string()],
                        force_https: Some(false),
                        tls_options: None,
                    },
                    PortConfig {
                        port: 443,
                        handlers: vec!["tls".to_string(), "http".to_string()],
                        force_https: Some(false),
                        tls_options: None,
                    },
                ],
                protocol: "tcp".to_string(),
                internal_port: 80,
                autostop: None,
                autostart: None,
                force_instance_description: None,
            }]),
            checks: None,
            restart: None,
            auto_destroy: None,
            dns: None,
            processes: None,
            files: None,
            init: None,
            mounts: None,
            containers: None,
        };
        
        let container_config = client.build_container_config("test-machine", "test-app", &config).await.unwrap();
        
        // Check that host config has port bindings
        let host_config = container_config.host_config.unwrap();
        let port_bindings = host_config.port_bindings.unwrap();
        
        // Check that port 80/tcp is mapped
        assert!(port_bindings.contains_key("80/tcp"));
        let bindings = port_bindings.get("80/tcp").unwrap().as_ref().unwrap();
        
        // Should have exactly one binding (not two like before)
        assert_eq!(bindings.len(), 1);
        
        // Check that it uses automatic port allocation (port "0")
        let binding = &bindings[0];
        assert_eq!(binding.host_ip.as_ref().unwrap(), "0.0.0.0");
        assert_eq!(binding.host_port.as_ref().unwrap(), "0");
    }
}