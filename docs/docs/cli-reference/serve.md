# minifly serve

Start the Minifly platform, including the API server and LiteFS integration.

## Usage

```bash
minifly serve [OPTIONS]
```

## Description

The `serve` command starts the complete Minifly platform with all necessary services:

- **API Server**: Provides Fly.io Machines API compatibility
- **LiteFS**: Distributed SQLite replication system
- **Health Monitoring**: Service dependency checks
- **Docker Integration**: Container lifecycle management

This is typically the first command you'll run when starting development with Minifly.

## Options

| Option | Short | Type | Description | Default |
|--------|-------|------|-------------|---------|
| `--daemon` | `-d` | Flag | Run in background as daemon | `false` |
| `--port` | `-p` | Number | Port for API server | `4280` |
| `--dev` | | Flag | Enable development mode with enhanced logging | `false` |

## Examples

### Basic Usage

Start the platform in foreground mode:

```bash
minifly serve
```

Output:
```
🚀 Starting Minifly Platform
📦 Starting services...
   • Starting API Server on port 4280...
   ✓ API Server is ready
   • Starting LiteFS...
   ✓ LiteFS configuration detected

✅ Minifly platform started successfully!
🌐 Services:
   API Server: http://localhost:4280
   LiteFS: Running

Press Ctrl+C to stop the platform
```

### Development Mode

Enable enhanced logging and debugging features:

```bash
minifly serve --dev
```

Development mode includes:
- Detailed debug logging
- Enhanced error messages
- Performance metrics
- Request/response tracing

### Custom Port

Run on a different port:

```bash
minifly serve --port 8080
```

### Background Mode

Run as a daemon process:

```bash
minifly serve --daemon
```

When running as a daemon:
- Process runs in background
- Output is suppressed
- Use `minifly stop` to shutdown
- Check status with `minifly status`

## Service Startup Sequence

Minifly starts services in the following order:

1. **Directory Setup**: Creates necessary data directories
2. **API Server**: Starts the HTTP server
3. **Health Check**: Waits for API server to be responsive
4. **LiteFS**: Starts distributed SQLite (if needed)
5. **Final Validation**: Ensures all services are healthy

## Health Checks

The serve command includes built-in health checks:

- **API Connectivity**: Verifies HTTP endpoints respond
- **Docker Integration**: Checks Docker daemon availability
- **LiteFS Status**: Validates distributed database setup
- **Resource Availability**: Ensures adequate disk space

## Configuration Detection

The serve command automatically detects and configures:

- **LiteFS Configuration**: Looks for `litefs.yml` files
- **Project Settings**: Scans for `fly.toml` files
- **Docker Environment**: Validates container runtime
- **Network Ports**: Checks for port conflicts

## Logging and Monitoring

### Standard Output

In foreground mode, you'll see:
- Service startup progress
- Health check results
- Error messages and warnings
- Shutdown notifications

### Development Mode Logging

With `--dev` flag:
- Request/response details
- Performance timings
- Debug-level messages
- Stack traces for errors

### Log Files

When running as daemon:
- Logs are written to `~/.minifly/logs/`
- Rotated daily to prevent disk usage issues
- Include structured data for analysis

## Shutdown

### Graceful Shutdown (Foreground)

Press `Ctrl+C` to trigger graceful shutdown:

```
🛑 Shutting down Minifly platform...
   • Stopping running machines...
   • Stopping LiteFS...
   • Stopping API server...
✅ Platform shutdown complete
```

### Daemon Shutdown

Use the stop command:

```bash
minifly stop
```

## Troubleshooting

### Port Already in Use

```bash
Error: Address already in use (os error 48)
```

**Solution**: Use a different port or stop the conflicting service:
```bash
minifly serve --port 4281
```

### Docker Not Available

```bash
Warning: Docker check failed - some features may not work
```

**Solution**: Ensure Docker is installed and running:
```bash
docker version
```

### Permission Denied

```bash
Error: Failed to create directory: Permission denied
```

**Solution**: Check directory permissions or run with appropriate privileges:
```bash
sudo minifly serve
# or
chmod 755 ~/.minifly
```

### LiteFS Configuration Error

```bash
Error: Failed to parse litefs.yml
```

**Solution**: Validate your LiteFS configuration:
```bash
# Check syntax
cat litefs.yml | yq .

# Use minimal config
minifly serve --dev  # Shows detailed error messages
```

## Integration with Other Commands

The serve command works seamlessly with other Minifly commands:

```bash
# Start platform
minifly serve --daemon

# Deploy an app (uses running platform)
minifly deploy

# Check status
minifly status

# View logs
minifly logs my-machine

# Stop platform
minifly stop
```

## Performance Considerations

### Resource Usage

- **Memory**: ~50MB base + containers
- **CPU**: Minimal when idle
- **Disk**: Logs and container data
- **Network**: Local ports only

### Scaling

For high-throughput development:
- Use SSD storage for database files
- Increase Docker resource limits
- Monitor with `minifly status`

---

**Next**: Learn about [`minifly dev`](./dev) for enhanced development workflows.