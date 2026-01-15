/**
 * Docker Skill - Container and image management via docker CLI
 *
 * This skill executes real docker commands for container management.
 * It uses the docker CLI from the environment or default location.
 *
 * Requirements:
 * - docker must be installed and in PATH
 * - Docker daemon must be running
 * - Appropriate permissions to access Docker socket
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
    name: "docker",
    version: "1.0.0",
    description: "Docker container and image management with native docker CLI integration",
    author: "Skill Engine"
  });
}

// Define available tools
export function getTools() {
  return JSON.stringify([
    // Container Lifecycle Tools
    {
      name: "run",
      description: "Create and start a new container",
      parameters: [
        { name: "image", paramType: "string", description: "Container image name[:tag]", required: true },
        { name: "name", paramType: "string", description: "Container name", required: false },
        { name: "detach", paramType: "boolean", description: "Run in background", required: false, defaultValue: "false" },
        { name: "ports", paramType: "string", description: "Port mappings (e.g., '8080:80' or '8080:80,443:443')", required: false },
        { name: "volumes", paramType: "string", description: "Volume mounts (e.g., '/host:/container')", required: false },
        { name: "env", paramType: "string", description: "Environment variables (e.g., 'KEY=value,FOO=bar')", required: false },
        { name: "network", paramType: "string", description: "Network to connect to", required: false },
        { name: "rm", paramType: "boolean", description: "Remove container when it exits", required: false, defaultValue: "false" },
        { name: "command", paramType: "string", description: "Command to run in container", required: false }
      ]
    },
    {
      name: "exec",
      description: "Execute a command in a running container",
      parameters: [
        { name: "container", paramType: "string", description: "Container name or ID", required: true },
        { name: "command", paramType: "string", description: "Command to execute", required: true },
        { name: "interactive", paramType: "boolean", description: "Keep STDIN open", required: false, defaultValue: "false" },
        { name: "tty", paramType: "boolean", description: "Allocate pseudo-TTY", required: false, defaultValue: "false" },
        { name: "user", paramType: "string", description: "Username or UID", required: false },
        { name: "workdir", paramType: "string", description: "Working directory inside container", required: false }
      ]
    },
    {
      name: "logs",
      description: "Fetch container logs",
      parameters: [
        { name: "container", paramType: "string", description: "Container name or ID", required: true },
        { name: "tail", paramType: "number", description: "Number of lines to show from end", required: false },
        { name: "follow", paramType: "boolean", description: "Follow log output", required: false, defaultValue: "false" },
        { name: "since", paramType: "string", description: "Show logs since timestamp (e.g., '2h', '2024-01-01')", required: false },
        { name: "timestamps", paramType: "boolean", description: "Show timestamps", required: false, defaultValue: "false" }
      ]
    },
    {
      name: "ps",
      description: "List containers",
      parameters: [
        { name: "all", paramType: "boolean", description: "Show all containers including stopped", required: false, defaultValue: "false" },
        { name: "filter", paramType: "string", description: "Filter output (e.g., 'status=running')", required: false },
        { name: "format", paramType: "string", description: "Pretty-print using Go template", required: false },
        { name: "quiet", paramType: "boolean", description: "Only display container IDs", required: false, defaultValue: "false" }
      ]
    },
    {
      name: "start",
      description: "Start one or more stopped containers",
      parameters: [
        { name: "container", paramType: "string", description: "Container name(s) or ID(s), comma-separated", required: true },
        { name: "attach", paramType: "boolean", description: "Attach STDOUT/STDERR", required: false, defaultValue: "false" }
      ]
    },
    {
      name: "stop",
      description: "Stop one or more running containers",
      parameters: [
        { name: "container", paramType: "string", description: "Container name(s) or ID(s), comma-separated", required: true },
        { name: "time", paramType: "number", description: "Seconds to wait before killing (default: 10)", required: false }
      ]
    },
    {
      name: "rm",
      description: "Remove one or more containers",
      parameters: [
        { name: "container", paramType: "string", description: "Container name(s) or ID(s), comma-separated", required: true },
        { name: "force", paramType: "boolean", description: "Force removal of running container", required: false, defaultValue: "false" },
        { name: "volumes", paramType: "boolean", description: "Remove associated anonymous volumes", required: false, defaultValue: "false" }
      ]
    },
    {
      name: "restart",
      description: "Restart one or more containers",
      parameters: [
        { name: "container", paramType: "string", description: "Container name(s) or ID(s), comma-separated", required: true },
        { name: "time", paramType: "number", description: "Seconds to wait before killing (default: 10)", required: false }
      ]
    },
    {
      name: "inspect",
      description: "Display detailed information about a container or image",
      parameters: [
        { name: "target", paramType: "string", description: "Container or image name/ID", required: true },
        { name: "format", paramType: "string", description: "Format output using Go template", required: false },
        { name: "type", paramType: "string", description: "Type of object (container, image)", required: false }
      ]
    },
    // Image Management Tools
    {
      name: "images",
      description: "List images",
      parameters: [
        { name: "all", paramType: "boolean", description: "Show all images including intermediate", required: false, defaultValue: "false" },
        { name: "filter", paramType: "string", description: "Filter output (e.g., 'dangling=true')", required: false },
        { name: "format", paramType: "string", description: "Pretty-print using Go template", required: false },
        { name: "quiet", paramType: "boolean", description: "Only show image IDs", required: false, defaultValue: "false" }
      ]
    },
    {
      name: "pull",
      description: "Pull an image from a registry",
      parameters: [
        { name: "image", paramType: "string", description: "Image name[:tag]", required: true },
        { name: "platform", paramType: "string", description: "Set platform (e.g., 'linux/amd64')", required: false }
      ]
    },
    {
      name: "push",
      description: "Push an image to a registry",
      parameters: [
        { name: "image", paramType: "string", description: "Image name[:tag]", required: true }
      ]
    },
    {
      name: "build",
      description: "Build an image from a Dockerfile",
      parameters: [
        { name: "context", paramType: "string", description: "Build context path", required: true },
        { name: "file", paramType: "string", description: "Path to Dockerfile", required: false },
        { name: "tag", paramType: "string", description: "Image tag (e.g., 'myapp:v1.0')", required: false },
        { name: "build_arg", paramType: "string", description: "Build-time variables (e.g., 'VERSION=1.0,ENV=prod')", required: false },
        { name: "no_cache", paramType: "boolean", description: "Do not use cache", required: false, defaultValue: "false" },
        { name: "platform", paramType: "string", description: "Set target platform", required: false }
      ]
    },
    {
      name: "tag",
      description: "Create a tag for an image",
      parameters: [
        { name: "source", paramType: "string", description: "Source image name[:tag]", required: true },
        { name: "target", paramType: "string", description: "Target image name[:tag]", required: true }
      ]
    },
    {
      name: "rmi",
      description: "Remove one or more images",
      parameters: [
        { name: "image", paramType: "string", description: "Image name(s) or ID(s), comma-separated", required: true },
        { name: "force", paramType: "boolean", description: "Force removal", required: false, defaultValue: "false" }
      ]
    },
    // Network Tools
    {
      name: "network-ls",
      description: "List networks",
      parameters: [
        { name: "filter", paramType: "string", description: "Filter output (e.g., 'driver=bridge')", required: false },
        { name: "format", paramType: "string", description: "Pretty-print using Go template", required: false }
      ]
    },
    {
      name: "network-create",
      description: "Create a network",
      parameters: [
        { name: "name", paramType: "string", description: "Network name", required: true },
        { name: "driver", paramType: "string", description: "Network driver (bridge, overlay, host, none)", required: false },
        { name: "subnet", paramType: "string", description: "Subnet in CIDR format", required: false }
      ]
    },
    {
      name: "network-connect",
      description: "Connect a container to a network",
      parameters: [
        { name: "network", paramType: "string", description: "Network name", required: true },
        { name: "container", paramType: "string", description: "Container name or ID", required: true },
        { name: "ip", paramType: "string", description: "IPv4 address", required: false }
      ]
    },
    {
      name: "network-disconnect",
      description: "Disconnect a container from a network",
      parameters: [
        { name: "network", paramType: "string", description: "Network name", required: true },
        { name: "container", paramType: "string", description: "Container name or ID", required: true },
        { name: "force", paramType: "boolean", description: "Force disconnection", required: false, defaultValue: "false" }
      ]
    },
    // Volume Tools
    {
      name: "volume-ls",
      description: "List volumes",
      parameters: [
        { name: "filter", paramType: "string", description: "Filter output (e.g., 'dangling=true')", required: false },
        { name: "format", paramType: "string", description: "Pretty-print using Go template", required: false }
      ]
    },
    {
      name: "volume-create",
      description: "Create a volume",
      parameters: [
        { name: "name", paramType: "string", description: "Volume name", required: true },
        { name: "driver", paramType: "string", description: "Volume driver (default: local)", required: false }
      ]
    },
    {
      name: "volume-rm",
      description: "Remove one or more volumes",
      parameters: [
        { name: "name", paramType: "string", description: "Volume name(s), comma-separated", required: true },
        { name: "force", paramType: "boolean", description: "Force removal", required: false, defaultValue: "false" }
      ]
    },
    {
      name: "volume-inspect",
      description: "Display detailed information about a volume",
      parameters: [
        { name: "name", paramType: "string", description: "Volume name", required: true },
        { name: "format", paramType: "string", description: "Format output using Go template", required: false }
      ]
    },
    // Docker Compose Tools
    {
      name: "compose-up",
      description: "Create and start containers defined in docker-compose.yml",
      parameters: [
        { name: "file", paramType: "string", description: "Path to compose file", required: false },
        { name: "detach", paramType: "boolean", description: "Run in background", required: false, defaultValue: "true" },
        { name: "build", paramType: "boolean", description: "Build images before starting", required: false, defaultValue: "false" },
        { name: "services", paramType: "string", description: "Specific services to start, comma-separated", required: false }
      ]
    },
    {
      name: "compose-down",
      description: "Stop and remove containers, networks created by compose up",
      parameters: [
        { name: "file", paramType: "string", description: "Path to compose file", required: false },
        { name: "volumes", paramType: "boolean", description: "Remove named volumes", required: false, defaultValue: "false" },
        { name: "remove_orphans", paramType: "boolean", description: "Remove orphan containers", required: false, defaultValue: "false" }
      ]
    },
    {
      name: "compose-ps",
      description: "List containers managed by Compose",
      parameters: [
        { name: "file", paramType: "string", description: "Path to compose file", required: false },
        { name: "all", paramType: "boolean", description: "Show all containers including stopped", required: false, defaultValue: "false" }
      ]
    },
    {
      name: "compose-logs",
      description: "View output from containers",
      parameters: [
        { name: "file", paramType: "string", description: "Path to compose file", required: false },
        { name: "service", paramType: "string", description: "Specific service to show logs for", required: false },
        { name: "follow", paramType: "boolean", description: "Follow log output", required: false, defaultValue: "false" },
        { name: "tail", paramType: "number", description: "Number of lines to show", required: false }
      ]
    },
    // System Tools
    {
      name: "system-info",
      description: "Display system-wide information",
      parameters: []
    },
    {
      name: "system-prune",
      description: "Remove unused data (stopped containers, unused networks, dangling images)",
      parameters: [
        { name: "all", paramType: "boolean", description: "Remove all unused images, not just dangling", required: false, defaultValue: "false" },
        { name: "volumes", paramType: "boolean", description: "Also prune volumes", required: false, defaultValue: "false" },
        { name: "force", paramType: "boolean", description: "Do not prompt for confirmation", required: false, defaultValue: "true" }
      ]
    },
    {
      name: "raw",
      description: "Execute any docker command directly",
      parameters: [
        { name: "args", paramType: "string", description: "Raw docker arguments", required: true }
      ]
    }
  ]);
}

// Validate config (no specific config needed for docker)
export function validateConfig(configJson) {
  return JSON.stringify({ ok: null });
}

// Execute a tool
export function executeTool(toolName, argsJson) {
  try {
    const args = JSON.parse(argsJson);

    // Security validation
    const securityCheck = validateSecurity(toolName, args);
    if (securityCheck) {
      return error(securityCheck);
    }

    let dockerArgs;

    switch (toolName) {
      // Container Lifecycle
      case "run":
        dockerArgs = buildRunArgs(args);
        break;
      case "exec":
        dockerArgs = buildExecArgs(args);
        break;
      case "logs":
        dockerArgs = buildLogsArgs(args);
        break;
      case "ps":
        dockerArgs = buildPsArgs(args);
        break;
      case "start":
        dockerArgs = buildStartArgs(args);
        break;
      case "stop":
        dockerArgs = buildStopArgs(args);
        break;
      case "rm":
        dockerArgs = buildRmArgs(args);
        break;
      case "restart":
        dockerArgs = buildRestartArgs(args);
        break;
      case "inspect":
        dockerArgs = buildInspectArgs(args);
        break;

      // Image Management
      case "images":
        dockerArgs = buildImagesArgs(args);
        break;
      case "pull":
        dockerArgs = buildPullArgs(args);
        break;
      case "push":
        dockerArgs = buildPushArgs(args);
        break;
      case "build":
        dockerArgs = buildBuildArgs(args);
        break;
      case "tag":
        dockerArgs = buildTagArgs(args);
        break;
      case "rmi":
        dockerArgs = buildRmiArgs(args);
        break;

      // Network Tools
      case "network-ls":
        dockerArgs = buildNetworkLsArgs(args);
        break;
      case "network-create":
        dockerArgs = buildNetworkCreateArgs(args);
        break;
      case "network-connect":
        dockerArgs = buildNetworkConnectArgs(args);
        break;
      case "network-disconnect":
        dockerArgs = buildNetworkDisconnectArgs(args);
        break;

      // Volume Tools
      case "volume-ls":
        dockerArgs = buildVolumeLsArgs(args);
        break;
      case "volume-create":
        dockerArgs = buildVolumeCreateArgs(args);
        break;
      case "volume-rm":
        dockerArgs = buildVolumeRmArgs(args);
        break;
      case "volume-inspect":
        dockerArgs = buildVolumeInspectArgs(args);
        break;

      // Docker Compose
      case "compose-up":
        dockerArgs = buildComposeUpArgs(args);
        break;
      case "compose-down":
        dockerArgs = buildComposeDownArgs(args);
        break;
      case "compose-ps":
        dockerArgs = buildComposePsArgs(args);
        break;
      case "compose-logs":
        dockerArgs = buildComposeLogsArgs(args);
        break;

      // System
      case "system-info":
        dockerArgs = ["info"];
        break;
      case "system-prune":
        dockerArgs = buildSystemPruneArgs(args);
        break;
      case "raw":
        dockerArgs = args.args.split(" ");
        break;

      default:
        return error(`Unknown tool: ${toolName}`);
    }

    return runDocker(dockerArgs);
  } catch (e) {
    return error(`Error executing ${toolName}: ${e.message || e}`);
  }
}

// Security validation
function validateSecurity(toolName, args) {
  // Check for blocked flags in raw command
  if (toolName === "raw") {
    const rawArgs = args.args || "";
    if (rawArgs.includes("--privileged")) {
      return "Security: --privileged flag is blocked for security reasons";
    }
    if (rawArgs.includes("/var/run/docker.sock")) {
      return "Security: Mounting docker.sock is blocked for security reasons";
    }
  }

  // Check for blocked flags in run command
  if (toolName === "run") {
    const volumes = args.volumes || "";
    if (volumes.includes("/var/run/docker.sock")) {
      return "Security: Mounting docker.sock is blocked for security reasons";
    }
    if (volumes.match(/^\/:/) || volumes.includes(",/:")) {
      return "Security: Mounting root filesystem is blocked for security reasons";
    }
  }

  return null;
}

// Build argument arrays for each tool

function buildRunArgs(args) {
  const dockerArgs = ["run"];

  if (args.detach === "true" || args.detach === true) {
    dockerArgs.push("-d");
  }

  if (args.rm === "true" || args.rm === true) {
    dockerArgs.push("--rm");
  }

  if (args.name) {
    dockerArgs.push("--name", args.name);
  }

  if (args.ports) {
    const ports = args.ports.split(",");
    for (const port of ports) {
      dockerArgs.push("-p", port.trim());
    }
  }

  if (args.volumes) {
    const vols = args.volumes.split(",");
    for (const vol of vols) {
      dockerArgs.push("-v", vol.trim());
    }
  }

  if (args.env) {
    const envs = args.env.split(",");
    for (const env of envs) {
      dockerArgs.push("-e", env.trim());
    }
  }

  if (args.network) {
    dockerArgs.push("--network", args.network);
  }

  dockerArgs.push(args.image);

  if (args.command) {
    const cmdParts = args.command.split(" ");
    dockerArgs.push(...cmdParts);
  }

  return dockerArgs;
}

function buildExecArgs(args) {
  const dockerArgs = ["exec"];

  if (args.interactive === "true" || args.interactive === true) {
    dockerArgs.push("-i");
  }

  if (args.tty === "true" || args.tty === true) {
    dockerArgs.push("-t");
  }

  if (args.user) {
    dockerArgs.push("-u", args.user);
  }

  if (args.workdir) {
    dockerArgs.push("-w", args.workdir);
  }

  dockerArgs.push(args.container);

  const cmdParts = args.command.split(" ");
  dockerArgs.push(...cmdParts);

  return dockerArgs;
}

function buildLogsArgs(args) {
  const dockerArgs = ["logs"];

  if (args.tail) {
    dockerArgs.push("--tail", String(args.tail));
  }

  if (args.follow === "true" || args.follow === true) {
    dockerArgs.push("-f");
  }

  if (args.since) {
    dockerArgs.push("--since", args.since);
  }

  if (args.timestamps === "true" || args.timestamps === true) {
    dockerArgs.push("-t");
  }

  dockerArgs.push(args.container);

  return dockerArgs;
}

function buildPsArgs(args) {
  const dockerArgs = ["ps"];

  if (args.all === "true" || args.all === true) {
    dockerArgs.push("-a");
  }

  if (args.filter) {
    dockerArgs.push("-f", args.filter);
  }

  if (args.format) {
    dockerArgs.push("--format", args.format);
  }

  if (args.quiet === "true" || args.quiet === true) {
    dockerArgs.push("-q");
  }

  return dockerArgs;
}

function buildStartArgs(args) {
  const dockerArgs = ["start"];

  if (args.attach === "true" || args.attach === true) {
    dockerArgs.push("-a");
  }

  const containers = args.container.split(",");
  dockerArgs.push(...containers.map(c => c.trim()));

  return dockerArgs;
}

function buildStopArgs(args) {
  const dockerArgs = ["stop"];

  if (args.time !== undefined) {
    dockerArgs.push("-t", String(args.time));
  }

  const containers = args.container.split(",");
  dockerArgs.push(...containers.map(c => c.trim()));

  return dockerArgs;
}

function buildRmArgs(args) {
  const dockerArgs = ["rm"];

  if (args.force === "true" || args.force === true) {
    dockerArgs.push("-f");
  }

  if (args.volumes === "true" || args.volumes === true) {
    dockerArgs.push("-v");
  }

  const containers = args.container.split(",");
  dockerArgs.push(...containers.map(c => c.trim()));

  return dockerArgs;
}

function buildRestartArgs(args) {
  const dockerArgs = ["restart"];

  if (args.time !== undefined) {
    dockerArgs.push("-t", String(args.time));
  }

  const containers = args.container.split(",");
  dockerArgs.push(...containers.map(c => c.trim()));

  return dockerArgs;
}

function buildInspectArgs(args) {
  const dockerArgs = ["inspect"];

  if (args.format) {
    dockerArgs.push("-f", args.format);
  }

  if (args.type) {
    dockerArgs.push("--type", args.type);
  }

  dockerArgs.push(args.target);

  return dockerArgs;
}

function buildImagesArgs(args) {
  const dockerArgs = ["images"];

  if (args.all === "true" || args.all === true) {
    dockerArgs.push("-a");
  }

  if (args.filter) {
    dockerArgs.push("-f", args.filter);
  }

  if (args.format) {
    dockerArgs.push("--format", args.format);
  }

  if (args.quiet === "true" || args.quiet === true) {
    dockerArgs.push("-q");
  }

  return dockerArgs;
}

function buildPullArgs(args) {
  const dockerArgs = ["pull"];

  if (args.platform) {
    dockerArgs.push("--platform", args.platform);
  }

  dockerArgs.push(args.image);

  return dockerArgs;
}

function buildPushArgs(args) {
  return ["push", args.image];
}

function buildBuildArgs(args) {
  const dockerArgs = ["build"];

  if (args.file) {
    dockerArgs.push("-f", args.file);
  }

  if (args.tag) {
    dockerArgs.push("-t", args.tag);
  }

  if (args.build_arg) {
    const buildArgs = args.build_arg.split(",");
    for (const arg of buildArgs) {
      dockerArgs.push("--build-arg", arg.trim());
    }
  }

  if (args.no_cache === "true" || args.no_cache === true) {
    dockerArgs.push("--no-cache");
  }

  if (args.platform) {
    dockerArgs.push("--platform", args.platform);
  }

  dockerArgs.push(args.context);

  return dockerArgs;
}

function buildTagArgs(args) {
  return ["tag", args.source, args.target];
}

function buildRmiArgs(args) {
  const dockerArgs = ["rmi"];

  if (args.force === "true" || args.force === true) {
    dockerArgs.push("-f");
  }

  const images = args.image.split(",");
  dockerArgs.push(...images.map(i => i.trim()));

  return dockerArgs;
}

function buildNetworkLsArgs(args) {
  const dockerArgs = ["network", "ls"];

  if (args.filter) {
    dockerArgs.push("-f", args.filter);
  }

  if (args.format) {
    dockerArgs.push("--format", args.format);
  }

  return dockerArgs;
}

function buildNetworkCreateArgs(args) {
  const dockerArgs = ["network", "create"];

  if (args.driver) {
    dockerArgs.push("-d", args.driver);
  }

  if (args.subnet) {
    dockerArgs.push("--subnet", args.subnet);
  }

  dockerArgs.push(args.name);

  return dockerArgs;
}

function buildNetworkConnectArgs(args) {
  const dockerArgs = ["network", "connect"];

  if (args.ip) {
    dockerArgs.push("--ip", args.ip);
  }

  dockerArgs.push(args.network, args.container);

  return dockerArgs;
}

function buildNetworkDisconnectArgs(args) {
  const dockerArgs = ["network", "disconnect"];

  if (args.force === "true" || args.force === true) {
    dockerArgs.push("-f");
  }

  dockerArgs.push(args.network, args.container);

  return dockerArgs;
}

function buildVolumeLsArgs(args) {
  const dockerArgs = ["volume", "ls"];

  if (args.filter) {
    dockerArgs.push("-f", args.filter);
  }

  if (args.format) {
    dockerArgs.push("--format", args.format);
  }

  return dockerArgs;
}

function buildVolumeCreateArgs(args) {
  const dockerArgs = ["volume", "create"];

  if (args.driver) {
    dockerArgs.push("-d", args.driver);
  }

  dockerArgs.push(args.name);

  return dockerArgs;
}

function buildVolumeRmArgs(args) {
  const dockerArgs = ["volume", "rm"];

  if (args.force === "true" || args.force === true) {
    dockerArgs.push("-f");
  }

  const names = args.name.split(",");
  dockerArgs.push(...names.map(n => n.trim()));

  return dockerArgs;
}

function buildVolumeInspectArgs(args) {
  const dockerArgs = ["volume", "inspect"];

  if (args.format) {
    dockerArgs.push("-f", args.format);
  }

  dockerArgs.push(args.name);

  return dockerArgs;
}

function buildComposeUpArgs(args) {
  const dockerArgs = ["compose"];

  if (args.file) {
    dockerArgs.push("-f", args.file);
  }

  dockerArgs.push("up");

  if (args.detach === "true" || args.detach === true || args.detach === undefined) {
    dockerArgs.push("-d");
  }

  if (args.build === "true" || args.build === true) {
    dockerArgs.push("--build");
  }

  if (args.services) {
    const services = args.services.split(",");
    dockerArgs.push(...services.map(s => s.trim()));
  }

  return dockerArgs;
}

function buildComposeDownArgs(args) {
  const dockerArgs = ["compose"];

  if (args.file) {
    dockerArgs.push("-f", args.file);
  }

  dockerArgs.push("down");

  if (args.volumes === "true" || args.volumes === true) {
    dockerArgs.push("-v");
  }

  if (args.remove_orphans === "true" || args.remove_orphans === true) {
    dockerArgs.push("--remove-orphans");
  }

  return dockerArgs;
}

function buildComposePsArgs(args) {
  const dockerArgs = ["compose"];

  if (args.file) {
    dockerArgs.push("-f", args.file);
  }

  dockerArgs.push("ps");

  if (args.all === "true" || args.all === true) {
    dockerArgs.push("-a");
  }

  return dockerArgs;
}

function buildComposeLogsArgs(args) {
  const dockerArgs = ["compose"];

  if (args.file) {
    dockerArgs.push("-f", args.file);
  }

  dockerArgs.push("logs");

  if (args.follow === "true" || args.follow === true) {
    dockerArgs.push("-f");
  }

  if (args.tail) {
    dockerArgs.push("--tail", String(args.tail));
  }

  if (args.service) {
    dockerArgs.push(args.service);
  }

  return dockerArgs;
}

function buildSystemPruneArgs(args) {
  const dockerArgs = ["system", "prune"];

  if (args.all === "true" || args.all === true) {
    dockerArgs.push("-a");
  }

  if (args.volumes === "true" || args.volumes === true) {
    dockerArgs.push("--volumes");
  }

  // Default to force to avoid interactive prompts
  if (args.force === "true" || args.force === true || args.force === undefined) {
    dockerArgs.push("-f");
  }

  return dockerArgs;
}

// Run docker command
function runDocker(args) {
  const docker = globalThis.process?.env?.DOCKER_CMD || "docker";
  const fullCmd = [docker, ...args].join(" ");

  // Return the command for native execution
  return success(`Command: ${fullCmd}`);
}
