# MongoDB Skill

Docker-based MongoDB client for database queries and administration.

## Quick Start

```bash
# Install the skill
skill install ./examples/docker-runtime-skills/mongodb-skill

# Run a query
skill run mongodb -- --eval "db.users.find().limit(5).toArray()" mongodb://localhost:27017/mydb

# With authentication
skill run mongodb -- --eval "db.users.countDocuments()" "mongodb://user:pass@localhost:27017/mydb"
```

## Configuration

Add to `.skill-engine.toml`:

```toml
[skills.mongodb]
source = "docker:mongo:7"
runtime = "docker"

[skills.mongodb.docker]
image = "mongo:7"
entrypoint = "mongosh"
network = "bridge"
memory = "256m"
rm = true
```

## Common Commands

| Operation | Command |
|-----------|---------|
| List databases | `skill run mongodb -- --eval "db.adminCommand('listDatabases')" URI` |
| List collections | `skill run mongodb -- --eval "db.getCollectionNames()" URI` |
| Find documents | `skill run mongodb -- --eval "db.coll.find().toArray()" URI` |
| Count documents | `skill run mongodb -- --eval "db.coll.countDocuments({})" URI` |
| Insert document | `skill run mongodb -- --eval "db.coll.insertOne({...})" URI` |
| Aggregate | `skill run mongodb -- --eval "db.coll.aggregate([...]).toArray()" URI` |

## Connection String Examples

```bash
# Local
mongodb://localhost:27017/mydb

# With auth
mongodb://user:password@localhost:27017/mydb

# Replica set
mongodb://host1,host2,host3/mydb?replicaSet=rs0

# MongoDB Atlas
mongodb+srv://user:password@cluster.mongodb.net/mydb
```

## Security Notes

- Use connection strings with credentials (not plain text passwords in shell)
- Enable TLS/SSL for remote connections
- Create read-only users for query-only operations

## Image Details

- **Image**: `mongo:7`
- **Size**: ~250MB compressed
- **Includes**: mongosh, mongodump, mongorestore, mongoexport, mongoimport
