// DNS integration tests temporarily disabled due to complex mocking requirements
// TODO: Enable when test infrastructure supports mock Docker and DNS resolver

/*
use minifly_core::models::{CreateMachineRequest, MachineConfig, GuestConfig};
use std::collections::HashMap;

mod common;
use common::*;

#[tokio::test]
async fn test_dns_registration() {
    let (url, _handle) = start_test_server().await;
    let client = test_client();
    
    // Create an app
    let create_app_req = CreateAppRequest {
        app_name: "dns-test-app".to_string(),
        org_slug: "personal".to_string(),
    };
    
    let response = client
        .post(&format!("{}/apps", url))
        .json(&create_app_req)
        .send()
        .await
        .expect("Failed to create app");
    
    assert_eq!(response.status(), 201);
    
    // Create a machine
    let mut env = HashMap::new();
    env.insert("TEST_VAR".to_string(), "test_value".to_string());
    
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
        name: Some("dns-test-machine".to_string()),
        region: Some("local".to_string()),
        config,
        skip_launch: Some(false),
        skip_service_registration: None,
        lease_ttl: None,
    };
    
    // Create machine - this should register with DNS
    let response = client
        .post(&format!("{}/apps/dns-test-app/machines", url))
        .json(&req)
        .send()
        .await
        .expect("Failed to create machine");
    
    assert_eq!(response.status(), 201);
    
    let machine_data: serde_json::Value = response.json().await.unwrap();
    let machine_id = machine_data["id"].as_str().unwrap();
    
    // Give DNS time to register
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    // We can't directly test DNS resolution here without access to AppState,
    // but we've verified the machine was created successfully
    // In a real integration test, we'd query the DNS resolver
    
    // Cleanup
    client
        .delete(&format!("{}/apps/dns-test-app/machines/{}", url, machine_id))
        .send()
        .await
        .expect("Failed to delete machine");
}

#[tokio::test]
async fn test_dns_cleanup_on_stop() {
    let (url, _handle) = start_test_server().await;
    let client = test_client();
    
    // Create an app
    let create_app_req = CreateAppRequest {
        app_name: "dns-cleanup-app".to_string(),
        org_slug: "personal".to_string(),
    };
    
    client
        .post(&format!("{}/apps", url))
        .json(&create_app_req)
        .send()
        .await
        .expect("Failed to create app");
    
    // Create and start a machine
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
        env: None,
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
        name: Some("cleanup-test-machine".to_string()),
        region: Some("local".to_string()),
        config,
        skip_launch: Some(false),
        skip_service_registration: None,
        lease_ttl: None,
    };
    
    let response = client
        .post(&format!("{}/apps/dns-cleanup-app/machines", url))
        .json(&req)
        .send()
        .await
        .expect("Failed to create machine");
    
    let machine_data: serde_json::Value = response.json().await.unwrap();
    let machine_id = machine_data["id"].as_str().unwrap();
    
    // Stop the machine - this should unregister from DNS
    let stop_response = client
        .post(&format!("{}/apps/dns-cleanup-app/machines/{}/stop", url, machine_id))
        .send()
        .await
        .expect("Failed to stop machine");
    
    assert_eq!(stop_response.status(), 200);
    
    // Cleanup
    client
        .delete(&format!("{}/apps/dns-cleanup-app/machines/{}", url, machine_id))
        .send()
        .await
        .expect("Failed to delete machine");
}
*/