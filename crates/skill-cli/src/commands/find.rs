use anyhow::{Context, Result};
use blake3;
use chrono::{DateTime, Utc};
use colored::*;
use serde::{Deserialize, Serialize};
use indicatif::{ProgressBar, ProgressStyle};
use skill_runtime::{
    InstanceManager, SearchPipeline, IndexDocument, SearchConfig,
    DocumentMetadata,
};
use std::collections::HashMap;
use std::fs;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

/// Embedding cache entry for faster repeated searches
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EmbeddingCacheEntry {
    tools: Vec<ToolDocument>,
    skill_md_hash: Option<String>,
    wasm_modified: u64,
    cached_at: u64,
}

/// Embedding cache stored on disk
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct EmbeddingCache {
    version: u32,
    entries: HashMap<String, EmbeddingCacheEntry>, // skill_name -> cache entry
}

#[allow(dead_code)] // Caching methods reserved for performance optimization
impl EmbeddingCache {
    const CURRENT_VERSION: u32 = 1;

    fn cache_path() -> Result<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("skill-engine");
        fs::create_dir_all(&cache_dir)?;
        Ok(cache_dir.join("tool-index.json"))
    }

    fn load() -> Self {
        Self::cache_path()
            .ok()
            .and_then(|p| fs::File::open(p).ok())
            .and_then(|f| serde_json::from_reader(BufReader::new(f)).ok())
            .filter(|c: &Self| c.version == Self::CURRENT_VERSION)
            .unwrap_or_default()
    }

    fn save(&self) -> Result<()> {
        let path = Self::cache_path()?;
        let file = fs::File::create(path)?;
        serde_json::to_writer(BufWriter::new(file), self)?;
        Ok(())
    }

    fn is_valid(&self, skill_name: &str, wasm_modified: u64, skill_md_hash: Option<&str>) -> bool {
        self.entries.get(skill_name).map_or(false, |entry| {
            entry.wasm_modified == wasm_modified
                && entry.skill_md_hash.as_deref() == skill_md_hash
        })
    }

    fn get(&self, skill_name: &str) -> Option<&Vec<ToolDocument>> {
        self.entries.get(skill_name).map(|e| &e.tools)
    }

    fn set(&mut self, skill_name: String, tools: Vec<ToolDocument>, wasm_modified: u64, skill_md_hash: Option<String>) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.entries.insert(skill_name, EmbeddingCacheEntry {
            tools,
            skill_md_hash,
            wasm_modified,
            cached_at: now,
        });
    }
}

/// SKILL.md frontmatter structure - flexible to handle various YAML formats
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // Fields are parsed for future use
struct SkillMdFrontmatter {
    name: String,
    description: String,
    #[serde(rename = "allowed-tools", default)]
    allowed_tools: Option<Vec<String>>,
    // Optional extended fields (reserved for future features)
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
    #[serde(default)]
    category: Option<String>,
}

/// Parsed SKILL.md content with additional metadata
#[derive(Debug, Clone)]
#[allow(dead_code)] // sections reserved for future section-based search
struct SkillMdContent {
    frontmatter: SkillMdFrontmatter,
    body: String,
    /// Sections parsed from markdown (heading -> content)
    sections: HashMap<String, String>,
}

/// Parameter documentation from SKILL.md
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ParameterDoc {
    pub name: String,
    pub required: bool,
    pub param_type: String,
    pub description: String,
}

/// Parameter signature for execution metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParameterSignature {
    /// Parameter name
    pub name: String,
    /// Parameter type (string, integer, boolean, array, object)
    pub param_type: String,
    /// Whether the parameter is required
    pub required: bool,
    /// Parameter description
    pub description: String,
    /// Default value if any
    pub default: Option<String>,
    /// Allowed values (enum)
    pub allowed_values: Vec<String>,
}

impl From<&ParameterDoc> for ParameterSignature {
    fn from(param: &ParameterDoc) -> Self {
        Self {
            name: param.name.clone(),
            param_type: param.param_type.clone(),
            required: param.required,
            description: param.description.clone(),
            default: None,
            allowed_values: Vec::new(),
        }
    }
}

/// Execution signature with typed parameters and return info
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ExecutionSignature {
    /// Tool name
    pub tool_name: String,
    /// Parameters with full type information
    pub parameters: Vec<ParameterSignature>,
    /// Return type description
    pub returns: String,
    /// Whether the tool supports streaming
    pub streaming: bool,
    /// Estimated execution time in milliseconds
    pub estimated_ms: Option<u32>,
}

/// Analytics data for tool usage tracking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolAnalytics {
    /// Number of times this tool has been invoked
    pub usage_count: u64,
    /// Success rate (0.0 to 1.0) - stored as percentage integer (0-100)
    pub success_rate_pct: u8,
    /// Average latency in milliseconds
    pub avg_latency_ms: u32,
    /// Last time the tool was used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used: Option<DateTime<Utc>>,
}

impl PartialEq for ToolAnalytics {
    fn eq(&self, other: &Self) -> bool {
        self.usage_count == other.usage_count
            && self.success_rate_pct == other.success_rate_pct
            && self.avg_latency_ms == other.avg_latency_ms
            && self.last_used == other.last_used
    }
}

impl Eq for ToolAnalytics {}

/// Represents a tool from a skill with its metadata for semantic search
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolDocument {
    // === Identity Fields ===
    /// Unique identifier in format: skill@instance/tool
    pub id: String,
    pub skill_name: String,
    pub instance_name: String,
    pub tool_name: String,
    pub description: String,

    // === Categorization ===
    /// Category from SKILL.md frontmatter (e.g., "kubernetes", "database", "messaging")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Tags from SKILL.md frontmatter for filtering
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Action verbs extracted from description (e.g., ["list", "get", "create"])
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub action_verbs: Vec<String>,
    /// Parameter names for keyword matching
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameter_names: Vec<String>,

    // === SKILL.md Content ===
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_md_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_documentation: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub usage_examples: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<ParameterDoc>,

    // === Execution Signature ===
    /// Detailed execution signature with typed parameters
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_signature: Option<ExecutionSignature>,

    // === Analytics ===
    /// Usage analytics (populated from analytics store)
    #[serde(default)]
    pub analytics: ToolAnalytics,

    // === Versioning ===
    /// Skill version from SKILL.md or manifest
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_version: Option<String>,
    /// When this document was indexed
    #[serde(default)]
    pub indexed_at: DateTime<Utc>,
    /// Content hash for cache invalidation (blake3 of SKILL.md + tool description)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,

    // === Embedding ===
    /// Combined text for embedding
    pub full_text: String,
}

impl ToolDocument {
    /// Create a new tool document with minimal fields (when SKILL.md is not available)
    fn new(
        skill_name: String,
        instance_name: String,
        tool_name: String,
        description: String,
    ) -> Self {
        let id = format!("{}@{}/{}", skill_name, instance_name, tool_name);
        let action_verbs = extract_action_keywords(&tool_name, &description);

        // Create rich text for embedding that includes all context
        let full_text = format!(
            "Skill: {} | Instance: {} | Tool: {} | Description: {}",
            skill_name, instance_name, tool_name, description
        );

        // Generate content hash from basic info
        let content_hash = compute_content_hash(&tool_name, &description, None);

        Self {
            id,
            skill_name,
            instance_name,
            tool_name,
            description,
            category: None,
            tags: Vec::new(),
            action_verbs,
            parameter_names: Vec::new(),
            skill_md_description: None,
            tool_documentation: None,
            usage_examples: Vec::new(),
            parameters: Vec::new(),
            execution_signature: None,
            analytics: ToolAnalytics::default(),
            skill_version: None,
            indexed_at: Utc::now(),
            content_hash: Some(content_hash),
            full_text,
        }
    }

    /// Create a tool document with rich metadata from SKILL.md
    fn with_skill_md(
        skill_name: String,
        instance_name: String,
        tool_name: String,
        description: String,
        skill_md_content: Option<&SkillMdContent>,
    ) -> Self {
        let id = format!("{}@{}/{}", skill_name, instance_name, tool_name);
        let mut usage_examples = Vec::new();
        let mut parameters = Vec::new();
        let mut tool_documentation = None;
        let skill_md_description = skill_md_content.as_ref().map(|md| md.frontmatter.description.clone());

        // Extract category and tags from frontmatter
        let category = skill_md_content.as_ref().and_then(|md| md.frontmatter.category.clone());
        let tags = skill_md_content
            .as_ref()
            .and_then(|md| md.frontmatter.tags.clone())
            .unwrap_or_default();
        let skill_version = skill_md_content.as_ref().and_then(|md| md.frontmatter.version.clone());

        // Extract tool-specific documentation from SKILL.md body
        if let Some(md) = skill_md_content {
            // Parse tool section from markdown
            let tool_section = extract_tool_section(&md.body, &tool_name);

            // Extract examples and parameters from the tool section
            if let Some(ref section_text) = tool_section {
                usage_examples = extract_examples(section_text);
                parameters = extract_parameters(section_text);
            }

            tool_documentation = tool_section;
        }

        // Extract action verbs from tool name and description
        let action_verbs = extract_action_keywords(&tool_name, &description);

        // Extract parameter names for keyword matching
        let parameter_names: Vec<String> = parameters.iter().map(|p| p.name.clone()).collect();

        // Build execution signature
        let execution_signature = Some(ExecutionSignature {
            tool_name: tool_name.clone(),
            parameters: parameters.iter().map(ParameterSignature::from).collect(),
            returns: "string".to_string(), // Default, could be enhanced from SKILL.md
            streaming: false,
            estimated_ms: None,
        });

        // Create WEIGHTED embedding text for better search relevance
        // Priority: tool_name > description > parameters > skill_context > examples
        let full_text = build_weighted_embedding_text(
            &skill_name,
            &instance_name,
            &tool_name,
            &description,
            skill_md_description.as_deref(),
            tool_documentation.as_deref(),
            &parameters,
            &usage_examples,
        );

        // Compute content hash for cache invalidation
        let content_hash = compute_content_hash(
            &tool_name,
            &description,
            tool_documentation.as_deref(),
        );

        Self {
            id,
            skill_name,
            instance_name,
            tool_name,
            description,
            category,
            tags,
            action_verbs,
            parameter_names,
            skill_md_description,
            tool_documentation,
            usage_examples,
            parameters,
            execution_signature,
            analytics: ToolAnalytics::default(),
            skill_version,
            indexed_at: Utc::now(),
            content_hash: Some(content_hash),
            full_text,
        }
    }

    /// Get the unique identifier for this tool
    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// Check if the content has changed based on hash
    pub fn content_changed(&self, other_hash: &str) -> bool {
        self.content_hash.as_deref() != Some(other_hash)
    }
}

/// Compute a blake3 hash of the tool content for cache invalidation
fn compute_content_hash(tool_name: &str, description: &str, documentation: Option<&str>) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(tool_name.as_bytes());
    hasher.update(description.as_bytes());
    if let Some(doc) = documentation {
        hasher.update(doc.as_bytes());
    }
    hasher.finalize().to_hex()[..16].to_string() // Use first 16 chars for brevity
}

/// Build weighted embedding text for better semantic search relevance
/// Repeats important terms to increase their weight in the embedding
fn build_weighted_embedding_text(
    skill_name: &str,
    _instance_name: &str,
    tool_name: &str,
    description: &str,
    skill_description: Option<&str>,
    tool_documentation: Option<&str>,
    parameters: &[ParameterDoc],
    examples: &[String],
) -> String {
    let mut parts = Vec::new();

    // HIGH WEIGHT: Tool name and description (repeated for emphasis)
    parts.push(format!("Tool: {} - {}", tool_name, description));
    parts.push(format!("{}: {}", tool_name, description)); // Repeat for weight

    // HIGH WEIGHT: Action verbs extracted from tool name and description
    let action_keywords = extract_action_keywords(tool_name, description);
    if !action_keywords.is_empty() {
        parts.push(format!("Actions: {}", action_keywords.join(", ")));
    }

    // MEDIUM WEIGHT: Skill context
    parts.push(format!("Skill: {}", skill_name));
    if let Some(skill_desc) = skill_description {
        parts.push(format!("Context: {}", skill_desc));
    }

    // MEDIUM WEIGHT: Parameters (important for understanding capabilities)
    if !parameters.is_empty() {
        let param_text: Vec<String> = parameters.iter()
            .map(|p| format!("{} ({})", p.name, p.description))
            .collect();
        parts.push(format!("Parameters: {}", param_text.join(", ")));
    }

    // LOWER WEIGHT: Documentation excerpt (first 200 chars)
    if let Some(doc) = tool_documentation {
        // Extract clean text without markdown
        let clean_doc = clean_markdown_for_embedding(doc);
        if clean_doc.len() > 200 {
            parts.push(format!("Details: {}...", &clean_doc[..200]));
        } else if !clean_doc.is_empty() {
            parts.push(format!("Details: {}", clean_doc));
        }
    }

    // LOWER WEIGHT: Example commands (useful for finding by usage pattern)
    for example in examples.iter().take(2) {
        parts.push(format!("Example: {}", example));
    }

    parts.join(" | ")
}

/// Extract action keywords from tool name and description
fn extract_action_keywords(tool_name: &str, description: &str) -> Vec<String> {
    let action_verbs = [
        "list", "get", "create", "delete", "update", "upload", "download",
        "send", "receive", "fetch", "search", "find", "run", "execute",
        "start", "stop", "restart", "deploy", "build", "test", "check",
        "validate", "generate", "parse", "convert", "transform", "filter",
        "sort", "merge", "split", "copy", "move", "rename", "echo", "greet",
        "hello", "invoke", "call", "query", "read", "write", "append",
    ];

    let combined = format!("{} {}", tool_name.replace('-', " "), description.to_lowercase());
    let words: Vec<&str> = combined.split_whitespace().collect();

    action_verbs.iter()
        .filter(|&&verb| words.iter().any(|w| w.contains(verb)))
        .map(|&s| s.to_string())
        .collect()
}

/// Clean markdown text for embedding (remove formatting)
fn clean_markdown_for_embedding(text: &str) -> String {
    text.lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with('#')
                && !trimmed.starts_with("```")
                && !trimmed.starts_with("**")
                && !trimmed.is_empty()
        })
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Parse SKILL.md file with YAML frontmatter
fn parse_skill_md(content: &str) -> Result<SkillMdContent> {
    // Check if content starts with --- (frontmatter delimiter)
    if !content.trim_start().starts_with("---") {
        return Err(anyhow::anyhow!("SKILL.md missing frontmatter"));
    }

    // Find the end of frontmatter (second ---)
    let lines: Vec<&str> = content.lines().collect();
    let mut frontmatter_start = None;
    let mut frontmatter_end = None;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed == "---" {
            if frontmatter_start.is_none() {
                frontmatter_start = Some(i);
            } else {
                frontmatter_end = Some(i);
                break;
            }
        }
    }

    let start = frontmatter_start.ok_or_else(|| anyhow::anyhow!("SKILL.md missing opening ---"))?;
    let end = frontmatter_end.ok_or_else(|| anyhow::anyhow!("SKILL.md frontmatter not properly closed"))?;

    // Extract frontmatter and body
    let frontmatter_str = lines[start + 1..end].join("\n");
    let body = lines[end + 1..].join("\n");

    // Parse YAML frontmatter
    let frontmatter: SkillMdFrontmatter = serde_yaml::from_str(&frontmatter_str)
        .with_context(|| format!("Failed to parse SKILL.md frontmatter: {}", frontmatter_str))?;

    // Parse markdown sections for quick lookup
    let sections = parse_markdown_sections(&body);

    Ok(SkillMdContent { frontmatter, body, sections })
}

/// Parse markdown into sections by heading
fn parse_markdown_sections(body: &str) -> HashMap<String, String> {
    let mut sections = HashMap::new();
    let lines: Vec<&str> = body.lines().collect();
    let mut current_heading: Option<String> = None;
    let mut current_content: Vec<&str> = Vec::new();

    for line in lines {
        if line.starts_with("## ") || line.starts_with("### ") {
            // Save previous section
            if let Some(heading) = current_heading.take() {
                sections.insert(heading, current_content.join("\n"));
                current_content.clear();
            }
            // Start new section
            current_heading = Some(line.trim_start_matches('#').trim().to_string());
        } else if current_heading.is_some() {
            current_content.push(line);
        }
    }

    // Save last section
    if let Some(heading) = current_heading {
        sections.insert(heading, current_content.join("\n"));
    }

    sections
}

/// Extract tool-specific section from SKILL.md body
fn extract_tool_section(body: &str, tool_name: &str) -> Option<String> {
    // Look for "### tool-name" heading
    let heading = format!("### {}", tool_name);
    let lines: Vec<&str> = body.lines().collect();

    let mut start_idx = None;
    for (i, line) in lines.iter().enumerate() {
        // Match the line, trimming whitespace
        if line.trim() == heading.trim() {
            start_idx = Some(i);
            break;
        }
    }

    let start_idx = start_idx?;

    // Find end of section (next ### or ## heading, or end of document)
    let mut end_idx = lines.len();
    for (i, line) in lines[start_idx + 1..].iter().enumerate() {
        let trimmed = line.trim();
        if (trimmed.starts_with("### ") && !trimmed.starts_with(&format!("### {}", tool_name)))
            || trimmed.starts_with("## ") {
            end_idx = start_idx + 1 + i;
            break;
        }
    }

    Some(lines[start_idx..end_idx].join("\n"))
}

/// Extract usage examples from tool documentation
fn extract_examples(tool_doc: &str) -> Vec<String> {
    let mut examples = Vec::new();
    let lines: Vec<&str> = tool_doc.lines().collect();

    let mut in_code_block = false;
    let mut current_example = Vec::new();

    for line in lines {
        if line.trim().starts_with("```") {
            if in_code_block {
                // End of code block
                if !current_example.is_empty() {
                    examples.push(current_example.join("\n"));
                    current_example.clear();
                }
            }
            in_code_block = !in_code_block;
        } else if in_code_block && line.contains("skill run") {
            current_example.push(line.to_string());
        }
    }

    examples
}

/// Extract parameter documentation from tool section
fn extract_parameters(tool_doc: &str) -> Vec<ParameterDoc> {
    let mut parameters = Vec::new();
    let lines: Vec<&str> = tool_doc.lines().collect();

    let mut in_parameters_section = false;

    for line in lines {
        if line.trim() == "**Parameters**:" {
            in_parameters_section = true;
            continue;
        }

        if in_parameters_section {
            // Stop at next section
            if line.trim().starts_with("**") && line.trim() != "**Parameters**:" {
                break;
            }

            // Parse parameter line: "- `param_name` (required): Description"
            if let Some(param) = parse_parameter_line(line) {
                parameters.push(param);
            }
        }
    }

    parameters
}

/// Parse a single parameter line from markdown
fn parse_parameter_line(line: &str) -> Option<ParameterDoc> {
    let trimmed = line.trim();
    if !trimmed.starts_with("- `") {
        return None;
    }

    // Extract parameter name (between ` `)
    let name_end = trimmed[3..].find('`')?;
    let name = trimmed[3..3 + name_end].to_string();

    // Check if required or optional
    let rest = &trimmed[3 + name_end + 1..];
    let required = rest.contains("(required)");

    // Extract description (after : )
    let desc_start = rest.find(':')?;
    let description = rest[desc_start + 1..].trim().to_string();

    // Try to infer type from context
    let param_type = if description.to_lowercase().contains("number") {
        "number"
    } else if description.to_lowercase().contains("boolean") || description.to_lowercase().contains("true/false") {
        "boolean"
    } else {
        "string"
    }.to_string();

    Some(ParameterDoc {
        name,
        required,
        param_type,
        description,
    })
}

/// Display a single search result with rich formatting
fn display_rich_result(rank: usize, similarity_score: f64, tool: &ToolDocument) {
    let score_percent = (similarity_score * 100.0) as u32;
    let score_color = if score_percent >= 80 {
        "green"
    } else if score_percent >= 60 {
        "yellow"
    } else {
        "red"
    };

    // Header: rank, score, tool identifier
    println!(
        "{}. {} {}",
        rank.to_string().bold(),
        format!("[{}% match]", score_percent).color(score_color),
        format!(
            "{}@{} → {}",
            tool.skill_name, tool.instance_name, tool.tool_name
        )
        .cyan()
        .bold()
    );

    // Short description
    println!("   {}", tool.description.dimmed());
    println!();

    // Context from SKILL.md (if available)
    if let Some(ref skill_desc) = tool.skill_md_description {
        println!("   {} {}", "📋 Context:".bold(), skill_desc);
        println!();
    }

    // Tool-specific documentation (if available)
    if let Some(ref tool_doc) = tool.tool_documentation {
        // Extract first 3 lines as excerpt
        let lines: Vec<&str> = tool_doc.lines().take(5).collect();
        if !lines.is_empty() {
            println!("   {} {}", "📖 Details:".bold(), lines[0]);
            for line in &lines[1..] {
                if !line.trim().is_empty() && !line.trim().starts_with("#") {
                    println!("      {}", line.dimmed());
                }
            }
            println!();
        }
    }

    // Usage command
    let usage_cmd = if tool.parameters.is_empty() {
        format!("skill run {}@{}:{}", tool.skill_name, tool.instance_name, tool.tool_name)
    } else {
        let param_hints: Vec<String> = tool.parameters.iter()
            .map(|p| if p.required {
                format!("--{} <value>", p.name)
            } else {
                format!("[--{} <value>]", p.name)
            })
            .collect();
        format!("skill run {}@{}:{} {}", tool.skill_name, tool.instance_name, tool.tool_name, param_hints.join(" "))
    };
    println!("   {} {}", "⚙️  Usage:".bold(), usage_cmd.cyan());
    println!();

    // Parameters (if available from SKILL.md)
    if !tool.parameters.is_empty() {
        println!("   {} Parameters:", "📝".bold());
        for param in &tool.parameters {
            let req_str = if param.required { "required" } else { "optional" };
            println!(
                "      • {} ({}, {}): {}",
                param.name.cyan(),
                param.param_type.yellow(),
                req_str.dimmed(),
                param.description.dimmed()
            );
        }
        println!();
    }

    // Usage example (if available from SKILL.md)
    if !tool.usage_examples.is_empty() {
        println!("   {} Example:", "💡".bold());
        for example in &tool.usage_examples {
            println!("      {}", example.green());
        }
        println!();
    }

    println!();
}

/// Get the path to the search configuration file
fn get_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Failed to get home directory")?;
    Ok(home.join(".skill-engine").join("search.toml"))
}

/// Load or create default search configuration
fn load_or_create_config(provider: &str, model: Option<&str>, is_json: bool) -> Result<(SearchConfig, bool)> {
    let config_path = get_config_path()?;
    let first_run = !config_path.exists();

    let mut config = if config_path.exists() {
        // Load existing config
        SearchConfig::from_toml_file(&config_path).unwrap_or_default()
    } else {
        // First run - create default config
        if !is_json {
            println!();
            println!("{}", "Welcome to Skill Engine Search!".bold().cyan());
            println!();
            println!("  This is your first time using semantic search.");
            println!("  Using FastEmbed (local, no API key required).");
            println!();
            println!("  {} Run {} for more options (OpenAI, Ollama, etc.)",
                "Tip:".yellow().bold(), "skill setup".cyan());
            println!();
        }

        let config = SearchConfig::default();

        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Save default config
        let toml_str = toml::to_string_pretty(&config)?;
        fs::write(&config_path, &toml_str)?;

        if !is_json {
            println!("  {} Saved config to {}", "✓".green(), config_path.display());
            println!();
        }

        config
    };

    // Override from CLI arguments (only if explicitly provided, not default)
    let provider_lower = provider.to_lowercase();

    // Only override if CLI args differ from config or it's not the default
    if provider_lower != "fastembed" || model.is_some() {
        match provider_lower.as_str() {
            "fastembed" => {
                config.embedding.provider = "fastembed".to_string();
                if let Some(m) = model {
                    config.embedding.model = m.to_string();
                }
            }
            "openai" => {
                config.embedding.provider = "openai".to_string();
                if let Some(m) = model {
                    config.embedding.model = m.to_string();
                } else if config.embedding.provider != "openai" {
                    config.embedding.model = "text-embedding-ada-002".to_string();
                }
                config.embedding.openai_api_key = std::env::var("OPENAI_API_KEY").ok();
                if config.embedding.openai_api_key.is_none() {
                    return Err(anyhow::anyhow!(
                        "OPENAI_API_KEY not set. Set it with: export OPENAI_API_KEY=your-key-here\n\
                         Or run 'skill setup' to configure a different provider."
                    ));
                }
            }
            "ollama" => {
                config.embedding.provider = "ollama".to_string();
                if let Some(m) = model {
                    config.embedding.model = m.to_string();
                } else if config.embedding.provider != "ollama" {
                    config.embedding.model = "nomic-embed-text".to_string();
                }
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unknown provider: {}. Supported: fastembed, openai, ollama\n\
                     Run 'skill setup' to configure interactively.",
                    provider
                ));
            }
        }
    }

    Ok((config, first_run))
}

/// Semantic tool search using SearchPipeline from skill-runtime
pub async fn execute(
    query: &str,
    top_k: Option<usize>,
    provider: &str,
    model: Option<&str>,
    format: &str,
) -> Result<()> {
    let top_k = top_k.unwrap_or(5);
    let is_json = format == "json";

    if !is_json {
        println!();
        println!("{} Searching for: {}", "→".cyan(), query.yellow());
    }

    // Load all skills and their tools
    let tool_documents = load_all_tools().await?;

    if tool_documents.is_empty() {
        if is_json {
            println!("{{\"results\": [], \"error\": \"No skills installed\"}}");
        } else {
            println!("{} No skills installed yet. Install a skill first with: skill install <source>", "!".yellow());
        }
        return Ok(());
    }

    if !is_json {
        println!(
            "{} Found {} tools, generating embeddings...",
            "→".cyan(),
            tool_documents.len()
        );
    }

    // Load or create configuration
    let (config, first_run) = load_or_create_config(provider, model, is_json)?;

    // Show provider info
    if !is_json {
        match config.embedding.provider.as_str() {
            "fastembed" => println!("{} Using FastEmbed (local, no API key required)", "✓".green()),
            "openai" => println!("{} Using OpenAI (requires OPENAI_API_KEY)", "→".cyan()),
            "ollama" => println!("{} Using Ollama (requires local Ollama server)", "→".cyan()),
            _ => {}
        }
    }

    // Initialize SearchPipeline with progress indication
    let pb = if !is_json && first_run {
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap());
        pb.set_message("Downloading embedding model (first run only)...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        Some(pb)
    } else {
        if !is_json {
            println!("{} Initializing search pipeline...", "→".cyan());
        }
        None
    };

    let pipeline = SearchPipeline::from_config(config).await
        .context("Failed to initialize search pipeline")?;

    if let Some(pb) = pb {
        pb.finish_with_message(format!("{} Model ready", "✓".green()));
    }

    // Convert ToolDocuments to IndexDocuments
    let index_docs: Vec<IndexDocument> = tool_documents
        .iter()
        .map(|tool| IndexDocument {
            id: tool.id.clone(),
            content: tool.full_text.clone(),
            metadata: DocumentMetadata {
                skill_name: Some(tool.skill_name.clone()),
                instance_name: Some(tool.instance_name.clone()),
                tool_name: Some(tool.tool_name.clone()),
                category: tool.category.clone(),
                tags: tool.tags.clone(),
                custom: HashMap::new(),
            },
        })
        .collect();

    // Index documents
    pipeline.index_documents(index_docs).await
        .context("Failed to index tools")?;

    if !is_json {
        println!("{} Searching for relevant tools...", "→".cyan());
    }

    // Search
    let search_results = pipeline.search(query, top_k).await
        .context("Failed to perform semantic search")?;

    // Convert search results back to the expected format (f64, String, ToolDocument)
    let results: Vec<(f64, String, ToolDocument)> = search_results
        .into_iter()
        .filter_map(|result| {
            // Find the original ToolDocument by ID
            tool_documents.iter()
                .find(|tool| tool.id == result.id)
                .map(|tool| (result.score as f64, result.id.clone(), tool.clone()))
        })
        .collect();

    // Output based on format
    match format {
        "json" => display_json_results(&results)?,
        "compact" => display_compact_results(&results),
        _ => display_rich_results(&results, top_k),
    }

    Ok(())
}

/// Display results as JSON (for programmatic consumption and MCP integration)
fn display_json_results(results: &[(f64, String, ToolDocument)]) -> Result<()> {
    #[derive(Serialize)]
    struct JsonParameter {
        name: String,
        #[serde(rename = "type")]
        param_type: String,
        required: bool,
        description: String,
    }

    #[derive(Serialize)]
    struct JsonExecutionSignature {
        skill: String,
        tool: String,
        instance: String,
        args: HashMap<String, serde_json::Value>,
    }

    #[derive(Serialize)]
    struct JsonResult {
        // Match score and relevance
        score: f64,
        score_percent: u32,
        relevance: String,

        // Tool identity
        id: String,
        skill: String,
        instance: String,
        tool: String,

        // Documentation
        description: String,
        skill_description: Option<String>,
        tool_documentation: Option<String>,

        // Parameters with types
        parameters: Vec<JsonParameter>,

        // Usage examples
        examples: Vec<String>,

        // Execution signature for direct use
        execution: JsonExecutionSignature,

        // CLI command
        cli_command: String,

        // Metadata
        category: Option<String>,
        tags: Vec<String>,
        version: Option<String>,
    }

    let json_results: Vec<JsonResult> = results.iter()
        .map(|(score, _, tool)| {
            let score_pct = (*score * 100.0) as u32;
            let relevance = match score_pct {
                80..=100 => "excellent",
                60..=79 => "good",
                40..=59 => "fair",
                _ => "partial",
            }.to_string();

            // Build execution args template
            let mut args = HashMap::new();
            for param in &tool.parameters {
                let placeholder = match param.param_type.as_str() {
                    "string" => serde_json::Value::String("<value>".to_string()),
                    "number" | "integer" => serde_json::Value::Number(0.into()),
                    "boolean" => serde_json::Value::Bool(true),
                    _ => serde_json::Value::String("<value>".to_string()),
                };
                args.insert(param.name.clone(), placeholder);
            }

            let execution = JsonExecutionSignature {
                skill: tool.skill_name.clone(),
                tool: tool.tool_name.clone(),
                instance: tool.instance_name.clone(),
                args,
            };

            let cli_command = if tool.parameters.is_empty() {
                format!("skill run {}@{}:{}", tool.skill_name, tool.instance_name, tool.tool_name)
            } else {
                let param_hints: Vec<String> = tool.parameters.iter()
                    .map(|p| if p.required {
                        format!("{}=<value>", p.name)
                    } else {
                        format!("[{}=<value>]", p.name)
                    })
                    .collect();
                format!("skill run {}@{}:{} {}", tool.skill_name, tool.instance_name, tool.tool_name, param_hints.join(" "))
            };

            JsonResult {
                score: *score,
                score_percent: score_pct,
                relevance,
                id: tool.id.clone(),
                skill: tool.skill_name.clone(),
                instance: tool.instance_name.clone(),
                tool: tool.tool_name.clone(),
                description: tool.description.clone(),
                skill_description: tool.skill_md_description.clone(),
                tool_documentation: tool.tool_documentation.clone(),
                parameters: tool.parameters.iter().map(|p| JsonParameter {
                    name: p.name.clone(),
                    param_type: p.param_type.clone(),
                    required: p.required,
                    description: p.description.clone(),
                }).collect(),
                examples: tool.usage_examples.clone(),
                execution,
                cli_command,
                category: tool.category.clone(),
                tags: tool.tags.clone(),
                version: tool.skill_version.clone(),
            }
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&json_results)?);
    Ok(())
}

/// Display results in compact format (one line per result)
fn display_compact_results(results: &[(f64, String, ToolDocument)]) {
    for (score, _, tool) in results {
        let score_pct = (score * 100.0) as u32;
        println!(
            "[{:3}%] {}@{}:{} - {}",
            score_pct,
            tool.skill_name,
            tool.instance_name,
            tool.tool_name,
            tool.description
        );
    }
}

/// Display results with rich formatting
fn display_rich_results(results: &[(f64, String, ToolDocument)], top_k: usize) {
    println!();
    println!("{}", "━".repeat(80).dimmed());
    println!();
    println!("{} Top {} matching tools:", "✓".green().bold(), top_k);
    println!();

    // Display results with rich formatting
    for (idx, (similarity_score, _doc_id, tool)) in results.iter().enumerate() {
        display_rich_result(idx + 1, *similarity_score, tool);
    }

    println!("{}", "━".repeat(80).dimmed());
    println!();
    println!(
        "{} Use {} to see all available tools",
        "💡".yellow(),
        "skill list --verbose".cyan()
    );
    println!();
}

/// Load all tools from all installed skills
async fn load_all_tools() -> Result<Vec<ToolDocument>> {
    let instance_manager = InstanceManager::new()?;
    let mut tool_documents = Vec::new();

    // Get list of all installed skills
    let registry_dir = dirs::home_dir()
        .context("Failed to get home directory")?
        .join(".skill-engine")
        .join("registry");

    if !registry_dir.exists() {
        return Ok(tool_documents);
    }

    // Iterate through all skills
    for entry in std::fs::read_dir(&registry_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let skill_name = entry
            .file_name()
            .to_string_lossy()
            .to_string();

        // Get all instances for this skill
        let instances = instance_manager
            .list_instances(&skill_name)
            .unwrap_or_default();

        for instance_name in instances {
            // Load the skill and get its tools
            if let Ok(tools) = load_skill_tools(&skill_name, &instance_name).await {
                for tool in tools {
                    tool_documents.push(tool);
                }
            }
        }
    }

    Ok(tool_documents)
}

/// Load tools from a specific skill instance
async fn load_skill_tools(
    skill_name: &str,
    instance_name: &str,
) -> Result<Vec<ToolDocument>> {
    use skill_runtime::{SkillEngine, SkillExecutor};

    let mut tool_documents = Vec::new();

    // Get skill directory
    let skill_dir = dirs::home_dir()
        .context("Failed to get home directory")?
        .join(".skill-engine")
        .join("registry")
        .join(skill_name);

    let skill_path = skill_dir.join(format!("{}.wasm", skill_name));

    if !skill_path.exists() {
        return Ok(tool_documents);
    }

    // Try to load SKILL.md if it exists (silently, no debug output)
    let skill_md_path = skill_dir.join("SKILL.md");
    let skill_md_content = if skill_md_path.exists() {
        fs::read_to_string(&skill_md_path)
            .ok()
            .and_then(|content| parse_skill_md(&content).ok())
    } else {
        None
    };

    // Load instance config
    let instance_manager = InstanceManager::new()?;
    let config = instance_manager
        .load_instance(skill_name, instance_name)
        .unwrap_or_default();

    // Load skill
    let engine = Arc::new(SkillEngine::new()?);
    let executor = SkillExecutor::load(
        engine,
        &skill_path,
        skill_name.to_string(),
        instance_name.to_string(),
        config,
    )
    .await?;

    // Get tools from skill
    let tools = executor.get_tools().await?;

    for tool in tools {
        // Use with_skill_md if SKILL.md available, otherwise use basic constructor
        let doc = if skill_md_content.is_some() {
            ToolDocument::with_skill_md(
                skill_name.to_string(),
                instance_name.to_string(),
                tool.name.clone(),
                tool.description.clone(),
                skill_md_content.as_ref(),
            )
        } else {
            ToolDocument::new(
                skill_name.to_string(),
                instance_name.to_string(),
                tool.name,
                tool.description,
            )
        };

        tool_documents.push(doc);
    }

    Ok(tool_documents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_document_new_creates_id() {
        let doc = ToolDocument::new(
            "test-skill".to_string(),
            "default".to_string(),
            "hello".to_string(),
            "Says hello".to_string(),
        );

        assert_eq!(doc.id, "test-skill@default/hello");
        assert_eq!(doc.skill_name, "test-skill");
        assert_eq!(doc.instance_name, "default");
        assert_eq!(doc.tool_name, "hello");
        assert_eq!(doc.description, "Says hello");
        assert!(doc.category.is_none());
        assert!(doc.tags.is_empty());
        assert!(doc.content_hash.is_some());
        assert!(doc.execution_signature.is_none());
    }

    #[test]
    fn test_tool_document_extracts_action_verbs() {
        let doc = ToolDocument::new(
            "kubernetes".to_string(),
            "default".to_string(),
            "list-pods".to_string(),
            "List all pods in the cluster".to_string(),
        );

        assert!(doc.action_verbs.contains(&"list".to_string()));
    }

    #[test]
    fn test_content_hash_changes_with_content() {
        let doc1 = ToolDocument::new(
            "test".to_string(),
            "default".to_string(),
            "hello".to_string(),
            "Says hello".to_string(),
        );

        let doc2 = ToolDocument::new(
            "test".to_string(),
            "default".to_string(),
            "hello".to_string(),
            "Says goodbye".to_string(),
        );

        // Different description should produce different hash
        assert_ne!(doc1.content_hash, doc2.content_hash);
    }

    #[test]
    fn test_content_hash_same_for_identical_content() {
        let doc1 = ToolDocument::new(
            "test".to_string(),
            "default".to_string(),
            "hello".to_string(),
            "Says hello".to_string(),
        );

        let doc2 = ToolDocument::new(
            "test".to_string(),
            "default".to_string(),
            "hello".to_string(),
            "Says hello".to_string(),
        );

        assert_eq!(doc1.content_hash, doc2.content_hash);
    }

    #[test]
    fn test_tool_document_with_skill_md() {
        let frontmatter = SkillMdFrontmatter {
            name: "test-skill".to_string(),
            description: "A test skill".to_string(),
            allowed_tools: Some(vec!["Read".to_string()]),
            version: Some("1.0.0".to_string()),
            author: None,
            tags: Some(vec!["testing".to_string(), "demo".to_string()]),
            category: Some("utilities".to_string()),
        };

        let skill_md = SkillMdContent {
            frontmatter,
            body: "# Test\n\n## Tools\n\n### hello\n\nSay hello to someone.\n".to_string(),
            sections: HashMap::new(),
        };

        let doc = ToolDocument::with_skill_md(
            "test-skill".to_string(),
            "default".to_string(),
            "hello".to_string(),
            "Says hello".to_string(),
            Some(&skill_md),
        );

        assert_eq!(doc.category, Some("utilities".to_string()));
        assert_eq!(doc.tags, vec!["testing".to_string(), "demo".to_string()]);
        assert_eq!(doc.skill_version, Some("1.0.0".to_string()));
        assert!(doc.execution_signature.is_some());
        assert!(doc.skill_md_description.is_some());
    }

    #[test]
    fn test_execution_signature_from_parameters() {
        let params = vec![
            ParameterDoc {
                name: "message".to_string(),
                required: true,
                param_type: "string".to_string(),
                description: "The message to display".to_string(),
            },
        ];

        let sig: ParameterSignature = (&params[0]).into();
        assert_eq!(sig.name, "message");
        assert!(sig.required);
        assert_eq!(sig.param_type, "string");
    }

    #[test]
    fn test_tool_analytics_default() {
        let analytics = ToolAnalytics::default();
        assert_eq!(analytics.usage_count, 0);
        assert_eq!(analytics.success_rate_pct, 0);
        assert_eq!(analytics.avg_latency_ms, 0);
        assert!(analytics.last_used.is_none());
    }

    #[test]
    fn test_tool_document_equality() {
        let doc1 = ToolDocument::new(
            "test".to_string(),
            "default".to_string(),
            "hello".to_string(),
            "Says hello".to_string(),
        );

        let doc2 = ToolDocument::new(
            "test".to_string(),
            "default".to_string(),
            "hello".to_string(),
            "Says hello".to_string(),
        );

        // Note: indexed_at will be different, so we only compare id
        assert_eq!(doc1.id, doc2.id);
    }

    #[test]
    fn test_compute_content_hash() {
        let hash1 = compute_content_hash("tool1", "desc1", None);
        let hash2 = compute_content_hash("tool1", "desc1", None);
        let hash3 = compute_content_hash("tool1", "desc2", None);
        let hash4 = compute_content_hash("tool1", "desc1", Some("extra doc"));

        // Same inputs produce same hash
        assert_eq!(hash1, hash2);
        // Different description produces different hash
        assert_ne!(hash1, hash3);
        // Added documentation produces different hash
        assert_ne!(hash1, hash4);
        // Hash is 16 chars (half of blake3 hex)
        assert_eq!(hash1.len(), 16);
    }
}
