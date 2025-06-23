mod auth;
mod db;
mod error;
mod handlers;
mod models;
mod state;
mod templates;
mod tenant;

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::{
    services::ServeDir,
    trace::TraceLayer,
};
use tower_sessions::{SessionManagerLayer, Expiry};
use tower_sessions_sqlx_store::SqliteStore;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::net::SocketAddr;

use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,todo_auth_app=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Get configuration
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:///litefs/app.db".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("Invalid PORT");

    // Initialize database
    let db = db::init_db(&database_url).await?;

    // Create session store
    let session_store = SqliteStore::new(db.clone());
    session_store.migrate().await?;

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Set to true in production with HTTPS
        .with_expiry(Expiry::OnInactivity(tower_sessions::cookie::time::Duration::hours(1)));

    // Create app state
    let state = AppState::new(db);

    // Build router
    let app = Router::new()
        // Public routes
        .route("/", get(handlers::home::index))
        .route("/login", get(handlers::auth::login_page).post(handlers::auth::login))
        .route("/signup", get(handlers::auth::signup_page).post(handlers::auth::signup))
        .route("/logout", post(handlers::auth::logout))
        
        // Protected routes
        .route("/dashboard", get(handlers::dashboard::dashboard))
        .route("/todos", post(handlers::todos::create_todo))
        .route("/todos/:id/toggle", post(handlers::todos::toggle_todo))
        .route("/todos/:id/delete", post(handlers::todos::delete_todo))
        .route("/todos/:id/image", post(handlers::todos::upload_image))
        .route("/region/:region", get(handlers::dashboard::region_dashboard))
        
        // Static files
        .nest_service("/static", ServeDir::new("static"))
        
        // Middleware
        .layer(session_layer)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Starting server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}