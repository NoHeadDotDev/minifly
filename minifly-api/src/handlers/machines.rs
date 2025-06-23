use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use chrono::Utc;
use minifly_core::models::{
    Machine, MachineState, MachineEvent, ImageRef,
    CreateMachineRequest, UpdateMachineRequest, StopMachineRequest,
    StartMachineResponse, StopMachineResponse, WaitMachineQuery,
    CreateLeaseRequest, LeaseResponse, Lease,
};
use minifly_core::{SuccessResponse, Error as CoreError};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{info, instrument};
use crate::state::AppState;
use crate::error::{ApiError, Result};
use crate::middleware::region::{log_machine_operation, get_machine_region};
use minifly_network::extract_container_ip;

pub async fn list_machines(
    State(state): State<AppState>,
    Path(app_name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Machine>>> {
    let machines = state.machines.read().unwrap();
    
    let mut result: Vec<Machine> = machines.values()
        .filter(|m| m.name.starts_with(&format!("{}-", app_name)))
        .cloned()
        .collect();
    
    // Apply filters
    if let Some(region) = params.get("region") {
        result.retain(|m| &m.region == region);
    }
    
    if let Some(include_deleted) = params.get("include_deleted") {
        if include_deleted != "true" {
            result.retain(|m| m.state != MachineState::Destroyed);
        }
    }
    
    Ok(Json(result))
}

#[instrument(skip(state), fields(app_name = %app_name, region = tracing::field::Empty))]
pub async fn create_machine(
    State(state): State<AppState>,
    Path(app_name): Path<String>,
    Json(req): Json<CreateMachineRequest>,
) -> Result<Json<Machine>> {
    let machine_id = state.generate_machine_id();
    let instance_id = state.generate_instance_id();
    let machine_index = state.machines.read().unwrap().len() as u32;
    let private_ip = state.generate_private_ip(&app_name, machine_index);
    
    // Get region from request or use default
    let region = get_machine_region(req.region.as_deref());
    tracing::Span::current().record("region", &region);
    
    info!(
        machine_id = %machine_id,
        app_name = %app_name,
        region = %region,
        image = %req.config.image,
        "Creating machine"
    );
    
    // Parse image reference
    let image_parts: Vec<&str> = req.config.image.split('/').collect();
    let (registry, repository, tag_digest) = match image_parts.len() {
        1 => ("registry-1.docker.io", "library", image_parts[0]),
        2 => ("registry-1.docker.io", image_parts[0], image_parts[1]),
        _ => (image_parts[0], image_parts[1], image_parts[2]),
    };
    
    let (tag, digest) = if let Some((t, d)) = tag_digest.split_once('@') {
        (t.to_string(), Some(d.to_string()))
    } else if let Some((_repo, t)) = tag_digest.split_once(':') {
        (t.to_string(), None)
    } else {
        ("latest".to_string(), None)
    };
    
    let machine = Machine {
        id: machine_id.clone(),
        name: req.name.unwrap_or_else(|| format!("{}-{}", app_name, machine_id)),
        state: if req.skip_launch.unwrap_or(false) {
            MachineState::Created
        } else {
            MachineState::Starting
        },
        region: region.clone(),
        image_ref: ImageRef {
            registry: registry.to_string(),
            repository: repository.to_string(),
            tag,
            digest,
        },
        instance_id: instance_id.clone(),
        private_ip,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        config: req.config.clone(),
        events: vec![MachineEvent {
            event_type: "launch".to_string(),
            status: "created".to_string(),
            source: "user".to_string(),
            timestamp: Utc::now().timestamp_millis() as u64,
        }],
    };
    
    // Create container
    if !req.skip_launch.unwrap_or(false) {
        // Start LiteFS if volumes are configured
        let has_volumes = req.config.mounts.as_ref()
            .map(|mounts| !mounts.is_empty())
            .unwrap_or(false);
        
        if has_volumes {
            let is_primary = req.config.env.as_ref()
                .and_then(|env| env.get("FLY_LITEFS_PRIMARY"))
                .map(|v| v == "true")
                .unwrap_or(true);
            
            if let Err(e) = state.litefs.start_for_machine_with_config(&machine_id, is_primary, Some(&app_name)).await {
                return Err(CoreError::LiteFSError(format!("Failed to start LiteFS: {}", e)).into());
            }
        }
        
        match state.docker.create_container(&machine_id, &app_name, &req.config).await {
            Ok(container_id) => {
                // Start container
                if let Err(e) = state.docker.start_container(&container_id).await {
                    // Clean up LiteFS if container start failed
                    if has_volumes {
                        let _ = state.litefs.stop_for_machine(&machine_id).await;
                    }
                    return Err(CoreError::DockerError(format!("Failed to start container: {}", e)).into());
                }
                
                // Wait a moment for container to get IP
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                
                // Get container IP and register with DNS
                if let Ok(container_info) = state.docker.inspect_container(&container_id).await {
                    if let Some(network_settings) = container_info.network_settings {
                        if let Some(networks) = network_settings.networks {
                            let networks_value = serde_json::to_value(&networks).unwrap_or_default();
                            if let Some(ip) = extract_container_ip(&networks_value) {
                                // Register with DNS resolver
                                if let Err(e) = state.dns_resolver.register_machine(&app_name, &machine_id, ip).await {
                                    tracing::warn!("Failed to register machine with DNS: {}", e);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                // Clean up LiteFS if container creation failed
                if has_volumes {
                    let _ = state.litefs.stop_for_machine(&machine_id).await;
                }
                return Err(CoreError::DockerError(format!("Failed to create container: {}", e)).into());
            }
        }
    }
    
    // Store machine
    state.machines.write().unwrap().insert(machine_id.clone(), machine.clone());
    
    // Handle lease if requested
    if let Some(ttl) = req.lease_ttl {
        let lease = create_machine_lease(&state, &machine_id, ttl, None);
        state.leases.write().unwrap().insert(machine_id.clone(), lease);
    }
    
    // Log successful creation
    log_machine_operation("create", &machine_id, &app_name, &region);
    
    info!(
        machine_id = %machine_id,
        app_name = %app_name,
        region = %region,
        state = ?machine.state,
        "Machine created successfully"
    );
    
    Ok(Json(machine))
}

pub async fn get_machine(
    State(state): State<AppState>,
    Path((_app_name, machine_id)): Path<(String, String)>,
) -> Result<Json<Machine>> {
    let machines = state.machines.read().unwrap();
    
    match machines.get(&machine_id) {
        Some(machine) => Ok(Json(machine.clone())),
        None => Err(CoreError::MachineNotFound(machine_id).into()),
    }
}

pub async fn update_machine(
    State(state): State<AppState>,
    Path((_app_name, machine_id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(req): Json<UpdateMachineRequest>,
) -> Result<Json<Machine>> {
    // Check lease if provided
    if let Some(nonce) = headers.get("fly-machine-lease-nonce") {
        let leases = state.leases.read().unwrap();
        if let Some(lease) = leases.get(&machine_id) {
            if lease.nonce != nonce.to_str().unwrap_or("") {
                return Err(CoreError::InvalidLeaseNonce.into());
            }
        }
    }
    
    let mut machines = state.machines.write().unwrap();
    
    match machines.get_mut(&machine_id) {
        Some(machine) => {
            machine.config = req.config;
            machine.updated_at = Utc::now();
            
            Ok(Json(machine.clone()))
        }
        None => Err(CoreError::MachineNotFound(machine_id).into()),
    }
}

pub async fn delete_machine(
    State(state): State<AppState>,
    Path((app_name, machine_id)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<SuccessResponse>> {
    let force = params.get("force").map(|v| v == "true").unwrap_or(false);
    
    // Check if machine exists and needs container operations
    let needs_stop = {
        let machines = state.machines.read().unwrap();
        match machines.get(&machine_id) {
            Some(machine) => machine.state == MachineState::Started || force,
            None => return Err(CoreError::MachineNotFound(machine_id.clone()).into()),
        }
    };
    
    // Stop and remove container if needed
    if needs_stop {
        let container_name = format!("minifly-{}-{}", app_name, machine_id);
        if let Err(e) = state.docker.stop_container(&container_name, Some(30)).await {
            if !force {
                return Err(CoreError::DockerError(format!("Failed to stop container: {}", e)).into());
            }
        }
        
        if let Err(e) = state.docker.remove_container(&container_name).await {
            return Err(CoreError::DockerError(format!("Failed to remove container: {}", e)).into());
        }
        
        // Stop LiteFS if running
        if state.litefs.is_running(&machine_id).await {
            if let Err(e) = state.litefs.stop_for_machine(&machine_id).await {
                if !force {
                    return Err(CoreError::LiteFSError(format!("Failed to stop LiteFS: {}", e)).into());
                }
            }
        }
    }
    
    // Update state and remove machine
    {
        let mut machines = state.machines.write().unwrap();
        if let Some(machine) = machines.get_mut(&machine_id) {
            machine.state = MachineState::Destroyed;
        }
        machines.remove(&machine_id);
    }
    
    // Unregister from DNS
    if let Err(e) = state.dns_resolver.unregister_machine(&app_name, &machine_id).await {
        tracing::warn!("Failed to unregister machine from DNS: {}", e);
    }
    
    Ok(Json(SuccessResponse { ok: true }))
}

#[instrument(skip(state), fields(app_name = %app_name, machine_id = %machine_id, region = tracing::field::Empty))]
pub async fn start_machine(
    State(state): State<AppState>,
    Path((app_name, machine_id)): Path<(String, String)>,
) -> Result<Json<StartMachineResponse>> {
    // Get previous state and region without holding the lock
    let (previous_state, region) = {
        let machines = state.machines.read().unwrap();
        match machines.get(&machine_id) {
            Some(machine) => (
                format!("{:?}", machine.state).to_lowercase(),
                machine.region.clone()
            ),
            None => return Err(CoreError::MachineNotFound(machine_id.clone()).into()),
        }
    };
    
    // Record region in tracing span
    tracing::Span::current().record("region", &region);
    
    info!(
        machine_id = %machine_id,
        app_name = %app_name,
        region = %region,
        previous_state = %previous_state,
        "Starting machine"
    );
    
    // Start container
    let container_name = format!("minifly-{}-{}", app_name, machine_id);
    if let Err(e) = state.docker.start_container(&container_name).await {
        return Err(CoreError::DockerError(format!("Failed to start container: {}", e)).into());
    }
    
    // Re-register with DNS after starting
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    if let Ok(container_info) = state.docker.inspect_container(&container_name).await {
        if let Some(network_settings) = container_info.network_settings {
            if let Some(networks) = network_settings.networks {
                let networks_value = serde_json::to_value(&networks).unwrap_or_default();
                if let Some(ip) = extract_container_ip(&networks_value) {
                    // Register with DNS resolver
                    if let Err(e) = state.dns_resolver.register_machine(&app_name, &machine_id, ip).await {
                        tracing::warn!("Failed to register machine with DNS: {}", e);
                    }
                }
            }
        }
    }
    
    // Update machine state
    {
        let mut machines = state.machines.write().unwrap();
        if let Some(machine) = machines.get_mut(&machine_id) {
            machine.state = MachineState::Started;
            machine.updated_at = Utc::now();
            machine.events.push(MachineEvent {
                event_type: "start".to_string(),
                status: "started".to_string(),
                source: "user".to_string(),
                timestamp: Utc::now().timestamp_millis() as u64,
            });
        }
    }
    
    // Log successful start
    log_machine_operation("start", &machine_id, &app_name, &region);
    
    info!(
        machine_id = %machine_id,
        app_name = %app_name,
        region = %region,
        "Machine started successfully"
    );
    
    Ok(Json(StartMachineResponse {
        previous_state,
        migrated: false,
        new_host: String::new(),
    }))
}

pub async fn stop_machine(
    State(state): State<AppState>,
    Path((app_name, machine_id)): Path<(String, String)>,
    req: Option<Json<StopMachineRequest>>,
) -> Result<Json<StopMachineResponse>> {
    // Check if machine exists
    {
        let machines = state.machines.read().unwrap();
        if !machines.contains_key(&machine_id) {
            return Err(CoreError::MachineNotFound(machine_id.clone()).into());
        }
    }
    
    let timeout = req.as_ref()
        .and_then(|Json(r)| r.timeout.as_ref())
        .and_then(|t| t.parse::<i64>().ok())
        .unwrap_or(30);
    
    // Stop container
    let container_name = format!("minifly-{}-{}", app_name, machine_id);
    if let Err(e) = state.docker.stop_container(&container_name, Some(timeout)).await {
        return Err(CoreError::DockerError(format!("Failed to stop container: {}", e)).into());
    }
    
    // Update machine state
    {
        let mut machines = state.machines.write().unwrap();
        if let Some(machine) = machines.get_mut(&machine_id) {
            machine.state = MachineState::Stopped;
            machine.updated_at = Utc::now();
            machine.events.push(MachineEvent {
                event_type: "stop".to_string(),
                status: "stopped".to_string(),
                source: "user".to_string(),
                timestamp: Utc::now().timestamp_millis() as u64,
            });
        }
    }
    
    // Unregister from DNS when stopped
    if let Err(e) = state.dns_resolver.unregister_machine(&app_name, &machine_id).await {
        tracing::warn!("Failed to unregister machine from DNS: {}", e);
    }
    
    Ok(Json(StopMachineResponse { ok: true }))
}

pub async fn suspend_machine(
    State(state): State<AppState>,
    Path((app_name, machine_id)): Path<(String, String)>,
) -> Result<Json<SuccessResponse>> {
    // Check if machine exists
    {
        let machines = state.machines.read().unwrap();
        if !machines.contains_key(&machine_id) {
            return Err(CoreError::MachineNotFound(machine_id.clone()).into());
        }
    }
    
    // Note: Docker doesn't support true suspend, so we'll just stop the container
    let container_name = format!("minifly-{}-{}", app_name, machine_id);
    if let Err(e) = state.docker.stop_container(&container_name, Some(30)).await {
        return Err(CoreError::DockerError(format!("Failed to suspend container: {}", e)).into());
    }
    
    // Update machine state
    {
        let mut machines = state.machines.write().unwrap();
        if let Some(machine) = machines.get_mut(&machine_id) {
            machine.state = MachineState::Suspended;
            machine.updated_at = Utc::now();
        }
    }
    
    Ok(Json(SuccessResponse { ok: true }))
}

pub async fn wait_machine(
    State(state): State<AppState>,
    Path((_app_name, machine_id)): Path<(String, String)>,
    Query(_query): Query<WaitMachineQuery>,
) -> Result<Json<Value>> {
    let machines = state.machines.read().unwrap();
    
    match machines.get(&machine_id) {
        Some(machine) => {
            // In a real implementation, this would wait for state changes
            // For now, we'll just return the current state
            Ok(Json(json!({
                "ok": true,
                "state": format!("{:?}", machine.state).to_lowercase(),
                "instance_id": machine.instance_id,
            })))
        }
        None => Err(CoreError::MachineNotFound(machine_id).into()),
    }
}

pub async fn create_lease(
    State(state): State<AppState>,
    Path((_app_name, machine_id)): Path<(String, String)>,
    Json(req): Json<CreateLeaseRequest>,
) -> Result<Json<LeaseResponse>> {
    let machines = state.machines.read().unwrap();
    
    if !machines.contains_key(&machine_id) {
        return Err(CoreError::MachineNotFound(machine_id.clone()).into());
    }
    
    let mut leases = state.leases.write().unwrap();
    
    // Check if lease already exists
    if leases.contains_key(&machine_id) {
        return Err(CoreError::LeaseConflict.into());
    }
    
    let lease = create_machine_lease(&state, &machine_id, req.ttl.unwrap_or(300), req.description);
    leases.insert(machine_id.clone(), lease.clone());
    
    Ok(Json(LeaseResponse {
        status: "success".to_string(),
        data: lease,
    }))
}

pub async fn get_lease(
    State(state): State<AppState>,
    Path((_app_name, machine_id)): Path<(String, String)>,
) -> Result<Json<LeaseResponse>> {
    let leases = state.leases.read().unwrap();
    
    match leases.get(&machine_id) {
        Some(lease) => Ok(Json(LeaseResponse {
            status: "success".to_string(),
            data: lease.clone(),
        })),
        None => Err(CoreError::NotFound.into()),
    }
}

pub async fn release_lease(
    State(state): State<AppState>,
    Path((_app_name, machine_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<SuccessResponse>> {
    let nonce = headers.get("fly-machine-lease-nonce")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::from(CoreError::BadRequest("Missing lease nonce".to_string())))?;
    
    let mut leases = state.leases.write().unwrap();
    
    match leases.get(&machine_id) {
        Some(lease) => {
            if lease.nonce != nonce {
                return Err(CoreError::InvalidLeaseNonce.into());
            }
            
            leases.remove(&machine_id);
            Ok(Json(SuccessResponse { ok: true }))
        }
        None => Err(CoreError::NotFound.into()),
    }
}

pub async fn get_metadata(
    State(_state): State<AppState>,
    Path((_app_name, _machine_id)): Path<(String, String)>,
) -> Result<Json<Value>> {
    // TODO: Implement metadata storage
    Ok(Json(json!({})))
}

pub async fn set_metadata(
    State(_state): State<AppState>,
    Path((_app_name, _machine_id, _key)): Path<(String, String, String)>,
    Json(_value): Json<Value>,
) -> Result<Json<SuccessResponse>> {
    // TODO: Implement metadata storage
    Ok(Json(SuccessResponse { ok: true }))
}

pub async fn delete_metadata(
    State(_state): State<AppState>,
    Path((_app_name, _machine_id, _key)): Path<(String, String, String)>,
) -> Result<Json<SuccessResponse>> {
    // TODO: Implement metadata storage
    Ok(Json(SuccessResponse { ok: true }))
}

fn create_machine_lease(_state: &AppState, _machine_id: &str, ttl: u32, description: Option<String>) -> Lease {
    use rand::Rng;
    use uuid::Uuid;
    let mut rng = rand::thread_rng();
    let nonce: String = (0..12)
        .map(|_| format!("{:x}", rng.gen::<u8>()))
        .collect();
    
    Lease {
        nonce,
        expires_at: Utc::now().timestamp() + ttl as i64,
        owner: "minifly@local".to_string(),
        description: description.unwrap_or_default(),
        version: format!("01{}", Uuid::new_v4().simple()),
    }
}