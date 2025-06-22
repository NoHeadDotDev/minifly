# minifly deploy

Deploy an application from source code.

## Synopsis

```bash
minifly deploy [OPTIONS]
```

## Description

The `deploy` command builds and deploys your application based on the configuration in `fly.toml`. It handles:
- Docker image building
- Multi-stage deployments
- Rolling updates
- Health check validation
- Automatic rollback on failure

## Options

- `--app <APP>` - Application name (overrides fly.toml)
- `--image <IMAGE>` - Deploy a pre-built image
- `--config <PATH>` - Path to fly.toml (default: ./fly.toml)
- `--dockerfile <PATH>` - Path to Dockerfile (default: ./Dockerfile)
- `--build-arg <KEY=VALUE>` - Build arguments (can be used multiple times)
- `--no-cache` - Build without cache
- `--strategy <STRATEGY>` - Deployment strategy: rolling, immediate, canary
- `--wait <SECONDS>` - Wait for deployment to complete (default: 300)
- `--watch` - Watch for changes and redeploy automatically
- `-h, --help` - Print help information

## Configuration

Deploy reads configuration from `fly.toml`:

```toml
app = "my-app"
primary_region = "sjc"

[build]
  dockerfile = "Dockerfile"
  [build.args]
    NODE_VERSION = "18"

[env]
  PORT = "8080"
  NODE_ENV = "production"

[deploy]
  strategy = "rolling"
  max_unavailable = 1

[[services]]
  internal_port = 8080
  protocol = "tcp"

  [[services.ports]]
    port = 80
    handlers = ["http"]

  [[services.ports]]
    port = 443
    handlers = ["tls", "http"]

[checks]
  [checks.web]
    grace_period = "5s"
    interval = "15s"
    method = "get"
    path = "/health"
    port = 8080
    timeout = "2s"
```

## Deployment Strategies

### Rolling (Default)
Updates machines one at a time:
```bash
minifly deploy --strategy rolling
```

### Immediate
Updates all machines at once:
```bash
minifly deploy --strategy immediate
```

### Canary
Deploys to one machine first, waits for validation:
```bash
minifly deploy --strategy canary
```

## Build Process

### With Dockerfile
```bash
# Uses Dockerfile in current directory
minifly deploy

# Use specific Dockerfile
minifly deploy --dockerfile Dockerfile.prod
```

### With Pre-built Image
```bash
# Deploy existing image
minifly deploy --image myregistry/myapp:v1.2.3
```

### Build Arguments
```bash
minifly deploy \
  --build-arg NODE_VERSION=18 \
  --build-arg BUILD_ENV=production
```

## Watch Mode

Automatically redeploy on file changes:

```bash
minifly deploy --watch
```

Watches for changes in:
- Source code files
- Dockerfile
- fly.toml
- Package files

Ignores:
- .git directory
- node_modules
- Build artifacts
- Log files

## Health Checks

Deployment waits for health checks to pass:

```toml
[checks]
  [checks.api]
    type = "http"
    method = "get"
    path = "/health"
    port = 8080
    interval = "10s"
    timeout = "2s"
    grace_period = "5s"
```

Health check types:
- `http` - HTTP endpoint check
- `tcp` - TCP connection check

## Examples

### Basic Deployment
```bash
$ minifly deploy
==> Building image
    Using Dockerfile: ./Dockerfile
    Building: ................................ done
    Image: registry.fly.io/my-app:deployment-12345
==> Deploying to Minifly
    Strategy: rolling
    Machines: 2
    Updating d891234567890... done
    Updating d891234567891... done
==> Deployment complete
    Visit your app: http://my-app.local.fly.dev
```

### Deploy with Watch Mode
```bash
$ minifly deploy --watch
==> Initial deployment
    ✓ Deployed successfully
==> Watching for changes...
    [10:30:45] Changed: src/main.rs
    [10:30:45] Rebuilding and deploying...
    [10:31:02] ✓ Deployed successfully
```

### Deploy Specific Image
```bash
$ minifly deploy --image nginx:alpine
==> Deploying image
    Image: nginx:alpine
    Strategy: rolling
    ✓ All machines updated
```

### Canary Deployment
```bash
$ minifly deploy --strategy canary
==> Canary deployment
    Updating 1 of 3 machines
    ✓ Machine d891234567890 updated
    ⏸  Waiting for manual validation...
    
? Continue with deployment? › Yes
    ✓ Updating remaining machines
```

## Rollback

If deployment fails, Minifly automatically rolls back:

```bash
$ minifly deploy
==> Deploying to Minifly
    Updating d891234567890... done
    Updating d891234567891... failed
    ⚠️  Health check failed
==> Rolling back
    ✓ Rolled back to previous version
```

## Multi-Region Deployment

Deploy to specific regions:

```toml
# fly.toml
app = "my-app"
primary_region = "sjc"

[[regions]]
  code = "ord"
  
[[regions]]
  code = "lhr"
```

## Secrets and Environment

Set secrets before deployment:

```bash
# Set secrets (coming soon)
minifly secrets set DATABASE_URL=postgres://...

# Deploy with environment
minifly deploy --env NODE_ENV=production
```

## Zero-Downtime Deployments

Ensure zero downtime with:
1. Health checks
2. Rolling strategy
3. Proper graceful shutdown
4. Connection draining

```toml
[deploy]
  strategy = "rolling"
  max_unavailable = 0
  
[services]
  [[services.ports]]
    handlers = ["http"]
    port = 80
    
[shutdown]
  grace_period = "30s"
  signal = "SIGTERM"
```

## Troubleshooting

### Build Failures
```bash
# Clean build
minifly deploy --no-cache

# Verbose output
MINIFLY_DEBUG=1 minifly deploy
```

### Health Check Failures
```bash
# Check logs
minifly logs <machine-id> --tail 100

# Bypass health checks (dangerous!)
minifly deploy --skip-health-checks
```

## See Also

- [apps](./apps) - Manage applications
- [machines](./machines) - Manage individual machines
- [logs](./logs) - View deployment logs
- [status](./status) - Check deployment status