//! Health check system for Minifly API server

use axum::{extract::State, response::Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error};
use crate::state::AppState;
use anyhow::Result;
use sqlx::Row;

/// Overall health status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Individual service health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub status: HealthStatus,
    pub message: String,
    pub last_checked: String,
    pub response_time_ms: Option<u64>,
    pub details: HashMap<String, serde_json::Value>,
}

/// Complete health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub version: String,
    pub uptime_seconds: u64,
    pub timestamp: String,
    pub services: HashMap<String, ServiceHealth>,
    pub summary: String,
}

/// Health check manager
pub struct HealthChecker {
    start_time: std::time::Instant,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }

    /// Perform comprehensive health check
    pub async fn check_health(&self, state: &AppState) -> HealthResponse {
        info!("Performing comprehensive health check");
        let start = std::time::Instant::now();
        
        let mut services = HashMap::new();
        
        // Check database connectivity
        services.insert("database".to_string(), self.check_database_health(state).await);
        
        // Check Docker daemon
        services.insert("docker".to_string(), self.check_docker_health(state).await);
        
        // Check LiteFS (if configured)
        services.insert("litefs".to_string(), self.check_litefs_health(state).await);
        
        // Check file system access
        services.insert("filesystem".to_string(), self.check_filesystem_health().await);
        
        // Determine overall status
        let overall_status = self.determine_overall_status(&services);
        let summary = self.generate_summary(&services, &overall_status);
        
        let health_check_duration = start.elapsed();
        
        info!(
            status = ?overall_status,
            duration_ms = health_check_duration.as_millis(),
            services_checked = services.len(),
            "Health check completed"
        );
        
        HealthResponse {
            status: overall_status,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            services,
            summary,
        }
    }

    /// Check database health
    async fn check_database_health(&self, state: &AppState) -> ServiceHealth {
        let start = std::time::Instant::now();
        
        match self.test_database_connection(state).await {
            Ok(_) => {
                ServiceHealth {
                    status: HealthStatus::Healthy,
                    message: "Database connection successful".to_string(),
                    last_checked: chrono::Utc::now().to_rfc3339(),
                    response_time_ms: Some(start.elapsed().as_millis() as u64),
                    details: HashMap::new(),
                }
            }
            Err(e) => {
                error!(error = %e, "Database health check failed");
                ServiceHealth {
                    status: HealthStatus::Unhealthy,
                    message: format!("Database connection failed: {}", e),
                    last_checked: chrono::Utc::now().to_rfc3339(),
                    response_time_ms: Some(start.elapsed().as_millis() as u64),
                    details: HashMap::new(),
                }
            }
        }
    }

    /// Check Docker daemon health
    async fn check_docker_health(&self, state: &AppState) -> ServiceHealth {
        let start = std::time::Instant::now();
        
        match self.test_docker_connection(state).await {
            Ok(info) => {
                let mut details = HashMap::new();
                details.insert("version".to_string(), serde_json::Value::String(info.version));
                details.insert("containers_running".to_string(), serde_json::Value::Number(info.containers_running.into()));
                
                ServiceHealth {
                    status: HealthStatus::Healthy,
                    message: "Docker daemon accessible".to_string(),
                    last_checked: chrono::Utc::now().to_rfc3339(),
                    response_time_ms: Some(start.elapsed().as_millis() as u64),
                    details,
                }
            }
            Err(e) => {
                error!(error = %e, "Docker health check failed");
                ServiceHealth {
                    status: HealthStatus::Unhealthy,
                    message: format!("Docker daemon unavailable: {}", e),
                    last_checked: chrono::Utc::now().to_rfc3339(),
                    response_time_ms: Some(start.elapsed().as_millis() as u64),
                    details: HashMap::new(),
                }
            }
        }
    }

    /// Check LiteFS health
    async fn check_litefs_health(&self, _state: &AppState) -> ServiceHealth {
        let start = std::time::Instant::now();
        
        // Check if LiteFS mount points are accessible
        match self.test_litefs_mounts().await {
            Ok(mount_info) => {
                ServiceHealth {
                    status: HealthStatus::Healthy,
                    message: "LiteFS mounts accessible".to_string(),
                    last_checked: chrono::Utc::now().to_rfc3339(),
                    response_time_ms: Some(start.elapsed().as_millis() as u64),
                    details: mount_info,
                }
            }
            Err(e) => {
                warn!(error = %e, "LiteFS health check failed");
                ServiceHealth {
                    status: HealthStatus::Degraded,
                    message: format!("LiteFS issues detected: {}", e),
                    last_checked: chrono::Utc::now().to_rfc3339(),
                    response_time_ms: Some(start.elapsed().as_millis() as u64),
                    details: HashMap::new(),
                }
            }
        }
    }

    /// Check filesystem health
    async fn check_filesystem_health(&self) -> ServiceHealth {
        let start = std::time::Instant::now();
        
        match self.test_filesystem_access().await {
            Ok(fs_info) => {
                ServiceHealth {
                    status: HealthStatus::Healthy,
                    message: "Filesystem access normal".to_string(),
                    last_checked: chrono::Utc::now().to_rfc3339(),
                    response_time_ms: Some(start.elapsed().as_millis() as u64),
                    details: fs_info,
                }
            }
            Err(e) => {
                error!(error = %e, "Filesystem health check failed");
                ServiceHealth {
                    status: HealthStatus::Unhealthy,
                    message: format!("Filesystem access issues: {}", e),
                    last_checked: chrono::Utc::now().to_rfc3339(),
                    response_time_ms: Some(start.elapsed().as_millis() as u64),
                    details: HashMap::new(),
                }
            }
        }
    }

    /// Test database connection
    async fn test_database_connection(&self, state: &AppState) -> Result<()> {
        // Simple query to test database connectivity
        let result = sqlx::query("SELECT 1 as test")
            .fetch_one(&state.db)
            .await?;
        
        let _test_value: i32 = result.get("test");
        Ok(())
    }

    /// Test Docker daemon connection
    async fn test_docker_connection(&self, state: &AppState) -> Result<DockerInfo> {
        let version = state.docker.version().await?;
        let containers = state.docker.list_containers(None).await?;
        
        let running_containers = containers.iter()
            .filter(|c| c.state.as_ref().map(|s| s == "running").unwrap_or(false))
            .count();

        Ok(DockerInfo {
            version: version.version.unwrap_or_else(|| "unknown".to_string()),
            containers_running: running_containers,
        })
    }

    /// Test LiteFS mount accessibility
    async fn test_litefs_mounts(&self) -> Result<HashMap<String, serde_json::Value>> {
        let mut details = HashMap::new();
        
        // Check if data directory exists and is writable
        let data_dir = std::path::Path::new("./data");
        if data_dir.exists() {
            details.insert("data_dir_exists".to_string(), serde_json::Value::Bool(true));
            
            // Try to write a test file
            let test_file = data_dir.join("health_check.tmp");
            match std::fs::write(&test_file, "health_check") {
                Ok(_) => {
                    details.insert("data_dir_writable".to_string(), serde_json::Value::Bool(true));
                    let _ = std::fs::remove_file(&test_file); // Clean up
                }
                Err(_) => {
                    details.insert("data_dir_writable".to_string(), serde_json::Value::Bool(false));
                }
            }
        } else {
            details.insert("data_dir_exists".to_string(), serde_json::Value::Bool(false));
        }
        
        Ok(details)
    }

    /// Test filesystem access
    async fn test_filesystem_access(&self) -> Result<HashMap<String, serde_json::Value>> {
        let mut details = HashMap::new();
        
        // Check current directory access
        let current_dir = std::env::current_dir()?;
        details.insert("current_dir".to_string(), 
                      serde_json::Value::String(current_dir.display().to_string()));
        
        // Check available disk space (simplified)
        if let Ok(metadata) = std::fs::metadata(&current_dir) {
            details.insert("current_dir_accessible".to_string(), serde_json::Value::Bool(true));
        } else {
            details.insert("current_dir_accessible".to_string(), serde_json::Value::Bool(false));
        }
        
        Ok(details)
    }

    /// Determine overall health status from individual services
    fn determine_overall_status(&self, services: &HashMap<String, ServiceHealth>) -> HealthStatus {
        let mut healthy_count = 0;
        let mut degraded_count = 0;
        let mut unhealthy_count = 0;
        
        for service in services.values() {
            match service.status {
                HealthStatus::Healthy => healthy_count += 1,
                HealthStatus::Degraded => degraded_count += 1,
                HealthStatus::Unhealthy => unhealthy_count += 1,
            }
        }
        
        // Determine overall status based on service health
        if unhealthy_count > 0 {
            HealthStatus::Unhealthy
        } else if degraded_count > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    /// Generate health summary message
    fn generate_summary(&self, services: &HashMap<String, ServiceHealth>, overall_status: &HealthStatus) -> String {
        let total_services = services.len();
        let healthy_services = services.values()
            .filter(|s| matches!(s.status, HealthStatus::Healthy))
            .count();
        
        match overall_status {
            HealthStatus::Healthy => {
                format!("All {} services are healthy", total_services)
            }
            HealthStatus::Degraded => {
                format!("{}/{} services healthy, some degraded", healthy_services, total_services)
            }
            HealthStatus::Unhealthy => {
                format!("{}/{} services healthy, critical issues detected", healthy_services, total_services)
            }
        }
    }
}

/// Docker information for health checks
#[derive(Debug)]
struct DockerInfo {
    version: String,
    containers_running: usize,
}

/// Health check endpoint handler
pub async fn health_handler(State(state): State<AppState>) -> Result<Json<HealthResponse>, StatusCode> {
    let health_checker = HealthChecker::new();
    let health_response = health_checker.check_health(&state).await;
    
    // Return appropriate HTTP status based on health
    match health_response.status {
        HealthStatus::Healthy => Ok(Json(health_response)),
        HealthStatus::Degraded => {
            // Return 200 but log the degraded state
            warn!("Platform is in degraded state: {}", health_response.summary);
            Ok(Json(health_response))
        }
        HealthStatus::Unhealthy => {
            // Return 503 Service Unavailable for unhealthy state
            error!("Platform is unhealthy: {}", health_response.summary);
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

/// Simple liveness probe endpoint
pub async fn liveness_handler() -> StatusCode {
    // Simple liveness check - if the server can respond, it's alive
    StatusCode::OK
}

/// Readiness probe endpoint
pub async fn readiness_handler(State(state): State<AppState>) -> StatusCode {
    let health_checker = HealthChecker::new();
    
    // Quick readiness check focusing on critical services
    let db_ready = health_checker.test_database_connection(&state).await.is_ok();
    let docker_ready = health_checker.test_docker_connection(&state).await.is_ok();
    
    if db_ready && docker_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}