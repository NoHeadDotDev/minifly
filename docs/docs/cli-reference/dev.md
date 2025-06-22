# minifly dev

Start Minifly in development mode with enhanced debugging and hot reloading.

## Synopsis

```bash
minifly dev [OPTIONS]
```

## Description

The `dev` command combines `serve` and `deploy --watch` for an optimized development experience:
- Starts the platform with enhanced logging
- Enables hot reloading for code changes
- Provides detailed debug output
- Automatically rebuilds and redeploys on changes
- Shows real-time logs from all services

## Options

- `--port <PORT>` - API server port (default: 4280)
- `--app <APP>` - Application to watch (defaults to app in fly.toml)
- `--no-watch` - Disable file watching
- `--no-logs` - Don't stream logs
- `--debug` - Enable debug logging for all components
- `-h, --help` - Print help information

## Features

### Automatic Platform Startup
```bash
$ minifly dev
🚀 Starting Minifly in development mode...
  ✓ API server started on http://localhost:4280
  ✓ Docker service ready
  ✓ LiteFS clusters initialized
  ✓ Network bridges configured
```

### File Watching
Automatically rebuilds and redeploys when files change:
- Source code files
- Dockerfile
- fly.toml
- Package files (package.json, Cargo.toml, etc.)

### Live Logging
Streams logs from all services in real-time:
```
[API] 2024-06-22T10:30:45Z INFO Server listening on :4280
[APP] 2024-06-22T10:30:46Z INFO Application starting
[APP] 2024-06-22T10:30:47Z INFO Listening on port 8080
```

### Enhanced Debugging
- Detailed error messages
- Stack traces for failures  
- Request/response logging
- Performance metrics
- Container inspection

## Examples

### Basic Development Mode
```bash
$ minifly dev
🚀 Starting Minifly in development mode...
  ✓ Platform ready
  ✓ Watching for changes in ./
  ✓ Streaming logs

[10:30:45] [API] Server started
[10:30:46] [APP] Deploying my-app
[10:30:52] [APP] ✓ Deployment complete
[10:31:15] [WATCH] Changed: src/main.rs
[10:31:15] [APP] Rebuilding...
[10:31:22] [APP] ✓ Redeployed successfully
```

### Specific Application
```bash
$ minifly dev --app my-app
🚀 Development mode for 'my-app'
  ✓ Found fly.toml
  ✓ Watching src/, Dockerfile, fly.toml
```

### Custom Port
```bash
$ minifly dev --port 5000
🚀 Starting Minifly in development mode...
  ✓ API server started on http://localhost:5000
```

### Without File Watching
```bash
$ minifly dev --no-watch
🚀 Starting Minifly in development mode...
  ✓ Platform ready
  ℹ File watching disabled
```

## Development Workflow

1. **Start Development Mode**
   ```bash
   minifly dev
   ```

2. **Make Code Changes**
   - Edit your source files
   - Save changes
   - Minifly automatically rebuilds and redeploys

3. **View Live Logs**
   - See compilation output
   - Watch application logs
   - Debug issues in real-time

4. **Test Changes**
   - Access your app at the proxied URL
   - Test API endpoints
   - Verify functionality

## File Watch Patterns

### Included by Default
- `**/*.{js,ts,jsx,tsx,py,go,rs,rb,java}`
- `**/Dockerfile*`
- `**/package*.json`
- `**/Cargo.toml`
- `**/go.mod`
- `**/requirements.txt`
- `**/Gemfile*`
- `fly.toml`

### Excluded by Default
- `.git/`
- `node_modules/`
- `target/` (Rust)
- `__pycache__/`
- `*.log`
- `.env*`
- Build artifacts

### Custom Watch Patterns (coming soon)
```bash
# Include additional patterns
minifly dev --watch "**/*.sql" --watch "config/*.yml"

# Exclude patterns  
minifly dev --ignore "tests/**" --ignore "docs/**"
```

## Debug Output

Enable comprehensive debugging:
```bash
$ minifly dev --debug
[DEBUG] Platform configuration:
  API Port: 4280
  Data Directory: ./data
  Docker Socket: /var/run/docker.sock

[DEBUG] Watching files:
  Pattern: **/*
  Ignoring: .git, node_modules, target

[DEBUG] Container created: d891234567890
[DEBUG] Port mapping: 0.0.0.0:32768 -> 8080/tcp
```

## Multi-Service Development

Develop multiple services together:

```bash
# Terminal 1: Frontend
cd frontend && minifly dev --app frontend

# Terminal 2: Backend API  
cd backend && minifly dev --app backend

# Terminal 3: Database
cd database && minifly dev --app postgres
```

## Performance Optimization

### Faster Rebuilds
- Use Docker build cache
- Optimize Dockerfile layers
- Use .dockerignore

### Reduce File Watching
```bash
# Watch only source directory
minifly dev --watch "src/**"
```

### Incremental Compilation
- Language-specific optimizations
- Use development containers
- Cache dependencies

## Integration with IDEs

### VS Code
```json
// .vscode/tasks.json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Minifly Dev",
      "type": "shell",
      "command": "minifly dev",
      "isBackground": true,
      "problemMatcher": []
    }
  ]
}
```

### IntelliJ IDEA
Configure as external tool:
- Program: `minifly`
- Arguments: `dev`
- Working directory: `$ProjectFileDir$`

## Environment Variables

Set development-specific variables:

```bash
# .env.development
DEBUG=true
LOG_LEVEL=debug
DATABASE_URL=postgres://localhost:5432/dev
```

Load in dev mode:
```bash
minifly dev --env-file .env.development
```

## Troubleshooting

### Platform Won't Start
```bash
# Check if port is in use
lsof -i :4280

# Use different port
minifly dev --port 4281

# Clean start
minifly stop --force && minifly dev
```

### File Changes Not Detected
```bash
# Check watch patterns
minifly dev --debug

# Force polling (for network drives)
MINIFLY_WATCH_POLL=true minifly dev
```

### Build Failures
```bash
# See full error output
minifly dev --debug

# Check Dockerfile
docker build . --no-cache

# Verify fly.toml
minifly config validate
```

## Best Practices

1. **Use .dockerignore** to speed up builds
2. **Structure code** for fast compilation
3. **Keep images small** for quick deployment
4. **Use build caching** effectively
5. **Monitor resource usage** during development

## Comparison with Production

| Feature | Dev Mode | Production |
|---------|----------|------------|
| Logging | Verbose | Structured |
| Rebuilds | Automatic | Manual |
| Optimization | Speed | Size |
| Error Details | Full | Limited |
| File Watching | Enabled | Disabled |

## See Also

- [serve](./serve) - Start platform normally
- [deploy](./deploy) - Deploy with options
- [logs](./logs) - View detailed logs
- [status](./status) - Check platform status