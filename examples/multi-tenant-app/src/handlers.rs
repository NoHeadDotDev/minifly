use askama::Template;
use axum::response::IntoResponse;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{Html, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::error;

use crate::{
    config::Config,
    db,
    middleware::TenantId,
};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    tenants: Vec<String>,
}

impl IntoResponse for IndexTemplate {
    fn into_response(self) -> Response {
        match self.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", err),
            ).into_response(),
        }
    }
}

#[derive(Template)]
#[template(path = "tenant.html")]
struct TenantTemplate {
    tenant_name: String,
    items: Vec<Item>,
    item_count: i32,
}

impl IntoResponse for TenantTemplate {
    fn into_response(self) -> Response {
        match self.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", err),
            ).into_response(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Item {
    id: String,
    name: String,
    description: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
struct CreateItemRequest {
    name: String,
    description: Option<String>,
}

pub fn routes() -> Router<Config> {
    Router::new()
        .route("/", get(index_handler))
        .route("/api/tenants", get(list_tenants_api))
        .route("/tenant/:tenant", get(tenant_dashboard))
        .route("/tenant/:tenant/items", get(list_items).post(create_item))
        .route("/api/items", get(list_items_api).post(create_item_api))
        .route("/health", get(health_check))
}

async fn index_handler(State(config): State<Config>) -> Result<impl IntoResponse, AppError> {
    let tenants = db::list_all_tenants(&config.database_path).await?;
    
    Ok(IndexTemplate { tenants })
}

async fn tenant_dashboard(
    State(config): State<Config>,
    Path(tenant): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let pool = db::get_tenant_db(&tenant, &config.database_path).await?;
    
    // Get items for this tenant
    let items = sqlx::query_as::<_, Item>("SELECT * FROM items ORDER BY created_at DESC LIMIT 10")
        .fetch_all(&pool)
        .await?;
    
    // Get item count
    let row: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM items")
        .fetch_one(&pool)
        .await?;
    let item_count = row.0;
    
    Ok(TenantTemplate {
        tenant_name: tenant,
        items,
        item_count,
    })
}

async fn list_items(
    State(config): State<Config>,
    Extension(TenantId(tenant)): Extension<TenantId>,
) -> Result<Html<String>, AppError> {
    let pool = db::get_tenant_db(&tenant, &config.database_path).await?;
    
    let items = sqlx::query_as::<_, Item>("SELECT * FROM items ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await?;
    
    let html = format!(
        "<h2>Items for tenant: {}</h2><ul>{}</ul>",
        tenant,
        items
            .iter()
            .map(|item| format!("<li>{} - {}</li>", item.name, item.description.as_deref().unwrap_or("")))
            .collect::<Vec<_>>()
            .join("")
    );
    
    Ok(Html(html))
}

async fn create_item(
    State(config): State<Config>,
    Extension(TenantId(tenant)): Extension<TenantId>,
    Json(req): Json<CreateItemRequest>,
) -> Result<Response, AppError> {
    let pool = db::get_tenant_db(&tenant, &config.database_path).await?;
    
    let now = chrono::Utc::now().to_rfc3339();
    let item = Item {
        id: Uuid::new_v4().to_string(),
        name: req.name,
        description: req.description,
        created_at: now.clone(),
        updated_at: now.clone(),
    };
    
    sqlx::query(
        "INSERT INTO items (id, name, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&item.id)
    .bind(&item.name)
    .bind(&item.description)
    .bind(&item.created_at)
    .bind(&item.updated_at)
    .execute(&pool)
    .await?;
    
    // Update item count
    sqlx::query("UPDATE tenant_info SET item_count = item_count + 1 WHERE id = 1")
        .execute(&pool)
        .await?;
    
    Ok((StatusCode::CREATED, Json(item)).into_response())
}

async fn list_tenants_api(State(config): State<Config>) -> Result<Json<Vec<String>>, AppError> {
    let tenants = db::list_all_tenants(&config.database_path).await?;
    Ok(Json(tenants))
}

async fn list_items_api(
    State(config): State<Config>,
    Extension(TenantId(tenant)): Extension<TenantId>,
) -> Result<Json<Vec<Item>>, AppError> {
    let pool = db::get_tenant_db(&tenant, &config.database_path).await?;
    
    let items = sqlx::query_as::<_, Item>("SELECT * FROM items ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await?;
    
    Ok(Json(items))
}

async fn create_item_api(
    State(config): State<Config>,
    Extension(TenantId(tenant)): Extension<TenantId>,
    Json(req): Json<CreateItemRequest>,
) -> Result<Response, AppError> {
    create_item(State(config), Extension(TenantId(tenant)), Json(req)).await
}

async fn health_check() -> &'static str {
    "OK"
}

// Error handling
#[derive(Debug)]
struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!("Application error: {:?}", self.0);
        
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal server error: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}