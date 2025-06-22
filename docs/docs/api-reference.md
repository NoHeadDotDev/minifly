# API Reference

The Minifly API implements the complete Fly.io Machines API v1, providing full compatibility for local development.

## Base URL

```
http://localhost:4280
```

## Authentication

Use the `Authorization` header with a Bearer token:

```bash
Authorization: Bearer <your-token>
```

## Endpoints

### Applications

#### List Applications
```http
GET /v1/apps
```

#### Create Application
```http
POST /v1/apps
Content-Type: application/json

{
  "app_name": "my-app",
  "org_slug": "personal"
}
```

#### Get Application
```http
GET /v1/apps/{app_name}
```

#### Delete Application
```http
DELETE /v1/apps/{app_name}
```

### Machines

#### List Machines
```http
GET /v1/apps/{app_name}/machines
```

#### Create Machine
```http
POST /v1/apps/{app_name}/machines
Content-Type: application/json

{
  "name": "my-machine",
  "region": "sjc",
  "config": {
    "image": "nginx:latest",
    "size": "shared-cpu-1x",
    "env": {
      "PORT": "8080"
    },
    "services": [
      {
        "ports": [
          {
            "port": 80,
            "handlers": ["http"]
          }
        ],
        "protocol": "tcp",
        "internal_port": 8080
      }
    ]
  }
}
```

#### Get Machine
```http
GET /v1/apps/{app_name}/machines/{machine_id}
```

#### Update Machine
```http
POST /v1/apps/{app_name}/machines/{machine_id}
Content-Type: application/json

{
  "config": {
    "image": "nginx:1.21"
  }
}
```

#### Start Machine
```http
POST /v1/apps/{app_name}/machines/{machine_id}/start
```

#### Stop Machine
```http
POST /v1/apps/{app_name}/machines/{machine_id}/stop
```

#### Restart Machine
```http
POST /v1/apps/{app_name}/machines/{machine_id}/restart
```

#### Delete Machine
```http
DELETE /v1/apps/{app_name}/machines/{machine_id}
```

### Volumes

#### List Volumes
```http
GET /v1/apps/{app_name}/volumes
```

#### Create Volume
```http
POST /v1/apps/{app_name}/volumes
Content-Type: application/json

{
  "name": "data",
  "size_gb": 1,
  "region": "sjc"
}
```

#### Get Volume
```http
GET /v1/apps/{app_name}/volumes/{volume_id}
```

#### Delete Volume
```http
DELETE /v1/apps/{app_name}/volumes/{volume_id}
```

### Logs

#### Stream Machine Logs
```http
GET /v1/apps/{app_name}/machines/{machine_id}/logs?follow=true
```

Server-Sent Events stream with log entries.

### Health

#### Platform Health
```http
GET /health
```

Returns overall platform health status.

## Response Formats

### Success Response
```json
{
  "id": "d891234567890",
  "name": "my-machine",
  "state": "started",
  "region": "sjc",
  "created_at": "2024-06-22T10:00:00Z",
  "updated_at": "2024-06-22T10:05:00Z",
  "config": {
    "image": "nginx:latest",
    "size": "shared-cpu-1x"
  }
}
```

### Error Response
```json
{
  "error": "Machine not found",
  "status": 404
}
```

## Machine States

- `created` - Machine is created but not started
- `starting` - Machine is starting up
- `started` - Machine is running
- `stopping` - Machine is shutting down
- `stopped` - Machine is stopped
- `failed` - Machine failed to start or crashed

## Rate Limiting

Local development has no rate limiting, but be mindful of resource usage.

## WebSocket Support

Machine exec and console access via WebSocket:

```
ws://localhost:4280/v1/apps/{app_name}/machines/{machine_id}/exec
```

## Complete API Compatibility

Minifly implements 100% of the Fly.io Machines API, ensuring your local development experience matches production exactly.