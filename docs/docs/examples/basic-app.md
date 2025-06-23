---
sidebar_position: 2
---

# Basic App Example

The basic app example demonstrates the fundamentals of deploying an application to Minifly. It's the perfect starting point for understanding how Minifly works.

## Overview

This example includes:
- A simple HTTP server that responds with "Hello from Minifly!"
- Basic fly.toml configuration
- Minimal Dockerfile
- Automatic deployment setup

## Project Structure

```
basic-app/
├── fly.toml          # Minifly deployment configuration
├── Dockerfile        # Container definition
├── server.js         # Simple Node.js HTTP server
└── README.md         # Example documentation
```

## Key Files

### fly.toml

```toml
app = "basic-app"
primary_region = "local"

[build]
  dockerfile = "Dockerfile"

[[services]]
  internal_port = 8080
  protocol = "tcp"
  
  [[services.ports]]
    port = 80
    handlers = ["http"]
```

This configuration:
- Names the app "basic-app"
- Uses a local Dockerfile for building
- Exposes port 8080 internally, mapped to port 80 externally

### Dockerfile

```dockerfile
FROM node:18-alpine

WORKDIR /app

COPY server.js .

EXPOSE 8080

CMD ["node", "server.js"]
```

A minimal Dockerfile that:
- Uses Node.js Alpine Linux image
- Copies the server file
- Exposes port 8080
- Runs the server

## Deployment

### Quick Deploy

From the example directory:

```bash
cd examples/basic-app
minifly deploy
```

### Auto-deployment

Start Minifly with auto-deployment:

```bash
cd examples/basic-app
minifly serve --dev
```

This will:
1. Start the Minifly platform
2. Automatically deploy the app
3. Watch for file changes
4. Redeploy on changes

## Understanding the Output

When deployed, you'll see:

```
🚀 Deploying app basic-app...
✓ App basic-app created
🔨 Building Docker image...
✓ Docker image built: basic-app-local:latest
🚀 Creating machine...
✓ Machine created: abc123
✅ Application deployed successfully!
🔗 Access your app at: http://localhost:32768
```

The port (32768 in this example) is automatically assigned by Docker to avoid conflicts.

## Testing the Deployment

1. **Check the app is running:**
   ```bash
   curl http://localhost:32768
   # Output: Hello from Minifly!
   ```

2. **View running machines:**
   ```bash
   minifly machine list basic-app
   ```

3. **Check logs:**
   ```bash
   minifly logs <machine-id>
   ```

## How It Works

1. **App Creation**: Minifly creates an app entry in its database
2. **Image Building**: Docker builds the image using your Dockerfile
3. **Container Creation**: A container is created with automatic port mapping
4. **Service Start**: The container starts and your app becomes accessible

## Port Allocation

Minifly uses Docker's automatic port allocation (port 0) to:
- Avoid port conflicts
- Allow multiple apps to run simultaneously
- Support development of multiple services

The actual port is determined after container creation and displayed in the output.

## Next Steps

Once comfortable with the basic app, try:
- Modifying the server response
- Adding environment variables
- Exploring the [Todo Auth App](./todo-auth-app) for a full-featured example
- Learning about [multi-tenant architectures](./rust-axum)

## Troubleshooting

### App doesn't respond
- Check if the container is running: `docker ps`
- View logs: `minifly logs <machine-id>`
- Ensure Minifly platform is running: `minifly status`

### Port conflicts
- Minifly automatically assigns ports, but if you see conflicts:
  - Stop other services on port 80
  - Or modify the fly.toml to use a different port

### Build failures
- Ensure Docker is running: `docker version`
- Check Dockerfile syntax
- Verify all files are present