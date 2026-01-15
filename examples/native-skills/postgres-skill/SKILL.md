# PostgreSQL Skill

PostgreSQL database operations with native psql CLI integration.

## Overview

This skill provides comprehensive PostgreSQL operations through the native psql CLI. It supports querying, schema inspection, administration, and backup/restore operations.

## Requirements

- **psql** must be installed and in PATH
- **pg_dump** and **pg_restore** for backup operations
- Database connection credentials

## Tools (18)

### Query Tools
- `query` - Execute SQL queries with format options
- `query-file` - Execute SQL from a file

### Schema Tools
- `databases` - List all databases
- `tables` - List tables in database
- `describe` - Describe table structure
- `columns` - List columns of a table
- `indexes` - List indexes on a table
- `constraints` - List constraints on a table
- `size` - Show database or table size

### Admin Tools
- `connections` - Show active database connections
- `running-queries` - Show currently running queries
- `cancel-query` - Cancel a running query by PID

### DDL Tools
- `create-database` - Create a new database
- `drop-database` - Drop a database (requires confirmation)
- `create-user` - Create a new database user/role
- `grant` - Grant privileges on database objects

### Backup Tools
- `backup` - Backup database using pg_dump
- `restore` - Restore database using pg_restore

## Security

- Passwords passed via PGPASSWORD environment variable
- DROP/TRUNCATE require explicit confirmation
- DELETE without WHERE is blocked
- SQL identifiers validated for injection prevention

## Configuration

Set password via environment: `export PGPASSWORD=your_password`
