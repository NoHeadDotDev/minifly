# Todo Auth App - Multi-Tenant Example

A comprehensive example demonstrating Minifly's multi-tenant capabilities with authentication, per-user apps/databases, region selection, and image storage.

## Features

- 🔐 **Email + Password Authentication**: Secure user authentication with cookie-based sessions
- 🌍 **Multi-Region Deployment**: Users select their region at signup, and their app/database is deployed there
- 👤 **Per-User Isolation**: Each user gets their own app instance and database
- 📝 **Todo Management**: Full CRUD operations for todo items
- 🖼️ **Image Storage**: Upload and store images with todos
- 🎨 **Responsive UI**: Clean, modern interface that works on all devices

## Quick Start

1. **Start Minifly platform** (if not already running):
   ```bash
   minifly serve --dev
   ```

2. **Deploy the app**:
   ```bash
   cd examples/todo-auth-app
   minifly deploy
   ```

3. **Access the app**: 
   - Check the deployment output for the URL (e.g., `http://localhost:32768`)
   - Or run `docker ps` to see the assigned port

## Architecture

### Multi-Tenant Design

Each user gets:
- A dedicated app instance (`todo-user-{userId}`)
- An isolated SQLite database in their chosen region
- Secure volume mounts for persistent storage

### Region Selection

Users choose from 9 global regions at signup:
- North America: `iad` (Virginia), `ord` (Chicago), `lax` (Los Angeles)
- Europe: `lhr` (London), `ams` (Amsterdam), `fra` (Frankfurt)
- Asia-Pacific: `syd` (Sydney), `nrt` (Tokyo), `sin` (Singapore)

### Security Features

- Passwords hashed with Argon2
- Session tokens stored securely
- Per-user data isolation
- CSRF protection via POST-only mutations

## Development

### Local Development

```bash
# Install dependencies
cargo build

# Set up environment
cp .env.example .env
# Edit .env with your settings

# Run locally
cargo run
```

### Environment Variables

- `DATABASE_URL`: SQLite database path (default: `sqlite:///litefs/app.db`)
- `SESSION_SECRET`: Secret key for session encryption (generate with `openssl rand -hex 32`)
- `MINIFLY_API_URL`: URL to Minifly API (default: `http://localhost:4280`)
- `PORT`: Application port (default: `8080`)

### Database Schema

The app uses SQLite with three main tables:
- `users`: User accounts with email and password
- `user_apps`: Tenant app deployments per region
- `todos`: User todo items with optional images

## How It Works

1. **User Signup**:
   - User provides email, password, and selects a region
   - Account is created in the main database
   - Minifly provisions a new app/database in the selected region
   - User is redirected to their dashboard

2. **Tenant Provisioning**:
   - Each user gets a unique app name: `todo-user-{userId}`
   - A Docker container is created with isolated storage
   - The container runs in the user's selected region
   - Status is tracked in the `user_apps` table

3. **Todo Management**:
   - Todos are stored in the main database
   - Images are base64-encoded and stored with todos
   - Each user only sees their own todos

4. **Region Dashboard**:
   - Users can see all their provisioned regions
   - Click on a region to view region-specific details
   - Future enhancement: Store todos in regional databases

## Extending the Example

### Ideas for Enhancement

1. **Regional Data Storage**: 
   - Move todo storage to regional databases
   - Implement data sync between regions

2. **Advanced Features**:
   - Todo sharing between users
   - Real-time updates with WebSockets
   - Export/import functionality

3. **Monitoring**:
   - Add metrics collection
   - Show resource usage per tenant
   - Implement usage quotas

## Troubleshooting

### Container fails to start
- Check if Minifly platform is running: `minifly status`
- Verify Docker is running: `docker ps`
- Check logs: `minifly logs <machine-id>`

### Authentication issues
- Ensure `SESSION_SECRET` is set and consistent
- Check browser cookies are enabled
- Verify database migrations ran successfully

### Tenant provisioning fails
- Check `MINIFLY_API_URL` is correct
- Ensure Minifly API is accessible from the container
- Check Docker has sufficient resources

## License

This example is part of the Minifly project and follows the same license terms.