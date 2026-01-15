# Google Cloud Platform (GCP) Skill

Manage Google Cloud resources with native gcloud CLI integration.

## Overview

The GCP skill provides AI agents with access to Google Cloud Platform services. Manage Compute Engine instances, Cloud Storage buckets, Cloud SQL databases, and more through the gcloud CLI.

**Runtime**: Native (wraps `gcloud` CLI)
**Source**: [examples/native-skills/gcp-skill](https://github.com/kubiyabot/skill/tree/main/examples/native-skills/gcp-skill)

## Installation

```bash
# Install gcloud CLI (prerequisite)
# macOS
brew install google-cloud-sdk

# Linux
curl https://sdk.cloud.google.com | bash

# Initialize gcloud
gcloud init

# Install the skill
skill install ./examples/native-skills/gcp-skill
```

## Configuration

Configure your GCP project:

```bash
skill config gcp \
  --set project=my-project-id \
  --set region=us-central1
```

Or use service account:

```bash
skill config gcp \
  --set credentials_file=/path/to/service-account.json
```

## Tools Reference

### compute-list

List Compute Engine instances.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `zone` | string | No | Filter by zone (default: all zones) |
| `filter` | string | No | Filter expression |
| `format` | string | No | Output: `json`, `table` (default: table) |

**Examples:**

```bash
# List all instances
skill run gcp compute-list

# Filter by zone
skill run gcp compute-list --zone us-central1-a

# Filter running instances
skill run gcp compute-list --filter "status=RUNNING"
```

**Output:**
```json
{
  "instances": [
    {
      "name": "web-server-1",
      "zone": "us-central1-a",
      "machineType": "e2-medium",
      "status": "RUNNING",
      "internalIp": "10.128.0.2",
      "externalIp": "34.123.45.67"
    }
  ]
}
```

### compute-start

Start a stopped Compute Engine instance.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instance` | string | Yes | Instance name |
| `zone` | string | Yes | Instance zone |

**Examples:**

```bash
skill run gcp compute-start \
  --instance web-server-1 \
  --zone us-central1-a
```

### compute-stop

Stop a running Compute Engine instance.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instance` | string | Yes | Instance name |
| `zone` | string | Yes | Instance zone |

**Examples:**

```bash
skill run gcp compute-stop \
  --instance web-server-1 \
  --zone us-central1-a
```

### compute-ssh

SSH into a Compute Engine instance.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instance` | string | Yes | Instance name |
| `zone` | string | Yes | Instance zone |
| `command` | string | No | Command to run (non-interactive) |

**Examples:**

```bash
# Run command on instance
skill run gcp compute-ssh \
  --instance web-server-1 \
  --zone us-central1-a \
  --command "uptime"
```

### storage-list

List Cloud Storage buckets or objects.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `bucket` | string | No | Bucket name (lists buckets if omitted) |
| `prefix` | string | No | Object prefix filter |

**Examples:**

```bash
# List all buckets
skill run gcp storage-list

# List objects in bucket
skill run gcp storage-list --bucket my-bucket

# Filter by prefix
skill run gcp storage-list --bucket my-bucket --prefix "logs/"
```

### storage-copy

Copy files to/from Cloud Storage.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `source` | string | Yes | Source path (local or gs://) |
| `destination` | string | Yes | Destination path |
| `recursive` | boolean | No | Copy directories recursively |

**Examples:**

```bash
# Upload file
skill run gcp storage-copy \
  --source ./backup.tar.gz \
  --destination gs://my-bucket/backups/

# Download file
skill run gcp storage-copy \
  --source gs://my-bucket/data.json \
  --destination ./local-data.json

# Upload directory
skill run gcp storage-copy \
  --source ./logs/ \
  --destination gs://my-bucket/logs/ \
  --recursive
```

### sql-list

List Cloud SQL instances.

**Examples:**

```bash
skill run gcp sql-list
```

**Output:**
```json
{
  "instances": [
    {
      "name": "production-db",
      "databaseVersion": "POSTGRES_14",
      "region": "us-central1",
      "state": "RUNNABLE",
      "tier": "db-custom-4-16384"
    }
  ]
}
```

### sql-connect

Connect to a Cloud SQL instance.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instance` | string | Yes | Instance name |
| `database` | string | No | Database name |
| `user` | string | No | Username (default: postgres/root) |

**Examples:**

```bash
skill run gcp sql-connect \
  --instance production-db \
  --database myapp \
  --user admin
```

### gke-list

List Google Kubernetes Engine clusters.

**Examples:**

```bash
skill run gcp gke-list
```

### gke-credentials

Get credentials for a GKE cluster.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `cluster` | string | Yes | Cluster name |
| `zone` | string | No | Cluster zone (or region) |
| `region` | string | No | Cluster region |

**Examples:**

```bash
skill run gcp gke-credentials \
  --cluster prod-cluster \
  --region us-central1
```

### functions-list

List Cloud Functions.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `region` | string | No | Filter by region |

**Examples:**

```bash
skill run gcp functions-list --region us-central1
```

### functions-logs

View Cloud Function logs.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `function` | string | Yes | Function name |
| `region` | string | Yes | Function region |
| `limit` | number | No | Number of log entries (default: 50) |

**Examples:**

```bash
skill run gcp functions-logs \
  --function my-function \
  --region us-central1 \
  --limit 100
```

## Common Workflows

### Deploy and Monitor Application

```bash
# 1. List running instances
skill run gcp compute-list --filter "status=RUNNING"

# 2. Upload new application
skill run gcp storage-copy \
  --source ./app.tar.gz \
  --destination gs://deployments/app-v2.tar.gz

# 3. SSH to deploy
skill run gcp compute-ssh \
  --instance web-server-1 \
  --zone us-central1-a \
  --command "cd /app && ./deploy.sh"
```

### Database Backup

```bash
# 1. List SQL instances
skill run gcp sql-list

# 2. Create backup (via export)
skill run gcp sql-export \
  --instance production-db \
  --database myapp \
  --destination gs://backups/db-backup.sql
```

### Kubernetes Cluster Access

```bash
# 1. List clusters
skill run gcp gke-list

# 2. Get credentials
skill run gcp gke-credentials \
  --cluster prod-cluster \
  --region us-central1

# 3. Use kubectl (credentials now configured)
kubectl get pods
```

## Security Considerations

- **Service Accounts**: Use dedicated service accounts with minimal permissions
- **IAM Roles**: Grant only required roles (Compute Viewer, Storage Object Admin, etc.)
- **Credential Rotation**: Rotate service account keys regularly
- **Audit Logging**: Enable Cloud Audit Logs for compliance

## Troubleshooting

### Authentication Failed

```
Error: Could not authenticate
```

**Solution**: Re-authenticate or check service account:

```bash
gcloud auth login
# or
gcloud auth activate-service-account --key-file=key.json
```

### Project Not Set

```
Error: Project not specified
```

**Solution**: Set default project:

```bash
skill config gcp --set project=my-project-id
```

### Permission Denied

```
Error: 403 Forbidden
```

**Solution**: Verify IAM permissions for the service account or user.

## Integration with Claude Code

```bash
# Natural language commands
"List all running GCP instances"
"Stop the web-server-1 instance"
"Upload backup.tar.gz to the backups bucket"
"Show me Cloud SQL instances"
```

## Next Steps

- [Azure Skill](./azure.md) - Microsoft Azure management
- [Kubernetes Skill](./kubernetes.md) - Container orchestration
- [Terraform Skill](./terraform.md) - Infrastructure as code
- [GCP Documentation](https://cloud.google.com/docs)
