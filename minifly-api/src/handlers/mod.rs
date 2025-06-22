use axum::{
    Router,
    routing::{get, post, delete},
};
use crate::state::AppState;

mod admin;
mod apps;
mod logs;
mod machines;
mod volumes;
mod health;

pub fn routes() -> Router<AppState> {
    Router::new()
        // Administrative endpoints
        .route("/admin/shutdown", post(admin::shutdown))
        .route("/admin/status", get(admin::system_status))
        
        // Health endpoints
        .route("/health", get(health::health_check))
        .route("/health/comprehensive", get(health::comprehensive_health))
        .route("/health/live", get(health::liveness))
        .route("/health/ready", get(health::readiness))
        
        // Apps endpoints
        .route("/apps", post(apps::create_app))
        .route("/apps/:app_name", get(apps::get_app))
        .route("/apps/:app_name", delete(apps::delete_app))
        
        // Machines endpoints
        .route("/apps/:app_name/machines", get(machines::list_machines))
        .route("/apps/:app_name/machines", post(machines::create_machine))
        .route("/apps/:app_name/machines/:machine_id", get(machines::get_machine))
        .route("/apps/:app_name/machines/:machine_id", post(machines::update_machine))
        .route("/apps/:app_name/machines/:machine_id", delete(machines::delete_machine))
        .route("/apps/:app_name/machines/:machine_id/start", post(machines::start_machine))
        .route("/apps/:app_name/machines/:machine_id/stop", post(machines::stop_machine))
        .route("/apps/:app_name/machines/:machine_id/suspend", post(machines::suspend_machine))
        .route("/apps/:app_name/machines/:machine_id/wait", get(machines::wait_machine))
        
        // Lease endpoints
        .route("/apps/:app_name/machines/:machine_id/lease", post(machines::create_lease))
        .route("/apps/:app_name/machines/:machine_id/lease", get(machines::get_lease))
        .route("/apps/:app_name/machines/:machine_id/lease", delete(machines::release_lease))
        
        // Metadata endpoints
        .route("/apps/:app_name/machines/:machine_id/metadata", get(machines::get_metadata))
        .route("/apps/:app_name/machines/:machine_id/metadata/:key", post(machines::set_metadata))
        .route("/apps/:app_name/machines/:machine_id/metadata/:key", delete(machines::delete_metadata))
        
        // Log endpoints
        .route("/apps/:app_name/machines/:machine_id/logs", get(logs::stream_machine_logs))
        .route("/apps/:app_name/machines/:machine_id/logs/summary", get(logs::get_logs_summary))
        
        // Volume endpoints
        .route("/apps/:app_name/volumes", get(volumes::list_volumes))
        .route("/apps/:app_name/volumes", post(volumes::create_volume))
        .route("/apps/:app_name/volumes/:volume_id", get(volumes::get_volume))
        .route("/apps/:app_name/volumes/:volume_id", delete(volumes::delete_volume))
}