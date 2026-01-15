//! MCP Server implementation - exposes skills as MCP tools to AI agents
//!
//! This server discovers installed skills and exposes them via the
//! Model Context Protocol (MCP), allowing AI agents like Claude to
//! discover and execute skill tools.

use anyhow::{Context, Result};
use rmcp::{
    ErrorData as McpError,
    handler::server::{
        router::{tool::ToolRoute, Router},
        ServerHandler,
    },
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion,
        ServerCapabilities, ServerInfo, Tool,
    },
    ServiceExt,
    transport::stdio,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use skill_runtime::{
    InstanceManager, LocalSkillLoader, SkillEngine, SkillExecutor, SkillManifest,
    SearchPipeline, IndexDocument, SearchConfig, DocumentMetadata,
};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Discovered skill tool information
#[derive(Debug, Clone)]
pub struct DiscoveredTool {
    pub skill_name: String,
    pub instance_name: String,
    pub tool_name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
    pub source_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ToolParameter {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub required: bool,
}

/// Request to execute a skill tool with context engineering features
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExecuteSkillRequest {
    /// The skill name (e.g., "kubernetes", "aws")
    #[schemars(description = "The skill name to execute")]
    pub skill: String,

    /// The tool name within the skill (e.g., "get", "describe")
    #[schemars(description = "The tool name within the skill")]
    pub tool: String,

    /// Instance name (default: "default")
    #[serde(default = "default_instance")]
    #[schemars(description = "The instance name (default: 'default')")]
    pub instance: String,

    /// Tool arguments as JSON object
    #[serde(default)]
    #[schemars(description = "Tool arguments as key-value pairs")]
    pub args: HashMap<String, serde_json::Value>,

    // === Context Engineering Options ===

    /// Maximum tokens/characters in output (default: unlimited)
    /// Use this to prevent context overflow from large outputs
    #[serde(default)]
    #[schemars(description = "Maximum characters in output. Use to prevent context overflow. Example: 4000")]
    pub max_output: Option<usize>,

    /// Truncation strategy when max_output is exceeded
    #[serde(default)]
    #[schemars(description = "How to truncate: 'head' (keep start), 'tail' (keep end), 'middle' (keep both ends), 'smart' (preserve structure)")]
    pub truncate: Option<String>,

    /// Grep/filter pattern to extract relevant lines from output
    #[serde(default)]
    #[schemars(description = "Regex pattern to filter output lines. Only matching lines are returned.")]
    pub grep: Option<String>,

    /// Invert grep match (like grep -v)
    #[serde(default)]
    #[schemars(description = "Invert grep: return lines that DON'T match the pattern")]
    pub grep_invert: Option<bool>,

    /// Return only first N lines (like head -n)
    #[serde(default)]
    #[schemars(description = "Return only first N lines of output")]
    pub head: Option<usize>,

    /// Return only last N lines (like tail -n)
    #[serde(default)]
    #[schemars(description = "Return only last N lines of output")]
    pub tail: Option<usize>,

    /// Output format transformation
    #[serde(default)]
    #[schemars(description = "Transform output: 'json' (parse as JSON), 'lines' (split into array), 'count' (line count only), 'summary' (AI summary)")]
    pub format: Option<String>,

    /// JSON path to extract (when output is JSON)
    #[serde(default)]
    #[schemars(description = "JSONPath expression to extract specific data from JSON output. Example: '.items[].metadata.name'")]
    pub jq: Option<String>,

    /// Include metadata about the execution
    #[serde(default)]
    #[schemars(description = "Include execution metadata (timing, truncation info, etc.)")]
    pub include_metadata: Option<bool>,
}

fn default_instance() -> String {
    "default".to_string()
}

/// Output processing result with context engineering metadata
#[derive(Debug, Serialize)]
struct ProcessedOutput {
    /// The processed output content
    content: String,
    /// Whether output was truncated
    truncated: bool,
    /// Original length before processing
    original_length: usize,
    /// Final length after processing
    final_length: usize,
    /// Number of lines matched by grep (if used)
    grep_matches: Option<usize>,
    /// Processing applied
    processing: Vec<String>,
}

/// Process tool output with context engineering transformations
fn process_output(
    output: &str,
    max_output: Option<usize>,
    truncate_strategy: Option<&str>,
    grep_pattern: Option<&str>,
    grep_invert: bool,
    head_lines: Option<usize>,
    tail_lines: Option<usize>,
    format: Option<&str>,
    jq_path: Option<&str>,
) -> ProcessedOutput {
    let original_length = output.len();
    let mut content = output.to_string();
    let mut processing = Vec::new();
    let mut truncated = false;
    let mut grep_matches = None;

    // Step 1: Apply grep filter first (most selective)
    if let Some(pattern) = grep_pattern {
        if let Ok(regex) = regex::Regex::new(pattern) {
            let lines: Vec<&str> = content.lines().collect();
            let filtered: Vec<&str> = lines
                .into_iter()
                .filter(|line| {
                    let matches = regex.is_match(line);
                    if grep_invert { !matches } else { matches }
                })
                .collect();
            grep_matches = Some(filtered.len());
            content = filtered.join("\n");
            processing.push(format!("grep(pattern='{}', invert={}, matches={})",
                pattern, grep_invert, grep_matches.unwrap_or(0)));
        }
    }

    // Step 2: Apply head/tail line limits
    if let Some(n) = head_lines {
        let lines: Vec<&str> = content.lines().take(n).collect();
        if content.lines().count() > n {
            truncated = true;
        }
        content = lines.join("\n");
        processing.push(format!("head({})", n));
    } else if let Some(n) = tail_lines {
        let all_lines: Vec<&str> = content.lines().collect();
        if all_lines.len() > n {
            truncated = true;
            content = all_lines[all_lines.len().saturating_sub(n)..].join("\n");
        }
        processing.push(format!("tail({})", n));
    }

    // Step 3: Apply jq path extraction for JSON
    if let Some(path) = jq_path {
        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&content) {
            content = extract_json_path(&json_val, path);
            processing.push(format!("jq('{}')", path));
        }
    }

    // Step 4: Apply format transformation
    if let Some(fmt) = format {
        match fmt {
            "json" => {
                // Try to parse and pretty-print JSON
                if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&content) {
                    content = serde_json::to_string_pretty(&json_val).unwrap_or(content);
                    processing.push("format(json)".to_string());
                }
            }
            "lines" => {
                // Convert to JSON array of lines
                let lines: Vec<&str> = content.lines().collect();
                content = serde_json::to_string(&lines).unwrap_or(content);
                processing.push("format(lines)".to_string());
            }
            "count" => {
                // Just return line count
                let count = content.lines().count();
                content = format!("{} lines", count);
                processing.push("format(count)".to_string());
            }
            "compact" => {
                // Remove empty lines and extra whitespace
                let lines: Vec<&str> = content.lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty())
                    .collect();
                content = lines.join("\n");
                processing.push("format(compact)".to_string());
            }
            _ => {}
        }
    }

    // Step 5: Apply max_output truncation last
    if let Some(max) = max_output {
        if content.len() > max {
            truncated = true;
            let strategy = truncate_strategy.unwrap_or("smart");
            content = truncate_content(&content, max, strategy);
            processing.push(format!("truncate({}, strategy='{}')", max, strategy));
        }
    }

    ProcessedOutput {
        final_length: content.len(),
        content,
        truncated,
        original_length,
        grep_matches,
        processing,
    }
}

/// Truncate content with different strategies
fn truncate_content(content: &str, max_len: usize, strategy: &str) -> String {
    if content.len() <= max_len {
        return content.to_string();
    }

    match strategy {
        "head" => {
            // Keep the beginning
            let truncated = &content[..max_len.saturating_sub(50)];
            format!("{}\n\n... [TRUNCATED: {} more characters]", truncated, content.len() - truncated.len())
        }
        "tail" => {
            // Keep the end
            let start = content.len().saturating_sub(max_len.saturating_sub(50));
            let truncated = &content[start..];
            format!("[TRUNCATED: {} characters omitted] ...\n\n{}", start, truncated)
        }
        "middle" => {
            // Keep both beginning and end
            let half = (max_len.saturating_sub(100)) / 2;
            let head = &content[..half];
            let tail = &content[content.len().saturating_sub(half)..];
            let omitted = content.len() - (head.len() + tail.len());
            format!("{}\n\n... [TRUNCATED: {} characters in middle] ...\n\n{}", head, omitted, tail)
        }
        "smart" | _ => {
            // Smart truncation: try to preserve structure (complete lines, JSON structure)
            smart_truncate(content, max_len)
        }
    }
}

/// Smart truncation that preserves structure
fn smart_truncate(content: &str, max_len: usize) -> String {
    // Check if it's JSON
    if content.trim().starts_with('{') || content.trim().starts_with('[') {
        return smart_truncate_json(content, max_len);
    }

    // For plain text, truncate at line boundaries
    let mut result = String::new();
    let mut remaining = max_len.saturating_sub(100); // Reserve space for truncation message
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();
    let mut included_lines = 0;

    for line in lines {
        if result.len() + line.len() + 1 > remaining {
            break;
        }
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(line);
        included_lines += 1;
    }

    if included_lines < total_lines {
        let omitted_lines = total_lines - included_lines;
        let omitted_chars = content.len() - result.len();
        result.push_str(&format!(
            "\n\n... [TRUNCATED: {} more lines, {} characters]\n\
             💡 Tip: Use grep='<pattern>' to filter, or head=N/tail=N to limit lines",
            omitted_lines, omitted_chars
        ));
    }

    result
}

/// Smart truncation for JSON content
fn smart_truncate_json(content: &str, max_len: usize) -> String {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
        // For arrays, truncate to fewer elements
        if let serde_json::Value::Array(arr) = &json {
            let total = arr.len();
            let mut truncated_arr = Vec::new();
            let mut current_len = 2; // For []

            for (idx, item) in arr.iter().enumerate() {
                let item_str = serde_json::to_string(item).unwrap_or_default();
                if current_len + item_str.len() + 2 > max_len.saturating_sub(150) {
                    // Add truncation notice
                    let notice = serde_json::json!({
                        "_truncated": true,
                        "_message": format!("... {} more items", total - idx),
                        "_total_items": total,
                        "_shown_items": idx,
                        "_tip": "Use jq='.items[0:10]' to select specific range, or grep to filter"
                    });
                    truncated_arr.push(notice);
                    break;
                }
                truncated_arr.push(item.clone());
                current_len += item_str.len() + 2;
            }

            return serde_json::to_string_pretty(&truncated_arr)
                .unwrap_or_else(|_| content[..max_len].to_string());
        }

        // For objects, try to pretty-print with truncation
        if let Ok(pretty) = serde_json::to_string_pretty(&json) {
            if pretty.len() <= max_len {
                return pretty;
            }
        }
    }

    // Fallback: simple truncation
    truncate_content(content, max_len, "head")
}

/// Extract data from JSON using a simple path expression
fn extract_json_path(json: &serde_json::Value, path: &str) -> String {
    let path = path.trim_start_matches('.');
    let parts: Vec<&str> = path.split('.').collect();

    let mut current = json.clone();
    for part in parts {
        if part.is_empty() {
            continue;
        }

        // Handle array access like "items[]" or "items[0]"
        if part.contains('[') {
            let (field, bracket) = part.split_once('[').unwrap_or((part, ""));

            // First get the field
            if !field.is_empty() {
                current = current.get(field).cloned().unwrap_or(serde_json::Value::Null);
            }

            // Then handle array access
            if bracket.starts_with(']') {
                // items[] - extract all items
                if let serde_json::Value::Array(arr) = current {
                    let extracted: Vec<String> = arr.iter()
                        .map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            other => serde_json::to_string(other).unwrap_or_default(),
                        })
                        .collect();
                    return extracted.join("\n");
                }
            } else if let Some(idx_str) = bracket.strip_suffix(']') {
                // items[0] or items[0:5]
                if idx_str.contains(':') {
                    // Range access items[0:5]
                    let range_parts: Vec<&str> = idx_str.split(':').collect();
                    if let (Ok(start), Ok(end)) = (
                        range_parts.get(0).unwrap_or(&"0").parse::<usize>(),
                        range_parts.get(1).unwrap_or(&"").parse::<usize>()
                    ) {
                        if let serde_json::Value::Array(arr) = current {
                            let sliced: Vec<_> = arr.iter().skip(start).take(end - start).cloned().collect();
                            return serde_json::to_string_pretty(&sliced).unwrap_or_default();
                        }
                    }
                } else if let Ok(idx) = idx_str.parse::<usize>() {
                    current = current.get(idx).cloned().unwrap_or(serde_json::Value::Null);
                }
            }
        } else {
            current = current.get(part).cloned().unwrap_or(serde_json::Value::Null);
        }
    }

    match current {
        serde_json::Value::String(s) => s,
        serde_json::Value::Null => "null".to_string(),
        other => serde_json::to_string_pretty(&other).unwrap_or_default(),
    }
}

/// Request to list available tools
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListSkillsRequest {
    /// Optional skill name to filter by
    #[schemars(description = "Optional skill name to filter tools by")]
    pub skill: Option<String>,

    /// Pagination offset (0-based index of first tool to return)
    #[serde(default)]
    #[schemars(description = "Pagination offset (0-based). Use with 'limit' to paginate through large tool lists.")]
    pub offset: Option<usize>,

    /// Maximum number of tools to return
    #[serde(default)]
    #[schemars(description = "Maximum number of tools to return (default: all). Use with 'offset' for pagination.")]
    pub limit: Option<usize>,
}

/// Response metadata for paginated results
#[derive(Debug, Serialize)]
pub struct PaginationInfo {
    pub total: usize,
    pub offset: usize,
    pub limit: Option<usize>,
    pub returned: usize,
    pub has_more: bool,
}

/// MCP Server that exposes skills as tools
#[derive(Clone)]
pub struct McpServer {
    /// Runtime engine for executing skills
    engine: Arc<SkillEngine>,
    /// Instance manager for configuration
    instance_manager: Arc<InstanceManager>,
    /// Local skill loader
    local_loader: Arc<LocalSkillLoader>,
    /// Discovered tools cache
    tools: Arc<RwLock<HashMap<String, DiscoveredTool>>>,
    /// Optional manifest for declarative skills
    manifest: Option<SkillManifest>,
    /// Search pipeline for semantic search (lazy initialized)
    search_pipeline: Arc<RwLock<Option<SearchPipeline>>>,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new() -> Result<Self> {
        let engine = Arc::new(SkillEngine::new()?);
        let instance_manager = Arc::new(InstanceManager::new()?);
        let local_loader = Arc::new(LocalSkillLoader::new()?);

        Ok(Self {
            engine,
            instance_manager,
            local_loader,
            tools: Arc::new(RwLock::new(HashMap::new())),
            manifest: None,
            search_pipeline: Arc::new(RwLock::new(None)),
        })
    }

    /// Create a new MCP server with a manifest
    pub fn with_manifest(manifest: SkillManifest) -> Result<Self> {
        let mut server = Self::new()?;
        server.manifest = Some(manifest);
        Ok(server)
    }

    /// Discover all available tools from installed skills and manifest
    pub async fn discover_tools(&self) -> Result<Vec<DiscoveredTool>> {
        let mut discovered = Vec::new();

        // Discover from installed skills
        let home = dirs::home_dir().context("Failed to get home directory")?;
        let registry_dir = home.join(".skill-engine").join("registry");

        if registry_dir.exists() {
            for entry in std::fs::read_dir(&registry_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    let skill_name = entry.file_name().to_string_lossy().to_string();
                    if let Ok(tools) = self.discover_skill_tools(&skill_name, None).await {
                        discovered.extend(tools);
                    }
                }
            }
        }

        // Discover from manifest
        if let Some(ref manifest) = self.manifest {
            for skill_name in manifest.skill_names() {
                if let Ok(resolved) = manifest.resolve_instance(skill_name, None) {
                    if let Ok(tools) = self
                        .discover_skill_tools_from_path(
                            skill_name,
                            &resolved.instance_name,
                            &PathBuf::from(&resolved.source),
                        )
                        .await
                    {
                        discovered.extend(tools);
                    }
                }
            }
        }

        // Update cache
        let mut cache = self.tools.write().await;
        for tool in &discovered {
            let key = format!("{}@{}:{}", tool.skill_name, tool.instance_name, tool.tool_name);
            cache.insert(key, tool.clone());
        }

        Ok(discovered)
    }

    /// Discover tools from an installed skill
    async fn discover_skill_tools(
        &self,
        skill_name: &str,
        instance_name: Option<&str>,
    ) -> Result<Vec<DiscoveredTool>> {
        let instances = self
            .instance_manager
            .list_instances(skill_name)
            .unwrap_or_default();

        let target_instances: Vec<String> = if let Some(name) = instance_name {
            vec![name.to_string()]
        } else if instances.is_empty() {
            vec!["default".to_string()]
        } else {
            instances
        };

        let mut tools = Vec::new();

        for instance in target_instances {
            // Try to find skill path
            let home = dirs::home_dir().context("Failed to get home directory")?;
            let skill_path = home
                .join(".skill-engine")
                .join("registry")
                .join(skill_name);

            if skill_path.exists() {
                if let Ok(skill_tools) =
                    self.discover_skill_tools_from_path(skill_name, &instance, &skill_path)
                        .await
                {
                    tools.extend(skill_tools);
                }
            }
        }

        Ok(tools)
    }

    /// Discover tools from a skill at a specific path
    async fn discover_skill_tools_from_path(
        &self,
        skill_name: &str,
        instance_name: &str,
        skill_path: &PathBuf,
    ) -> Result<Vec<DiscoveredTool>> {
        let mut tools = Vec::new();

        // Try to load SKILL.md for rich documentation
        if let Some(skill_md) = self.local_loader.load_skill_md(skill_path) {
            for (tool_name, tool_doc) in skill_md.tool_docs {
                let parameters: Vec<ToolParameter> = tool_doc
                    .parameters
                    .iter()
                    .map(|p| ToolParameter {
                        name: p.name.clone(),
                        param_type: "string".to_string(),
                        description: p.description.clone(),
                        required: p.required,
                    })
                    .collect();

                tools.push(DiscoveredTool {
                    skill_name: skill_name.to_string(),
                    instance_name: instance_name.to_string(),
                    tool_name,
                    description: tool_doc.description,
                    parameters,
                    source_path: Some(skill_path.clone()),
                });
            }
        }

        // If no tools found from SKILL.md, try to load from WASM
        if tools.is_empty() {
            if let Ok(_component) = self.local_loader.load_skill(skill_path, &self.engine).await {
                // Load instance config
                let config = self
                    .instance_manager
                    .load_instance(skill_name, instance_name)
                    .unwrap_or_default();

                // Create executor to get tool list
                if let Ok(executor) = SkillExecutor::load(
                    self.engine.clone(),
                    skill_path,
                    skill_name.to_string(),
                    instance_name.to_string(),
                    config,
                )
                .await
                {
                    if let Ok(skill_tools) = executor.get_tools().await {
                        for tool in skill_tools {
                            let parameters: Vec<ToolParameter> = tool
                                .parameters
                                .iter()
                                .map(|p| ToolParameter {
                                    name: p.name.clone(),
                                    param_type: format!("{:?}", p.param_type),
                                    description: p.description.clone(),
                                    required: p.required,
                                })
                                .collect();

                            tools.push(DiscoveredTool {
                                skill_name: skill_name.to_string(),
                                instance_name: instance_name.to_string(),
                                tool_name: tool.name,
                                description: tool.description,
                                parameters,
                                source_path: Some(skill_path.clone()),
                            });
                        }
                    }
                }
            }
        }

        Ok(tools)
    }

    /// Execute a skill tool
    pub async fn execute_skill_tool(
        &self,
        skill_name: &str,
        instance_name: &str,
        tool_name: &str,
        args: HashMap<String, serde_json::Value>,
    ) -> Result<skill_runtime::ExecutionResult> {
        // Find skill path
        let skill_path = if let Some(ref manifest) = self.manifest {
            if let Some(skill) = manifest.get_skill(skill_name) {
                let source = &skill.source;
                if source.starts_with("./") || source.starts_with("../") {
                    manifest.base_dir.join(source)
                } else {
                    PathBuf::from(source)
                }
            } else {
                let home = dirs::home_dir().context("Failed to get home directory")?;
                home.join(".skill-engine")
                    .join("registry")
                    .join(skill_name)
            }
        } else {
            let home = dirs::home_dir().context("Failed to get home directory")?;
            home.join(".skill-engine")
                .join("registry")
                .join(skill_name)
        };

        // Convert args to Vec<(String, String)>
        let args_vec: Vec<(String, String)> = args
            .iter()
            .map(|(k, v)| {
                let value = match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string().trim_matches('"').to_string(),
                };
                (k.clone(), value)
            })
            .collect();

        // Check if this is a native command skill (has SKILL.md but no WASM)
        // Try loading via WASM first, fall back to native command execution
        let wasm_path = self.find_wasm_in_path(&skill_path);

        if let Ok(wasm_file) = wasm_path {
            // WASM skill - execute via runtime
            let config = self
                .instance_manager
                .load_instance(skill_name, instance_name)
                .unwrap_or_default();

            let executor = SkillExecutor::load(
                self.engine.clone(),
                &wasm_file,  // Pass the actual WASM file path, not directory
                skill_name.to_string(),
                instance_name.to_string(),
                config,
            )
            .await?;

            let result = executor.execute_tool(tool_name, args_vec).await?;

            // Check if the WASM skill returns a native command to execute
            if result.success && result.output.starts_with("Command: ") {
                return self.execute_native_command(&result.output).await;
            }

            Ok(result)
        } else {
            // Native command skill - execute directly based on SKILL.md
            self.execute_native_skill(skill_name, tool_name, args_vec, &skill_path).await
        }
    }

    /// Find WASM file in a skill path
    fn find_wasm_in_path(&self, path: &PathBuf) -> Result<PathBuf> {
        // If it's a direct wasm file, return it
        if path.extension().map_or(false, |ext| ext == "wasm") && path.exists() {
            return Ok(path.clone());
        }

        // If it's a directory, search for wasm files
        if path.is_dir() {
            let candidates = vec![
                path.join("skill.wasm"),
                path.join("dist/skill.wasm"),
            ];

            for candidate in candidates {
                if candidate.exists() {
                    return Ok(candidate);
                }
            }
        }

        anyhow::bail!("No WASM file found in: {}", path.display())
    }

    /// Execute a native command skill (from SKILL.md)
    async fn execute_native_skill(
        &self,
        skill_name: &str,
        tool_name: &str,
        args: Vec<(String, String)>,
        skill_path: &PathBuf,
    ) -> Result<skill_runtime::ExecutionResult> {
        use std::process::Stdio;
        use tokio::process::Command;

        // Load SKILL.md to understand the tool's command pattern
        let skill_md = self.local_loader.load_skill_md(skill_path)
            .ok_or_else(|| anyhow::anyhow!("No SKILL.md found for native skill: {}", skill_name))?;

        // Build the command based on skill name and tool
        let command_str = self.build_native_command(skill_name, tool_name, &args, &skill_md)?;

        tracing::info!(command = %command_str, "Executing native command");

        // Parse the command
        let parts: Vec<&str> = command_str.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(skill_runtime::ExecutionResult {
                success: false,
                output: String::new(),
                error_message: Some("Empty command".to_string()),
                metadata: None,
            });
        }

        let program = parts[0];
        let cmd_args = &parts[1..];

        // Security check: Only allow specific commands
        let allowed_commands = ["kubectl", "helm", "git", "curl", "jq", "aws", "gcloud", "az", "docker", "terraform"];
        if !allowed_commands.contains(&program) {
            return Ok(skill_runtime::ExecutionResult {
                success: false,
                output: String::new(),
                error_message: Some(format!(
                    "Command '{}' not allowed. Allowed: {}",
                    program,
                    allowed_commands.join(", ")
                )),
                metadata: None,
            });
        }

        // Execute the command
        let result = Command::new(program)
            .args(cmd_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                if output.status.success() {
                    Ok(skill_runtime::ExecutionResult {
                        success: true,
                        output: stdout,
                        error_message: if stderr.is_empty() {
                            None
                        } else {
                            Some(stderr)
                        },
                        metadata: None,
                    })
                } else {
                    Ok(skill_runtime::ExecutionResult {
                        success: false,
                        output: stdout,
                        error_message: Some(if stderr.is_empty() {
                            format!("Command exited with status: {}", output.status)
                        } else {
                            stderr
                        }),
                        metadata: None,
                    })
                }
            }
            Err(e) => Ok(skill_runtime::ExecutionResult {
                success: false,
                output: String::new(),
                error_message: Some(format!("Failed to execute command: {}", e)),
                metadata: None,
            }),
        }
    }

    /// Build a native command from skill definition and arguments
    fn build_native_command(
        &self,
        skill_name: &str,
        tool_name: &str,
        args: &[(String, String)],
        skill_md: &skill_runtime::SkillMdContent,
    ) -> Result<String> {
        // Get the base command from the skill's allowed-tools or infer from name
        let base_command = match skill_name {
            "kubernetes" => "kubectl",
            "aws" => "aws",
            "docker" => "docker",
            "terraform" => "terraform",
            "helm" => "helm",
            _ => {
                // Try to get from allowed-tools in SKILL.md
                if let Some(ref allowed) = skill_md.frontmatter.allowed_tools {
                    allowed.split(',').next().unwrap_or(skill_name).trim()
                } else {
                    skill_name
                }
            }
        };

        // Build the command based on tool name and args
        let mut cmd_parts = vec![base_command.to_string()];

        // Map tool names to subcommands
        // For kubernetes skill
        if skill_name == "kubernetes" {
            match tool_name {
                "get" => {
                    cmd_parts.push("get".to_string());
                    for (key, value) in args {
                        match key.as_str() {
                            "resource" => cmd_parts.push(value.clone()),
                            "name" => cmd_parts.push(value.clone()),
                            "namespace" | "n" => {
                                cmd_parts.push("-n".to_string());
                                cmd_parts.push(value.clone());
                            }
                            "output" | "o" => {
                                cmd_parts.push("-o".to_string());
                                cmd_parts.push(value.clone());
                            }
                            "all-namespaces" | "A" => {
                                if value == "true" {
                                    cmd_parts.push("-A".to_string());
                                }
                            }
                            "selector" | "l" => {
                                cmd_parts.push("-l".to_string());
                                cmd_parts.push(value.clone());
                            }
                            _ => {}
                        }
                    }
                }
                "describe" => {
                    cmd_parts.push("describe".to_string());
                    for (key, value) in args {
                        match key.as_str() {
                            "resource" => cmd_parts.push(value.clone()),
                            "name" => cmd_parts.push(value.clone()),
                            "namespace" | "n" => {
                                cmd_parts.push("-n".to_string());
                                cmd_parts.push(value.clone());
                            }
                            _ => {}
                        }
                    }
                }
                "logs" => {
                    cmd_parts.push("logs".to_string());
                    for (key, value) in args {
                        match key.as_str() {
                            "pod" | "name" => cmd_parts.push(value.clone()),
                            "container" | "c" => {
                                cmd_parts.push("-c".to_string());
                                cmd_parts.push(value.clone());
                            }
                            "namespace" | "n" => {
                                cmd_parts.push("-n".to_string());
                                cmd_parts.push(value.clone());
                            }
                            "tail" => {
                                cmd_parts.push("--tail".to_string());
                                cmd_parts.push(value.clone());
                            }
                            "follow" | "f" => {
                                if value == "true" {
                                    cmd_parts.push("-f".to_string());
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "cluster-info" => {
                    cmd_parts.push("cluster-info".to_string());
                }
                "config" => {
                    cmd_parts.push("config".to_string());
                    for (key, value) in args {
                        match key.as_str() {
                            "subcommand" => cmd_parts.push(value.clone()),
                            "context" => cmd_parts.push(value.clone()),
                            _ => {}
                        }
                    }
                }
                "create" => {
                    cmd_parts.push("create".to_string());
                    for (key, value) in args {
                        match key.as_str() {
                            "resource" => cmd_parts.push(value.clone()),
                            "name" => cmd_parts.push(value.clone()),
                            "namespace" | "n" => {
                                cmd_parts.push("-n".to_string());
                                cmd_parts.push(value.clone());
                            }
                            "image" => {
                                cmd_parts.push("--image".to_string());
                                cmd_parts.push(value.clone());
                            }
                            _ => {}
                        }
                    }
                }
                "delete" => {
                    cmd_parts.push("delete".to_string());
                    for (key, value) in args {
                        match key.as_str() {
                            "resource" => cmd_parts.push(value.clone()),
                            "name" => cmd_parts.push(value.clone()),
                            "namespace" | "n" => {
                                cmd_parts.push("-n".to_string());
                                cmd_parts.push(value.clone());
                            }
                            _ => {}
                        }
                    }
                }
                "scale" => {
                    cmd_parts.push("scale".to_string());
                    let mut resource_set = false;
                    for (key, value) in args {
                        match key.as_str() {
                            "resource" => {
                                cmd_parts.push(value.clone());
                                resource_set = true;
                            }
                            "name" => {
                                if resource_set {
                                    // Append name to last element
                                    if let Some(last) = cmd_parts.last_mut() {
                                        last.push('/');
                                        last.push_str(value);
                                    }
                                } else {
                                    cmd_parts.push(value.clone());
                                }
                            }
                            "replicas" => {
                                cmd_parts.push(format!("--replicas={}", value));
                            }
                            "namespace" | "n" => {
                                cmd_parts.push("-n".to_string());
                                cmd_parts.push(value.clone());
                            }
                            _ => {}
                        }
                    }
                }
                "top" => {
                    cmd_parts.push("top".to_string());
                    for (key, value) in args {
                        match key.as_str() {
                            "resource" => cmd_parts.push(value.clone()),
                            "name" => cmd_parts.push(value.clone()),
                            "namespace" | "n" => {
                                cmd_parts.push("-n".to_string());
                                cmd_parts.push(value.clone());
                            }
                            _ => {}
                        }
                    }
                }
                "rollout" => {
                    cmd_parts.push("rollout".to_string());
                    for (key, value) in args {
                        match key.as_str() {
                            "action" => cmd_parts.push(value.clone()),
                            "resource" => cmd_parts.push(value.clone()),
                            "name" => cmd_parts.push(value.clone()),
                            "namespace" | "n" => {
                                cmd_parts.push("-n".to_string());
                                cmd_parts.push(value.clone());
                            }
                            _ => {}
                        }
                    }
                }
                "apply" => {
                    cmd_parts.push("apply".to_string());
                    for (key, value) in args {
                        match key.as_str() {
                            "file" | "f" => {
                                cmd_parts.push("-f".to_string());
                                cmd_parts.push(value.clone());
                            }
                            "namespace" | "n" => {
                                cmd_parts.push("-n".to_string());
                                cmd_parts.push(value.clone());
                            }
                            _ => {}
                        }
                    }
                }
                "exec" => {
                    cmd_parts.push("exec".to_string());
                    let mut pod_name = String::new();
                    let mut container = String::new();
                    let mut namespace = String::new();
                    let mut command = String::new();

                    for (key, value) in args {
                        match key.as_str() {
                            "pod" | "name" => pod_name = value.clone(),
                            "container" | "c" => container = value.clone(),
                            "namespace" | "n" => namespace = value.clone(),
                            "command" => command = value.clone(),
                            _ => {}
                        }
                    }

                    if !namespace.is_empty() {
                        cmd_parts.push("-n".to_string());
                        cmd_parts.push(namespace);
                    }
                    cmd_parts.push(pod_name);
                    if !container.is_empty() {
                        cmd_parts.push("-c".to_string());
                        cmd_parts.push(container);
                    }
                    cmd_parts.push("--".to_string());
                    cmd_parts.extend(command.split_whitespace().map(|s| s.to_string()));
                }
                "label" => {
                    cmd_parts.push("label".to_string());
                    for (key, value) in args {
                        match key.as_str() {
                            "resource" => cmd_parts.push(value.clone()),
                            "name" => cmd_parts.push(value.clone()),
                            "labels" => cmd_parts.push(value.clone()),
                            "namespace" | "n" => {
                                cmd_parts.push("-n".to_string());
                                cmd_parts.push(value.clone());
                            }
                            _ => {}
                        }
                    }
                }
                "annotate" => {
                    cmd_parts.push("annotate".to_string());
                    for (key, value) in args {
                        match key.as_str() {
                            "resource" => cmd_parts.push(value.clone()),
                            "name" => cmd_parts.push(value.clone()),
                            "annotations" => cmd_parts.push(value.clone()),
                            "namespace" | "n" => {
                                cmd_parts.push("-n".to_string());
                                cmd_parts.push(value.clone());
                            }
                            _ => {}
                        }
                    }
                }
                "cordon" => {
                    cmd_parts.push("cordon".to_string());
                    for (key, value) in args {
                        if key == "node" || key == "name" {
                            cmd_parts.push(value.clone());
                        }
                    }
                }
                "uncordon" => {
                    cmd_parts.push("uncordon".to_string());
                    for (key, value) in args {
                        if key == "node" || key == "name" {
                            cmd_parts.push(value.clone());
                        }
                    }
                }
                "drain" => {
                    cmd_parts.push("drain".to_string());
                    for (key, value) in args {
                        match key.as_str() {
                            "node" | "name" => cmd_parts.push(value.clone()),
                            "ignore-daemonsets" => {
                                if value == "true" {
                                    cmd_parts.push("--ignore-daemonsets".to_string());
                                }
                            }
                            "delete-emptydir-data" => {
                                if value == "true" {
                                    cmd_parts.push("--delete-emptydir-data".to_string());
                                }
                            }
                            "force" => {
                                if value == "true" {
                                    cmd_parts.push("--force".to_string());
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "taint" => {
                    cmd_parts.push("taint".to_string());
                    cmd_parts.push("nodes".to_string());
                    for (key, value) in args {
                        match key.as_str() {
                            "node" | "name" => cmd_parts.push(value.clone()),
                            "taint" => cmd_parts.push(value.clone()),
                            _ => {}
                        }
                    }
                }
                "raw" => {
                    // Direct kubectl command passthrough
                    cmd_parts.clear();
                    for (key, value) in args {
                        if key == "command" {
                            return Ok(value.clone());
                        }
                    }
                    return Err(anyhow::anyhow!("raw tool requires 'command' argument"));
                }
                _ => {
                    // Generic passthrough
                    cmd_parts.push(tool_name.to_string());
                    for (_, value) in args {
                        cmd_parts.push(value.clone());
                    }
                }
            }
        } else {
            // For other skills, just pass tool name and args
            cmd_parts.push(tool_name.to_string());
            for (_, value) in args {
                cmd_parts.push(value.clone());
            }
        }

        Ok(cmd_parts.join(" "))
    }

    /// Execute a native command from skill output
    async fn execute_native_command(
        &self,
        output: &str,
    ) -> Result<skill_runtime::ExecutionResult> {
        use std::process::Stdio;
        use tokio::process::Command;

        // Extract the command from "Command: kubectl ..."
        let first_line = output.lines().next().unwrap_or("");
        let command_str = first_line.strip_prefix("Command: ").unwrap_or(first_line);

        // Parse the command
        let parts: Vec<&str> = command_str.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(skill_runtime::ExecutionResult {
                success: false,
                output: String::new(),
                error_message: Some("Empty command".to_string()),
                metadata: None,
            });
        }

        let program = parts[0];
        let cmd_args = &parts[1..];

        // Security check: Only allow specific commands
        let allowed_commands = ["kubectl", "helm", "git", "curl", "jq", "aws", "gcloud", "az", "docker", "terraform"];
        if !allowed_commands.contains(&program) {
            return Ok(skill_runtime::ExecutionResult {
                success: false,
                output: String::new(),
                error_message: Some(format!(
                    "Command '{}' not allowed. Allowed: {}",
                    program,
                    allowed_commands.join(", ")
                )),
                metadata: None,
            });
        }

        tracing::info!(command = %command_str, "Executing native command");

        // Execute the command
        let result = Command::new(program)
            .args(cmd_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                if output.status.success() {
                    Ok(skill_runtime::ExecutionResult {
                        success: true,
                        output: stdout,
                        error_message: if stderr.is_empty() {
                            None
                        } else {
                            Some(stderr)
                        },
                        metadata: None,
                    })
                } else {
                    Ok(skill_runtime::ExecutionResult {
                        success: false,
                        output: stdout,
                        error_message: Some(if stderr.is_empty() {
                            format!("Command exited with status: {}", output.status)
                        } else {
                            stderr
                        }),
                        metadata: None,
                    })
                }
            }
            Err(e) => Ok(skill_runtime::ExecutionResult {
                success: false,
                output: String::new(),
                error_message: Some(format!("Failed to execute command: {}", e)),
                metadata: None,
            }),
        }
    }

    /// Get tools for list_skills response with optional pagination
    pub async fn list_skills_output(
        &self,
        filter_skill: Option<&str>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> String {
        let tools = self.tools.read().await;

        // Collect and filter tools
        let mut all_tools: Vec<&DiscoveredTool> = tools.values()
            .filter(|tool| {
                filter_skill.map_or(true, |filter| tool.skill_name == filter)
            })
            .collect();

        // Sort by skill name then tool name for consistent ordering
        all_tools.sort_by(|a, b| {
            (&a.skill_name, &a.tool_name).cmp(&(&b.skill_name, &b.tool_name))
        });

        let total = all_tools.len();
        let offset = offset.unwrap_or(0);

        // Apply pagination
        let paginated_tools: Vec<&DiscoveredTool> = if let Some(limit) = limit {
            all_tools.into_iter().skip(offset).take(limit).collect()
        } else {
            all_tools.into_iter().skip(offset).collect()
        };

        let returned = paginated_tools.len();
        let has_more = offset + returned < total;

        let mut output = String::new();

        if total == 0 {
            output.push_str("No skills found. Install skills with `skill install <source>`\n");
            return output;
        }

        // Add pagination info header
        output.push_str(&format!(
            "📊 **Pagination**: Showing {} of {} tools",
            returned, total
        ));
        if offset > 0 || limit.is_some() {
            output.push_str(&format!(" (offset: {}", offset));
            if let Some(l) = limit {
                output.push_str(&format!(", limit: {}", l));
            }
            output.push(')');
        }
        if has_more {
            let next_offset = offset + returned;
            output.push_str(&format!("\n💡 **Next page**: Use offset={}", next_offset));
        }
        output.push_str("\n\n");

        // Group tools by skill for display
        let mut grouped: HashMap<String, Vec<&DiscoveredTool>> = HashMap::new();
        for tool in paginated_tools {
            grouped.entry(tool.skill_name.clone()).or_default().push(tool);
        }

        output.push_str("Available Skills and Tools:\n\n");

        // Sort skill names for consistent output
        let mut skill_names: Vec<_> = grouped.keys().cloned().collect();
        skill_names.sort();

        for skill_name in skill_names {
            let skill_tools = grouped.get(&skill_name).unwrap();
            output.push_str(&format!("## {}\n", skill_name));
            for tool in skill_tools {
                output.push_str(&format!("  - **{}**: {}\n", tool.tool_name, tool.description));
                if !tool.parameters.is_empty() {
                    for param in &tool.parameters {
                        let req = if param.required { " (required)" } else { "" };
                        output.push_str(&format!("    - `{}`: {}{}\n", param.name, param.description, req));
                    }
                }
            }
            output.push('\n');
        }

        output
    }

    /// Search for skills using semantic vector search via SearchPipeline
    pub async fn search_skills(&self, query: &str, top_k: usize) -> Result<String> {
        let tools = self.tools.read().await;

        if tools.is_empty() {
            return Ok("No skills installed. Install skills with `skill install <source>`".to_string());
        }

        // Initialize SearchPipeline lazily
        let mut pipeline_lock = self.search_pipeline.write().await;
        if pipeline_lock.is_none() {
            let config = SearchConfig::default();
            let pipeline = SearchPipeline::from_config(config).await
                .map_err(|e| anyhow::anyhow!("Failed to create search pipeline: {}", e))?;
            *pipeline_lock = Some(pipeline);
        }
        let pipeline = pipeline_lock.as_ref().unwrap();

        // Build index documents from discovered tools with rich context
        let index_docs: Vec<IndexDocument> = tools.values().map(|t| {
            // Build rich text for better semantic matching
            let param_text = t.parameters.iter()
                .map(|p| {
                    let req = if p.required { "required" } else { "optional" };
                    format!("{} ({}, {}): {}", p.name, p.param_type, req, p.description)
                })
                .collect::<Vec<_>>()
                .join("; ");

            let full_text = format!(
                "Tool: {} | Description: {} | Skill: {} | Parameters: {}",
                t.tool_name,
                t.description,
                t.skill_name,
                if param_text.is_empty() { "none".to_string() } else { param_text }
            );

            IndexDocument {
                id: format!("{}@{}:{}", t.skill_name, t.instance_name, t.tool_name),
                content: full_text,
                metadata: DocumentMetadata {
                    skill_name: Some(t.skill_name.clone()),
                    instance_name: Some(t.instance_name.clone()),
                    tool_name: Some(t.tool_name.clone()),
                    category: None,
                    tags: Vec::new(),
                    custom: HashMap::new(),
                },
            }
        }).collect();

        // Index documents
        pipeline.index_documents(index_docs).await
            .map_err(|e| anyhow::anyhow!("Failed to index tools: {}", e))?;

        // Search
        let results = pipeline.search(query, top_k).await
            .map_err(|e| anyhow::anyhow!("Search failed: {}", e))?;

        // Format rich results for AI consumption
        let mut output = String::new();
        output.push_str(&format!("## 🔍 Search Results for: \"{}\"\n\n", query));

        if results.is_empty() {
            output.push_str("No matching tools found for your query.\n\n");
            output.push_str("**Suggestions:**\n");
            output.push_str("- Try different keywords\n");
            output.push_str("- Use `list_skills` to see all available tools\n");
            output.push_str("- Install more skills with `skill install <source>`\n");
        } else {
            output.push_str(&format!("Found **{}** relevant tools:\n\n", results.len()));

            for (i, result) in results.iter().enumerate() {
                let score_pct = (result.score * 100.0) as u32;
                let relevance = match score_pct {
                    80..=100 => "🟢 Excellent match",
                    60..=79 => "🟡 Good match",
                    40..=59 => "🟠 Fair match",
                    _ => "🔴 Partial match",
                };

                // Get full tool info from cache
                let tool_info = tools.get(&result.id);

                let skill_name = result.metadata.skill_name.as_deref().unwrap_or("unknown");
                let instance_name = result.metadata.instance_name.as_deref().unwrap_or("default");
                let tool_name = result.metadata.tool_name.as_deref().unwrap_or("unknown");

                output.push_str(&format!("---\n\n### {}. **{}** ({}% - {})\n\n",
                    i + 1, tool_name, score_pct, relevance));

                // Description
                if let Some(t) = tool_info {
                    output.push_str(&format!("**Description:** {}\n\n", t.description));

                    // Parameters section
                    if !t.parameters.is_empty() {
                        output.push_str("**Parameters:**\n");
                        for param in &t.parameters {
                            let req_badge = if param.required { "🔴 required" } else { "⚪ optional" };
                            output.push_str(&format!(
                                "- `{}` ({}) - {} [{}]\n",
                                param.name, param.param_type, param.description, req_badge
                            ));
                        }
                        output.push('\n');
                    }

                    // Execution signature
                    output.push_str("**How to Execute:**\n");
                    output.push_str("```json\n");
                    output.push_str("{\n");
                    output.push_str(&format!("  \"skill\": \"{}\",\n", skill_name));
                    output.push_str(&format!("  \"tool\": \"{}\",\n", tool_name));
                    output.push_str(&format!("  \"instance\": \"{}\",\n", instance_name));

                    if !t.parameters.is_empty() {
                        output.push_str("  \"args\": {\n");
                        for (idx, param) in t.parameters.iter().enumerate() {
                            let comma = if idx < t.parameters.len() - 1 { "," } else { "" };
                            let placeholder = match param.param_type.as_str() {
                                "string" => "\"<value>\"",
                                "number" | "integer" => "0",
                                "boolean" => "true",
                                _ => "\"<value>\"",
                            };
                            let comment = if param.required { " // required" } else { " // optional" };
                            output.push_str(&format!("    \"{}\": {}{}{}\n",
                                param.name, placeholder, comma, comment));
                        }
                        output.push_str("  }\n");
                    } else {
                        output.push_str("  \"args\": {}\n");
                    }
                    output.push_str("}\n");
                    output.push_str("```\n\n");
                } else {
                    // Fallback if tool info not in cache
                    output.push_str(&format!("**Skill:** {} | **Instance:** {}\n\n", skill_name, instance_name));
                    output.push_str(&format!(
                        "**Execute with:** `execute(skill='{}', tool='{}', instance='{}')`\n\n",
                        skill_name, tool_name, instance_name
                    ));
                }
            }

            // Summary and tips
            output.push_str("---\n\n");
            output.push_str("**💡 Tips:**\n");
            output.push_str("- Use `execute` tool with the JSON structure shown above\n");
            output.push_str("- Required parameters must be provided\n");
            output.push_str("- Use `list_skills` to see all available tools\n");
        }

        Ok(output)
    }

    /// Generate AI-powered examples for a skill's tools
    #[cfg(feature = "ai-ingestion")]
    pub async fn generate_examples(
        &self,
        skill_name: &str,
        tool_name: Option<&str>,
        _count: usize,
    ) -> Result<String> {
        use skill_runtime::{SearchConfig, SearchPipeline, GenerationEvent, IndexDocument, DocumentMetadata, parse_skill_md};
        use tokio_stream::StreamExt;

        // Get skill path
        let home = dirs::home_dir().context("Failed to get home directory")?;
        let skill_dir = home.join(".skill-engine").join("registry").join(skill_name);

        if !skill_dir.exists() {
            anyhow::bail!("Skill '{}' not found in registry", skill_name);
        }

        // Parse SKILL.md
        let skill_md_path = skill_dir.join("SKILL.md");
        if !skill_md_path.exists() {
            anyhow::bail!("No SKILL.md found for skill '{}'", skill_name);
        }

        let skill_md = parse_skill_md(&skill_md_path)
            .context("Failed to parse SKILL.md")?;

        // Filter tools if specific tool requested
        let tools: Vec<_> = skill_md.tool_docs.into_values()
            .filter(|t| tool_name.map_or(true, |name| t.name == name))
            .collect();

        if tools.is_empty() {
            if let Some(name) = tool_name {
                anyhow::bail!("Tool '{}' not found in skill '{}'", name, skill_name);
            }
            anyhow::bail!("No tools found in skill '{}'", skill_name);
        }

        // Load search config
        let config_path = home.join(".skill-engine").join("search.toml");
        let config = if config_path.exists() {
            SearchConfig::from_toml_file(&config_path)?
        } else {
            SearchConfig::default()
        };

        if !config.ai_ingestion.enabled {
            anyhow::bail!(
                "AI ingestion not enabled. Enable it with `skill setup` or \
                 edit ~/.skill-engine/search.toml"
            );
        }

        // Create pipeline
        let pipeline = SearchPipeline::from_config(config).await
            .context("Failed to create search pipeline")?;

        if !pipeline.has_example_generator() {
            anyhow::bail!("LLM provider not available. Check your AI ingestion configuration.");
        }

        // Build documents
        let documents: Vec<IndexDocument> = tools.iter()
            .map(|t| IndexDocument {
                id: format!("{}:{}", skill_name, t.name),
                content: format!(
                    "Tool: {}\nDescription: {}\nParameters: {}",
                    t.name, t.description,
                    t.parameters.iter()
                        .map(|p| format!("{} ({})", p.name, p.param_type))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                metadata: DocumentMetadata {
                    skill_name: Some(skill_name.to_string()),
                    tool_name: Some(t.name.clone()),
                    ..Default::default()
                },
            })
            .collect();

        // Stream generation and collect results
        let mut stream = Box::pin(pipeline.index_documents_stream(documents, tools.clone()));
        let mut all_examples = Vec::new();
        let mut output = String::new();

        output.push_str(&format!("## Generated Examples for {}\n\n", skill_name));

        while let Some(event) = stream.next().await {
            match event {
                GenerationEvent::Started { tool_name, .. } => {
                    output.push_str(&format!("### {}\n\n", tool_name));
                }
                GenerationEvent::Example { example } => {
                    all_examples.push(example.clone());
                    output.push_str(&format!(
                        "**Command:** `{}`\n**Explanation:** {}\n\n",
                        example.command, example.explanation
                    ));
                }
                GenerationEvent::ToolCompleted { examples_generated, valid_examples, .. } => {
                    output.push_str(&format!(
                        "_Generated {} examples ({} valid)_\n\n",
                        examples_generated, valid_examples
                    ));
                }
                GenerationEvent::Error { message, tool_name, .. } => {
                    let prefix = tool_name.map(|n| format!("[{}] ", n)).unwrap_or_default();
                    output.push_str(&format!("⚠️ {}Error: {}\n\n", prefix, message));
                }
                GenerationEvent::Completed { total_examples, total_valid, total_tools, .. } => {
                    output.push_str(&format!(
                        "---\n\n**Summary:** {} examples ({} valid) for {} tools\n",
                        total_examples, total_valid, total_tools
                    ));
                }
                _ => {}
            }
        }

        Ok(output)
    }

    /// Generate examples - stub when feature is disabled
    #[cfg(not(feature = "ai-ingestion"))]
    pub async fn generate_examples(
        &self,
        _skill_name: &str,
        _tool_name: Option<&str>,
        _count: usize,
    ) -> Result<String> {
        anyhow::bail!(
            "AI example generation not available. \
             Rebuild with --features ai-ingestion"
        )
    }

    /// Run the MCP server using stdio transport
    pub async fn run(self) -> Result<()> {
        tracing::info!("Starting MCP server with stdio transport");

        // Discover tools first
        let discovered = self.discover_tools().await?;
        tracing::info!("Discovered {} tools from skills", discovered.len());

        // Create the router with our tools
        let router = Router::new(self)
            .with_tool(execute_tool_route())
            .with_tool(list_skills_tool_route())
            .with_tool(search_skills_tool_route())
            .with_tool(generate_examples_tool_route());

        // Run with stdio transport
        // Note: Don't await the serve call, just await the waiting()
        router.serve(stdio())
            .await?
            .waiting()
            .await?;

        Ok(())
    }

    /// Run the MCP server using HTTP streaming transport (SSE)
    pub async fn run_http(host: &str, port: u16, manifest: Option<SkillManifest>) -> Result<()> {
        use rmcp::transport::streamable_http_server::{
            StreamableHttpService, session::local::LocalSessionManager,
        };

        tracing::info!("Starting MCP server with HTTP streaming at {}:{}", host, port);

        // Create factory function that creates a new server instance per session
        let manifest_clone = manifest.clone();
        let server_factory = move || -> std::result::Result<McpServer, std::io::Error> {
            let manifest = manifest_clone.clone();
            let server = if let Some(m) = manifest {
                McpServer::with_manifest(m)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
            } else {
                McpServer::new()
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
            };
            Ok(server)
        };

        // Create the streamable HTTP service
        let service = StreamableHttpService::new(
            server_factory,
            LocalSessionManager::default().into(),
            Default::default(),
        );

        // Create axum router
        let router = axum::Router::new().nest_service("/mcp", service);

        // Bind and serve
        let addr = format!("{}:{}", host, port);
        let tcp_listener = tokio::net::TcpListener::bind(&addr).await
            .map_err(|e| anyhow::anyhow!("Failed to bind to {}: {}", addr, e))?;

        tracing::info!("MCP HTTP server ready at http://{}/mcp", addr);

        axum::serve(tcp_listener, router)
            .with_graceful_shutdown(async {
                tokio::signal::ctrl_c().await.ok();
            })
            .await
            .map_err(|e| anyhow::anyhow!("HTTP server error: {}", e))?;

        Ok(())
    }
}

// ServerHandler implementation
impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "Skill Engine MCP Server - Execute installed skills and their tools. \
                 Use `list_skills` to discover available skills, then `execute` to run tools. \
                 Example: execute(skill='kubernetes', tool='get', args={resource: 'pods'})"
                    .to_string(),
            ),
        }
    }
}

// Tool route definitions

/// Create the execute tool route with context engineering features
fn execute_tool_route() -> ToolRoute<McpServer> {
    use futures::FutureExt;
    use rmcp::handler::server::tool::ToolCallContext;

    let execute_schema: serde_json::Map<String, serde_json::Value> = serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "skill": {
                "type": "string",
                "description": "The skill name to execute (e.g., 'kubernetes', 'aws')"
            },
            "tool": {
                "type": "string",
                "description": "The tool name within the skill (e.g., 'get', 'describe')"
            },
            "instance": {
                "type": "string",
                "description": "The instance name (default: 'default')",
                "default": "default"
            },
            "args": {
                "type": "object",
                "description": "Tool arguments as key-value pairs",
                "additionalProperties": true
            },
            // Context Engineering Options
            "max_output": {
                "type": "integer",
                "description": "Maximum characters in output to prevent context overflow (e.g., 4000 for ~1000 tokens)"
            },
            "truncate": {
                "type": "string",
                "enum": ["head", "tail", "middle", "smart"],
                "description": "Truncation strategy: 'head' (keep start), 'tail' (keep end), 'middle' (keep both ends), 'smart' (preserve structure, default)"
            },
            "grep": {
                "type": "string",
                "description": "Regex pattern to filter output lines. Only matching lines are returned. Example: 'error|warning'"
            },
            "grep_invert": {
                "type": "boolean",
                "description": "Invert grep match - return lines that DON'T match the pattern"
            },
            "head": {
                "type": "integer",
                "description": "Return only first N lines of output"
            },
            "tail": {
                "type": "integer",
                "description": "Return only last N lines of output"
            },
            "format": {
                "type": "string",
                "enum": ["json", "lines", "count", "compact"],
                "description": "Transform output: 'json' (pretty-print), 'lines' (array), 'count' (line count), 'compact' (remove whitespace)"
            },
            "jq": {
                "type": "string",
                "description": "JSONPath to extract from JSON output. Examples: '.items[].name', '.metadata', '.items[0:5]'"
            },
            "include_metadata": {
                "type": "boolean",
                "description": "Include execution metadata (timing, truncation info, original size)"
            }
        },
        "required": ["skill", "tool"]
    })).unwrap();

    let tool = Tool {
        name: Cow::Borrowed("execute"),
        title: None,
        description: Some(Cow::Borrowed(
            "Execute a skill tool with context engineering features. \
             Use max_output to limit response size, grep to filter, jq to extract JSON fields. \
             Examples:\n\
             - Basic: execute(skill='k8s', tool='get', args={resource:'pods'})\n\
             - With filter: execute(skill='k8s', tool='get', args={...}, grep='Running', head=10)\n\
             - JSON extract: execute(skill='k8s', tool='get', args={...}, jq='.items[].metadata.name')\n\
             - Size limit: execute(skill='k8s', tool='logs', args={...}, max_output=4000, truncate='tail')"
        )),
        input_schema: Arc::new(execute_schema),
        output_schema: None,
        annotations: None,
        icons: None,
        meta: None,
    };

    ToolRoute::new_dyn(tool, |ctx: ToolCallContext<'_, McpServer>| {
        async move {
            let start_time = std::time::Instant::now();
            let args = ctx.arguments.clone().unwrap_or_default();
            let request: ExecuteSkillRequest = serde_json::from_value(serde_json::Value::Object(args))
                .map_err(|e| McpError::invalid_params(format!("Invalid parameters: {}", e), None))?;

            // Execute the skill tool
            let result = ctx.service
                .execute_skill_tool(&request.skill, &request.instance, &request.tool, request.args)
                .await
                .map_err(|e| McpError::internal_error(format!("Skill execution failed: {}", e), None))?;

            let elapsed = start_time.elapsed();

            if result.success {
                // Apply context engineering transformations
                let processed = process_output(
                    &result.output,
                    request.max_output,
                    request.truncate.as_deref(),
                    request.grep.as_deref(),
                    request.grep_invert.unwrap_or(false),
                    request.head,
                    request.tail,
                    request.format.as_deref(),
                    request.jq.as_deref(),
                );

                // Build response
                let output = if request.include_metadata.unwrap_or(false) {
                    // Include rich metadata for debugging/transparency
                    let mut response = String::new();

                    if processed.truncated || !processed.processing.is_empty() {
                        response.push_str("📊 **Execution Metadata**\n");
                        response.push_str(&format!("- Execution time: {:?}\n", elapsed));
                        response.push_str(&format!("- Original size: {} chars\n", processed.original_length));
                        response.push_str(&format!("- Final size: {} chars\n", processed.final_length));

                        if processed.truncated {
                            response.push_str("- ⚠️ Output was truncated\n");
                        }

                        if let Some(matches) = processed.grep_matches {
                            response.push_str(&format!("- Grep matches: {} lines\n", matches));
                        }

                        if !processed.processing.is_empty() {
                            response.push_str(&format!("- Processing: {}\n", processed.processing.join(" → ")));
                        }

                        response.push_str("\n---\n\n");
                    }

                    response.push_str(&processed.content);
                    response
                } else {
                    processed.content
                };

                Ok(CallToolResult::success(vec![Content::text(output)]))
            } else {
                // Error response with helpful context
                let error_msg = result.error_message.unwrap_or_else(|| "Unknown error".to_string());
                let error_output = format!(
                    "❌ **Execution Failed**\n\n\
                     **Skill:** {} | **Tool:** {} | **Instance:** {}\n\n\
                     **Error:** {}\n\n\
                     💡 **Tips:**\n\
                     - Use `list_skills` to verify the skill/tool exists\n\
                     - Use `search_skills` to find the right tool for your task\n\
                     - Check that required arguments are provided",
                    request.skill, request.tool, request.instance, error_msg
                );
                Ok(CallToolResult::error(vec![Content::text(error_output)]))
            }
        }.boxed()
    })
}

/// Create the list_skills tool route
fn list_skills_tool_route() -> ToolRoute<McpServer> {
    use futures::FutureExt;
    use rmcp::handler::server::tool::ToolCallContext;

    let list_schema: serde_json::Map<String, serde_json::Value> = serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "skill": {
                "type": "string",
                "description": "Optional skill name to filter tools by"
            },
            "offset": {
                "type": "integer",
                "description": "Pagination offset (0-based index). Use with 'limit' to paginate through large tool lists.",
                "minimum": 0
            },
            "limit": {
                "type": "integer",
                "description": "Maximum number of tools to return. Use with 'offset' for pagination.",
                "minimum": 1
            }
        }
    })).unwrap();

    let tool = Tool {
        name: Cow::Borrowed("list_skills"),
        title: None,
        description: Some(Cow::Borrowed("List all available skills and their tools. Supports pagination with offset/limit parameters.")),
        input_schema: Arc::new(list_schema),
        output_schema: None,
        annotations: None,
        icons: None,
        meta: None,
    };

    ToolRoute::new_dyn(tool, |ctx: ToolCallContext<'_, McpServer>| {
        async move {
            let args = ctx.arguments.clone().unwrap_or_default();
            let request: ListSkillsRequest = serde_json::from_value(serde_json::Value::Object(args))
                .unwrap_or(ListSkillsRequest { skill: None, offset: None, limit: None });
            let output = ctx.service.list_skills_output(
                request.skill.as_deref(),
                request.offset,
                request.limit,
            ).await;
            Ok(CallToolResult::success(vec![Content::text(output)]))
        }.boxed()
    })
}

/// Request to search for skills
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchSkillsRequest {
    /// Natural language query describing what you want to do
    #[schemars(description = "Natural language query (e.g., 'list running pods', 'get aws s3 buckets')")]
    pub query: String,

    /// Maximum number of results to return
    #[serde(default = "default_top_k")]
    #[schemars(description = "Maximum number of results to return (default: 5)")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    5
}

/// Request to generate AI-powered examples for a skill tool
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GenerateExamplesRequest {
    /// The skill name to generate examples for
    #[schemars(description = "The skill name (e.g., 'kubernetes', 'aws')")]
    pub skill: String,

    /// Optional tool name within the skill
    #[schemars(description = "Optional tool name within the skill. If not provided, generates for all tools.")]
    pub tool: Option<String>,

    /// Number of examples to generate per tool
    #[serde(default = "default_example_count")]
    #[schemars(description = "Number of examples to generate per tool (default: 5)")]
    pub count: usize,
}

fn default_example_count() -> usize {
    5
}

/// Create the search_skills tool route
fn search_skills_tool_route() -> ToolRoute<McpServer> {
    use futures::FutureExt;
    use rmcp::handler::server::tool::ToolCallContext;

    let search_schema: serde_json::Map<String, serde_json::Value> = serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "Natural language query describing what you want to do (e.g., 'list running pods', 'get aws s3 buckets')"
            },
            "top_k": {
                "type": "integer",
                "description": "Maximum number of results to return (default: 5)",
                "default": 5
            }
        },
        "required": ["query"]
    })).unwrap();

    let tool = Tool {
        name: Cow::Borrowed("search_skills"),
        title: None,
        description: Some(Cow::Borrowed("Search for relevant skills and tools using natural language. Uses semantic vector search to find the best matching tools for your task.")),
        input_schema: Arc::new(search_schema),
        output_schema: None,
        annotations: None,
        icons: None,
        meta: None,
    };

    ToolRoute::new_dyn(tool, |ctx: ToolCallContext<'_, McpServer>| {
        async move {
            let args = ctx.arguments.clone().unwrap_or_default();
            let request: SearchSkillsRequest = serde_json::from_value(serde_json::Value::Object(args))
                .map_err(|e| McpError::invalid_params(format!("Invalid parameters: {}", e), None))?;

            let output = ctx.service.search_skills(&request.query, request.top_k).await
                .map_err(|e| McpError::internal_error(format!("Search failed: {}", e), None))?;

            Ok(CallToolResult::success(vec![Content::text(output)]))
        }.boxed()
    })
}

/// Create the generate_examples tool route
fn generate_examples_tool_route() -> ToolRoute<McpServer> {
    use futures::FutureExt;
    use rmcp::handler::server::tool::ToolCallContext;

    let schema: serde_json::Map<String, serde_json::Value> = serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "skill": {
                "type": "string",
                "description": "The skill name to generate examples for (e.g., 'kubernetes', 'aws')"
            },
            "tool": {
                "type": "string",
                "description": "Optional tool name within the skill. If not provided, generates for all tools."
            },
            "count": {
                "type": "integer",
                "description": "Number of examples to generate per tool (default: 5)",
                "default": 5
            }
        },
        "required": ["skill"]
    })).unwrap();

    let tool = Tool {
        name: Cow::Borrowed("generate_examples"),
        title: None,
        description: Some(Cow::Borrowed(
            "Generate AI-powered usage examples for a skill's tools. \
             Uses LLMs to create realistic command examples with explanations. \
             Requires AI ingestion to be enabled (use `skill setup` to configure)."
        )),
        input_schema: Arc::new(schema),
        output_schema: None,
        annotations: None,
        icons: None,
        meta: None,
    };

    ToolRoute::new_dyn(tool, |ctx: ToolCallContext<'_, McpServer>| {
        async move {
            let args = ctx.arguments.clone().unwrap_or_default();
            let request: GenerateExamplesRequest = serde_json::from_value(serde_json::Value::Object(args))
                .map_err(|e| McpError::invalid_params(format!("Invalid parameters: {}", e), None))?;

            let output = ctx.service.generate_examples(
                &request.skill,
                request.tool.as_deref(),
                request.count,
            ).await
                .map_err(|e| McpError::internal_error(format!("Example generation failed: {}", e), None))?;

            Ok(CallToolResult::success(vec![Content::text(output)]))
        }.boxed()
    })
}
