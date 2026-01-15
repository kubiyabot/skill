# MySQL Skill

Execute MySQL queries and manage databases through a containerized client.

## Overview

The MySQL skill provides AI agents with MySQL database access. Run queries, manage schemas, export data, and monitor database health through a secure Docker-based MySQL client.

**Runtime**: Docker (containerized `mysql` client)
**Source**: [examples/docker-skills/mysql-skill](https://github.com/kubiyabot/skill/tree/main/examples/docker-skills/mysql-skill)

## Installation

```bash
# Install the skill
skill install github:kubiyabot/skill:mysql

# Or from local directory
skill install ./examples/docker-skills/mysql-skill
```

## Configuration

Configure your MySQL connection:

```bash
skill config mysql \
  --set host=localhost \
  --set port=3306 \
  --set user=myuser \
  --set password=mypassword \
  --set database=mydb
```

Or via environment variables:

```bash
export MYSQL_HOST=localhost
export MYSQL_PORT=3306
export MYSQL_USER=myuser
export MYSQL_PASSWORD=mypassword
export MYSQL_DATABASE=mydb
```

## Tools Reference

### query

Execute a SQL query and return results.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sql` | string | Yes | SQL query to execute |
| `database` | string | No | Database name (overrides default) |
| `format` | string | No | Output format: `table`, `json`, `csv` (default: table) |

**Examples:**

```bash
# Simple query
skill run mysql query --sql "SELECT * FROM users LIMIT 10"

# Query with JSON output
skill run mysql query \
  --sql "SELECT id, name, email FROM users WHERE active = 1" \
  --format json

# Query specific database
skill run mysql query \
  --sql "SELECT COUNT(*) as total FROM orders" \
  --database sales
```

**Output (JSON format):**
```json
{
  "rows": [
    {"id": 1, "name": "John Doe", "email": "john@example.com"},
    {"id": 2, "name": "Jane Smith", "email": "jane@example.com"}
  ],
  "row_count": 2,
  "execution_time_ms": 12
}
```

### execute

Execute a write operation (INSERT, UPDATE, DELETE).

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sql` | string | Yes | SQL statement to execute |
| `database` | string | No | Database name (overrides default) |

**Examples:**

```bash
# Insert data
skill run mysql execute \
  --sql "INSERT INTO users (name, email) VALUES ('New User', 'new@example.com')"

# Update records
skill run mysql execute \
  --sql "UPDATE users SET active = 0 WHERE last_login < '2024-01-01'"

# Delete records
skill run mysql execute \
  --sql "DELETE FROM sessions WHERE expired_at < NOW()"
```

**Output:**
```json
{
  "affected_rows": 5,
  "last_insert_id": 123,
  "execution_time_ms": 45
}
```

### show-databases

List all databases on the server.

**Examples:**

```bash
skill run mysql show-databases
```

**Output:**
```json
{
  "databases": [
    "information_schema",
    "mysql",
    "performance_schema",
    "myapp_production",
    "myapp_staging"
  ]
}
```

### show-tables

List all tables in a database.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `database` | string | No | Database name (uses default if not specified) |

**Examples:**

```bash
# List tables in default database
skill run mysql show-tables

# List tables in specific database
skill run mysql show-tables --database myapp_production
```

### describe-table

Get table schema information.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `table` | string | Yes | Table name |
| `database` | string | No | Database name |

**Examples:**

```bash
skill run mysql describe-table --table users
```

**Output:**
```json
{
  "columns": [
    {"name": "id", "type": "int", "nullable": false, "key": "PRI", "default": null, "extra": "auto_increment"},
    {"name": "name", "type": "varchar(255)", "nullable": false, "key": "", "default": null, "extra": ""},
    {"name": "email", "type": "varchar(255)", "nullable": false, "key": "UNI", "default": null, "extra": ""},
    {"name": "created_at", "type": "timestamp", "nullable": false, "key": "", "default": "CURRENT_TIMESTAMP", "extra": ""}
  ],
  "indexes": [
    {"name": "PRIMARY", "columns": ["id"], "unique": true},
    {"name": "idx_email", "columns": ["email"], "unique": true}
  ]
}
```

### show-processlist

Show active database connections and queries.

**Examples:**

```bash
skill run mysql show-processlist
```

**Output:**
```json
{
  "processes": [
    {
      "id": 123,
      "user": "app_user",
      "host": "10.0.0.5:54321",
      "db": "myapp_production",
      "command": "Query",
      "time": 2,
      "state": "Sending data",
      "info": "SELECT * FROM large_table"
    }
  ]
}
```

### status

Get MySQL server status and metrics.

**Examples:**

```bash
skill run mysql status
```

**Output:**
```json
{
  "version": "8.0.35",
  "uptime_seconds": 864000,
  "connections": {
    "current": 25,
    "max": 151,
    "total": 45678
  },
  "queries": {
    "total": 1234567,
    "per_second": 142.5
  },
  "threads": {
    "connected": 25,
    "running": 3
  }
}
```

### export

Export query results to a file.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sql` | string | Yes | SQL query to export |
| `format` | string | No | Export format: `csv`, `json`, `sql` (default: csv) |
| `database` | string | No | Database name |

**Examples:**

```bash
# Export to CSV
skill run mysql export \
  --sql "SELECT * FROM users WHERE created_at > '2025-01-01'" \
  --format csv

# Export to JSON
skill run mysql export \
  --sql "SELECT * FROM orders" \
  --format json
```

## Common Workflows

### Database Health Check

```bash
# 1. Check server status
skill run mysql status

# 2. Check active connections
skill run mysql show-processlist

# 3. Check for slow queries
skill run mysql query \
  --sql "SELECT * FROM information_schema.processlist WHERE TIME > 30"
```

### Schema Investigation

```bash
# 1. List all tables
skill run mysql show-tables --database myapp

# 2. Describe table structure
skill run mysql describe-table --table users

# 3. Check table sizes
skill run mysql query --sql "
  SELECT table_name,
         ROUND((data_length + index_length) / 1024 / 1024, 2) AS size_mb
  FROM information_schema.tables
  WHERE table_schema = 'myapp'
  ORDER BY size_mb DESC
"
```

### Data Analysis

```bash
# Count records by status
skill run mysql query \
  --sql "SELECT status, COUNT(*) as count FROM orders GROUP BY status"

# Find recent activity
skill run mysql query \
  --sql "SELECT * FROM audit_log ORDER BY created_at DESC LIMIT 20" \
  --format json
```

## Security Considerations

- **Credentials**: Never store passwords in plain text; use skill config or env vars
- **Network**: Docker container runs in isolated network by default
- **Read-Only Users**: Create read-only database users for query-only access
- **Query Limits**: Use LIMIT clauses to prevent large result sets
- **Parameterized Queries**: Avoid SQL injection by not interpolating user input

## Troubleshooting

### Connection Refused

```
Error: Can't connect to MySQL server
```

**Solution**: Verify host, port, and network connectivity:

```bash
skill config mysql --set host=correct-hostname --set port=3306
```

### Access Denied

```
Error: Access denied for user
```

**Solution**: Verify credentials:

```bash
skill config mysql --set user=correct-user --set password=correct-password
```

### Timeout Error

```
Error: Query execution timeout
```

**Solution**: For long-running queries, consider:
1. Adding appropriate indexes
2. Breaking query into smaller chunks
3. Using LIMIT clauses

## Integration with Claude Code

```bash
# Natural language commands
"Show me all tables in the production database"
"How many users signed up this month?"
"What's the schema of the orders table?"
"Show active database connections"
```

## Next Steps

- [PostgreSQL Skill](./postgres.md) - PostgreSQL database access
- [MongoDB Skill](./mongodb.md) - MongoDB document store
- [Redis Skill](./redis.md) - Redis cache operations
- [MySQL Documentation](https://dev.mysql.com/doc/)
