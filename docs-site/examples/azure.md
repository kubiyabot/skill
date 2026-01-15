# Azure Skill

Manage Microsoft Azure resources with native Azure CLI integration.

## Overview

The Azure skill provides AI agents with access to Microsoft Azure services. Manage virtual machines, storage accounts, databases, and more through the Azure CLI.

**Runtime**: Native (wraps `az` CLI)
**Source**: [examples/native-skills/azure-skill](https://github.com/kubiyabot/skill/tree/main/examples/native-skills/azure-skill)

## Installation

```bash
# Install Azure CLI (prerequisite)
# macOS
brew install azure-cli

# Linux
curl -sL https://aka.ms/InstallAzureCLIDeb | sudo bash

# Login to Azure
az login

# Install the skill
skill install ./examples/native-skills/azure-skill
```

## Configuration

Configure your Azure subscription:

```bash
skill config azure \
  --set subscription=YOUR_SUBSCRIPTION_ID \
  --set resource_group=my-resource-group
```

Or use service principal:

```bash
skill config azure \
  --set client_id=YOUR_CLIENT_ID \
  --set client_secret=YOUR_SECRET \
  --set tenant_id=YOUR_TENANT_ID
```

## Tools Reference

### vm-list

List Azure Virtual Machines.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `resource_group` | string | No | Filter by resource group |
| `show_details` | boolean | No | Include detailed info (default: false) |

**Examples:**

```bash
# List all VMs
skill run azure vm-list

# Filter by resource group
skill run azure vm-list --resource_group production

# Show detailed info
skill run azure vm-list --show_details
```

**Output:**
```json
{
  "vms": [
    {
      "name": "web-vm-01",
      "resourceGroup": "production",
      "location": "eastus",
      "vmSize": "Standard_D2s_v3",
      "powerState": "VM running",
      "publicIp": "20.123.45.67"
    }
  ]
}
```

### vm-start

Start a stopped virtual machine.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | VM name |
| `resource_group` | string | Yes | Resource group |

**Examples:**

```bash
skill run azure vm-start \
  --name web-vm-01 \
  --resource_group production
```

### vm-stop

Stop (deallocate) a virtual machine.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | VM name |
| `resource_group` | string | Yes | Resource group |

**Examples:**

```bash
skill run azure vm-stop \
  --name web-vm-01 \
  --resource_group production
```

### vm-run-command

Run a command on a VM.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | VM name |
| `resource_group` | string | Yes | Resource group |
| `command` | string | Yes | Shell command to run |

**Examples:**

```bash
skill run azure vm-run-command \
  --name web-vm-01 \
  --resource_group production \
  --command "systemctl status nginx"
```

### storage-list

List storage accounts.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `resource_group` | string | No | Filter by resource group |

**Examples:**

```bash
# List all storage accounts
skill run azure storage-list

# Filter by resource group
skill run azure storage-list --resource_group production
```

### storage-blob-list

List blobs in a container.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `account` | string | Yes | Storage account name |
| `container` | string | Yes | Container name |
| `prefix` | string | No | Blob prefix filter |

**Examples:**

```bash
skill run azure storage-blob-list \
  --account mystorageaccount \
  --container backups

# With prefix filter
skill run azure storage-blob-list \
  --account mystorageaccount \
  --container logs \
  --prefix "2025/01/"
```

### storage-blob-upload

Upload a blob to storage.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `account` | string | Yes | Storage account name |
| `container` | string | Yes | Container name |
| `source` | string | Yes | Local file path |
| `name` | string | No | Blob name (default: filename) |

**Examples:**

```bash
skill run azure storage-blob-upload \
  --account mystorageaccount \
  --container backups \
  --source ./backup.tar.gz \
  --name "db-backup-2025-01-13.tar.gz"
```

### sql-list

List Azure SQL databases.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `server` | string | No | Filter by server name |
| `resource_group` | string | No | Filter by resource group |

**Examples:**

```bash
skill run azure sql-list

skill run azure sql-list --server my-sql-server
```

### aks-list

List Azure Kubernetes Service clusters.

**Examples:**

```bash
skill run azure aks-list
```

### aks-credentials

Get credentials for an AKS cluster.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Cluster name |
| `resource_group` | string | Yes | Resource group |

**Examples:**

```bash
skill run azure aks-credentials \
  --name prod-cluster \
  --resource_group kubernetes
```

### resource-list

List all resources in a subscription or resource group.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `resource_group` | string | No | Filter by resource group |
| `type` | string | No | Filter by resource type |

**Examples:**

```bash
# List all resources
skill run azure resource-list

# Filter by resource group
skill run azure resource-list --resource_group production

# Filter by type
skill run azure resource-list --type "Microsoft.Compute/virtualMachines"
```

### monitor-metrics

Query Azure Monitor metrics.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `resource` | string | Yes | Resource ID |
| `metric` | string | Yes | Metric name |
| `interval` | string | No | Time interval (default: PT1H) |
| `aggregation` | string | No | Aggregation: Average, Total, Count, etc. |

**Examples:**

```bash
skill run azure monitor-metrics \
  --resource "/subscriptions/.../virtualMachines/web-vm-01" \
  --metric "Percentage CPU" \
  --aggregation Average
```

## Common Workflows

### VM Maintenance

```bash
# 1. List VMs needing updates
skill run azure vm-list --resource_group production

# 2. Run update command
skill run azure vm-run-command \
  --name web-vm-01 \
  --resource_group production \
  --command "apt update && apt upgrade -y"

# 3. Restart if needed
skill run azure vm-restart \
  --name web-vm-01 \
  --resource_group production
```

### Backup to Storage

```bash
# 1. Create backup locally
tar -czf backup.tar.gz /data

# 2. Upload to Azure
skill run azure storage-blob-upload \
  --account backupstorage \
  --container daily-backups \
  --source ./backup.tar.gz

# 3. List backups
skill run azure storage-blob-list \
  --account backupstorage \
  --container daily-backups
```

### AKS Cluster Access

```bash
# 1. List clusters
skill run azure aks-list

# 2. Get credentials
skill run azure aks-credentials \
  --name prod-cluster \
  --resource_group kubernetes

# 3. Use kubectl
kubectl get nodes
```

## Security Considerations

- **Service Principals**: Use dedicated service principals with minimal permissions
- **RBAC**: Assign Azure RBAC roles at the resource group level
- **Managed Identity**: Use managed identities when running in Azure
- **Key Vault**: Store secrets in Azure Key Vault, not in config

## Troubleshooting

### Not Logged In

```
Error: Please run 'az login'
```

**Solution**: Authenticate with Azure:

```bash
az login
# or with service principal
az login --service-principal -u CLIENT_ID -p SECRET --tenant TENANT_ID
```

### Subscription Not Set

```
Error: No subscription set
```

**Solution**: Set default subscription:

```bash
az account set --subscription YOUR_SUBSCRIPTION_ID
```

### Permission Denied

```
Error: AuthorizationFailed
```

**Solution**: Verify the user/service principal has required RBAC roles.

## Integration with Claude Code

```bash
# Natural language commands
"List all running Azure VMs"
"Stop the web-vm-01 virtual machine"
"Upload backup file to Azure storage"
"Show me AKS clusters"
```

## Next Steps

- [GCP Skill](./gcp.md) - Google Cloud management
- [DigitalOcean Skill](./digitalocean.md) - DigitalOcean droplets
- [Kubernetes Skill](./kubernetes.md) - Container orchestration
- [Azure Documentation](https://docs.microsoft.com/azure)
