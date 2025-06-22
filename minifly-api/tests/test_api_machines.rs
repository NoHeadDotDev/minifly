// Integration tests temporarily disabled due to complex mocking requirements
// TODO: Implement proper test infrastructure with mock Docker and database

/*
use minifly_api::*;
use minifly_core::models::*;
use reqwest::StatusCode;
use serde_json::json;
use std::collections::HashMap;

mod common;
use common::*;

#[tokio::test]
async fn test_create_machine() {
    let (url, _handle) = start_test_server().await;
    let client = test_client();
    
    // Create app first
    let app_req = CreateAppRequest {
        name: "machine-test-app".to_string(),
        org_slug: "test-org".to_string(),
    };
    
    client
        .post(&format!("{}/v1/apps", url))
        .header("fly-region", "local")
        .json(&app_req)
        .send()
        .await
        .expect("Failed to create app");
    
    // Create machine
    let machine_req = CreateMachineRequest {
        name: Some("test-machine".to_string()),
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
        region: Some("local".to_string()),
        skip_launch: false,
        skip_service_registration: false,
    };
    
    let response = client
        .post(&format!("{}/v1/apps/machine-test-app/machines", url))
        .header("fly-region", "local")
        .json(&machine_req)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_status!(response, StatusCode::CREATED);
    
    let machine: Machine = response.json().await.expect("Failed to parse response");
    assert_eq!(machine.app_name, "machine-test-app");
    assert_eq!(machine.config.image, "nginx:latest");
    assert_eq!(machine.state, MachineState::Created);
}

#[tokio::test]
async fn test_list_machines() {
    let (url, _handle) = start_test_server().await;
    let client = test_client();
    
    // Create app
    let app_req = CreateAppRequest {
        name: "list-machines-app".to_string(),
        org_slug: "test-org".to_string(),
    };
    
    client
        .post(&format!("{}/v1/apps", url))
        .header("fly-region", "local")
        .json(&app_req)
        .send()
        .await
        .expect("Failed to create app");
    
    // Create multiple machines
    for i in 0..3 {
        let machine_req = CreateMachineRequest {
            name: Some(format!("machine-{}", i)),
            config: MachineConfig {
                image: "nginx:latest".to_string(),
                guest: None,
                env: HashMap::new(),
                services: vec![],
                mounts: vec![],
                size: MachineSize::Shared1x,
                restart: None,
                auto_destroy: false,
                schedule: None,
            },
            region: Some("local".to_string()),
            skip_launch: false,
            skip_service_registration: false,
        };
        
        client
            .post(&format!("{}/v1/apps/list-machines-app/machines", url))
            .header("fly-region", "local")
            .json(&machine_req)
            .send()
            .await
            .expect("Failed to create machine");
    }
    
    // List machines
    let response = client
        .get(&format!("{}/v1/apps/list-machines-app/machines", url))
        .header("fly-region", "local")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_status!(response, StatusCode::OK);
    
    let machines: Vec<Machine> = response.json().await.expect("Failed to parse response");
    assert_eq!(machines.len(), 3);
}

#[tokio::test]
async fn test_get_machine() {
    let (url, _handle) = start_test_server().await;
    let client = test_client();
    
    // Create app and machine
    let app_req = CreateAppRequest {
        name: "get-machine-app".to_string(),
        org_slug: "test-org".to_string(),
    };
    
    client
        .post(&format!("{}/v1/apps", url))
        .header("fly-region", "local")
        .json(&app_req)
        .send()
        .await
        .expect("Failed to create app");
    
    let machine_req = CreateMachineRequest {
        name: Some("get-test-machine".to_string()),
        config: MachineConfig {
            image: "nginx:latest".to_string(),
            guest: None,
            env: HashMap::new(),
            services: vec![],
            mounts: vec![],
            size: MachineSize::Shared1x,
            restart: None,
            auto_destroy: false,
            schedule: None,
        },
        region: Some("local".to_string()),
        skip_launch: false,
        skip_service_registration: false,
    };
    
    let create_response = client
        .post(&format!("{}/v1/apps/get-machine-app/machines", url))
        .header("fly-region", "local")
        .json(&machine_req)
        .send()
        .await
        .expect("Failed to create machine");
    
    let created_machine: Machine = create_response.json().await.expect("Failed to parse response");
    
    // Get the machine
    let response = client
        .get(&format!("{}/v1/apps/get-machine-app/machines/{}", url, created_machine.id))
        .header("fly-region", "local")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_status!(response, StatusCode::OK);
    
    let machine: Machine = response.json().await.expect("Failed to parse response");
    assert_eq!(machine.id, created_machine.id);
}

#[tokio::test]
async fn test_machine_lifecycle() {
    let (url, _handle) = start_test_server().await;
    let client = test_client();
    
    // Create app and machine
    let app_req = CreateAppRequest {
        name: "lifecycle-app".to_string(),
        org_slug: "test-org".to_string(),
    };
    
    client
        .post(&format!("{}/v1/apps", url))
        .header("fly-region", "local")
        .json(&app_req)
        .send()
        .await
        .expect("Failed to create app");
    
    let machine_req = CreateMachineRequest {
        name: Some("lifecycle-machine".to_string()),
        config: MachineConfig {
            image: "nginx:latest".to_string(),
            guest: None,
            env: HashMap::new(),
            services: vec![],
            mounts: vec![],
            size: MachineSize::Shared1x,
            restart: None,
            auto_destroy: false,
            schedule: None,
        },
        region: Some("local".to_string()),
        skip_launch: false,
        skip_service_registration: false,
    };
    
    let create_response = client
        .post(&format!("{}/v1/apps/lifecycle-app/machines", url))
        .header("fly-region", "local")
        .json(&machine_req)
        .send()
        .await
        .expect("Failed to create machine");
    
    let machine: Machine = create_response.json().await.expect("Failed to parse response");
    
    // Start machine
    let response = client
        .post(&format!("{}/v1/apps/lifecycle-app/machines/{}/start", url, machine.id))
        .header("fly-region", "local")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_status!(response, StatusCode::OK);
    
    // Stop machine
    let stop_req = StopMachineRequest {
        signal: Some("SIGTERM".to_string()),
        timeout: None,
    };
    
    let response = client
        .post(&format!("{}/v1/apps/lifecycle-app/machines/{}/stop", url, machine.id))
        .header("fly-region", "local")
        .json(&stop_req)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_status!(response, StatusCode::OK);
    
    // Delete machine
    let response = client
        .delete(&format!("{}/v1/apps/lifecycle-app/machines/{}", url, machine.id))
        .header("fly-region", "local")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_status!(response, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_machine_metadata() {
    let (url, _handle) = start_test_server().await;
    let client = test_client();
    
    // Create app and machine
    let app_req = CreateAppRequest {
        name: "metadata-app".to_string(),
        org_slug: "test-org".to_string(),
    };
    
    client
        .post(&format!("{}/v1/apps", url))
        .header("fly-region", "local")
        .json(&app_req)
        .send()
        .await
        .expect("Failed to create app");
    
    let machine_req = CreateMachineRequest {
        name: Some("metadata-machine".to_string()),
        config: MachineConfig {
            image: "nginx:latest".to_string(),
            guest: None,
            env: HashMap::new(),
            services: vec![],
            mounts: vec![],
            size: MachineSize::Shared1x,
            restart: None,
            auto_destroy: false,
            schedule: None,
        },
        region: Some("local".to_string()),
        skip_launch: false,
        skip_service_registration: false,
    };
    
    let create_response = client
        .post(&format!("{}/v1/apps/metadata-app/machines", url))
        .header("fly-region", "local")
        .json(&machine_req)
        .send()
        .await
        .expect("Failed to create machine");
    
    let machine: Machine = create_response.json().await.expect("Failed to parse response");
    
    // Set metadata
    let metadata_value = json!({"version": "1.0", "env": "test"});
    
    let response = client
        .post(&format!("{}/v1/apps/metadata-app/machines/{}/metadata/deployment", url, machine.id))
        .header("fly-region", "local")
        .json(&metadata_value)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_status!(response, StatusCode::NO_CONTENT);
    
    // Get metadata
    let response = client
        .get(&format!("{}/v1/apps/metadata-app/machines/{}/metadata", url, machine.id))
        .header("fly-region", "local")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_status!(response, StatusCode::OK);
    
    let metadata: HashMap<String, serde_json::Value> = response.json().await.expect("Failed to parse response");
    assert!(metadata.contains_key("deployment"));
    assert_eq!(metadata.get("deployment").unwrap(), &metadata_value);
    
    // Delete metadata
    let response = client
        .delete(&format!("{}/v1/apps/metadata-app/machines/{}/metadata/deployment", url, machine.id))
        .header("fly-region", "local")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_status!(response, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_machine_lease() {
    let (url, _handle) = start_test_server().await;
    let client = test_client();
    
    // Create app and machine
    let app_req = CreateAppRequest {
        name: "lease-app".to_string(),
        org_slug: "test-org".to_string(),
    };
    
    client
        .post(&format!("{}/v1/apps", url))
        .header("fly-region", "local")
        .json(&app_req)
        .send()
        .await
        .expect("Failed to create app");
    
    let machine_req = CreateMachineRequest {
        name: Some("lease-machine".to_string()),
        config: MachineConfig {
            image: "nginx:latest".to_string(),
            guest: None,
            env: HashMap::new(),
            services: vec![],
            mounts: vec![],
            size: MachineSize::Shared1x,
            restart: None,
            auto_destroy: false,
            schedule: None,
        },
        region: Some("local".to_string()),
        skip_launch: false,
        skip_service_registration: false,
    };
    
    let create_response = client
        .post(&format!("{}/v1/apps/lease-app/machines", url))
        .header("fly-region", "local")
        .json(&machine_req)
        .send()
        .await
        .expect("Failed to create machine");
    
    let machine: Machine = create_response.json().await.expect("Failed to parse response");
    
    // Create lease
    let lease_req = CreateLeaseRequest {
        ttl_seconds: 60,
        description: Some("test lease".to_string()),
    };
    
    let response = client
        .post(&format!("{}/v1/apps/lease-app/machines/{}/lease", url, machine.id))
        .header("fly-region", "local")
        .json(&lease_req)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_status!(response, StatusCode::CREATED);
    
    let lease: Lease = response.json().await.expect("Failed to parse response");
    assert_eq!(lease.machine_id, machine.id);
    assert_eq!(lease.description, Some("test lease".to_string()));
    
    // Get lease
    let response = client
        .get(&format!("{}/v1/apps/lease-app/machines/{}/lease", url, machine.id))
        .header("fly-region", "local")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_status!(response, StatusCode::OK);
    
    // Release lease
    let response = client
        .delete(&format!("{}/v1/apps/lease-app/machines/{}/lease", url, machine.id))
        .header("fly-region", "local")
        .header("fly-machine-lease-nonce", &lease.nonce)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_status!(response, StatusCode::NO_CONTENT);
}
*/