use anyhow::Result;
use axum::Router;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod handlers;
mod middleware;

use crate::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,multi_tenant_app=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Multi-Tenant Application");

    // Load configuration
    let config = Config::from_env()?;
    
    // Initialize database directory
    db::init_db_directory(&config.database_path).await?;
    
    // Build application routes
    let app = Router::new()
        .merge(handlers::routes())
        .layer(axum::middleware::from_fn(middleware::extract_tenant))
        .layer(TraceLayer::new_for_http())
        .with_state(config.clone());

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::Server::from_tcp(listener.into_std()?)?.serve(app.into_make_service()).await?;
    
    Ok(())
}