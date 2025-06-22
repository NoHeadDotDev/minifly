# Getting Started

Welcome to **Minifly**, the local development simulator for Fly.io! This guide will help you get up and running in minutes.

## What is Minifly?

Minifly is a local development tool that simulates the Fly.io platform on your machine, providing:

- 🚀 **Local Fly.io API** - Complete Machines API compatibility
- 🗄️ **LiteFS Integration** - Distributed SQLite with local replication
- 🐳 **Docker Management** - Automatic container lifecycle management
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

1. **Clone the repository**:
   ```bash
   git clone https://github.com/minifly/minifly.git
   cd minifly
   ```

2. **Build Minifly**:
   ```bash
   cargo build --release
   ```

3. **Initialize your environment**:
   ```bash
   ./target/release/minifly init
   ```

4. **Start the platform**:
   ```bash
   ./target/release/minifly serve
   ```

### Your First Deployment

1. **Create a simple app**:
   ```bash
   mkdir my-first-app
   cd my-first-app
   ```

2. **Create a `fly.toml`**:
   ```toml
   app = "my-first-app"
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

3. **Create a simple Dockerfile**:
   ```dockerfile
   FROM nginx:alpine
   COPY index.html /usr/share/nginx/html/
   EXPOSE 80
   CMD ["nginx", "-g", "daemon off;"]
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
       <p>Your app is running locally with Fly.io compatibility.</p>
   </body>
   </html>
   ```

5. **Deploy your app**:
   ```bash
   minifly deploy
   ```

6. **View your app**:
   Open [http://localhost:80](http://localhost:80) in your browser!

## Next Steps

Now that you have Minifly running, here's what to explore next:

- **[Development Workflow](./guides/development-workflow)** - Learn about watch mode and hot reloading
- **[Multi-tenant Apps](./guides/multi-tenant-apps)** - Build apps with SQLite/LiteFS per tenant
- **[CLI Reference](./cli-reference/overview)** - Complete command reference
- **[Examples](./examples/rust-axum)** - Real-world application examples

## Getting Help

- 📖 **Documentation**: You're reading it!
- 💬 **GitHub Discussions**: [Ask questions and share ideas](https://github.com/minifly/minifly/discussions)
- 🐛 **Issues**: [Report bugs](https://github.com/minifly/minifly/issues)
- 💡 **Feature Requests**: [Suggest improvements](https://github.com/minifly/minifly/issues/new?template=feature_request.md)

Ready to dive deeper? Check out our [comprehensive guides](./guides/installation) next!