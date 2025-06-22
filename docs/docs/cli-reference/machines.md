# minifly machines

Manage Fly Machines (containers) for your applications.

## Synopsis

```bash
minifly machines [COMMAND] [OPTIONS]
```

## Commands

### list

List machines for an application.

```bash
minifly machines list --app <APP> [OPTIONS]
```

**Options:**
- `--app <APP>` - Application name (required)
- `--json` - Output in JSON format
- `-h, --help` - Print help information

**Example:**
```bash
$ minifly machines list --app my-app
ID              NAME            STATE     REGION    IMAGE           CREATED
d891234567890   web-1           started   sjc       nginx:latest    2024-06-22 10:00:00
d891234567891   web-2           started   ord       nginx:latest    2024-06-22 10:05:00
```

### create

Create a new machine.

```bash
minifly machines create --app <APP> --image <IMAGE> [OPTIONS]
```

**Required Options:**
- `--app <APP>` - Application name
- `--image <IMAGE>` - Docker image

**Additional Options:**
- `--name <NAME>` - Machine name (auto-generated if not provided)
- `--region <REGION>` - Region code (default: sjc)
- `--size <SIZE>` - Machine size (default: shared-cpu-1x)
- `--env <KEY=VALUE>` - Environment variables (can be used multiple times)
- `--port <PORT>` - Expose port (format: `80:8080/tcp`)
- `--volume <VOLUME>` - Attach volume (format: `volume_name:/mount/path`)
- `--cmd <CMD>` - Override container command
- `--entrypoint <ENTRYPOINT>` - Override container entrypoint
- `-h, --help` - Print help information

**Example:**
```bash
$ minifly machines create \
    --app my-app \
    --image nginx:latest \
    --name web-1 \
    --region sjc \
    --env PORT=8080 \
    --port 80:8080/tcp
✓ Created machine d891234567890 (web-1) in sjc
```

### start

Start a stopped machine.

```bash
minifly machines start <MACHINE_ID> [OPTIONS]
```

**Arguments:**
- `<MACHINE_ID>` - Machine ID

**Options:**
- `-h, --help` - Print help information

**Example:**
```bash
$ minifly machines start d891234567890
✓ Started machine d891234567890
```

### stop

Stop a running machine.

```bash
minifly machines stop <MACHINE_ID> [OPTIONS]
```

**Arguments:**
- `<MACHINE_ID>` - Machine ID

**Options:**
- `--timeout <SECONDS>` - Graceful shutdown timeout (default: 30)
- `-h, --help` - Print help information

**Example:**
```bash
$ minifly machines stop d891234567890
✓ Stopped machine d891234567890
```

### restart

Restart a machine.

```bash
minifly machines restart <MACHINE_ID> [OPTIONS]
```

**Arguments:**
- `<MACHINE_ID>` - Machine ID

**Options:**
- `--timeout <SECONDS>` - Stop timeout (default: 30)
- `-h, --help` - Print help information

**Example:**
```bash
$ minifly machines restart d891234567890
✓ Restarted machine d891234567890
```

### show

Show detailed information about a machine.

```bash
minifly machines show <MACHINE_ID> [OPTIONS]
```

**Arguments:**
- `<MACHINE_ID>` - Machine ID

**Options:**
- `--json` - Output in JSON format
- `-h, --help` - Print help information

**Example:**
```bash
$ minifly machines show d891234567890
Machine: d891234567890
Name: web-1
State: started
Region: sjc
Image: nginx:latest
Size: shared-cpu-1x
IP: 10.0.1.2
Created: 2024-06-22 10:00:00
Updated: 2024-06-22 14:30:00

Environment:
  PORT=8080
  NODE_ENV=production

Services:
  tcp 80 -> 8080 [http]
```

### update

Update a machine's configuration.

```bash
minifly machines update <MACHINE_ID> [OPTIONS]
```

**Arguments:**
- `<MACHINE_ID>` - Machine ID

**Options:**
- `--image <IMAGE>` - New Docker image
- `--env <KEY=VALUE>` - Update environment variables
- `--size <SIZE>` - Change machine size
- `--cmd <CMD>` - Update command
- `--entrypoint <ENTRYPOINT>` - Update entrypoint
- `-h, --help` - Print help information

**Example:**
```bash
$ minifly machines update d891234567890 \
    --image nginx:1.21 \
    --env VERSION=1.21
✓ Updated machine d891234567890
```

### delete

Delete a machine.

```bash
minifly machines delete <MACHINE_ID> [OPTIONS]
```

**Arguments:**
- `<MACHINE_ID>` - Machine ID

**Options:**
- `--force` - Skip confirmation
- `-h, --help` - Print help information

**Example:**
```bash
$ minifly machines delete d891234567890
⚠️  This will delete machine d891234567890
? Are you sure? › Yes
✓ Deleted machine d891234567890
```

### exec

Execute a command in a running machine.

```bash
minifly machines exec <MACHINE_ID> <COMMAND> [ARGS...] [OPTIONS]
```

**Arguments:**
- `<MACHINE_ID>` - Machine ID
- `<COMMAND>` - Command to execute
- `[ARGS...]` - Command arguments

**Options:**
- `-h, --help` - Print help information

**Example:**
```bash
$ minifly machines exec d891234567890 /bin/bash
root@d891234567890:/# 
```

## Machine States

- `created` - Machine created but not started
- `starting` - Machine is booting up
- `started` - Machine is running
- `stopping` - Machine is shutting down
- `stopped` - Machine is stopped
- `failed` - Machine failed to start
- `destroyed` - Machine is deleted

## Machine Sizes

Available machine sizes:
- `shared-cpu-1x` - 1 shared CPU, 256MB RAM
- `shared-cpu-2x` - 2 shared CPUs, 512MB RAM
- `shared-cpu-4x` - 4 shared CPUs, 1GB RAM
- `shared-cpu-8x` - 8 shared CPUs, 2GB RAM

## Regions

Minifly simulates multiple regions:
- `sjc` - San Jose, California (default)
- `ord` - Chicago, Illinois
- `lhr` - London, United Kingdom
- `nrt` - Tokyo, Japan
- `syd` - Sydney, Australia

## Networking

Machines within an app can communicate:
- Internal DNS: `<machine-name>.internal`
- IPv6 networking: `fdaa:0:1:a7b::/64`
- Service discovery via internal DNS

## Volumes

Attach persistent storage to machines:

```bash
# Create a volume
minifly volumes create my-data --app my-app --size 1

# Attach during machine creation
minifly machines create \
    --app my-app \
    --image postgres:15 \
    --volume my-data:/var/lib/postgresql/data
```

## Health Checks

Machines support health checks:

```toml
# In fly.toml
[checks]
  [checks.web]
    grace_period = "5s"
    interval = "15s"
    method = "get"
    path = "/health"
    port = 8080
    timeout = "2s"
    type = "http"
```

## Examples

### Deploy a Web Application
```bash
# Create machine with web service
minifly machines create \
    --app my-app \
    --image myapp:latest \
    --port 80:8080/tcp \
    --env PORT=8080
```

### Run a Database
```bash
# Create volume for data persistence
minifly volumes create pgdata --app my-app --size 10

# Create PostgreSQL machine
minifly machines create \
    --app my-app \
    --image postgres:15 \
    --name db-primary \
    --env POSTGRES_PASSWORD=secret \
    --volume pgdata:/var/lib/postgresql/data
```

### Scale Horizontally
```bash
# Create multiple web machines
for i in {1..3}; do
  minifly machines create \
    --app my-app \
    --image myapp:latest \
    --name "web-$i" \
    --region sjc \
    --port 80:8080/tcp
done
```

## See Also

- [apps](./apps) - Manage applications
- [logs](./logs) - View machine logs
- [deploy](./deploy) - Deploy from source