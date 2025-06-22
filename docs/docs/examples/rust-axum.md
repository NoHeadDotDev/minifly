# Rust + Axum Example

A complete example of building a multi-tenant web application with Rust, Axum, and LiteFS.

## Quick Start

```bash
# Create new project
minifly init
# Choose: Rust + Axum + LiteFS

# Start the platform
minifly serve

# Deploy the application
minifly deploy
```

## Project Structure

```
my-app/
├── Cargo.toml
├── Dockerfile
├── fly.toml
├── litefs.yml
├── src/
│   └── main.rs
├── templates/
│   └── index.html
├── migrations/
│   └── 001_initial.sql
└── data/
    └── litefs/
```

## Key Features

- Multi-tenant architecture
- SQLite with LiteFS replication
- Askama templating
- Structured logging
- Health checks
- Docker containerization

## Code Example

```rust
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use sqlx::SqlitePool;

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to tenant-specific database
    let db_path = "/litefs/primary.db";
    let pool = SqlitePool::connect(&db_path).await?;
    
    let app = Router::new()
        .route("/", get(index))
        .route("/tenant/:id", get(tenant_page))
        .route("/health", get(health_check))
        .with_state(pool);
    
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

## Deployment

```toml
# fly.toml
app = "my-rust-app"
primary_region = "sjc"

[build]
  dockerfile = "Dockerfile"

[env]
  DATABASE_URL = "/litefs/primary.db"

[[services]]
  internal_port = 8080
  protocol = "tcp"
```

## Multi-Tenant Pattern

Each tenant gets isolated data:

```rust
async fn get_tenant_data(
    Path(tenant_id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<User>>> {
    let users = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE tenant_id = ?",
        tenant_id
    )
    .fetch_all(&pool)
    .await?;
    
    Ok(Json(users))
}
```

## Learn More

- [Full source code](https://github.com/NoHeadDotDev/minifly/tree/main/examples/multi-tenant-app)
- [Axum documentation](https://docs.rs/axum)
- [LiteFS documentation](https://fly.io/docs/litefs)