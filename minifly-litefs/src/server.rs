use axum::{
    extract::{Path, State as AxumState, Query},
    routing::{get, post, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use crate::manager::LiteFSManager;
use minifly_core::Error;
use crate::Result;

#[derive(Clone)]
pub struct ServerState {
    manager: Arc<LiteFSManager>,
}

#[derive(Debug, Serialize)]
pub struct LiteFSStatus {
    pub machine_id: String,
    pub is_running: bool,
    pub is_primary: bool,
    pub mount_path: String,
    pub proxy_url: String,
}

#[derive(Debug, Deserialize)]
pub struct StartRequest {
    pub is_primary: bool,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub fn create_router(manager: Arc<LiteFSManager>) -> Router {
    let state = ServerState { manager };
    
    Router::new()
        .route("/health", get(health_check))
        .route("/instances", get(list_instances))
        .route("/instances/:machine_id", get(get_instance))
        .route("/instances/:machine_id/start", post(start_instance))
        .route("/instances/:machine_id/stop", post(stop_instance))
        .route("/instances/:machine_id/status", get(get_status))
        .with_state(state)
}

async fn health_check() -> Json<ApiResponse<String>> {
    Json(ApiResponse {
        success: true,
        data: Some("LiteFS server is running".to_string()),
        error: None,
    })
}

async fn list_instances(
    AxumState(state): AxumState<ServerState>,
) -> Json<ApiResponse<Vec<String>>> {
    // TODO: Implement listing of all LiteFS instances
    Json(ApiResponse {
        success: true,
        data: Some(vec![]),
        error: None,
    })
}

async fn get_instance(
    AxumState(state): AxumState<ServerState>,
    Path(machine_id): Path<String>,
) -> Json<ApiResponse<LiteFSStatus>> {
    let is_running = state.manager.is_running(&machine_id).await;
    
    if !is_running {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some("LiteFS instance not found".to_string()),
        });
    }
    
    let status = LiteFSStatus {
        machine_id: machine_id.clone(),
        is_running,
        is_primary: true, // TODO: Get actual primary status
        mount_path: state.manager.get_mount_path(&machine_id).to_string_lossy().to_string(),
        proxy_url: state.manager.get_proxy_url(&machine_id),
    };
    
    Json(ApiResponse {
        success: true,
        data: Some(status),
        error: None,
    })
}

async fn start_instance(
    AxumState(state): AxumState<ServerState>,
    Path(machine_id): Path<String>,
    Json(req): Json<StartRequest>,
) -> Json<ApiResponse<LiteFSStatus>> {
    match state.manager.start_for_machine(&machine_id, req.is_primary).await {
        Ok(_) => {
            let status = LiteFSStatus {
                machine_id: machine_id.clone(),
                is_running: true,
                is_primary: req.is_primary,
                mount_path: state.manager.get_mount_path(&machine_id).to_string_lossy().to_string(),
                proxy_url: state.manager.get_proxy_url(&machine_id),
            };
            
            Json(ApiResponse {
                success: true,
                data: Some(status),
                error: None,
            })
        }
        Err(e) => {
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            })
        }
    }
}

async fn stop_instance(
    AxumState(state): AxumState<ServerState>,
    Path(machine_id): Path<String>,
) -> Json<ApiResponse<String>> {
    match state.manager.stop_for_machine(&machine_id).await {
        Ok(_) => {
            Json(ApiResponse {
                success: true,
                data: Some(format!("LiteFS instance {} stopped", machine_id)),
                error: None,
            })
        }
        Err(e) => {
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            })
        }
    }
}

async fn get_status(
    AxumState(state): AxumState<ServerState>,
    Path(machine_id): Path<String>,
) -> Json<ApiResponse<bool>> {
    let is_running = state.manager.is_running(&machine_id).await;
    
    Json(ApiResponse {
        success: true,
        data: Some(is_running),
        error: None,
    })
}

pub async fn run_server(manager: Arc<LiteFSManager>, port: u16) -> Result<()> {
    let app = create_router(manager);
    
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    
    let listener = tokio::net::TcpListener::bind(addr).await
        .map_err(|e| Error::LiteFSError(format!("Failed to bind to port {}: {}", port, e)))?;
    
    tracing::info!("LiteFS HTTP server listening on {}", addr);
    
    axum::serve(listener, app).await
        .map_err(|e| Error::LiteFSError(format!("Server error: {}", e)))?;
    
    Ok(())
}