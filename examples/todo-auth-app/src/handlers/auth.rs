use axum::{
    extract::State,
    response::{Html, Redirect},
    Form,
};
use askama::Template;
use sqlx::{Pool, Sqlite};
use tower_sessions::Session;

use crate::{
    auth::{hash_password, set_session_user, verify_password, clear_session},
    error::{AppError, AppResult},
    models::{LoginForm, SignupForm, User, AVAILABLE_REGIONS},
    templates::{LoginPage, SignupPage},
    tenant::provision_tenant_app,
};

pub async fn login_page() -> AppResult<Html<String>> {
    let template = LoginPage {
        error: None,
        email: String::new(),
    };
    Ok(Html(template.render().unwrap()))
}

pub async fn signup_page() -> AppResult<Html<String>> {
    let template = SignupPage {
        error: None,
        email: String::new(),
        regions: AVAILABLE_REGIONS,
    };
    Ok(Html(template.render().unwrap()))
}

pub async fn login(
    State(db): State<Pool<Sqlite>>,
    session: Session,
    Form(form): Form<LoginForm>,
) -> AppResult<Redirect> {
    // Validate email
    if form.email.is_empty() || !form.email.contains('@') {
        return Err(AppError::Validation("Invalid email address".to_string()));
    }

    // Find user
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = ?1"
    )
    .bind(&form.email)
    .fetch_optional(&db)
    .await?
    .ok_or_else(|| AppError::Auth("Invalid email or password".to_string()))?;

    // Verify password
    if !verify_password(&form.password, &user.password_hash)? {
        return Err(AppError::Auth("Invalid email or password".to_string()));
    }

    // Set session
    set_session_user(&session, &user).await?;

    Ok(Redirect::to("/dashboard"))
}

pub async fn signup(
    State(db): State<Pool<Sqlite>>,
    session: Session,
    Form(form): Form<SignupForm>,
) -> AppResult<Redirect> {
    // Validate input
    if form.email.is_empty() || !form.email.contains('@') {
        return Err(AppError::Validation("Invalid email address".to_string()));
    }
    
    if form.password.len() < 8 {
        return Err(AppError::Validation("Password must be at least 8 characters".to_string()));
    }
    
    if !AVAILABLE_REGIONS.iter().any(|(code, _)| code == &form.region) {
        return Err(AppError::Validation("Invalid region selected".to_string()));
    }

    // Check if user already exists
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE email = ?1"
    )
    .bind(&form.email)
    .fetch_one(&db)
    .await?;

    if existing > 0 {
        return Err(AppError::Auth("Email already registered".to_string()));
    }

    // Hash password
    let password_hash = hash_password(&form.password)?;

    // Create user
    let user_id = uuid::Uuid::new_v4().to_string();
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (id, email, password_hash) VALUES (?1, ?2, ?3) RETURNING *"
    )
    .bind(&user_id)
    .bind(&form.email)
    .bind(&password_hash)
    .fetch_one(&db)
    .await?;

    // Provision tenant app in selected region
    match provision_tenant_app(&user.id, &user.email, &form.region).await {
        Ok((app_name, machine_id)) => {
            // Save tenant app info
            sqlx::query(
                "INSERT INTO user_apps (user_id, app_name, region, machine_id, status) 
                 VALUES (?1, ?2, ?3, ?4, ?5)"
            )
            .bind(&user.id)
            .bind(&app_name)
            .bind(&form.region)
            .bind(&machine_id)
            .bind("active")
            .execute(&db)
            .await?;
        }
        Err(e) => {
            // Log error but don't fail signup
            tracing::error!("Failed to provision tenant app: {}", e);
            
            // Save pending app info
            sqlx::query(
                "INSERT INTO user_apps (user_id, app_name, region, status) 
                 VALUES (?1, ?2, ?3, ?4)"
            )
            .bind(&user.id)
            .bind(format!("todo-user-{}", &user.id[..8]))
            .bind(&form.region)
            .bind("pending")
            .execute(&db)
            .await?;
        }
    }

    // Set session
    set_session_user(&session, &user).await?;

    Ok(Redirect::to("/dashboard"))
}

pub async fn logout(session: Session) -> AppResult<Redirect> {
    clear_session(&session).await?;
    Ok(Redirect::to("/"))
}