---
sidebar_position: 5
---

# Production Config Example

This example demonstrates how to use production Fly.io configurations with Minifly, including complex service definitions, health checks, and environment management.

## Overview

Learn how to:
- Use advanced fly.toml configurations
- Handle production environment variables
- Configure health checks and monitoring
- Manage secrets and sensitive data
- Set up multi-service applications

## Key Configuration Features

### Complex fly.toml

```toml
app = "production-app"
primary_region = "iad"
kill_signal = "SIGTERM"
kill_timeout = 5

[build]
  dockerfile = "Dockerfile"
  args = {
    BUILDKIT_INLINE_CACHE = "1"
  }

[env]
  NODE_ENV = "production"
  LOG_LEVEL = "info"
  PORT = "8080"

[experimental]
  auto_rollback = true

[[services]]
  internal_port = 8080
  protocol = "tcp"
  auto_stop_machines = true
  auto_start_machines = true
  
  [services.concurrency]
    type = "connections"
    hard_limit = 25
    soft_limit = 20
    
  [[services.ports]]
    port = 80
    handlers = ["http"]
    force_https = true
    
  [[services.ports]]
    port = 443
    handlers = ["tls", "http"]
    
  [[services.tcp_checks]]
    grace_period = "10s"
    interval = "15s"
    restart_limit = 0
    timeout = "2s"
    
  [[services.http_checks]]
    interval = "10s"
    grace_period = "5s"
    method = "get"
    path = "/health"
    protocol = "http"
    restart_limit = 0
    timeout = "2s"
    tls_skip_verify = false
    
    [services.http_checks.headers]
      X-Health-Check = "minifly"

[mounts]
  source = "app_data"
  destination = "/data"
  
[metrics]
  path = "/metrics"
  port = 9091
```

### Environment Management

Minifly translates Fly.io environment variables:

```bash
# Fly.io variables automatically set by Minifly
FLY_APP_NAME=production-app
FLY_MACHINE_ID=abc123
FLY_REGION=local
FLY_PUBLIC_IP=127.0.0.1
FLY_PRIVATE_IP=172.19.0.2
```

### Secrets Handling

Create `.fly.secrets` file:

```env
DATABASE_URL=postgres://user:pass@db:5432/myapp
REDIS_URL=redis://redis:6379
API_KEY=secret-api-key
SESSION_SECRET=very-secret-session-key
```

Load secrets during deployment:

```bash
# Secrets are automatically loaded from .fly.secrets
minifly deploy
```

## Advanced Features

### Health Checks

Minifly supports both TCP and HTTP health checks:

```go
// Health check endpoint
func healthHandler(w http.ResponseWriter, r *http.Request) {
    // Check dependencies
    if !isDatabaseHealthy() {
        w.WriteHeader(http.StatusServiceUnavailable)
        json.NewEncoder(w).Encode(map[string]string{
            "status": "unhealthy",
            "reason": "database connection failed",
        })
        return
    }
    
    w.WriteHeader(http.StatusOK)
    json.NewEncoder(w).Encode(map[string]string{
        "status": "healthy",
        "version": os.Getenv("APP_VERSION"),
    })
}
```

### Multi-Process Applications

```toml
[processes]
  web = "node server.js"
  worker = "node worker.js"
  
[[services]]
  processes = ["web"]
  internal_port = 8080
  
[[services]]
  processes = ["worker"]
  internal_port = 9090
```

### Resource Limits

```toml
[[services]]
  [services.resources]
    cpu_kind = "shared"
    cpus = 1
    memory_mb = 512
```

### Auto-scaling Configuration

```toml
[[services]]
  auto_stop_machines = true
  auto_start_machines = true
  min_machines_running = 1
  
  [services.concurrency]
    type = "requests"
    hard_limit = 1000
    soft_limit = 800
```

## Deployment Strategies

### Blue-Green Deployment

```toml
[deploy]
  strategy = "bluegreen"
  max_unavailable = 0.33
  wait_timeout = "5m"
```

### Rolling Deployment

```toml
[deploy]
  strategy = "rolling"
  max_unavailable = 1
```

### Canary Deployment

```toml
[deploy]
  strategy = "canary"
  canary_increment = 33
  wait_timeout = "30s"
```

## Monitoring Integration

### Prometheus Metrics

```toml
[metrics]
  path = "/metrics"
  port = 9091
```

### Custom Metrics Endpoint

```go
func metricsHandler(w http.ResponseWriter, r *http.Request) {
    fmt.Fprintf(w, "# HELP app_requests_total Total requests\n")
    fmt.Fprintf(w, "# TYPE app_requests_total counter\n")
    fmt.Fprintf(w, "app_requests_total{method=\"GET\"} %d\n", getRequests)
    fmt.Fprintf(w, "app_requests_total{method=\"POST\"} %d\n", postRequests)
}
```

## Production Best Practices

### 1. Graceful Shutdown

```go
func main() {
    srv := &http.Server{Addr: ":8080"}
    
    go func() {
        if err := srv.ListenAndServe(); err != http.ErrServerClosed {
            log.Fatalf("ListenAndServe(): %v", err)
        }
    }()
    
    // Wait for interrupt signal
    quit := make(chan os.Signal, 1)
    signal.Notify(quit, os.Interrupt, syscall.SIGTERM)
    <-quit
    
    // Graceful shutdown with timeout
    ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
    defer cancel()
    
    if err := srv.Shutdown(ctx); err != nil {
        log.Fatal("Server forced to shutdown:", err)
    }
}
```

### 2. Health Check Implementation

```javascript
// Comprehensive health check
app.get('/health', async (req, res) => {
  const checks = {
    database: await checkDatabase(),
    redis: await checkRedis(),
    disk: await checkDiskSpace(),
    memory: process.memoryUsage(),
  };
  
  const healthy = Object.values(checks).every(check => check.healthy);
  
  res.status(healthy ? 200 : 503).json({
    status: healthy ? 'healthy' : 'unhealthy',
    checks,
    timestamp: new Date().toISOString(),
  });
});
```

### 3. Structured Logging

```javascript
const winston = require('winston');

const logger = winston.createLogger({
  level: process.env.LOG_LEVEL || 'info',
  format: winston.format.json(),
  transports: [
    new winston.transports.Console({
      format: winston.format.combine(
        winston.format.timestamp(),
        winston.format.errors({ stack: true }),
        winston.format.json()
      ),
    }),
  ],
});
```

## Migration from Fly.io

### Compatibility Notes

✅ **Fully Supported:**
- Basic service configuration
- Environment variables
- Health checks
- Volume mounts
- Dockerfile builds

⚠️ **Simulated/Partial:**
- Auto-scaling (simulated with pause/unpause)
- Regions (all run as "local")
- Private networking (uses Docker networking)

❌ **Not Supported:**
- Fly Postgres/Redis addons
- Anycast IPs
- Certificate management
- Global load balancing

### Migration Checklist

1. **Update connection strings** to use local services
2. **Replace Fly addons** with local Docker containers
3. **Adjust health check endpoints** if needed
4. **Update secrets management** to use .fly.secrets
5. **Test locally** with `minifly deploy`

## Troubleshooting

### Service won't start
- Check health check configuration
- Verify all required environment variables
- Review logs: `minifly logs <machine-id>`

### Secrets not loading
- Ensure `.fly.secrets` exists
- Check file permissions
- Verify secret names match env vars

### Performance issues
- Adjust resource limits
- Check concurrency settings
- Monitor with `docker stats`

## Summary

This example shows how Minifly can handle production-grade configurations, making it an excellent tool for:
- Local development of Fly.io applications
- Testing complex deployments
- Learning production best practices
- Migrating applications to/from Fly.io