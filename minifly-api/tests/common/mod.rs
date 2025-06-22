use axum::{Router, routing::get};
use minifly_api::{create_app, AppState, Config};
use minifly_core::models::*;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use tokio::net::TcpListener;
use uuid::Uuid;

/// Test configuration with ephemeral ports
pub fn test_config() -> Config {
    Config {
        port: 0, // Let OS assign port
        database_url: ":memory:".to_string(), // In-memory SQLite for tests
        docker_host: None,
        data_dir: "/tmp/minifly-test".into(),
        internal_network_prefix: "fdaa:0:".to_string(),
        dns_port: 0, // Let OS assign port
        litefs_port: 0, // Let OS assign port
    }
}

/// Create a test app state - for now use mock state for testing
pub fn test_app_state() -> AppState {
    // For integration tests, we'll use a simplified mock state
    // The actual AppState requires Docker and database connections
    AppState {
        config: test_config(),
        db: todo!("Mock database for tests"), 
        docker: todo!("Mock Docker for tests"),
        litefs: todo!("Mock LiteFS for tests"),
        leases: Arc::new(RwLock::new(HashMap::new())),
        machines: Arc::new(RwLock::new(HashMap::new())),
        apps: Arc::new(RwLock::new(HashMap::new())),
        start_time: std::time::Instant::now(),
    }
}

/// Start a test server and return its URL
pub async fn start_test_server() -> (String, tokio::task::JoinHandle<()>) {
    let state = test_app_state();
    let app = create_app(state);
    
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to port");
    
    let port = listener.local_addr().unwrap().port();
    let url = format!("http://127.0.0.1:{}", port);
    
    let handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("Failed to start server");
    });
    
    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    (url, handle)
}

/// Create a test app
pub fn create_test_app(name: &str) -> App {
    App {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        organization: Organization {
            id: "test-org".to_string(),
            slug: "test-org".to_string(),
            name: "Test Organization".to_string(),
        },
        status: AppStatus::Running,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

/// Create a test machine
pub fn create_test_machine(app_name: &str, machine_name: &str) -> Machine {
    let id = format!("d{}", Uuid::new_v4().to_string().replace("-", "")[..15]);
    Machine {
        id: id.clone(),
        name: format!("{}-{}", app_name, machine_name),
        state: MachineState::Created,
        region: "local".to_string(),
        config: MachineConfig {
            image: "nginx:latest".to_string(),
            guest: Some(GuestConfig {
                cpus: 1,
                memory_mb: 256,
                cpu_kind: CpuKind::Shared,
            }),
            env: HashMap::new(),
            services: vec![],
            mounts: vec![],
            size: MachineSize::Shared1x,
            restart: Some(RestartConfig {
                policy: RestartPolicy::Always,
                max_retries: Some(3),
            }),
            auto_destroy: false,
            schedule: None,
        },
        app_name: app_name.to_string(),
        private_ip: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        version: 1,
        events: vec![],
        checks: HashMap::new(),
        nonce: None,
    }
}

/// Create a test volume
pub fn create_test_volume(name: &str, app_name: &str) -> Volume {
    Volume {
        id: format!("vol_{}", Uuid::new_v4().to_string().replace("-", "")[..10]),
        name: name.to_string(),
        app_id: app_name.to_string(),
        size_gb: 1,
        region: "local".to_string(),
        zone: "local-a".to_string(),
        encrypted: false,
        created_at: chrono::Utc::now(),
        attached_machine_id: None,
        attached_allocation_id: None,
        block_size: 4096,
        blocks: 262144,
        blocks_avail: 262144,
        blocks_free: 262144,
        fstype: "ext4".to_string(),
        snapshot_retention: None,
    }
}

/// HTTP client for testing
pub fn test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("Failed to create test client")
}

/// Assert that a response has a specific status code
#[macro_export]
macro_rules! assert_status {
    ($response:expr, $status:expr) => {
        assert_eq!(
            $response.status(),
            $status,
            "Expected status {}, got {} with body: {:?}",
            $status,
            $response.status(),
            $response.text().await.unwrap_or_default()
        );
    };
}

/// Assert that a JSON response matches expected value
#[macro_export]
macro_rules! assert_json_response {
    ($response:expr, $expected:expr) => {
        let status = $response.status();
        let body = $response.text().await.expect("Failed to read response body");
        
        assert!(
            status.is_success(),
            "Expected success status, got {} with body: {}",
            status,
            body
        );
        
        let actual: serde_json::Value = serde_json::from_str(&body)
            .expect(&format!("Failed to parse JSON response: {}", body));
        
        assert_eq!(actual, $expected);
    };
}