# Use a simple alpine base with a basic application
FROM alpine:latest

# Install sqlite and netcat
RUN apk add --no-cache sqlite netcat-openbsd

# Create app directory
WORKDIR /app

# Create a simple HTTP server script
RUN echo '#!/bin/sh' > /app/server.sh && \
    echo 'echo "Multi-tenant app running on port 8080"' >> /app/server.sh && \
    echo 'echo "Database: SQLite"' >> /app/server.sh && \
    echo 'echo "Using LiteFS for replication"' >> /app/server.sh && \
    echo 'echo "Starting HTTP server..."' >> /app/server.sh && \
    echo 'while true; do' >> /app/server.sh && \
    echo '  echo -e "HTTP/1.1 200 OK\\n\\n<h1>Multi-Tenant Demo App</h1><p>Running on Minifly!</p><p>Database: SQLite with LiteFS</p>" | nc -l -p 8080' >> /app/server.sh && \
    echo 'done' >> /app/server.sh && \
    chmod +x /app/server.sh

# Expose port
EXPOSE 8080

# Start the server
CMD ["/app/server.sh"]