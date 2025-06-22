# minifly init

Initialize a new Minifly project or update configuration.

## Synopsis

```bash
minifly init [OPTIONS]
```

## Description

The `init` command provides an interactive setup experience for:
- Creating Minifly configuration
- Scaffolding new projects with templates
- Setting up multi-tenant applications
- Configuring development environments

## Options

- `-h, --help` - Print help information

## Interactive Modes

### New Project

When run in an empty directory, `init` offers to create a complete project:

1. **Template Selection** - Choose from 5 pre-configured templates
2. **Project Configuration** - Set app name and description
3. **File Generation** - Creates all necessary files
4. **Dependencies** - Sets up package files

### Existing Project

When run in a directory with existing files, `init` offers to:

- Update Minifly configuration only
- Add Minifly-specific files (fly.toml, litefs.yml)
- Reinitialize with a new template

## Available Templates

### Rust + Axum + LiteFS
- Multi-tenant web application
- Askama templating
- SQLite with LiteFS replication
- Docker ready

### Node.js + Express + SQLite
- Express.js server
- SQLite database
- ESM modules
- Docker ready

### Python + FastAPI + SQLite
- FastAPI with async support
- SQLite with aiosqlite
- Pydantic models
- Docker ready

### Go + Gin + SQLite
- Gin web framework
- SQLite database
- Structured logging
- Docker ready

### Minimal Docker
- Basic Dockerfile
- fly.toml configuration
- Ready for any language

## Generated Files

### Common Files
- `fly.toml` - Fly.io configuration
- `litefs.yml` - LiteFS configuration
- `docker-compose.yml` - Local development setup
- `README.md` - Project documentation
- `.gitignore` - Git ignore rules

### Template-Specific Files
- Source code in appropriate language
- Package/dependency files
- Database migrations
- Example data scripts

## Examples

### Initialize New Project
```bash
$ minifly init
🚀 Welcome to Minifly!
📦 Setting up a new Minifly project!

? How would you like to initialize? › Initialize with project template
? Choose a project template › Rust + Axum + LiteFS
? Application name › my-app
? Description › A multi-tenant SaaS application

📝 Creating project files...
✅ Project created successfully!
```

### Update Configuration
```bash
$ minifly init
📁 Existing project detected!

? What would you like to do? › Update Minifly configuration only
? API URL › http://localhost:4280
? API Token (optional) › 

✅ Configuration saved!
```

### Add Templates to Existing Project
```bash
$ minifly init
📁 Existing project detected!

? What would you like to do? › Add Minifly project templates
? Add Docker Compose configuration? › Yes
? Add LiteFS configuration? › Yes
? Add Multi-tenant example? › Yes

✅ Templates added!
```

## Configuration File

Creates or updates `~/.config/minifly/config.toml`:

```toml
api_url = "http://localhost:4280"
token = "your-optional-token"
```

## Multi-Tenant Setup

When selecting a multi-tenant template, `init` creates:

- Database per tenant architecture
- Tenant routing middleware
- Example tenant data
- Migration scripts
- Development helpers

## Next Steps

After initialization:

1. Start the platform: `minifly serve`
2. Deploy your app: `minifly deploy`
3. View logs: `minifly logs <machine-id>`
4. Check status: `minifly status`

## See Also

- [serve](./serve) - Start the Minifly platform
- [deploy](./deploy) - Deploy applications
- [apps](./apps) - Manage applications