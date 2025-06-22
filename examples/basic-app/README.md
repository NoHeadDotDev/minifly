# Basic Minifly Example

This example demonstrates how to deploy a simple nginx application using Minifly.

## Prerequisites

- Minifly API server running (`minifly-api`)
- Docker installed and running

## Deployment Steps

1. Start the Minifly API server:
   ```bash
   minifly-api
   ```

2. Initialize Minifly CLI:
   ```bash
   minifly init
   ```

3. Create the app:
   ```bash
   minifly apps create example-app
   ```

4. Create a machine:
   ```bash
   minifly machines create --app example-app --image nginx:alpine
   ```

5. List machines:
   ```bash
   minifly machines list --app example-app
   ```

6. Access your application:
   - The nginx server will be running on the port mapped by Docker
   - Use `docker ps` to find the mapped port

## Using the API directly

You can also use the API directly with curl:

```bash
# Create a machine
curl -X POST http://localhost:4280/v1/apps/example-app/machines \
  -H "Authorization: Bearer test-token" \
  -H "Content-Type: application/json" \
  -d '{
    "config": {
      "image": "nginx:alpine",
      "guest": {
        "cpu_kind": "shared",
        "cpus": 1,
        "memory_mb": 256
      },
      "services": [{
        "ports": [{
          "port": 80,
          "handlers": ["http"]
        }],
        "protocol": "tcp",
        "internal_port": 80
      }]
    }
  }'
```