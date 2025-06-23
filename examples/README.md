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

### 4. Production Config (`production-config/`) ⭐ ENHANCED
Comprehensive example of production config compatibility features.

**Features:**
- 🔐 **Secrets Management** - CLI-based secrets with automatic injection
- 🗄️ **LiteFS Production Adaptation** - Automatic production config adaptation
- 🌐 **Service Discovery** - `.internal` DNS resolution testing
- 📁 **Volume Mapping** - Production volume configurations
- ⚙️ **Environment Variables** - Complete Fly.io variable injection
- 🧪 **Feature Testing** - Comprehensive test endpoints and scripts

**Quick Start:**
```bash
cd production-config && ./run-demo.sh
```

**Test Features:**
```bash
# In another terminal after starting the app
cd production-config && ./test-features.sh
```

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
- Use `minifly secrets set KEY=value` to manage application secrets
- Production `fly.toml` and `litefs.yml` configs work without modifications
- `.internal` domains provide service discovery between applications
- Check `docker ps` to see running containers
- Use `minifly logs <machine-id>` to debug issues
- Each example has its own README with detailed instructions

## New Production Config Compatibility Features

Minifly now supports production Fly.io configurations out of the box:

- **🔐 Secrets Management**: Use `minifly secrets set/list/remove` for secure secrets
- **🗄️ LiteFS Integration**: Production `litefs.yml` automatically adapted for local dev
- **🌐 Service Discovery**: `.internal` DNS domains work just like in production
- **📁 Volume Mapping**: Production volume configs mapped to local directories
- **⚙️ Environment Variables**: All Fly.io variables (FLY_*) automatically injected

See the `production-config/` example for a comprehensive demonstration!

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