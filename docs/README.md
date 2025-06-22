# Minifly Documentation

This directory contains the Docusaurus documentation site for Minifly.

## Local Development

```bash
npm install
npm start
```

This will start a local development server at http://localhost:3000.

## Building

```bash
npm run build
```

This creates a static build in the `build/` directory.

## Deploying to Fly.io

The documentation is deployed to Fly.io at https://minifly-docs.fly.dev.

### First-time deployment:

```bash
# Login to Fly.io
fly auth login

# Deploy (will create the app if it doesn't exist)
./deploy.sh
```

### Subsequent deployments:

```bash
./deploy.sh
```

Or manually:

```bash
fly deploy --app minifly-docs
```

## Configuration

- The Fly.io app configuration is in `fly.toml`
- The Dockerfile builds and serves the static site
- The site URL is configured in `docusaurus.config.js`

## Adding Documentation

1. Add new docs to the `docs/` directory
2. Update `sidebars.js` if needed
3. Test locally with `npm start`
4. Deploy with `./deploy.sh`