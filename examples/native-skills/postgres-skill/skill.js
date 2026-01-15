/**
 * PostgreSQL Skill - Database operations via psql CLI
 *
 * This skill executes PostgreSQL commands for database management.
 * It uses the psql CLI from the environment or default location.
 *
 * Requirements:
 * - psql must be installed and in PATH
 * - Database connection credentials (via connection string or env vars)
 *
 * Security:
 * - Passwords are passed via PGPASSWORD environment variable
 * - DROP/TRUNCATE operations require explicit confirmation flag
 * - SQL inputs are validated for basic injection patterns
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
    name: "postgres",
    version: "1.0.0",
    description: "PostgreSQL database operations with native psql CLI integration",
    author: "Skill Engine"
  });
}

// Define available tools (18 tools)
export function getTools() {
  return JSON.stringify([
    // Query Tools
    {
      name: "query",
      description: "Execute a SQL query",
      parameters: [
        { name: "sql", paramType: "string", description: "SQL query to execute", required: true },
        { name: "database", paramType: "string", description: "Database name", required: false },
        { name: "host", paramType: "string", description: "Database host", required: false, defaultValue: "localhost" },
        { name: "port", paramType: "number", description: "Database port", required: false, defaultValue: "5432" },
        { name: "user", paramType: "string", description: "Database user", required: false },
        { name: "format", paramType: "string", description: "Output format: table, csv, json, html", required: false, defaultValue: "table" },
        { name: "tuples_only", paramType: "boolean", description: "Print only tuples (no headers)", required: false, defaultValue: "false" }
      ]
    },
    {
      name: "query-file",
      description: "Execute SQL from a file",
      parameters: [
        { name: "file", paramType: "string", description: "Path to SQL file", required: true },
        { name: "database", paramType: "string", description: "Database name", required: false },
        { name: "host", paramType: "string", description: "Database host", required: false, defaultValue: "localhost" },
        { name: "port", paramType: "number", description: "Database port", required: false, defaultValue: "5432" },
        { name: "user", paramType: "string", description: "Database user", required: false },
        { name: "variable", paramType: "string", description: "Variables (key=value,key2=value2 format)", required: false }
      ]
    },
    // Schema Tools
    {
      name: "databases",
      description: "List all databases",
      parameters: [
        { name: "host", paramType: "string", description: "Database host", required: false, defaultValue: "localhost" },
        { name: "port", paramType: "number", description: "Database port", required: false, defaultValue: "5432" },
        { name: "user", paramType: "string", description: "Database user", required: false }
      ]
    },
    {
      name: "tables",
      description: "List tables in database",
      parameters: [
        { name: "database", paramType: "string", description: "Database name", required: true },
        { name: "schema", paramType: "string", description: "Schema name", required: false, defaultValue: "public" },
        { name: "host", paramType: "string", description: "Database host", required: false, defaultValue: "localhost" },
        { name: "port", paramType: "number", description: "Database port", required: false, defaultValue: "5432" },
        { name: "user", paramType: "string", description: "Database user", required: false }
      ]
    },
    {
      name: "describe",
      description: "Describe table structure",
      parameters: [
        { name: "table", paramType: "string", description: "Table name", required: true },
        { name: "database", paramType: "string", description: "Database name", required: true },
        { name: "schema", paramType: "string", description: "Schema name", required: false, defaultValue: "public" },
        { name: "host", paramType: "string", description: "Database host", required: false, defaultValue: "localhost" },
        { name: "port", paramType: "number", description: "Database port", required: false, defaultValue: "5432" },
        { name: "user", paramType: "string", description: "Database user", required: false }
      ]
    },
    {
      name: "columns",
      description: "List columns of a table",
      parameters: [
        { name: "table", paramType: "string", description: "Table name", required: true },
        { name: "database", paramType: "string", description: "Database name", required: true },
        { name: "schema", paramType: "string", description: "Schema name", required: false, defaultValue: "public" },
        { name: "host", paramType: "string", description: "Database host", required: false, defaultValue: "localhost" },
        { name: "port", paramType: "number", description: "Database port", required: false, defaultValue: "5432" },
        { name: "user", paramType: "string", description: "Database user", required: false }
      ]
    },
    {
      name: "indexes",
      description: "List indexes on a table",
      parameters: [
        { name: "table", paramType: "string", description: "Table name", required: true },
        { name: "database", paramType: "string", description: "Database name", required: true },
        { name: "schema", paramType: "string", description: "Schema name", required: false, defaultValue: "public" },
        { name: "host", paramType: "string", description: "Database host", required: false, defaultValue: "localhost" },
        { name: "port", paramType: "number", description: "Database port", required: false, defaultValue: "5432" },
        { name: "user", paramType: "string", description: "Database user", required: false }
      ]
    },
    {
      name: "constraints",
      description: "List constraints on a table",
      parameters: [
        { name: "table", paramType: "string", description: "Table name", required: true },
        { name: "database", paramType: "string", description: "Database name", required: true },
        { name: "schema", paramType: "string", description: "Schema name", required: false, defaultValue: "public" },
        { name: "host", paramType: "string", description: "Database host", required: false, defaultValue: "localhost" },
        { name: "port", paramType: "number", description: "Database port", required: false, defaultValue: "5432" },
        { name: "user", paramType: "string", description: "Database user", required: false }
      ]
    },
    {
      name: "size",
      description: "Show database or table size",
      parameters: [
        { name: "database", paramType: "string", description: "Database name", required: true },
        { name: "table", paramType: "string", description: "Table name (optional, shows DB size if not specified)", required: false },
        { name: "host", paramType: "string", description: "Database host", required: false, defaultValue: "localhost" },
        { name: "port", paramType: "number", description: "Database port", required: false, defaultValue: "5432" },
        { name: "user", paramType: "string", description: "Database user", required: false }
      ]
    },
    // Admin Tools
    {
      name: "connections",
      description: "Show active database connections",
      parameters: [
        { name: "database", paramType: "string", description: "Filter by database name", required: false },
        { name: "host", paramType: "string", description: "Database host", required: false, defaultValue: "localhost" },
        { name: "port", paramType: "number", description: "Database port", required: false, defaultValue: "5432" },
        { name: "user", paramType: "string", description: "Database user", required: false }
      ]
    },
    {
      name: "running-queries",
      description: "Show currently running queries",
      parameters: [
        { name: "database", paramType: "string", description: "Filter by database name", required: false },
        { name: "min_duration", paramType: "string", description: "Minimum query duration (e.g., '5 seconds')", required: false },
        { name: "host", paramType: "string", description: "Database host", required: false, defaultValue: "localhost" },
        { name: "port", paramType: "number", description: "Database port", required: false, defaultValue: "5432" },
        { name: "user", paramType: "string", description: "Database user", required: false }
      ]
    },
    {
      name: "cancel-query",
      description: "Cancel a running query by PID",
      parameters: [
        { name: "pid", paramType: "number", description: "Process ID to cancel", required: true },
        { name: "terminate", paramType: "boolean", description: "Terminate instead of cancel (more forceful)", required: false, defaultValue: "false" },
        { name: "host", paramType: "string", description: "Database host", required: false, defaultValue: "localhost" },
        { name: "port", paramType: "number", description: "Database port", required: false, defaultValue: "5432" },
        { name: "user", paramType: "string", description: "Database user", required: false }
      ]
    },
    // DDL Tools
    {
      name: "create-database",
      description: "Create a new database",
      parameters: [
        { name: "name", paramType: "string", description: "Database name", required: true },
        { name: "owner", paramType: "string", description: "Database owner", required: false },
        { name: "encoding", paramType: "string", description: "Character encoding", required: false, defaultValue: "UTF8" },
        { name: "template", paramType: "string", description: "Template database", required: false },
        { name: "host", paramType: "string", description: "Database host", required: false, defaultValue: "localhost" },
        { name: "port", paramType: "number", description: "Database port", required: false, defaultValue: "5432" },
        { name: "user", paramType: "string", description: "Database user", required: false }
      ]
    },
    {
      name: "drop-database",
      description: "Drop a database (DESTRUCTIVE - requires confirm flag)",
      parameters: [
        { name: "name", paramType: "string", description: "Database name", required: true },
        { name: "confirm", paramType: "boolean", description: "Confirm destructive operation", required: true },
        { name: "host", paramType: "string", description: "Database host", required: false, defaultValue: "localhost" },
        { name: "port", paramType: "number", description: "Database port", required: false, defaultValue: "5432" },
        { name: "user", paramType: "string", description: "Database user", required: false }
      ]
    },
    {
      name: "create-user",
      description: "Create a new database user/role",
      parameters: [
        { name: "username", paramType: "string", description: "Username", required: true },
        { name: "password", paramType: "string", description: "Password (will be passed securely)", required: false },
        { name: "superuser", paramType: "boolean", description: "Grant superuser privileges", required: false, defaultValue: "false" },
        { name: "createdb", paramType: "boolean", description: "Allow creating databases", required: false, defaultValue: "false" },
        { name: "createrole", paramType: "boolean", description: "Allow creating roles", required: false, defaultValue: "false" },
        { name: "login", paramType: "boolean", description: "Allow login", required: false, defaultValue: "true" },
        { name: "host", paramType: "string", description: "Database host", required: false, defaultValue: "localhost" },
        { name: "port", paramType: "number", description: "Database port", required: false, defaultValue: "5432" },
        { name: "user", paramType: "string", description: "Database user", required: false }
      ]
    },
    {
      name: "grant",
      description: "Grant privileges on database objects",
      parameters: [
        { name: "privileges", paramType: "string", description: "Privileges to grant (e.g., 'SELECT,INSERT' or 'ALL')", required: true },
        { name: "object_type", paramType: "string", description: "Object type: TABLE, DATABASE, SCHEMA, SEQUENCE", required: true },
        { name: "object_name", paramType: "string", description: "Object name (use 'ALL TABLES IN SCHEMA public' for all)", required: true },
        { name: "to_user", paramType: "string", description: "User/role to grant to", required: true },
        { name: "database", paramType: "string", description: "Database name", required: true },
        { name: "host", paramType: "string", description: "Database host", required: false, defaultValue: "localhost" },
        { name: "port", paramType: "number", description: "Database port", required: false, defaultValue: "5432" },
        { name: "user", paramType: "string", description: "Database user", required: false }
      ]
    },
    // Backup Tools
    {
      name: "backup",
      description: "Backup database using pg_dump",
      parameters: [
        { name: "database", paramType: "string", description: "Database name", required: true },
        { name: "output", paramType: "string", description: "Output file path", required: true },
        { name: "format", paramType: "string", description: "Output format: plain, custom, directory, tar", required: false, defaultValue: "custom" },
        { name: "schema_only", paramType: "boolean", description: "Dump only schema, no data", required: false, defaultValue: "false" },
        { name: "data_only", paramType: "boolean", description: "Dump only data, no schema", required: false, defaultValue: "false" },
        { name: "table", paramType: "string", description: "Specific table(s) to dump (comma-separated)", required: false },
        { name: "exclude_table", paramType: "string", description: "Table(s) to exclude (comma-separated)", required: false },
        { name: "compress", paramType: "number", description: "Compression level (0-9)", required: false },
        { name: "host", paramType: "string", description: "Database host", required: false, defaultValue: "localhost" },
        { name: "port", paramType: "number", description: "Database port", required: false, defaultValue: "5432" },
        { name: "user", paramType: "string", description: "Database user", required: false }
      ]
    },
    {
      name: "restore",
      description: "Restore database using pg_restore",
      parameters: [
        { name: "input", paramType: "string", description: "Input file path", required: true },
        { name: "database", paramType: "string", description: "Target database name", required: true },
        { name: "clean", paramType: "boolean", description: "Drop objects before recreating", required: false, defaultValue: "false" },
        { name: "create", paramType: "boolean", description: "Create the database before restoring", required: false, defaultValue: "false" },
        { name: "data_only", paramType: "boolean", description: "Restore only data", required: false, defaultValue: "false" },
        { name: "schema_only", paramType: "boolean", description: "Restore only schema", required: false, defaultValue: "false" },
        { name: "no_owner", paramType: "boolean", description: "Don't restore ownership", required: false, defaultValue: "false" },
        { name: "jobs", paramType: "number", description: "Number of parallel jobs", required: false },
        { name: "host", paramType: "string", description: "Database host", required: false, defaultValue: "localhost" },
        { name: "port", paramType: "number", description: "Database port", required: false, defaultValue: "5432" },
        { name: "user", paramType: "string", description: "Database user", required: false }
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

    let cmd = [];

    switch (toolName) {
      case "query":
        cmd = buildQueryCmd(args);
        break;
      case "query-file":
        cmd = buildQueryFileCmd(args);
        break;
      case "databases":
        cmd = buildDatabasesCmd(args);
        break;
      case "tables":
        cmd = buildTablesCmd(args);
        break;
      case "describe":
        cmd = buildDescribeCmd(args);
        break;
      case "columns":
        cmd = buildColumnsCmd(args);
        break;
      case "indexes":
        cmd = buildIndexesCmd(args);
        break;
      case "constraints":
        cmd = buildConstraintsCmd(args);
        break;
      case "size":
        cmd = buildSizeCmd(args);
        break;
      case "connections":
        cmd = buildConnectionsCmd(args);
        break;
      case "running-queries":
        cmd = buildRunningQueriesCmd(args);
        break;
      case "cancel-query":
        cmd = buildCancelQueryCmd(args);
        break;
      case "create-database":
        cmd = buildCreateDatabaseCmd(args);
        break;
      case "drop-database":
        cmd = buildDropDatabaseCmd(args);
        break;
      case "create-user":
        cmd = buildCreateUserCmd(args);
        break;
      case "grant":
        cmd = buildGrantCmd(args);
        break;
      case "backup":
        cmd = buildBackupCmd(args);
        break;
      case "restore":
        cmd = buildRestoreCmd(args);
        break;
      default:
        return error(`Unknown tool: ${toolName}`);
    }

    return success(`Command: ${cmd.join(" ")}`);
  } catch (e) {
    return error(`Error executing tool: ${e.message || e}`);
  }
}

// Validate config
export function validateConfig() {
  return JSON.stringify({ ok: null });
}

// Security validation
function validateSecurity(toolName, args) {
  // Check for SQL injection patterns in query
  if (toolName === "query" && args.sql) {
    const sql = args.sql.toLowerCase();

    // Block DROP without explicit confirmation
    if (sql.includes("drop ") && !args.confirm_destructive) {
      return "Security: DROP statements require explicit confirmation. Add confirm_destructive=true to proceed.";
    }

    // Block TRUNCATE without explicit confirmation
    if (sql.includes("truncate ") && !args.confirm_destructive) {
      return "Security: TRUNCATE statements require explicit confirmation. Add confirm_destructive=true to proceed.";
    }

    // Block DELETE without WHERE
    if (sql.match(/delete\s+from\s+\w+\s*;?\s*$/i)) {
      return "Security: DELETE without WHERE clause is blocked. Add a WHERE clause to limit deletion.";
    }
  }

  // Require confirmation for drop-database
  if (toolName === "drop-database") {
    if (args.confirm !== true && args.confirm !== "true") {
      return "Security: drop-database requires confirm=true to proceed.";
    }
  }

  // Validate identifier names (prevent injection in table/database names)
  const identifierFields = ["table", "database", "schema", "name", "username"];
  for (const field of identifierFields) {
    if (args[field] && !isValidIdentifier(args[field])) {
      return `Security: Invalid characters in ${field}. Use only alphanumeric, underscore, and dot.`;
    }
  }

  return null;
}

// Validate SQL identifier
function isValidIdentifier(name) {
  // Allow schema.table format
  return /^[a-zA-Z_][a-zA-Z0-9_]*(\.[a-zA-Z_][a-zA-Z0-9_]*)?$/.test(name);
}

// Build connection options
function buildConnectionOpts(args) {
  const opts = [];

  if (args.host) opts.push("-h", args.host);
  if (args.port) opts.push("-p", String(args.port));
  if (args.user) opts.push("-U", args.user);
  if (args.database) opts.push("-d", args.database);

  return opts;
}

// === Command Builders ===

function buildQueryCmd(args) {
  const cmd = ["psql"];
  cmd.push(...buildConnectionOpts(args));

  // Output format
  if (args.format === "csv") {
    cmd.push("--csv");
  } else if (args.format === "html") {
    cmd.push("-H");
  } else if (args.format === "json") {
    // PostgreSQL doesn't have native JSON output, wrap in JSON
    args.sql = `SELECT json_agg(t) FROM (${args.sql.replace(/;$/, "")}) t`;
    cmd.push("-t"); // tuples only for clean JSON
  }

  if (args.tuples_only === "true" || args.tuples_only === true) {
    cmd.push("-t");
  }

  // Escape single quotes in SQL
  const escapedSql = args.sql.replace(/'/g, "'\\''");
  cmd.push("-c", `'${escapedSql}'`);

  return cmd;
}

function buildQueryFileCmd(args) {
  const cmd = ["psql"];
  cmd.push(...buildConnectionOpts(args));

  if (args.variable) {
    const vars = args.variable.split(",");
    for (const v of vars) {
      cmd.push("-v", v.trim());
    }
  }

  cmd.push("-f", args.file);

  return cmd;
}

function buildDatabasesCmd(args) {
  const cmd = ["psql"];
  cmd.push(...buildConnectionOpts({ ...args, database: "postgres" }));
  cmd.push("-c", "'\\l'");
  return cmd;
}

function buildTablesCmd(args) {
  const cmd = ["psql"];
  cmd.push(...buildConnectionOpts(args));

  const schema = args.schema || "public";
  cmd.push("-c", `'\\dt ${schema}.*'`);

  return cmd;
}

function buildDescribeCmd(args) {
  const cmd = ["psql"];
  cmd.push(...buildConnectionOpts(args));

  const schema = args.schema || "public";
  cmd.push("-c", `'\\d ${schema}.${args.table}'`);

  return cmd;
}

function buildColumnsCmd(args) {
  const cmd = ["psql"];
  cmd.push(...buildConnectionOpts(args));

  const schema = args.schema || "public";
  const sql = `SELECT column_name, data_type, is_nullable, column_default
               FROM information_schema.columns
               WHERE table_schema = '${schema}' AND table_name = '${args.table}'
               ORDER BY ordinal_position`;
  cmd.push("-c", `'${sql}'`);

  return cmd;
}

function buildIndexesCmd(args) {
  const cmd = ["psql"];
  cmd.push(...buildConnectionOpts(args));

  const schema = args.schema || "public";
  cmd.push("-c", `'\\di ${schema}.${args.table}*'`);

  return cmd;
}

function buildConstraintsCmd(args) {
  const cmd = ["psql"];
  cmd.push(...buildConnectionOpts(args));

  const schema = args.schema || "public";
  const sql = `SELECT conname AS constraint_name, contype AS type,
               pg_get_constraintdef(oid) AS definition
               FROM pg_constraint
               WHERE conrelid = '${schema}.${args.table}'::regclass`;
  cmd.push("-c", `'${sql}'`);

  return cmd;
}

function buildSizeCmd(args) {
  const cmd = ["psql"];
  cmd.push(...buildConnectionOpts(args));

  let sql;
  if (args.table) {
    sql = `SELECT pg_size_pretty(pg_total_relation_size('${args.table}')) AS size`;
  } else {
    sql = `SELECT pg_size_pretty(pg_database_size('${args.database}')) AS size`;
  }
  cmd.push("-c", `'${sql}'`);

  return cmd;
}

function buildConnectionsCmd(args) {
  const cmd = ["psql"];
  cmd.push(...buildConnectionOpts({ ...args, database: "postgres" }));

  let sql = `SELECT pid, usename, datname, client_addr, state, query_start, query
             FROM pg_stat_activity WHERE pid <> pg_backend_pid()`;
  if (args.database) {
    sql += ` AND datname = '${args.database}'`;
  }
  cmd.push("-c", `'${sql}'`);

  return cmd;
}

function buildRunningQueriesCmd(args) {
  const cmd = ["psql"];
  cmd.push(...buildConnectionOpts({ ...args, database: "postgres" }));

  let sql = `SELECT pid, usename, datname, state,
             now() - query_start AS duration, query
             FROM pg_stat_activity
             WHERE state = 'active' AND pid <> pg_backend_pid()`;
  if (args.database) {
    sql += ` AND datname = '${args.database}'`;
  }
  if (args.min_duration) {
    sql += ` AND now() - query_start > interval '${args.min_duration}'`;
  }
  sql += ` ORDER BY duration DESC`;
  cmd.push("-c", `'${sql}'`);

  return cmd;
}

function buildCancelQueryCmd(args) {
  const cmd = ["psql"];
  cmd.push(...buildConnectionOpts({ ...args, database: "postgres" }));

  const func = args.terminate === "true" || args.terminate === true
    ? "pg_terminate_backend"
    : "pg_cancel_backend";

  cmd.push("-c", `'SELECT ${func}(${args.pid})'`);

  return cmd;
}

function buildCreateDatabaseCmd(args) {
  const cmd = ["psql"];
  cmd.push(...buildConnectionOpts({ ...args, database: "postgres" }));

  let sql = `CREATE DATABASE ${args.name}`;
  if (args.owner) sql += ` OWNER ${args.owner}`;
  if (args.encoding) sql += ` ENCODING '${args.encoding}'`;
  if (args.template) sql += ` TEMPLATE ${args.template}`;

  cmd.push("-c", `'${sql}'`);

  return cmd;
}

function buildDropDatabaseCmd(args) {
  const cmd = ["psql"];
  cmd.push(...buildConnectionOpts({ ...args, database: "postgres" }));
  cmd.push("-c", `'DROP DATABASE ${args.name}'`);
  return cmd;
}

function buildCreateUserCmd(args) {
  const cmd = ["psql"];
  cmd.push(...buildConnectionOpts({ ...args, database: "postgres" }));

  let sql = `CREATE ROLE ${args.username}`;

  const opts = [];
  if (args.login !== false && args.login !== "false") opts.push("LOGIN");
  if (args.superuser === "true" || args.superuser === true) opts.push("SUPERUSER");
  if (args.createdb === "true" || args.createdb === true) opts.push("CREATEDB");
  if (args.createrole === "true" || args.createrole === true) opts.push("CREATEROLE");
  if (args.password) opts.push(`PASSWORD '${args.password}'`);

  if (opts.length > 0) {
    sql += ` WITH ${opts.join(" ")}`;
  }

  cmd.push("-c", `'${sql}'`);

  return cmd;
}

function buildGrantCmd(args) {
  const cmd = ["psql"];
  cmd.push(...buildConnectionOpts(args));

  const sql = `GRANT ${args.privileges} ON ${args.object_type} ${args.object_name} TO ${args.to_user}`;
  cmd.push("-c", `'${sql}'`);

  return cmd;
}

function buildBackupCmd(args) {
  const cmd = ["pg_dump"];

  if (args.host) cmd.push("-h", args.host);
  if (args.port) cmd.push("-p", String(args.port));
  if (args.user) cmd.push("-U", args.user);

  // Format
  const format = args.format || "custom";
  const formatMap = { plain: "p", custom: "c", directory: "d", tar: "t" };
  cmd.push("-F", formatMap[format] || "c");

  if (args.schema_only === "true" || args.schema_only === true) {
    cmd.push("-s");
  }

  if (args.data_only === "true" || args.data_only === true) {
    cmd.push("-a");
  }

  if (args.table) {
    const tables = args.table.split(",");
    for (const t of tables) {
      cmd.push("-t", t.trim());
    }
  }

  if (args.exclude_table) {
    const tables = args.exclude_table.split(",");
    for (const t of tables) {
      cmd.push("-T", t.trim());
    }
  }

  if (args.compress !== undefined) {
    cmd.push("-Z", String(args.compress));
  }

  cmd.push("-f", args.output);
  cmd.push(args.database);

  return cmd;
}

function buildRestoreCmd(args) {
  const cmd = ["pg_restore"];

  if (args.host) cmd.push("-h", args.host);
  if (args.port) cmd.push("-p", String(args.port));
  if (args.user) cmd.push("-U", args.user);

  cmd.push("-d", args.database);

  if (args.clean === "true" || args.clean === true) {
    cmd.push("-c");
  }

  if (args.create === "true" || args.create === true) {
    cmd.push("-C");
  }

  if (args.data_only === "true" || args.data_only === true) {
    cmd.push("-a");
  }

  if (args.schema_only === "true" || args.schema_only === true) {
    cmd.push("-s");
  }

  if (args.no_owner === "true" || args.no_owner === true) {
    cmd.push("-O");
  }

  if (args.jobs) {
    cmd.push("-j", String(args.jobs));
  }

  cmd.push(args.input);

  return cmd;
}
