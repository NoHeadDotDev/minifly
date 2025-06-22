use anyhow::{Result, Context};
use colored::*;
use dialoguer::{Confirm, Input, Password, Select};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::config::Config;

/// Project template information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectTemplate {
    name: String,
    description: String,
    language: String,
    framework: String,
    features: Vec<String>,
    files: HashMap<String, String>,
}

/// Handle the enhanced init command with project templates
/// 
/// This provides multiple initialization modes:
/// 1. Basic configuration setup
/// 2. Project scaffolding with templates
/// 3. Multi-tenant example setup
/// 
/// # Arguments
/// * `config` - Current Minifly configuration
pub async fn handle(config: &Config) -> Result<()> {
    println!("{}", "🚀 Welcome to Minifly!".bold().blue());
    println!("Local Fly.io development simulator with incredible DX\n");
    
    // Check if this is a new project or configuration update
    let init_mode = if Path::new("fly.toml").exists() || Path::new("Dockerfile").exists() {
        println!("📁 Existing project detected!");
        let choices = vec![
            "Update Minifly configuration only",
            "Add Minifly project templates",
            "Reinitialize with new template",
            "Cancel"
        ];
        
        let selection = Select::new()
            .with_prompt("What would you like to do?")
            .items(&choices)
            .default(0)
            .interact()?;
            
        match selection {
            0 => InitMode::ConfigOnly,
            1 => InitMode::AddTemplates,
            2 => InitMode::Reinitialize,
            _ => return Ok(()),
        }
    } else {
        println!("📦 Setting up a new Minifly project!");
        let choices = vec![
            "Initialize with project template",
            "Configuration setup only",
        ];
        
        let selection = Select::new()
            .with_prompt("How would you like to initialize?")
            .items(&choices)
            .default(0)
            .interact()?;
            
        match selection {
            0 => InitMode::NewProject,
            _ => InitMode::ConfigOnly,
        }
    };
    
    // Handle configuration setup
    let new_config = setup_configuration(config).await?;
    
    // Handle project setup based on mode
    match init_mode {
        InitMode::ConfigOnly => {
            println!("\n{}", "✅ Configuration saved!".green());
        }
        InitMode::NewProject => {
            setup_new_project(&new_config).await?;
        }
        InitMode::AddTemplates => {
            add_project_templates().await?;
        }
        InitMode::Reinitialize => {
            if Confirm::new()
                .with_prompt("This will overwrite existing files. Continue?")
                .default(false)
                .interact()? 
            {
                setup_new_project(&new_config).await?;
            }
        }
    }
    
    // Show getting started information
    show_getting_started();
    
    Ok(())
}

/// Initialization modes
#[derive(Debug)]
enum InitMode {
    ConfigOnly,
    NewProject,
    AddTemplates,
    Reinitialize,
}

/// Setup Minifly configuration
async fn setup_configuration(config: &Config) -> Result<Config> {
    println!("\n{}", "⚙️  Configuration Setup".bold());
    
    let api_url: String = Input::new()
        .with_prompt("API URL")
        .default(config.api_url.clone())
        .interact_text()?;
    
    let token: String = Password::new()
        .with_prompt("API Token (optional)")
        .allow_empty_password(true)
        .interact()?;
    
    let new_config = Config {
        api_url,
        token: if token.is_empty() { None } else { Some(token) },
    };
    
    new_config.save().context("Failed to save configuration")?;
    
    Ok(new_config)
}

/// Setup a new project with templates
async fn setup_new_project(_config: &Config) -> Result<()> {
    println!("\n{}", "📋 Project Template Selection".bold());
    
    let templates = get_available_templates();
    let template_names: Vec<&str> = templates.iter().map(|t| t.name.as_str()).collect();
    
    // Show template options
    for (i, template) in templates.iter().enumerate() {
        println!("  {}. {} ({})", 
            (i + 1).to_string().cyan(),
            template.name.bold(),
            template.language.green()
        );
        println!("     {}", template.description.dimmed());
        if !template.features.is_empty() {
            println!("     Features: {}", template.features.join(", ").blue());
        }
        println!();
    }
    
    let selection = Select::new()
        .with_prompt("Choose a project template")
        .items(&template_names)
        .default(0)
        .interact()?;
    
    let selected_template = &templates[selection];
    
    // Get project details
    let app_name: String = Input::new()
        .with_prompt("Application name")
        .default("my-minifly-app".to_string())
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                Ok(())
            } else {
                Err("App name can only contain letters, numbers, hyphens, and underscores")
            }
        })
        .interact_text()?;
    
    let description: String = Input::new()
        .with_prompt("Description")
        .default(format!("A {} application built with Minifly", selected_template.framework))
        .interact_text()?;
    
    // Generate project files
    println!("\n{} Creating project files...", "📝".blue());
    create_project_files(selected_template, &app_name, &description).await?;
    
    // Create data directories
    fs::create_dir_all("data/litefs").context("Failed to create data directories")?;
    fs::create_dir_all("data/apps").context("Failed to create data directories")?;
    
    println!("{} Project created successfully!", "✅".green());
    
    Ok(())
}

/// Add project templates to existing project
async fn add_project_templates() -> Result<()> {
    println!("\n{}", "📂 Adding Minifly Templates".bold());
    
    let templates_to_add = vec![
        "Docker Compose configuration",
        "LiteFS configuration", 
        "Multi-tenant example",
        "GitHub Actions workflow",
        "Development scripts",
    ];
    
    for template in templates_to_add {
        if Confirm::new()
            .with_prompt(&format!("Add {}?", template))
            .default(true)
            .interact()? 
        {
            match template {
                "Docker Compose configuration" => create_docker_compose().await?,
                "LiteFS configuration" => create_litefs_config().await?,
                "Multi-tenant example" => create_multitenant_example().await?,
                "GitHub Actions workflow" => create_github_workflow().await?,
                "Development scripts" => create_dev_scripts().await?,
                _ => {}
            }
        }
    }
    
    println!("{} Templates added!", "✅".green());
    Ok(())
}

/// Get available project templates
fn get_available_templates() -> Vec<ProjectTemplate> {
    vec![
        ProjectTemplate {
            name: "Rust + Axum + LiteFS".to_string(),
            description: "Multi-tenant web app with Axum, Askama templates, and LiteFS".to_string(),
            language: "Rust".to_string(),
            framework: "Axum".to_string(),
            features: vec![
                "Multi-tenant architecture".to_string(),
                "LiteFS database per tenant".to_string(),
                "Askama templating".to_string(),
                "Docker ready".to_string(),
            ],
            files: HashMap::new(),
        },
        ProjectTemplate {
            name: "Node.js + Express + SQLite".to_string(),
            description: "Express.js web server with SQLite database".to_string(),
            language: "JavaScript".to_string(),
            framework: "Express".to_string(),
            features: vec![
                "Express.js server".to_string(),
                "SQLite database".to_string(),
                "Docker ready".to_string(),
                "ESM modules".to_string(),
            ],
            files: HashMap::new(),
        },
        ProjectTemplate {
            name: "Python + FastAPI + SQLite".to_string(),
            description: "FastAPI application with async SQLite support".to_string(),
            language: "Python".to_string(),
            framework: "FastAPI".to_string(),
            features: vec![
                "FastAPI with async".to_string(),
                "SQLite with aiosqlite".to_string(),
                "Pydantic models".to_string(),
                "Docker ready".to_string(),
            ],
            files: HashMap::new(),
        },
        ProjectTemplate {
            name: "Go + Gin + SQLite".to_string(),
            description: "Gin web framework with SQLite database".to_string(),
            language: "Go".to_string(),
            framework: "Gin".to_string(),
            features: vec![
                "Gin web framework".to_string(),
                "SQLite database".to_string(),
                "Structured logging".to_string(),
                "Docker ready".to_string(),
            ],
            files: HashMap::new(),
        },
        ProjectTemplate {
            name: "Minimal Docker".to_string(),
            description: "Basic Docker setup for any language".to_string(),
            language: "Any".to_string(),
            framework: "Docker".to_string(),
            features: vec![
                "Dockerfile".to_string(),
                "fly.toml".to_string(),
                "Basic setup".to_string(),
            ],
            files: HashMap::new(),
        },
    ]
}

/// Create project files based on template
async fn create_project_files(template: &ProjectTemplate, app_name: &str, description: &str) -> Result<()> {
    match template.name.as_str() {
        "Rust + Axum + LiteFS" => create_rust_axum_project(app_name, description).await,
        "Node.js + Express + SQLite" => create_node_express_project(app_name, description).await,
        "Python + FastAPI + SQLite" => create_python_fastapi_project(app_name, description).await,
        "Go + Gin + SQLite" => create_go_gin_project(app_name, description).await,
        "Minimal Docker" => create_minimal_docker_project(app_name, description).await,
        _ => Ok(()),
    }
}

/// Create Rust + Axum project
async fn create_rust_axum_project(app_name: &str, description: &str) -> Result<()> {
    // Cargo.toml
    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
description = "{}"

[dependencies]
axum = "0.7"
tokio = {{ version = "1.40", features = ["full"] }}
tower = "0.5"
tower-http = {{ version = "0.6", features = ["fs", "trace"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
askama = "0.12"
askama_axum = "0.4"
sqlx = {{ version = "0.8", features = ["runtime-tokio-rustls", "sqlite"] }}
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
uuid = {{ version = "1.10", features = ["v4", "serde"] }}
"#, app_name, description);
    
    fs::write("Cargo.toml", cargo_toml).context("Failed to write Cargo.toml")?;
    
    // src/main.rs
    let main_rs = r#"use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use askama::Template;
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row};
use std::sync::Arc;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{info, warn};

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    tenant: String,
    users: Vec<User>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct User {
    id: i64,
    name: String,
    email: String,
    tenant_id: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .init();
    
    // Connect to SQLite database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./data/app.db".to_string());
    
    let db = SqlitePool::connect(&database_url).await?;
    
    // Run migrations
    sqlx::migrate!("./migrations").run(&db).await?;
    
    let state = AppState { db };
    
    let app = Router::new()
        .route("/", get(index))
        .route("/tenant/:tenant_id", get(tenant_page))
        .route("/tenant/:tenant_id/users", post(create_user))
        .route("/health", get(health_check))
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);
    
    info!("Starting server on port {}", port);
    
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn index() -> Html<&'static str> {
    Html("<html>
        <head><title>Minifly Multi-Tenant App</title></head>
        <body>
            <h1>Welcome to Minifly!</h1>
            <p>This is a multi-tenant application example.</p>
            <p>Try visiting: <a href=\"/tenant/acme\">/tenant/acme</a></p>
        </body>
    </html>")
}

async fn tenant_page(
    Path(tenant_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let users = sqlx::query_as::<_, User>(
        "SELECT id, name, email, tenant_id FROM users WHERE tenant_id = ?"
    )
    .bind(&tenant_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let template = IndexTemplate {
        tenant: tenant_id,
        users,
    };
    
    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

async fn create_user(
    Path(tenant_id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<User>, StatusCode> {
    let user_id = sqlx::query(
        "INSERT INTO users (name, email, tenant_id) VALUES (?, ?, ?) RETURNING id"
    )
    .bind(&req.name)
    .bind(&req.email)
    .bind(&tenant_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .get::<i64, _>("id");
    
    let user = User {
        id: user_id,
        name: req.name,
        email: req.email,
        tenant_id,
    };
    
    Ok(Json(user))
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "minifly-example",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}
"#;
    
    fs::create_dir_all("src").context("Failed to create src directory")?;
    fs::write("src/main.rs", main_rs).context("Failed to write main.rs")?;
    
    // Create templates directory and index template
    fs::create_dir_all("templates").context("Failed to create templates directory")?;
    let index_template = r#"<!DOCTYPE html>
<html>
<head>
    <title>{{ tenant }} - Minifly App</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }
        .tenant-badge { background: #0066ff; color: white; padding: 4px 8px; border-radius: 4px; }
        .user-card { border: 1px solid #ddd; padding: 10px; margin: 10px 0; border-radius: 4px; }
    </style>
</head>
<body>
    <h1>Welcome to <span class="tenant-badge">{{ tenant }}</span></h1>
    <p>This is a multi-tenant application powered by Minifly!</p>
    
    <h2>Users ({{ users.len() }})</h2>
    {% if users.is_empty() %}
        <p>No users yet. Create one using the API!</p>
        <pre>curl -X POST http://localhost:8080/tenant/{{ tenant }}/users \
  -H "Content-Type: application/json" \
  -d '{"name": "John Doe", "email": "john@example.com"}'</pre>
    {% else %}
        {% for user in users %}
        <div class="user-card">
            <strong>{{ user.name }}</strong> - {{ user.email }}
            <small>(ID: {{ user.id }})</small>
        </div>
        {% endfor %}
    {% endif %}
    
    <hr>
    <p><a href="/">← Back to Home</a></p>
</body>
</html>"#;
    
    fs::write("templates/index.html", index_template).context("Failed to write template")?;
    
    // Create migrations
    fs::create_dir_all("migrations").context("Failed to create migrations directory")?;
    let migration = r#"CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    tenant_id TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_tenant_id ON users(tenant_id);"#;
    
    fs::write("migrations/001_initial.sql", migration).context("Failed to write migration")?;
    
    // Create common project files
    create_common_files(app_name, description).await?;
    
    println!("   ✓ Created Rust project with Axum + LiteFS");
    Ok(())
}

/// Create Node.js + Express project
async fn create_node_express_project(app_name: &str, description: &str) -> Result<()> {
    let package_json = format!(r#"{{
  "name": "{}",
  "version": "0.1.0",
  "description": "{}",
  "type": "module",
  "main": "src/server.js",
  "scripts": {{
    "start": "node src/server.js",
    "dev": "node --watch src/server.js",
    "test": "echo \"Error: no test specified\" && exit 1"
  }},
  "dependencies": {{
    "express": "^4.19.0",
    "sqlite3": "^5.1.0",
    "helmet": "^7.1.0",
    "cors": "^2.8.5"
  }},
  "engines": {{
    "node": ">=18"
  }}
}}"#, app_name, description);
    
    fs::write("package.json", package_json).context("Failed to write package.json")?;
    
    let server_js = r#"import express from 'express';
import sqlite3 from 'sqlite3';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import helmet from 'helmet';
import cors from 'cors';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const app = express();
const port = process.env.PORT || 8080;

// Middleware
app.use(helmet());
app.use(cors());
app.use(express.json());
app.use(express.static(join(__dirname, '../static')));

// Database setup
const db = new sqlite3.Database('./data/app.db');

// Initialize database
db.serialize(() => {
    db.run(`CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        email TEXT NOT NULL,
        tenant_id TEXT NOT NULL,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    )`);
    
    db.run(`CREATE INDEX IF NOT EXISTS idx_users_tenant_id ON users(tenant_id)`);
});

// Routes
app.get('/', (req, res) => {
    res.send(`
        <html>
            <head><title>Minifly Node.js App</title></head>
            <body>
                <h1>Welcome to Minifly!</h1>
                <p>This is a Node.js + Express application.</p>
                <p>Try visiting: <a href="/tenant/acme">/tenant/acme</a></p>
            </body>
        </html>
    `);
});

app.get('/tenant/:tenantId', (req, res) => {
    const { tenantId } = req.params;
    
    db.all(
        'SELECT * FROM users WHERE tenant_id = ? ORDER BY created_at DESC',
        [tenantId],
        (err, users) => {
            if (err) {
                return res.status(500).json({ error: 'Database error' });
            }
            
            const userList = users.map(user => 
                `<div style="border: 1px solid #ddd; padding: 10px; margin: 10px 0;">
                    <strong>${user.name}</strong> - ${user.email}
                    <small>(ID: ${user.id})</small>
                </div>`
            ).join('');
            
            res.send(`
                <html>
                    <head><title>${tenantId} - Minifly App</title></head>
                    <body>
                        <h1>Welcome to <span style="background: #0066ff; color: white; padding: 4px 8px; border-radius: 4px;">${tenantId}</span></h1>
                        <p>This is a multi-tenant Node.js application!</p>
                        <h2>Users (${users.length})</h2>
                        ${userList || '<p>No users yet.</p>'}
                        <hr>
                        <p><a href="/">← Back to Home</a></p>
                    </body>
                </html>
            `);
        }
    );
});

app.post('/tenant/:tenantId/users', (req, res) => {
    const { tenantId } = req.params;
    const { name, email } = req.body;
    
    if (!name || !email) {
        return res.status(400).json({ error: 'Name and email are required' });
    }
    
    db.run(
        'INSERT INTO users (name, email, tenant_id) VALUES (?, ?, ?)',
        [name, email, tenantId],
        function(err) {
            if (err) {
                return res.status(500).json({ error: 'Database error' });
            }
            
            res.json({
                id: this.lastID,
                name,
                email,
                tenant_id: tenantId
            });
        }
    );
});

app.get('/health', (req, res) => {
    res.json({
        status: 'healthy',
        service: 'minifly-node-example',
        timestamp: new Date().toISOString()
    });
});

app.listen(port, '0.0.0.0', () => {
    console.log(`🚀 Server running on port ${port}`);
    console.log(`📍 Region: ${process.env.FLY_REGION || 'local'}`);
});
"#;
    
    fs::create_dir_all("src").context("Failed to create src directory")?;
    fs::write("src/server.js", server_js).context("Failed to write server.js")?;
    
    create_common_files(app_name, description).await?;
    
    println!("   ✓ Created Node.js project with Express + SQLite");
    Ok(())
}

/// Create Python + FastAPI project
async fn create_python_fastapi_project(app_name: &str, description: &str) -> Result<()> {
    let requirements_txt = r#"fastapi==0.104.1
uvicorn[standard]==0.24.0
aiosqlite==0.19.0
pydantic==2.5.0
"#;
    
    fs::write("requirements.txt", requirements_txt).context("Failed to write requirements.txt")?;
    
    let main_py = format!(r#"from fastapi import FastAPI, HTTPException, Path
from fastapi.responses import HTMLResponse
from pydantic import BaseModel
import aiosqlite
import os
from typing import List
import asyncio

app = FastAPI(
    title="{}",
    description="{}",
    version="0.1.0"
)

DATABASE_URL = os.getenv("DATABASE_URL", "./data/app.db")

class User(BaseModel):
    id: int
    name: str
    email: str
    tenant_id: str

class CreateUserRequest(BaseModel):
    name: str
    email: str

async def init_db():
    async with aiosqlite.connect(DATABASE_URL) as db:
        await db.execute("""
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                email TEXT NOT NULL,
                tenant_id TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
        """)
        await db.execute("CREATE INDEX IF NOT EXISTS idx_users_tenant_id ON users(tenant_id)")
        await db.commit()

@app.on_event("startup")
async def startup_event():
    await init_db()

@app.get("/", response_class=HTMLResponse)
async def home():
    return """
    <html>
        <head><title>Minifly FastAPI App</title></head>
        <body>
            <h1>Welcome to Minifly!</h1>
            <p>This is a Python + FastAPI application.</p>
            <p>Try visiting: <a href="/tenant/acme">/tenant/acme</a></p>
            <p>API docs: <a href="/docs">/docs</a></p>
        </body>
    </html>
    """

@app.get("/tenant/{{tenant_id}}", response_class=HTMLResponse)
async def tenant_page(tenant_id: str = Path(...)):
    async with aiosqlite.connect(DATABASE_URL) as db:
        async with db.execute(
            "SELECT id, name, email, tenant_id FROM users WHERE tenant_id = ? ORDER BY id DESC",
            (tenant_id,)
        ) as cursor:
            rows = await cursor.fetchall()
            users = [
                {{"id": row[0], "name": row[1], "email": row[2], "tenant_id": row[3]}}
                for row in rows
            ]
    
    user_list = "".join([
        f'<div style="border: 1px solid #ddd; padding: 10px; margin: 10px 0;">'
        f'<strong>{{user["name"]}}</strong> - {{user["email"]}}'
        f'<small>(ID: {{user["id"]}})</small></div>'
        for user in users
    ])
    
    return f"""
    <html>
        <head><title>{{tenant_id}} - Minifly App</title></head>
        <body>
            <h1>Welcome to <span style="background: #0066ff; color: white; padding: 4px 8px; border-radius: 4px;">{{tenant_id}}</span></h1>
            <p>This is a multi-tenant FastAPI application!</p>
            <h2>Users ({{len(users)}})</h2>
            {{user_list or "<p>No users yet.</p>"}}
            <hr>
            <p><a href="/">← Back to Home</a> | <a href="/docs">API Docs</a></p>
        </body>
    </html>
    """

@app.post("/tenant/{{tenant_id}}/users", response_model=User)
async def create_user(
    request: CreateUserRequest,
    tenant_id: str = Path(...)
):
    async with aiosqlite.connect(DATABASE_URL) as db:
        cursor = await db.execute(
            "INSERT INTO users (name, email, tenant_id) VALUES (?, ?, ?)",
            (request.name, request.email, tenant_id)
        )
        await db.commit()
        user_id = cursor.lastrowid
    
    return User(
        id=user_id,
        name=request.name,
        email=request.email,
        tenant_id=tenant_id
    )

@app.get("/tenant/{{tenant_id}}/users", response_model=List[User])
async def get_users(tenant_id: str = Path(...)):
    async with aiosqlite.connect(DATABASE_URL) as db:
        async with db.execute(
            "SELECT id, name, email, tenant_id FROM users WHERE tenant_id = ?",
            (tenant_id,)
        ) as cursor:
            rows = await cursor.fetchall()
            return [
                User(id=row[0], name=row[1], email=row[2], tenant_id=row[3])
                for row in rows
            ]

@app.get("/health")
async def health_check():
    return {{
        "status": "healthy",
        "service": "minifly-fastapi-example",
        "timestamp": "2024-06-22T10:30:00Z"  # In real app, use datetime.utcnow()
    }}

if __name__ == "__main__":
    import uvicorn
    port = int(os.getenv("PORT", 8080))
    uvicorn.run(app, host="0.0.0.0", port=port)
"#, app_name, description);
    
    fs::write("main.py", main_py).context("Failed to write main.py")?;
    
    create_common_files(app_name, description).await?;
    
    println!("   ✓ Created Python project with FastAPI + SQLite");
    Ok(())
}

/// Create Go + Gin project
async fn create_go_gin_project(app_name: &str, description: &str) -> Result<()> {
    let go_mod = format!("module {}\n\ngo 1.21\n\nrequire (\n    github.com/gin-gonic/gin v1.9.1\n    github.com/mattn/go-sqlite3 v1.14.18\n)\n", app_name);
    fs::write("go.mod", go_mod).context("Failed to write go.mod")?;
    
    let main_go = r#"package main

import (
    "database/sql"
    "fmt"
    "log"
    "net/http"
    "os"
    "strconv"
    "time"

    "github.com/gin-gonic/gin"
    _ "github.com/mattn/go-sqlite3"
)

type User struct {{
    ID       int    `json:"id" db:"id"`
    Name     string `json:"name" db:"name"`
    Email    string `json:"email" db:"email"`
    TenantID string `json:"tenant_id" db:"tenant_id"`
}}

type CreateUserRequest struct {{
    Name  string `json:"name" binding:"required"`
    Email string `json:"email" binding:"required"`
}}

var db *sql.DB

func initDB() {{
    var err error
    dbPath := os.Getenv("DATABASE_URL")
    if dbPath == "" {{
        dbPath = "./data/app.db"
    }}
    
    db, err = sql.Open("sqlite3", dbPath)
    if err != nil {{
        log.Fatal("Failed to connect to database:", err)
    }}
    
    // Create table
    _, err = db.Exec(`
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            tenant_id TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
    `)
    if err != nil {{
        log.Fatal("Failed to create table:", err)
    }}
    
    // Create index
    _, err = db.Exec("CREATE INDEX IF NOT EXISTS idx_users_tenant_id ON users(tenant_id)")
    if err != nil {{
        log.Fatal("Failed to create index:", err)
    }}
}}

func main() {{
    initDB()
    defer db.Close()
    
    r := gin.Default()
    
    r.GET("/", func(c *gin.Context) {{
        c.Header("Content-Type", "text/html")
        c.String(http.StatusOK, `
            <html>
                <head><title>Minifly Go App</title></head>
                <body>
                    <h1>Welcome to Minifly!</h1>
                    <p>This is a Go + Gin application.</p>
                    <p>Try visiting: <a href="/tenant/acme">/tenant/acme</a></p>
                </body>
            </html>
        `)
    }})
    
    r.GET("/tenant/:tenantId", func(c *gin.Context) {{
        tenantID := c.Param("tenantId")
        
        rows, err := db.Query("SELECT id, name, email, tenant_id FROM users WHERE tenant_id = ? ORDER BY id DESC", tenantID)
        if err != nil {{
            c.JSON(http.StatusInternalServerError, gin.H{{"error": "Database error"}})
            return
        }}
        defer rows.Close()
        
        var users []User
        for rows.Next() {{
            var user User
            err := rows.Scan(&user.ID, &user.Name, &user.Email, &user.TenantID)
            if err != nil {{
                continue
            }}
            users = append(users, user)
        }}
        
        userList := ""
        for _, user := range users {{
            userList += fmt.Sprintf(`
                <div style="border: 1px solid #ddd; padding: 10px; margin: 10px 0;">
                    <strong>%%s</strong> - %%s
                    <small>(ID: %%d)</small>
                </div>
            `, user.Name, user.Email, user.ID)
        }}
        
        if userList == "" {{
            userList = "<p>No users yet.</p>"
        }}
        
        html := fmt.Sprintf(`
            <html>
                <head><title>%%s - Minifly App</title></head>
                <body>
                    <h1>Welcome to <span style="background: #0066ff; color: white; padding: 4px 8px; border-radius: 4px;">%%s</span></h1>
                    <p>This is a multi-tenant Go application!</p>
                    <h2>Users (%%d)</h2>
                    %%s
                    <hr>
                    <p><a href="/">← Back to Home</a></p>
                </body>
            </html>
        `, tenantID, tenantID, len(users), userList)
        
        c.Header("Content-Type", "text/html")
        c.String(http.StatusOK, html)
    }})
    
    r.POST("/tenant/:tenantId/users", func(c *gin.Context) {{
        tenantID := c.Param("tenantId")
        
        var req CreateUserRequest
        if err := c.ShouldBindJSON(&req); err != nil {{
            c.JSON(http.StatusBadRequest, gin.H{{"error": err.Error()}})
            return
        }}
        
        result, err := db.Exec("INSERT INTO users (name, email, tenant_id) VALUES (?, ?, ?)",
            req.Name, req.Email, tenantID)
        if err != nil {{
            c.JSON(http.StatusInternalServerError, gin.H{{"error": "Database error"}})
            return
        }}
        
        userID, _ := result.LastInsertId()
        
        user := User{{
            ID:       int(userID),
            Name:     req.Name,
            Email:    req.Email,
            TenantID: tenantID,
        }}
        
        c.JSON(http.StatusCreated, user)
    }})
    
    r.GET("/health", func(c *gin.Context) {{
        c.JSON(http.StatusOK, gin.H{{
            "status":    "healthy",
            "service":   "minifly-go-example",
            "timestamp": time.Now().Format(time.RFC3339),
        }})
    }})
    
    port := os.Getenv("PORT")
    if port == "" {{
        port = "8080"
    }}
    
    log.Printf("🚀 Server starting on port %%s", port)
    log.Printf("📍 Region: %%s", getEnv("FLY_REGION", "local"))
    
    r.Run(":" + port)
}}

func getEnv(key, defaultValue string) string {{
    if value := os.Getenv(key); value != "" {{
        return value
    }}
    return defaultValue
}}
"#;
    
    fs::write("main.go", main_go).context("Failed to write main.go")?;
    
    create_common_files(app_name, description).await?;
    
    println!("   ✓ Created Go project with Gin + SQLite");
    Ok(())
}

/// Create minimal Docker project
async fn create_minimal_docker_project(app_name: &str, description: &str) -> Result<()> {
    let dockerfile = r#"# Choose your base image
FROM alpine:latest

# Install any required packages
RUN apk add --no-cache ca-certificates

# Create app directory
WORKDIR /app

# Copy your application files
# COPY . .

# Expose port
EXPOSE 8080

# Add a simple health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

# Start your application
CMD ["echo", "Configure this Dockerfile for your application"]
"#;
    
    fs::write("Dockerfile", dockerfile).context("Failed to write Dockerfile")?;
    
    create_common_files(app_name, description).await?;
    
    println!("   ✓ Created minimal Docker project");
    Ok(())
}

/// Create common project files (fly.toml, litefs.yml, docker-compose.yml, etc.)
async fn create_common_files(app_name: &str, description: &str) -> Result<()> {
    // fly.toml
    let fly_toml = format!(r#"app = "{}"
primary_region = "sjc"

[build]

[env]
  DATABASE_URL = "sqlite:///data/app.db"

[http_service]
  internal_port = 8080
  force_https = true
  auto_stop_machines = true
  auto_start_machines = true
  min_machines_running = 0
  processes = ["app"]

[[vm]]
  cpu_kind = "shared"
  cpus = 1
  memory_mb = 256

[[statics]]
  guest_path = "/app/static"
  url_prefix = "/static/"
"#, app_name);
    
    fs::write("fly.toml", fly_toml).context("Failed to write fly.toml")?;
    
    // LiteFS configuration
    create_litefs_config().await?;
    
    // Docker Compose for local development
    create_docker_compose().await?;
    
    // README.md
    let readme = format!(r#"# {}

{}

## Quick Start with Minifly

1. **Start Minifly platform:**
   ```bash
   minifly serve
   ```

2. **Create and deploy your app:**
   ```bash
   minifly apps create {}
   minifly deploy
   ```

3. **View logs:**
   ```bash
   minifly logs <machine-id> --follow
   ```

4. **Check status:**
   ```bash
   minifly status
   ```

## Local Development

### With Docker Compose
```bash
docker-compose up
```

### Direct execution
```bash
# Install dependencies first, then:
# For Rust: cargo run
# For Node.js: npm start
# For Python: python main.py
# For Go: go run main.go
```

## Multi-Tenant Architecture

This application demonstrates Fly.io's multi-tenant patterns:

- **Tenant isolation:** Each tenant gets isolated data
- **Regional deployment:** Deploy close to users
- **Auto-scaling:** Machines start/stop based on demand
- **Database per tenant:** SQLite with LiteFS replication

### Testing Different Tenants

Visit different tenant pages:
- http://localhost:8080/tenant/acme
- http://localhost:8080/tenant/widgets-inc
- http://localhost:8080/tenant/startup-xyz

### API Usage

Create users for a tenant:
```bash
curl -X POST http://localhost:8080/tenant/acme/users \
  -H "Content-Type: application/json" \
  -d '{{"name": "John Doe", "email": "john@acme.com"}}'
```

## Minifly Features Demonstrated

- ✅ Multi-tenant application architecture
- ✅ SQLite with LiteFS for distributed data
- ✅ Docker containerization
- ✅ Health checks and monitoring
- ✅ Region-aware deployment
- ✅ Auto-scaling configuration

## Learn More

- [Minifly Documentation](https://docs.minifly.dev)
- [Fly.io Documentation](https://fly.io/docs)
- [LiteFS Documentation](https://fly.io/docs/litefs)
"#, app_name, description, app_name);
    
    fs::write("README.md", readme).context("Failed to write README.md")?;
    
    // .gitignore
    let gitignore = r#"# Dependencies
node_modules/
target/
__pycache__/
*.pyc
go.sum

# Database files
*.db
*.db-*
data/

# OS files
.DS_Store
Thumbs.db

# IDE files
.vscode/
.idea/
*.swp
*.swo

# Build artifacts
dist/
build/

# Environment variables
.env
.env.local

# Logs
*.log
logs/

# Runtime
*.pid
*.pid.lock
"#;
    
    fs::write(".gitignore", gitignore).context("Failed to write .gitignore")?;
    
    Ok(())
}

/// Create LiteFS configuration
async fn create_litefs_config() -> Result<()> {
    let litefs_yml = r#"# LiteFS configuration for distributed SQLite
fuse:
  dir: "/litefs"

data:
  dir: "/data/litefs"

proxy:
  addr: ":20202"
  target: "localhost:8080"
  db: "app.db"

lease:
  type: "consul"
  candidate: true
  promote: true
  
  consul:
    url: "http://localhost:8500"
    key: "minifly/leader"

exec:
  - cmd: "/app/server"
    if-candidate: true

log:
  level: "INFO"
"#;
    
    fs::write("litefs.yml", litefs_yml).context("Failed to write litefs.yml")?;
    Ok(())
}

/// Create Docker Compose configuration
async fn create_docker_compose() -> Result<()> {
    let docker_compose = r#"version: '3.8'

services:
  app:
    build: .
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=sqlite:///data/app.db
      - FLY_REGION=local
      - PORT=8080
    volumes:
      - ./data:/data
    depends_on:
      - consul
    restart: unless-stopped

  consul:
    image: consul:1.15
    ports:
      - "8500:8500"
    command: >
      consul agent -dev -ui -client=0.0.0.0
      -log-level=INFO
    environment:
      - CONSUL_BIND_INTERFACE=eth0
    restart: unless-stopped

volumes:
  data:
    driver: local
"#;
    
    fs::write("docker-compose.yml", docker_compose).context("Failed to write docker-compose.yml")?;
    Ok(())
}

/// Create multi-tenant example
async fn create_multitenant_example() -> Result<()> {
    fs::create_dir_all("examples").context("Failed to create examples directory")?;
    
    let example_script = r#"#!/bin/bash
# Multi-tenant example script
set -e

echo "🚀 Setting up multi-tenant example data..."

# Wait for app to be ready
echo "Waiting for app to be ready..."
until curl -f http://localhost:8080/health > /dev/null 2>&1; do
    sleep 1
done

# Create users for different tenants
echo "Creating example users..."

# ACME Corp users
curl -X POST http://localhost:8080/tenant/acme/users \
  -H "Content-Type: application/json" \
  -d '{"name": "Alice Smith", "email": "alice@acme.com"}' \
  -w "\n"

curl -X POST http://localhost:8080/tenant/acme/users \
  -H "Content-Type: application/json" \
  -d '{"name": "Bob Johnson", "email": "bob@acme.com"}' \
  -w "\n"

# Widgets Inc users  
curl -X POST http://localhost:8080/tenant/widgets-inc/users \
  -H "Content-Type: application/json" \
  -d '{"name": "Carol Davis", "email": "carol@widgets.com"}' \
  -w "\n"

# Startup XYZ users
curl -X POST http://localhost:8080/tenant/startup-xyz/users \
  -H "Content-Type: application/json" \
  -d '{"name": "David Wilson", "email": "david@startup.xyz"}' \
  -w "\n"

echo ""
echo "✅ Example data created!"
echo ""
echo "Visit these tenant pages:"
echo "  - http://localhost:8080/tenant/acme"
echo "  - http://localhost:8080/tenant/widgets-inc" 
echo "  - http://localhost:8080/tenant/startup-xyz"
"#;
    
    fs::write("examples/setup-tenants.sh", example_script).context("Failed to write example script")?;
    
    // Make script executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata("examples/setup-tenants.sh")
            .context("Failed to get script metadata")?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions("examples/setup-tenants.sh", perms)
            .context("Failed to set script permissions")?;
    }
    
    Ok(())
}

/// Create GitHub Actions workflow
async fn create_github_workflow() -> Result<()> {
    fs::create_dir_all(".github/workflows").context("Failed to create workflows directory")?;
    
    let workflow = r#"name: Deploy to Fly.io

on:
  push:
    branches: [main]

jobs:
  deploy:
    name: Deploy app
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - uses: superfly/flyctl-actions/setup-flyctl@master
      
      - run: flyctl deploy --remote-only
        env:
          FLY_API_TOKEN: ${{ secrets.FLY_API_TOKEN }}
"#;
    
    fs::write(".github/workflows/fly.yml", workflow).context("Failed to write GitHub workflow")?;
    Ok(())
}

/// Create development scripts
async fn create_dev_scripts() -> Result<()> {
    fs::create_dir_all("scripts").context("Failed to create scripts directory")?;
    
    let dev_script = r#"#!/bin/bash
# Development environment setup script
set -e

echo "🛠️  Setting up development environment..."

# Create data directories
mkdir -p data/litefs data/apps

# Check for required tools
check_tool() {
    if ! command -v $1 &> /dev/null; then
        echo "❌ $1 is not installed"
        return 1
    else
        echo "✅ $1 is available"
        return 0
    fi
}

echo "Checking required tools..."
check_tool docker
check_tool minifly

# Start Minifly platform
echo "Starting Minifly platform..."
minifly serve --dev &
MINIFLY_PID=$!

# Wait for platform to be ready
echo "Waiting for Minifly platform..."
sleep 5

# Create app
echo "Creating application..."
minifly apps create $(basename $(pwd)) || true

echo ""
echo "🎉 Development environment ready!"
echo ""
echo "Next steps:"
echo "  1. Deploy your app: minifly deploy"
echo "  2. View logs: minifly logs <machine-id> --follow"
echo "  3. Check status: minifly status"
echo ""
echo "To stop: kill $MINIFLY_PID"
"#;
    
    fs::write("scripts/dev-setup.sh", dev_script).context("Failed to write dev script")?;
    
    // Make script executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata("scripts/dev-setup.sh")
            .context("Failed to get script metadata")?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions("scripts/dev-setup.sh", perms)
            .context("Failed to set script permissions")?;
    }
    
    Ok(())
}

/// Show getting started information
fn show_getting_started() {
    println!("\n{}", "🎉 Project initialized successfully!".green().bold());
    println!("\n{}", "🚀 Getting Started:".bold());
    println!("  1. {} Start the Minifly platform", "minifly serve".yellow());
    println!("  2. {} Deploy your application", "minifly deploy".yellow());
    println!("  3. {} Stream logs in real-time", "minifly logs <machine-id> --follow".yellow());
    println!("  4. {} Check platform status", "minifly status".yellow());
    
    println!("\n{}", "📚 Useful Commands:".bold());
    println!("  {} - Create a new app", "minifly apps create <name>".cyan());
    println!("  {} - List all machines", "minifly machines list --app <name>".cyan());
    println!("  {} - Proxy to a service", "minifly proxy <machine-id>".cyan());
    println!("  {} - Stop the platform", "minifly stop".cyan());
    
    println!("\n{}", "📖 Learn More:".bold());
    println!("  • Documentation: {}", "https://docs.minifly.dev".blue());
    println!("  • Examples: {}", "./examples/".blue());
    println!("  • Development: {}", "./scripts/dev-setup.sh".blue());
    
    println!("\n{}", "✨ Happy coding with Minifly!".green());
}