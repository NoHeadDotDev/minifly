//! Minifly CLI command implementations.
//!
//! This module contains all the command implementations for the Minifly CLI.
//! Each submodule represents a top-level command with its associated functionality.
//!
//! ## Commands
//!
//! - [`apps`] - Application management (create, list, delete)
//! - [`deploy`] - Application deployment with production config compatibility
//! - [`dev`] - Development mode with auto-reload
//! - [`init`] - Project initialization
//! - [`logs`] - Log viewing and streaming
//! - [`machines`] - Machine lifecycle management
//! - [`proxy`] - Service proxying
//! - [`secrets`] - Secrets management (.fly.secrets files)
//! - [`serve`] - Start the Minifly platform
//! - [`status`] - Platform status monitoring
//! - [`stop`] - Stop the platform

pub mod apps;
pub mod dependencies;
pub mod deploy;
pub mod dev;
pub mod init;
pub mod logs;
pub mod machines;
pub mod proxy;
pub mod secrets;
pub mod serve;
pub mod status;
pub mod stop;