//! Core types for Minifly
//!
//! This module contains all the fundamental data structures that represent the Fly.io API
//! surface within Minifly. These types are used throughout the system for API communication,
//! configuration management, and internal data representation.
//!
//! # Architecture
//!
//! The type hierarchy follows the Fly.io platform structure:
//! - [`App`] - Top-level application containers
//! - [`Machine`] - Individual compute instances within apps
//! - [`MachineConfig`] - Configuration for machine behavior and resources
//! - [`Service`] - Network service definitions
//! - [`Guest`] - Resource allocation specifications
//!
//! # Usage Patterns
//!
//! Most types are designed to be:
//! - **Serializable** - Can be converted to/from JSON for API communication
//! - **Cloneable** - Can be safely duplicated for concurrent operations
//! - **Debuggable** - Provide useful output for troubleshooting
//!
//! # Examples
//!
//! ```rust
//! use minifly::{App, Machine, MachineConfig, Guest};
//! use std::collections::HashMap;
//!
//! // Create a basic application
//! let app = App {
//!     name: "my-web-app".to_string(),
//!     organization: "my-org".to_string(),
//!     status: "deployed".to_string(),
//!     deployed: true,
//!     hostname: "my-web-app.fly.dev".to_string(),
//!     app_url: "https://my-web-app.fly.dev".to_string(),
//!     platform_version: "v2".to_string(),
//! };
//!
//! // Create a machine configuration
//! let config = MachineConfig {
//!     image: "nginx:latest".to_string(),
//!     env: HashMap::new(),
//!     guest: Guest::default(), // 1 vCPU, 256MB RAM
//!     ..MachineConfig::default()
//! };
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a Fly.io application with deployment status and configuration.
///
/// Applications are the top-level containers for machines and services in the Fly.io
/// platform. Each app has a unique name within an organization and can contain
/// multiple machines across different regions.
///
/// # Fields
///
/// - `name` - Unique application identifier within the organization
/// - `organization` - Organization/account that owns this application
/// - `status` - Current deployment status (e.g., "deployed", "pending", "suspended")
/// - `deployed` - Whether the application has been successfully deployed
/// - `hostname` - Primary hostname for accessing the application
/// - `app_url` - Full URL where the application is accessible
/// - `platform_version` - Fly.io platform version being used
///
/// # Examples
///
/// ```rust
/// use minifly::App;
/// 
/// let app = App {
///     name: "my-web-app".to_string(),
///     organization: "acme-corp".to_string(),
///     status: "deployed".to_string(),
///     deployed: true,
///     hostname: "my-web-app.fly.dev".to_string(),
///     app_url: "https://my-web-app.fly.dev".to_string(),
///     platform_version: "v2".to_string(),
/// };
///
/// assert_eq!(app.name, "my-web-app");
/// assert!(app.deployed);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    /// Unique application name within the organization
    pub name: String,
    /// Organization that owns this application
    pub organization: String,
    /// Current deployment status
    pub status: String,
    /// Whether the application has been deployed
    pub deployed: bool,
    /// Primary hostname for the application
    pub hostname: String,
    /// Full URL where the application is accessible
    pub app_url: String,
    /// Fly.io platform version being used
    pub platform_version: String,
}

/// Represents a Fly.io machine (compute instance) with full lifecycle information.
///
/// Machines are individual compute instances that run within applications. Each machine
/// has its own configuration, state, and can be managed independently. Machines can be
/// started, stopped, and updated, and they maintain an audit trail of events.
///
/// # Lifecycle States
///
/// Machines progress through various states during their lifecycle:
/// - `created` - Machine has been defined but not started
/// - `starting` - Machine is in the process of starting up
/// - `started` - Machine is running and ready to serve traffic
/// - `stopping` - Machine is gracefully shutting down
/// - `stopped` - Machine has been stopped
/// - `destroying` - Machine is being deleted
/// - `destroyed` - Machine has been permanently removed
///
/// # Fields
///
/// - `id` - Unique identifier for this machine
/// - `name` - Human-readable name for the machine
/// - `state` - Current lifecycle state (see [`MachineState`])
/// - `region` - Geographic region where the machine is running
/// - `instance_id` - Platform-specific instance identifier
/// - `private_ip` - Internal IP address for machine communication
/// - `config` - Complete machine configuration (see [`MachineConfig`])
/// - `image_ref` - Container image information (see [`ImageRef`])
/// - `created_at` - ISO8601 timestamp when the machine was created
/// - `updated_at` - ISO8601 timestamp when the machine was last modified
/// - `events` - Chronological list of machine lifecycle events
///
/// # Examples
///
/// ```rust
/// use minifly::{Machine, MachineConfig, ImageRef, MachineEvent};
/// use std::collections::HashMap;
/// 
/// let machine = Machine {
///     id: "e28657f9c050e8".to_string(),
///     name: "web-server-1".to_string(),
///     state: "started".to_string(),
///     region: "sjc".to_string(),
///     instance_id: "01H9EXAMPLE".to_string(),
///     private_ip: "fdaa:0:1::3".to_string(),
///     config: MachineConfig::default(),
///     image_ref: ImageRef {
///         registry: "registry.fly.io".to_string(),
///         repository: "my-app".to_string(),
///         tag: "latest".to_string(),
///         digest: "sha256:abc123...".to_string(),
///         labels: HashMap::new(),
///     },
///     created_at: "2024-06-22T10:30:00Z".to_string(),
///     updated_at: "2024-06-22T10:31:00Z".to_string(),
///     events: vec![],
/// };
///
/// assert_eq!(machine.state, "started");
/// assert_eq!(machine.region, "sjc");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Machine {
    /// Unique identifier for this machine
    pub id: String,
    /// Human-readable name for the machine
    pub name: String,
    /// Current lifecycle state (see MachineState enum)
    pub state: String,
    /// Geographic region where the machine is running
    pub region: String,
    /// Platform-specific instance identifier
    pub instance_id: String,
    /// Internal IPv6 address for machine communication
    pub private_ip: String,
    /// Complete machine configuration
    pub config: MachineConfig,
    /// Container image information and metadata
    pub image_ref: ImageRef,
    /// ISO8601 timestamp when the machine was created
    pub created_at: String,
    /// ISO8601 timestamp when the machine was last modified
    pub updated_at: String,
    /// Chronological list of machine lifecycle events
    pub events: Vec<MachineEvent>,
}

/// Complete configuration for a machine including resources, networking, and behavior.
///
/// MachineConfig defines all aspects of how a machine should be configured and behave,
/// including the container image, resource allocation, environment variables, networking,
/// and restart policies. This configuration is used when creating or updating machines.
///
/// # Configuration Categories
///
/// - **Image**: Container image and runtime configuration
/// - **Resources**: CPU, memory, and other compute resources (see [`Guest`])
/// - **Networking**: Service definitions and port mappings (see [`Service`])
/// - **Environment**: Environment variables and runtime settings
/// - **Lifecycle**: Restart policies and auto-destruction behavior
///
/// # Examples
///
/// ```rust
/// use minifly::{MachineConfig, Guest, Service, RestartPolicy};
/// use std::collections::HashMap;
///
/// // Basic web server configuration
/// let mut env = HashMap::new();
/// env.insert("PORT".to_string(), "8080".to_string());
/// env.insert("NODE_ENV".to_string(), "production".to_string());
///
/// let config = MachineConfig {
///     image: "node:18-alpine".to_string(),
///     env,
///     guest: Guest {
///         cpu_kind: "shared".to_string(),
///         cpus: 1,
///         memory_mb: 512,
///         kernel_args: None,
///     },
///     services: vec![],
///     restart: RestartPolicy {
///         policy: "on-failure".to_string(),
///         max_retries: 3,
///     },
///     auto_destroy: false,
///     kill_timeout: Some(30),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineConfig {
    /// Container image to run (e.g., "nginx:latest", "registry.fly.io/my-app:v1.0.0")
    pub image: String,
    /// Environment variables to set in the container
    pub env: HashMap<String, String>,
    /// Network service definitions and port mappings
    pub services: Vec<Service>,
    /// Resource allocation (CPU, memory) configuration
    pub guest: Guest,
    /// Restart behavior when the container exits
    pub restart: RestartPolicy,
    /// Whether to automatically destroy the machine when it stops
    pub auto_destroy: bool,
    /// Maximum time in seconds to wait for graceful shutdown before killing
    pub kill_timeout: Option<i32>,
}

/// Guest configuration for machine resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guest {
    pub cpu_kind: String,
    pub cpus: i32,
    pub memory_mb: i32,
    pub kernel_args: Option<Vec<String>>,
}

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub protocol: String,
    pub internal_port: i32,
    pub ports: Vec<Port>,
    pub force_https: bool,
    pub auto_stop_machines: bool,
    pub auto_start_machines: bool,
    pub min_machines_running: i32,
}

/// Port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub port: i32,
    pub handlers: Vec<String>,
    pub force_https: bool,
}

/// Restart policy for machines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartPolicy {
    pub policy: String,
    pub max_retries: i32,
}

/// Image reference information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRef {
    pub registry: String,
    pub repository: String,
    pub tag: String,
    pub digest: String,
    pub labels: HashMap<String, String>,
}

/// Machine event for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineEvent {
    pub id: String,
    pub type_: String,
    pub status: String,
    pub source: String,
    pub timestamp: i64,
    pub request: HashMap<String, serde_json::Value>,
}

/// Request to create a new machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMachineRequest {
    pub name: Option<String>,
    pub config: MachineConfig,
    pub region: Option<String>,
    pub skip_launch: bool,
    pub skip_service_registration: bool,
}

/// Request to start a machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartMachineRequest {
    pub timeout: Option<i32>,
}

/// Request to stop a machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopMachineRequest {
    pub timeout: Option<i32>,
    pub signal: Option<String>,
}

/// Enumeration of all possible machine lifecycle states.
///
/// Machine states represent the current phase in the machine's lifecycle, from creation
/// to destruction. State transitions are managed by the Fly.io platform and follow a
/// predictable pattern with some states being terminal.
///
/// # State Transitions
///
/// ```text
/// Created -> Starting -> Started -> Stopping -> Stopped
///    |          |          |          |          |
///    |          |          |          |          |
///    +--> Destroying --> Destroyed   |          |
///    |          ^          ^          |          |
///    |          |          |          |          |
///    |          +----------+---------+          |
///    |                     |                     |
///    +--> Replacing -------+---------------------+
/// ```
///
/// # State Descriptions
///
/// - [`Created`] - Machine definition exists but hasn't been started
/// - [`Starting`] - Machine is initializing and booting up
/// - [`Started`] - Machine is running and ready to serve traffic
/// - [`Stopping`] - Machine is gracefully shutting down
/// - [`Stopped`] - Machine has stopped and can be restarted
/// - [`Replacing`] - Machine is being replaced with a new version
/// - [`Destroying`] - Machine is being permanently deleted
/// - [`Destroyed`] - Machine has been permanently removed (terminal state)
///
/// # Examples
///
/// ```rust
/// use minifly::MachineState;
///
/// let state = MachineState::Started;
/// assert_eq!(format!("{:?}", state), "Started");
///
/// // States can be serialized to lowercase strings
/// let json = serde_json::to_string(&state).unwrap();
/// assert_eq!(json, "\"started\"");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MachineState {
    /// Machine definition exists but hasn't been started
    Created,
    /// Machine is initializing and booting up
    Starting,
    /// Machine is running and ready to serve traffic
    Started,
    /// Machine is gracefully shutting down
    Stopping,
    /// Machine has stopped and can be restarted
    Stopped,
    /// Machine is being replaced with a new version
    Replacing,
    /// Machine is being permanently deleted
    Destroying,
    /// Machine has been permanently removed (terminal state)
    Destroyed,
}

impl Default for Guest {
    fn default() -> Self {
        Self {
            cpu_kind: "shared".to_string(),
            cpus: 1,
            memory_mb: 256,
            kernel_args: None,
        }
    }
}

impl Default for RestartPolicy {
    fn default() -> Self {
        Self {
            policy: "on-failure".to_string(),
            max_retries: 5,
        }
    }
}

impl Default for MachineConfig {
    fn default() -> Self {
        Self {
            image: "nginx:latest".to_string(),
            env: HashMap::new(),
            services: vec![],
            guest: Guest::default(),
            restart: RestartPolicy::default(),
            auto_destroy: false,
            kill_timeout: Some(5),
        }
    }
}