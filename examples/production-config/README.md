# Production Config Compatibility Example

This example demonstrates how to use production Fly.io configurations with Minifly without modifications.

## Features Demonstrated

- Environment variable translation (FLY_* variables)
- Secrets management (.fly.secrets)
- Volume mapping
- LiteFS production configuration
- Dockerfile with Fly.io build args
- Service discovery (.internal DNS)

## Project Structure

```
production-config/
├── fly.toml          # Production Fly.io configuration
├── Dockerfile        # Production Dockerfile with Fly.io features
├── litefs.yml        # Production LiteFS configuration
├── .fly.secrets      # Local secrets (not committed to git)
├── src/
│   └── main.rs       # Application code using Fly.io features
└── README.md         # This file
```

## Quick Start

The easiest way to run this example:

```bash
cd examples/production-config

# Set up secrets for the demo
minifly secrets set DATABASE_URL=sqlite:///data/production.db
minifly secrets set DATABASE_PATH=/data
minifly secrets set SECRET_KEY=dev-secret-key-12345
minifly secrets set API_TOKEN=dev-api-token-67890

# Start with auto-deployment
minifly serve --dev
```

This will automatically:
- ✅ Start the Minifly platform
- ✅ Detect and deploy the project
- ✅ Inject Fly.io environment variables
- ✅ Load secrets from `.fly.secrets.production-app`
- ✅ Adapt production `litefs.yml` for local development
- ✅ Enable `.internal` DNS resolution
- ✅ Enable file watching and hot reloading
- ✅ Show you the URL with the assigned port

## Manual Setup (Advanced)

For advanced control:

1. Create a `.fly.secrets` file with your secrets:
```bash
DATABASE_URL=postgres://user:pass@localhost/myapp
SECRET_KEY=your-secret-key
API_TOKEN=your-api-token
```

2. Start platform and deploy manually:
```bash
minifly serve  # Terminal 1
minifly deploy # Terminal 2
```

## Key Features

### Environment Variables

The application automatically receives Fly.io environment variables:
- `FLY_APP_NAME` - Your app name
- `FLY_MACHINE_ID` - Unique machine identifier
- `FLY_REGION` - Region (always "local" in development)
- `FLY_PUBLIC_IP` - Public IP (127.0.0.1 in development)
- `FLY_PRIVATE_IP` - Private IP for internal communication

### Secrets Management

Secrets are loaded from `.fly.secrets` files:
- `.fly.secrets` - Default secrets for all apps
- `.fly.secrets.<app-name>` - App-specific secrets (takes precedence)

Manage secrets with CLI:
```bash
minifly secrets set DATABASE_URL=postgres://localhost/myapp
minifly secrets list
minifly secrets remove API_TOKEN
```

### Volume Mapping

Volumes defined in fly.toml are automatically mapped to local directories:
```toml
[mounts]
source = "myapp_data"
destination = "/data"
```

This creates a local directory at `./minifly-data/<app>/<machine>/volumes/myapp_data/`

### Service Discovery

Applications can resolve other services using .internal domains:
- `myapp.internal` - Resolves to all machines in the app
- `machine-id.vm.myapp.internal` - Resolves to specific machine

### LiteFS Compatibility

Production litefs.yml files are automatically adapted:
- Consul lease → Static lease for local development
- Paths adjusted to local directories
- Debug mode enabled
- Primary node configuration handled automatically

## Running the Example

```bash
# Deploy the application
cd examples/production-config
minifly deploy

# Check status
minifly status

# View logs
minifly machines list production-app
minifly logs <machine-id>

# Test service discovery
curl http://localhost:8080/health
curl http://localhost:8080/discover
```

## Testing Features

The example app includes endpoints to test various production config compatibility features:

### Core Endpoints
- `GET /` or `/health` - Basic health check showing all Fly.io environment variables
- `GET /secrets` - Display loaded secrets (values redacted for security)
- `GET /volumes` - Test volume mounting and persistence
- `GET /discover` - Show .internal DNS domain information
- `GET /test-dns` - Test .internal DNS resolution capabilities
- `GET /database` - Test LiteFS database connection and operations
- `POST /database/records?name=test` - Create a database record

### Example Tests

```bash
# Test environment variable injection
curl http://localhost:8080/health | jq

# Test secrets loading
curl http://localhost:8080/secrets | jq

# Test volume mounting
curl http://localhost:8080/volumes | jq

# Test service discovery
curl http://localhost:8080/discover | jq
curl http://localhost:8080/test-dns | jq

# Test database operations (if LiteFS is configured)
curl http://localhost:8080/database | jq
curl -X POST "http://localhost:8080/database/records?name=MyRecord" | jq
```

### Expected Behavior

With the new production config compatibility features, you should see:

1. **Environment Variables**: All standard Fly.io variables (FLY_APP_NAME, FLY_MACHINE_ID, etc.)
2. **Secrets**: Database credentials, API keys, and other secrets from `.fly.secrets.production-app`
3. **Volumes**: Persistent storage mounted at `/data` and mapped to local directories
4. **DNS**: Internal service discovery domains properly configured
5. **LiteFS**: Production database configuration adapted for local development