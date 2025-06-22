// Environment variable translation tests temporarily disabled due to complex mocking requirements
// TODO: Enable when test infrastructure supports mock Docker

/*
use std::collections::HashMap;
use minifly_core::models::{CreateMachineRequest, MachineConfig, GuestConfig, CreateAppRequest};

mod common;
use common::*;

#[tokio::test]
async fn test_fly_env_vars_translation() {
    let (url, _handle) = start_test_server().await;
    let client = test_client();
    
    // Create an app
    let create_app_req = CreateAppRequest {
        app_name: "env-test-app".to_string(),
        org_slug: "personal".to_string(),
    };
    
    client
        .post(&format!("{}/apps", url))
        .json(&create_app_req)
        .send()
        .await
        .expect("Failed to create app");
    
    // Create a machine with some existing env vars
    let mut env = HashMap::new();
    env.insert("MY_VAR".to_string(), "my_value".to_string());
    env.insert("NODE_ENV".to_string(), "production".to_string()); // Should not be overridden
    
    let config = MachineConfig {
        image: "alpine:latest".to_string(),
        guest: GuestConfig {
            cpu_kind: "shared".to_string(),
            cpus: 1,
            memory_mb: 256,
            gpu_kind: None,
            gpus: None,
            kernel_args: None,
        },
        env: Some(env),
        services: None,
        mounts: None,
        restart: None,
        checks: None,
        auto_destroy: None,
        dns: None,
        processes: None,
        files: None,
        init: None,
        containers: None,
    };
    
    let req = CreateMachineRequest {
        name: Some("env-test-machine".to_string()),
        region: Some("local".to_string()),
        config,
        skip_launch: Some(true), // Skip actual container creation for unit test
        skip_service_registration: None,
        lease_ttl: None,
    };
    
    let response = client
        .post(&format!("{}/apps/env-test-app/machines", url))
        .json(&req)
        .send()
        .await
        .expect("Failed to create machine");
    
    assert_eq!(response.status(), 201);
    
    let machine_data: serde_json::Value = response.json().await.unwrap();
    
    // The actual environment variable translation happens inside the Docker container
    // so we can only verify the machine was created successfully
    // In integration tests, we would inspect the running container
    
    assert_eq!(machine_data["name"], "env-test-machine");
    assert_eq!(machine_data["region"], "local");
}

#[tokio::test]
async fn test_tigris_endpoint_translation() {
    let (url, _handle) = start_test_server().await;
    let client = test_client();
    
    let create_app_req = CreateAppRequest {
        app_name: "tigris-test-app".to_string(),
        org_slug: "personal".to_string(),
    };
    
    client
        .post(&format!("{}/apps", url))
        .json(&create_app_req)
        .send()
        .await
        .expect("Failed to create app");
    
    // Create a machine with Tigris/S3 endpoints
    let mut env = HashMap::new();
    env.insert("TIGRIS_ENDPOINT".to_string(), "https://fly.storage.tigris.dev".to_string());
    env.insert("AWS_ENDPOINT_URL".to_string(), "https://fly.storage.tigris.dev".to_string());
    
    let config = MachineConfig {
        image: "alpine:latest".to_string(),
        guest: GuestConfig {
            cpu_kind: "shared".to_string(),
            cpus: 1,
            memory_mb: 256,
            gpu_kind: None,
            gpus: None,
            kernel_args: None,
        },
        env: Some(env),
        services: None,
        mounts: None,
        restart: None,
        checks: None,
        auto_destroy: None,
        dns: None,
        processes: None,
        files: None,
        init: None,
        containers: None,
    };
    
    let req = CreateMachineRequest {
        name: Some("tigris-test-machine".to_string()),
        region: Some("local".to_string()),
        config,
        skip_launch: Some(true),
        skip_service_registration: None,
        lease_ttl: None,
    };
    
    let response = client
        .post(&format!("{}/apps/tigris-test-app/machines", url))
        .json(&req)
        .send()
        .await
        .expect("Failed to create machine");
    
    assert_eq!(response.status(), 201);
    
    // When the container is created, TIGRIS_ENDPOINT and AWS_ENDPOINT_URL
    // should be translated to point to local MinIO (http://localhost:9000)
}
*/