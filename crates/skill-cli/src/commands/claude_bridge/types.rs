//! Type definitions for Claude Bridge
//!
//! These types represent the Claude Agent Skills format and generation options.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Options for generating Claude Agent Skills
#[derive(Debug, Clone, Default)]
pub struct GenerateOptions {
    /// Output directory for generated skills (default: ~/.claude/skills)
    pub output_dir: PathBuf,

    /// Specific skill to generate (if None, generates all)
    pub skill_name: Option<String>,

    /// Path to manifest file (if None, auto-detect)
    pub manifest_path: Option<PathBuf>,

    /// Force overwrite existing files
    #[allow(dead_code)]
    pub force: bool,

    /// Dry run - show what would be generated without writing
    pub dry_run: bool,

    /// Skip generating scripts
    pub no_scripts: bool,

    /// Generate for project-level Claude Code config
    #[allow(dead_code)]
    pub project: bool,
}

/// Result of generation
#[derive(Debug, Clone, Default)]
pub struct GenerateResult {
    /// Names of successfully generated skills
    pub generated_skills: Vec<String>,

    /// Dry run output (if dry_run was true)
    pub dry_run_output: Vec<String>,

    /// Any warnings encountered
    pub warnings: Vec<String>,
}

/// A Claude Agent Skill ready for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeSkill {
    /// Skill name (lowercase, max 64 chars)
    pub name: String,

    /// Short description (max 1024 chars)
    pub description: String,

    /// Tools provided by this skill
    pub tools: Vec<ClaudeTool>,

    /// Tool categories for organization
    pub categories: HashMap<String, Vec<String>>,

    /// When to use this skill (triggers/scenarios)
    pub when_to_use: Vec<String>,

    /// Source skill runtime type
    pub runtime: SkillRuntimeType,
}

/// A tool within a Claude Skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeTool {
    /// Tool name
    pub name: String,

    /// Short description
    pub description: String,

    /// Tool parameters
    pub parameters: Vec<ClaudeToolParameter>,

    /// Usage examples
    pub examples: Vec<ToolExample>,

    /// Category this tool belongs to
    pub category: Option<String>,

    /// Whether this tool supports streaming
    pub streaming: bool,
}

/// A parameter for a Claude tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeToolParameter {
    /// Parameter name
    pub name: String,

    /// Parameter type (string, number, boolean, etc.)
    pub param_type: String,

    /// Parameter description
    pub description: String,

    /// Whether this parameter is required
    pub required: bool,

    /// Default value if not required
    pub default_value: Option<String>,

    /// Enum values if this is an enum type
    pub enum_values: Option<Vec<String>>,
}

/// Usage example for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExample {
    /// Description of what this example does
    pub description: String,

    /// MCP execute call
    pub mcp_call: String,

    /// Script invocation
    pub script_call: String,
}

/// Runtime type of the source skill
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SkillRuntimeType {
    Wasm,
    Native,
    Docker,
}

impl Default for SkillRuntimeType {
    fn default() -> Self {
        Self::Wasm
    }
}

/// Intermediate representation of a skill before validation
#[derive(Debug, Clone)]
pub struct RawSkill {
    /// Skill name from manifest
    pub name: String,

    /// Description from manifest or SKILL.md
    pub description: Option<String>,

    /// Source path or URL
    pub source: String,

    /// Runtime type
    pub runtime: SkillRuntimeType,

    /// Tools discovered from the skill
    pub tools: Vec<RawTool>,

    /// SKILL.md content if available
    pub skill_md_content: Option<String>,
}

/// Intermediate representation of a tool before validation
#[derive(Debug, Clone)]
pub struct RawTool {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: String,

    /// Parameters
    pub parameters: Vec<RawToolParameter>,

    /// Whether streaming is supported
    pub streaming: bool,
}

/// Intermediate representation of a parameter
#[derive(Debug, Clone)]
pub struct RawToolParameter {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
}

/// Validated skill (passes all Claude Agent Skills requirements)
#[derive(Debug, Clone)]
pub struct ValidatedSkill {
    /// Validated name (lowercase, <= 64 chars)
    pub name: String,

    /// Validated description (<= 1024 chars)
    pub description: String,

    /// Source info
    #[allow(dead_code)]
    pub source: String,

    /// Runtime type
    pub runtime: SkillRuntimeType,

    /// Validated tools
    pub tools: Vec<ValidatedTool>,

    /// Original SKILL.md content
    #[allow(dead_code)]
    pub skill_md_content: Option<String>,
}

/// Validated tool
#[derive(Debug, Clone)]
pub struct ValidatedTool {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ValidatedParameter>,
    pub streaming: bool,
}

/// Validated parameter
#[derive(Debug, Clone)]
pub struct ValidatedParameter {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
}
