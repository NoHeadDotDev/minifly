//! Logging utilities for Minifly CLI

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize default logging for the CLI
pub fn init_default_logging() -> anyhow::Result<()> {
    let format = if std::env::var("MINIFLY_LOG_JSON").is_ok() {
        "json"
    } else {
        "human"
    };

    let level = std::env::var("MINIFLY_LOG_LEVEL")
        .unwrap_or_else(|_| "minifly=info".to_string());

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&level));

    let subscriber = tracing_subscriber::registry()
        .with(env_filter);

    match format {
        "json" => {
            subscriber
                .with(
                    tracing_subscriber::fmt::layer()
                        .json()
                        .with_current_span(false)
                        .with_span_list(true)
                        .with_target(true)
                )
                .init();
        }
        _ => {
            subscriber
                .with(
                    tracing_subscriber::fmt::layer()
                        .pretty()
                        .with_target(false)
                )
                .init();
        }
    }

    tracing::info!(
        service = "minifly-cli",
        version = env!("CARGO_PKG_VERSION"),
        log.format = format,
        log.level = level,
        "Minifly CLI logging initialized"
    );

    Ok(())
}