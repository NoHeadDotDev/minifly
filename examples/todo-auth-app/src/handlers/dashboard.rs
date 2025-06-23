use axum::{
    extract::{Path, State},
    response::Html,
};
use askama::Template;
use sqlx::{Pool, Sqlite};

use crate::{
    auth::AuthUser,
    error::{AppError, AppResult},
    models::{Todo, UserApp},
    templates::DashboardPage,
    tenant::get_tenant_app_url,
};

pub async fn dashboard(
    AuthUser(user): AuthUser,
    State(db): State<Pool<Sqlite>>,
) -> AppResult<Html<String>> {
    // Get user's todos
    let todos = sqlx::query_as::<_, Todo>(
        "SELECT * FROM todos WHERE user_id = ?1 ORDER BY created_at DESC"
    )
    .bind(&user.id)
    .fetch_all(&db)
    .await?;

    // Get user's apps
    let user_apps = sqlx::query_as::<_, UserApp>(
        "SELECT * FROM user_apps WHERE user_id = ?1 ORDER BY created_at"
    )
    .bind(&user.id)
    .fetch_all(&db)
    .await?;

    // Build region info with URLs
    let mut regions_with_info = Vec::new();
    for app in user_apps {
        let url = get_tenant_app_url(&app.app_name).await;
        regions_with_info.push((app, url));
    }

    let template = DashboardPage {
        user,
        todos,
        regions: regions_with_info,
        selected_region: None,
    };
    
    Ok(Html(template.render().unwrap()))
}

pub async fn region_dashboard(
    AuthUser(user): AuthUser,
    Path(region): Path<String>,
    State(db): State<Pool<Sqlite>>,
) -> AppResult<Html<String>> {
    // Verify user has access to this region
    let _user_app = sqlx::query_as::<_, UserApp>(
        "SELECT * FROM user_apps WHERE user_id = ?1 AND region = ?2"
    )
    .bind(&user.id)
    .bind(&region)
    .fetch_optional(&db)
    .await?
    .ok_or(AppError::NotFound)?;

    // Get todos (same as main dashboard for now)
    let todos = sqlx::query_as::<_, Todo>(
        "SELECT * FROM todos WHERE user_id = ?1 ORDER BY created_at DESC"
    )
    .bind(&user.id)
    .fetch_all(&db)
    .await?;

    // Get all user apps for sidebar
    let all_user_apps = sqlx::query_as::<_, UserApp>(
        "SELECT * FROM user_apps WHERE user_id = ?1 ORDER BY created_at"
    )
    .bind(&user.id)
    .fetch_all(&db)
    .await?;

    let mut regions_with_info = Vec::new();
    for app in all_user_apps {
        let url = get_tenant_app_url(&app.app_name).await;
        regions_with_info.push((app, url));
    }

    let template = DashboardPage {
        user,
        todos,
        regions: regions_with_info,
        selected_region: Some(region),
    };
    
    Ok(Html(template.render().unwrap()))
}