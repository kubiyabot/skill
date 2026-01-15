---
name: kubernetes
description: Kubernetes cluster management with real kubectl integration. Use when you need to manage pods, deployments, services, or any K8s resources.
allowed-tools: Bash, skill-run
---

# Kubernetes Skill

Comprehensive Kubernetes cluster management through native kubectl execution. This skill provides 18+ kubectl tools for managing your clusters.

## When to Use

- Managing Kubernetes resources (pods, deployments, services, configmaps, secrets)
- Viewing pod logs and debugging containers
- Scaling deployments up or down
- Managing cluster configuration and contexts
- Performing rollout operations (restart, undo, status)
- Node management (cordon, uncordon, drain, taint)
- Creating and deleting resources

## Requirements

- `kubectl` must be installed and in PATH
- Valid kubeconfig (default: `~/.kube/config` or `KUBECONFIG` env var)
- Appropriate cluster access permissions

## Tools Provided

### get
Get Kubernetes resources (pods, services, deployments, nodes, etc.)

**Parameters**:
- `resource` (required): Resource type (pods, services, deployments, nodes, namespaces, configmaps, secrets, ingress, pv, pvc, jobs, cronjobs, daemonsets, statefulsets, replicasets, events, endpoints, all)
- `name` (optional): Specific resource name
- `namespace` (optional): Kubernetes namespace (default: default, use 'all' for all namespaces)
- `selector` (optional): Label selector (e.g., app=nginx)
- `output` (optional): Output format (wide, yaml, json, name)

**Example**:
```bash
skill run kubernetes get resource=pods namespace=kube-system
skill run kubernetes get resource=deployments output=wide
skill run kubernetes get resource=nodes
```

### describe
Show detailed information about a resource.

**Parameters**:
- `resource` (required): Resource type (pod, service, deployment, node, etc.)
- `name` (required): Resource name
- `namespace` (optional): Kubernetes namespace

**Example**:
```bash
skill run kubernetes describe resource=pod name=nginx-xxxxx namespace=default
```

### logs
Get logs from a pod.

**Parameters**:
- `pod` (required): Pod name
- `namespace` (optional): Kubernetes namespace
- `container` (optional): Container name (for multi-container pods)
- `tail` (optional): Number of lines to show from end
- `follow` (optional): Stream logs in real-time
- `previous` (optional): Show logs from previous container instance

**Example**:
```bash
skill run kubernetes logs pod=nginx-xxxxx namespace=default tail=100
```

### exec
Execute a command in a container.

**Parameters**:
- `pod` (required): Pod name
- `namespace` (optional): Kubernetes namespace
- `container` (optional): Container name
- `command` (required): Command to execute

**Example**:
```bash
skill run kubernetes exec pod=nginx-xxxxx command="ls -la /var/log"
```

### apply
Apply a configuration from YAML/JSON content.

**Parameters**:
- `content` (required): YAML or JSON content to apply
- `namespace` (optional): Kubernetes namespace
- `dry_run` (optional): Run in dry-run mode

**Example**:
```bash
skill run kubernetes apply content="apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test-config\ndata:\n  key: value"
```

### delete
Delete Kubernetes resources.

**Parameters**:
- `resource` (required): Resource type
- `name` (required): Resource name
- `namespace` (optional): Kubernetes namespace
- `force` (optional): Force deletion
- `grace_period` (optional): Grace period in seconds

**Example**:
```bash
skill run kubernetes delete resource=pod name=nginx-xxxxx namespace=default
skill run kubernetes delete resource=namespace name=test-namespace
```

### scale
Scale a deployment, replicaset, or statefulset.

**Parameters**:
- `resource` (required): Resource type (deployment, replicaset, statefulset)
- `name` (required): Resource name
- `namespace` (optional): Kubernetes namespace
- `replicas` (required): Desired replica count

**Example**:
```bash
skill run kubernetes scale resource=deployment name=nginx namespace=default replicas=3
```

### rollout
Manage rollouts for deployments.

**Parameters**:
- `resource` (required): Resource type (deployment, daemonset, statefulset)
- `name` (required): Resource name
- `namespace` (optional): Kubernetes namespace
- `action` (required): Rollout action (status, history, undo, restart, pause, resume)

**Example**:
```bash
skill run kubernetes rollout resource=deployment name=nginx action=status
skill run kubernetes rollout resource=deployment name=nginx action=restart
```

### top
Display resource usage (CPU/memory).

**Parameters**:
- `resource` (required): Resource type (pods, nodes)
- `namespace` (optional): Kubernetes namespace
- `containers` (optional): Show container-level metrics

**Example**:
```bash
skill run kubernetes top resource=pods namespace=default
skill run kubernetes top resource=nodes
```

### cluster-info
Display cluster information.

**Example**:
```bash
skill run kubernetes cluster-info
```

### config
Manage kubeconfig (view, current-context, get-contexts, use-context).

**Parameters**:
- `action` (required): Config action (view, current-context, get-contexts, use-context)
- `context` (optional): Context name (for use-context)

**Example**:
```bash
skill run kubernetes config action=view
skill run kubernetes config action=current-context
skill run kubernetes config action=use-context context=minikube
```

### create
Create resources (namespace, secret, configmap, deployment, service).

**Parameters**:
- `resource` (required): Resource type
- `name` (required): Resource name
- `namespace` (optional): Kubernetes namespace
- `from_literal` (optional): Key=value pairs for configmap/secret
- `image` (optional): Container image (for deployment)
- `port` (optional): Port number
- `type` (optional): Secret type or Service type

**Example**:
```bash
skill run kubernetes create resource=namespace name=test-namespace
skill run kubernetes create resource=deployment name=nginx namespace=test image=nginx:latest port=80
skill run kubernetes create resource=configmap name=myconfig namespace=default from_literal="key1=value1,key2=value2"
```

### label
Add or update labels on resources.

**Parameters**:
- `resource` (required): Resource type
- `name` (required): Resource name
- `namespace` (optional): Kubernetes namespace
- `labels` (required): Comma-separated key=value pairs

**Example**:
```bash
skill run kubernetes label resource=deployment name=nginx namespace=default labels="env=prod,tier=frontend"
```

### annotate
Add or update annotations on resources.

**Parameters**:
- `resource` (required): Resource type
- `name` (required): Resource name
- `namespace` (optional): Kubernetes namespace
- `annotations` (required): Comma-separated key=value pairs

**Example**:
```bash
skill run kubernetes annotate resource=deployment name=nginx namespace=default annotations="description=Web server"
```

### cordon
Mark a node as unschedulable.

**Parameters**:
- `node` (required): Node name

**Example**:
```bash
skill run kubernetes cordon node=worker-node-1
```

### uncordon
Mark a node as schedulable.

**Parameters**:
- `node` (required): Node name

**Example**:
```bash
skill run kubernetes uncordon node=worker-node-1
```

### drain
Drain a node for maintenance.

**Parameters**:
- `node` (required): Node name
- `ignore_daemonsets` (optional): Ignore DaemonSet pods
- `delete_emptydir_data` (optional): Delete pods using emptyDir
- `force` (optional): Force drain

**Example**:
```bash
skill run kubernetes drain node=worker-node-1 ignore_daemonsets=true
```

### taint
Add a taint to a node.

**Parameters**:
- `node` (required): Node name
- `taint` (required): Taint in key=value:effect format

**Example**:
```bash
skill run kubernetes taint node=worker-node-1 taint="dedicated=gpu:NoSchedule"
```

### raw
Execute any kubectl command directly.

**Parameters**:
- `args` (required): Raw kubectl arguments

**Example**:
```bash
skill run kubernetes raw args="version --client"
skill run kubernetes raw args="api-resources"
skill run kubernetes raw args="get pods -A -o wide"
```

## Configuration

This skill uses your existing kubectl configuration. No additional configuration is required.

To use a specific kubeconfig:
```bash
export KUBECONFIG=/path/to/kubeconfig
```

Or configure in `.skill-engine.toml`:
```toml
[skills.kubernetes.instances.prod]
config.kubeconfig = "${KUBECONFIG:-~/.kube/config}"
```

## Security Notes

- This skill executes real kubectl commands against your cluster
- Commands are validated against an allowlist (kubectl, helm, etc.)
- Ensure your kubeconfig has appropriate permissions
- Be cautious with delete and drain operations
