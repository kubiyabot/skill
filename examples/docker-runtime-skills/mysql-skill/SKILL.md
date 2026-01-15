---
name: mysql
description: MySQL database client with Docker-based mysql CLI
runtime: docker
---

# MySQL Skill

Execute MySQL queries and manage databases through a containerized MySQL client.

## Overview

This skill provides AI agents with MySQL database access through a Docker-based mysql CLI client. Run queries, manage schemas, export data, and monitor database health.

## When to Use This Skill

- Execute SQL queries against MySQL databases
- Manage database schemas and tables
- Export query results
- Monitor database connections and status
- Perform database administration tasks

## Prerequisites

- Docker installed and running
- Network access to target MySQL server
- MySQL user credentials with appropriate permissions

## Configuration

Add to your `.skill-engine.toml`:

```toml
[skills.mysql]
source = "docker:mysql:8"
runtime = "docker"
description = "MySQL database client"

[skills.mysql.docker]
image = "mysql:8"
entrypoint = "mysql"
network = "bridge"
memory = "256m"
rm = true
environment = [
  "MYSQL_PWD=${MYSQL_PASSWORD}"
]
```

## Configuration Explained

| Setting | Value | Description |
|---------|-------|-------------|
| `image` | `mysql:8` | Official MySQL 8 image with mysql client |
| `entrypoint` | `mysql` | Use the mysql command-line client |
| `network` | `bridge` | Bridge network to connect to external databases |
| `memory` | `256m` | Memory limit for the container |
| `rm` | `true` | Auto-remove container after execution |

## Usage

Arguments after `--` are passed directly to the mysql client:

```bash
skill run mysql -- [mysql options]
```

## Common Operations

### Connect and Run Query

```bash
# Simple query
skill run mysql -- -h localhost -u myuser -e "SELECT * FROM users LIMIT 10" mydb

# With explicit password (not recommended - use env var)
skill run mysql -- -h localhost -u myuser -p'password' -e "SHOW DATABASES"

# Using environment variable for password
MYSQL_PASSWORD=secret skill run mysql -- -h localhost -u myuser -e "SHOW TABLES" mydb
```

### List Databases and Tables

```bash
# List all databases
skill run mysql -- -h localhost -u root -e "SHOW DATABASES"

# List tables in a database
skill run mysql -- -h localhost -u root -e "SHOW TABLES" mydb

# Describe table structure
skill run mysql -- -h localhost -u root -e "DESCRIBE users" mydb
```

### Data Queries

```bash
# Select with conditions
skill run mysql -- -h localhost -u myuser -e "SELECT id, name, email FROM users WHERE active = 1" mydb

# Aggregate query
skill run mysql -- -h localhost -u myuser -e "SELECT status, COUNT(*) as count FROM orders GROUP BY status" mydb

# Join query
skill run mysql -- -h localhost -u myuser -e "SELECT u.name, COUNT(o.id) FROM users u LEFT JOIN orders o ON u.id = o.user_id GROUP BY u.id" mydb
```

### Data Modification

```bash
# Insert data
skill run mysql -- -h localhost -u myuser -e "INSERT INTO users (name, email) VALUES ('John', 'john@example.com')" mydb

# Update data
skill run mysql -- -h localhost -u myuser -e "UPDATE users SET active = 0 WHERE last_login < '2024-01-01'" mydb

# Delete data
skill run mysql -- -h localhost -u myuser -e "DELETE FROM sessions WHERE expired_at < NOW()" mydb
```

### Output Formats

```bash
# Tab-separated (default)
skill run mysql -- -h localhost -u myuser -e "SELECT * FROM users" mydb

# Vertical format (like \G)
skill run mysql -- -h localhost -u myuser -E -e "SELECT * FROM users LIMIT 1" mydb

# HTML output
skill run mysql -- -h localhost -u myuser -H -e "SELECT * FROM users" mydb

# XML output
skill run mysql -- -h localhost -u myuser -X -e "SELECT * FROM users" mydb

# Skip column names
skill run mysql -- -h localhost -u myuser -N -e "SELECT COUNT(*) FROM users" mydb
```

### Server Information

```bash
# Show server status
skill run mysql -- -h localhost -u root -e "SHOW STATUS"

# Show process list
skill run mysql -- -h localhost -u root -e "SHOW PROCESSLIST"

# Show variables
skill run mysql -- -h localhost -u root -e "SHOW VARIABLES LIKE 'max_connections'"

# Show grants
skill run mysql -- -h localhost -u root -e "SHOW GRANTS FOR 'myuser'@'%'"
```

### Schema Management

```bash
# Create database
skill run mysql -- -h localhost -u root -e "CREATE DATABASE newdb CHARACTER SET utf8mb4"

# Create table
skill run mysql -- -h localhost -u root -e "CREATE TABLE logs (id INT AUTO_INCREMENT PRIMARY KEY, message TEXT, created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)" newdb

# Add index
skill run mysql -- -h localhost -u root -e "CREATE INDEX idx_created ON logs(created_at)" newdb

# Alter table
skill run mysql -- -h localhost -u root -e "ALTER TABLE users ADD COLUMN phone VARCHAR(20)" mydb
```

## Advanced Examples

### Execute SQL File

```bash
# Mount local file and execute
skill run mysql -- -h localhost -u root < /path/to/script.sql mydb
```

### Connect to Remote Database

```bash
# Connect to RDS
skill run mysql -- -h mydb.abc123.us-east-1.rds.amazonaws.com -P 3306 -u admin mydb

# Connect with SSL
skill run mysql -- -h secure-db.example.com --ssl-mode=REQUIRED -u myuser mydb
```

### Batch Operations

```bash
# Multiple statements
skill run mysql -- -h localhost -u root -e "
  START TRANSACTION;
  INSERT INTO audit_log (action) VALUES ('batch_start');
  UPDATE users SET processed = 1 WHERE processed = 0 LIMIT 100;
  INSERT INTO audit_log (action) VALUES ('batch_end');
  COMMIT;
" mydb
```

## Troubleshooting

### Connection Refused

```
ERROR 2003 (HY000): Can't connect to MySQL server
```

**Solutions:**
- Verify the MySQL server is running
- Check host and port are correct
- Ensure network connectivity (container to host)

### Access Denied

```
ERROR 1045 (28000): Access denied for user
```

**Solutions:**
- Verify username and password
- Check user has permissions from container's IP
- Use `MYSQL_PASSWORD` environment variable

### Unknown Database

```
ERROR 1049 (42000): Unknown database 'mydb'
```

**Solution:** Verify database name exists: `SHOW DATABASES`

## Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| 2003 | Can't connect | Check host/port/firewall |
| 1045 | Access denied | Verify credentials |
| 1049 | Unknown database | Check database name |
| 1146 | Table doesn't exist | Check table name |
| 2013 | Lost connection | Check query timeout |

## Docker Image Details

| Property | Value |
|----------|-------|
| Image | `mysql:8` |
| Size | ~400MB (compressed ~150MB) |
| Platforms | linux/amd64, linux/arm64 |
| Includes | mysql, mysqldump, mysqlimport, mysqladmin |

## Security Considerations

- Use environment variables for passwords (MYSQL_PWD)
- Connect over SSL when possible
- Use read-only users for query operations
- Container runs with limited memory
- Network is bridge mode (isolated)

## Resources

- [MySQL Documentation](https://dev.mysql.com/doc/)
- [MySQL CLI Reference](https://dev.mysql.com/doc/refman/8.0/en/mysql.html)
- [Docker MySQL Image](https://hub.docker.com/_/mysql)
