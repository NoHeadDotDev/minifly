# minifly status

Check the status of the Minifly platform and services.

## Synopsis

```bash
minifly status [OPTIONS]
```

## Description

The `status` command provides comprehensive information about:
- Platform health and availability
- Running services (API, LiteFS, Docker)
- Application and machine statistics
- Resource usage
- Recent activity

## Options

- `--service <SERVICE>` - Check specific service (api, docker, litefs)
- `--json` - Output in JSON format
- `--watch` - Continuously update status
- `--interval <SECONDS>` - Update interval for watch mode (default: 5)
- `-h, --help` - Print help information

## Output Sections

### Platform Status
```bash
$ minifly status

🚀 Minifly Platform Status
─────────────────────────
Platform:     Running
API Server:   http://localhost:4280
Started:      2024-06-22 10:00:00 (2 hours ago)
Version:      0.1.0
```

### Services Health
```
Services:
  ✓ API Server    Healthy   Response time: 5ms
  ✓ Docker        Healthy   Containers: 12 running
  ✓ LiteFS        Healthy   Clusters: 3 active
  ✓ Network       Healthy   Bridges: 4 configured
```

### Applications Summary
```
Applications:
  Total:        5
  With Machines: 4
  
  my-app        2 machines   sjc, ord
  test-app      1 machine    sjc
  db-cluster    3 machines   sjc
  web-service   2 machines   sjc, lhr
```

### Resource Usage
```
Resources:
  CPU Usage:    23.5%
  Memory:       1.2GB / 8.0GB
  Disk:         5.7GB / 100GB
  Containers:   12 running, 3 stopped
```

### Recent Activity
```
Recent Activity:
  10:45:32  Machine started     d891234567890 (my-app)
  10:42:15  App created        new-service
  10:40:08  Machine stopped    d891234567891 (test-app)
  10:35:22  Deploy completed   my-app
```

## Service-Specific Status

### API Server Status
```bash
$ minifly status --service api
API Server Status:
  URL:          http://localhost:4280
  Status:       Running
  Uptime:       2h 15m
  Requests:     1,234
  Avg Response: 8ms
  Active Conns: 5
```

### Docker Status
```bash
$ minifly status --service docker
Docker Service Status:
  Status:       Running
  Version:      24.0.5
  Containers:   12 running, 3 stopped
  Images:       25
  Volumes:      8
  Networks:     5
```

### LiteFS Status
```bash
$ minifly status --service litefs
LiteFS Service Status:
  Status:       Running
  Version:      0.5.0
  Clusters:     3
  Primary DBs:  3
  Replicas:     6
  Total Size:   127MB
```

## Watch Mode

Continuously monitor status:

```bash
# Update every 5 seconds
minifly status --watch

# Update every 2 seconds
minifly status --watch --interval 2
```

In watch mode:
- Updates clear the screen
- Press `Ctrl+C` to exit
- Highlights changes between updates

## JSON Output

For programmatic access:

```bash
$ minifly status --json
{
  "platform": {
    "status": "running",
    "version": "0.1.0",
    "api_url": "http://localhost:4280",
    "uptime_seconds": 7200
  },
  "services": {
    "api": {
      "status": "healthy",
      "response_time_ms": 5
    },
    "docker": {
      "status": "healthy",
      "containers_running": 12
    },
    "litefs": {
      "status": "healthy",
      "clusters": 3
    }
  },
  "applications": {
    "total": 5,
    "with_machines": 4,
    "list": [
      {
        "name": "my-app",
        "machines": 2,
        "regions": ["sjc", "ord"]
      }
    ]
  },
  "resources": {
    "cpu_percent": 23.5,
    "memory_used_gb": 1.2,
    "memory_total_gb": 8.0,
    "disk_used_gb": 5.7,
    "disk_total_gb": 100.0
  }
}
```

## Health Indicators

### Service States
- `✓ Healthy` - Service operating normally
- `⚠ Degraded` - Service experiencing issues
- `✗ Unhealthy` - Service is down
- `- Unknown` - Cannot determine status

### Platform States
- `Running` - All services operational
- `Degraded` - Some services have issues
- `Stopped` - Platform is not running
- `Starting` - Platform is starting up

## Examples

### Quick Health Check
```bash
$ minifly status
🚀 Minifly Platform Status
─────────────────────────
Platform: Running ✓
All services healthy
```

### Detailed Service Check
```bash
$ minifly status --service docker
Docker Service Status:
  Status:       Running
  Version:      24.0.5
  API Version:  1.43
  
  Resources:
    Containers: 12 running, 3 stopped, 15 total
    Images:     25 (2.3GB)
    Volumes:    8 (156MB)
    Networks:   5
    
  System:
    OS:         Darwin 23.5.0
    Kernel:     6.9.3
    CPUs:       10
    Memory:     32GB
```

### Monitoring Script
```bash
#!/bin/bash
# Check if platform is healthy
if minifly status --json | jq -e '.platform.status == "running"' > /dev/null; then
  echo "Platform healthy"
else
  echo "Platform issues detected"
  exit 1
fi
```

### Watch During Deployment
```bash
# In terminal 1
minifly deploy

# In terminal 2
minifly status --watch
```

## Troubleshooting

### Platform Not Running
```bash
$ minifly status
✗ Minifly platform is not running

Start with: minifly serve
```

### Service Issues
```bash
$ minifly status
Platform: Degraded ⚠

Services:
  ✓ API Server    Healthy
  ✗ Docker        Unhealthy   Error: Cannot connect to Docker daemon
  ✓ LiteFS        Healthy

Run 'minifly serve' to restart services
```

### Connection Errors
```bash
# Check if API is accessible
curl http://localhost:4280/health

# Check with debug logging
MINIFLY_DEBUG=1 minifly status
```

## Integration

### CI/CD Pipelines
```bash
# Wait for platform to be ready
while ! minifly status --json | jq -e '.platform.status == "running"' > /dev/null; do
  echo "Waiting for platform..."
  sleep 2
done
```

### Monitoring Systems
```bash
# Export metrics
minifly status --json | \
  jq '.resources | to_entries | .[] | "\(.key)=\(.value)"'
```

## See Also

- [serve](./serve) - Start the platform
- [logs](./logs) - View service logs
- [machines](./machines) - List machine statuses