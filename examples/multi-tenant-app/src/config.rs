use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_path: String,
    pub port: u16,
    pub primary: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_path: std::env::var("DATABASE_PATH")
                .unwrap_or_else(|_| "./data".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
            primary: std::env::var("FLY_LITEFS_PRIMARY")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        })
    }
}