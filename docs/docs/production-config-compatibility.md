# Production Config Compatibility

Minifly now supports using production Fly.io configurations without modifications! This means you can develop locally with the exact same `fly.toml`, `Dockerfile`, and `litefs.yml` files that you deploy to production.

## Key Features

### 🔧 Environment Variable Translation

Minifly automatically injects Fly.io-specific environment variables to ensure your application works the same locally as in production:

```bash
FLY_APP_NAME=myapp
FLY_MACHINE_ID=d14d6f29e417981
FLY_REGION=local
FLY_PUBLIC_IP=127.0.0.1
FLY_PRIVATE_IP=172.19.0.2
FLY_CONSUL_URL=http://localhost:8500
PRIMARY_REGION=local
```

For applications using Tigris/S3, endpoints are automatically redirected to local MinIO:
```bash
TIGRIS_ENDPOINT=http://localhost:9000
AWS_ENDPOINT_URL=http://localhost:9000
AWS_ENDPOINT_URL_S3=http://localhost:9000
```

### 🔐 Secrets Management

Manage application secrets without committing them to git:

```bash
# Set secrets for your app
minifly secrets set DATABASE_URL=postgres://localhost/myapp
minifly secrets set SECRET_KEY=your-secret-key
minifly secrets set API_TOKEN=sk-1234567890

# List current secrets (values are hidden)
minifly secrets list

# Remove secrets
minifly secrets remove API_TOKEN
```

#### Secrets Files

Secrets are stored in `.fly.secrets` files:

```bash
# Default secrets for all apps
.fly.secrets

# App-specific secrets (takes precedence)
.fly.secrets.<app-name>
```

File format:
```bash
# Comments start with #
DATABASE_URL=postgres://localhost/myapp
SECRET_KEY=your-secret-key-here
API_TOKEN=sk-1234567890
```

### 📁 Volume Mapping

Fly.io volumes are automatically mapped to local directories:

```toml
# In fly.toml
[mounts]
source = "myapp_data" 
destination = "/data"
```

This creates a local directory at:
```
./minifly-data/<app>/<machine>/volumes/myapp_data/
```

All volume operations work exactly like in production!

### 🗄️ LiteFS Configuration Compatibility

Production `litefs.yml` files are automatically adapted for local development:

**Production litefs.yml:**
```yaml
lease:
  type: "consul"
  advertise-url: "http://${HOSTNAME}.vm.${FLY_APP_NAME}.internal:20202"
  candidate: ${FLY_LITEFS_PRIMARY}

consul:
  url: "http://${FLY_CONSUL_URL}"
  key: "litefs/${FLY_APP_NAME}/primary"
```

**Automatically becomes:**
```yaml
lease:
  type: "static"  # Consul → Static for local dev
  advertise-url: "http://machine-id:20202"
  candidate: true

# Consul config removed for local development
```

### 🐳 Dockerfile Compatibility

Dockerfiles with Fly.io build arguments work automatically:

```dockerfile
# Production Dockerfile
ARG FLY_APP_NAME
ARG FLY_REGION  
ARG FLY_BUILD_ID

RUN echo "Building ${FLY_APP_NAME} in ${FLY_REGION}"
```

Minifly automatically injects these build arguments during local builds.

### 🌐 Service Discovery

Applications can resolve other services using `.internal` domains just like in production:

```rust
// Resolve all machines for an app
let ips = resolve("myapp.internal").await;

// Resolve specific machine
let ips = resolve("machine-id.vm.myapp.internal").await;
```

Supported domain formats:
- `<app>.internal` - All machine IPs for the app
- `<machine-id>.vm.<app>.internal` - Specific machine IP
- `fly-local-6pn.internal` - Local Docker DNS server

### ✅ Fly.toml Validation

Minifly validates your production configuration and shows helpful warnings:

```bash
📖 Reading fly.toml...
🚀 Deploying app myapp...

⚠️  Compatibility warnings found:
   • auto_stop_machines is simulated with container pause/unpause
   • Experimental features may not be fully supported in local development
   • Primary region is ignored - all machines run in 'local' region
```

## Usage Examples

### Basic Deployment

```bash
# Deploy with production configs - no changes needed!
minifly deploy

# Deploy with watch mode for development
minifly deploy --watch
```

### Multi-App Development

```bash
# Set app-specific secrets
minifly secrets set DATABASE_URL=postgres://localhost/frontend_db
minifly secrets set --app backend API_KEY=backend-secret

# Deploy both apps
cd frontend && minifly deploy &
cd backend && minifly deploy &
```

### Database Applications

```bash
# Production fly.toml with LiteFS works directly
minifly deploy

# Database files appear locally at:
# ./minifly-data/myapp/machine-id/litefs/data/
```

## Supported Fly.io Features

| Feature | Support Level | Notes |
|---------|---------------|--------|
| Environment Variables | ✅ Full | All FLY_* variables injected |
| Secrets | ✅ Full | Local .fly.secrets files |
| Volumes | ✅ Full | Mapped to local directories |
| LiteFS | ✅ Full | Production configs adapted |
| Service Discovery | ✅ Full | .internal domains work |
| Docker Builds | ✅ Full | Build arguments injected |
| Multiple Services | ✅ Full | Each service gets own container |
| Auto Stop/Start | ⚠️ Simulated | Container pause/unpause |
| Metrics | ⚠️ Limited | Endpoints not auto-configured |
| Multi-process Apps | ⚠️ Simulated | Separate containers per process |
| Primary Regions | ➖ Ignored | All machines run in 'local' |
| Experimental Features | ⚠️ Varies | May not be fully supported |

## Migration Guide

### From Development-Specific Configs

If you currently maintain separate development configurations:

1. **Remove development fly.toml** - Use your production config directly
2. **Move secrets to .fly.secrets** - Remove hardcoded development secrets
3. **Update paths** - Volumes now map to `./minifly-data/` instead of custom paths
4. **Test thoroughly** - Verify all features work with production configs

### Environment Variable Changes

Replace development-specific environment variables:

```bash
# Old development approach ❌
export DATABASE_URL=sqlite:./dev.db
export FLY_APP_NAME=myapp-dev

# New approach ✅
# No exports needed - Minifly handles automatically
minifly secrets set DATABASE_URL=sqlite:///data/production.db
```

## Best Practices

### 1. Secrets Management

```bash
# ✅ Do: Use secrets for sensitive data
minifly secrets set API_KEY=secret-value

# ❌ Don't: Hardcode in fly.toml
[env]
API_KEY = "secret-value"  # Never do this!
```

### 2. Volume Organization

```bash
# ✅ Do: Use descriptive volume names
[mounts]
source = "user_uploads"
destination = "/app/uploads"

# ❌ Don't: Use generic names
[mounts]  
source = "data"
destination = "/data"
```

### 3. Service Discovery

```rust
// ✅ Do: Use .internal domains for service communication
let api_url = "http://api.internal:8080";

// ❌ Don't: Hardcode localhost
let api_url = "http://localhost:8080";
```

### 4. Development vs Production

```bash
# ✅ Do: Use same configs for dev and prod
minifly deploy  # Uses production fly.toml

# ❌ Don't: Maintain separate configs
minifly deploy -f fly.dev.toml  # No longer needed!
```

## Troubleshooting

### Secrets Not Loading

```bash
# Check if secrets exist
minifly secrets list

# Verify file format
cat .fly.secrets
# Should be KEY=VALUE format, no quotes needed
```

### Volume Mount Issues

```bash
# Check volume directory exists
ls -la ./minifly-data/<app>/<machine>/volumes/

# Verify permissions
sudo chown -R $USER ./minifly-data/
```

### Service Discovery Not Working

```bash
# Check DNS registrations (requires debug access to Minifly API)
curl http://localhost:4280/debug/dns

# Verify containers are running
minifly machines list <app>
```

### LiteFS Issues

```bash
# Check LiteFS adaptation
cat ./minifly-data/<app>/<machine>/litefs/config.yml

# Verify database path
minifly logs <machine-id> | grep litefs
```

## Advanced Configuration

### Custom DNS Resolution

For complex service topologies, you can customize DNS behavior:

```rust
// In your application
let resolver = DnsResolver::new();
resolver.add_custom_mapping("legacy-api.internal", "192.168.1.100");
```

### Volume Sync

Sync volumes between multiple developers:

```bash
# Export volume data
tar -czf myapp-data.tar.gz ./minifly-data/myapp/

# Import on another machine
tar -xzf myapp-data.tar.gz
```

### Production Secrets Integration

Sync secrets from production (be careful with sensitive data):

```bash
# Export from Fly.io (requires flyctl)
fly secrets list --app production-app

# Import to Minifly
minifly secrets set DATABASE_URL=prod-compatible-url
minifly secrets set API_KEY=development-key  # Use dev-safe values
```

This production config compatibility makes Minifly a true drop-in replacement for Fly.io development, eliminating the friction between development and production environments.