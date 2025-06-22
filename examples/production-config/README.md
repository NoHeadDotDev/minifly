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

## Setup

1. Create a `.fly.secrets` file with your secrets:
```bash
DATABASE_URL=postgres://user:pass@localhost/myapp
SECRET_KEY=your-secret-key
API_TOKEN=your-api-token
```

2. Deploy the application:
```bash
minifly deploy
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

The example app includes endpoints to test various features:
- `/` - Basic health check showing environment variables
- `/secrets` - Display loaded secrets (redacted)
- `/volumes` - Test volume persistence
- `/discover` - Test .internal DNS resolution
- `/database` - Test LiteFS database operations