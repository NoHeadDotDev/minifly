# CLI Reference

The Minifly CLI provides a comprehensive set of commands for managing your local Fly.io development environment.

## Available Commands

- [Overview](./overview) - Complete command reference
- [serve](./serve) - Start the Minifly platform
- [init](./init) - Initialize a new project
- [apps](./apps) - Manage applications
- [machines](./machines) - Manage machines
- [deploy](./deploy) - Deploy applications
- [secrets](./secrets) - Manage application secrets
- [logs](./logs) - View logs
- [status](./status) - Check platform status
- [stop](./stop) - Stop the platform
- [proxy](./proxy) - Proxy to services
- [dev](./dev) - Development mode

## Quick Start

```bash
# Install Minifly
cargo install minifly

# Initialize configuration
minifly init

# Start the platform
minifly serve

# Deploy your first app
minifly deploy
```

## Global Options

All commands support these global options:

- `--config <PATH>` - Path to config file (default: ~/.config/minifly/config.toml)
- `--json` - Output in JSON format
- `--debug` - Enable debug logging
- `-h, --help` - Show help information
- `-V, --version` - Show version information

## Environment Variables

- `MINIFLY_API_URL` - Override API server URL
- `MINIFLY_TOKEN` - API authentication token
- `MINIFLY_DEBUG` - Enable debug mode
- `MINIFLY_LOG_JSON` - Use JSON log format

## Configuration

Minifly stores configuration in `~/.config/minifly/config.toml`:

```toml
api_url = "http://localhost:4280"
token = "your-api-token"
```

Use `minifly init` to create or update this configuration.