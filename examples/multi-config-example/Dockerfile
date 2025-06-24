FROM node:18-alpine

WORKDIR /app

# Simple Node.js app to demonstrate environment differences
RUN echo 'const http = require("http"); \
const server = http.createServer((req, res) => { \
  res.writeHead(200, {"Content-Type": "application/json"}); \
  res.end(JSON.stringify({ \
    environment: process.env.NODE_ENV || "unknown", \
    logLevel: process.env.LOG_LEVEL || "info", \
    apiUrl: process.env.API_URL || "not set", \
    timestamp: new Date().toISOString() \
  })); \
}); \
server.listen(8080, () => console.log("Server running on port 8080"));' > server.js

EXPOSE 8080
CMD ["node", "server.js"]