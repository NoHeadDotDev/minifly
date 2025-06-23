# minifly secrets

Manage application secrets for local development.

## Overview

The `minifly secrets` command provides secure secrets management for your applications without committing sensitive data to git. Secrets are stored in local `.fly.secrets` files that are automatically excluded from version control.

## Commands

### `minifly secrets set`

Set one or more secrets for your application.

```bash
minifly secrets set KEY=VALUE [KEY2=VALUE2 ...]
```

**Examples:**
```bash
# Set a single secret
minifly secrets set DATABASE_URL=postgres://localhost/myapp

# Set multiple secrets at once
minifly secrets set \
  SECRET_KEY=your-secret-key \
  API_TOKEN=sk-1234567890 \
  ADMIN_PASSWORD=secure-password

# Set secrets with special characters (use quotes)
minifly secrets set 'REDIS_URL=redis://user:pass@localhost:6379/0'
```

### `minifly secrets list`

List all secrets for the current or specified application.

```bash
minifly secrets list [APP_NAME]
```

**Examples:**
```bash
# List secrets for current directory's app
minifly secrets list

# List secrets for specific app
minifly secrets list myapp
```

**Output:**
```
🔐 Secrets for app 'myapp':
   • DATABASE_URL
   • SECRET_KEY
   • API_TOKEN
   • ADMIN_PASSWORD

4 secrets loaded from .fly.secrets.myapp
```

### `minifly secrets remove`

Remove one or more secrets from your application.

```bash
minifly secrets remove KEY [KEY2 ...]
```

**Examples:**
```bash
# Remove a single secret
minifly secrets remove API_TOKEN

# Remove multiple secrets
minifly secrets remove SECRET_KEY ADMIN_PASSWORD
```

## Secrets Files

Secrets are stored in local files using a simple KEY=VALUE format:

### File Priority

1. **`.fly.secrets.<app-name>`** - App-specific secrets (highest priority)
2. **`.fly.secrets`** - Default secrets for all apps

App-specific secrets take precedence over default secrets when both exist.

### File Format

```bash
# Comments start with #
DATABASE_URL=postgres://localhost/myapp
SECRET_KEY=your-secret-key-here
API_TOKEN=sk-1234567890

# Values can contain special characters
REDIS_URL=redis://user:pass@localhost:6379/0

# No quotes needed around values
WEBHOOK_URL=https://example.com/webhook?token=abc123
```

### Git Integration

Secrets files are automatically excluded from git via `.gitignore`:

```gitignore
# Minifly secrets (DO NOT COMMIT)
.fly.secrets
.fly.secrets.*
```

## Security Best Practices

### ✅ Do

- Use different secrets for development and production
- Store production secrets in Fly.io's secrets management
- Use meaningful secret names (DATABASE_URL vs DB_URL)
- Regularly rotate secrets
- Use app-specific secrets files for multi-app projects

### ❌ Don't

- Commit `.fly.secrets` files to git
- Share secrets files via email or chat
- Use production secrets in development
- Hardcode secrets in `fly.toml` or source code
- Use weak or predictable secret values

## Environment Variable Integration

Secrets are automatically loaded as environment variables when your application starts. No manual sourcing or export commands required - Minifly handles the integration seamlessly during container creation:

```rust
// In your Rust application
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL secret not set");

let secret_key = std::env::var("SECRET_KEY")
    .expect("SECRET_KEY secret not set");
```

```javascript
// In your Node.js application
const databaseUrl = process.env.DATABASE_URL;
const secretKey = process.env.SECRET_KEY;
```

## Multi-App Development

For projects with multiple applications:

```bash
# Set secrets for frontend app
cd frontend
minifly secrets set API_BASE_URL=http://localhost:3001

# Set secrets for backend app  
cd ../backend
minifly secrets set DATABASE_URL=postgres://localhost/backend_db
minifly secrets set JWT_SECRET=backend-jwt-secret
```

This creates separate secrets files:
- `frontend/.fly.secrets.frontend`
- `backend/.fly.secrets.backend`

## Migration from Environment Variables

If you're currently using environment variables for secrets:

```bash
# Old approach ❌
export DATABASE_URL=postgres://localhost/myapp
export SECRET_KEY=dev-secret
minifly deploy

# New approach ✅  
minifly secrets set DATABASE_URL=postgres://localhost/myapp
minifly secrets set SECRET_KEY=dev-secret
minifly deploy  # Secrets automatically loaded
```

## Troubleshooting

### Secrets Not Loading

1. **Check file exists:**
   ```bash
   ls -la .fly.secrets*
   ```

2. **Verify file format:**
   ```bash
   cat .fly.secrets
   # Should be KEY=VALUE format, no quotes around values
   ```

3. **Check app name:**
   ```bash
   grep "^app" fly.toml
   # Ensure app name matches secrets file suffix
   ```

### Permission Issues

```bash
# Fix file permissions
chmod 600 .fly.secrets*

# Ensure ownership
chown $USER .fly.secrets*
```

### Special Characters in Values

```bash
# ✅ Correct - no quotes needed
DATABASE_URL=postgres://user:pass@localhost/db

# ❌ Incorrect - quotes will be included in value  
DATABASE_URL="postgres://user:pass@localhost/db"
```

### Debugging Secret Loading

Check if secrets are loaded by examining environment variables in your app:

```rust
// Add to your application for debugging
for (key, value) in std::env::vars() {
    if key.contains("SECRET") || key.contains("TOKEN") || key.contains("PASSWORD") {
        println!("Loaded secret: {} = [REDACTED]", key);
    }
}
```

## Examples

### Database Application

```bash
# Set database connection secrets
minifly secrets set \
  DATABASE_URL=postgres://localhost/myapp \
  REDIS_URL=redis://localhost:6379/0 \
  DB_ENCRYPTION_KEY=32-char-encryption-key

# Deploy with secrets
minifly deploy
```

### API Integration

```bash
# Set API credentials
minifly secrets set \
  STRIPE_SECRET_KEY=sk_test_123... \
  SENDGRID_API_KEY=SG.abc123... \
  JWT_SECRET=your-jwt-secret

# List to verify
minifly secrets list
```

### Multi-Environment Setup

```bash
# Default secrets (shared across apps)
minifly secrets set LOG_LEVEL=debug

# App-specific secrets
cd app1
minifly secrets set DATABASE_URL=postgres://localhost/app1_db

cd ../app2  
minifly secrets set DATABASE_URL=postgres://localhost/app2_db
```

The secrets management system ensures your sensitive data stays secure while providing a seamless development experience that matches production Fly.io behavior.