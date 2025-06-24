# Minifly 🚀

**Local Fly.io development simulator with incredible DX**

Minifly provides a complete local development environment that simulates the Fly.io platform, allowing you to develop, test, and debug your applications with the same APIs and behavior you'll see in production.

## ✨ Features

- 🚀 **Complete Fly.io API Compatibility** - Full Machines API with Docker integration
- 🗄️ **LiteFS Integration** - Distributed SQLite with local replication testing
- 🔥 **Incredible Developer Experience** - Hot reloading, watch mode, structured logging
- 🌍 **Multi-region Simulation** - Test region-specific behavior locally
- 📊 **Real-time Monitoring** - Comprehensive status dashboards and logging
- 🐳 **Docker Management** - Automatic container lifecycle management
- ⚡ **Lightning Fast** - Instant deployments and real-time feedback
- 🎯 **Auto-deployment** - Just run `minifly serve --dev` in your project directory
- 🔌 **No Port Conflicts** - Automatic port allocation for all containers

## 📖 Documentation

**Full documentation is available at [https://minifly-docs.fly.dev](https://minifly-docs.fly.dev)**

Quick links:
- **[Getting Started](https://minifly-docs.fly.dev/docs/getting-started)** - Complete setup guide with examples
- **[CLI Reference](https://minifly-docs.fly.dev/docs/cli-reference)** - All commands and options
- **[API Reference](https://minifly-docs.fly.dev/docs/api-reference)** - Complete API documentation  
- **[Examples](https://github.com/NoHeadDotDev/minifly/tree/main/examples)** - Real-world application examples

### Run Documentation Locally

```bash
cd docs
npm install
npm start
```

This will start a local Docusaurus server at `http://localhost:3000`.

### LiteFS Support

Minifly includes comprehensive LiteFS support:

- **Automatic Binary Management**: Downloads and manages LiteFS binary automatically
- **FUSE Filesystem**: Mounts SQLite databases through FUSE for transparent replication
- **Primary/Replica Support**: Simulates primary election and read replicas
- **Cluster Management**: Create multi-node SQLite clusters locally
- **Compatible with Fly.io**: Uses the same LiteFS configuration format

## Legal Notice

**Minifly is not affiliated with, endorsed by, or sponsored by Fly.io.** This is an independent project created for local development purposes. Fly.io is a trademark of Fly.io, Inc.

## Architecture

Minifly consists of several components:

- **minifly-api**: REST API server implementing the Machines API
- **minifly-litefs**: LiteFS FUSE filesystem and replication (coming soon)
- **minifly-network**: Networking and DNS simulation (coming soon)
- **minifly-cli**: Command-line interface
- **minifly-core**: Shared types and models

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Docker or Podman
- SQLite

### Quick Start

The easiest way to get started:

```bash
# Install Minifly
cargo install minifly-cli

# In your project directory with a fly.toml file
minifly serve --dev
```

That's it! Minifly will:
- ✅ Start the platform automatically
- ✅ Detect your project configuration
- ✅ Deploy your application
- ✅ Show you the URL with the assigned port
- ✅ Watch for file changes and redeploy

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/minifly
cd minifly

# Build all components
cargo build --release

# Install the CLI
cargo install --path minifly-cli
```

### Manual Usage

For more control, you can use individual commands:

```bash
# Start the platform
minifly serve

# Deploy from fly.toml
minifly deploy

# Deploy with specific config files (NEW!)
minifly deploy --config fly.production.toml
minifly deploy --config fly.staging.toml --litefs-config litefs.prod.yml

# Create apps and machines manually
minifly apps create my-app
minifly machines create --app my-app --image nginx:latest

# List and manage machines
minifly machines list --app my-app
minifly machines start <machine-id>
minifly machines stop <machine-id>

# View logs
minifly logs <machine-id>
```

## 🎯 Developer Experience

Minifly is designed to provide an incredible developer experience:

### One-Command Development

Just run `minifly serve --dev` in any directory with a `fly.toml` file:

```bash
cd my-fly-app
minifly serve --dev
# 🚀 Platform starts
# 📦 App deploys automatically
# 🔗 Access your app at: http://localhost:32768
# 👀 Watching for changes...
```

### No More Port Conflicts

Minifly automatically assigns available ports to your containers:
- No more "port already in use" errors
- Run multiple apps simultaneously
- Each deployment shows you the exact URL

### File Watching & Hot Reload

In development mode (`--dev`), Minifly watches your files and automatically redeploys when you make changes:
- Watches `fly.toml`, `Dockerfile`, and source files
- Instant feedback on changes
- See deployment logs in real-time

### Clean Shutdown

Press Ctrl+C once to gracefully shut down:
- All containers stop cleanly
- Resources are properly released
- No hanging processes

## API Compatibility

Minifly implements the Fly.io Machines API v1. You can use the same API endpoints:

```bash
# Create a machine
curl -X POST http://localhost:4280/v1/apps/my-app/machines \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "config": {
      "image": "nginx:latest",
      "guest": {
        "cpu_kind": "shared",
        "cpus": 1,
        "memory_mb": 256
      }
    }
  }'
```

## LiteFS Usage

### Creating a Machine with LiteFS

```bash
# Create a machine with LiteFS volume
minifly machines create --app my-app \
  --image myapp:latest \
  --env DATABASE_URL=/litefs/app.db \
  --env FLY_LITEFS_PRIMARY=true \
  --mount volume=sqlite_data,path=/litefs
```

### Running Examples

```bash
# Basic usage example
cargo run --example basic_usage

# LiteFS cluster example
cargo run --example litefs_cluster

# Multi-tenant application example
cd examples/multi-tenant-app
./run-dev.sh  # Run locally
# OR deploy with Minifly:
minifly deploy
```

## Configuration

### Multiple Configuration Files (NEW!)

Minifly now supports multiple configuration files for different environments, just like Fly.io:

#### Environment-specific fly.toml

```bash
# Use explicit config file
minifly deploy --config fly.production.toml
minifly deploy --config fly.staging.toml

# Or use environment variables for automatic discovery
FLY_ENV=dev minifly deploy      # Uses fly.dev.toml if it exists
MINIFLY_ENV=staging minifly deploy  # Uses fly.staging.toml if it exists
```

#### Environment-specific litefs.yml

```bash
# Use explicit config file
minifly deploy --litefs-config litefs.production.yml

# Or use environment variables
LITEFS_CONFIG_PATH=litefs.prod.yml minifly deploy
FLY_ENV=dev minifly deploy  # Uses litefs.dev.yml if it exists
```

#### Configuration Precedence

1. Explicit `--config` or `--litefs-config` flags (highest priority)
2. `LITEFS_CONFIG_PATH` environment variable (for LiteFS)
3. Environment-specific files based on `FLY_ENV` or `MINIFLY_ENV`
4. Default files (`fly.toml` and `litefs.yml`)

### Environment Variables

- `MINIFLY_API_PORT`: API server port (default: 4280)
- `MINIFLY_DATABASE_URL`: SQLite database URL (default: sqlite:minifly.db)
- `DOCKER_HOST`: Docker socket path
- `MINIFLY_NETWORK_PREFIX`: IPv6 network prefix (default: fdaa:0:)
- `MINIFLY_DATA_DIR`: Data directory for LiteFS and volumes (default: ./data)
- `FLY_ENV` or `MINIFLY_ENV`: Environment name for config file discovery
- `LITEFS_CONFIG_PATH`: Path to LiteFS configuration file

### CLI Configuration

The CLI stores configuration in `~/.config/minifly/config.toml`:

```toml
api_url = "http://localhost:4280"
token = "your-api-token"
```

## Development

### Project Structure

```
minifly/
├── minifly-api/          # Machines API server
├── minifly-litefs/       # LiteFS implementation
├── minifly-network/      # Networking simulation
├── minifly-cli/          # CLI application
└── minifly-core/         # Shared types and utilities
```

### Running Tests

```bash
cargo test --workspace
```

### Building Individual Components

```bash
# Build only the API server
cargo build -p minifly-api

# Build only the CLI
cargo build -p minifly-cli
```

## Examples

Check out our example applications in the `examples/` directory:

### 🔐 Todo Auth App (Featured)

A comprehensive multi-tenant SaaS application demonstrating:
- **Email + password authentication** with secure sessions
- **Per-user app isolation** with dedicated databases
- **Multi-region deployment** with user-selectable regions
- **Image upload and storage** for todos
- **Modern responsive UI** with server-side rendering

```bash
cd examples/todo-auth-app
minifly deploy  # or ./run.sh
```

### 🏢 Multi-Tenant Application

A database-per-tenant architecture example featuring:
- **Per-tenant SQLite databases** managed by LiteFS
- **Axum web framework** with async Rust
- **Automatic tenant detection** from headers, subdomains, or paths
- **Production-ready patterns** for SaaS applications

```bash
cd examples/multi-tenant-app
./run.sh
```

### 🚀 More Examples

- **Basic App** - Simple HTTP server to get started
- **Production Config** - Advanced fly.toml features and configurations

See the [examples directory](./examples) for all available examples.

## Roadmap

- [x] Basic Machines API implementation
- [x] Docker container management
- [x] CLI tool
- [x] LiteFS integration with actual binary
- [x] LiteFS process management
- [x] fly.toml deployment support
- [x] Multi-tenant application example
- [ ] SQLite state persistence
- [ ] LiteFS cluster coordination
- [ ] Network simulation
- [ ] Volume management
- [ ] Health checks
- [ ] Autoscaling simulation

## Differences from Fly.io

While Minifly aims to closely simulate Fly.io's behavior, there are some differences:

1. **Local Only**: Runs entirely on your local machine
2. **Simplified Networking**: Uses Docker networking instead of WireGuard
3. **No Multi-region**: All "regions" are simulated locally
4. **Limited Autoscaling**: Basic autoscaling simulation only
5. **Storage**: Uses local Docker volumes instead of distributed storage

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details