use std::path::PathBuf;
use std::process::Command;
use tracing::{info, warn};
use reqwest;
use tokio::fs;
use crate::config::LiteFSConfig;
use crate::process::LiteFSProcessManager;
use minifly_core::Error;
use crate::Result;

const LITEFS_VERSION: &str = "development";

pub struct LiteFSManager {
    base_dir: PathBuf,
    binary_path: PathBuf,
    process_manager: LiteFSProcessManager,
}

impl LiteFSManager {
    pub async fn new(base_dir: PathBuf) -> Result<Self> {
        // Create directories
        fs::create_dir_all(&base_dir).await
            .map_err(|e| Error::LiteFSError(format!("Failed to create base dir: {}", e)))?;
        
        let bin_dir = base_dir.join("bin");
        fs::create_dir_all(&bin_dir).await
            .map_err(|e| Error::LiteFSError(format!("Failed to create bin dir: {}", e)))?;
        
        let binary_path = bin_dir.join("litefs");
        
        // Check if LiteFS binary exists
        let binary_exists = match Command::new(&binary_path)
            .arg("--version")
            .output() {
            Ok(output) => {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout);
                    info!("Found LiteFS binary: {}", version.trim());
                    true
                } else {
                    false
                }
            }
            Err(_) => false,
        };
        
        if !binary_exists {
            // Try system LiteFS first
            let system_litefs = match Command::new("litefs")
                .arg("--version")
                .output() {
                Ok(output) if output.status.success() => {
                    info!("Using system LiteFS binary");
                    Some(PathBuf::from("litefs"))
                }
                _ => None,
            };
            
            let final_binary_path = if let Some(path) = system_litefs {
                path
            } else {
                warn!("LiteFS binary not found. LiteFS features will be disabled.");
                warn!("To enable LiteFS, install it manually from: https://github.com/superfly/litefs/releases");
                // Use a dummy path, but LiteFS won't actually work
                PathBuf::from("litefs")
            };
            
            let process_manager = LiteFSProcessManager::new(final_binary_path.clone());
            
            Ok(Self {
                base_dir,
                binary_path: final_binary_path,
                process_manager,
            })
        } else {
            let process_manager = LiteFSProcessManager::new(binary_path.clone());
            
            Ok(Self {
                base_dir,
                binary_path,
                process_manager,
            })
        }
    }
    
    async fn download_litefs(target_path: &PathBuf) -> Result<()> {
        let arch = if cfg!(target_arch = "x86_64") {
            "amd64"
        } else if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            return Err(Error::LiteFSError("Unsupported architecture".to_string()));
        };
        
        let os = if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "macos") {
            return Err(Error::LiteFSError(
                "LiteFS doesn't provide macOS binaries. Please use Docker or a Linux VM for full LiteFS support.".to_string()
            ));
        } else {
            return Err(Error::LiteFSError("Unsupported OS".to_string()));
        };
        
        let url = format!(
            "https://github.com/superfly/litefs/releases/download/v{}/litefs-v{}-{}-{}.tar.gz",
            LITEFS_VERSION, LITEFS_VERSION, os, arch
        );
        
        info!("Downloading LiteFS from {}", url);
        
        // Download the tarball
        let response = reqwest::get(&url).await
            .map_err(|e| Error::LiteFSError(format!("Failed to download LiteFS: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(Error::LiteFSError(format!("Failed to download LiteFS: HTTP {}", response.status())));
        }
        
        let bytes = response.bytes().await
            .map_err(|e| Error::LiteFSError(format!("Failed to read response: {}", e)))?;
        
        // Save to temp file
        let temp_path = target_path.with_extension("tar.gz");
        fs::write(&temp_path, &bytes).await
            .map_err(|e| Error::LiteFSError(format!("Failed to write tarball: {}", e)))?;
        
        // Extract the binary
        let output = Command::new("tar")
            .arg("-xzf")
            .arg(&temp_path)
            .arg("-C")
            .arg(target_path.parent().unwrap())
            .arg("litefs")
            .output()
            .map_err(|e| Error::LiteFSError(format!("Failed to extract tarball: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::LiteFSError(format!("Failed to extract tarball: {}", stderr)));
        }
        
        // Clean up temp file
        let _ = fs::remove_file(&temp_path).await;
        
        // Make binary executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&target_path).await
                .map_err(|e| Error::LiteFSError(format!("Failed to get permissions: {}", e)))?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&target_path, perms).await
                .map_err(|e| Error::LiteFSError(format!("Failed to set permissions: {}", e)))?;
        }
        
        info!("LiteFS binary downloaded and installed");
        Ok(())
    }
    
    pub async fn start_for_machine(&self, machine_id: &str, is_primary: bool) -> Result<()> {
        // Check if we have a real LiteFS binary
        if self.binary_path == PathBuf::from("litefs") {
            // Check if litefs actually exists
            if Command::new("litefs").arg("--version").output().is_err() {
                warn!("Skipping LiteFS start for machine {} - LiteFS not installed", machine_id);
                return Ok(());
            }
        }
        
        let mount_dir = self.base_dir.join("mounts").join(machine_id);
        let data_dir = self.base_dir.join("data").join(machine_id);
        let config_dir = self.base_dir.join("configs");
        
        // Create directories
        fs::create_dir_all(&mount_dir).await
            .map_err(|e| Error::LiteFSError(format!("Failed to create mount dir: {}", e)))?;
        fs::create_dir_all(&data_dir).await
            .map_err(|e| Error::LiteFSError(format!("Failed to create data dir: {}", e)))?;
        fs::create_dir_all(&config_dir).await
            .map_err(|e| Error::LiteFSError(format!("Failed to create config dir: {}", e)))?;
        
        let config = LiteFSConfig::for_local_dev(machine_id, mount_dir, data_dir, is_primary);
        
        self.process_manager.start_litefs(machine_id, &config, &config_dir).await?;
        
        Ok(())
    }
    
    pub async fn stop_for_machine(&self, machine_id: &str) -> Result<()> {
        self.process_manager.stop_litefs(machine_id).await?;
        
        // Clean up mount point
        let mount_dir = self.base_dir.join("mounts").join(machine_id);
        if mount_dir.exists() {
            // Try to unmount first
            let _ = Command::new("umount")
                .arg(&mount_dir)
                .output();
            
            // Remove directory
            if let Err(e) = fs::remove_dir_all(&mount_dir).await {
                warn!("Failed to remove mount dir: {}", e);
            }
        }
        
        Ok(())
    }
    
    pub async fn is_running(&self, machine_id: &str) -> bool {
        self.process_manager.is_running(machine_id).await
    }
    
    pub async fn stop_all(&self) -> Result<()> {
        self.process_manager.stop_all().await?;
        
        // Clean up all mount points
        let mounts_dir = self.base_dir.join("mounts");
        if mounts_dir.exists() {
            if let Ok(mut entries) = fs::read_dir(&mounts_dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if path.is_dir() {
                        // Try to unmount
                        let _ = Command::new("umount")
                            .arg(&path)
                            .output();
                        
                        // Remove directory
                        if let Err(e) = fs::remove_dir_all(&path).await {
                            warn!("Failed to remove mount dir: {}", e);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    pub fn get_mount_path(&self, machine_id: &str) -> PathBuf {
        self.base_dir.join("mounts").join(machine_id)
    }
    
    pub fn get_proxy_url(&self, machine_id: &str) -> String {
        format!("http://{}:20202", machine_id)
    }
}