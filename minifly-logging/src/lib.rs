use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use uuid::Uuid;

/// Logging configuration for Minifly services
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub service_name: String,
    pub service_version: String,
    pub environment: String,
    pub format: LogFormat,
    pub level: String,
}

/// Log output format options
#[derive(Debug, Clone)]
pub enum LogFormat {
    /// Human-readable format for development
    Human,
    /// JSON format for production and log aggregation
    Json,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            service_name: "minifly".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            environment: "development".to_string(),
            format: LogFormat::Human,
            level: "info".to_string(),
        }
    }
}

impl LoggingConfig {
    /// Create a new logging configuration
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            ..Default::default()
        }
    }

    /// Set the log format
    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    /// Set the log level
    pub fn with_level(mut self, level: &str) -> Self {
        self.level = level.to_string();
        self
    }

    /// Set the environment
    pub fn with_environment(mut self, environment: &str) -> Self {
        self.environment = environment.to_string();
        self
    }

    /// Build config from environment variables
    pub fn from_env(service_name: &str) -> Self {
        let format = match std::env::var("MINIFLY_LOG_FORMAT").as_deref() {
            Ok("json") => LogFormat::Json,
            _ => LogFormat::Human,
        };

        let level = std::env::var("MINIFLY_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
        let environment = std::env::var("MINIFLY_ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        Self {
            service_name: service_name.to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            environment,
            format,
            level,
        }
    }
}

/// Initialize structured logging for a Minifly service
pub fn init_logging(config: LoggingConfig) -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    let subscriber = tracing_subscriber::registry()
        .with(env_filter);

    match config.format {
        LogFormat::Json => {
            subscriber
                .with(
                    tracing_subscriber::fmt::layer()
                        .json()
                        .with_current_span(false)
                        .with_span_list(true)
                        .with_target(true)
                        .with_thread_ids(true)
                        .with_thread_names(true)
                )
                .init();
        }
        LogFormat::Human => {
            subscriber
                .with(
                    tracing_subscriber::fmt::layer()
                        .pretty()
                        .with_target(true)
                        .with_thread_ids(false)
                        .with_thread_names(false)
                )
                .init();
        }
    }

    // Set global context
    tracing::info!(
        service.name = %config.service_name,
        service.version = %config.service_version,
        environment = %config.environment,
        log.format = ?config.format,
        log.level = %config.level,
        "Structured logging initialized"
    );

    Ok(())
}

/// Standard field names for consistent logging across all Minifly components
pub mod fields {
    // Identity and correlation fields
    pub const CORRELATION_ID: &str = "correlation_id";
    pub const REQUEST_ID: &str = "request_id";
    pub const USER_ID: &str = "user_id";
    pub const SESSION_ID: &str = "session_id";

    // Business entities
    pub const APP_NAME: &str = "app.name";
    pub const MACHINE_ID: &str = "machine.id";
    pub const REGION: &str = "region";
    pub const IMAGE: &str = "image";
    pub const CONTAINER_ID: &str = "container.id";

    // Operations
    pub const OPERATION: &str = "operation";
    pub const OPERATION_TYPE: &str = "operation.type";
    pub const OPERATION_STATUS: &str = "operation.status";
    pub const DURATION_MS: &str = "duration_ms";

    // HTTP context
    pub const HTTP_METHOD: &str = "http.method";
    pub const HTTP_PATH: &str = "http.path";
    pub const HTTP_STATUS: &str = "http.status";
    pub const HTTP_USER_AGENT: &str = "http.user_agent";

    // Error context
    pub const ERROR_TYPE: &str = "error.type";
    pub const ERROR_MESSAGE: &str = "error.message";
    pub const ERROR_STACK: &str = "error.stack";
    pub const ERROR_ID: &str = "error.id";

    // LiteFS specific
    pub const LITEFS_MOUNT_PATH: &str = "litefs.mount_path";
    pub const LITEFS_IS_PRIMARY: &str = "litefs.is_primary";
    pub const LITEFS_CONFIG_PATH: &str = "litefs.config_path";

    // Docker specific
    pub const DOCKER_IMAGE: &str = "docker.image";
    pub const DOCKER_CONTAINER_NAME: &str = "docker.container.name";
    pub const DOCKER_NETWORK: &str = "docker.network";
}

/// Generate a new correlation ID
pub fn new_correlation_id() -> String {
    Uuid::new_v4().to_string()
}

/// Generate a new request ID
pub fn new_request_id() -> String {
    Uuid::new_v4().to_string()
}

/// Generate a new error ID
pub fn new_error_id() -> String {
    Uuid::new_v4().to_string()
}

/// Macro for creating structured operation spans
#[macro_export]
macro_rules! operation_span {
    ($operation:expr, $($field:ident = $value:expr),* $(,)?) => {
        tracing::info_span!(
            "operation",
            operation = $operation,
            correlation_id = %$crate::new_correlation_id(),
            $($field = $value,)*
        )
    };
}

/// Macro for logging operation completion
#[macro_export]
macro_rules! log_operation_result {
    ($result:expr, $success_msg:expr, $error_msg:expr) => {
        match &$result {
            Ok(_) => {
                tracing::info!(
                    operation.status = "success",
                    $success_msg
                );
            }
            Err(e) => {
                tracing::error!(
                    operation.status = "failed",
                    error.message = %e,
                    $error_msg
                );
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.service_name, "minifly");
        assert_eq!(config.environment, "development");
        assert!(matches!(config.format, LogFormat::Human));
    }

    #[test]
    fn test_logging_config_builder() {
        let config = LoggingConfig::new("test-service")
            .with_format(LogFormat::Json)
            .with_level("debug")
            .with_environment("production");

        assert_eq!(config.service_name, "test-service");
        assert_eq!(config.level, "debug");
        assert_eq!(config.environment, "production");
        assert!(matches!(config.format, LogFormat::Json));
    }

    #[test]
    fn test_correlation_id_generation() {
        let id1 = new_correlation_id();
        let id2 = new_correlation_id();
        
        assert_ne!(id1, id2);
        assert!(uuid::Uuid::parse_str(&id1).is_ok());
        assert!(uuid::Uuid::parse_str(&id2).is_ok());
    }
}