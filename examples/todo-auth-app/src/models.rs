use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserApp {
    pub id: String,
    pub user_id: String,
    pub app_name: String,
    pub region: String,
    pub machine_id: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub image_data: Option<String>,
    pub image_mime_type: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SignupForm {
    pub email: String,
    pub password: String,
    pub region: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTodoForm {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionUser {
    pub id: String,
    pub email: String,
}

pub const AVAILABLE_REGIONS: &[(&str, &str)] = &[
    ("iad", "Ashburn, Virginia (US)"),
    ("ord", "Chicago, Illinois (US)"),
    ("lax", "Los Angeles, California (US)"),
    ("lhr", "London, United Kingdom"),
    ("ams", "Amsterdam, Netherlands"),
    ("fra", "Frankfurt, Germany"),
    ("syd", "Sydney, Australia"),
    ("nrt", "Tokyo, Japan"),
    ("sin", "Singapore"),
];