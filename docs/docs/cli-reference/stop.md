# minifly stop

Stop the Minifly platform gracefully.

## Synopsis

```bash
minifly stop [OPTIONS]
```

## Description

The `stop` command gracefully shuts down the Minifly platform, ensuring:
- All machines are stopped cleanly
- Data is persisted properly
- LiteFS clusters are shutdown safely
- Temporary resources are cleaned up
- No data loss occurs

## Options

- `--force` - Force immediate shutdown (skip graceful shutdown)
- `--timeout <SECONDS>` - Maximum time to wait for graceful shutdown (default: 60)
- `--keep-data` - Preserve data directories after shutdown
- `-h, --help` - Print help information

## Shutdown Process

1. **Signal Services** - Send shutdown signal to all services
2. **Stop Machines** - Gracefully stop all running machines
3. **Sync Data** - Ensure all data is written to disk
4. **Stop LiteFS** - Cleanly shutdown LiteFS clusters
5. **Stop API** - Shutdown the API server
6. **Cleanup** - Remove temporary files and resources

## Examples

### Basic Stop
```bash
$ minifly stop
🔄 Initiating graceful shutdown...
  ✓ Stopping 5 machines
  ✓ Syncing data
  ✓ Stopping LiteFS clusters
  ✓ Stopping API server
  ✓ Cleaning up resources
✅ Minifly platform stopped successfully
```

### Force Stop
```bash
$ minifly stop --force
⚠️  Forcing immediate shutdown...
✅ Platform stopped (some data may be lost)
```

### Stop with Extended Timeout
```bash
$ minifly stop --timeout 120
🔄 Initiating graceful shutdown (timeout: 120s)...
  ✓ All services stopped cleanly
```

## Data Preservation

By default, stop removes temporary data:
- Container logs
- Temporary build artifacts
- Cache files

To preserve all data:
```bash
minifly stop --keep-data
```

Preserved data includes:
- Application databases
- Volume data
- LiteFS data
- Configuration files

## Graceful vs Force Shutdown

### Graceful (Default)
- Sends SIGTERM to processes
- Waits for clean shutdown
- Ensures data consistency
- May take up to 60 seconds

### Force (--force)
- Sends SIGKILL to processes
- Immediate termination
- Risk of data loss
- Use only when graceful fails

## Integration with System

### Systemd Service
```ini
[Unit]
Description=Minifly Platform
After=docker.service

[Service]
Type=simple
ExecStart=/usr/local/bin/minifly serve
ExecStop=/usr/local/bin/minifly stop
Restart=on-failure
TimeoutStopSec=90

[Install]
WantedBy=multi-user.target
```

### Shell Script
```bash
#!/bin/bash
# Graceful restart script

echo "Stopping Minifly..."
minifly stop --timeout 30

echo "Waiting for cleanup..."
sleep 5

echo "Starting Minifly..."
minifly serve
```

## Handling Running Applications

When stopping the platform:
- Running machines receive SIGTERM
- Apps have 30 seconds to shutdown gracefully
- Persistent data is preserved
- Machines can be restarted with `minifly serve`

## Troubleshooting

### Stop Hangs
```bash
# Check what's still running
minifly status

# Force stop if necessary
minifly stop --force

# Check processes manually
ps aux | grep minifly
```

### Permission Errors
```bash
# May need sudo for some cleanups
sudo minifly stop

# Or run with debug
MINIFLY_DEBUG=1 minifly stop
```

### Data Not Cleaned
```bash
# Manual cleanup if needed
rm -rf ~/.local/share/minifly/temp/*
rm -rf /tmp/minifly-*
```

## Exit Codes

- `0` - Successful shutdown
- `1` - General error
- `2` - Timeout reached
- `3` - Platform not running
- `130` - Interrupted (Ctrl+C)

## Signal Handling

The stop command handles signals:
- `SIGTERM` - Graceful shutdown
- `SIGINT` - Graceful shutdown (Ctrl+C)
- `SIGKILL` - Cannot be caught (force kill)

## Best Practices

1. **Always use graceful shutdown** unless there's an issue
2. **Allow sufficient timeout** for large deployments
3. **Check status after stop** to ensure clean shutdown
4. **Backup data** before force stops
5. **Monitor logs** during shutdown for issues

## Stop vs Restart

To restart the platform:
```bash
# Stop and start
minifly stop && minifly serve

# Or use system service
systemctl restart minifly
```

## Partial Shutdown

Stop specific services (coming soon):
```bash
# Stop only LiteFS
minifly stop --service litefs

# Stop specific app
minifly apps stop my-app
```

## See Also

- [serve](./serve) - Start the platform
- [status](./status) - Check platform status
- [machines](./machines) - Stop individual machines