---
name: mongodb
description: MongoDB database client with Docker-based mongosh CLI
runtime: docker
---

# MongoDB Skill

Query and manage MongoDB databases through a containerized mongosh client.

## Overview

This skill provides AI agents with MongoDB database access through a Docker-based mongosh (MongoDB Shell) client. Run queries, manage collections, perform aggregations, and analyze data.

## When to Use This Skill

- Execute MongoDB queries and aggregations
- Manage collections and indexes
- Insert, update, and delete documents
- Analyze data with aggregation pipelines
- Perform database administration tasks

## Prerequisites

- Docker installed and running
- Network access to target MongoDB server
- MongoDB user credentials with appropriate permissions

## Configuration

Add to your `.skill-engine.toml`:

```toml
[skills.mongodb]
source = "docker:mongo:7"
runtime = "docker"
description = "MongoDB database client"

[skills.mongodb.docker]
image = "mongo:7"
entrypoint = "mongosh"
network = "bridge"
memory = "256m"
rm = true
```

## Configuration Explained

| Setting | Value | Description |
|---------|-------|-------------|
| `image` | `mongo:7` | Official MongoDB 7 image with mongosh |
| `entrypoint` | `mongosh` | Use the MongoDB Shell client |
| `network` | `bridge` | Bridge network to connect to external databases |
| `memory` | `256m` | Memory limit for the container |
| `rm` | `true` | Auto-remove container after execution |

## Usage

Arguments after `--` are passed directly to mongosh:

```bash
skill run mongodb -- [mongosh options] [connection string] [script]
```

## Common Operations

### Connect and Run Query

```bash
# Connect to local MongoDB
skill run mongodb -- --eval "db.users.find().limit(5)" mongodb://localhost:27017/mydb

# Connect with authentication
skill run mongodb -- --eval "db.users.find()" "mongodb://user:password@localhost:27017/mydb"

# Connect to replica set
skill run mongodb -- --eval "db.users.countDocuments()" "mongodb://host1,host2,host3/mydb?replicaSet=rs0"
```

### List Databases and Collections

```bash
# List all databases
skill run mongodb -- --eval "db.adminCommand('listDatabases')" mongodb://localhost:27017

# List collections in a database
skill run mongodb -- --eval "db.getCollectionNames()" mongodb://localhost:27017/mydb

# Get collection stats
skill run mongodb -- --eval "db.users.stats()" mongodb://localhost:27017/mydb
```

### Query Documents

```bash
# Find all documents
skill run mongodb -- --eval "db.users.find().toArray()" mongodb://localhost:27017/mydb

# Find with filter
skill run mongodb -- --eval "db.users.find({status: 'active'}).toArray()" mongodb://localhost:27017/mydb

# Find with projection
skill run mongodb -- --eval "db.users.find({}, {name: 1, email: 1}).toArray()" mongodb://localhost:27017/mydb

# Find one document
skill run mongodb -- --eval "db.users.findOne({email: 'john@example.com'})" mongodb://localhost:27017/mydb

# Count documents
skill run mongodb -- --eval "db.users.countDocuments({status: 'active'})" mongodb://localhost:27017/mydb
```

### Insert Documents

```bash
# Insert one document
skill run mongodb -- --eval "db.users.insertOne({name: 'Alice', email: 'alice@example.com', status: 'active'})" mongodb://localhost:27017/mydb

# Insert many documents
skill run mongodb -- --eval "db.logs.insertMany([{event: 'login', user: 'alice'}, {event: 'logout', user: 'bob'}])" mongodb://localhost:27017/mydb
```

### Update Documents

```bash
# Update one document
skill run mongodb -- --eval "db.users.updateOne({email: 'alice@example.com'}, {\$set: {status: 'inactive'}})" mongodb://localhost:27017/mydb

# Update many documents
skill run mongodb -- --eval "db.users.updateMany({lastLogin: {\$lt: new Date('2024-01-01')}}, {\$set: {status: 'dormant'}})" mongodb://localhost:27017/mydb

# Upsert
skill run mongodb -- --eval "db.settings.updateOne({key: 'theme'}, {\$set: {value: 'dark'}}, {upsert: true})" mongodb://localhost:27017/mydb
```

### Delete Documents

```bash
# Delete one document
skill run mongodb -- --eval "db.sessions.deleteOne({_id: ObjectId('...')})" mongodb://localhost:27017/mydb

# Delete many documents
skill run mongodb -- --eval "db.logs.deleteMany({createdAt: {\$lt: new Date('2024-01-01')}})" mongodb://localhost:27017/mydb
```

### Aggregation Pipelines

```bash
# Group and count
skill run mongodb -- --eval "db.orders.aggregate([{\$match: {status: 'completed'}}, {\$group: {_id: '\$customer', total: {\$sum: '\$amount'}, count: {\$sum: 1}}}, {\$sort: {total: -1}}, {\$limit: 10}]).toArray()" mongodb://localhost:27017/mydb

# Date-based aggregation
skill run mongodb -- --eval "db.events.aggregate([{\$match: {type: 'purchase'}}, {\$group: {_id: {\$dateToString: {format: '%Y-%m-%d', date: '\$timestamp'}}, count: {\$sum: 1}, revenue: {\$sum: '\$amount'}}}, {\$sort: {_id: -1}}]).toArray()" mongodb://localhost:27017/mydb
```

### Index Management

```bash
# List indexes
skill run mongodb -- --eval "db.users.getIndexes()" mongodb://localhost:27017/mydb

# Create index
skill run mongodb -- --eval "db.users.createIndex({email: 1}, {unique: true})" mongodb://localhost:27017/mydb

# Create compound index
skill run mongodb -- --eval "db.orders.createIndex({customer: 1, createdAt: -1})" mongodb://localhost:27017/mydb

# Drop index
skill run mongodb -- --eval "db.users.dropIndex('email_1')" mongodb://localhost:27017/mydb
```

## Advanced Examples

### Connect to MongoDB Atlas

```bash
# Atlas connection
skill run mongodb -- --eval "db.users.find().limit(5).toArray()" "mongodb+srv://user:password@cluster.mongodb.net/mydb"
```

### Execute JavaScript File

```bash
# Run a script file (mount volume first)
skill run mongodb -- mongodb://localhost:27017/mydb /scripts/migration.js
```

### Output as JSON

```bash
# Use EJSON for strict JSON output
skill run mongodb -- --eval "EJSON.stringify(db.users.find().limit(5).toArray())" mongodb://localhost:27017/mydb
```

## Troubleshooting

### Connection Refused

```
MongoNetworkError: connect ECONNREFUSED
```

**Solutions:**
- Verify MongoDB server is running
- Check host and port are correct
- Ensure container can reach the database (network mode)

### Authentication Failed

```
MongoServerError: Authentication failed
```

**Solutions:**
- Verify username and password
- Check authentication database (usually `admin`)
- Ensure user has permissions for the database

### Command Not Found

```
SyntaxError: Unexpected token
```

**Solution:** Escape special characters in shell. Use single quotes for --eval.

## Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| ECONNREFUSED | Can't connect | Check host/port/firewall |
| Authentication failed | Bad credentials | Verify user/password |
| not authorized | Missing permissions | Check user roles |
| ns not found | Collection missing | Verify collection name |

## Docker Image Details

| Property | Value |
|----------|-------|
| Image | `mongo:7` |
| Size | ~700MB (compressed ~250MB) |
| Platforms | linux/amd64, linux/arm64 |
| Includes | mongosh, mongodump, mongorestore, mongoexport |

## Security Considerations

- Use connection strings with credentials (avoid plain text in scripts)
- Connect over TLS/SSL when possible
- Use read-only users for query operations
- Container runs with limited memory
- Network is bridge mode (isolated from host network)

## Resources

- [MongoDB Documentation](https://www.mongodb.com/docs/)
- [mongosh Reference](https://www.mongodb.com/docs/mongodb-shell/)
- [Docker MongoDB Image](https://hub.docker.com/_/mongo)
