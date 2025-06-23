# Minifly Examples

This directory contains example applications demonstrating how to use Minifly for local development.

## Available Examples

### 1. Basic App (`basic-app/`)
A simple "Hello World" application showing the basics of deploying to Minifly.

**Features:**
- Simple HTTP server
- Basic fly.toml configuration
- Dockerfile setup
- Auto-deployment with `minifly serve --dev`

**Run:** `cd basic-app && ../run.sh`

### 2. Multi-Tenant App (`multi-tenant-app/`)
Demonstrates multi-tenant architecture with per-tenant databases.

**Features:**
- Tenant isolation
- LiteFS for distributed SQLite
- Dynamic tenant routing
- Per-tenant data storage

**Run:** `cd multi-tenant-app && ./run.sh`

### 3. Todo Auth App (`todo-auth-app/`) ⭐ NEW
A comprehensive example showcasing authentication and multi-tenant capabilities.

**Features:**
- 🔐 Email + password authentication with sessions
- 🌍 Per-user apps and databases in selected regions
- 📝 Todo management with image uploads
- 🎨 Modern, responsive UI
- 👤 Complete user isolation

**Run:** `cd todo-auth-app && ./run.sh`

### 4. Production Config (`production-config/`)
Shows how to use production Fly.io configurations with Minifly.

**Features:**
- Complex fly.toml with multiple services
- Environment variable handling
- Health checks and monitoring
- Production-ready patterns

**Run:** `cd production-config && minifly deploy`

## Getting Started

1. **Install Minifly** (if not already installed):
   ```bash
   cargo install minifly
   ```

2. **Start the Minifly platform**:
   ```bash
   minifly serve --dev
   ```

3. **Choose an example** and follow its README

## Development Tips

- Use `minifly serve --dev` for auto-deployment and hot reloading
- Check `docker ps` to see running containers
- Use `minifly logs <machine-id>` to debug issues
- Each example has its own README with detailed instructions

## Creating Your Own Example

To create a new example:

1. Create a new directory under `examples/`
2. Add a `fly.toml` configuration
3. Create a `Dockerfile` or use a pre-built image
4. Add a `README.md` explaining the example
5. (Optional) Add a `run.sh` script for easy deployment

## Contributing

We welcome new examples! Please ensure your example:
- Demonstrates a specific Minifly feature or use case
- Includes clear documentation
- Works with `minifly deploy` out of the box
- Follows the existing example structure