/**
 * Terraform Skill - Infrastructure as Code management via terraform CLI
 *
 * This skill executes real terraform commands for IaC management.
 * It uses the terraform CLI from the environment or default location.
 *
 * Requirements:
 * - terraform must be installed and in PATH
 * - Appropriate cloud provider credentials configured
 * - Valid Terraform configuration files in working directory
 */

// Helper functions for result formatting
function success(output) {
  return JSON.stringify({
    ok: { success: true, output, errorMessage: null }
  });
}

function error(message) {
  return JSON.stringify({ err: message });
}

// Define skill metadata
export function getMetadata() {
  return JSON.stringify({
    name: "terraform",
    version: "1.0.0",
    description: "Terraform infrastructure as code management with native CLI integration",
    author: "Skill Engine"
  });
}

// Define available tools (20 tools)
export function getTools() {
  return JSON.stringify([
    // Core Workflow Tools
    {
      name: "init",
      description: "Initialize a Terraform working directory",
      parameters: [
        { name: "backend", paramType: "boolean", description: "Configure backend", required: false, defaultValue: "true" },
        { name: "backend_config", paramType: "string", description: "Backend configuration (key=value format)", required: false },
        { name: "upgrade", paramType: "boolean", description: "Upgrade modules and plugins", required: false, defaultValue: "false" },
        { name: "reconfigure", paramType: "boolean", description: "Reconfigure backend, ignoring saved configuration", required: false, defaultValue: "false" },
        { name: "migrate_state", paramType: "boolean", description: "Migrate state to new backend", required: false, defaultValue: "false" }
      ]
    },
    {
      name: "plan",
      description: "Generate and show an execution plan",
      parameters: [
        { name: "out", paramType: "string", description: "Save plan to file", required: false },
        { name: "var", paramType: "string", description: "Variables (key=value,key2=value2 format)", required: false },
        { name: "var_file", paramType: "string", description: "Variable file path", required: false },
        { name: "target", paramType: "string", description: "Target specific resources (comma-separated)", required: false },
        { name: "destroy", paramType: "boolean", description: "Plan for destroy", required: false, defaultValue: "false" },
        { name: "refresh", paramType: "boolean", description: "Refresh state before planning", required: false, defaultValue: "true" },
        { name: "detailed_exitcode", paramType: "boolean", description: "Return detailed exit codes", required: false, defaultValue: "false" }
      ]
    },
    {
      name: "apply",
      description: "Apply changes to infrastructure",
      parameters: [
        { name: "plan_file", paramType: "string", description: "Apply a saved plan file", required: false },
        { name: "auto_approve", paramType: "boolean", description: "Skip interactive approval", required: false, defaultValue: "false" },
        { name: "var", paramType: "string", description: "Variables (key=value,key2=value2 format)", required: false },
        { name: "var_file", paramType: "string", description: "Variable file path", required: false },
        { name: "target", paramType: "string", description: "Target specific resources (comma-separated)", required: false },
        { name: "parallelism", paramType: "number", description: "Number of parallel operations", required: false },
        { name: "refresh", paramType: "boolean", description: "Refresh state before applying", required: false, defaultValue: "true" }
      ]
    },
    {
      name: "destroy",
      description: "Destroy Terraform-managed infrastructure",
      parameters: [
        { name: "auto_approve", paramType: "boolean", description: "Skip interactive approval", required: false, defaultValue: "false" },
        { name: "var", paramType: "string", description: "Variables (key=value,key2=value2 format)", required: false },
        { name: "var_file", paramType: "string", description: "Variable file path", required: false },
        { name: "target", paramType: "string", description: "Target specific resources (comma-separated)", required: false },
        { name: "parallelism", paramType: "number", description: "Number of parallel operations", required: false }
      ]
    },
    {
      name: "validate",
      description: "Validate the Terraform configuration files",
      parameters: [
        { name: "json", paramType: "boolean", description: "Output in JSON format", required: false, defaultValue: "false" }
      ]
    },
    {
      name: "fmt",
      description: "Format Terraform configuration files",
      parameters: [
        { name: "check", paramType: "boolean", description: "Check if files are formatted (no changes)", required: false, defaultValue: "false" },
        { name: "diff", paramType: "boolean", description: "Display diff of changes", required: false, defaultValue: "false" },
        { name: "recursive", paramType: "boolean", description: "Process subdirectories", required: false, defaultValue: "false" },
        { name: "write", paramType: "boolean", description: "Write changes to files", required: false, defaultValue: "true" }
      ]
    },
    {
      name: "output",
      description: "Show output values from state",
      parameters: [
        { name: "name", paramType: "string", description: "Specific output to show", required: false },
        { name: "json", paramType: "boolean", description: "Output in JSON format", required: false, defaultValue: "false" },
        { name: "raw", paramType: "boolean", description: "Output raw value (for single output)", required: false, defaultValue: "false" },
        { name: "state", paramType: "string", description: "Path to state file", required: false }
      ]
    },
    {
      name: "show",
      description: "Show current state or a saved plan",
      parameters: [
        { name: "plan_file", paramType: "string", description: "Show a saved plan file", required: false },
        { name: "json", paramType: "boolean", description: "Output in JSON format", required: false, defaultValue: "false" }
      ]
    },
    // State Management Tools
    {
      name: "state-list",
      description: "List resources in the state",
      parameters: [
        { name: "address", paramType: "string", description: "Filter by address pattern", required: false },
        { name: "state", paramType: "string", description: "Path to state file", required: false },
        { name: "id", paramType: "string", description: "Filter by resource ID", required: false }
      ]
    },
    {
      name: "state-show",
      description: "Show attributes of a single resource in the state",
      parameters: [
        { name: "address", paramType: "string", description: "Resource address", required: true },
        { name: "state", paramType: "string", description: "Path to state file", required: false }
      ]
    },
    {
      name: "state-mv",
      description: "Move a resource in the state",
      parameters: [
        { name: "source", paramType: "string", description: "Source resource address", required: true },
        { name: "destination", paramType: "string", description: "Destination resource address", required: true },
        { name: "state", paramType: "string", description: "Path to state file", required: false },
        { name: "dry_run", paramType: "boolean", description: "Preview the move without making changes", required: false, defaultValue: "false" }
      ]
    },
    {
      name: "state-rm",
      description: "Remove resources from the state",
      parameters: [
        { name: "address", paramType: "string", description: "Resource addresses (comma-separated)", required: true },
        { name: "state", paramType: "string", description: "Path to state file", required: false },
        { name: "dry_run", paramType: "boolean", description: "Preview the removal without making changes", required: false, defaultValue: "false" }
      ]
    },
    {
      name: "state-pull",
      description: "Pull current state and output to stdout",
      parameters: []
    },
    {
      name: "state-push",
      description: "Push local state to remote backend",
      parameters: [
        { name: "state_file", paramType: "string", description: "Path to state file to push", required: true },
        { name: "force", paramType: "boolean", description: "Force push even with newer remote state", required: false, defaultValue: "false" }
      ]
    },
    // Resource Management Tools
    {
      name: "import",
      description: "Import existing infrastructure into Terraform state",
      parameters: [
        { name: "address", paramType: "string", description: "Resource address to import into", required: true },
        { name: "id", paramType: "string", description: "Resource ID in the provider", required: true },
        { name: "var", paramType: "string", description: "Variables (key=value,key2=value2 format)", required: false },
        { name: "var_file", paramType: "string", description: "Variable file path", required: false },
        { name: "config", paramType: "string", description: "Path to Terraform configuration", required: false }
      ]
    },
    {
      name: "refresh",
      description: "Update local state file against real resources",
      parameters: [
        { name: "var", paramType: "string", description: "Variables (key=value,key2=value2 format)", required: false },
        { name: "var_file", paramType: "string", description: "Variable file path", required: false },
        { name: "target", paramType: "string", description: "Target specific resources (comma-separated)", required: false }
      ]
    },
    {
      name: "taint",
      description: "Mark a resource for recreation on next apply",
      parameters: [
        { name: "address", paramType: "string", description: "Resource address to taint", required: true },
        { name: "state", paramType: "string", description: "Path to state file", required: false }
      ]
    },
    {
      name: "untaint",
      description: "Remove the taint from a resource",
      parameters: [
        { name: "address", paramType: "string", description: "Resource address to untaint", required: true },
        { name: "state", paramType: "string", description: "Path to state file", required: false }
      ]
    },
    // Workspace Tools
    {
      name: "workspace-list",
      description: "List available workspaces",
      parameters: []
    },
    {
      name: "workspace-select",
      description: "Select or create a workspace",
      parameters: [
        { name: "name", paramType: "string", description: "Workspace name", required: true },
        { name: "create", paramType: "boolean", description: "Create workspace if it doesn't exist", required: false, defaultValue: "false" }
      ]
    }
  ]);
}

// Execute a tool
export function executeTool(toolName, argsJson) {
  try {
    const args = JSON.parse(argsJson);

    // Security validation
    const securityError = validateSecurity(toolName, args);
    if (securityError) {
      return error(securityError);
    }

    let tfArgs = [];

    switch (toolName) {
      case "init":
        tfArgs = buildInitArgs(args);
        break;
      case "plan":
        tfArgs = buildPlanArgs(args);
        break;
      case "apply":
        tfArgs = buildApplyArgs(args);
        break;
      case "destroy":
        tfArgs = buildDestroyArgs(args);
        break;
      case "validate":
        tfArgs = buildValidateArgs(args);
        break;
      case "fmt":
        tfArgs = buildFmtArgs(args);
        break;
      case "output":
        tfArgs = buildOutputArgs(args);
        break;
      case "show":
        tfArgs = buildShowArgs(args);
        break;
      case "state-list":
        tfArgs = buildStateListArgs(args);
        break;
      case "state-show":
        tfArgs = buildStateShowArgs(args);
        break;
      case "state-mv":
        tfArgs = buildStateMvArgs(args);
        break;
      case "state-rm":
        tfArgs = buildStateRmArgs(args);
        break;
      case "state-pull":
        tfArgs = ["state", "pull"];
        break;
      case "state-push":
        tfArgs = buildStatePushArgs(args);
        break;
      case "import":
        tfArgs = buildImportArgs(args);
        break;
      case "refresh":
        tfArgs = buildRefreshArgs(args);
        break;
      case "taint":
        tfArgs = buildTaintArgs(args);
        break;
      case "untaint":
        tfArgs = buildUntaintArgs(args);
        break;
      case "workspace-list":
        tfArgs = ["workspace", "list"];
        break;
      case "workspace-select":
        tfArgs = buildWorkspaceSelectArgs(args);
        break;
      default:
        return error(`Unknown tool: ${toolName}`);
    }

    return runTerraform(tfArgs);
  } catch (e) {
    return error(`Error executing tool: ${e.message || e}`);
  }
}

// Validate config (optional environment check)
export function validateConfig() {
  return JSON.stringify({ ok: null });
}

// Security validation
function validateSecurity(toolName, args) {
  // Block force push without explicit acknowledgment
  if (toolName === "state-push" && args.force === true) {
    // Allow but warn
    console.warn("Warning: Force pushing state. This may overwrite remote state.");
  }

  // Warn about destructive operations without auto_approve
  if (toolName === "destroy" && args.auto_approve === true) {
    // Allow but this is dangerous
    console.warn("Warning: Auto-approve enabled for destroy. All resources will be destroyed.");
  }

  // Block force-unlock (not implemented - too dangerous)
  // Note: force-unlock is intentionally not exposed as a tool

  return null;
}

// Run terraform command
function runTerraform(args) {
  const terraform = globalThis.process?.env?.TERRAFORM_CMD || "terraform";
  const fullCmd = [terraform, ...args].join(" ");

  // Return the command for host execution
  return success(`Command: ${fullCmd}`);
}

// === Argument Builders ===

function buildInitArgs(args) {
  const tfArgs = ["init"];

  if (args.backend === "false" || args.backend === false) {
    tfArgs.push("-backend=false");
  }

  if (args.backend_config) {
    const configs = args.backend_config.split(",");
    for (const config of configs) {
      tfArgs.push(`-backend-config=${config.trim()}`);
    }
  }

  if (args.upgrade === "true" || args.upgrade === true) {
    tfArgs.push("-upgrade");
  }

  if (args.reconfigure === "true" || args.reconfigure === true) {
    tfArgs.push("-reconfigure");
  }

  if (args.migrate_state === "true" || args.migrate_state === true) {
    tfArgs.push("-migrate-state");
  }

  // Always use input=false for non-interactive execution
  tfArgs.push("-input=false");

  return tfArgs;
}

function buildPlanArgs(args) {
  const tfArgs = ["plan"];

  if (args.out) {
    tfArgs.push(`-out=${args.out}`);
  }

  if (args.var) {
    const vars = args.var.split(",");
    for (const v of vars) {
      tfArgs.push(`-var=${v.trim()}`);
    }
  }

  if (args.var_file) {
    tfArgs.push(`-var-file=${args.var_file}`);
  }

  if (args.target) {
    const targets = args.target.split(",");
    for (const t of targets) {
      tfArgs.push(`-target=${t.trim()}`);
    }
  }

  if (args.destroy === "true" || args.destroy === true) {
    tfArgs.push("-destroy");
  }

  if (args.refresh === "false" || args.refresh === false) {
    tfArgs.push("-refresh=false");
  }

  if (args.detailed_exitcode === "true" || args.detailed_exitcode === true) {
    tfArgs.push("-detailed-exitcode");
  }

  tfArgs.push("-input=false");

  return tfArgs;
}

function buildApplyArgs(args) {
  const tfArgs = ["apply"];

  if (args.plan_file) {
    tfArgs.push(args.plan_file);
  } else {
    if (args.var) {
      const vars = args.var.split(",");
      for (const v of vars) {
        tfArgs.push(`-var=${v.trim()}`);
      }
    }

    if (args.var_file) {
      tfArgs.push(`-var-file=${args.var_file}`);
    }

    if (args.target) {
      const targets = args.target.split(",");
      for (const t of targets) {
        tfArgs.push(`-target=${t.trim()}`);
      }
    }
  }

  if (args.auto_approve === "true" || args.auto_approve === true) {
    tfArgs.push("-auto-approve");
  }

  if (args.parallelism) {
    tfArgs.push(`-parallelism=${args.parallelism}`);
  }

  if (args.refresh === "false" || args.refresh === false) {
    tfArgs.push("-refresh=false");
  }

  tfArgs.push("-input=false");

  return tfArgs;
}

function buildDestroyArgs(args) {
  const tfArgs = ["destroy"];

  if (args.var) {
    const vars = args.var.split(",");
    for (const v of vars) {
      tfArgs.push(`-var=${v.trim()}`);
    }
  }

  if (args.var_file) {
    tfArgs.push(`-var-file=${args.var_file}`);
  }

  if (args.target) {
    const targets = args.target.split(",");
    for (const t of targets) {
      tfArgs.push(`-target=${t.trim()}`);
    }
  }

  if (args.auto_approve === "true" || args.auto_approve === true) {
    tfArgs.push("-auto-approve");
  }

  if (args.parallelism) {
    tfArgs.push(`-parallelism=${args.parallelism}`);
  }

  tfArgs.push("-input=false");

  return tfArgs;
}

function buildValidateArgs(args) {
  const tfArgs = ["validate"];

  if (args.json === "true" || args.json === true) {
    tfArgs.push("-json");
  }

  return tfArgs;
}

function buildFmtArgs(args) {
  const tfArgs = ["fmt"];

  if (args.check === "true" || args.check === true) {
    tfArgs.push("-check");
  }

  if (args.diff === "true" || args.diff === true) {
    tfArgs.push("-diff");
  }

  if (args.recursive === "true" || args.recursive === true) {
    tfArgs.push("-recursive");
  }

  if (args.write === "false" || args.write === false) {
    tfArgs.push("-write=false");
  }

  return tfArgs;
}

function buildOutputArgs(args) {
  const tfArgs = ["output"];

  if (args.json === "true" || args.json === true) {
    tfArgs.push("-json");
  }

  if (args.raw === "true" || args.raw === true) {
    tfArgs.push("-raw");
  }

  if (args.state) {
    tfArgs.push(`-state=${args.state}`);
  }

  if (args.name) {
    tfArgs.push(args.name);
  }

  return tfArgs;
}

function buildShowArgs(args) {
  const tfArgs = ["show"];

  if (args.json === "true" || args.json === true) {
    tfArgs.push("-json");
  }

  if (args.plan_file) {
    tfArgs.push(args.plan_file);
  }

  return tfArgs;
}

function buildStateListArgs(args) {
  const tfArgs = ["state", "list"];

  if (args.state) {
    tfArgs.push(`-state=${args.state}`);
  }

  if (args.id) {
    tfArgs.push(`-id=${args.id}`);
  }

  if (args.address) {
    tfArgs.push(args.address);
  }

  return tfArgs;
}

function buildStateShowArgs(args) {
  const tfArgs = ["state", "show"];

  if (args.state) {
    tfArgs.push(`-state=${args.state}`);
  }

  tfArgs.push(args.address);

  return tfArgs;
}

function buildStateMvArgs(args) {
  const tfArgs = ["state", "mv"];

  if (args.state) {
    tfArgs.push(`-state=${args.state}`);
  }

  if (args.dry_run === "true" || args.dry_run === true) {
    tfArgs.push("-dry-run");
  }

  tfArgs.push(args.source, args.destination);

  return tfArgs;
}

function buildStateRmArgs(args) {
  const tfArgs = ["state", "rm"];

  if (args.state) {
    tfArgs.push(`-state=${args.state}`);
  }

  if (args.dry_run === "true" || args.dry_run === true) {
    tfArgs.push("-dry-run");
  }

  const addresses = args.address.split(",");
  tfArgs.push(...addresses.map(a => a.trim()));

  return tfArgs;
}

function buildStatePushArgs(args) {
  const tfArgs = ["state", "push"];

  if (args.force === "true" || args.force === true) {
    tfArgs.push("-force");
  }

  tfArgs.push(args.state_file);

  return tfArgs;
}

function buildImportArgs(args) {
  const tfArgs = ["import"];

  if (args.var) {
    const vars = args.var.split(",");
    for (const v of vars) {
      tfArgs.push(`-var=${v.trim()}`);
    }
  }

  if (args.var_file) {
    tfArgs.push(`-var-file=${args.var_file}`);
  }

  if (args.config) {
    tfArgs.push(`-config=${args.config}`);
  }

  tfArgs.push("-input=false");
  tfArgs.push(args.address, args.id);

  return tfArgs;
}

function buildRefreshArgs(args) {
  const tfArgs = ["refresh"];

  if (args.var) {
    const vars = args.var.split(",");
    for (const v of vars) {
      tfArgs.push(`-var=${v.trim()}`);
    }
  }

  if (args.var_file) {
    tfArgs.push(`-var-file=${args.var_file}`);
  }

  if (args.target) {
    const targets = args.target.split(",");
    for (const t of targets) {
      tfArgs.push(`-target=${t.trim()}`);
    }
  }

  tfArgs.push("-input=false");

  return tfArgs;
}

function buildTaintArgs(args) {
  const tfArgs = ["taint"];

  if (args.state) {
    tfArgs.push(`-state=${args.state}`);
  }

  tfArgs.push(args.address);

  return tfArgs;
}

function buildUntaintArgs(args) {
  const tfArgs = ["untaint"];

  if (args.state) {
    tfArgs.push(`-state=${args.state}`);
  }

  tfArgs.push(args.address);

  return tfArgs;
}

function buildWorkspaceSelectArgs(args) {
  const tfArgs = ["workspace"];

  if (args.create === "true" || args.create === true) {
    tfArgs.push("new");
  } else {
    tfArgs.push("select");
  }

  tfArgs.push(args.name);

  return tfArgs;
}
