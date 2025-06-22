use anyhow::{Context, Result};
use bollard::{
    Docker,
    container::{Config as ContainerConfig, CreateContainerOptions, StartContainerOptions},
    image::CreateImageOptions,
    service::{HostConfig, PortBinding, RestartPolicy, RestartPolicyNameEnum},
};
use futures::StreamExt;
use minifly_core::models::{MachineConfig, GuestConfig};
use std::collections::HashMap;
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
        let container_config = self.build_container_config(machine_id, app_name, config)?;
        
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
    
    fn build_container_config(
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
        
        // Set environment variables
        if let Some(env) = &config.env {
            let env_vec: Vec<String> = env.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            container_config.env = Some(env_vec);
        }
        
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
        
        // Set port bindings
        if let Some(services) = &config.services {
            let mut port_bindings = HashMap::new();
            
            for service in services {
                let internal_port = format!("{}/tcp", service.internal_port);
                let mut bindings = vec![];
                
                for port_config in &service.ports {
                    bindings.push(PortBinding {
                        host_ip: Some("0.0.0.0".to_string()),
                        host_port: Some(port_config.port.to_string()),
                    });
                }
                
                port_bindings.insert(internal_port, Some(bindings));
            }
            
            host_config.port_bindings = Some(port_bindings);
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
}