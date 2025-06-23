---
sidebar_position: 3
---

# Todo Auth App Example

A comprehensive example demonstrating Minifly's multi-tenant capabilities with authentication, per-user apps/databases, region selection, and image storage.

## Overview

This example showcases:
- 🔐 Secure email/password authentication with sessions
- 🌍 Multi-region deployment with user choice
- 👤 Per-user app and database isolation
- 📝 Full-featured todo management
- 🖼️ Image upload and storage
- 🎨 Modern, responsive web interface

## Architecture

### Multi-Tenant Design

Each user gets:
- A dedicated app instance: `todo-user-{userId}`
- An isolated SQLite database
- Secure volume mounts for persistent storage
- Deployment in their chosen region

### Technology Stack

- **Backend**: Rust with Axum web framework
- **Database**: SQLite with SQLx and migrations
- **Auth**: Argon2 password hashing + tower-sessions
- **Templates**: Askama for server-side rendering
- **Frontend**: Vanilla CSS/JS with responsive design

## Key Features

### 1. Authentication System

```rust
// Secure password hashing
let password_hash = hash_password(&form.password)?;

// Session management
set_session_user(&session, &user).await?;

// Protected routes with extractors
async fn dashboard(
    AuthUser(user): AuthUser,
    State(db): State<Pool<Sqlite>>,
) -> Result<Html<String>>
```

### 2. Region Selection

Users choose from 9 global regions at signup:

```rust
pub const AVAILABLE_REGIONS: &[(&str, &str)] = &[
    ("iad", "Ashburn, Virginia (US)"),
    ("ord", "Chicago, Illinois (US)"),
    ("lhr", "London, United Kingdom"),
    ("fra", "Frankfurt, Germany"),
    ("syd", "Sydney, Australia"),
    // ... more regions
];
```

### 3. Tenant Provisioning

Automatic app creation for each user:

```rust
// Generate unique app name
let app_name = format!("todo-user-{}", &user_id[..8]);

// Create app via Minifly API
let create_app_req = CreateAppRequest {
    app_name: app_name.clone(),
    org_slug: "personal".to_string(),
};

// Deploy machine in user's region
let machine_config = MachineConfig {
    image: "ghcr.io/livebud/bud/sqlitedb:latest",
    region: Some(user_selected_region),
    // ... configuration
};
```

### 4. Image Storage

Base64-encoded images stored with todos:

```rust
// Handle multipart upload
while let Some(field) = multipart.next_field().await? {
    if field.name() == Some("image") {
        let data = field.bytes().await?;
        
        // Validate size (5MB limit)
        if data.len() > 5 * 1024 * 1024 {
            return Err(AppError::Validation("Image too large"));
        }
        
        // Encode and store
        let encoded = base64::encode(&data);
        // ... save to database
    }
}
```

## Deployment

### Prerequisites

1. Minifly platform running:
   ```bash
   minifly serve --dev
   ```

2. Navigate to example:
   ```bash
   cd examples/todo-auth-app
   ```

### Deploy Options

**Option 1: Quick deploy with script**
```bash
./run.sh
```

**Option 2: Manual deploy**
```bash
minifly deploy
```

**Option 3: Development mode**
```bash
minifly serve --dev  # From example directory
```

## Configuration

### Environment Variables

Create `.env` file:

```env
# Database
DATABASE_URL=sqlite:///litefs/app.db

# Session secret (generate with: openssl rand -hex 32)
SESSION_SECRET=your-secret-key-here

# Minifly API
MINIFLY_API_URL=http://localhost:4280

# Server
PORT=8080
RUST_LOG=info,todo_auth_app=debug
```

### Database Schema

The app uses three main tables:

```sql
-- Users table
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- User apps (tenant deployments)
CREATE TABLE user_apps (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    app_name TEXT NOT NULL,
    region TEXT NOT NULL,
    machine_id TEXT,
    status TEXT DEFAULT 'provisioning'
);

-- Todos with image support
CREATE TABLE todos (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    completed BOOLEAN DEFAULT FALSE,
    image_data TEXT,  -- Base64 encoded
    image_mime_type TEXT
);
```

## User Flow

1. **Sign Up**
   - User provides email and password
   - Selects preferred region from dropdown
   - Account created with hashed password
   - Tenant app provisioned in selected region

2. **Dashboard**
   - Shows all user's todos
   - Lists provisioned regions with status
   - Allows switching between regions

3. **Todo Management**
   - Create todos with title and description
   - Mark as complete/incomplete
   - Upload images (up to 5MB)
   - Delete todos

## Development

### Local Development

```bash
# Install dependencies
cargo build

# Set up environment
cp .env.example .env

# Run locally (without Minifly)
cargo run
```

### Project Structure

```
todo-auth-app/
├── src/
│   ├── main.rs          # Application entry point
│   ├── auth.rs          # Authentication logic
│   ├── db.rs            # Database setup
│   ├── error.rs         # Error handling
│   ├── models.rs        # Data structures
│   ├── tenant.rs        # Tenant provisioning
│   ├── templates.rs     # Template definitions
│   └── handlers/        # Route handlers
├── templates/           # HTML templates
├── static/             # CSS and JavaScript
├── migrations/         # Database migrations
└── Cargo.toml         # Dependencies
```

## Security Features

- **Password Security**: Argon2 hashing with salt
- **Session Management**: Secure cookie-based sessions
- **CSRF Protection**: State-changing operations use POST
- **Input Validation**: Email format, password length
- **SQL Injection Prevention**: Parameterized queries via SQLx

## Extending the Example

### Ideas for Enhancement

1. **Regional Data Storage**
   - Move todos to regional databases
   - Implement cross-region sync

2. **Advanced Features**
   - Real-time updates with WebSockets
   - Todo sharing between users
   - Export/import functionality
   - Rich text editor for descriptions

3. **Monitoring**
   - Add metrics collection
   - Resource usage per tenant
   - Usage quotas and limits

## Troubleshooting

### Container fails to start
```bash
# Check platform status
minifly status

# View logs
minifly logs <machine-id>

# Verify Docker
docker ps
```

### Authentication issues
- Ensure `SESSION_SECRET` is set
- Check cookies are enabled
- Verify database migrations ran

### Tenant provisioning fails
- Check `MINIFLY_API_URL` is accessible
- Ensure sufficient Docker resources
- View Minifly API logs

## Key Takeaways

This example demonstrates:
- Building secure multi-tenant applications
- Integrating with Minifly's API for dynamic provisioning
- Handling file uploads and storage
- Creating responsive web interfaces
- Implementing production-ready authentication

Perfect for learning how to build SaaS applications with per-customer isolation!