// Datadog Skill - Native dogshell CLI wrapper
// Provides Datadog monitoring through the dogshell CLI (pip install datadog)

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
    name: "datadog",
    version: "1.0.0",
    description: "Datadog monitoring and observability with native dogshell CLI integration",
    author: "Skill Engine"
  });
}

export function getTools() {
  return JSON.stringify([
    {
      name: "metric-query",
      description: "Query time-series metrics from Datadog",
      parameters: [
        { name: "query", paramType: "string", description: "Datadog metrics query", required: true },
        { name: "from", paramType: "string", description: "Start time (seconds or relative like -1h)", required: false },
        { name: "to", paramType: "string", description: "End time (default: now)", required: false }
      ]
    },
    {
      name: "monitor-list",
      description: "List Datadog monitors",
      parameters: [
        { name: "name", paramType: "string", description: "Filter by monitor name", required: false },
        { name: "tags", paramType: "string", description: "Filter by tags (comma-separated)", required: false },
        { name: "status", paramType: "string", description: "Filter by status: Alert, Warn, OK, No Data", required: false }
      ]
    },
    {
      name: "monitor-get",
      description: "Get details of a specific monitor",
      parameters: [
        { name: "id", paramType: "string", description: "Monitor ID", required: true }
      ]
    },
    {
      name: "monitor-mute",
      description: "Mute a monitor to silence alerts",
      parameters: [
        { name: "id", paramType: "string", description: "Monitor ID", required: true },
        { name: "scope", paramType: "string", description: "Scope to mute (e.g., host:web-01)", required: false },
        { name: "end", paramType: "string", description: "End time in seconds or duration", required: false }
      ]
    },
    {
      name: "monitor-unmute",
      description: "Unmute a muted monitor",
      parameters: [
        { name: "id", paramType: "string", description: "Monitor ID", required: true },
        { name: "scope", paramType: "string", description: "Scope to unmute", required: false }
      ]
    },
    {
      name: "event-post",
      description: "Post a custom event to Datadog",
      parameters: [
        { name: "title", paramType: "string", description: "Event title", required: true },
        { name: "text", paramType: "string", description: "Event description", required: true },
        { name: "alert_type", paramType: "string", description: "Type: info, warning, error, success", required: false, defaultValue: "info" },
        { name: "tags", paramType: "string", description: "Comma-separated tags", required: false }
      ]
    },
    {
      name: "host-list",
      description: "List hosts reporting to Datadog",
      parameters: [
        { name: "filter", paramType: "string", description: "Filter expression", required: false },
        { name: "count", paramType: "number", description: "Maximum hosts to return", required: false }
      ]
    },
    {
      name: "host-mute",
      description: "Mute a host",
      parameters: [
        { name: "hostname", paramType: "string", description: "Host name", required: true },
        { name: "message", paramType: "string", description: "Mute reason", required: false },
        { name: "end", paramType: "string", description: "End time in seconds or duration", required: false }
      ]
    },
    {
      name: "host-unmute",
      description: "Unmute a muted host",
      parameters: [
        { name: "hostname", paramType: "string", description: "Host name", required: true }
      ]
    },
    {
      name: "downtime-schedule",
      description: "Schedule a downtime",
      parameters: [
        { name: "scope", paramType: "string", description: "Scope for downtime", required: true },
        { name: "start", paramType: "string", description: "Start time in seconds", required: false },
        { name: "end", paramType: "string", description: "End time in seconds or duration", required: false },
        { name: "message", paramType: "string", description: "Downtime message", required: false }
      ]
    },
    {
      name: "downtime-list",
      description: "List scheduled downtimes",
      parameters: [
        { name: "current_only", paramType: "boolean", description: "Show only current downtimes", required: false }
      ]
    },
    {
      name: "dashboard-list",
      description: "List Datadog dashboards",
      parameters: []
    },
    {
      name: "service-check",
      description: "Post a service check",
      parameters: [
        { name: "check", paramType: "string", description: "Check name", required: true },
        { name: "host", paramType: "string", description: "Host name", required: true },
        { name: "status", paramType: "string", description: "Status: ok, warning, critical, unknown", required: true },
        { name: "message", paramType: "string", description: "Check message", required: false },
        { name: "tags", paramType: "string", description: "Comma-separated tags", required: false }
      ]
    }
  ]);
}

export function executeTool(toolName, argsJson) {
  try {
    const args = JSON.parse(argsJson);

    let dogArgs = [];

    switch (toolName) {
      case "metric-query":
        dogArgs = buildMetricQueryArgs(args);
        break;
      case "monitor-list":
        dogArgs = buildMonitorListArgs(args);
        break;
      case "monitor-get":
        dogArgs = buildMonitorGetArgs(args);
        break;
      case "monitor-mute":
        dogArgs = buildMonitorMuteArgs(args);
        break;
      case "monitor-unmute":
        dogArgs = buildMonitorUnmuteArgs(args);
        break;
      case "event-post":
        dogArgs = buildEventPostArgs(args);
        break;
      case "host-list":
        dogArgs = buildHostListArgs(args);
        break;
      case "host-mute":
        dogArgs = buildHostMuteArgs(args);
        break;
      case "host-unmute":
        dogArgs = buildHostUnmuteArgs(args);
        break;
      case "downtime-schedule":
        dogArgs = buildDowntimeScheduleArgs(args);
        break;
      case "downtime-list":
        dogArgs = buildDowntimeListArgs(args);
        break;
      case "dashboard-list":
        dogArgs = ["dashboard", "list"];
        break;
      case "service-check":
        dogArgs = buildServiceCheckArgs(args);
        break;
      default:
        return error(`Unknown tool: ${toolName}`);
    }

    return runCommand(dogArgs);
  } catch (e) {
    return error(`Error executing tool: ${e.message || e}`);
  }
}

export function validateConfig(configJson) {
  return JSON.stringify({ ok: null });
}

// Build functions for each tool

function buildMetricQueryArgs(args) {
  if (!args.query) throw new Error("query is required");

  const dogArgs = ["metric", "query"];

  // Calculate time range
  const now = Math.floor(Date.now() / 1000);
  let fromTime = now - 3600; // Default: 1 hour ago
  let toTime = now;

  if (args.from) {
    if (args.from.startsWith("-")) {
      // Relative time like -1h, -30m, -24h
      const match = args.from.match(/^-(\d+)([hmd])$/);
      if (match) {
        const value = parseInt(match[1]);
        const unit = match[2];
        const seconds = unit === "h" ? value * 3600 : unit === "m" ? value * 60 : value * 86400;
        fromTime = now - seconds;
      }
    } else {
      fromTime = parseInt(args.from);
    }
  }

  if (args.to && args.to !== "now") {
    toTime = parseInt(args.to);
  }

  dogArgs.push(String(fromTime), String(toTime), args.query);

  return dogArgs;
}

function buildMonitorListArgs(args) {
  const dogArgs = ["monitor", "show_all"];

  if (args.name) {
    dogArgs.push("--name", args.name);
  }

  if (args.tags) {
    dogArgs.push("--tags", args.tags);
  }

  if (args.status) {
    // Map status to monitor_tags format
    dogArgs.push("--monitor_tags", `status:${args.status.toLowerCase()}`);
  }

  return dogArgs;
}

function buildMonitorGetArgs(args) {
  if (!args.id) throw new Error("id is required");
  return ["monitor", "show", args.id];
}

function buildMonitorMuteArgs(args) {
  if (!args.id) throw new Error("id is required");

  const dogArgs = ["monitor", "mute", args.id];

  if (args.scope) {
    dogArgs.push("--scope", args.scope);
  }

  if (args.end) {
    dogArgs.push("--end", args.end);
  }

  return dogArgs;
}

function buildMonitorUnmuteArgs(args) {
  if (!args.id) throw new Error("id is required");

  const dogArgs = ["monitor", "unmute", args.id];

  if (args.scope) {
    dogArgs.push("--scope", args.scope);
  }

  return dogArgs;
}

function buildEventPostArgs(args) {
  if (!args.title) throw new Error("title is required");
  if (!args.text) throw new Error("text is required");

  const dogArgs = ["event", "post", args.title, args.text];

  const alertType = args.alert_type || "info";
  dogArgs.push("--alert_type", alertType);

  if (args.tags) {
    dogArgs.push("--tags", args.tags);
  }

  return dogArgs;
}

function buildHostListArgs(args) {
  const dogArgs = ["host", "list"];

  if (args.filter) {
    dogArgs.push("--filter", args.filter);
  }

  if (args.count) {
    dogArgs.push("--count", String(args.count));
  }

  return dogArgs;
}

function buildHostMuteArgs(args) {
  if (!args.hostname) throw new Error("hostname is required");

  const dogArgs = ["host", "mute", args.hostname];

  if (args.message) {
    dogArgs.push("--message", args.message);
  }

  if (args.end) {
    dogArgs.push("--end", args.end);
  }

  return dogArgs;
}

function buildHostUnmuteArgs(args) {
  if (!args.hostname) throw new Error("hostname is required");
  return ["host", "unmute", args.hostname];
}

function buildDowntimeScheduleArgs(args) {
  if (!args.scope) throw new Error("scope is required");

  const dogArgs = ["downtime", "schedule", args.scope];

  if (args.start) {
    dogArgs.push("--start", args.start);
  }

  if (args.end) {
    dogArgs.push("--end", args.end);
  }

  if (args.message) {
    dogArgs.push("--message", args.message);
  }

  return dogArgs;
}

function buildDowntimeListArgs(args) {
  const dogArgs = ["downtime", "show_all"];

  if (args.current_only === "true" || args.current_only === true) {
    dogArgs.push("--current_only");
  }

  return dogArgs;
}

function buildServiceCheckArgs(args) {
  if (!args.check) throw new Error("check is required");
  if (!args.host) throw new Error("host is required");
  if (!args.status) throw new Error("status is required");

  // Map status to numeric value
  const statusMap = { ok: 0, warning: 1, critical: 2, unknown: 3 };
  const statusNum = statusMap[args.status.toLowerCase()];
  if (statusNum === undefined) {
    throw new Error("status must be: ok, warning, critical, or unknown");
  }

  const dogArgs = ["service_check", "check", args.check, args.host, String(statusNum)];

  if (args.message) {
    dogArgs.push("--message", args.message);
  }

  if (args.tags) {
    dogArgs.push("--tags", args.tags);
  }

  return dogArgs;
}

function runCommand(args) {
  const cli = globalThis.process?.env?.DOG_CMD || "dog";
  const fullCmd = [cli, ...args].join(" ");
  return success(`Command: ${fullCmd}`);
}
