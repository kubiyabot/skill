// Azure Skill - Native az CLI wrapper
// Provides Microsoft Azure management through the az CLI

function success(output) {
  return JSON.stringify({
    ok: { success: true, output, errorMessage: null }
  });
}

function error(message) {
  return JSON.stringify({ err: message });
}

export function getMetadata() {
  return JSON.stringify({
    name: "azure",
    version: "1.0.0",
    description: "Microsoft Azure management with native az CLI integration",
    author: "Skill Engine"
  });
}

export function getTools() {
  return JSON.stringify([
    {
      name: "vm-list",
      description: "List Azure Virtual Machines",
      parameters: [
        { name: "resource_group", paramType: "string", description: "Filter by resource group", required: false },
        { name: "show_details", paramType: "boolean", description: "Include detailed info", required: false }
      ]
    },
    {
      name: "vm-start",
      description: "Start a stopped virtual machine",
      parameters: [
        { name: "name", paramType: "string", description: "VM name", required: true },
        { name: "resource_group", paramType: "string", description: "Resource group", required: true }
      ]
    },
    {
      name: "vm-stop",
      description: "Stop (deallocate) a virtual machine",
      parameters: [
        { name: "name", paramType: "string", description: "VM name", required: true },
        { name: "resource_group", paramType: "string", description: "Resource group", required: true }
      ]
    },
    {
      name: "vm-restart",
      description: "Restart a virtual machine",
      parameters: [
        { name: "name", paramType: "string", description: "VM name", required: true },
        { name: "resource_group", paramType: "string", description: "Resource group", required: true }
      ]
    },
    {
      name: "vm-run-command",
      description: "Run a command on a VM",
      parameters: [
        { name: "name", paramType: "string", description: "VM name", required: true },
        { name: "resource_group", paramType: "string", description: "Resource group", required: true },
        { name: "command", paramType: "string", description: "Shell command to run", required: true }
      ]
    },
    {
      name: "storage-list",
      description: "List storage accounts",
      parameters: [
        { name: "resource_group", paramType: "string", description: "Filter by resource group", required: false }
      ]
    },
    {
      name: "storage-blob-list",
      description: "List blobs in a container",
      parameters: [
        { name: "account", paramType: "string", description: "Storage account name", required: true },
        { name: "container", paramType: "string", description: "Container name", required: true },
        { name: "prefix", paramType: "string", description: "Blob prefix filter", required: false }
      ]
    },
    {
      name: "storage-blob-upload",
      description: "Upload a blob to storage",
      parameters: [
        { name: "account", paramType: "string", description: "Storage account name", required: true },
        { name: "container", paramType: "string", description: "Container name", required: true },
        { name: "source", paramType: "string", description: "Local file path", required: true },
        { name: "name", paramType: "string", description: "Blob name (default: filename)", required: false }
      ]
    },
    {
      name: "storage-blob-download",
      description: "Download a blob from storage",
      parameters: [
        { name: "account", paramType: "string", description: "Storage account name", required: true },
        { name: "container", paramType: "string", description: "Container name", required: true },
        { name: "name", paramType: "string", description: "Blob name", required: true },
        { name: "destination", paramType: "string", description: "Local file path", required: true }
      ]
    },
    {
      name: "sql-list",
      description: "List Azure SQL servers",
      parameters: [
        { name: "resource_group", paramType: "string", description: "Filter by resource group", required: false }
      ]
    },
    {
      name: "sql-db-list",
      description: "List databases on a SQL server",
      parameters: [
        { name: "server", paramType: "string", description: "SQL server name", required: true },
        { name: "resource_group", paramType: "string", description: "Resource group", required: true }
      ]
    },
    {
      name: "aks-list",
      description: "List Azure Kubernetes Service clusters",
      parameters: [
        { name: "resource_group", paramType: "string", description: "Filter by resource group", required: false }
      ]
    },
    {
      name: "aks-credentials",
      description: "Get credentials for an AKS cluster (updates kubeconfig)",
      parameters: [
        { name: "name", paramType: "string", description: "Cluster name", required: true },
        { name: "resource_group", paramType: "string", description: "Resource group", required: true },
        { name: "admin", paramType: "boolean", description: "Get admin credentials", required: false }
      ]
    },
    {
      name: "resource-list",
      description: "List all resources in a subscription or resource group",
      parameters: [
        { name: "resource_group", paramType: "string", description: "Filter by resource group", required: false },
        { name: "type", paramType: "string", description: "Filter by resource type", required: false }
      ]
    },
    {
      name: "resource-group-list",
      description: "List resource groups",
      parameters: []
    },
    {
      name: "monitor-metrics",
      description: "Query Azure Monitor metrics",
      parameters: [
        { name: "resource", paramType: "string", description: "Resource ID", required: true },
        { name: "metric", paramType: "string", description: "Metric name", required: true },
        { name: "interval", paramType: "string", description: "Time interval (default: PT1H)", required: false },
        { name: "aggregation", paramType: "string", description: "Aggregation type", required: false }
      ]
    }
  ]);
}

export function executeTool(toolName, argsJson) {
  try {
    const args = JSON.parse(argsJson);

    let azArgs = [];

    switch (toolName) {
      case "vm-list":
        azArgs = buildVmListArgs(args);
        break;
      case "vm-start":
        azArgs = buildVmStartArgs(args);
        break;
      case "vm-stop":
        azArgs = buildVmStopArgs(args);
        break;
      case "vm-restart":
        azArgs = buildVmRestartArgs(args);
        break;
      case "vm-run-command":
        azArgs = buildVmRunCommandArgs(args);
        break;
      case "storage-list":
        azArgs = buildStorageListArgs(args);
        break;
      case "storage-blob-list":
        azArgs = buildStorageBlobListArgs(args);
        break;
      case "storage-blob-upload":
        azArgs = buildStorageBlobUploadArgs(args);
        break;
      case "storage-blob-download":
        azArgs = buildStorageBlobDownloadArgs(args);
        break;
      case "sql-list":
        azArgs = buildSqlListArgs(args);
        break;
      case "sql-db-list":
        azArgs = buildSqlDbListArgs(args);
        break;
      case "aks-list":
        azArgs = buildAksListArgs(args);
        break;
      case "aks-credentials":
        azArgs = buildAksCredentialsArgs(args);
        break;
      case "resource-list":
        azArgs = buildResourceListArgs(args);
        break;
      case "resource-group-list":
        azArgs = ["group", "list", "--output", "json"];
        break;
      case "monitor-metrics":
        azArgs = buildMonitorMetricsArgs(args);
        break;
      default:
        return error(`Unknown tool: ${toolName}`);
    }

    return runCommand(azArgs);
  } catch (e) {
    return error(`Error executing tool: ${e.message || e}`);
  }
}

export function validateConfig(configJson) {
  return JSON.stringify({ ok: null });
}

// Build functions for each tool

function buildVmListArgs(args) {
  const azArgs = ["vm", "list", "--output", "json"];

  if (args.resource_group) {
    azArgs.push("--resource-group", args.resource_group);
  }

  if (args.show_details === "true" || args.show_details === true) {
    azArgs.push("--show-details");
  }

  return azArgs;
}

function buildVmStartArgs(args) {
  if (!args.name) throw new Error("name is required");
  if (!args.resource_group) throw new Error("resource_group is required");

  return ["vm", "start", "--name", args.name, "--resource-group", args.resource_group];
}

function buildVmStopArgs(args) {
  if (!args.name) throw new Error("name is required");
  if (!args.resource_group) throw new Error("resource_group is required");

  return ["vm", "deallocate", "--name", args.name, "--resource-group", args.resource_group];
}

function buildVmRestartArgs(args) {
  if (!args.name) throw new Error("name is required");
  if (!args.resource_group) throw new Error("resource_group is required");

  return ["vm", "restart", "--name", args.name, "--resource-group", args.resource_group];
}

function buildVmRunCommandArgs(args) {
  if (!args.name) throw new Error("name is required");
  if (!args.resource_group) throw new Error("resource_group is required");
  if (!args.command) throw new Error("command is required");

  return [
    "vm", "run-command", "invoke",
    "--name", args.name,
    "--resource-group", args.resource_group,
    "--command-id", "RunShellScript",
    "--scripts", args.command
  ];
}

function buildStorageListArgs(args) {
  const azArgs = ["storage", "account", "list", "--output", "json"];

  if (args.resource_group) {
    azArgs.push("--resource-group", args.resource_group);
  }

  return azArgs;
}

function buildStorageBlobListArgs(args) {
  if (!args.account) throw new Error("account is required");
  if (!args.container) throw new Error("container is required");

  const azArgs = [
    "storage", "blob", "list",
    "--account-name", args.account,
    "--container-name", args.container,
    "--output", "json",
    "--auth-mode", "login"
  ];

  if (args.prefix) {
    azArgs.push("--prefix", args.prefix);
  }

  return azArgs;
}

function buildStorageBlobUploadArgs(args) {
  if (!args.account) throw new Error("account is required");
  if (!args.container) throw new Error("container is required");
  if (!args.source) throw new Error("source is required");

  const azArgs = [
    "storage", "blob", "upload",
    "--account-name", args.account,
    "--container-name", args.container,
    "--file", args.source,
    "--auth-mode", "login"
  ];

  if (args.name) {
    azArgs.push("--name", args.name);
  }

  return azArgs;
}

function buildStorageBlobDownloadArgs(args) {
  if (!args.account) throw new Error("account is required");
  if (!args.container) throw new Error("container is required");
  if (!args.name) throw new Error("name is required");
  if (!args.destination) throw new Error("destination is required");

  return [
    "storage", "blob", "download",
    "--account-name", args.account,
    "--container-name", args.container,
    "--name", args.name,
    "--file", args.destination,
    "--auth-mode", "login"
  ];
}

function buildSqlListArgs(args) {
  const azArgs = ["sql", "server", "list", "--output", "json"];

  if (args.resource_group) {
    azArgs.push("--resource-group", args.resource_group);
  }

  return azArgs;
}

function buildSqlDbListArgs(args) {
  if (!args.server) throw new Error("server is required");
  if (!args.resource_group) throw new Error("resource_group is required");

  return [
    "sql", "db", "list",
    "--server", args.server,
    "--resource-group", args.resource_group,
    "--output", "json"
  ];
}

function buildAksListArgs(args) {
  const azArgs = ["aks", "list", "--output", "json"];

  if (args.resource_group) {
    azArgs.push("--resource-group", args.resource_group);
  }

  return azArgs;
}

function buildAksCredentialsArgs(args) {
  if (!args.name) throw new Error("name is required");
  if (!args.resource_group) throw new Error("resource_group is required");

  const azArgs = [
    "aks", "get-credentials",
    "--name", args.name,
    "--resource-group", args.resource_group
  ];

  if (args.admin === "true" || args.admin === true) {
    azArgs.push("--admin");
  }

  return azArgs;
}

function buildResourceListArgs(args) {
  const azArgs = ["resource", "list", "--output", "json"];

  if (args.resource_group) {
    azArgs.push("--resource-group", args.resource_group);
  }

  if (args.type) {
    azArgs.push("--resource-type", args.type);
  }

  return azArgs;
}

function buildMonitorMetricsArgs(args) {
  if (!args.resource) throw new Error("resource is required");
  if (!args.metric) throw new Error("metric is required");

  const azArgs = [
    "monitor", "metrics", "list",
    "--resource", args.resource,
    "--metric", args.metric,
    "--output", "json"
  ];

  if (args.interval) {
    azArgs.push("--interval", args.interval);
  }

  if (args.aggregation) {
    azArgs.push("--aggregation", args.aggregation);
  }

  return azArgs;
}

function runCommand(args) {
  const cli = globalThis.process?.env?.AZ_CMD || "az";
  const fullCmd = [cli, ...args].join(" ");
  return success(`Command: ${fullCmd}`);
}
