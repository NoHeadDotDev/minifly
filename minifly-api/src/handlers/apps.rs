use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use minifly_core::models::{App, AppStatus, CreateAppRequest, AppResponse, Organization};
use minifly_core::{SuccessResponse, Error as CoreError};
use uuid::Uuid;
use crate::state::AppState;
use crate::error::Result;

pub async fn create_app(
    State(state): State<AppState>,
    Json(req): Json<CreateAppRequest>,
) -> Result<Json<AppResponse>> {
    let app = App {
        id: Uuid::new_v4(),
        name: req.app_name.clone(),
        organization_id: req.org_slug.clone(),
        status: AppStatus::Pending,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Store in memory (in production, this would be in the database)
    state.apps.write().unwrap().insert(app.name.clone(), app.clone());
    
    let response = AppResponse {
        id: app.id.to_string(),
        name: app.name,
        organization: Organization {
            id: Uuid::new_v4().to_string(),
            slug: req.org_slug,
            name: "Default Organization".to_string(),
        },
        status: "pending".to_string(),
        created_at: app.created_at.to_rfc3339(),
    };
    
    Ok(Json(response))
}

pub async fn get_app(
    State(state): State<AppState>,
    Path(app_name): Path<String>,
) -> Result<Json<AppResponse>> {
    let apps = state.apps.read().unwrap();
    
    match apps.get(&app_name) {
        Some(app) => {
            let response = AppResponse {
                id: app.id.to_string(),
                name: app.name.clone(),
                organization: Organization {
                    id: Uuid::new_v4().to_string(),
                    slug: app.organization_id.clone(),
                    name: "Default Organization".to_string(),
                },
                status: match app.status {
                    AppStatus::Pending => "pending",
                    AppStatus::Deployed => "deployed",
                    AppStatus::Suspended => "suspended",
                }.to_string(),
                created_at: app.created_at.to_rfc3339(),
            };
            
            Ok(Json(response))
        }
        None => Err(CoreError::AppNotFound(app_name).into()),
    }
}

pub async fn delete_app(
    State(state): State<AppState>,
    Path(app_name): Path<String>,
) -> Result<Json<SuccessResponse>> {
    let mut apps = state.apps.write().unwrap();
    
    match apps.remove(&app_name) {
        Some(_) => Ok(Json(SuccessResponse { ok: true })),
        None => Err(CoreError::AppNotFound(app_name).into()),
    }
}