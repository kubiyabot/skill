# Terraform Skill

Terraform infrastructure as code management with native CLI integration.

## Overview

This skill provides comprehensive Terraform operations through the native terraform CLI. It wraps terraform commands and returns them for host execution, enabling IaC management for cloud infrastructure.

## Requirements

- **terraform** must be installed and in PATH
- Appropriate cloud provider credentials configured (AWS, GCP, Azure, etc.)
- Valid Terraform configuration files in working directory

## Tools

### Core Workflow (8 tools)

#### init
Initialize a Terraform working directory.

**Parameters:**
- `backend` (boolean, optional): Configure backend (default: true)
- `backend_config` (string, optional): Backend configuration (key=value format)
- `upgrade` (boolean, optional): Upgrade modules and plugins
- `reconfigure` (boolean, optional): Reconfigure backend, ignoring saved configuration
- `migrate_state` (boolean, optional): Migrate state to new backend

**Example:**
```json
{"upgrade": true}
```

#### plan
Generate and show an execution plan.

**Parameters:**
- `out` (string, optional): Save plan to file
- `var` (string, optional): Variables (key=value,key2=value2 format)
- `var_file` (string, optional): Variable file path
- `target` (string, optional): Target specific resources (comma-separated)
- `destroy` (boolean, optional): Plan for destroy
- `refresh` (boolean, optional): Refresh state before planning (default: true)
- `detailed_exitcode` (boolean, optional): Return detailed exit codes

**Example:**
```json
{"out": "tfplan", "var": "environment=staging,region=us-west-2"}
```

#### apply
Apply changes to infrastructure.

**Parameters:**
- `plan_file` (string, optional): Apply a saved plan file
- `auto_approve` (boolean, optional): Skip interactive approval
- `var` (string, optional): Variables (key=value,key2=value2 format)
- `var_file` (string, optional): Variable file path
- `target` (string, optional): Target specific resources (comma-separated)
- `parallelism` (number, optional): Number of parallel operations
- `refresh` (boolean, optional): Refresh state before applying (default: true)

**Example:**
```json
{"plan_file": "tfplan", "auto_approve": true}
```

#### destroy
Destroy Terraform-managed infrastructure.

**Parameters:**
- `auto_approve` (boolean, optional): Skip interactive approval
- `var` (string, optional): Variables (key=value,key2=value2 format)
- `var_file` (string, optional): Variable file path
- `target` (string, optional): Target specific resources (comma-separated)
- `parallelism` (number, optional): Number of parallel operations

**Example:**
```json
{"target": "aws_instance.web", "auto_approve": true}
```

#### validate
Validate the Terraform configuration files.

**Parameters:**
- `json` (boolean, optional): Output in JSON format

**Example:**
```json
{"json": true}
```

#### fmt
Format Terraform configuration files.

**Parameters:**
- `check` (boolean, optional): Check if files are formatted (no changes)
- `diff` (boolean, optional): Display diff of changes
- `recursive` (boolean, optional): Process subdirectories
- `write` (boolean, optional): Write changes to files (default: true)

**Example:**
```json
{"check": true, "recursive": true}
```

#### output
Show output values from state.

**Parameters:**
- `name` (string, optional): Specific output to show
- `json` (boolean, optional): Output in JSON format
- `raw` (boolean, optional): Output raw value (for single output)
- `state` (string, optional): Path to state file

**Example:**
```json
{"name": "instance_ip", "raw": true}
```

#### show
Show current state or a saved plan.

**Parameters:**
- `plan_file` (string, optional): Show a saved plan file
- `json` (boolean, optional): Output in JSON format

**Example:**
```json
{"plan_file": "tfplan", "json": true}
```

### State Management (6 tools)

#### state-list
List resources in the state.

**Parameters:**
- `address` (string, optional): Filter by address pattern
- `state` (string, optional): Path to state file
- `id` (string, optional): Filter by resource ID

**Example:**
```json
{"address": "module.vpc"}
```

#### state-show
Show attributes of a single resource in the state.

**Parameters:**
- `address` (string, required): Resource address
- `state` (string, optional): Path to state file

**Example:**
```json
{"address": "aws_instance.web"}
```

#### state-mv
Move a resource in the state.

**Parameters:**
- `source` (string, required): Source resource address
- `destination` (string, required): Destination resource address
- `state` (string, optional): Path to state file
- `dry_run` (boolean, optional): Preview the move without making changes

**Example:**
```json
{
  "source": "aws_instance.old",
  "destination": "aws_instance.new",
  "dry_run": true
}
```

#### state-rm
Remove resources from the state.

**Parameters:**
- `address` (string, required): Resource addresses (comma-separated)
- `state` (string, optional): Path to state file
- `dry_run` (boolean, optional): Preview the removal without making changes

**Example:**
```json
{"address": "aws_instance.temp,aws_security_group.temp", "dry_run": true}
```

#### state-pull
Pull current state and output to stdout.

**Parameters:** None

#### state-push
Push local state to remote backend.

**Parameters:**
- `state_file` (string, required): Path to state file to push
- `force` (boolean, optional): Force push even with newer remote state

**Example:**
```json
{"state_file": "terraform.tfstate"}
```

### Resource Management (4 tools)

#### import
Import existing infrastructure into Terraform state.

**Parameters:**
- `address` (string, required): Resource address to import into
- `id` (string, required): Resource ID in the provider
- `var` (string, optional): Variables (key=value,key2=value2 format)
- `var_file` (string, optional): Variable file path
- `config` (string, optional): Path to Terraform configuration

**Example:**
```json
{"address": "aws_instance.web", "id": "i-1234567890abcdef0"}
```

#### refresh
Update local state file against real resources.

**Parameters:**
- `var` (string, optional): Variables (key=value,key2=value2 format)
- `var_file` (string, optional): Variable file path
- `target` (string, optional): Target specific resources (comma-separated)

**Example:**
```json
{"target": "aws_instance.web"}
```

#### taint
Mark a resource for recreation on next apply.

**Parameters:**
- `address` (string, required): Resource address to taint
- `state` (string, optional): Path to state file

**Example:**
```json
{"address": "aws_instance.web"}
```

#### untaint
Remove the taint from a resource.

**Parameters:**
- `address` (string, required): Resource address to untaint
- `state` (string, optional): Path to state file

**Example:**
```json
{"address": "aws_instance.web"}
```

### Workspace Management (2 tools)

#### workspace-list
List available workspaces.

**Parameters:** None

#### workspace-select
Select or create a workspace.

**Parameters:**
- `name` (string, required): Workspace name
- `create` (boolean, optional): Create workspace if it doesn't exist

**Examples:**
```json
// Select existing workspace
{"name": "production"}

// Create new workspace
{"name": "staging", "create": true}
```

## Security

This skill includes security considerations:

1. **Force Operations**: `force` flag for state-push is allowed but logged as a warning
2. **Destructive Operations**: `destroy` with `auto_approve` is allowed but logged as a warning
3. **Force-unlock**: Intentionally not exposed as it's too dangerous for automated use
4. **Non-interactive**: All commands include `-input=false` to prevent hanging

## Configuration

No configuration required. Terraform uses:
- Provider credentials from environment (AWS_ACCESS_KEY_ID, GOOGLE_CREDENTIALS, etc.)
- Backend configuration from Terraform files
- Workspace from working directory

Optional environment variable:
- `TERRAFORM_CMD`: Path to terraform executable (default: terraform)

## Workflow Examples

### Basic Infrastructure Deployment

```bash
# Initialize
skill run terraform-skill init

# Plan changes
skill run terraform-skill plan --out tfplan

# Apply changes
skill run terraform-skill apply --plan_file tfplan
```

### Multi-Environment with Workspaces

```bash
# Create staging workspace
skill run terraform-skill workspace-select --name staging --create true

# Plan with environment-specific variables
skill run terraform-skill plan --var_file staging.tfvars --out staging.tfplan

# Apply
skill run terraform-skill apply --plan_file staging.tfplan
```

### State Management

```bash
# List all resources
skill run terraform-skill state-list

# Move a resource
skill run terraform-skill state-mv --source module.old.aws_instance.web --destination aws_instance.web --dry_run true

# Import existing resource
skill run terraform-skill import --address aws_instance.imported --id i-1234567890abcdef0
```
