use crate::config::Config;
use crate::docker::DockerClient;
use anyhow::Result;
use minifly_core::models::{App, Machine, Lease};
use minifly_litefs::manager::LiteFSManager;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: SqlitePool,
    pub docker: DockerClient,
    pub litefs: Arc<LiteFSManager>,
    pub leases: Arc<RwLock<HashMap<String, Lease>>>,
    pub machines: Arc<RwLock<HashMap<String, Machine>>>,
    pub apps: Arc<RwLock<HashMap<String, App>>>,
    pub start_time: Instant,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self> {
        // Initialize database connection
        let db = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&config.database_url)
            .await?;
        
        // Run migrations
        sqlx::migrate!("./migrations").run(&db).await?;
        
        // Initialize Docker client
        let docker = DockerClient::new(config.docker_host.as_deref())?;
        
        // Initialize LiteFS manager
        let litefs_base_dir = PathBuf::from(&config.data_dir).join("litefs");
        let litefs = Arc::new(LiteFSManager::new(litefs_base_dir).await?);
        
        Ok(Self {
            config,
            db,
            docker,
            litefs,
            leases: Arc::new(RwLock::new(HashMap::new())),
            machines: Arc::new(RwLock::new(HashMap::new())),
            apps: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
        })
    }
    
    pub fn generate_machine_id(&self) -> String {
        // Generate a 15-character hex ID similar to Fly.io
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..8).map(|_| rng.gen()).collect();
        hex::encode(bytes)[..15].to_string()
    }
    
    pub fn generate_instance_id(&self) -> String {
        // Generate instance ID in Fly.io format
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let chars: String = (0..26)
            .map(|_| {
                let idx = rng.gen_range(0..36);
                if idx < 10 {
                    (b'0' + idx) as char
                } else {
                    (b'A' + idx - 10) as char
                }
            })
            .collect();
        chars
    }
    
    pub fn generate_private_ip(&self, app_id: &str, machine_index: u32) -> String {
        // Generate IPv6 address in Fly.io format: fdaa:0:app_id:a7b:machine_index::2
        let app_hash = {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(app_id);
            let result = hasher.finalize();
            format!("{:x}", u16::from_be_bytes([result[0], result[1]]))
        };
        
        format!("{}{}:a7b:{}::2", self.config.internal_network_prefix, app_hash, machine_index)
    }
}