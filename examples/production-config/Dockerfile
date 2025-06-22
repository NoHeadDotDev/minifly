# Production Dockerfile with Fly.io specific features
FROM rust:1.75 as builder

# Fly.io build arguments (automatically injected by Minifly)
ARG FLY_APP_NAME
ARG FLY_REGION
ARG FLY_BUILD_ID

# Display build info
RUN echo "Building app: ${FLY_APP_NAME} in region: ${FLY_REGION} with build ID: ${FLY_BUILD_ID}"

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/production-app /app/production-app

# Create data directory for volumes
RUN mkdir -p /data

# Fly.io expects the app to listen on PORT
ENV PORT=8080
EXPOSE 8080

# Health check endpoint
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

CMD ["./production-app"]