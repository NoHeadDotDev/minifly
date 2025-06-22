# Minifly 🚀

[![Crates.io](https://img.shields.io/crates/v/minifly.svg)](https://crates.io/crates/minifly)
[![Documentation](https://img.shields.io/badge/docs-minifly--docs.fly.dev-blue)](https://minifly-docs.fly.dev)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Local Fly.io development simulator with incredible developer experience**

Minifly provides a complete local development environment that simulates the Fly.io platform, allowing you to develop, test, and debug your applications with the same APIs and behavior you'll see in production.

## 🚀 Quick Start

### Install from crates.io

```bash
cargo install minifly
```

### Initialize and start

```bash
# Initialize Minifly environment
minifly init

# Start the platform
minifly serve

# Deploy your first app
minifly deploy
```

## ✨ Features

- 🚀 **Complete Fly.io API Compatibility** - Full Machines API with Docker integration
- 🗄️ **LiteFS Integration** - Distributed SQLite with local replication testing
- 🔥 **Incredible Developer Experience** - Hot reloading, watch mode, structured logging
- 🌍 **Multi-region Simulation** - Test region-specific behavior locally
- 📊 **Real-time Monitoring** - Comprehensive status dashboards and logging
- 🐳 **Docker Management** - Automatic container lifecycle management
- ⚡ **Lightning Fast** - Instant deployments and real-time feedback

## 📋 Commands

### Platform Management

```bash
# Start the Minifly platform
minifly serve

# Start in development mode with enhanced logging
minifly serve --dev

# Stop the platform
minifly stop
```

### Application Management

```bash
# Create an application
minifly apps create my-app

# List applications
minifly apps list

# Delete an application
minifly apps delete my-app
```

### Machine Management

```bash
# Create a machine
minifly machines create --app my-app --image nginx:latest

# List machines
minifly machines list --app my-app

# Start/stop machines
minifly machines start <machine-id>
minifly machines stop <machine-id>
```

### Development Workflow

```bash
# Deploy with automatic redeployment on changes
minifly deploy --watch

# View real-time logs with region context
minifly logs <machine-id> --follow

# Check platform status
minifly status
```

## 🏗️ Multi-tenant Applications

Minifly excels at simulating multi-tenant architectures with per-tenant databases:

```rust
// Each tenant gets their own replicated database
let db_path = format!("/litefs/{}.db", tenant_id);
let pool = SqlitePool::connect(&db_path).await?;
```

## 📊 Structured Logging

Get comprehensive observability with built-in structured logging:

```bash
# Enable debug logging
MINIFLY_DEBUG=1 minifly serve

# JSON formatted logs for production
MINIFLY_LOG_JSON=1 minifly deploy
```

All operations include:
- **Correlation IDs** for request tracking
- **Region context** for multi-region testing
- **Performance metrics** and timing information
- **Structured error context** for debugging

## 🌐 API Compatibility

Minifly implements the complete Fly.io Machines API v1:

```bash
# All standard Fly.io API endpoints work locally
curl -X POST http://localhost:4280/v1/apps/my-app/machines \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"config": {"image": "nginx:latest"}}'
```

## 🔧 Configuration

### Environment Variables

- `MINIFLY_API_PORT`: API server port (default: 4280)
- `MINIFLY_LOG_LEVEL`: Log level (default: info)
- `MINIFLY_LOG_JSON`: Use JSON logging format
- `MINIFLY_DEBUG`: Enable debug logging
- `MINIFLY_DATA_DIR`: Data directory for volumes (default: ./data)

### Configuration File

Create `~/.config/minifly/config.toml`:

```toml
api_url = "http://localhost:4280"
token = "your-api-token"

[logging]
level = "info"
format = "human"  # or "json"
```

## 🚀 Examples

### Simple Web App

```toml
# fly.toml
app = "my-web-app"
primary_region = "local"

[[services]]
internal_port = 8080
protocol = "tcp"

  [[services.ports]]
  port = 80
  handlers = ["http"]

  [[services.ports]]
  port = 443
  handlers = ["tls", "http"]
```

```bash
# Deploy and watch for changes
minifly deploy --watch
```

### Multi-tenant SaaS

```toml
# fly.toml for multi-tenant app
app = "saas-app"
primary_region = "local"

[env]
DATABASE_URL = "/litefs/primary.db"
TENANT_ISOLATION = "database"

[[mounts]]
source = "sqlite_data"
destination = "/litefs"
```

## 📚 Documentation

- **[Full Documentation](https://minifly-docs.fly.dev)** - Complete documentation site
- **[Getting Started Guide](https://minifly-docs.fly.dev/docs/getting-started)** - Complete setup tutorial
- **[API Reference](https://minifly-docs.fly.dev/docs/api-reference)** - Full API documentation
- **[GitHub Repository](https://github.com/NoHeadDotDev/minifly)** - Source code and examples

## 🤝 Contributing

Contributions are welcome! Please see our [Contributing Guide](https://github.com/NoHeadDotDev/minifly/blob/main/CONTRIBUTING.md) for details.

## 📄 License

Licensed under the MIT License. See [LICENSE](https://github.com/NoHeadDotDev/minifly/blob/main/LICENSE) for details.

---

**Happy local development!** 🎉

For questions and support, visit our [GitHub repository](https://github.com/NoHeadDotDev/minifly).