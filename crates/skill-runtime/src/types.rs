use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata about a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    /// Unique name of the skill
    pub name: String,
    /// Semantic version number
    pub version: String,
    /// Human-readable description of the skill
    pub description: String,
    /// Author or organization name
    pub author: String,
    /// URL to the source code repository
    pub repository: Option<String>,
    /// License identifier (e.g., MIT, Apache-2.0)
    pub license: Option<String>,
}

/// Definition of a tool provided by a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool identifier
    pub name: String,
    /// Human-readable description of what the tool does
    pub description: String,
    /// List of parameters accepted by this tool
    pub parameters: Vec<Parameter>,
    /// Whether this tool supports streaming output
    pub streaming: bool,
}

/// Parameter definition for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter identifier
    pub name: String,
    #[serde(rename = "type")]
    /// Data type of the parameter
    pub param_type: ParameterType,
    /// Human-readable description of the parameter
    pub description: String,
    /// Whether this parameter must be provided
    pub required: bool,
    /// Default value if not provided
    pub default_value: Option<String>,
}

/// Supported parameter types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    /// Text string value
    String,
    /// Numeric value (integer or float)
    Number,
    /// Boolean true/false value
    Boolean,
    /// File path or file content
    File,
    /// JSON object or structured data
    Json,
    /// Array of values
    Array,
}

/// Result of tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Whether the execution completed successfully
    pub success: bool,
    /// Standard output or result data
    pub output: String,
    /// Error message if execution failed
    pub error_message: Option<String>,
    /// Additional metadata about the execution
    pub metadata: Option<HashMap<String, String>>,
}

/// Chunk of streaming output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// Type of stream chunk
    pub chunk_type: StreamChunkType,
    /// Content of the chunk
    pub data: String,
}

/// Type of stream chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamChunkType {
    /// Standard output stream
    Stdout,
    /// Standard error stream
    Stderr,
    /// Progress update
    Progress,
    /// Metadata information
    Metadata,
}

/// Configuration key-value pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValue {
    /// Configuration key name
    pub key: String,
    /// Configuration value
    pub value: String,
    /// Whether this is a sensitive value
    pub secret: bool,
}

/// Skill dependency declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Name of the required skill
    pub skill_name: String,
    /// Version requirement (e.g., ">=1.0.0")
    pub version_constraint: String,
    /// Whether this dependency is optional
    pub optional: bool,
}

/// Log level for host logging
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Trace-level logging (most verbose)
    Trace,
    /// Debug-level logging
    Debug,
    /// Informational logging
    Info,
    /// Warning-level logging
    Warn,
    /// Error-level logging
    Error,
}

impl From<LogLevel> for tracing::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}
