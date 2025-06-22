# Multi-Tenant Application Example

This example demonstrates how to build a multi-tenant Rust application using Axum and Askama that integrates with Minifly for local development. Each tenant has their own isolated SQLite database managed by LiteFS.

## Architecture Overview

### Key Features

1. **Database Per Tenant**: Each tenant gets their own SQLite database file (`/litefs/{tenant}.db`)
2. **Automatic Database Creation**: Databases are created on-demand when a tenant is first accessed
3. **LiteFS Integration**: All databases are managed by LiteFS for automatic replication
4. **Tenant Isolation**: Complete data isolation between tenants
5. **Multiple Tenant Detection Methods**:
   - HTTP Header (`X-Tenant`)
   - Subdomain (e.g., `tenant1.example.com`)
   - URL Path (e.g., `/tenant/tenant1/...`)

### Technology Stack

- **Axum**: Modern web framework for Rust
- **Askama**: Type-safe templating engine
- **SQLx**: Async SQL toolkit with compile-time checked queries
- **LiteFS**: Distributed SQLite replication
- **Minifly**: Local Fly.io development environment

## Running Locally with Minifly

### Prerequisites

1. Minifly API server running
2. Docker installed
3. LiteFS binary available (built from source or downloaded)

### Quick Start

1. **Start Minifly API server** (in a separate terminal):
   ```bash
   MINIFLY_DATABASE_URL="sqlite::memory:" cargo run --bin minifly-api
   ```

2. **Deploy the application using Minifly**:
   ```bash
   cd examples/multi-tenant-app
   
   # Build and deploy (once deploy command is implemented)
   minifly deploy
   
   # Or manually:
   # Create the app
   minifly apps create multi-tenant-demo
   
   # Build Docker image
   docker build -t multi-tenant-app .
   
   # Create a machine
   curl -X POST http://localhost:4280/v1/apps/multi-tenant-demo/machines \
     -H "Content-Type: application/json" \
     -d @- << EOF
   {
     "config": {
       "image": "multi-tenant-app:latest",
       "guest": {
         "cpu_kind": "shared",
         "cpus": 1,
         "memory_mb": 512
       },
       "env": {
         "DATABASE_PATH": "/litefs",
         "RUST_LOG": "info,multi_tenant_app=debug",
         "FLY_LITEFS_PRIMARY": "true"
       },
       "mounts": [{
         "volume": "tenant_data",
         "path": "/litefs"
       }],
       "services": [{
         "ports": [{
           "port": 80,
           "handlers": ["http"]
         }],
         "protocol": "tcp",
         "internal_port": 8080
       }]
     }
   }
   EOF
   ```

3. **Access the application**:
   ```bash
   # View all tenants
   curl http://localhost:80/
   
   # Create item for a specific tenant
   curl -X POST http://localhost:80/api/items \
     -H "X-Tenant: acme-corp" \
     -H "Content-Type: application/json" \
     -d '{"name": "Project Alpha", "description": "Q1 2024 Initiative"}'
   
   # View tenant dashboard
   curl http://localhost:80/tenant/acme-corp
   ```

### Development Mode

For local development without Docker:

```bash
# Run directly
DATABASE_PATH=./data cargo run

# Access at http://localhost:8080
```

## API Endpoints

### Tenant Management

- `GET /` - List all tenants
- `GET /api/tenants` - List all tenants (JSON)
- `GET /tenant/{tenant}` - Tenant dashboard

### Item Management

- `GET /api/items` - List items for current tenant
- `POST /api/items` - Create new item
- `GET /tenant/{tenant}/items` - List items for specific tenant
- `POST /tenant/{tenant}/items` - Create item for specific tenant

### Health Check

- `GET /health` - Application health status

## Tenant Identification

The application identifies tenants using the following precedence:

1. **X-Tenant Header**: `curl -H "X-Tenant: acme-corp" http://localhost/api/items`
2. **Subdomain**: `http://acme-corp.localhost/api/items`
3. **URL Path**: `http://localhost/tenant/acme-corp/items`
4. **Default**: Falls back to "default" tenant

## Database Schema

Each tenant database contains:

```sql
-- Items table
CREATE TABLE items (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tenant metadata
CREATE TABLE tenant_info (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    item_count INTEGER DEFAULT 0,
    last_activity TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

## Production Deployment

### Deploying to Fly.io

1. **Install Fly CLI**: https://fly.io/docs/getting-started/installing-flyctl/

2. **Create Fly app**:
   ```bash
   fly apps create multi-tenant-app
   ```

3. **Deploy**:
   ```bash
   fly deploy
   ```

4. **Scale for high availability**:
   ```bash
   # Add replicas in different regions
   fly scale count 3 --region dfw,ord,lax
   ```

### LiteFS Configuration

The `litefs.yml` file configures:

- **FUSE Mount**: `/litefs` directory for database files
- **HTTP Proxy**: Port 20202 for LiteFS admin interface
- **Replication**: Automatic sync between primary and replicas
- **Lease Management**: Primary election using Consul (production) or static (development)

## Performance Considerations

### Connection Pooling

Each tenant gets its own SQLite connection pool with:
- Maximum 5 connections per tenant
- Connections are cached and reused
- Automatic connection cleanup on idle

### Scaling Strategies

1. **Horizontal Scaling**: Add more Fly machines in different regions
2. **Read Replicas**: LiteFS automatically creates read replicas
3. **Primary Election**: Automatic failover with LiteFS lease management
4. **Connection Limits**: Monitor SQLite connection usage per tenant

### Best Practices

1. **Tenant Naming**: Use URL-safe characters only (alphanumeric, dash, underscore)
2. **Database Size**: Monitor individual tenant database sizes
3. **Backup Strategy**: Regular exports using LiteFS backup features
4. **Migration Strategy**: Run migrations on first tenant access
5. **Monitoring**: Track per-tenant metrics and usage

## Testing

### Unit Tests

```bash
cargo test
```

### Integration Tests

```bash
# Start Minifly and deploy the app
# Run integration tests
cargo test --features integration
```

### Load Testing

```bash
# Create multiple tenants
for i in {1..10}; do
  curl -X POST http://localhost/api/items \
    -H "X-Tenant: tenant-$i" \
    -H "Content-Type: application/json" \
    -d '{"name": "Load Test Item", "description": "Testing"}'
done
```

## Troubleshooting

### Common Issues

1. **Database locked errors**: Ensure write transactions are short
2. **FUSE mount issues**: Check LiteFS logs and permissions
3. **Replication lag**: Monitor LiteFS replication status
4. **Connection pool exhaustion**: Increase pool size or optimize queries

### Debug Commands

```bash
# View LiteFS status
curl http://localhost:20202/

# Check database files
ls -la /litefs/

# View application logs
docker logs <container-id>
```

## Next Steps

1. **Add Authentication**: Implement tenant-specific authentication
2. **Add Metrics**: Export per-tenant usage metrics
3. **Add Backup**: Implement automated backup strategy
4. **Add Rate Limiting**: Implement per-tenant rate limits
5. **Add Webhooks**: Notify tenants of data changes