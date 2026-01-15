---
name: azure
description: Microsoft Azure management with native az CLI integration
allowed-tools: Bash
---

# Azure Skill

Comprehensive Microsoft Azure management using the native az CLI.

## Overview

This skill provides AI agents with full access to Microsoft Azure resources through the az CLI. Manage virtual machines, storage accounts, SQL databases, AKS clusters, and more.

## Requirements

- Azure CLI installed (`az`)
- Authenticated with `az login` or service principal
- Subscription set with `az account set --subscription SUBSCRIPTION_ID`

## Configuration

Set your default subscription and resource group:

```bash
skill config azure --set subscription=YOUR_SUBSCRIPTION_ID --set resource_group=my-resource-group
```

Or use environment variables:

```bash
export AZURE_SUBSCRIPTION_ID=your-subscription-id
export AZURE_DEFAULTS_GROUP=my-resource-group
```

## Tools

### vm-list

List Azure Virtual Machines.

**Parameters:**
- `resource_group` (optional, string): Filter by resource group
- `show_details` (optional, boolean): Include detailed info (default: false)

**Examples:**
```bash
skill run azure vm-list
skill run azure vm-list --resource_group production
skill run azure vm-list --show_details
```

### vm-start

Start a stopped virtual machine.

**Parameters:**
- `name` (required, string): VM name
- `resource_group` (required, string): Resource group

**Examples:**
```bash
skill run azure vm-start --name web-vm-01 --resource_group production
```

### vm-stop

Stop (deallocate) a virtual machine.

**Parameters:**
- `name` (required, string): VM name
- `resource_group` (required, string): Resource group

**Examples:**
```bash
skill run azure vm-stop --name web-vm-01 --resource_group production
```

### vm-restart

Restart a virtual machine.

**Parameters:**
- `name` (required, string): VM name
- `resource_group` (required, string): Resource group

**Examples:**
```bash
skill run azure vm-restart --name web-vm-01 --resource_group production
```

### vm-run-command

Run a command on a VM.

**Parameters:**
- `name` (required, string): VM name
- `resource_group` (required, string): Resource group
- `command` (required, string): Shell command to run

**Examples:**
```bash
skill run azure vm-run-command --name web-vm-01 --resource_group production --command "systemctl status nginx"
```

### storage-list

List storage accounts.

**Parameters:**
- `resource_group` (optional, string): Filter by resource group

**Examples:**
```bash
skill run azure storage-list
skill run azure storage-list --resource_group production
```

### storage-blob-list

List blobs in a container.

**Parameters:**
- `account` (required, string): Storage account name
- `container` (required, string): Container name
- `prefix` (optional, string): Blob prefix filter

**Examples:**
```bash
skill run azure storage-blob-list --account mystorageaccount --container backups
skill run azure storage-blob-list --account mystorageaccount --container logs --prefix "2025/01/"
```

### storage-blob-upload

Upload a blob to storage.

**Parameters:**
- `account` (required, string): Storage account name
- `container` (required, string): Container name
- `source` (required, string): Local file path
- `name` (optional, string): Blob name (default: filename)

**Examples:**
```bash
skill run azure storage-blob-upload --account mystorageaccount --container backups --source ./backup.tar.gz
skill run azure storage-blob-upload --account mystorageaccount --container backups --source ./backup.tar.gz --name "db-backup-2025-01-14.tar.gz"
```

### storage-blob-download

Download a blob from storage.

**Parameters:**
- `account` (required, string): Storage account name
- `container` (required, string): Container name
- `name` (required, string): Blob name
- `destination` (required, string): Local file path

**Examples:**
```bash
skill run azure storage-blob-download --account mystorageaccount --container backups --name backup.tar.gz --destination ./local-backup.tar.gz
```

### sql-list

List Azure SQL servers.

**Parameters:**
- `resource_group` (optional, string): Filter by resource group

**Examples:**
```bash
skill run azure sql-list
skill run azure sql-list --resource_group production
```

### sql-db-list

List databases on a SQL server.

**Parameters:**
- `server` (required, string): SQL server name
- `resource_group` (required, string): Resource group

**Examples:**
```bash
skill run azure sql-db-list --server my-sql-server --resource_group production
```

### aks-list

List Azure Kubernetes Service clusters.

**Parameters:**
- `resource_group` (optional, string): Filter by resource group

**Examples:**
```bash
skill run azure aks-list
skill run azure aks-list --resource_group kubernetes
```

### aks-credentials

Get credentials for an AKS cluster (updates kubeconfig).

**Parameters:**
- `name` (required, string): Cluster name
- `resource_group` (required, string): Resource group
- `admin` (optional, boolean): Get admin credentials

**Examples:**
```bash
skill run azure aks-credentials --name prod-cluster --resource_group kubernetes
skill run azure aks-credentials --name prod-cluster --resource_group kubernetes --admin
```

### resource-list

List all resources in a subscription or resource group.

**Parameters:**
- `resource_group` (optional, string): Filter by resource group
- `type` (optional, string): Filter by resource type (e.g., Microsoft.Compute/virtualMachines)

**Examples:**
```bash
skill run azure resource-list
skill run azure resource-list --resource_group production
skill run azure resource-list --type "Microsoft.Compute/virtualMachines"
```

### resource-group-list

List resource groups.

**Examples:**
```bash
skill run azure resource-group-list
```

### monitor-metrics

Query Azure Monitor metrics.

**Parameters:**
- `resource` (required, string): Resource ID
- `metric` (required, string): Metric name
- `interval` (optional, string): Time interval (default: PT1H)
- `aggregation` (optional, string): Aggregation: Average, Total, Count, Minimum, Maximum

**Examples:**
```bash
skill run azure monitor-metrics --resource "/subscriptions/.../virtualMachines/web-vm-01" --metric "Percentage CPU" --aggregation Average
```

## Security Considerations

- Use service principals with minimal RBAC permissions
- Assign roles at resource group level, not subscription
- Use managed identities when running in Azure
- Store secrets in Azure Key Vault

## Troubleshooting

### Not Logged In
```
ERROR: Please run 'az login' to setup account.
```
**Solution:** Run `az login` or authenticate with service principal

### Subscription Not Set
```
ERROR: No subscription found.
```
**Solution:** Run `az account set --subscription YOUR_SUBSCRIPTION_ID`

### Permission Denied
```
ERROR: AuthorizationFailed
```
**Solution:** Verify the user/service principal has required RBAC roles
