# MySQL Skill

Docker-based MySQL client for database queries and administration.

## Quick Start

```bash
# Install the skill
skill install ./examples/docker-runtime-skills/mysql-skill

# Run a query
skill run mysql -- -h localhost -u myuser -e "SELECT * FROM users" mydb

# With password via environment
MYSQL_PASSWORD=secret skill run mysql -- -h myhost -u myuser -e "SHOW DATABASES"
```

## Configuration

Add to `.skill-engine.toml`:

```toml
[skills.mysql]
source = "docker:mysql:8"
runtime = "docker"

[skills.mysql.docker]
image = "mysql:8"
entrypoint = "mysql"
network = "bridge"
memory = "256m"
rm = true
environment = ["MYSQL_PWD=${MYSQL_PASSWORD:-}"]
```

## Common Commands

| Operation | Command |
|-----------|---------|
| List databases | `skill run mysql -- -h HOST -u USER -e "SHOW DATABASES"` |
| List tables | `skill run mysql -- -h HOST -u USER -e "SHOW TABLES" DB` |
| Run query | `skill run mysql -- -h HOST -u USER -e "SELECT * FROM table" DB` |
| Describe table | `skill run mysql -- -h HOST -u USER -e "DESCRIBE table" DB` |
| Server status | `skill run mysql -- -h HOST -u USER -e "SHOW STATUS"` |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `MYSQL_PASSWORD` | Database password (recommended over -p flag) |
| `MYSQL_HOST` | Default host if not specified |

## Security Notes

- Always use `MYSQL_PASSWORD` environment variable instead of `-p` flag
- Use SSL connections for remote databases (`--ssl-mode=REQUIRED`)
- Create read-only users for query-only operations

## Image Details

- **Image**: `mysql:8`
- **Size**: ~150MB compressed
- **Includes**: mysql, mysqldump, mysqladmin, mysqlimport
