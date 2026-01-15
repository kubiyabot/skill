# PostgreSQL Skill

PostgreSQL CLI client (psql) for database operations.

## Usage

```bash
# Connect to database
skill run postgres -- -h localhost -U postgres -d mydb

# Run SQL command
skill run postgres -- -h localhost -U postgres -c "SELECT * FROM users"

# Execute SQL file
skill run postgres -- -h localhost -U postgres -f queries.sql

# List databases
skill run postgres -- -h localhost -U postgres -l

# Describe tables
skill run postgres -- -h localhost -U postgres -d mydb -c "\dt"
```

## Configuration

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

## Security

- **Network**: `bridge` - Required for database connections
- **Memory**: 256MB limit
- **Password**: Use `PGPASSWORD` environment variable

## Environment Variables

| Variable | Description |
|----------|-------------|
| `PGPASSWORD` | PostgreSQL password |
| `PGHOST` | Default host |
| `PGPORT` | Default port (5432) |
| `PGUSER` | Default username |
| `PGDATABASE` | Default database |

## Common Operations

| Operation | Command |
|-----------|---------|
| Connect | `-h host -U user -d database` |
| Run query | `-c "SELECT * FROM table"` |
| Run file | `-f script.sql` |
| List DBs | `-l` |
| List tables | `-c "\dt"` |
| Describe | `-c "\d tablename"` |
| Export CSV | `-c "COPY table TO STDOUT CSV"` |
| Import CSV | `-c "COPY table FROM STDIN CSV"` |

## Docker Image

- **Image**: `postgres:16-alpine`
- **Size**: ~80MB
- **PostgreSQL**: 16.x
- **Includes**: psql, pg_dump, pg_restore
