use anyhow::Result;
use axum::Router;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::info;
use minifly_logging::{LoggingConfig, LogFormat};

mod config;
mod docker;
mod error;
mod handlers;
mod health;
mod middleware;
mod state;

// use middleware::region; // Used via middleware::region in the layer

use crate::config::Config;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging
    let logging_config = LoggingConfig::from_env("minifly-api")
        .with_level("minifly_api=debug,tower_http=debug,minifly_logging=info");
    
    minifly_logging::init_logging(logging_config)?;

    info!(
        service = "minifly-api",
        version = env!("CARGO_PKG_VERSION"),
        "Starting Minifly API server"
    );

    // Load configuration
    let config = Config::from_env()?;
    
    // Initialize application state
    let state = AppState::new(config.clone()).await?;
    
    // Build our application with routes
    let app = Router::new()
        .nest("/v1", handlers::routes())
        .layer(axum::middleware::from_fn(middleware::region::region_middleware))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Bind to address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!(
        server.address = %addr,
        server.port = config.port,
        "API server binding to address"
    );
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}