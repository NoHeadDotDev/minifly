# Getting Started

Welcome to **Minifly**, the local development simulator for Fly.io! This guide will help you get up and running in minutes.

## What is Minifly?

Minifly is a local development tool that simulates the Fly.io platform on your machine, providing:

- 🚀 **Local Fly.io API** - Complete Machines API compatibility
- 🗄️ **LiteFS Integration** - Distributed SQLite with local replication
- 🐳 **Docker Management** - Automatic container lifecycle management
- ⚙️ **Production Config Compatibility** - Use production fly.toml without modifications
- 🔐 **Secrets Management** - Local .fly.secrets files (git-ignored)
- 📁 **Volume Mapping** - Fly.io volumes mapped to local directories
- 🌐 **Service Discovery** - .internal DNS resolution
- 🌍 **Multi-region Simulation** - Test region-specific behavior locally
- 📊 **Real-time Monitoring** - Structured logging with region context
- 🔄 **Hot Reloading** - Watch mode for automatic redeployment

## Quick Start

### Prerequisites

Before you begin, make sure you have:

- [Rust](https://rustup.rs/) (latest stable)
- [Docker](https://docs.docker.com/get-docker/) 
- [Git](https://git-scm.com/)

### Installation

1. **Install from crates.io**:
   ```bash
   cargo install minifly
   ```

   Or clone from source:
   ```bash
   git clone https://github.com/NoHeadDotDev/minifly.git
   cd minifly
   ```

2. **Initialize your environment**:
   ```bash
   minifly init
   ```

3. **Start the platform**:
   ```bash
   minifly serve
   ```
   
   If building from source:
   ```bash
   cargo build --release
   ./target/release/minifly serve
   ```

### Your First Deployment

#### Option 1: Use Production Config (Recommended)

If you have an existing Fly.io app, you can use it directly:

```bash
# Navigate to your existing app directory
cd my-existing-fly-app

# Set up any secrets needed
minifly secrets set SECRET_KEY=development-key

# Deploy using production fly.toml - no changes needed!
minifly deploy
```

#### Option 2: Create a New App

1. **Create a simple app**:
   ```bash
   mkdir my-first-app
   cd my-first-app
   ```

2. **Create a `fly.toml`** (production-ready):
   ```toml
   app = "my-first-app"
   primary_region = "iad"

   [build]
   dockerfile = "Dockerfile"

   [env]
   PORT = "8080"

   [[services]]
   internal_port = 8080
   protocol = "tcp"

   [[services.ports]]
   port = 80
   handlers = ["http"]
   ```

3. **Create a Dockerfile with Fly.io features**:
   ```dockerfile
   FROM nginx:alpine
   
   # Fly.io build args (automatically injected by Minifly)
   ARG FLY_APP_NAME
   ARG FLY_REGION
   
   COPY index.html /usr/share/nginx/html/
   
   # Create health check endpoint
   RUN echo '<!DOCTYPE html><html><body>OK</body></html>' > /usr/share/nginx/html/health
   
   EXPOSE 8080
   CMD ["nginx", "-g", "daemon off;", "-p", "8080:8080"]
   ```

4. **Create an index.html**:
   ```html
   <!DOCTYPE html>
   <html>
   <head>
       <title>My First Minifly App</title>
   </head>
   <body>
       <h1>Hello from Minifly! 🚀</h1>
       <p>App: <span id="app-name">Loading...</span></p>
       <p>Region: <span id="region">Loading...</span></p>
       <script>
           // These would be populated by your backend in a real app
           document.getElementById('app-name').textContent = 'my-first-app';
           document.getElementById('region').textContent = 'local';
       </script>
   </body>
   </html>
   ```

5. **Set up secrets** (optional):
   ```bash
   minifly secrets set ADMIN_PASSWORD=admin123
   ```

6. **Deploy your app**:
   ```bash
   minifly deploy
   ```

7. **View your app**:
   Open [http://localhost:80](http://localhost:80) in your browser!

## Next Steps

Now that you have Minifly running, here's what to explore next:

- **[Production Config Compatibility](./production-config-compatibility)** - Use your production configs locally
- **[CLI Reference](./cli-reference/)** - Complete command reference
- **[API Reference](./api-reference)** - Machines API documentation  
- **[Examples](./examples/rust-axum)** - Real-world application examples

## Getting Help

- 📖 **Documentation**: You're reading it!
- 💬 **GitHub Discussions**: [Ask questions and share ideas](https://github.com/NoHeadDotDev/minifly/discussions)
- 🐛 **Issues**: [Report bugs](https://github.com/NoHeadDotDev/minifly/issues)
- 💡 **Feature Requests**: [Suggest improvements](https://github.com/NoHeadDotDev/minifly/issues/new?template=feature_request.md)

Ready to dive deeper? Check out our [CLI Reference](./cli-reference/) next!