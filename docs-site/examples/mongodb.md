# MongoDB Skill

Query and manage MongoDB databases through a containerized mongosh client.

## Overview

The MongoDB skill provides AI agents with MongoDB database access. Run queries, manage collections, perform aggregations, and analyze data through a secure Docker-based MongoDB shell.

**Runtime**: Docker (containerized `mongosh` client)
**Source**: [examples/docker-skills/mongodb-skill](https://github.com/kubiyabot/skill/tree/main/examples/docker-skills/mongodb-skill)

## Installation

```bash
# Install the skill
skill install github:kubiyabot/skill:mongodb

# Or from local directory
skill install ./examples/docker-skills/mongodb-skill
```

## Configuration

Configure your MongoDB connection:

```bash
skill config mongodb \
  --set uri="mongodb://localhost:27017" \
  --set database=mydb
```

Or via environment variables:

```bash
export MONGODB_URI="mongodb://user:password@localhost:27017"
export MONGODB_DATABASE=mydb
```

For MongoDB Atlas:

```bash
skill config mongodb \
  --set uri="mongodb+srv://user:password@cluster.mongodb.net" \
  --set database=mydb
```

## Tools Reference

### find

Query documents from a collection.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `collection` | string | Yes | Collection name |
| `filter` | json | No | Query filter (default: {}) |
| `projection` | json | No | Fields to include/exclude |
| `sort` | json | No | Sort order |
| `limit` | number | No | Maximum documents (default: 20) |
| `database` | string | No | Database name (overrides default) |

**Examples:**

```bash
# Find all documents
skill run mongodb find --collection users

# Find with filter
skill run mongodb find \
  --collection users \
  --filter '{"status": "active"}'

# Find with projection and sort
skill run mongodb find \
  --collection orders \
  --filter '{"total": {"$gt": 100}}' \
  --projection '{"customer": 1, "total": 1, "date": 1}' \
  --sort '{"date": -1}' \
  --limit 10
```

**Output:**
```json
{
  "documents": [
    {"_id": "65a...", "customer": "John", "total": 150, "date": "2025-01-13"},
    {"_id": "65b...", "customer": "Jane", "total": 120, "date": "2025-01-12"}
  ],
  "count": 2
}
```

### find-one

Find a single document.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `collection` | string | Yes | Collection name |
| `filter` | json | Yes | Query filter |
| `projection` | json | No | Fields to include/exclude |

**Examples:**

```bash
# Find by ID
skill run mongodb find-one \
  --collection users \
  --filter '{"_id": "65a1b2c3d4e5f6789"}'

# Find by unique field
skill run mongodb find-one \
  --collection users \
  --filter '{"email": "john@example.com"}'
```

### insert

Insert one or more documents.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `collection` | string | Yes | Collection name |
| `document` | json | Yes | Document(s) to insert |

**Examples:**

```bash
# Insert single document
skill run mongodb insert \
  --collection users \
  --document '{"name": "Alice", "email": "alice@example.com", "status": "active"}'

# Insert multiple documents
skill run mongodb insert \
  --collection logs \
  --document '[{"event": "login", "user": "alice"}, {"event": "logout", "user": "bob"}]'
```

**Output:**
```json
{
  "acknowledged": true,
  "insertedIds": ["65c1d2e3f4a5b6789"]
}
```

### update

Update documents in a collection.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `collection` | string | Yes | Collection name |
| `filter` | json | Yes | Query filter |
| `update` | json | Yes | Update operations |
| `multi` | boolean | No | Update all matching (default: false) |
| `upsert` | boolean | No | Insert if not found (default: false) |

**Examples:**

```bash
# Update single document
skill run mongodb update \
  --collection users \
  --filter '{"email": "alice@example.com"}' \
  --update '{"$set": {"status": "inactive"}}'

# Update multiple documents
skill run mongodb update \
  --collection users \
  --filter '{"lastLogin": {"$lt": "2024-01-01"}}' \
  --update '{"$set": {"status": "dormant"}}' \
  --multi

# Upsert operation
skill run mongodb update \
  --collection settings \
  --filter '{"key": "theme"}' \
  --update '{"$set": {"value": "dark"}}' \
  --upsert
```

**Output:**
```json
{
  "acknowledged": true,
  "matchedCount": 5,
  "modifiedCount": 5
}
```

### delete

Delete documents from a collection.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `collection` | string | Yes | Collection name |
| `filter` | json | Yes | Query filter |
| `multi` | boolean | No | Delete all matching (default: false) |

**Examples:**

```bash
# Delete single document
skill run mongodb delete \
  --collection sessions \
  --filter '{"_id": "65a1b2c3d4e5f6789"}'

# Delete multiple documents
skill run mongodb delete \
  --collection logs \
  --filter '{"createdAt": {"$lt": "2024-01-01"}}' \
  --multi
```

### aggregate

Run an aggregation pipeline.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `collection` | string | Yes | Collection name |
| `pipeline` | json | Yes | Aggregation pipeline stages |

**Examples:**

```bash
# Group and count
skill run mongodb aggregate \
  --collection orders \
  --pipeline '[
    {"$match": {"status": "completed"}},
    {"$group": {"_id": "$customer", "total": {"$sum": "$amount"}, "count": {"$sum": 1}}},
    {"$sort": {"total": -1}},
    {"$limit": 10}
  ]'

# Date-based aggregation
skill run mongodb aggregate \
  --collection events \
  --pipeline '[
    {"$match": {"type": "purchase"}},
    {"$group": {
      "_id": {"$dateToString": {"format": "%Y-%m-%d", "date": "$timestamp"}},
      "count": {"$sum": 1},
      "revenue": {"$sum": "$amount"}
    }},
    {"$sort": {"_id": -1}}
  ]'
```

### list-collections

List all collections in a database.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `database` | string | No | Database name |

**Examples:**

```bash
skill run mongodb list-collections
```

**Output:**
```json
{
  "collections": [
    {"name": "users", "type": "collection"},
    {"name": "orders", "type": "collection"},
    {"name": "system.views", "type": "collection"}
  ]
}
```

### stats

Get collection statistics.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `collection` | string | Yes | Collection name |

**Examples:**

```bash
skill run mongodb stats --collection users
```

**Output:**
```json
{
  "ns": "mydb.users",
  "count": 15420,
  "size": 4521984,
  "avgObjSize": 293,
  "storageSize": 2097152,
  "indexes": 3,
  "indexSize": 524288
}
```

### create-index

Create an index on a collection.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `collection` | string | Yes | Collection name |
| `keys` | json | Yes | Index keys |
| `options` | json | No | Index options (unique, sparse, etc.) |

**Examples:**

```bash
# Simple index
skill run mongodb create-index \
  --collection users \
  --keys '{"email": 1}'

# Unique index
skill run mongodb create-index \
  --collection users \
  --keys '{"username": 1}' \
  --options '{"unique": true}'

# Compound index
skill run mongodb create-index \
  --collection orders \
  --keys '{"customer": 1, "date": -1}'
```

## Common Workflows

### Data Exploration

```bash
# 1. List collections
skill run mongodb list-collections

# 2. Check collection stats
skill run mongodb stats --collection users

# 3. Sample documents
skill run mongodb find --collection users --limit 5

# 4. Analyze data distribution
skill run mongodb aggregate \
  --collection users \
  --pipeline '[{"$group": {"_id": "$status", "count": {"$sum": 1}}}]'
```

### Performance Analysis

```bash
# 1. Check collection size
skill run mongodb stats --collection orders

# 2. Analyze query patterns
skill run mongodb aggregate \
  --collection orders \
  --pipeline '[
    {"$group": {"_id": "$customer", "orders": {"$sum": 1}}},
    {"$match": {"orders": {"$gt": 100}}}
  ]'

# 3. Create index for common queries
skill run mongodb create-index \
  --collection orders \
  --keys '{"customer": 1, "createdAt": -1}'
```

### Data Cleanup

```bash
# 1. Find old records
skill run mongodb find \
  --collection logs \
  --filter '{"createdAt": {"$lt": "2024-01-01"}}' \
  --limit 5

# 2. Count records to delete
skill run mongodb aggregate \
  --collection logs \
  --pipeline '[
    {"$match": {"createdAt": {"$lt": "2024-01-01"}}},
    {"$count": "total"}
  ]'

# 3. Delete old records
skill run mongodb delete \
  --collection logs \
  --filter '{"createdAt": {"$lt": "2024-01-01"}}' \
  --multi
```

## Security Considerations

- **Connection Strings**: Never expose passwords in logs; use skill config
- **Network Isolation**: Docker container provides network isolation
- **Read-Only Users**: Create read-only database users for query operations
- **Query Limits**: Always use `--limit` to prevent large result sets
- **Filter Validation**: Validate filters to prevent injection attacks

## Troubleshooting

### Connection Failed

```
Error: Connection refused
```

**Solution**: Verify connection URI and network access:

```bash
skill config mongodb --set uri="mongodb://correct-host:27017"
```

### Authentication Failed

```
Error: Authentication failed
```

**Solution**: Verify credentials in connection URI:

```bash
skill config mongodb --set uri="mongodb://user:correct-password@host:27017"
```

### Query Timeout

```
Error: Operation exceeded time limit
```

**Solution**:
1. Add appropriate indexes for the query
2. Use more specific filters
3. Reduce limit size

## Integration with Claude Code

```bash
# Natural language commands
"Find all active users in MongoDB"
"Show me orders from the last week"
"What collections exist in the database?"
"Count documents by status in the orders collection"
```

## Next Steps

- [MySQL Skill](./mysql.md) - MySQL database access
- [PostgreSQL Skill](./postgres.md) - PostgreSQL queries
- [Redis Skill](./redis.md) - Redis cache operations
- [MongoDB Documentation](https://www.mongodb.com/docs/)
