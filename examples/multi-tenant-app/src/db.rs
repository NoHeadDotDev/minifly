use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, info};

// Cache for tenant database pools
static TENANT_POOLS: OnceCell<Arc<RwLock<HashMap<String, SqlitePool>>>> = OnceCell::new();

pub async fn init_db_directory(path: &str) -> Result<()> {
    fs::create_dir_all(path).await
        .context("Failed to create database directory")?;
    
    TENANT_POOLS.set(Arc::new(RwLock::new(HashMap::new())))
        .map_err(|_| anyhow::anyhow!("Failed to initialize tenant pools"))?;
    
    Ok(())
}

pub async fn get_tenant_db(tenant: &str, db_path: &str) -> Result<SqlitePool> {
    let pools = TENANT_POOLS.get()
        .ok_or_else(|| anyhow::anyhow!("Tenant pools not initialized"))?;
    
    // Check if pool already exists
    {
        let pools_read = pools.read().await;
        if let Some(pool) = pools_read.get(tenant) {
            debug!("Using existing pool for tenant: {}", tenant);
            return Ok(pool.clone());
        }
    }
    
    // Create new pool
    info!("Creating new database pool for tenant: {}", tenant);
    let db_file = format!("{}/{}.db", db_path, tenant);
    let db_url = format!("sqlite:{}", db_file);
    
    // Ensure database file exists
    if !Path::new(&db_file).exists() {
        fs::write(&db_file, b"").await
            .context("Failed to create database file")?;
    }
    
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .context("Failed to connect to database")?;
    
    // Run migrations
    run_migrations(&pool).await?;
    
    // Store in cache
    {
        let mut pools_write = pools.write().await;
        pools_write.insert(tenant.to_string(), pool.clone());
    }
    
    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    debug!("Running migrations");
    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS items (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await
    .context("Failed to create items table")?;
    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tenant_info (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            item_count INTEGER DEFAULT 0
        )
        "#,
    )
    .execute(pool)
    .await
    .context("Failed to create tenant_info table")?;
    
    // Insert tenant info if not exists
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO tenant_info (id, name) VALUES (1, ?)
        "#,
    )
    .bind("Default Tenant")
    .execute(pool)
    .await
    .context("Failed to insert tenant info")?;
    
    Ok(())
}

pub async fn list_all_tenants(db_path: &str) -> Result<Vec<String>> {
    let mut tenants = Vec::new();
    
    let mut entries = fs::read_dir(db_path).await
        .context("Failed to read database directory")?;
    
    while let Some(entry) = entries.next_entry().await? {
        if let Some(filename) = entry.file_name().to_str() {
            if filename.ends_with(".db") {
                let tenant_name = filename.trim_end_matches(".db");
                tenants.push(tenant_name.to_string());
            }
        }
    }
    
    Ok(tenants)
}