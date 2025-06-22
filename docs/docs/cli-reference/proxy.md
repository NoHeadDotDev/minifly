# minifly proxy

Create a proxy connection to a machine's services.

## Synopsis

```bash
minifly proxy <MACHINE_ID> [OPTIONS]
```

## Description

The `proxy` command establishes a local proxy to services running in a machine, allowing you to:
- Access internal services without exposing ports
- Debug applications locally
- Connect database clients to containerized databases
- Test internal APIs
- Access admin interfaces safely

## Arguments

- `<MACHINE_ID>` - Machine ID to proxy to

## Options

- `--port <LOCAL:REMOTE>` - Port mapping (default: 8080:8080)
- `--bind <ADDRESS>` - Local bind address (default: 127.0.0.1)
- `--app <APP>` - Application name (for machine lookup)
- `-h, --help` - Print help information

## Port Mapping

### Default Mapping
```bash
# Proxy local port 8080 to machine's port 8080
minifly proxy d891234567890
```

### Custom Mapping
```bash
# Local port 3000 to machine's port 8080
minifly proxy d891234567890 --port 3000:8080

# Multiple ports (coming soon)
minifly proxy d891234567890 --port 3000:8080 --port 5432:5432
```

## Examples

### Web Application
```bash
$ minifly proxy d891234567890
✓ Proxying localhost:8080 -> d891234567890:8080
Press Ctrl+C to stop

# In browser: http://localhost:8080
```

### Database Connection
```bash
# Proxy PostgreSQL
$ minifly proxy db-machine-id --port 5432:5432
✓ Proxying localhost:5432 -> db-machine-id:5432

# Connect with psql
$ psql -h localhost -p 5432 -U postgres mydb
```

### Custom Local Port
```bash
# Use local port 3000
$ minifly proxy d891234567890 --port 3000:8080
✓ Proxying localhost:3000 -> d891234567890:8080
```

### Bind to All Interfaces
```bash
# Allow external connections (careful!)
$ minifly proxy d891234567890 --bind 0.0.0.0
✓ Proxying 0.0.0.0:8080 -> d891234567890:8080
⚠️  Warning: Proxy accessible from all network interfaces
```

## Use Cases

### Development Debugging
```bash
# Access internal debug endpoints
minifly proxy web-machine --port 9229:9229
# Now connect debugger to localhost:9229
```

### Database Management
```bash
# PostgreSQL
minifly proxy postgres-machine --port 5432:5432
pgadmin4 # Connect to localhost:5432

# MySQL
minifly proxy mysql-machine --port 3306:3306
mysql -h 127.0.0.1 -P 3306 -u root -p

# Redis
minifly proxy redis-machine --port 6379:6379
redis-cli -h localhost -p 6379
```

### API Testing
```bash
# Internal API access
minifly proxy api-machine --port 8000:3000

# Test with curl
curl http://localhost:8000/internal/health
```

### Admin Interfaces
```bash
# RabbitMQ Management
minifly proxy rabbitmq-machine --port 15672:15672
# Visit http://localhost:15672

# Elasticsearch
minifly proxy elastic-machine --port 9200:9200
curl http://localhost:9200/_cluster/health
```

## Machine Discovery

Find machines by name:
```bash
# List machines first
minifly machines list --app my-app

# Proxy by name (coming soon)
minifly proxy --app my-app --name web-1
```

## Security Considerations

1. **Local Only by Default** - Binds to 127.0.0.1
2. **No Authentication** - Proxy doesn't add auth
3. **Temporary** - Only while command runs
4. **Direct Connection** - No encryption added

### Secure Usage
```bash
# Keep local only
minifly proxy d891234567890 --bind 127.0.0.1

# Use SSH tunnel for remote access
ssh -L 8080:localhost:8080 user@jumphost
minifly proxy d891234567890
```

## Connection Details

The proxy creates a TCP tunnel:
```
Your Computer          Minifly Proxy          Docker Container
localhost:8080   <-->   TCP Tunnel    <-->   container:8080
```

## Multiple Proxies

Run multiple proxy commands in different terminals:
```bash
# Terminal 1: Web app
minifly proxy web-machine --port 3000:3000

# Terminal 2: Database
minifly proxy db-machine --port 5432:5432

# Terminal 3: Cache
minifly proxy cache-machine --port 6379:6379
```

## Troubleshooting

### Connection Refused
```bash
# Check machine is running
minifly machines show d891234567890

# Check service is listening
minifly logs d891234567890 --tail 50

# Try different port
minifly proxy d891234567890 --port 8081:80
```

### Port Already in Use
```bash
# Check what's using the port
lsof -i :8080

# Use different local port
minifly proxy d891234567890 --port 8081:8080
```

### Proxy Drops Connection
```bash
# Check machine health
minifly machines show d891234567890

# View logs
minifly logs d891234567890 --follow

# Restart machine if needed
minifly machines restart d891234567890
```

## Advanced Usage

### Proxy Chain (coming soon)
```bash
# Proxy through multiple machines
minifly proxy web-machine --via gateway-machine
```

### Service Discovery (coming soon)
```bash
# Proxy to service by name
minifly proxy --service postgres.my-app.internal
```

### Load Balancing (coming soon)
```bash
# Proxy to multiple backends
minifly proxy --app my-app --service web --balance
```

## Integration

### Docker Compose Override
```yaml
# docker-compose.override.yml
version: '3'
services:
  db:
    ports:
      - "5432:5432"  # Equivalent to proxy
```

### Development Scripts
```bash
#!/bin/bash
# dev.sh - Start development environment

# Start platform
minifly serve &

# Wait for startup
sleep 5

# Create proxies
minifly proxy web-machine --port 3000:3000 &
minifly proxy db-machine --port 5432:5432 &

# Wait for user
echo "Development environment ready!"
echo "Web: http://localhost:3000"
echo "DB: postgresql://localhost:5432"
wait
```

## See Also

- [machines](./machines) - List available machines
- [logs](./logs) - Debug connection issues
