//! # Minifly
//!
//! Local Fly.io development simulator with incredible developer experience.
//!
//! Minifly provides a complete local development environment that simulates the Fly.io platform,
//! allowing you to develop, test, and debug your applications with the same APIs and behavior
//! you'll see in production.
//!
//! ## Features
//!
//! - **Complete Fly.io API Compatibility** - Full Machines API with Docker integration
//! - **LiteFS Integration** - Distributed SQLite with local replication testing  
//! - **Incredible Developer Experience** - Hot reloading, watch mode, structured logging
//! - **Multi-region Simulation** - Test region-specific behavior locally
//! - **Real-time Monitoring** - Comprehensive status dashboards and logging
//! - **Docker Management** - Automatic container lifecycle management
//!
//! ## Quick Start
//!
//! ```bash
//! # Install minifly
//! cargo install minifly
//!
//! # Initialize and start
//! minifly init
//! minifly serve
//!
//! # Deploy your first app
//! minifly deploy
//! ```
//!
//! ## Examples
//!
//! ### Basic Usage
//!
//! ```rust,no_run
//! use minifly::{Config, ApiClient};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration
//!     let config = Config::load()?;
//!     
//!     // Create API client
//!     let client = ApiClient::new(&config)?;
//!     
//!     // Create an application
//!     client.create_app("my-app").await?;
//!     
//!     // Deploy a machine
//!     let machine = client.create_machine("my-app", "nginx:latest", None, None).await?;
//!     println!("Created machine: {}", machine.id);
//!     
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod client;
pub mod logging;
pub mod types;

pub use config::Config;
pub use client::ApiClient;
pub use types::*;