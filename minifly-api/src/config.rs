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