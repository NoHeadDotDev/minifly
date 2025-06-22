use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, error};
use crate::config::LiteFSConfig;
use minifly_core::Error;
use crate::Result;

pub struct LiteFSProcess {
    child: Option<Child>,
    config_path: PathBuf,
    binary_path: PathBuf,
    machine_id: String,
}

impl LiteFSProcess {
    pub fn new(machine_id: String, binary_path: PathBuf, config_path: PathBuf) -> Self {
        Self {
            child: None,
            config_path,
            binary_path,
            machine_id,
        }
    }
    
    pub fn start(&mut self) -> Result<()> {
        if self.child.is_some() {
            return Err(Error::LiteFSError("LiteFS process already running".to_string()));
        }
        
        info!("Starting LiteFS for machine {}", self.machine_id);
        
        let mut cmd = Command::new(&self.binary_path);
        cmd.arg("mount")
            .arg("-config")
            .arg(&self.config_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        match cmd.spawn() {
            Ok(child) => {
                info!("LiteFS process started with PID: {:?}", child.id());
                self.child = Some(child);
                Ok(())
            }
            Err(e) => {
                error!("Failed to start LiteFS: {}", e);
                Err(Error::LiteFSError(format!("Failed to start LiteFS: {}", e)))
            }
        }
    }
    
    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            info!("Stopping LiteFS process for machine {}", self.machine_id);
            
            match child.kill() {
                Ok(_) => {
                    match child.wait() {
                        Ok(status) => {
                            info!("LiteFS process stopped with status: {:?}", status);
                            Ok(())
                        }
                        Err(e) => {
                            warn!("Error waiting for LiteFS process to stop: {}", e);
                            Ok(())
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to stop LiteFS process: {}", e);
                    Err(Error::LiteFSError(format!("Failed to stop LiteFS: {}", e)))
                }
            }
        } else {
            Ok(())
        }
    }
    
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited
                    self.child = None;
                    false
                }
                Ok(None) => {
                    // Process is still running
                    true
                }
                Err(_) => {
                    // Error checking status, assume not running
                    self.child = None;
                    false
                }
            }
        } else {
            false
        }
    }
}

impl Drop for LiteFSProcess {
    fn drop(&mut self) {
        if let Err(e) = self.stop() {
            error!("Error stopping LiteFS process in drop: {}", e);
        }
    }
}

pub struct LiteFSProcessManager {
    processes: Arc<Mutex<std::collections::HashMap<String, LiteFSProcess>>>,
    binary_path: PathBuf,
}

impl LiteFSProcessManager {
    pub fn new(binary_path: PathBuf) -> Self {
        Self {
            processes: Arc::new(Mutex::new(std::collections::HashMap::new())),
            binary_path,
        }
    }
    
    pub async fn start_litefs(&self, machine_id: &str, config: &LiteFSConfig, config_dir: &PathBuf) -> Result<()> {
        let config_path = config_dir.join(format!("{}.yml", machine_id));
        
        // Write config file
        let yaml = config.to_yaml()
            .map_err(|e| Error::LiteFSError(format!("Failed to serialize config: {}", e)))?;
        
        tokio::fs::write(&config_path, yaml).await
            .map_err(|e| Error::LiteFSError(format!("Failed to write config: {}", e)))?;
        
        let mut processes = self.processes.lock().await;
        
        let mut process = LiteFSProcess::new(
            machine_id.to_string(),
            self.binary_path.clone(),
            config_path,
        );
        
        process.start()?;
        processes.insert(machine_id.to_string(), process);
        
        Ok(())
    }
    
    pub async fn stop_litefs(&self, machine_id: &str) -> Result<()> {
        let mut processes = self.processes.lock().await;
        
        if let Some(mut process) = processes.remove(machine_id) {
            process.stop()?;
        }
        
        Ok(())
    }
    
    pub async fn is_running(&self, machine_id: &str) -> bool {
        let mut processes = self.processes.lock().await;
        
        if let Some(process) = processes.get_mut(machine_id) {
            process.is_running()
        } else {
            false
        }
    }
    
    pub async fn stop_all(&self) -> Result<()> {
        let mut processes = self.processes.lock().await;
        
        for (machine_id, mut process) in processes.drain() {
            info!("Stopping LiteFS for machine {}", machine_id);
            if let Err(e) = process.stop() {
                error!("Failed to stop LiteFS for machine {}: {}", machine_id, e);
            }
        }
        
        Ok(())
    }
}