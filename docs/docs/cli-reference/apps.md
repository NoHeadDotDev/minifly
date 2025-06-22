# minifly apps

Manage Minifly applications.

## Synopsis

```bash
minifly apps [COMMAND] [OPTIONS]
```

## Commands

### list

List all applications.

```bash
minifly apps list [OPTIONS]
```

**Options:**
- `--json` - Output in JSON format
- `-h, --help` - Print help information

**Example:**
```bash
$ minifly apps list
NAME           ORGANIZATION    STATUS    MACHINES    CREATED
my-app         personal        deployed  2           2024-06-22 10:00:00
test-app       personal        created   0           2024-06-22 11:00:00
```

### create

Create a new application.

```bash
minifly apps create <NAME> [OPTIONS]
```

**Arguments:**
- `<NAME>` - Application name (lowercase, alphanumeric, hyphens allowed)

**Options:**
- `--org <ORG>` - Organization slug (default: personal)
- `-h, --help` - Print help information

**Example:**
```bash
$ minifly apps create my-awesome-app
✓ Created app 'my-awesome-app' in organization 'personal'
```

### show

Show detailed information about an application.

```bash
minifly apps show <NAME> [OPTIONS]
```

**Arguments:**
- `<NAME>` - Application name

**Options:**
- `--json` - Output in JSON format
- `-h, --help` - Print help information

**Example:**
```bash
$ minifly apps show my-app
Application: my-app
Organization: personal
Status: deployed
Machines: 2
Regions: sjc, ord
Created: 2024-06-22 10:00:00
Updated: 2024-06-22 14:30:00
```

### delete

Delete an application and all its resources.

```bash
minifly apps delete <NAME> [OPTIONS]
```

**Arguments:**
- `<NAME>` - Application name

**Options:**
- `--force` - Skip confirmation prompt
- `-h, --help` - Print help information

**Example:**
```bash
$ minifly apps delete test-app
⚠️  This will delete app 'test-app' and all its resources
? Are you sure? › Yes
✓ Deleted app 'test-app'
```

## Application Names

Application names must:
- Be 3-30 characters long
- Contain only lowercase letters, numbers, and hyphens
- Start with a letter
- End with a letter or number
- Be globally unique within your Minifly instance

## Organization Management

Applications belong to organizations. In local development, the default organization is "personal".

## Application States

- `created` - Application created but no machines deployed
- `deployed` - Application has running machines
- `suspended` - Application is suspended (machines stopped)

## Environment Variables

Applications can have environment variables set at creation or updated later:

```bash
# Set during creation (coming soon)
minifly apps create my-app --env PORT=8080 --env NODE_ENV=production

# Update existing app (use machines update)
minifly machines update <machine-id> --env NEW_VAR=value
```

## Volumes

Applications can have persistent volumes attached:

```bash
# List volumes for an app
minifly volumes list --app my-app

# Create a volume
minifly volumes create my-data --app my-app --size 1
```

## Networking

Each application gets its own internal network for machine-to-machine communication:
- Internal DNS: `<machine-name>.internal`
- IPv6 addressing within the app network
- Isolated from other applications

## Examples

### Create and Deploy a Simple App
```bash
# Create the app
minifly apps create hello-world

# Deploy from current directory
minifly deploy --app hello-world

# List machines
minifly machines list --app hello-world
```

### Multi-Region Application
```bash
# Create app
minifly apps create global-app

# Deploy to multiple regions
minifly machines create --app global-app --region sjc --image nginx
minifly machines create --app global-app --region ord --image nginx
minifly machines create --app global-app --region lhr --image nginx
```

## See Also

- [machines](./machines) - Manage application machines
- [deploy](./deploy) - Deploy applications
- [logs](./logs) - View application logs