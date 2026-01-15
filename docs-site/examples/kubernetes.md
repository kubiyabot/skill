# Kubernetes Skill

Manage Kubernetes clusters with native kubectl integration through a SKILL.md wrapper.

## Overview

The Kubernetes skill provides AI agents with safe, structured access to kubectl commands. Instead of running arbitrary kubectl commands, the skill exposes specific operations with validated parameters.

**Runtime**: Native (wraps `kubectl` CLI)  
**Source**: [examples/native-skills/kubernetes-skill](https://github.com/kubiyabot/skill/tree/main/examples/native-skills/kubernetes-skill)

## Installation

```bash
# Install kubectl (prerequisite)
# macOS
brew install kubectl

# Linux
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
chmod +x kubectl
sudo mv kubectl /usr/local/bin/

# Install the skill
skill install ./examples/native-skills/kubernetes-skill
```

## Tools Reference

### get

Get Kubernetes resources (pods, deployments, services, etc.)

**Parameters:**
- `resource` (string, required): Resource type (pods, deployments, services, nodes, etc.)
- `name` (string, optional): Specific resource name
- `namespace` (string, optional): Kubernetes namespace (default: current context)
- `all_namespaces` (boolean, optional): Query all namespaces

**Examples:**

```bash
# List all pods in current namespace
skill run kubernetes get --resource pods

# Get specific pod
skill run kubernetes get --resource pods --name nginx-123

# List pods in specific namespace
skill run kubernetes get --resource pods --namespace production

# List all pods across all namespaces
skill run kubernetes get --resource pods --all-namespaces

# List deployments
skill run kubernetes get --resource deployments

# List services
skill run kubernetes get --resource services
```

**Output:**
```json
{
  "items": [
    {
      "name": "nginx-deployment-7d64c8d9c9-8xk2p",
      "namespace": "default",
      "status": "Running",
      "restarts": 0,
      "age": "2d"
    }
  ]
}
```

### apply

Apply Kubernetes manifests from files or directories.

**Parameters:**
- `file` (string, required): Path to manifest file or directory
- `namespace` (string, optional): Target namespace
- `dry_run` (boolean, optional, default: false): Preview changes without applying

**Examples:**

```bash
# Apply single file
skill run kubernetes apply --file deployment.yaml

# Apply with namespace
skill run kubernetes apply --file app.yaml --namespace staging

# Dry run to preview changes
skill run kubernetes apply --file manifest.yaml --dry-run

# Apply entire directory
skill run kubernetes apply --file ./k8s/
```

### delete

Delete Kubernetes resources.

**Parameters:**
- `resource` (string, required): Resource type
- `name` (string, required): Resource name
- `namespace` (string, optional): Namespace
- `force` (boolean, optional, default: false): Force deletion

**Examples:**

```bash
# Delete pod
skill run kubernetes delete --resource pod --name nginx-123

# Delete deployment
skill run kubernetes delete --resource deployment --name myapp

# Force delete (skip grace period)
skill run kubernetes delete --resource pod --name stuck-pod --force
```

### logs

View pod logs.

**Parameters:**
- `pod` (string, required): Pod name
- `namespace` (string, optional): Namespace
- `container` (string, optional): Container name (for multi-container pods)
- `follow` (boolean, optional, default: false): Stream logs
- `tail` (number, optional): Number of lines from end (default: 100)
- `since` (string, optional): Time duration (e.g., "1h", "30m")

**Examples:**

```bash
# Get last 100 lines
skill run kubernetes logs --pod nginx-123

# Follow logs (streaming)
skill run kubernetes logs --pod myapp-456 --follow

# Last 50 lines
skill run kubernetes logs --pod api-789 --tail 50

# Logs from last hour
skill run kubernetes logs --pod worker-111 --since 1h

# Specific container in multi-container pod
skill run kubernetes logs --pod complex-pod --container sidecar
```

### exec

Execute commands in pods.

**Parameters:**
- `pod` (string, required): Pod name
- `namespace` (string, optional): Namespace
- `container` (string, optional): Container name
- `command` (string, required): Command to execute

**Examples:**

```bash
# Execute command
skill run kubernetes exec --pod nginx-123 --command "ls -la /app"

# Interactive shell
skill run kubernetes exec --pod debug-pod --command "/bin/sh"

# Check environment variables
skill run kubernetes exec --pod myapp --command "env"
```

### scale

Scale deployments or replica sets.

**Parameters:**
- `resource` (string, required): Resource type (deployment, replicaset, statefulset)
- `name` (string, required): Resource name
- `replicas` (number, required): Desired replica count
- `namespace` (string, optional): Namespace

**Examples:**

```bash
# Scale up
skill run kubernetes scale --resource deployment --name nginx --replicas 5

# Scale down
skill run kubernetes scale --resource deployment --name api --replicas 2

# Scale to zero
skill run kubernetes scale --resource deployment --name worker --replicas 0
```

### port-forward

Forward local ports to pods.

**Parameters:**
- `pod` (string, required): Pod name
- `local_port` (number, required): Local port
- `remote_port` (number, required): Pod port
- `namespace` (string, optional): Namespace

**Examples:**

```bash
# Forward to pod
skill run kubernetes port-forward --pod redis-123 --local-port 6379 --remote-port 6379

# Access service locally
skill run kubernetes port-forward --pod postgres-456 --local-port 5432 --remote-port 5432
```

### describe

Get detailed resource descriptions.

**Parameters:**
- `resource` (string, required): Resource type
- `name` (string, required): Resource name
- `namespace` (string, optional): Namespace

**Examples:**

```bash
# Describe pod
skill run kubernetes describe --resource pod --name nginx-123

# Describe deployment
skill run kubernetes describe --resource deployment --name myapp

# Describe node
skill run kubernetes describe --resource node --name worker-node-1
```

## Common Workflows

### Deployment Debugging

```bash
# 1. Check deployment status
skill run kubernetes get --resource deployments

# 2. Get pods for deployment
skill run kubernetes get --resource pods --selector app=myapp

# 3. Check pod details
skill run kubernetes describe --resource pod --name myapp-pod-123

# 4. View logs
skill run kubernetes logs --pod myapp-pod-123 --tail 200

# 5. Execute debug command
skill run kubernetes exec --pod myapp-pod-123 --command "curl localhost:8080/health"
```

### Rolling Update

```bash
# 1. Update deployment manifest
# (edit deployment.yaml)

# 2. Apply changes
skill run kubernetes apply --file deployment.yaml

# 3. Watch rollout
skill run kubernetes rollout status --resource deployment --name myapp

# 4. Verify new pods
skill run kubernetes get --resource pods --selector app=myapp
```

### Scaling Application

```bash
# 1. Check current replicas
skill run kubernetes get --resource deployment --name api

# 2. Scale up for traffic spike
skill run kubernetes scale --resource deployment --name api --replicas 10

# 3. Monitor pods coming up
skill run kubernetes get --resource pods --selector app=api

# 4. Scale back down
skill run kubernetes scale --resource deployment --name api --replicas 3
```

## Configuration

### Kubeconfig

The skill uses your local kubeconfig:

```bash
# Check current context
kubectl config current-context

# List contexts
kubectl config get-contexts

# Switch context
kubectl config use-context production

# Set namespace
kubectl config set-context --current --namespace=staging
```

### Skill Configuration

For Claude Code integration, configure in `.mcp.json`:

```json
{
  "mcpServers": {
    "skill-engine": {
      "command": "skill",
      "args": ["mcp"],
      "env": {
        "KUBECONFIG": "/Users/you/.kube/config",
        "KUBECTL_NAMESPACE": "default"
      }
    }
  }
}
```

## Security Considerations

### Allowed Operations

The skill only exposes specific kubectl operations with validated parameters. This prevents:
- Arbitrary command execution
- Namespace escalation
- Unintended resource deletion

### RBAC Integration

The skill respects your kubectl RBAC permissions:

```yaml
# Example: Read-only access
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: skill-reader
rules:
- apiGroups: [""]
  resources: ["pods", "services", "deployments"]
  verbs: ["get", "list"]
```

### Audit Logging

All skill executions are logged:

```bash
# View execution history
skill history | grep kubernetes

# Execution details
skill history show <execution-id>
```

## Troubleshooting

### kubectl not found

```bash
# Verify kubectl installation
which kubectl
kubectl version

# Add to PATH if needed
export PATH="/usr/local/bin:$PATH"
```

### Connection refused

```bash
# Check cluster connectivity
kubectl cluster-info

# Verify kubeconfig
kubectl config view

# Test connection
kubectl get nodes
```

### Permission denied

```bash
# Check your RBAC permissions
kubectl auth can-i get pods
kubectl auth can-i delete deployments

# Request additional permissions from cluster admin
```

## Related Resources

- **[Kubectl Documentation](https://kubernetes.io/docs/reference/kubectl/)** - Official kubectl reference
- **[SKILL.md Format](../../guides/manifest.md)** - How this skill is defined
- **[Native Skills Guide](../../guides/native-skills.md)** - Creating CLI wrappers
- **[Kubernetes Skill Source](https://github.com/kubiyabot/skill/tree/main/examples/native-skills/kubernetes-skill)** - View implementation

## Next Steps

- **[Create Custom Kubernetes Tools](../../guides/developing-skills.md)** - Add your own kubectl wrappers
- **[Helm Skill](./helm.md)** - Kubernetes package management
- **[Terraform Skill](./terraform.md)** - Infrastructure as Code
