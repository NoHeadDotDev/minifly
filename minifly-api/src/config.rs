use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub docker_host: Option<String>,
    pub data_dir: String,
    pub internal_network_prefix: String,
    pub dns_port: u16,
    pub litefs_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            port: std::env::var("MINIFLY_API_PORT")
                .unwrap_or_else(|_| "4280".to_string())
                .parse()?,
            database_url: std::env::var("MINIFLY_DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:minifly.db".to_string()),
            docker_host: std::env::var("DOCKER_HOST").ok(),
            data_dir: std::env::var("MINIFLY_DATA_DIR")
                .unwrap_or_else(|_| "./data".to_string()),
            internal_network_prefix: std::env::var("MINIFLY_NETWORK_PREFIX")
                .unwrap_or_else(|_| "fdaa:0:".to_string()),
            dns_port: std::env::var("MINIFLY_DNS_PORT")
                .unwrap_or_else(|_| "5353".to_string())
                .parse()?,
            litefs_port: std::env::var("MINIFLY_LITEFS_PORT")
                .unwrap_or_else(|_| "20202".to_string())
                .parse()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use serial_test::serial;
    
    #[test]
    #[serial]
    fn test_default_config() {
        // Clear relevant env vars
        env::remove_var("MINIFLY_API_PORT");
        env::remove_var("MINIFLY_DATABASE_URL");
        env::remove_var("DOCKER_HOST");
        env::remove_var("MINIFLY_DATA_DIR");
        env::remove_var("MINIFLY_NETWORK_PREFIX");
        env::remove_var("MINIFLY_DNS_PORT");
        env::remove_var("MINIFLY_LITEFS_PORT");
        
        let config = Config::from_env().unwrap();
        
        assert_eq!(config.port, 4280);
        assert_eq!(config.database_url, "sqlite:minifly.db");
        assert_eq!(config.docker_host, None);
        assert_eq!(config.data_dir, "./data");
        assert_eq!(config.internal_network_prefix, "fdaa:0:");
        assert_eq!(config.dns_port, 5353);
        assert_eq!(config.litefs_port, 20202);
    }
    
    #[test]
    #[serial]
    fn test_custom_config() {
        env::set_var("MINIFLY_API_PORT", "8080");
        env::set_var("MINIFLY_DATABASE_URL", "sqlite:test.db");
        env::set_var("DOCKER_HOST", "tcp://localhost:2375");
        env::set_var("MINIFLY_DATA_DIR", "/tmp/minifly");
        env::set_var("MINIFLY_NETWORK_PREFIX", "fd00::");
        env::set_var("MINIFLY_DNS_PORT", "5454");
        env::set_var("MINIFLY_LITEFS_PORT", "30303");
        
        let config = Config::from_env().unwrap();
        
        assert_eq!(config.port, 8080);
        assert_eq!(config.database_url, "sqlite:test.db");
        assert_eq!(config.docker_host, Some("tcp://localhost:2375".to_string()));
        assert_eq!(config.data_dir, "/tmp/minifly");
        assert_eq!(config.internal_network_prefix, "fd00::");
        assert_eq!(config.dns_port, 5454);
        assert_eq!(config.litefs_port, 30303);
        
        // Clean up
        env::remove_var("MINIFLY_API_PORT");
        env::remove_var("MINIFLY_DATABASE_URL");
        env::remove_var("DOCKER_HOST");
        env::remove_var("MINIFLY_DATA_DIR");
        env::remove_var("MINIFLY_NETWORK_PREFIX");
        env::remove_var("MINIFLY_DNS_PORT");
        env::remove_var("MINIFLY_LITEFS_PORT");
    }
    
    #[test]
    #[serial]
    fn test_invalid_port() {
        env::set_var("MINIFLY_API_PORT", "invalid");
        
        let result = Config::from_env();
        assert!(result.is_err());
        
        env::remove_var("MINIFLY_API_PORT");
    }
    
    #[test]
    #[serial]
    fn test_invalid_dns_port() {
        env::set_var("MINIFLY_DNS_PORT", "99999");
        
        let result = Config::from_env();
        assert!(result.is_err());
        
        env::remove_var("MINIFLY_DNS_PORT");
    }
    
    #[test]
    #[serial]
    fn test_partial_config() {
        env::set_var("MINIFLY_API_PORT", "9090");
        env::set_var("MINIFLY_DATA_DIR", "/custom/data");
        
        let config = Config::from_env().unwrap();
        
        // Custom values
        assert_eq!(config.port, 9090);
        assert_eq!(config.data_dir, "/custom/data");
        
        // Default values
        assert_eq!(config.database_url, "sqlite:minifly.db");
        assert_eq!(config.docker_host, None);
        assert_eq!(config.internal_network_prefix, "fdaa:0:");
        assert_eq!(config.dns_port, 5353);
        assert_eq!(config.litefs_port, 20202);
        
        env::remove_var("MINIFLY_API_PORT");
        env::remove_var("MINIFLY_DATA_DIR");
    }
}