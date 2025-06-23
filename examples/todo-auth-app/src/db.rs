use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::path::Path;

pub async fn init_db(database_url: &str) -> Result<Pool<Sqlite>, sqlx::Error> {
    // Extract path from URL
    let db_path = database_url.strip_prefix("sqlite://").unwrap_or(database_url);
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = Path::new(db_path).parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }

    // Create database pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}