# CLI Reference

The Minifly CLI provides a comprehensive set of commands for managing your local Fly.io development environment. This reference covers all available commands and their options.

## Global Options

All Minifly commands support these global options:

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--api-url` | `-a` | API endpoint URL | `http://localhost:4280` |
| `--token` | `-t` | Authentication token | None |
| `--help` | `-h` | Show help information | - |

## Commands Overview

### Platform Management

| Command | Description |
|---------|-------------|
| [`minifly serve`](./serve) | Start the Minifly platform (API server + LiteFS) |
| [`minifly dev`](./dev) | Development mode with auto-reload and log streaming |
| [`minifly stop`](./stop) | Stop the Minifly platform |
| [`minifly status`](./status) | Show comprehensive platform status |

### Application Management

| Command | Description |
|---------|-------------|
| [`minifly apps`](./apps) | Manage applications |
| [`minifly deploy`](./deploy) | Deploy an application with optional watch mode |

### Machine Management

| Command | Description |
|---------|-------------|
| [`minifly machines`](./machines) | Manage machines (containers) |
| [`minifly logs`](./logs) | View logs from machines with region context |

### Utilities

| Command | Description |
|---------|-------------|
| [`minifly init`](./init) | Initialize Minifly environment |
| [`minifly proxy`](./proxy) | Proxy to a running service |

## Command Structure

Minifly follows a hierarchical command structure:

```bash
minifly [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS] [ARGS]
```

### Examples

```bash
# Start platform in development mode
minifly serve --dev

# Deploy with watch mode
minifly deploy --watch

# List machines for an app
minifly machines list --app my-app

# Follow logs for a specific machine
minifly logs --follow machine-123

# Show detailed status
minifly status
```

## Getting Help

Use the `--help` flag with any command to see detailed usage information:

```bash
# General help
minifly --help

# Command-specific help
minifly serve --help
minifly deploy --help
minifly machines --help
```

## Configuration

Minifly can be configured through:

1. **Command-line arguments** (highest priority)
2. **Environment variables**
3. **Configuration file** (created by `minifly init`)

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `MINIFLY_API_URL` | API server URL | `http://localhost:4280` |
| `MINIFLY_TOKEN` | Authentication token | None |
| `MINIFLY_LOG_LEVEL` | Logging level | `info` |

### Configuration File

The `minifly init` command creates a configuration file at `~/.minifly/config.toml`:

```toml
api_url = "http://localhost:4280"
# token = "your-token-here"
```

## Exit Codes

Minifly uses standard exit codes:

| Code | Description |
|------|-------------|
| `0` | Success |
| `1` | General error |
| `2` | Command-line usage error |
| `3` | API communication error |
| `4` | Docker/container error |
| `5` | LiteFS error |

## Tips and Tricks

### Auto-completion

Set up shell auto-completion for a better experience:

```bash
# For bash
minifly --generate-completion bash > ~/.minifly-completion.bash
echo 'source ~/.minifly-completion.bash' >> ~/.bashrc

# For zsh
minifly --generate-completion zsh > ~/.minifly-completion.zsh
echo 'source ~/.minifly-completion.zsh' >> ~/.zshrc
```

### Aliases

Create helpful aliases for common workflows:

```bash
# Quick development setup
alias mf-dev="minifly serve --dev"

# Deploy and watch
alias mf-deploy="minifly deploy --watch"

# Quick status check
alias mf-status="minifly status"
```

### Multiple Environments

You can run multiple Minifly instances on different ports:

```bash
# Development environment
minifly serve --port 4280

# Testing environment
minifly serve --port 4281

# Use with specific environment
minifly --api-url http://localhost:4281 status
```

---

Ready to explore specific commands? Start with [`minifly serve`](./serve) to learn about platform management.