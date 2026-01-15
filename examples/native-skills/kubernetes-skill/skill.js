/**
 * Kubernetes Skill - Real cluster management via kubectl
 *
 * This skill executes real kubectl commands against your Kubernetes cluster.
 * It uses the kubeconfig from the environment or default location.
 *
 * Requirements:
 * - kubectl must be installed and in PATH
 * - Valid kubeconfig (default: ~/.kube/config or KUBECONFIG env var)
 */

// Define skill metadata
export function getMetadata() {
  return JSON.stringify({
    name: "kubernetes",
    version: "0.2.0",
    description: "Kubernetes cluster management - real kubectl integration",
    author: "Skill Engine"
  });
}

// Define available tools
export function getTools() {
  return JSON.stringify([
    {
      name: "get",
      description: "Get Kubernetes resources (pods, services, deployments, nodes, etc.)",
      parameters: [
        {
          name: "resource",
          paramType: "string",
          description: "Resource type: pods, services, deployments, nodes, namespaces, configmaps, secrets, ingress, pv, pvc, jobs, cronjobs, daemonsets, statefulsets, replicasets, events, endpoints, all",
          required: true
        },
        {
          name: "name",
          paramType: "string",
          description: "Specific resource name (optional)",
          required: false
        },
        {
          name: "namespace",
          paramType: "string",
          description: "Kubernetes namespace (default: default, use 'all' for all namespaces)",
          required: false,
          defaultValue: "default"
        },
        {
          name: "selector",
          paramType: "string",
          description: "Label selector (e.g., app=nginx)",
          required: false
        },
        {
          name: "output",
          paramType: "string",
          description: "Output format: wide, yaml, json, name",
          required: false
        }
      ]
    },
    {
      name: "describe",
      description: "Show detailed information about a resource",
      parameters: [
        {
          name: "resource",
          paramType: "string",
          description: "Resource type (pod, service, deployment, node, etc.)",
          required: true
        },
        {
          name: "name",
          paramType: "string",
          description: "Resource name",
          required: true
        },
        {
          name: "namespace",
          paramType: "string",
          description: "Kubernetes namespace",
          required: false,
          defaultValue: "default"
        }
      ]
    },
    {
      name: "logs",
      description: "Get logs from a pod",
      parameters: [
        {
          name: "pod",
          paramType: "string",
          description: "Pod name",
          required: true
        },
        {
          name: "namespace",
          paramType: "string",
          description: "Kubernetes namespace",
          required: false,
          defaultValue: "default"
        },
        {
          name: "container",
          paramType: "string",
          description: "Container name (for multi-container pods)",
          required: false
        },
        {
          name: "tail",
          paramType: "number",
          description: "Number of lines to show (default: 100)",
          required: false,
          defaultValue: "100"
        },
        {
          name: "follow",
          paramType: "boolean",
          description: "Follow log output (streaming)",
          required: false,
          defaultValue: "false"
        },
        {
          name: "previous",
          paramType: "boolean",
          description: "Show logs from previous container instance",
          required: false,
          defaultValue: "false"
        },
        {
          name: "since",
          paramType: "string",
          description: "Show logs since duration (e.g., 1h, 30m, 1d)",
          required: false
        }
      ]
    },
    {
      name: "exec",
      description: "Execute a command in a pod",
      parameters: [
        {
          name: "pod",
          paramType: "string",
          description: "Pod name",
          required: true
        },
        {
          name: "command",
          paramType: "string",
          description: "Command to execute",
          required: true
        },
        {
          name: "namespace",
          paramType: "string",
          description: "Kubernetes namespace",
          required: false,
          defaultValue: "default"
        },
        {
          name: "container",
          paramType: "string",
          description: "Container name (for multi-container pods)",
          required: false
        }
      ]
    },
    {
      name: "apply",
      description: "Apply a configuration from YAML/JSON content",
      parameters: [
        {
          name: "content",
          paramType: "string",
          description: "YAML or JSON content to apply",
          required: true
        },
        {
          name: "namespace",
          paramType: "string",
          description: "Kubernetes namespace",
          required: false,
          defaultValue: "default"
        },
        {
          name: "dry_run",
          paramType: "boolean",
          description: "Run in dry-run mode (server-side)",
          required: false,
          defaultValue: "false"
        }
      ]
    },
    {
      name: "delete",
      description: "Delete Kubernetes resources",
      parameters: [
        {
          name: "resource",
          paramType: "string",
          description: "Resource type (pod, service, deployment, etc.)",
          required: true
        },
        {
          name: "name",
          paramType: "string",
          description: "Resource name",
          required: true
        },
        {
          name: "namespace",
          paramType: "string",
          description: "Kubernetes namespace",
          required: false,
          defaultValue: "default"
        },
        {
          name: "force",
          paramType: "boolean",
          description: "Force deletion",
          required: false,
          defaultValue: "false"
        },
        {
          name: "grace_period",
          paramType: "number",
          description: "Grace period in seconds (0 for immediate)",
          required: false
        }
      ]
    },
    {
      name: "scale",
      description: "Scale a deployment, statefulset, or replicaset",
      parameters: [
        {
          name: "resource",
          paramType: "string",
          description: "Resource type (deployment, statefulset, replicaset)",
          required: false,
          defaultValue: "deployment"
        },
        {
          name: "name",
          paramType: "string",
          description: "Resource name",
          required: true
        },
        {
          name: "replicas",
          paramType: "number",
          description: "Number of replicas",
          required: true
        },
        {
          name: "namespace",
          paramType: "string",
          description: "Kubernetes namespace",
          required: false,
          defaultValue: "default"
        }
      ]
    },
    {
      name: "rollout",
      description: "Manage rollouts (status, history, restart, undo)",
      parameters: [
        {
          name: "action",
          paramType: "string",
          description: "Rollout action: status, history, restart, undo, pause, resume",
          required: true
        },
        {
          name: "resource",
          paramType: "string",
          description: "Resource type (deployment, statefulset, daemonset)",
          required: false,
          defaultValue: "deployment"
        },
        {
          name: "name",
          paramType: "string",
          description: "Resource name",
          required: true
        },
        {
          name: "namespace",
          paramType: "string",
          description: "Kubernetes namespace",
          required: false,
          defaultValue: "default"
        },
        {
          name: "revision",
          paramType: "number",
          description: "Revision number (for undo)",
          required: false
        }
      ]
    },
    {
      name: "top",
      description: "Display resource usage (CPU/memory) for pods or nodes",
      parameters: [
        {
          name: "resource",
          paramType: "string",
          description: "Resource type: pods or nodes",
          required: true
        },
        {
          name: "name",
          paramType: "string",
          description: "Specific resource name (optional)",
          required: false
        },
        {
          name: "namespace",
          paramType: "string",
          description: "Kubernetes namespace (for pods)",
          required: false,
          defaultValue: "default"
        },
        {
          name: "containers",
          paramType: "boolean",
          description: "Show container-level metrics (for pods)",
          required: false,
          defaultValue: "false"
        }
      ]
    },
    {
      name: "cluster-info",
      description: "Display cluster information",
      parameters: []
    },
    {
      name: "config",
      description: "Manage kubeconfig (view, current-context, get-contexts, use-context)",
      parameters: [
        {
          name: "action",
          paramType: "string",
          description: "Config action: view, current-context, get-contexts, use-context",
          required: true
        },
        {
          name: "context",
          paramType: "string",
          description: "Context name (for use-context)",
          required: false
        }
      ]
    },
    {
      name: "port-forward",
      description: "Forward local ports to a pod (returns command to run)",
      parameters: [
        {
          name: "pod",
          paramType: "string",
          description: "Pod name",
          required: true
        },
        {
          name: "ports",
          paramType: "string",
          description: "Port mapping (e.g., 8080:80 or 8080)",
          required: true
        },
        {
          name: "namespace",
          paramType: "string",
          description: "Kubernetes namespace",
          required: false,
          defaultValue: "default"
        }
      ]
    },
    {
      name: "create",
      description: "Create resources (namespace, secret, configmap, etc.)",
      parameters: [
        {
          name: "resource",
          paramType: "string",
          description: "Resource type: namespace, secret, configmap, deployment, service",
          required: true
        },
        {
          name: "name",
          paramType: "string",
          description: "Resource name",
          required: true
        },
        {
          name: "namespace",
          paramType: "string",
          description: "Kubernetes namespace (not for namespace creation)",
          required: false,
          defaultValue: "default"
        },
        {
          name: "from_literal",
          paramType: "string",
          description: "Key=value pairs for configmap/secret (comma-separated)",
          required: false
        },
        {
          name: "image",
          paramType: "string",
          description: "Container image (for deployment)",
          required: false
        },
        {
          name: "port",
          paramType: "number",
          description: "Port number (for service/deployment)",
          required: false
        },
        {
          name: "type",
          paramType: "string",
          description: "Secret type (generic, docker-registry, tls) or Service type (ClusterIP, NodePort, LoadBalancer)",
          required: false
        }
      ]
    },
    {
      name: "label",
      description: "Add or update labels on resources",
      parameters: [
        {
          name: "resource",
          paramType: "string",
          description: "Resource type",
          required: true
        },
        {
          name: "name",
          paramType: "string",
          description: "Resource name",
          required: true
        },
        {
          name: "labels",
          paramType: "string",
          description: "Labels to set (key=value, comma-separated)",
          required: true
        },
        {
          name: "namespace",
          paramType: "string",
          description: "Kubernetes namespace",
          required: false,
          defaultValue: "default"
        },
        {
          name: "overwrite",
          paramType: "boolean",
          description: "Overwrite existing labels",
          required: false,
          defaultValue: "false"
        }
      ]
    },
    {
      name: "annotate",
      description: "Add or update annotations on resources",
      parameters: [
        {
          name: "resource",
          paramType: "string",
          description: "Resource type",
          required: true
        },
        {
          name: "name",
          paramType: "string",
          description: "Resource name",
          required: true
        },
        {
          name: "annotations",
          paramType: "string",
          description: "Annotations to set (key=value, comma-separated)",
          required: true
        },
        {
          name: "namespace",
          paramType: "string",
          description: "Kubernetes namespace",
          required: false,
          defaultValue: "default"
        },
        {
          name: "overwrite",
          paramType: "boolean",
          description: "Overwrite existing annotations",
          required: false,
          defaultValue: "false"
        }
      ]
    },
    {
      name: "cordon",
      description: "Mark node as unschedulable",
      parameters: [
        {
          name: "node",
          paramType: "string",
          description: "Node name",
          required: true
        }
      ]
    },
    {
      name: "uncordon",
      description: "Mark node as schedulable",
      parameters: [
        {
          name: "node",
          paramType: "string",
          description: "Node name",
          required: true
        }
      ]
    },
    {
      name: "drain",
      description: "Drain node for maintenance",
      parameters: [
        {
          name: "node",
          paramType: "string",
          description: "Node name",
          required: true
        },
        {
          name: "force",
          paramType: "boolean",
          description: "Force drain even with pods not managed by ReplicationController, ReplicaSet, Job, DaemonSet or StatefulSet",
          required: false,
          defaultValue: "false"
        },
        {
          name: "ignore_daemonsets",
          paramType: "boolean",
          description: "Ignore DaemonSet-managed pods",
          required: false,
          defaultValue: "true"
        },
        {
          name: "delete_emptydir_data",
          paramType: "boolean",
          description: "Delete pods using emptyDir",
          required: false,
          defaultValue: "false"
        },
        {
          name: "grace_period",
          paramType: "number",
          description: "Grace period in seconds",
          required: false
        }
      ]
    },
    {
      name: "taint",
      description: "Add or remove taints from nodes",
      parameters: [
        {
          name: "node",
          paramType: "string",
          description: "Node name",
          required: true
        },
        {
          name: "taint",
          paramType: "string",
          description: "Taint to add (key=value:effect) or remove (key:effect- or key-)",
          required: true
        }
      ]
    },
    {
      name: "raw",
      description: "Execute any kubectl command directly",
      parameters: [
        {
          name: "args",
          paramType: "string",
          description: "kubectl arguments (e.g., 'get pods -o wide')",
          required: true
        }
      ]
    }
  ]);
}

// Execute a tool by running kubectl
export async function executeTool(toolName, argsJson) {
  try {
    const args = JSON.parse(argsJson);

    // Extract proxy URL if injected by the runtime
    const injectedProxyUrl = args._proxy_url;
    if (injectedProxyUrl) {
      delete args._proxy_url; // Don't pass to kubectl args builder
    }

    let kubectlArgs = [];

    switch (toolName) {
      case "get":
        kubectlArgs = buildGetArgs(args);
        break;
      case "describe":
        kubectlArgs = buildDescribeArgs(args);
        break;
      case "logs":
        kubectlArgs = buildLogsArgs(args);
        break;
      case "exec":
        kubectlArgs = buildExecArgs(args);
        break;
      case "apply":
        kubectlArgs = buildApplyArgs(args);
        break;
      case "delete":
        kubectlArgs = buildDeleteArgs(args);
        break;
      case "scale":
        kubectlArgs = buildScaleArgs(args);
        break;
      case "rollout":
        kubectlArgs = buildRolloutArgs(args);
        break;
      case "top":
        kubectlArgs = buildTopArgs(args);
        break;
      case "cluster-info":
        kubectlArgs = ["cluster-info"];
        break;
      case "config":
        kubectlArgs = buildConfigArgs(args);
        break;
      case "port-forward":
        return buildPortForwardResponse(args);
      case "create":
        kubectlArgs = buildCreateArgs(args);
        break;
      case "label":
        kubectlArgs = buildLabelArgs(args);
        break;
      case "annotate":
        kubectlArgs = buildAnnotateArgs(args);
        break;
      case "cordon":
        kubectlArgs = ["cordon", args.node];
        break;
      case "uncordon":
        kubectlArgs = ["uncordon", args.node];
        break;
      case "drain":
        kubectlArgs = buildDrainArgs(args);
        break;
      case "taint":
        kubectlArgs = ["taint", "nodes", args.node, args.taint];
        break;
      case "raw":
        kubectlArgs = args.args.split(" ");
        break;
      default:
        return error(`Unknown tool: ${toolName}`);
    }

    // Execute kubectl command, passing injected proxy URL if available
    const result = await runKubectl(kubectlArgs, injectedProxyUrl);
    return result;

  } catch (e) {
    return error(`Error executing tool: ${e.message || e}`);
  }
}

// Validate configuration
export async function validateConfig() {
  return JSON.stringify({ ok: null });
}

// Helper functions
function success(output) {
  return JSON.stringify({
    ok: { success: true, output, errorMessage: null }
  });
}

function error(message) {
  return JSON.stringify({ err: message });
}

// Build kubectl argument arrays for each tool
function buildGetArgs(args) {
  const kubectlArgs = ["get"];

  if (args.name) {
    kubectlArgs.push(args.name);
  }

  if (args.namespace === "all") {
    kubectlArgs.push("--all-namespaces");
  } else if (args.namespace) {
    kubectlArgs.push("-n", args.namespace);
  }

  if (args.selector) {
    kubectlArgs.push("-l", args.selector);
  }

  if (args.output) {
    kubectlArgs.push("-o", args.output);
  }

  return kubectlArgs;
}

function buildDescribeArgs(args) {
  const kubectlArgs = ["describe", args.resource, args.name];

  if (args.namespace) {
    kubectlArgs.push("-n", args.namespace);
  }

  return kubectlArgs;
}

function buildLogsArgs(args) {
  const kubectlArgs = ["logs", args.pod];

  if (args.namespace) {
    kubectlArgs.push("-n", args.namespace);
  }

  if (args.container) {
    kubectlArgs.push("-c", args.container);
  }

  if (args.tail) {
    kubectlArgs.push("--tail", String(args.tail));
  }

  if (args.follow === "true" || args.follow === true) {
    kubectlArgs.push("-f");
  }

  if (args.previous === "true" || args.previous === true) {
    kubectlArgs.push("--previous");
  }

  if (args.since) {
    kubectlArgs.push("--since", args.since);
  }

  return kubectlArgs;
}

function buildExecArgs(args) {
  const kubectlArgs = ["exec", args.pod];

  if (args.namespace) {
    kubectlArgs.push("-n", args.namespace);
  }

  if (args.container) {
    kubectlArgs.push("-c", args.container);
  }

  kubectlArgs.push("--");

  // Split command into args
  const cmdParts = args.command.split(" ");
  kubectlArgs.push(...cmdParts);

  return kubectlArgs;
}

function buildApplyArgs(args) {
  // For apply, we need to use stdin
  const kubectlArgs = ["apply", "-f", "-"];

  if (args.namespace) {
    kubectlArgs.push("-n", args.namespace);
  }

  if (args.dry_run === "true" || args.dry_run === true) {
    kubectlArgs.push("--dry-run=server");
  }

  // Store content for stdin
  kubectlArgs._stdinContent = args.content;

  return kubectlArgs;
}

function buildDeleteArgs(args) {
  const kubectlArgs = ["delete", args.resource, args.name];

  if (args.namespace) {
    kubectlArgs.push("-n", args.namespace);
  }

  if (args.force === "true" || args.force === true) {
    kubectlArgs.push("--force");
  }

  if (args.grace_period !== undefined) {
    kubectlArgs.push("--grace-period", String(args.grace_period));
  }

  return kubectlArgs;
}

function buildScaleArgs(args) {
  const resource = args.resource || "deployment";
  const kubectlArgs = ["scale", `${resource}/${args.name}`, `--replicas=${args.replicas}`];

  if (args.namespace) {
    kubectlArgs.push("-n", args.namespace);
  }

  return kubectlArgs;
}

function buildRolloutArgs(args) {
  const resource = args.resource || "deployment";
  const kubectlArgs = ["rollout", args.action, `${resource}/${args.name}`];

  if (args.namespace) {
    kubectlArgs.push("-n", args.namespace);
  }

  if (args.action === "undo" && args.revision) {
    kubectlArgs.push(`--to-revision=${args.revision}`);
  }

  return kubectlArgs;
}

function buildTopArgs(args) {
  const kubectlArgs = ["top", args.resource];

  if (args.name) {
    kubectlArgs.push(args.name);
  }

  if (args.resource === "pods" && args.namespace) {
    if (args.namespace === "all") {
      kubectlArgs.push("--all-namespaces");
    } else {
      kubectlArgs.push("-n", args.namespace);
    }
  }

  if (args.containers === "true" || args.containers === true) {
    kubectlArgs.push("--containers");
  }

  return kubectlArgs;
}

function buildConfigArgs(args) {
  switch (args.action) {
    case "view":
      return ["config", "view"];
    case "current-context":
      return ["config", "current-context"];
    case "get-contexts":
      return ["config", "get-contexts"];
    case "use-context":
      if (!args.context) {
        throw new Error("context parameter required for use-context");
      }
      return ["config", "use-context", args.context];
    default:
      throw new Error(`Unknown config action: ${args.action}`);
  }
}

function buildPortForwardResponse(args) {
  const ns = args.namespace || "default";
  const cmd = `kubectl port-forward -n ${ns} ${args.pod} ${args.ports}`;

  return success(`Port forwarding is a long-running operation that requires an interactive terminal.

To forward ports, run this command in your terminal:

  ${cmd}

This will forward local port(s) to the pod until you press Ctrl+C.

Alternative: Use 'kubectl port-forward' directly or a tool like 'kubefwd' for multiple services.`);
}

function buildCreateArgs(args) {
  const kubectlArgs = ["create", args.resource, args.name];

  // Namespace doesn't apply to namespace creation
  if (args.resource !== "namespace" && args.namespace) {
    kubectlArgs.push("-n", args.namespace);
  }

  // Handle resource-specific options
  switch (args.resource) {
    case "deployment":
      if (args.image) {
        kubectlArgs.push(`--image=${args.image}`);
      }
      if (args.port) {
        kubectlArgs.push(`--port=${args.port}`);
      }
      break;
    case "service":
      // service needs special handling
      // kubectl create service clusterip NAME --tcp=PORT:TARGETPORT
      break;
    case "configmap":
    case "secret":
      if (args.from_literal) {
        const pairs = args.from_literal.split(",");
        for (const pair of pairs) {
          kubectlArgs.push(`--from-literal=${pair.trim()}`);
        }
      }
      if (args.resource === "secret" && args.type) {
        kubectlArgs.splice(1, 0, args.type); // insert type after 'secret'
      }
      break;
  }

  return kubectlArgs;
}

function buildLabelArgs(args) {
  const kubectlArgs = ["label", args.resource, args.name];

  if (args.namespace) {
    kubectlArgs.push("-n", args.namespace);
  }

  // Add labels
  const labels = args.labels.split(",");
  for (const label of labels) {
    kubectlArgs.push(label.trim());
  }

  if (args.overwrite === "true" || args.overwrite === true) {
    kubectlArgs.push("--overwrite");
  }

  return kubectlArgs;
}

function buildAnnotateArgs(args) {
  const kubectlArgs = ["annotate", args.resource, args.name];

  if (args.namespace) {
    kubectlArgs.push("-n", args.namespace);
  }

  // Add annotations
  const annotations = args.annotations.split(",");
  for (const annotation of annotations) {
    kubectlArgs.push(annotation.trim());
  }

  if (args.overwrite === "true" || args.overwrite === true) {
    kubectlArgs.push("--overwrite");
  }

  return kubectlArgs;
}

function buildDrainArgs(args) {
  const kubectlArgs = ["drain", args.node];

  if (args.force === "true" || args.force === true) {
    kubectlArgs.push("--force");
  }

  if (args.ignore_daemonsets === "true" || args.ignore_daemonsets === true || args.ignore_daemonsets === undefined) {
    kubectlArgs.push("--ignore-daemonsets");
  }

  if (args.delete_emptydir_data === "true" || args.delete_emptydir_data === true) {
    kubectlArgs.push("--delete-emptydir-data");
  }

  if (args.grace_period !== undefined) {
    kubectlArgs.push("--grace-period", String(args.grace_period));
  }

  return kubectlArgs;
}

// Run kubectl command using the Skill Engine's process execution
// This function uses environment variable KUBECTL_CMD or defaults to 'kubectl'
// injectedProxyUrl: proxy URL injected by the Skill Engine runtime (bypasses env var limitations)
async function runKubectl(args, injectedProxyUrl = null) {
  // Get kubectl path from environment or use default
  const kubectl = globalThis.process?.env?.KUBECTL_CMD || "kubectl";

  // Check for stdin content (for apply)
  const stdinContent = args._stdinContent;
  if (stdinContent) {
    delete args._stdinContent;
  }

  // Build the full command
  const fullCmd = [kubectl, ...args].join(" ");

  // Since WASM can't directly execute processes, we need to use
  // the host's command execution capability or return instructions

  // For now, return the command that would be executed
  // The Skill Engine runtime can intercept this and execute it

  // Try to use fetch to call a local kubectl proxy if available
  // This is a common pattern for K8s integrations

  // Check if we have access to a kubectl proxy
  // First check injected proxy URL (from runtime), then fall back to env var
  const proxyUrl = injectedProxyUrl || globalThis.process?.env?.KUBECTL_PROXY_URL;

  if (proxyUrl) {
    // Use kubectl proxy API
    return await executeViaProxy(proxyUrl, args);
  }

  // Return the command for manual execution or host processing
  return success(`Command: ${fullCmd}

Note: This skill is running in WASM and cannot directly execute kubectl.

To execute this command:
1. Copy and run it in your terminal
2. Or set KUBECTL_PROXY_URL to a running 'kubectl proxy' endpoint
3. Or use the Skill Engine's native kubectl integration

For native execution, start kubectl proxy:
  kubectl proxy --port=8001 &

Then set the environment variable and run again.`);
}

// Execute kubectl command via kubectl proxy API
async function executeViaProxy(proxyUrl, args) {
  // This would use the Kubernetes API directly
  // For complex commands, we'd need to translate to API calls

  // For simple GET operations, we can use the API directly
  const cmd = args[0];

  if (cmd === "get") {
    const resource = args[1];
    const namespace = args.includes("-n") ? args[args.indexOf("-n") + 1] : "default";
    const allNamespaces = args.includes("--all-namespaces");

    let apiPath;
    if (allNamespaces) {
      apiPath = `/api/v1/${resource}`;
    } else {
      apiPath = `/api/v1/namespaces/${namespace}/${resource}`;
    }

    // Check for specific resource name
    const resourceIndex = args.indexOf(resource);
    if (resourceIndex + 1 < args.length && !args[resourceIndex + 1].startsWith("-")) {
      apiPath += `/${args[resourceIndex + 1]}`;
    }

    try {
      const response = await fetch(`${proxyUrl}${apiPath}`);
      if (!response.ok) {
        return error(`API error: ${response.status} ${response.statusText}`);
      }

      const data = await response.json();

      // Format output similar to kubectl
      if (data.items) {
        // List response
        let output = formatResourceList(resource, data.items);
        return success(output);
      } else {
        // Single resource
        return success(JSON.stringify(data, null, 2));
      }
    } catch (e) {
      return error(`Failed to call Kubernetes API: ${e.message || e}`);
    }
  }

  // For other commands, return the command string
  return success(`Command would be: kubectl ${args.join(" ")}\n\nKubectl proxy API doesn't support this operation directly. Run the command manually.`);
}

// Format resource list as table (similar to kubectl)
function formatResourceList(resource, items) {
  if (!items || items.length === 0) {
    return `No ${resource} found.`;
  }

  switch (resource) {
    case "pods":
      return formatPodsTable(items);
    case "services":
    case "svc":
      return formatServicesTable(items);
    case "deployments":
    case "deploy":
      return formatDeploymentsTable(items);
    case "nodes":
      return formatNodesTable(items);
    case "namespaces":
    case "ns":
      return formatNamespacesTable(items);
    default:
      // Generic format
      return items.map(item => item.metadata?.name || JSON.stringify(item)).join("\n");
  }
}

function formatPodsTable(pods) {
  let output = "NAME                                      READY   STATUS    RESTARTS   AGE\n";

  for (const pod of pods) {
    const name = pod.metadata.name;
    const containers = pod.status?.containerStatuses || [];
    const ready = `${containers.filter(c => c.ready).length}/${containers.length}`;
    const status = pod.status?.phase || "Unknown";
    const restarts = containers.reduce((sum, c) => sum + (c.restartCount || 0), 0);
    const age = getAge(pod.metadata.creationTimestamp);

    output += `${name.padEnd(42)}${ready.padEnd(8)}${status.padEnd(10)}${String(restarts).padEnd(11)}${age}\n`;
  }

  return output;
}

function formatServicesTable(services) {
  let output = "NAME                TYPE           CLUSTER-IP       PORT(S)\n";

  for (const svc of services) {
    const name = svc.metadata.name;
    const type = svc.spec?.type || "ClusterIP";
    const clusterIP = svc.spec?.clusterIP || "<none>";
    const ports = (svc.spec?.ports || []).map(p => `${p.port}/${p.protocol}`).join(",") || "<none>";

    output += `${name.padEnd(20)}${type.padEnd(15)}${clusterIP.padEnd(17)}${ports}\n`;
  }

  return output;
}

function formatDeploymentsTable(deployments) {
  let output = "NAME                READY   UP-TO-DATE   AVAILABLE   AGE\n";

  for (const dep of deployments) {
    const name = dep.metadata.name;
    const ready = `${dep.status?.readyReplicas || 0}/${dep.spec?.replicas || 0}`;
    const upToDate = dep.status?.updatedReplicas || 0;
    const available = dep.status?.availableReplicas || 0;
    const age = getAge(dep.metadata.creationTimestamp);

    output += `${name.padEnd(20)}${ready.padEnd(8)}${String(upToDate).padEnd(13)}${String(available).padEnd(12)}${age}\n`;
  }

  return output;
}

function formatNodesTable(nodes) {
  let output = "NAME                STATUS   ROLES    AGE   VERSION\n";

  for (const node of nodes) {
    const name = node.metadata.name;
    const conditions = node.status?.conditions || [];
    const readyCondition = conditions.find(c => c.type === "Ready");
    const status = readyCondition?.status === "True" ? "Ready" : "NotReady";
    const roles = Object.keys(node.metadata?.labels || {})
      .filter(l => l.startsWith("node-role.kubernetes.io/"))
      .map(l => l.replace("node-role.kubernetes.io/", ""))
      .join(",") || "<none>";
    const age = getAge(node.metadata.creationTimestamp);
    const version = node.status?.nodeInfo?.kubeletVersion || "unknown";

    output += `${name.padEnd(20)}${status.padEnd(9)}${roles.padEnd(9)}${age.padEnd(6)}${version}\n`;
  }

  return output;
}

function formatNamespacesTable(namespaces) {
  let output = "NAME                STATUS   AGE\n";

  for (const ns of namespaces) {
    const name = ns.metadata.name;
    const status = ns.status?.phase || "Active";
    const age = getAge(ns.metadata.creationTimestamp);

    output += `${name.padEnd(20)}${status.padEnd(9)}${age}\n`;
  }

  return output;
}

function getAge(timestamp) {
  if (!timestamp) return "unknown";

  const created = new Date(timestamp);
  const now = new Date();
  const diffMs = now - created;
  const diffSecs = Math.floor(diffMs / 1000);
  const diffMins = Math.floor(diffSecs / 60);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffDays > 0) return `${diffDays}d`;
  if (diffHours > 0) return `${diffHours}h`;
  if (diffMins > 0) return `${diffMins}m`;
  return `${diffSecs}s`;
}
