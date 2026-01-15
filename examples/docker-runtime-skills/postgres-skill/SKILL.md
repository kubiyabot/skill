---
name: postgres
description: PostgreSQL CLI client (psql) for database operations running inside a Docker container
runtime: docker
---

# PostgreSQL Skill

Access PostgreSQL databases using the `psql` CLI client running inside a Docker container. Execute queries, manage databases, run scripts, export/import data, and perform database operations without installing PostgreSQL locally.

## Overview

This skill provides the `psql` command-line client through a containerized runtime. Connect to any PostgreSQL database (local or remote), execute SQL commands, run migration scripts, and manage your databases - all while keeping your host system clean.

## When to Use This Skill

**Use this skill when you need to**:
- Connect to PostgreSQL databases without local installation
- Execute SQL queries and commands
- Run database migration scripts
- Export and import data (CSV, SQL dumps)
- List databases, tables, and schemas
- Inspect table structures and relationships
- Execute administrative tasks
- Test database connections
- Run ad-hoc queries during development

## Prerequisites

### Docker

The PostgreSQL client runs inside a Docker container:

```bash
# macOS (using Homebrew)
brew install --cask docker

# Linux (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install docker.io
sudo systemctl start docker
```

### PostgreSQL Server

You need access to a PostgreSQL server:
- Local PostgreSQL instance
- Remote database server
- Cloud-hosted PostgreSQL (AWS RDS, Google Cloud SQL, Azure, etc.)
- Docker PostgreSQL container

## Configuration

Add to your `.skill-engine.toml`:

```toml
[skills.postgres]
source = "docker:postgres:16-alpine"
runtime = "docker"
description = "PostgreSQL CLI client"

[skills.postgres.docker]
image = "postgres:16-alpine"
entrypoint = "psql"
environment = ["PGPASSWORD=${PGPASSWORD:-}"]
network = "bridge"
memory = "256m"
rm = true
```

### Configuration Explained

- **image**: `postgres:16-alpine` - PostgreSQL 16.x Alpine image (~80MB)
- **entrypoint**: `psql` - PostgreSQL interactive terminal
- **environment**: Sets `PGPASSWORD` from host environment
- **network**: `bridge` - Allows connections to databases (local and remote)
- **memory**: `256m` - Limits container to 256MB RAM
- **rm**: `true` - Automatically removes container after execution

## Usage

PostgreSQL skill uses pass-through syntax - all arguments after `--` are passed directly to `psql`:

```bash
skill run postgres -- [psql arguments]
```

## Connection Methods

### Using Command-Line Arguments

```bash
# Basic connection
skill run postgres -- -h localhost -U postgres -d mydb

# With specific port
skill run postgres -- -h localhost -p 5432 -U myuser -d mydb

# Remote database
skill run postgres -- -h db.example.com -U admin -d production
```

### Using Environment Variables

```bash
# Set connection details
export PGHOST=localhost
export PGPORT=5432
export PGUSER=postgres
export PGDATABASE=mydb
export PGPASSWORD=secret

# Connect with defaults
skill run postgres --
```

### Using Connection URI

```bash
# PostgreSQL connection string
skill run postgres -- postgresql://user:password@host:5432/database

# With SSL
skill run postgres -- "postgresql://user:password@host:5432/database?sslmode=require"
```

## Common Operations

### Running Queries

```bash
# Execute single SQL command
skill run postgres -- -h localhost -U postgres -c "SELECT * FROM users;"

# Multiple commands
skill run postgres -- -h localhost -U postgres -c "SELECT count(*) FROM users; SELECT version();"

# Query with output formatting
skill run postgres -- -h localhost -U postgres -c "SELECT * FROM users;" -x  # Expanded display
```

### Database Management

```bash
# List all databases
skill run postgres -- -h localhost -U postgres -l

# Create database
skill run postgres -- -h localhost -U postgres -c "CREATE DATABASE newdb;"

# Drop database
skill run postgres -- -h localhost -U postgres -c "DROP DATABASE olddb;"

# Connect to specific database
skill run postgres -- -h localhost -U postgres -d mydb
```

### Table Operations

```bash
# List tables in current database
skill run postgres -- -h localhost -U postgres -d mydb -c "\dt"

# List tables with sizes
skill run postgres -- -h localhost -U postgres -d mydb -c "\dt+"

# Describe table structure
skill run postgres -- -h localhost -U postgres -d mydb -c "\d users"

# Show table constraints
skill run postgres -- -h localhost -U postgres -d mydb -c "\d+ users"
```

### Schema Operations

```bash
# List schemas
skill run postgres -- -h localhost -U postgres -d mydb -c "\dn"

# List tables in specific schema
skill run postgres -- -h localhost -U postgres -d mydb -c "\dt myschema.*"

# Set search path
skill run postgres -- -h localhost -U postgres -d mydb -c "SET search_path TO myschema,public;"
```

### Executing SQL Files

```bash
# Run SQL script
skill run postgres -- -h localhost -U postgres -d mydb -f migration.sql

# Run with error handling
skill run postgres -- -h localhost -U postgres -d mydb -f script.sql -v ON_ERROR_STOP=1

# Run multiple files
for file in migrations/*.sql; do
  skill run postgres -- -h localhost -U postgres -d mydb -f "$file"
done
```

### Data Export

```bash
# Export table to CSV
skill run postgres -- -h localhost -U postgres -d mydb -c "COPY users TO STDOUT CSV HEADER" > users.csv

# Export query results to CSV
skill run postgres -- -h localhost -U postgres -d mydb -c "COPY (SELECT * FROM users WHERE active=true) TO STDOUT CSV HEADER" > active_users.csv

# Export with custom delimiter
skill run postgres -- -h localhost -U postgres -d mydb -c "COPY users TO STDOUT CSV HEADER DELIMITER '|'" > users.csv

# Export as SQL INSERT statements
skill run postgres -- -h localhost -U postgres -d mydb -c "\copy users TO 'users.sql'"
```

### Data Import

```bash
# Import CSV into table
skill run postgres -- -h localhost -U postgres -d mydb -c "COPY users FROM STDIN CSV HEADER" < users.csv

# Import with specific columns
skill run postgres -- -h localhost -U postgres -d mydb -c "COPY users(name,email) FROM STDIN CSV HEADER" < partial_users.csv

# Import SQL file
skill run postgres -- -h localhost -U postgres -d mydb -f dump.sql
```

### User Management

```bash
# List users/roles
skill run postgres -- -h localhost -U postgres -c "\du"

# Create user
skill run postgres -- -h localhost -U postgres -c "CREATE USER newuser WITH PASSWORD 'password';"

# Grant privileges
skill run postgres -- -h localhost -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE mydb TO newuser;"

# Change password
skill run postgres -- -h localhost -U postgres -c "ALTER USER myuser WITH PASSWORD 'newpassword';"
```

### Indexes & Performance

```bash
# List indexes
skill run postgres -- -h localhost -U postgres -d mydb -c "\di"

# Create index
skill run postgres -- -h localhost -U postgres -d mydb -c "CREATE INDEX idx_email ON users(email);"

# Analyze query performance
skill run postgres -- -h localhost -U postgres -d mydb -c "EXPLAIN ANALYZE SELECT * FROM users WHERE email='test@example.com';"

# Vacuum database
skill run postgres -- -h localhost -U postgres -d mydb -c "VACUUM ANALYZE;"
```

## Meta-Commands

PostgreSQL `psql` supports special backslash commands:

### Information Commands

```bash
# \l - List databases
skill run postgres -- -h localhost -U postgres -c "\l"

# \dt - List tables
skill run postgres -- -h localhost -U postgres -d mydb -c "\dt"

# \dv - List views
skill run postgres -- -h localhost -U postgres -d mydb -c "\dv"

# \df - List functions
skill run postgres -- -h localhost -U postgres -d mydb -c "\df"

# \dn - List schemas
skill run postgres -- -h localhost -U postgres -d mydb -c "\dn"

# \du - List users/roles
skill run postgres -- -h localhost -U postgres -c "\du"

# \d table - Describe table
skill run postgres -- -h localhost -U postgres -d mydb -c "\d users"
```

### Output Formatting

```bash
# \x - Toggle expanded display (vertical)
skill run postgres -- -h localhost -U postgres -d mydb -c "\x" -c "SELECT * FROM users;"

# \t - Tuples only (no headers/footers)
skill run postgres -- -h localhost -U postgres -d mydb -c "\t" -c "SELECT name FROM users;"

# \a - Aligned/unaligned output
skill run postgres -- -h localhost -U postgres -d mydb -c "\a" -c "SELECT * FROM users;"
```

## Advanced Examples

### Database Backup

```bash
# Dump entire database
docker run --rm postgres:16-alpine pg_dump -h host -U user -d mydb > backup.sql

# Dump specific table
docker run --rm postgres:16-alpine pg_dump -h host -U user -d mydb -t users > users_backup.sql

# Dump schema only
docker run --rm postgres:16-alpine pg_dump -h host -U user -d mydb --schema-only > schema.sql

# Dump data only
docker run --rm postgres:16-alpine pg_dump -h host -U user -d mydb --data-only > data.sql
```

### Database Restore

```bash
# Restore from dump
skill run postgres -- -h localhost -U postgres -d mydb -f backup.sql

# Restore with progress
docker run --rm -i postgres:16-alpine pg_restore -h host -U user -d mydb -v < backup.dump
```

### Batch Operations

```bash
# Run multiple queries in transaction
skill run postgres -- -h localhost -U postgres -d mydb << EOF
BEGIN;
UPDATE users SET status='active' WHERE last_login > NOW() - INTERVAL '30 days';
DELETE FROM sessions WHERE expires_at < NOW();
COMMIT;
EOF
```

### Connection Testing

```bash
# Test connection
skill run postgres -- -h localhost -U postgres -c "SELECT version();"

# Check connection info
skill run postgres -- -h localhost -U postgres -c "SELECT current_database(), current_user, inet_server_addr(), inet_server_port();"

# Check server status
skill run postgres -- -h localhost -U postgres -c "SELECT pg_is_in_recovery();"
```

### Query Monitoring

```bash
# Show active queries
skill run postgres -- -h localhost -U postgres -c "SELECT pid, usename, query, state FROM pg_stat_activity WHERE state='active';"

# Show database sizes
skill run postgres -- -h localhost -U postgres -c "SELECT datname, pg_size_pretty(pg_database_size(datname)) FROM pg_database;"

# Show table sizes
skill run postgres -- -h localhost -U postgres -d mydb -c "SELECT schemaname, tablename, pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size FROM pg_tables ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC LIMIT 10;"
```

## Environment Variables

### PostgreSQL Standard Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `PGHOST` | Database host | `localhost` |
| `PGPORT` | Database port | `5432` |
| `PGUSER` | Username | `postgres` |
| `PGPASSWORD` | Password | `secret` |
| `PGDATABASE` | Database name | `mydb` |
| `PGSSLMODE` | SSL mode | `require`, `prefer`, `disable` |

### Usage

```bash
# Set environment variables
export PGHOST=localhost
export PGUSER=postgres
export PGPASSWORD=secret
export PGDATABASE=mydb

# Connect without arguments
skill run postgres --

# Or for single command
PGHOST=localhost PGPASSWORD=secret skill run postgres -- -U postgres -c "SELECT 1;"
```

## Security Best Practices

### Password Management

```bash
# Use environment variable (preferred)
export PGPASSWORD=your_password
skill run postgres -- -h host -U user -d db

# Use .pgpass file (on host)
# Create ~/.pgpass with: hostname:port:database:username:password
chmod 600 ~/.pgpass

# Use connection URI with password
skill run postgres -- "postgresql://user:pass@host:5432/db"
```

### SSL Connections

```bash
# Require SSL
skill run postgres -- "postgresql://user:pass@host:5432/db?sslmode=require"

# Verify CA
skill run postgres -- "postgresql://user:pass@host:5432/db?sslmode=verify-ca"

# Verify full certificate
skill run postgres -- "postgresql://user:pass@host:5432/db?sslmode=verify-full"
```

### Read-Only Access

```bash
# Start transaction in read-only mode
skill run postgres -- -h host -U user -d db -c "BEGIN READ ONLY; SELECT * FROM sensitive_table; COMMIT;"

# Grant read-only access
skill run postgres -- -h host -U postgres -c "GRANT SELECT ON ALL TABLES IN SCHEMA public TO readonly_user;"
```

## Troubleshooting

### Connection Refused

```bash
# Check if PostgreSQL is running
docker ps | grep postgres

# Check host is accessible
ping db-host

# Verify port
telnet localhost 5432
```

### Authentication Failed

```bash
# Verify credentials
skill run postgres -- -h localhost -U postgres -c "SELECT 1;"

# Check PGPASSWORD is set
echo $PGPASSWORD

# Try explicit password
skill run postgres -- -h localhost -U postgres -W  # Prompts for password
```

### Permission Denied

```bash
# Check user permissions
skill run postgres -- -h localhost -U postgres -c "\du"

# Grant necessary privileges
skill run postgres -- -h localhost -U postgres -c "GRANT ALL ON DATABASE mydb TO myuser;"
```

### Cannot Connect to Docker Network

```bash
# If connecting to host PostgreSQL from container
# Use host.docker.internal instead of localhost (macOS/Windows)
skill run postgres -- -h host.docker.internal -U postgres

# On Linux, use host network mode
# (requires custom config)
```

## Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| "connection refused" | PostgreSQL not running or wrong port | Check server status and port |
| "password authentication failed" | Wrong credentials | Verify username/password |
| "database does not exist" | Database not created | Use `-l` to list databases |
| "permission denied" | Insufficient privileges | Check user grants |
| "could not connect to server" | Network issue or wrong host | Verify host and network connectivity |

## PostgreSQL Client Tools

The container includes additional tools:

### pg_dump (Backup)

```bash
docker run --rm -e PGPASSWORD=secret postgres:16-alpine pg_dump -h localhost -U postgres mydb > backup.sql
```

### pg_restore (Restore)

```bash
docker run --rm -e PGPASSWORD=secret -i postgres:16-alpine pg_restore -h localhost -U postgres -d mydb < backup.dump
```

### createdb / dropdb

```bash
# Create database
docker run --rm -e PGPASSWORD=secret postgres:16-alpine createdb -h localhost -U postgres newdb

# Drop database
docker run --rm -e PGPASSWORD=secret postgres:16-alpine dropdb -h localhost -U postgres olddb
```

## Docker Image Details

- **Image**: `postgres:16-alpine`
- **Base**: Alpine Linux
- **Size**: ~80MB
- **PostgreSQL Version**: 16.x
- **Included Tools**: psql, pg_dump, pg_restore, createdb, dropdb, pg_isready
- **Platforms**: linux/amd64, linux/arm64

## Resources

- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [psql Command Reference](https://www.postgresql.org/docs/current/app-psql.html)
- [SQL Commands](https://www.postgresql.org/docs/current/sql-commands.html)
- [PostgreSQL Tutorial](https://www.postgresqltutorial.com/)
- [Connection Strings](https://www.postgresql.org/docs/current/libpq-connect.html#LIBPQ-CONNSTRING)

## Quick Reference

### Essential Commands Table

| Operation | Command |
|-----------|---------|
| Connect | `-h host -U user -d database` |
| Run query | `-c "SELECT * FROM table"` |
| Run script | `-f script.sql` |
| List databases | `-l` |
| List tables | `-c "\dt"` |
| Describe table | `-c "\d tablename"` |
| Export CSV | `-c "COPY table TO STDOUT CSV HEADER" > file.csv` |
| Import CSV | `-c "COPY table FROM STDIN CSV HEADER" < file.csv` |
| Backup database | `pg_dump -h host -U user db > backup.sql` |
| Restore database | `-f backup.sql` |
