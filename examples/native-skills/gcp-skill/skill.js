// GCP Skill - Native gcloud CLI wrapper
// Provides Google Cloud Platform management through the gcloud CLI

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
    name: "gcp",
    version: "1.0.0",
    description: "Google Cloud Platform management with native gcloud CLI integration",
    author: "Skill Engine"
  });
}

export function getTools() {
  return JSON.stringify([
    {
      name: "compute-list",
      description: "List Compute Engine instances",
      parameters: [
        { name: "zone", paramType: "string", description: "Filter by zone (e.g., us-central1-a)", required: false },
        { name: "filter", paramType: "string", description: "Filter expression (e.g., status=RUNNING)", required: false },
        { name: "format", paramType: "string", description: "Output format: json, table, yaml", required: false, defaultValue: "json" }
      ]
    },
    {
      name: "compute-start",
      description: "Start a stopped Compute Engine instance",
      parameters: [
        { name: "instance", paramType: "string", description: "Instance name", required: true },
        { name: "zone", paramType: "string", description: "Instance zone", required: true }
      ]
    },
    {
      name: "compute-stop",
      description: "Stop a running Compute Engine instance",
      parameters: [
        { name: "instance", paramType: "string", description: "Instance name", required: true },
        { name: "zone", paramType: "string", description: "Instance zone", required: true }
      ]
    },
    {
      name: "compute-ssh",
      description: "Run a command on a Compute Engine instance via SSH",
      parameters: [
        { name: "instance", paramType: "string", description: "Instance name", required: true },
        { name: "zone", paramType: "string", description: "Instance zone", required: true },
        { name: "command", paramType: "string", description: "Command to execute", required: true }
      ]
    },
    {
      name: "storage-list",
      description: "List Cloud Storage buckets or objects",
      parameters: [
        { name: "bucket", paramType: "string", description: "Bucket name (lists buckets if omitted)", required: false },
        { name: "prefix", paramType: "string", description: "Object prefix filter", required: false }
      ]
    },
    {
      name: "storage-copy",
      description: "Copy files to/from Cloud Storage",
      parameters: [
        { name: "source", paramType: "string", description: "Source path (local or gs://)", required: true },
        { name: "destination", paramType: "string", description: "Destination path", required: true },
        { name: "recursive", paramType: "boolean", description: "Copy directories recursively", required: false }
      ]
    },
    {
      name: "sql-list",
      description: "List Cloud SQL instances",
      parameters: [
        { name: "format", paramType: "string", description: "Output format: json, table", required: false, defaultValue: "json" }
      ]
    },
    {
      name: "sql-databases",
      description: "List databases in a Cloud SQL instance",
      parameters: [
        { name: "instance", paramType: "string", description: "Cloud SQL instance name", required: true }
      ]
    },
    {
      name: "gke-list",
      description: "List Google Kubernetes Engine clusters",
      parameters: [
        { name: "region", paramType: "string", description: "Filter by region", required: false },
        { name: "format", paramType: "string", description: "Output format: json, table", required: false, defaultValue: "json" }
      ]
    },
    {
      name: "gke-credentials",
      description: "Get credentials for a GKE cluster (updates kubeconfig)",
      parameters: [
        { name: "cluster", paramType: "string", description: "Cluster name", required: true },
        { name: "zone", paramType: "string", description: "Cluster zone (for zonal clusters)", required: false },
        { name: "region", paramType: "string", description: "Cluster region (for regional clusters)", required: false }
      ]
    },
    {
      name: "functions-list",
      description: "List Cloud Functions",
      parameters: [
        { name: "region", paramType: "string", description: "Filter by region", required: false }
      ]
    },
    {
      name: "functions-logs",
      description: "View Cloud Function logs",
      parameters: [
        { name: "function", paramType: "string", description: "Function name", required: true },
        { name: "region", paramType: "string", description: "Function region", required: true },
        { name: "limit", paramType: "number", description: "Number of log entries", required: false, defaultValue: "50" }
      ]
    },
    {
      name: "projects-list",
      description: "List accessible GCP projects",
      parameters: []
    },
    {
      name: "iam-list",
      description: "List IAM policy bindings for a project",
      parameters: [
        { name: "project", paramType: "string", description: "Project ID (uses default if not specified)", required: false }
      ]
    }
  ]);
}

export function executeTool(toolName, argsJson) {
  try {
    const args = JSON.parse(argsJson);

    let gcloudArgs = [];

    switch (toolName) {
      case "compute-list":
        gcloudArgs = buildComputeListArgs(args);
        break;
      case "compute-start":
        gcloudArgs = buildComputeStartArgs(args);
        break;
      case "compute-stop":
        gcloudArgs = buildComputeStopArgs(args);
        break;
      case "compute-ssh":
        gcloudArgs = buildComputeSshArgs(args);
        break;
      case "storage-list":
        gcloudArgs = buildStorageListArgs(args);
        break;
      case "storage-copy":
        gcloudArgs = buildStorageCopyArgs(args);
        break;
      case "sql-list":
        gcloudArgs = buildSqlListArgs(args);
        break;
      case "sql-databases":
        gcloudArgs = buildSqlDatabasesArgs(args);
        break;
      case "gke-list":
        gcloudArgs = buildGkeListArgs(args);
        break;
      case "gke-credentials":
        gcloudArgs = buildGkeCredentialsArgs(args);
        break;
      case "functions-list":
        gcloudArgs = buildFunctionsListArgs(args);
        break;
      case "functions-logs":
        gcloudArgs = buildFunctionsLogsArgs(args);
        break;
      case "projects-list":
        gcloudArgs = ["projects", "list", "--format=json"];
        break;
      case "iam-list":
        gcloudArgs = buildIamListArgs(args);
        break;
      default:
        return error(`Unknown tool: ${toolName}`);
    }

    return runCommand(gcloudArgs);
  } catch (e) {
    return error(`Error executing tool: ${e.message || e}`);
  }
}

export function validateConfig(configJson) {
  return JSON.stringify({ ok: null });
}

// Build functions for each tool

function buildComputeListArgs(args) {
  const gcloudArgs = ["compute", "instances", "list"];

  if (args.zone) {
    gcloudArgs.push(`--zones=${args.zone}`);
  }

  if (args.filter) {
    gcloudArgs.push(`--filter=${args.filter}`);
  }

  const format = args.format || "json";
  gcloudArgs.push(`--format=${format}`);

  return gcloudArgs;
}

function buildComputeStartArgs(args) {
  if (!args.instance) {
    throw new Error("instance is required");
  }
  if (!args.zone) {
    throw new Error("zone is required");
  }

  return ["compute", "instances", "start", args.instance, `--zone=${args.zone}`];
}

function buildComputeStopArgs(args) {
  if (!args.instance) {
    throw new Error("instance is required");
  }
  if (!args.zone) {
    throw new Error("zone is required");
  }

  return ["compute", "instances", "stop", args.instance, `--zone=${args.zone}`];
}

function buildComputeSshArgs(args) {
  if (!args.instance) {
    throw new Error("instance is required");
  }
  if (!args.zone) {
    throw new Error("zone is required");
  }
  if (!args.command) {
    throw new Error("command is required");
  }

  return ["compute", "ssh", args.instance, `--zone=${args.zone}`, "--command", args.command];
}

function buildStorageListArgs(args) {
  if (args.bucket) {
    const gcloudArgs = ["storage", "ls"];
    let path = `gs://${args.bucket}`;
    if (args.prefix) {
      path += `/${args.prefix}`;
    }
    gcloudArgs.push(path);
    return gcloudArgs;
  }

  return ["storage", "buckets", "list", "--format=json"];
}

function buildStorageCopyArgs(args) {
  if (!args.source) {
    throw new Error("source is required");
  }
  if (!args.destination) {
    throw new Error("destination is required");
  }

  const gcloudArgs = ["storage", "cp"];

  if (args.recursive === "true" || args.recursive === true) {
    gcloudArgs.push("-r");
  }

  gcloudArgs.push(args.source, args.destination);

  return gcloudArgs;
}

function buildSqlListArgs(args) {
  const gcloudArgs = ["sql", "instances", "list"];
  const format = args.format || "json";
  gcloudArgs.push(`--format=${format}`);
  return gcloudArgs;
}

function buildSqlDatabasesArgs(args) {
  if (!args.instance) {
    throw new Error("instance is required");
  }

  return ["sql", "databases", "list", `--instance=${args.instance}`, "--format=json"];
}

function buildGkeListArgs(args) {
  const gcloudArgs = ["container", "clusters", "list"];

  if (args.region) {
    gcloudArgs.push(`--region=${args.region}`);
  }

  const format = args.format || "json";
  gcloudArgs.push(`--format=${format}`);

  return gcloudArgs;
}

function buildGkeCredentialsArgs(args) {
  if (!args.cluster) {
    throw new Error("cluster is required");
  }

  const gcloudArgs = ["container", "clusters", "get-credentials", args.cluster];

  if (args.region) {
    gcloudArgs.push(`--region=${args.region}`);
  } else if (args.zone) {
    gcloudArgs.push(`--zone=${args.zone}`);
  } else {
    throw new Error("Either zone or region is required");
  }

  return gcloudArgs;
}

function buildFunctionsListArgs(args) {
  const gcloudArgs = ["functions", "list", "--format=json"];

  if (args.region) {
    gcloudArgs.push(`--regions=${args.region}`);
  }

  return gcloudArgs;
}

function buildFunctionsLogsArgs(args) {
  if (!args.function) {
    throw new Error("function is required");
  }
  if (!args.region) {
    throw new Error("region is required");
  }

  const gcloudArgs = ["functions", "logs", "read", args.function, `--region=${args.region}`];

  const limit = args.limit || 50;
  gcloudArgs.push(`--limit=${limit}`);

  return gcloudArgs;
}

function buildIamListArgs(args) {
  const gcloudArgs = ["projects", "get-iam-policy"];

  if (args.project) {
    gcloudArgs.push(args.project);
  } else {
    // Use current project from config
    gcloudArgs.push("$(gcloud config get-value project)");
  }

  gcloudArgs.push("--format=json");

  return gcloudArgs;
}

function runCommand(args) {
  const cli = globalThis.process?.env?.GCLOUD_CMD || "gcloud";
  const fullCmd = [cli, ...args].join(" ");
  return success(`Command: ${fullCmd}`);
}
