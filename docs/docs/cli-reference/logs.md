# minifly logs

View and stream logs from machines.

## Synopsis

```bash
minifly logs <MACHINE_ID> [OPTIONS]
```

## Description

The `logs` command displays logs from running machines with real-time streaming support. It provides:
- Historical log retrieval
- Real-time log streaming
- Structured log parsing
- Multi-region log aggregation
- Powerful filtering options

## Arguments

- `<MACHINE_ID>` - Machine ID to view logs from

## Options

- `--app <APP>` - Filter by application name
- `--follow, -f` - Stream logs in real-time
- `--tail <LINES>` - Number of recent lines to show (default: 100)
- `--since <TIME>` - Show logs since timestamp (e.g., "2h", "2024-06-22T10:00:00Z")
- `--until <TIME>` - Show logs until timestamp
- `--timestamps, -t` - Show timestamps
- `--no-color` - Disable colored output
- `--json` - Output logs in JSON format
- `--region <REGION>` - Filter by region
- `--level <LEVEL>` - Filter by log level (debug, info, warn, error)
- `-h, --help` - Print help information

## Real-time Streaming

Follow logs as they're generated:

```bash
# Stream logs
minifly logs d891234567890 --follow

# Stream with timestamps
minifly logs d891234567890 -f -t
```

## Historical Logs

View past logs:

```bash
# Last 50 lines
minifly logs d891234567890 --tail 50

# Logs from last 2 hours
minifly logs d891234567890 --since 2h

# Logs between timestamps
minifly logs d891234567890 \
  --since "2024-06-22T10:00:00Z" \
  --until "2024-06-22T12:00:00Z"
```

## Output Formats

### Default Format
```
[2024-06-22T10:30:45Z] [INFO] Server starting on port 8080
[2024-06-22T10:30:46Z] [INFO] Connected to database
[2024-06-22T10:30:47Z] [WARN] Deprecated API endpoint accessed
```

### With Region Context
```
[sjc] [2024-06-22T10:30:45Z] [INFO] Server starting on port 8080
[ord] [2024-06-22T10:30:46Z] [INFO] Replica synchronized
```

### JSON Format
```json
{
  "timestamp": "2024-06-22T10:30:45Z",
  "level": "INFO",
  "message": "Server starting on port 8080",
  "machine_id": "d891234567890",
  "region": "sjc",
  "app": "my-app"
}
```

## Log Levels

Filter by severity:

```bash
# Only errors and above
minifly logs d891234567890 --level error

# Debug and above
minifly logs d891234567890 --level debug
```

Levels (in order):
- `debug` - Detailed debugging information
- `info` - General informational messages
- `warn` - Warning messages
- `error` - Error messages
- `fatal` - Fatal errors

## Structured Logs

Minifly automatically parses structured logs:

```bash
# Application outputs JSON logs
{"level":"info","msg":"Request processed","duration":45,"status":200}

# Minifly displays formatted
[INFO] Request processed duration=45ms status=200
```

## Multi-Machine Logs

View logs from multiple machines:

```bash
# All machines in an app (coming soon)
minifly logs --app my-app --follow

# Specific machines (coming soon)
minifly logs d891234567890,d891234567891 --follow
```

## Examples

### Basic Log Viewing
```bash
$ minifly logs d891234567890
[2024-06-22T10:30:45Z] Starting application...
[2024-06-22T10:30:46Z] Listening on port 8080
[2024-06-22T10:30:47Z] Ready to accept connections
```

### Stream Logs with Filters
```bash
$ minifly logs d891234567890 -f --level warn
[2024-06-22T10:35:12Z] [WARN] Slow query detected: 1.5s
[2024-06-22T10:36:45Z] [ERROR] Failed to connect to cache
[2024-06-22T10:36:46Z] [WARN] Falling back to database
```

### Debug Application Issues
```bash
# View recent errors
$ minifly logs d891234567890 --tail 200 --level error

# Stream logs during deployment
$ minifly deploy & minifly logs d891234567890 -f

# Check specific time period
$ minifly logs d891234567890 \
    --since "10 minutes ago" \
    --level debug
```

### Export Logs
```bash
# Export to file
minifly logs d891234567890 --json > logs.json

# Export time range
minifly logs d891234567890 \
  --since "2024-06-22T00:00:00Z" \
  --until "2024-06-22T23:59:59Z" \
  --json > daily-logs.json
```

## Performance Considerations

- Use `--tail` to limit initial log retrieval
- Use `--since` to avoid loading old logs
- JSON format is more efficient for parsing
- Streaming (`--follow`) uses Server-Sent Events

## Log Retention

Local Minifly retains logs based on:
- Disk space available
- Maximum age (default: 7 days)
- Maximum size per app (default: 1GB)

## Troubleshooting

### No Logs Appearing
```bash
# Check if machine is running
minifly machines show d891234567890

# Check with debug output
MINIFLY_DEBUG=1 minifly logs d891234567890
```

### Logs Cut Off
```bash
# Increase tail limit
minifly logs d891234567890 --tail 1000

# Use time-based filtering
minifly logs d891234567890 --since 1h
```

## Integration

### With Unix Tools
```bash
# Search logs
minifly logs d891234567890 | grep ERROR

# Count occurrences
minifly logs d891234567890 --json | jq '.level' | sort | uniq -c

# Monitor specific pattern
minifly logs d891234567890 -f | grep --line-buffered "timeout"
```

### Monitoring Scripts
```bash
#!/bin/bash
# Alert on errors
minifly logs d891234567890 -f --level error | while read line; do
  echo "ALERT: $line" | mail -s "App Error" ops@example.com
done
```

## See Also

- [machines](./machines) - Manage machines
- [status](./status) - Check machine status
- [deploy](./deploy) - Deploy and monitor logs