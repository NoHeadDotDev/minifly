pub mod config;
pub mod manager;
pub mod process;
pub mod server;

use minifly_core::Error;

pub type Result<T> = std::result::Result<T, Error>;