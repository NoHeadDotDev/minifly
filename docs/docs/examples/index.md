---
sidebar_position: 1
---

# Examples Overview

Minifly comes with several example applications that demonstrate different features and use cases. Each example includes complete source code, deployment configuration, and documentation.

## Available Examples

### 🚀 [Basic App](./basic-app)
A simple "Hello World" application perfect for getting started with Minifly.

**What you'll learn:**
- Basic fly.toml configuration
- Simple Dockerfile setup
- Auto-deployment with `minifly serve --dev`
- Port allocation and container management

**Best for:** First-time users wanting to understand the basics

---

### 🔐 [Todo Auth App](./todo-auth-app) ⭐ Recommended
A comprehensive example showcasing authentication and multi-tenant capabilities.

**What you'll learn:**
- Email + password authentication with secure sessions
- Multi-tenant architecture with per-user isolation
- Region selection and deployment
- File uploads and storage
- Modern web app patterns with Rust and Axum

**Best for:** Developers building multi-tenant SaaS applications

---

### 🏢 [Multi-Tenant App](./rust-axum)
Demonstrates multi-tenant architecture with per-tenant databases using LiteFS.

**What you'll learn:**
- Tenant isolation strategies
- LiteFS for distributed SQLite
- Dynamic tenant routing
- Middleware patterns

**Best for:** Understanding database-per-tenant architectures

---

### ⚙️ [Production Config](./production-config)
Shows how to use production Fly.io configurations with Minifly.

**What you'll learn:**
- Complex fly.toml configurations
- Environment variable handling
- Health checks and monitoring
- Production deployment patterns

**Best for:** Teams migrating from or deploying to Fly.io

## Quick Start

1. **Start Minifly** (if not already running):
   ```bash
   minifly serve --dev
   ```

2. **Choose an example** and navigate to its directory:
   ```bash
   cd examples/todo-auth-app
   ```

3. **Deploy the example**:
   ```bash
   minifly deploy
   # or use the provided run script
   ./run.sh
   ```

## Creating Your Own Example

To create a new example application:

1. Create a directory under `examples/`
2. Add a `fly.toml` configuration file
3. Include a `Dockerfile` or specify a Docker image
4. Write a comprehensive `README.md`
5. (Optional) Add a `run.sh` script for easy deployment

### Example Structure

```
my-example/
├── fly.toml          # Minifly deployment configuration
├── Dockerfile        # Container definition
├── README.md         # Documentation
├── run.sh           # Optional deployment script
└── src/             # Application source code
```

## Tips for Working with Examples

- **Auto-deployment**: Run `minifly serve --dev` from an example directory to automatically deploy on startup
- **File watching**: In dev mode, changes trigger automatic redeployment
- **Logs**: Use `minifly logs <machine-id>` to debug issues
- **Cleanup**: Stop all containers with `minifly machine stop --all`

## Contributing Examples

We welcome contributions! Good examples should:

- Demonstrate specific Minifly features
- Include clear, educational code
- Have comprehensive documentation
- Work out-of-the-box with `minifly deploy`
- Follow Rust and web development best practices

Submit your examples via pull request to the [Minifly repository](https://github.com/your-repo/minifly).