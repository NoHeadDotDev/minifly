//! Network simulation and service discovery for Minifly.
//!
//! This crate provides networking functionality to simulate Fly.io's internal
//! networking features in local development environments.
//!
//! ## Features
//!
//! - **DNS Resolution**: Resolve `.internal` domains for service discovery
//! - **Container IP Extraction**: Extract IP addresses from Docker containers
//! - **Service Registration**: Automatic registration of machines with DNS
//!
//! ## Example
//!
//! ```rust
//! use minifly_network::{InternalDnsResolver, extract_container_ip};
//! use std::net::{IpAddr, Ipv4Addr};
//!
//! # tokio_test::block_on(async {
//! // Create a DNS resolver
//! let resolver = InternalDnsResolver::new();
//! let ip = IpAddr::V4(Ipv4Addr::new(172, 19, 0, 2));
//!
//! // Register a machine
//! resolver.register_machine("myapp", "machine-1", ip).await.unwrap();
//!
//! // Resolve app.internal
//! let ips = resolver.resolve("myapp.internal").await.unwrap();
//! assert_eq!(ips, vec![ip]);
//! # });
//! ```

pub mod dns;

pub use dns::{InternalDnsResolver, extract_container_ip};