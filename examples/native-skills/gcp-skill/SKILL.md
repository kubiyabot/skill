---
name: gcp
description: Google Cloud Platform management with native gcloud CLI integration
allowed-tools: Bash
---

# GCP Skill

Comprehensive Google Cloud Platform management using the native gcloud CLI.

## Overview

This skill provides AI agents with full access to Google Cloud Platform resources through the gcloud CLI. Manage Compute Engine instances, Cloud Storage, Cloud SQL, GKE clusters, and more.

## Requirements

- Google Cloud SDK installed (`gcloud` CLI)
- Authenticated with `gcloud auth login` or service account
- Project configured with `gcloud config set project PROJECT_ID`

## Configuration

Set your default project and region:

```bash
skill config gcp --set project=my-project-id --set region=us-central1
```

Or use environment variables:

```bash
export CLOUDSDK_CORE_PROJECT=my-project-id
export CLOUDSDK_COMPUTE_REGION=us-central1
```

## Tools

### compute-list

List Compute Engine instances.

**Parameters:**
- `zone` (optional, string): Filter by zone (e.g., us-central1-a)
- `filter` (optional, string): Filter expression (e.g., "status=RUNNING")
- `format` (optional, string): Output format: json, table, yaml (default: json)

**Examples:**
```bash
skill run gcp compute-list
skill run gcp compute-list --zone us-central1-a
skill run gcp compute-list --filter "status=RUNNING"
```

### compute-start

Start a stopped Compute Engine instance.

**Parameters:**
- `instance` (required, string): Instance name
- `zone` (required, string): Instance zone

**Examples:**
```bash
skill run gcp compute-start --instance web-server-1 --zone us-central1-a
```

### compute-stop

Stop a running Compute Engine instance.

**Parameters:**
- `instance` (required, string): Instance name
- `zone` (required, string): Instance zone

**Examples:**
```bash
skill run gcp compute-stop --instance web-server-1 --zone us-central1-a
```

### compute-ssh

Run a command on a Compute Engine instance via SSH.

**Parameters:**
- `instance` (required, string): Instance name
- `zone` (required, string): Instance zone
- `command` (required, string): Command to execute

**Examples:**
```bash
skill run gcp compute-ssh --instance web-server-1 --zone us-central1-a --command "uptime"
```

### storage-list

List Cloud Storage buckets or objects.

**Parameters:**
- `bucket` (optional, string): Bucket name (lists buckets if omitted)
- `prefix` (optional, string): Object prefix filter

**Examples:**
```bash
skill run gcp storage-list
skill run gcp storage-list --bucket my-bucket
skill run gcp storage-list --bucket my-bucket --prefix "logs/"
```

### storage-copy

Copy files to/from Cloud Storage.

**Parameters:**
- `source` (required, string): Source path (local or gs://)
- `destination` (required, string): Destination path
- `recursive` (optional, boolean): Copy directories recursively

**Examples:**
```bash
skill run gcp storage-copy --source ./backup.tar.gz --destination gs://my-bucket/backups/
skill run gcp storage-copy --source gs://my-bucket/data.json --destination ./local-data.json
skill run gcp storage-copy --source ./logs/ --destination gs://my-bucket/logs/ --recursive
```

### sql-list

List Cloud SQL instances.

**Parameters:**
- `format` (optional, string): Output format: json, table (default: json)

**Examples:**
```bash
skill run gcp sql-list
```

### sql-databases

List databases in a Cloud SQL instance.

**Parameters:**
- `instance` (required, string): Cloud SQL instance name

**Examples:**
```bash
skill run gcp sql-databases --instance production-db
```

### gke-list

List Google Kubernetes Engine clusters.

**Parameters:**
- `region` (optional, string): Filter by region
- `format` (optional, string): Output format: json, table (default: json)

**Examples:**
```bash
skill run gcp gke-list
skill run gcp gke-list --region us-central1
```

### gke-credentials

Get credentials for a GKE cluster (updates kubeconfig).

**Parameters:**
- `cluster` (required, string): Cluster name
- `zone` (optional, string): Cluster zone (for zonal clusters)
- `region` (optional, string): Cluster region (for regional clusters)

**Examples:**
```bash
skill run gcp gke-credentials --cluster prod-cluster --region us-central1
skill run gcp gke-credentials --cluster dev-cluster --zone us-central1-a
```

### functions-list

List Cloud Functions.

**Parameters:**
- `region` (optional, string): Filter by region

**Examples:**
```bash
skill run gcp functions-list
skill run gcp functions-list --region us-central1
```

### functions-logs

View Cloud Function logs.

**Parameters:**
- `function` (required, string): Function name
- `region` (required, string): Function region
- `limit` (optional, number): Number of log entries (default: 50)

**Examples:**
```bash
skill run gcp functions-logs --function my-function --region us-central1
skill run gcp functions-logs --function my-function --region us-central1 --limit 100
```

### projects-list

List accessible GCP projects.

**Examples:**
```bash
skill run gcp projects-list
```

### iam-list

List IAM policy bindings for a project.

**Parameters:**
- `project` (optional, string): Project ID (uses default if not specified)

**Examples:**
```bash
skill run gcp iam-list
skill run gcp iam-list --project my-project
```

## Security Considerations

- Use service accounts with minimal required permissions
- Enable Cloud Audit Logs for compliance
- Use VPC Service Controls for sensitive data
- Rotate service account keys regularly

## Troubleshooting

### Authentication Failed
```
ERROR: (gcloud.auth) You do not currently have an active account selected.
```
**Solution:** Run `gcloud auth login` or `gcloud auth activate-service-account`

### Project Not Set
```
ERROR: (gcloud) The project property is set to the empty string
```
**Solution:** Run `gcloud config set project PROJECT_ID`

### Permission Denied
```
ERROR: (gcloud.compute.instances.list) Required permission
```
**Solution:** Ensure the account has the required IAM roles
