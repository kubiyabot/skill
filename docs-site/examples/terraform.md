# Terraform Skill

Infrastructure as Code management with native Terraform CLI integration.

## Overview

The Terraform skill provides comprehensive Terraform operations through the native `terraform` CLI. It enables AI agents to manage cloud infrastructure declaratively with safety and validation built-in.

**Runtime**: Native (wraps terraform CLI)
**Tools**: 15
**Use Cases**: Infrastructure provisioning, cloud automation, IaC workflows

## Installation

```bash
# Install from example
skill install ./examples/native-skills/terraform-skill

# Verify installation
skill list terraform
```

## Requirements

- Terraform 1.5+ installed and in PATH
- Cloud provider credentials configured (AWS, GCP, Azure, etc.)
- Valid Terraform configuration files in working directory

### Cloud Provider Setup

**AWS**:
```bash
export AWS_ACCESS_KEY_ID=your_key
export AWS_SECRET_ACCESS_KEY=your_secret
export AWS_REGION=us-west-2
```

**Google Cloud**:
```bash
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/key.json
export GOOGLE_PROJECT=your-project-id
```

**Azure**:
```bash
export ARM_CLIENT_ID=your_client_id
export ARM_CLIENT_SECRET=your_secret
export ARM_SUBSCRIPTION_ID=your_subscription
export ARM_TENANT_ID=your_tenant
```

## Quick Start

### Basic Workflow

```bash
# 1. Initialize Terraform
skill run terraform init

# 2. Preview changes
skill run terraform plan

# 3. Apply changes
skill run terraform apply auto_approve=false

# 4. Destroy (when needed)
skill run terraform destroy auto_approve=false
```

### With Variables

```bash
# Plan with variables
skill run terraform plan \
  var=environment=staging,region=us-west-2 \
  var_file=staging.tfvars

# Apply with variables
skill run terraform apply \
  var_file=staging.tfvars \
  auto_approve=true
```

## Core Tools

### init

Initialize a Terraform working directory.

**Parameters**:
- `backend` (boolean, optional): Configure backend (default: true)
- `backend_config` (string, optional): Backend configuration
- `upgrade` (boolean, optional): Upgrade modules and plugins
- `reconfigure` (boolean, optional): Reconfigure backend
- `migrate_state` (boolean, optional): Migrate state to new backend

**Examples**:

Basic initialization:
```bash
skill run terraform init
```

With backend reconfiguration:
```bash
skill run terraform init \
  reconfigure=true \
  backend_config=bucket=my-tf-state
```

Upgrade modules:
```bash
skill run terraform init upgrade=true
```

### plan

Generate and show an execution plan.

**Parameters**:
- `out` (string, optional): Save plan to file
- `var` (string, optional): Variables (key=value,key2=value2)
- `var_file` (string, optional): Variable file path
- `target` (string, optional): Target specific resources
- `destroy` (boolean, optional): Plan for destroy
- `refresh` (boolean, optional): Refresh state (default: true)
- `detailed_exitcode` (boolean, optional): Return detailed exit codes

**Examples**:

Preview all changes:
```bash
skill run terraform plan
```

Save plan for later:
```bash
skill run terraform plan out=production.tfplan
```

Plan with variables:
```bash
skill run terraform plan \
  var=environment=production,replicas=5 \
  var_file=prod.tfvars
```

Target specific resources:
```bash
skill run terraform plan target=aws_instance.web,aws_s3_bucket.data
```

### apply

Apply changes to infrastructure.

**Parameters**:
- `plan_file` (string, optional): Apply a saved plan file
- `auto_approve` (boolean, optional): Skip interactive approval
- `var` (string, optional): Variables
- `var_file` (string, optional): Variable file path
- `target` (string, optional): Target specific resources
- `parallelism` (number, optional): Number of parallel operations
- `refresh` (boolean, optional): Refresh state (default: true)

**Examples**:

Apply with confirmation:
```bash
skill run terraform apply
```

Apply saved plan:
```bash
skill run terraform apply plan_file=production.tfplan
```

Auto-approve (for CI/CD):
```bash
skill run terraform apply auto_approve=true
```

Target specific resources:
```bash
skill run terraform apply \
  target=aws_instance.web \
  auto_approve=true
```

### destroy

Destroy Terraform-managed infrastructure.

**Parameters**:
- `auto_approve` (boolean, optional): Skip interactive approval
- `var` (string, optional): Variables
- `var_file` (string, optional): Variable file path
- `target` (string, optional): Target specific resources
- `parallelism` (number, optional): Number of parallel operations

**Examples**:

Destroy with confirmation:
```bash
skill run terraform destroy
```

Destroy specific resource:
```bash
skill run terraform destroy \
  target=aws_instance.temp \
  auto_approve=true
```

### validate

Validate Terraform configuration files.

**Parameters**:
- `json` (boolean, optional): Output in JSON format

**Examples**:

Validate configuration:
```bash
skill run terraform validate
```

JSON output:
```bash
skill run terraform validate json=true
```

## State Management Tools

### state list

List resources in the state.

**Parameters**:
- `id` (string, optional): Filter by resource ID

**Example**:
```bash
skill run terraform state list
```

### state show

Show detailed state for a resource.

**Parameters**:
- `address` (string, required): Resource address

**Example**:
```bash
skill run terraform state show address=aws_instance.web
```

### state pull

Pull current state and output to stdout.

**Example**:
```bash
skill run terraform state pull
```

### state push

Push local state to remote backend.

**Parameters**:
- `force` (boolean, optional): Force push without locks

**Example**:
```bash
skill run terraform state push force=true
```

## Workspace Tools

### workspace list

List available workspaces.

**Example**:
```bash
skill run terraform workspace list
```

### workspace new

Create a new workspace.

**Parameters**:
- `name` (string, required): Workspace name

**Example**:
```bash
skill run terraform workspace new name=staging
```

### workspace select

Switch to a different workspace.

**Parameters**:
- `name` (string, required): Workspace name

**Example**:
```bash
skill run terraform workspace select name=production
```

### workspace delete

Delete a workspace.

**Parameters**:
- `name` (string, required): Workspace name
- `force` (boolean, optional): Force deletion

**Example**:
```bash
skill run terraform workspace delete name=staging force=true
```

## Utility Tools

### output

Read outputs from state file.

**Parameters**:
- `name` (string, optional): Specific output name
- `json` (boolean, optional): JSON format

**Examples**:

All outputs:
```bash
skill run terraform output
```

Specific output:
```bash
skill run terraform output name=instance_ip
```

JSON format:
```bash
skill run terraform output json=true
```

### import

Import existing infrastructure into Terraform.

**Parameters**:
- `address` (string, required): Resource address
- `id` (string, required): Provider-specific resource ID

**Example**:
```bash
skill run terraform import \
  address=aws_instance.web \
  id=i-abc123def456
```

### taint

Mark a resource for recreation.

**Parameters**:
- `address` (string, required): Resource address

**Example**:
```bash
skill run terraform taint address=aws_instance.web
```

### untaint

Remove taint from a resource.

**Parameters**:
- `address` (string, required): Resource address

**Example**:
```bash
skill run terraform untaint address=aws_instance.web
```

## Common Workflows

### Multi-Environment Deployment

```bash
# Development
skill run terraform workspace select name=dev
skill run terraform plan var_file=dev.tfvars
skill run terraform apply var_file=dev.tfvars auto_approve=true

# Staging
skill run terraform workspace select name=staging
skill run terraform plan var_file=staging.tfvars
skill run terraform apply var_file=staging.tfvars

# Production (with saved plan)
skill run terraform workspace select name=prod
skill run terraform plan var_file=prod.tfvars out=prod.tfplan
# Review prod.tfplan carefully
skill run terraform apply plan_file=prod.tfplan
```

### Safe Infrastructure Updates

```bash
# 1. Pull latest code
git pull origin main

# 2. Validate configuration
skill run terraform validate

# 3. Preview changes
skill run terraform plan out=changes.tfplan

# 4. Review plan
terraform show changes.tfplan

# 5. Apply if approved
skill run terraform apply plan_file=changes.tfplan

# 6. Verify outputs
skill run terraform output
```

### State Management

```bash
# List all resources
skill run terraform state list

# Inspect specific resource
skill run terraform state show address=aws_instance.web

# Pull state for backup
skill run terraform state pull > backup.tfstate

# Move resource in state
terraform state mv aws_instance.old aws_instance.new

# Remove resource from state (without destroying)
terraform state rm aws_instance.temp
```

### Workspace Management

```bash
# Create workspaces for environments
skill run terraform workspace new name=development
skill run terraform workspace new name=staging
skill run terraform workspace new name=production

# List workspaces
skill run terraform workspace list

# Switch between environments
skill run terraform workspace select name=staging

# Delete old workspace
skill run terraform workspace delete name=old-env
```

## Best Practices

### 1. Always Plan Before Apply

```bash
# Generate and review plan
skill run terraform plan out=changes.tfplan

# Review the plan file
terraform show changes.tfplan

# Apply only after review
skill run terraform apply plan_file=changes.tfplan
```

### 2. Use Remote State

```hcl
# backend.tf
terraform {
  backend "s3" {
    bucket = "my-terraform-state"
    key    = "production/terraform.tfstate"
    region = "us-west-2"
    encrypt = true
    dynamodb_table = "terraform-locks"
  }
}
```

Initialize with backend:
```bash
skill run terraform init backend_config=bucket=my-terraform-state
```

### 3. Use Variables Files

```bash
# dev.tfvars
environment = "development"
instance_type = "t3.micro"
replicas = 1

# prod.tfvars
environment = "production"
instance_type = "m5.large"
replicas = 3
```

Apply with variables:
```bash
skill run terraform apply var_file=prod.tfvars
```

### 4. Target Specific Resources When Needed

```bash
# Update only web servers
skill run terraform apply target=aws_instance.web

# Destroy only test resources
skill run terraform destroy target=aws_instance.test auto_approve=true
```

### 5. Validate Before Commit

```bash
# In CI/CD pipeline
skill run terraform fmt check=true
skill run terraform validate
skill run terraform plan
```

## Security Considerations

### State Files

- **Never commit state files to git**
- Use remote backends (S3, GCS, Azure Storage)
- Enable state encryption
- Use state locking (DynamoDB, etc.)

### Credentials

- **Never hardcode credentials**
- Use environment variables
- Use cloud provider IAM roles
- Rotate credentials regularly

### Approval Process

```bash
# Require manual approval for production
skill run terraform plan out=prod.tfplan
# Send prod.tfplan for review
# Apply only after approval
skill run terraform apply plan_file=prod.tfplan
```

## Troubleshooting

### "Error: Backend initialization required"

```bash
skill run terraform init reconfigure=true
```

### "Error: State lock held"

```bash
# Force unlock (use with caution)
terraform force-unlock LOCK_ID
```

### "Error: Resource already exists"

```bash
# Import existing resource
skill run terraform import \
  address=aws_instance.web \
  id=i-existingid
```

### "Error: Provider configuration changed"

```bash
skill run terraform init upgrade=true reconfigure=true
```

## Integration with Claude Code

Claude Code can use the Terraform skill for infrastructure management:

```
You: "Deploy the staging environment with terraform"

Claude: I'll deploy the staging environment using Terraform.
        [Uses terraform skill]

        1. Selecting staging workspace
        2. Running plan with staging.tfvars
        3. Applying changes...

        Deployment complete! Created:
        - 3 EC2 instances
        - 1 Load balancer
        - 2 RDS databases
```

## Related Documentation

- [Skill Development Guide](../guides/developing-skills.md) - Create custom skills
- [Security Model](../guides/advanced/security.md) - Security best practices
- [CLI Reference](../api/cli.md) - Command-line interface

## External Resources

- [Terraform Documentation](https://www.terraform.io/docs)
- [Terraform Best Practices](https://www.terraform-best-practices.com/)
- [Cloud Provider Docs](https://registry.terraform.io/browse/providers)
