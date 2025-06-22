//! LiteFS configuration management for Minifly.
//!
//! This module provides types and functions for managing LiteFS configurations,
//! including automatic adaptation of production configurations for local development.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main LiteFS configuration structure.
/// 
/// This represents the complete LiteFS configuration that can be loaded from
/// a `litefs.yml` file. It supports both production and development configurations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteFSConfig {
    // Mount configuration
    pub fuse: FuseConfig,
    
    // Data directory
    pub data: DataConfig,
    
    // HTTP proxy configuration
    pub proxy: Option<ProxyConfig>,
    
    // Lease configuration
    pub lease: LeaseConfig,
    
    // Logging configuration
    pub log: Option<LogConfig>,
    
    // Consul configuration (for production)
    pub consul: Option<ConsulConfig>,
    
    // Static configuration (for local development)
    #[serde(rename = "static")]
    pub static_config: Option<StaticConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuseConfig {
    pub dir: PathBuf,
    #[serde(default = "default_debug")]
    pub debug: bool,
    #[serde(default = "default_allow_other")]
    pub allow_other: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConfig {
    pub dir: PathBuf,
    #[serde(default = "default_compress")]
    pub compress: bool,
    #[serde(default = "default_retention")]
    pub retention: String,
    #[serde(default = "default_retention_monitor_interval")]
    pub retention_monitor_interval: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    #[serde(default = "default_proxy_addr")]
    pub addr: String,
    pub target: String,
    pub db: String,
    #[serde(default = "default_passthrough")]
    pub passthrough: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseConfig {
    #[serde(rename = "type", default = "default_lease_type")]
    pub lease_type: String,
    pub advertise_url: Option<String>,
    pub candidate: Option<bool>,
    pub promote: Option<bool>,
    pub demote: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    #[serde(default = "default_log_format")]
    pub format: String,
    #[serde(default = "default_log_level")]
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsulConfig {
    pub url: String,
    pub advertise_url: String,
    pub key: Option<String>,
    pub ttl: Option<String>,
    pub lock_ttl: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticConfig {
    pub primary: bool,
    pub hostname: String,
    pub advertise_url: String,
}

// Default functions for serde
fn default_debug() -> bool { false }
fn default_allow_other() -> bool { false }
fn default_compress() -> bool { true }
fn default_retention() -> String { "24h".to_string() }
fn default_retention_monitor_interval() -> String { "1h".to_string() }
fn default_proxy_addr() -> String { ":20202".to_string() }
fn default_passthrough() -> Vec<String> { vec![] }
fn default_lease_type() -> String { "static".to_string() }
fn default_log_format() -> String { "text".to_string() }
fn default_log_level() -> String { "info".to_string() }

impl Default for LiteFSConfig {
    fn default() -> Self {
        Self {
            fuse: FuseConfig {
                dir: PathBuf::from("/litefs"),
                debug: false,
                allow_other: false,
            },
            data: DataConfig {
                dir: PathBuf::from("/var/lib/litefs"),
                compress: true,
                retention: "24h".to_string(),
                retention_monitor_interval: "1h".to_string(),
            },
            proxy: Some(ProxyConfig {
                addr: ":20202".to_string(),
                target: "localhost:8080".to_string(),
                db: "db".to_string(),
                passthrough: vec![],
            }),
            lease: LeaseConfig {
                lease_type: "static".to_string(),
                advertise_url: None,
                candidate: Some(true),
                promote: Some(true),
                demote: Some(false),
            },
            log: Some(LogConfig {
                format: "text".to_string(),
                level: "info".to_string(),
            }),
            consul: None,
            static_config: Some(StaticConfig {
                primary: true,
                hostname: "localhost".to_string(),
                advertise_url: "http://localhost:20202".to_string(),
            }),
        }
    }
}

impl LiteFSConfig {
    pub fn for_local_dev(machine_id: &str, mount_dir: PathBuf, data_dir: PathBuf, is_primary: bool) -> Self {
        Self {
            fuse: FuseConfig {
                dir: mount_dir,
                debug: true,
                allow_other: true,
            },
            data: DataConfig {
                dir: data_dir,
                compress: true,
                retention: "24h".to_string(),
                retention_monitor_interval: "1h".to_string(),
            },
            proxy: Some(ProxyConfig {
                addr: ":20202".to_string(),
                target: "localhost:8080".to_string(),
                db: "db".to_string(),
                passthrough: vec![],
            }),
            lease: LeaseConfig {
                lease_type: "static".to_string(),
                advertise_url: Some(format!("http://{}:20202", machine_id)),
                candidate: Some(is_primary),
                promote: Some(is_primary),
                demote: Some(false),
            },
            log: Some(LogConfig {
                format: "text".to_string(),
                level: "debug".to_string(),
            }),
            consul: None,
            static_config: Some(StaticConfig {
                primary: is_primary,
                hostname: machine_id.to_string(),
                advertise_url: format!("http://{}:20202", machine_id),
            }),
        }
    }
    
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }
    
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }
    
    /// Creates a local development configuration from a production litefs.yml.
    /// 
    /// This function automatically adapts production LiteFS configurations for local
    /// development by:
    /// 
    /// - Converting Consul lease to static lease
    /// - Adjusting file paths to local directories
    /// - Enabling debug mode and allow_other for FUSE
    /// - Setting up static primary configuration
    /// - Removing Consul-specific settings
    /// 
    /// # Arguments
    /// 
    /// * `content` - The raw YAML content of the production litefs.yml
    /// * `machine_id` - Unique machine identifier for this instance
    /// * `app_name` - Name of the application
    /// 
    /// # Returns
    /// 
    /// A `LiteFSConfig` adapted for local development.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use minifly_litefs::config::LiteFSConfig;
    /// 
    /// let production_yaml = r#"
    /// fuse:
    ///   dir: "/litefs"
    /// data:
    ///   dir: "/var/lib/litefs"
    /// lease:
    ///   type: "consul"
    /// "#;
    /// 
    /// let config = LiteFSConfig::from_production_config(
    ///     production_yaml,
    ///     "machine-123",
    ///     "myapp"
    /// ).unwrap();
    /// 
    /// assert_eq!(config.lease.lease_type, "static");
    /// ```
    pub fn from_production_config(content: &str, machine_id: &str, app_name: &str) -> Result<Self, anyhow::Error> {
        let mut config: LiteFSConfig = serde_yaml::from_str(content)?;
        
        // Override lease configuration for local development
        if config.lease.lease_type == "consul" {
            // Use static lease locally instead of consul
            config.lease.lease_type = "static".to_string();
            config.lease.candidate = Some(true);
            config.lease.promote = Some(true);
            config.lease.advertise_url = Some(format!("http://{}:20202", machine_id));
        }
        
        // Adjust paths to local environment
        let base_path = PathBuf::from(format!("./minifly-data/{}/litefs/{}", app_name, machine_id));
        config.fuse.dir = base_path.join("mount");
        config.data.dir = base_path.join("data");
        
        // Enable debug mode for local development
        config.fuse.debug = true;
        config.fuse.allow_other = true;
        
        // Update log level
        if let Some(ref mut log) = config.log {
            log.level = "debug".to_string();
        }
        
        // Set static configuration for local primary
        config.static_config = Some(StaticConfig {
            primary: true,
            hostname: machine_id.to_string(),
            advertise_url: format!("http://{}:20202", machine_id),
        });
        
        // Clear consul config as it's not used locally
        config.consul = None;
        
        Ok(config)
    }
}