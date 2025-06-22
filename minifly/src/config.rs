//! Configuration management for Minifly CLI

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Minifly CLI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// API server URL
    pub api_url: String,
    /// Authentication token
    pub token: Option<String>,
    /// Default region for operations
    pub default_region: String,
    /// Timeout for API requests in seconds
    pub timeout: u64,
    /// Whether to verify SSL certificates
    pub verify_ssl: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:4280".to_string(),
            token: None,
            default_region: "local".to_string(),
            timeout: 30,
            verify_ssl: true,
        }
    }
}

impl Config {
    /// Load configuration from file or environment variables
    pub fn load() -> Result<Self> {
        let mut config = Self::default();
        
        // Try to load from config file first
        if let Ok(file_config) = Self::load_from_file() {
            config = file_config;
        }
        
        // Override with environment variables
        if let Ok(api_url) = std::env::var("MINIFLY_API_URL") {
            config.api_url = api_url;
        }
        
        if let Ok(token) = std::env::var("MINIFLY_TOKEN") {
            config.token = Some(token);
        }
        
        if let Ok(region) = std::env::var("MINIFLY_REGION") {
            config.default_region = region;
        }
        
        if let Ok(timeout) = std::env::var("MINIFLY_TIMEOUT") {
            config.timeout = timeout.parse().unwrap_or(30);
        }
        
        if let Ok(verify) = std::env::var("MINIFLY_VERIFY_SSL") {
            config.verify_ssl = verify.parse().unwrap_or(true);
        }
        
        Ok(config)
    }
    
    /// Load configuration from the config file
    fn load_from_file() -> Result<Self> {
        let config_path = Self::config_file_path()?;
        
        if !config_path.exists() {
            return Ok(Self::default());
        }
        
        let content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;
            
        let config: Self = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;
            
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;
        
        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }
        
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
            
        std::fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;
            
        Ok(())
    }
    
    /// Get the path to the config file
    fn config_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("minifly");
            
        Ok(config_dir.join("config.toml"))
    }
    
    /// Initialize configuration with default values
    pub fn init() -> Result<()> {
        let config = Self::default();
        config.save()?;
        
        println!("✅ Minifly configuration initialized at: {}", 
                 Self::config_file_path()?.display());
        println!("📝 Edit the config file to customize settings");
        
        Ok(())
    }
}