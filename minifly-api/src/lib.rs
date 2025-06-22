pub mod config;
pub mod docker;
pub mod error;
pub mod handlers;
pub mod health;
pub mod middleware;
pub mod state;

use axum::Router;
use tower_http::trace::TraceLayer;

pub use state::AppState;
pub use config::Config;

/// Create the main application router
pub fn create_app(state: AppState) -> Router {
    Router::new()
        .nest("/v1", handlers::routes())
        .layer(axum::middleware::from_fn(middleware::region::region_middleware))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}