# Kubernetes Advanced Operations

This document covers advanced Kubernetes operations. Only read this file when the user explicitly needs:
- Node maintenance (drain, cordon, taint)
- Rollout management
- Resource creation from scratch
- Troubleshooting complex issues

---

## Node Maintenance

### When to Use Node Operations

Use these operations when:
- User mentions "maintenance", "upgrade", or "patching" nodes
- User wants to remove a node from the cluster
- User reports node issues (NotReady, disk pressure, memory pressure)

### Safe Node Drain Procedure

**Always follow this sequence** when draining a node:

```
Step 1: Cordon (prevent new pods)
            │
            ▼
Step 2: Verify workloads can move
            │
            ▼
Step 3: Drain (evict pods)
            │
            ▼
Step 4: Perform maintenance
            │
            ▼
Step 5: Uncordon (allow scheduling)
```

#### Step 1: Cordon the Node

First, prevent new pods from being scheduled:

```bash
kubectl cordon <node-name>
```

Verify the node shows `SchedulingDisabled`:

```bash
kubectl get nodes | grep <node-name>
```

#### Step 2: Check What Will Be Evicted

Before draining, show the user what pods will be affected:

```bash
kubectl get pods --all-namespaces --field-selector spec.nodeName=<node-name>
```

**Important**: If you see pods from these namespaces, warn the user:
- `kube-system` - Core cluster components
- `monitoring` - Monitoring stack
- Any namespace with "prod" or "production"

#### Step 3: Drain the Node

Standard drain (safe for most cases):

```bash
kubectl drain <node-name> --ignore-daemonsets --delete-emptydir-data
```

If drain fails due to PodDisruptionBudgets:

```bash
# Show the user what's blocking
kubectl get pdb --all-namespaces

# Only with explicit user confirmation:
kubectl drain <node-name> --ignore-daemonsets --delete-emptydir-data --disable-eviction
```

**Never use `--force` unless**:
1. User explicitly requests it
2. You've warned them about data loss risk
3. The pods are stateless

#### Step 4: After Maintenance - Uncordon

```bash
kubectl uncordon <node-name>
```

Verify pods are being scheduled again:

```bash
kubectl get pods --all-namespaces --field-selector spec.nodeName=<node-name>
```

### Node Taints

Use taints when the user wants to:
- Reserve nodes for specific workloads
- Prevent scheduling on problematic nodes
- Create dedicated node pools

#### Adding a Taint

```bash
kubectl taint nodes <node-name> <key>=<value>:<effect>
```

Effects explained to user:
| Effect | What Happens |
|--------|--------------|
| `NoSchedule` | New pods won't schedule (existing stay) |
| `PreferNoSchedule` | Soft preference, may still schedule |
| `NoExecute` | Evicts existing pods too |

**Example** - Reserve node for GPU workloads:

```bash
kubectl taint nodes gpu-node-1 gpu=true:NoSchedule
```

#### Removing a Taint

Add `-` at the end:

```bash
kubectl taint nodes <node-name> <key>=<value>:<effect>-
```

---

## Rollout Management

### When to Use Rollout Commands

Use these when the user:
- Deployed a bad version and needs to rollback
- Wants to check deployment progress
- Needs to pause/resume a rolling update
- Asks about deployment history

### Checking Rollout Status

Before any rollout operation, check current status:

```bash
kubectl rollout status deployment/<name> -n <namespace>
```

If stuck, show detailed status:

```bash
kubectl describe deployment <name> -n <namespace> | grep -A 10 "Conditions:"
```

### Rolling Back

#### Quick Rollback (Previous Version)

```bash
kubectl rollout undo deployment/<name> -n <namespace>
```

#### Rollback to Specific Version

First, show available revisions:

```bash
kubectl rollout history deployment/<name> -n <namespace>
```

Then rollback to specific revision:

```bash
kubectl rollout undo deployment/<name> -n <namespace> --to-revision=<number>
```

### Pause/Resume Rollouts

Use when user wants to:
- Inspect a canary deployment
- Wait for manual verification
- Coordinate with other changes

```bash
# Pause
kubectl rollout pause deployment/<name> -n <namespace>

# Resume
kubectl rollout resume deployment/<name> -n <namespace>
```

### Restart Deployment

When user wants to restart all pods (e.g., to pick up ConfigMap changes):

```bash
kubectl rollout restart deployment/<name> -n <namespace>
```

**Note**: This is a rolling restart, not a hard restart. Pods are replaced gradually.

---

## Resource Creation

### When to Create vs Apply

| User Intent | Command | Reason |
|-------------|---------|--------|
| "Create new resource" | `kubectl create` | Fails if exists (safer) |
| "Update or create" | `kubectl apply` | Idempotent |
| "Replace completely" | `kubectl replace` | Deletes and recreates |

### Creating Namespaces

```bash
kubectl create namespace <name>
```

**Naming conventions** - suggest to user:
- Use lowercase, hyphens only
- Include environment: `app-name-dev`, `app-name-prod`
- Keep under 63 characters

### Creating ConfigMaps

From literal values:

```bash
kubectl create configmap <name> -n <namespace> \
  --from-literal=key1=value1 \
  --from-literal=key2=value2
```

From file:

```bash
kubectl create configmap <name> -n <namespace> --from-file=<path>
```

### Creating Secrets

**Security warning**: Always inform user that secrets are base64 encoded, not encrypted.

From literal:

```bash
kubectl create secret generic <name> -n <namespace> \
  --from-literal=username=admin \
  --from-literal=password=secret123
```

**Better practice** - suggest to user:
```bash
# Read from environment variable (doesn't appear in shell history)
kubectl create secret generic <name> -n <namespace> \
  --from-literal=password="$DB_PASSWORD"
```

### Creating Deployments

Quick deployment:

```bash
kubectl create deployment <name> -n <namespace> \
  --image=<image> \
  --replicas=<count> \
  --port=<container-port>
```

**Always suggest**: After creating, expose if needed:

```bash
kubectl expose deployment <name> -n <namespace> \
  --port=<service-port> \
  --target-port=<container-port> \
  --type=ClusterIP
```

---

## Troubleshooting Guide

### Pod Won't Start

Follow this decision tree:

```
Pod Status?
    │
    ├─► Pending
    │       │
    │       └─► Check: kubectl describe pod <name> -n <namespace>
    │           Look for:
    │           - "Insufficient cpu/memory" → Scale down or add nodes
    │           - "No nodes match selector" → Check nodeSelector/affinity
    │           - "PersistentVolumeClaim not found" → Create PVC
    │
    ├─► ImagePullBackOff
    │       │
    │       └─► Check: kubectl describe pod <name> -n <namespace>
    │           Look for:
    │           - "unauthorized" → Check imagePullSecrets
    │           - "not found" → Verify image name/tag
    │           - "timeout" → Network/registry issues
    │
    ├─► CrashLoopBackOff
    │       │
    │       └─► Check: kubectl logs <pod> -n <namespace> --previous
    │           Common causes:
    │           - Missing config/secrets
    │           - Database connection failed
    │           - Port already in use
    │           - OOMKilled (check resources.limits)
    │
    └─► Running but not working
            │
            └─► Check:
                1. kubectl logs <pod> -n <namespace>
                2. kubectl exec -it <pod> -n <namespace> -- /bin/sh
                3. Test connectivity from inside pod
```

### Service Not Accessible

```
Can't reach service?
    │
    ├─► Check service exists
    │   kubectl get svc <name> -n <namespace>
    │
    ├─► Check endpoints (pods backing the service)
    │   kubectl get endpoints <name> -n <namespace>
    │
    │   If endpoints empty:
    │   - Check selector matches pod labels
    │   - Check pods are Running
    │
    ├─► Check from inside cluster
    │   kubectl run test --rm -it --image=busybox -- wget -qO- <service>:<port>
    │
    └─► For external access
        - ClusterIP: Only internal
        - NodePort: Check node firewall, use <nodeIP>:<nodePort>
        - LoadBalancer: Check cloud provider, get external IP
```

### High Resource Usage

When user reports slow cluster or OOM issues:

```bash
# Check node resource usage
kubectl top nodes

# Check pod resource usage (requires metrics-server)
kubectl top pods -n <namespace> --sort-by=memory

# Find pods without limits (potential resource hogs)
kubectl get pods -n <namespace> -o json | jq '.items[] | select(.spec.containers[].resources.limits == null) | .metadata.name'
```

---

## Emergency Procedures

### Force Delete Stuck Pod

**Only use when**: Pod is stuck in `Terminating` for >5 minutes and user confirms.

```bash
kubectl delete pod <name> -n <namespace> --grace-period=0 --force
```

**Warning**: Tell user this may cause:
- Data loss if pod was writing
- Split-brain if pod is part of stateful set

### Force Delete Stuck Namespace

**Only use when**: Namespace stuck in `Terminating` and user has removed all finalizers.

```bash
# Get namespace JSON
kubectl get namespace <name> -o json > ns.json

# Edit to remove finalizers (show user what you're removing)
# Change: "finalizers": ["kubernetes"] to "finalizers": []

# Apply via API
kubectl replace --raw "/api/v1/namespaces/<name>/finalize" -f ns.json
```

### Emergency Pod Kill (All Pods in Namespace)

**Extreme caution**: Only with explicit user confirmation and understanding.

```bash
kubectl delete pods --all -n <namespace>
```

For stateless apps, this triggers fresh pod creation. For stateful apps, warn about data implications.

---

## Best Practices to Suggest

When creating resources, suggest these to users:

1. **Always use namespaces** - Don't put everything in `default`
2. **Set resource requests/limits** - Prevents resource starvation
3. **Use labels consistently** - `app`, `env`, `team`, `version`
4. **Add readiness/liveness probes** - Enables self-healing
5. **Use ConfigMaps/Secrets** - Don't hardcode configuration
6. **Consider PodDisruptionBudgets** - For production workloads

---

## Related Documentation

- For basic operations, see [SKILL.md](SKILL.md)
- For Helm chart operations, see [HELM.md](HELM.md) (if available)
- For monitoring setup, see [MONITORING.md](MONITORING.md) (if available)
