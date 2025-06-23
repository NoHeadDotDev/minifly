use axum::{
    extract::{Multipart, Path, State},
    response::Redirect,
    Form,
};
use sqlx::{Pool, Sqlite};
use base64::{Engine as _, engine::general_purpose};

use crate::{
    auth::AuthUser,
    error::{AppError, AppResult},
    models::CreateTodoForm,
};

pub async fn create_todo(
    AuthUser(user): AuthUser,
    State(db): State<Pool<Sqlite>>,
    Form(form): Form<CreateTodoForm>,
) -> AppResult<Redirect> {
    if form.title.trim().is_empty() {
        return Err(AppError::Validation("Todo title cannot be empty".to_string()));
    }

    let todo_id = uuid::Uuid::new_v4().to_string();
    
    sqlx::query(
        "INSERT INTO todos (id, user_id, title, description) VALUES (?1, ?2, ?3, ?4)"
    )
    .bind(&todo_id)
    .bind(&user.id)
    .bind(form.title.trim())
    .bind(form.description.as_deref().map(|s| s.trim()))
    .execute(&db)
    .await?;

    Ok(Redirect::to("/dashboard"))
}

pub async fn toggle_todo(
    AuthUser(user): AuthUser,
    Path(todo_id): Path<String>,
    State(db): State<Pool<Sqlite>>,
) -> AppResult<Redirect> {
    // Verify ownership and toggle
    let result = sqlx::query(
        "UPDATE todos SET completed = NOT completed 
         WHERE id = ?1 AND user_id = ?2"
    )
    .bind(&todo_id)
    .bind(&user.id)
    .execute(&db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Redirect::to("/dashboard"))
}

pub async fn delete_todo(
    AuthUser(user): AuthUser,
    Path(todo_id): Path<String>,
    State(db): State<Pool<Sqlite>>,
) -> AppResult<Redirect> {
    // Verify ownership and delete
    let result = sqlx::query(
        "DELETE FROM todos WHERE id = ?1 AND user_id = ?2"
    )
    .bind(&todo_id)
    .bind(&user.id)
    .execute(&db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Redirect::to("/dashboard"))
}

pub async fn upload_image(
    AuthUser(user): AuthUser,
    Path(todo_id): Path<String>,
    State(db): State<Pool<Sqlite>>,
    mut multipart: Multipart,
) -> AppResult<Redirect> {
    let mut image_data = None;
    let mut mime_type = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::BadRequest(format!("Failed to read multipart: {}", e))
    })? {
        let name = field.name().unwrap_or_default();
        
        if name == "image" {
            let content_type = field.content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            
            // Validate content type
            if !content_type.starts_with("image/") {
                return Err(AppError::Validation("Only image files are allowed".to_string()));
            }
            
            // Read image data (limit to 5MB)
            let data = field.bytes().await.map_err(|e| {
                AppError::BadRequest(format!("Failed to read image: {}", e))
            })?;
            
            if data.len() > 5 * 1024 * 1024 {
                return Err(AppError::Validation("Image size must be less than 5MB".to_string()));
            }
            
            // Optionally resize image here using the `image` crate
            // For now, we'll just store the original
            
            // Encode to base64
            let encoded = general_purpose::STANDARD.encode(&data);
            
            image_data = Some(encoded);
            mime_type = Some(content_type);
            break;
        }
    }

    if let (Some(data), Some(mime)) = (image_data, mime_type) {
        // Verify ownership and update
        let result = sqlx::query(
            "UPDATE todos SET image_data = ?1, image_mime_type = ?2 
             WHERE id = ?3 AND user_id = ?4"
        )
        .bind(&data)
        .bind(&mime)
        .bind(&todo_id)
        .bind(&user.id)
        .execute(&db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
    } else {
        return Err(AppError::BadRequest("No image provided".to_string()));
    }

    Ok(Redirect::to("/dashboard"))
}