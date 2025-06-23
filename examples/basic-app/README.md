# Basic Minifly Example

This example demonstrates how to deploy a simple nginx application using Minifly with production config compatibility.

## Quick Start (Recommended)

With Minifly's auto-deployment, you can run this example with just one command!

### Prerequisites

- [Minifly installed](../../docs/docs/getting-started.md): `cargo install minifly-cli`
- Docker installed and running

### Steps

1. **Run with auto-deployment**:
   ```bash
   cd examples/basic-app
   minifly serve --dev
   ```

2. **Access your application**:
   
   Minifly will show you the URL with the automatically assigned port, for example:
   ```bash
   🔗 Access your app at: http://localhost:32768
   ```
   
   Open the URL shown in your terminal, or use curl:
   ```bash
   # Use the port shown in your terminal output
   curl http://localhost:32768
   ```

That's it! 🚀

Minifly will automatically:
- ✅ Start the platform
- ✅ Detect the project configuration
- ✅ Deploy the nginx container
- ✅ Assign an available port (no more port conflicts!)
- ✅ Enable file watching for changes
- ✅ Show real-time logs

## Advanced: Manual Deployment

For advanced users who want to control each step, you can also deploy manually:

### Steps

1. **Start Minifly platform**:
   ```bash
   minifly serve
   ```

2. **Deploy the app** (in a new terminal):
   ```bash
   cd examples/basic-app
   minifly deploy
   ```

### What Minifly Does Automatically

When you run `minifly deploy`, Minifly automatically:

- ✅ **Reads fly.toml** - Uses your production configuration
- ✅ **Pulls nginx:alpine** - Downloads the Docker image
- ✅ **Creates the app** - Automatically creates "example-app"
- ✅ **Starts the machine** - Runs the nginx container
- ✅ **Maps ports** - Automatically assigns available port (no conflicts!)
- ✅ **Injects Fly variables** - FLY_APP_NAME, FLY_REGION, etc.
- ✅ **Shows warnings** - Alerts about auto_stop_machines simulation

## Example Output

```bash
$ minifly deploy
📖 Reading fly.toml...
🚀 Deploying app example-app...

⚠️  Compatibility warnings found:
   • auto_stop_machines is simulated with container pause/unpause
   • auto_start_machines is not fully supported - machines start manually

✓ App example-app already exists
🔨 Using image: nginx:alpine
🚀 Creating machine...
✓ Machine created: d89ad568e4178e1
⏳ Waiting for machine to start...

✅ Application deployed successfully!
🔗 Access your app at: http://localhost:32768

📝 To check machine status:
   minifly machines list example-app

📋 To view logs:
   minifly logs d89ad568e4178e1
```

## Development Workflow

### Watch Mode

Enable automatic redeployment when files change:

```bash
minifly deploy --watch
```

### View Logs

```bash
# List machines to get machine ID
minifly machines list example-app

# View logs
minifly logs <machine-id>

# Follow logs in real-time
minifly logs <machine-id> --follow
```

### Check Status

```bash
# Platform status
minifly status

# Machine status
minifly machines list example-app
```

### Stop the App

```bash
# Stop the machine
minifly machines stop <machine-id>

# Or stop Minifly entirely
minifly stop
```

## Advanced Usage

### Add Secrets

```bash
# Add some secrets
minifly secrets set NGINX_WORKER_PROCESSES=4
minifly secrets set APP_VERSION=1.0.0

# List secrets
minifly secrets list
```

### Custom Configuration

You can modify the `fly.toml` to experiment with different configurations:

```toml
# Add environment variables
[env]
APP_ENV = "development"
DEBUG = "true"

# Add health checks
[[services.http_checks]]
interval = "10s"
grace_period = "5s"
method = "GET"
path = "/"
protocol = "http"
timeout = "2s"
```

### Multiple Instances

Scale up by creating more machines:

```bash
# Create additional machines
minifly machines create --app example-app --image nginx:alpine
minifly machines create --app example-app --image nginx:alpine

# List all machines
minifly machines list example-app
```

## Troubleshooting

### Finding Your App's Port

Since Minifly automatically assigns available ports, you can find your app's port by:

```bash
# Check running containers and their port mappings
docker ps --filter "name=minifly-example-app"

# Or check the deployment output which shows the assigned port
```

### Container Won't Start

```bash
# Check Docker is running
docker ps

# View detailed logs
minifly logs <machine-id>

# Check machine status
minifly machines list example-app
```

### App Not Accessible

```bash
# Verify the machine is running
minifly machines list example-app

# Check port mapping
docker ps

# Test with curl (use the port shown in deployment output)
curl -v http://localhost:<PORT>
```

## Legacy Manual Approach

If you prefer the manual approach (not recommended for new users):

<details>
<summary>Click to expand manual steps</summary>

1. **Start Minifly API server**:
   ```bash
   minifly serve
   ```

2. **Create the app manually**:
   ```bash
   minifly apps create example-app
   ```

3. **Create a machine manually**:
   ```bash
   minifly machines create --app example-app --image nginx:alpine
   ```

4. **List machines**:
   ```bash
   minifly machines list --app example-app
   ```

</details>

## Next Steps

- Try the [multi-tenant-app example](../multi-tenant-app/) for a more complex application
- Explore [production-config example](../production-config/) to see all compatibility features
- Read the [Production Config Compatibility guide](../../docs/docs/production-config-compatibility.md)

## API Usage

You can also interact with the Minifly API directly:

```bash
# Create a machine via API
curl -X POST http://localhost:4280/v1/apps/example-app/machines \
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