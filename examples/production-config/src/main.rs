use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{sqlite::SqlitePool, Row};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    app_name: String,
    machine_id: String,
    region: String,
    private_ip: String,
    public_ip: String,
    port: String,
    environment: HashMap<String, String>,
}

#[derive(Serialize)]
struct SecretsResponse {
    loaded_secrets: Vec<String>,
    note: String,
}

#[derive(Serialize)]
struct VolumeResponse {
    volume_path: String,
    files: Vec<String>,
    write_test: String,
}

#[derive(Serialize)]
struct DiscoveryResponse {
    app_internal: String,
    machine_internal: String,
    consul_url: String,
    test_result: String,
}

#[derive(Serialize)]
struct DatabaseResponse {
    database_path: String,
    records_count: i64,
    operation_result: String,
}

#[derive(Deserialize)]
struct CreateRecordQuery {
    name: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Get port from environment (set by Fly.io or Minifly)
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;

    info!(
        "Starting production-app on port {} (app: {}, machine: {}, region: {})",
        port,
        env::var("FLY_APP_NAME").unwrap_or_else(|_| "unknown".to_string()),
        env::var("FLY_MACHINE_ID").unwrap_or_else(|_| "unknown".to_string()),
        env::var("FLY_REGION").unwrap_or_else(|_| "unknown".to_string())
    );

    // Initialize database if DATABASE_PATH is set (LiteFS)
    let db_pool = if let Ok(db_path) = env::var("DATABASE_PATH") {
        let db_url = format!("sqlite://{}/production.db", db_path);
        info!("Connecting to database at: {}", db_url);
        
        let pool = SqlitePool::connect(&db_url).await?;
        
        // Create table if it doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS records (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await?;
        
        Some(pool)
    } else {
        warn!("DATABASE_PATH not set, database endpoints will be unavailable");
        None
    };

    // Build routes
    let app = Router::new()
        .route("/", get(health))
        .route("/health", get(health))
        .route("/secrets", get(secrets))
        .route("/volumes", get(volumes))
        .route("/discover", get(discover))
        .route("/database", get(database_info))
        .route("/database/records", post(create_record))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(db_pool);

    info!("Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> Json<HealthResponse> {
    // Collect Fly.io environment variables
    let mut env_vars = HashMap::new();
    for (key, value) in env::vars() {
        if key.starts_with("FLY_") || key == "PORT" || key == "NODE_ENV" {
            env_vars.insert(key, value);
        }
    }

    Json(HealthResponse {
        status: "ok".to_string(),
        app_name: env::var("FLY_APP_NAME").unwrap_or_else(|_| "unknown".to_string()),
        machine_id: env::var("FLY_MACHINE_ID").unwrap_or_else(|_| "unknown".to_string()),
        region: env::var("FLY_REGION").unwrap_or_else(|_| "unknown".to_string()),
        private_ip: env::var("FLY_PRIVATE_IP").unwrap_or_else(|_| "unknown".to_string()),
        public_ip: env::var("FLY_PUBLIC_IP").unwrap_or_else(|_| "unknown".to_string()),
        port: env::var("PORT").unwrap_or_else(|_| "8080".to_string()),
        environment: env_vars,
    })
}

async fn secrets() -> Json<SecretsResponse> {
    // Collect secret keys (don't expose values)
    let secret_keys: Vec<String> = env::vars()
        .filter(|(key, _)| {
            key.contains("SECRET") || 
            key.contains("KEY") || 
            key.contains("TOKEN") || 
            key.contains("PASSWORD") ||
            key == "DATABASE_URL"
        })
        .map(|(key, _)| key)
        .collect();

    Json(SecretsResponse {
        loaded_secrets: secret_keys,
        note: "Secret values are redacted for security".to_string(),
    })
}

async fn volumes() -> Json<VolumeResponse> {
    let volume_path = "/data".to_string();
    
    // List files in volume
    let files = fs::read_dir(&volume_path)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.file_name().to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_else(|_| vec!["Volume not mounted".to_string()]);

    // Test write to volume
    let test_file = format!("{}/test-{}.txt", volume_path, Uuid::new_v4());
    let write_result = match fs::write(&test_file, "Minifly volume test") {
        Ok(_) => {
            let _ = fs::remove_file(&test_file); // Clean up
            "Successfully wrote to volume".to_string()
        }
        Err(e) => format!("Failed to write to volume: {}", e),
    };

    Json(VolumeResponse {
        volume_path,
        files,
        write_test: write_result,
    })
}

async fn discover() -> Json<DiscoveryResponse> {
    let app_name = env::var("FLY_APP_NAME").unwrap_or_else(|_| "production-app".to_string());
    let machine_id = env::var("FLY_MACHINE_ID").unwrap_or_else(|_| "unknown".to_string());
    
    // In production, these would resolve via Fly.io's internal DNS
    // In Minifly, these are handled by the internal DNS resolver
    let app_internal = format!("{}.internal", app_name);
    let machine_internal = format!("{}.vm.{}.internal", machine_id, app_name);
    let consul_url = env::var("FLY_CONSUL_URL").unwrap_or_else(|_| "http://localhost:8500".to_string());

    // Test DNS resolution (would work in real deployment)
    let test_result = "DNS resolution testing not implemented in example".to_string();

    Json(DiscoveryResponse {
        app_internal,
        machine_internal,
        consul_url,
        test_result,
    })
}

async fn database_info(
    axum::extract::State(db): axum::extract::State<Option<SqlitePool>>,
) -> Result<Json<DatabaseResponse>, StatusCode> {
    let Some(pool) = db else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let db_path = env::var("DATABASE_PATH").unwrap_or_else(|_| "/tmp".to_string());
    
    // Count records
    let count = sqlx::query("SELECT COUNT(*) as count FROM records")
        .fetch_one(&pool)
        .await
        .map(|row| row.get::<i64, _>("count"))
        .unwrap_or(0);

    Ok(Json(DatabaseResponse {
        database_path: db_path,
        records_count: count,
        operation_result: "Database connection successful".to_string(),
    }))
}

async fn create_record(
    axum::extract::State(db): axum::extract::State<Option<SqlitePool>>,
    Query(params): Query<CreateRecordQuery>,
) -> Result<Json<Value>, StatusCode> {
    let Some(pool) = db else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let name = params.name.unwrap_or_else(|| format!("Record-{}", Uuid::new_v4()));
    let id = Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO records (id, name) VALUES (?, ?)")
        .bind(&id)
        .bind(&name)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "id": id,
        "name": name,
        "message": "Record created successfully"
    })))
}