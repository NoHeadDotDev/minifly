//! Service dependency management for Minifly platform

use anyhow::{Result, Context};
use colored::*;
use std::time::{Duration, Instant};
use crate::client::ApiClient;

/// Service dependency information
#[derive(Debug, Clone)]
pub struct ServiceDependency {
    pub name: String,
    pub check_fn: CheckFunction,
    pub required: bool,
    pub timeout_seconds: u64,
    pub retry_count: u32,
}

/// Function type for service checks
pub type CheckFunction = fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool>> + Send>>;

/// Dependency check results
#[derive(Debug)]
pub struct DependencyCheckResult {
    pub service: String,
    pub available: bool,
    pub response_time_ms: u64,
    pub error: Option<String>,
}

/// Service dependency manager
pub struct DependencyManager {
    dependencies: Vec<ServiceDependency>,
}

impl DependencyManager {
    pub fn new() -> Self {
        Self {
            dependencies: vec![
                ServiceDependency {
                    name: "Docker".to_string(),
                    check_fn: check_docker,
                    required: true,
                    timeout_seconds: 10,
                    retry_count: 3,
                },
                ServiceDependency {
                    name: "SQLite".to_string(),
                    check_fn: check_sqlite,
                    required: true,
                    timeout_seconds: 5,
                    retry_count: 2,
                },
                ServiceDependency {
                    name: "File System".to_string(),
                    check_fn: check_filesystem,
                    required: true,
                    timeout_seconds: 5,
                    retry_count: 1,
                },
            ],
        }
    }
    
    /// Check if a service is required
    pub fn is_required(&self, service_name: &str) -> bool {
        self.dependencies.iter()
            .find(|d| d.name == service_name)
            .map(|d| d.required)
            .unwrap_or(false)
    }

    /// Check all service dependencies
    pub async fn check_all_dependencies(&self) -> Vec<DependencyCheckResult> {
        println!("{}", "🔍 Checking service dependencies...".blue());
        
        let mut results = Vec::new();
        
        for dependency in &self.dependencies {
            let result = self.check_single_dependency(dependency).await;
            
            let status_icon = if result.available { "✅" } else { "❌" };
            let status_color = if result.available { 
                format!("Available ({}ms)", result.response_time_ms).green()
            } else { 
                "Unavailable".red() 
            };
            
            println!("  {} {}: {}", 
                status_icon, 
                dependency.name, 
                status_color
            );
            
            if let Some(ref error) = result.error {
                println!("    {}", format!("Error: {}", error).red().dimmed());
            }
            
            results.push(result);
        }
        
        results
    }

    /// Check a single service dependency with retries
    async fn check_single_dependency(&self, dependency: &ServiceDependency) -> DependencyCheckResult {
        let start_time = Instant::now();
        let mut last_error = None;
        
        for attempt in 1..=dependency.retry_count {
            match tokio::time::timeout(
                Duration::from_secs(dependency.timeout_seconds),
                (dependency.check_fn)()
            ).await {
                Ok(Ok(true)) => {
                    return DependencyCheckResult {
                        service: dependency.name.clone(),
                        available: true,
                        response_time_ms: start_time.elapsed().as_millis() as u64,
                        error: None,
                    };
                }
                Ok(Ok(false)) => {
                    last_error = Some("Service check returned false".to_string());
                }
                Ok(Err(e)) => {
                    last_error = Some(e.to_string());
                }
                Err(_) => {
                    last_error = Some("Timeout waiting for service".to_string());
                }
            }
            
            // Wait before retry (except on last attempt)
            if attempt < dependency.retry_count {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
        
        DependencyCheckResult {
            service: dependency.name.clone(),
            available: false,
            response_time_ms: start_time.elapsed().as_millis() as u64,
            error: last_error,
        }
    }

    /// Wait for required dependencies to become available
    pub async fn wait_for_dependencies(&self, max_wait_seconds: u64) -> Result<()> {
        println!("{}", "⏳ Waiting for required dependencies...".yellow());
        
        let start_time = Instant::now();
        let max_duration = Duration::from_secs(max_wait_seconds);
        
        loop {
            let results = self.check_all_dependencies().await;
            
            let required_deps_available = results.iter()
                .filter(|r| {
                    self.dependencies.iter()
                        .find(|d| d.name == r.service)
                        .map(|d| d.required)
                        .unwrap_or(false)
                })
                .all(|r| r.available);
            
            if required_deps_available {
                println!("{}", "✅ All required dependencies are available!".green());
                return Ok(());
            }
            
            if start_time.elapsed() >= max_duration {
                return Err(anyhow::anyhow!(
                    "Timeout waiting for required dependencies after {}s", 
                    max_wait_seconds
                ));
            }
            
            println!("{}", "   Retrying in 2 seconds...".dimmed());
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    /// Get dependency summary for display
    pub async fn get_dependency_summary(&self) -> (usize, usize, Vec<String>) {
        let results = self.check_all_dependencies().await;
        
        let total = results.len();
        let available = results.iter().filter(|r| r.available).count();
        let failed_services = results.iter()
            .filter(|r| !r.available)
            .map(|r| r.service.clone())
            .collect();
        
        (total, available, failed_services)
    }
}

/// Check if Docker is available
fn check_docker() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool>> + Send>> {
    Box::pin(async move {
        match tokio::process::Command::new("docker")
            .args(&["version", "--format", "{{.Server.Version}}"])
            .output()
            .await 
        {
            Ok(output) => Ok(output.status.success() && !output.stdout.is_empty()),
            Err(_) => Ok(false),
        }
    })
}

/// Check if SQLite is available
fn check_sqlite() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool>> + Send>> {
    Box::pin(async move {
        // Check if SQLite CLI is available
        match tokio::process::Command::new("sqlite3")
            .arg("--version")
            .output()
            .await
        {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    })
}

/// Check if file system is accessible
fn check_filesystem() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool>> + Send>> {
    Box::pin(async move {
        // Test creating and removing a temporary file
        let test_file = std::env::temp_dir().join("minifly_test.tmp");
        
        match tokio::fs::write(&test_file, "test").await {
            Ok(_) => {
                match tokio::fs::remove_file(&test_file).await {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            Err(_) => Ok(false),
        }
    })
}

/// Startup orchestration for the platform
pub async fn startup_with_dependencies(api_client: &ApiClient, port: u16) -> Result<()> {
    println!("{}", "🚀 Starting Minifly Platform with dependency checks...".blue().bold());
    
    let dep_manager = DependencyManager::new();
    
    // Check initial dependencies
    let results = dep_manager.check_all_dependencies().await;
    let failed_deps: Vec<_> = results.iter()
        .filter(|r| !r.available)
        .collect();
    
    if !failed_deps.is_empty() {
        println!("\n{}", "⚠️  Some dependencies are not available:".yellow().bold());
        for dep in &failed_deps {
            println!("  • {}: {}", dep.service.red(), 
                    dep.error.as_ref().unwrap_or(&"Unknown error".to_string()).dimmed());
        }
        
        // Check if any required dependencies are missing
        let required_missing = failed_deps.iter()
            .any(|r| dep_manager.dependencies.iter()
                .find(|d| d.name == r.service)
                .map(|d| d.required)
                .unwrap_or(false));
        
        if required_missing {
            println!("\n{}", "❌ Cannot start platform: required dependencies are missing".red().bold());
            println!("{}", "Please ensure Docker and SQLite are installed and available".yellow());
            return Err(anyhow::anyhow!("Required dependencies not available"));
        } else {
            println!("\n{}", "⚠️  Continuing with degraded functionality...".yellow());
        }
    }
    
    // Wait for API server to become available (if starting it)
    println!("\n{}", "🔧 Waiting for API server to start...".blue());
    let api_start_time = Instant::now();
    
    loop {
        if api_client.health_check().await.unwrap_or(false) {
            println!("{}", "✅ API server is ready!".green());
            break;
        }
        
        if api_start_time.elapsed().as_secs() > 30 {
            return Err(anyhow::anyhow!("Timeout waiting for API server to start"));
        }
        
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    println!("\n{}", "🎉 Platform startup complete!".green().bold());
    println!("  • API server: {}", format!("http://localhost:{}", port).cyan());
    println!("  • Health endpoint: {}", format!("http://localhost:{}/health", port).cyan());
    
    Ok(())
}

/// Quick dependency check for status commands
pub async fn quick_dependency_check() -> (bool, Vec<String>) {
    let dep_manager = DependencyManager::new();
    let (total, available, failed) = dep_manager.get_dependency_summary().await;
    
    (total == available, failed)
}