//! API types for request and response payloads

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Host service requirement with current status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SkillServiceRequirement {
    /// Service name (e.g., "kubectl-proxy")
    pub name: String,
    /// Human-readable description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// If true, service enhances functionality but isn't required
    pub optional: bool,
    /// Default port the service runs on
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_port: Option<u16>,
    /// Current service status
    pub status: ServiceStatus,
}

/// Summary information about an installed skill
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SkillSummary {
    /// Unique skill name
    pub name: String,
    /// Skill version
    pub version: String,
    /// Short description
    pub description: String,
    /// Source (git URL, local path, registry)
    pub source: String,
    /// Runtime type (wasm, docker, native)
    pub runtime: String,
    /// Number of tools provided
    pub tools_count: usize,
    /// Number of configured instances
    pub instances_count: usize,
    /// Last time a tool was executed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<DateTime<Utc>>,
    /// Total execution count
    pub execution_count: u64,
    /// Required host services with their current status
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_services: Vec<SkillServiceRequirement>,
}

/// Detailed skill information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SkillDetail {
    /// Basic summary
    #[serde(flatten)]
    pub summary: SkillSummary,
    /// Full description (markdown)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_description: Option<String>,
    /// Author information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Repository URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// License
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// Tools provided by this skill
    pub tools: Vec<ToolInfo>,
    /// Configured instances
    pub instances: Vec<InstanceInfo>,
}

/// Information about a tool
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolInfo {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Parameters
    pub parameters: Vec<ParameterInfo>,
    /// Whether this tool supports streaming
    pub streaming: bool,
}

/// Information about a parameter
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ParameterInfo {
    /// Parameter name
    pub name: String,
    /// Parameter type
    #[serde(rename = "type")]
    pub param_type: String,
    /// Description
    pub description: String,
    /// Whether parameter is required
    pub required: bool,
    /// Default value if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
}

/// Information about a skill instance
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstanceInfo {
    /// Instance name
    pub name: String,
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether this is the default instance
    pub is_default: bool,
    /// Configuration keys (values hidden for security)
    pub config_keys: Vec<String>,
}

/// Request to install a skill
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstallSkillRequest {
    /// Source to install from (git URL, local path, registry)
    pub source: String,
    /// Optional name override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Git ref (branch, tag, commit)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_ref: Option<String>,
    /// Whether to force reinstall
    #[serde(default)]
    pub force: bool,
}

/// Response from installing a skill
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstallSkillResponse {
    /// Whether installation succeeded
    pub success: bool,
    /// Installed skill name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Installed version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Number of tools installed
    pub tools_count: usize,
}

/// Request to execute a tool
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExecutionRequest {
    /// Skill name
    pub skill: String,
    /// Tool name
    pub tool: String,
    /// Instance to use (defaults to "default")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    /// Arguments as key-value pairs
    #[serde(default)]
    pub args: HashMap<String, serde_json::Value>,
    /// Whether to stream output
    #[serde(default)]
    pub stream: bool,
    /// Timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
}

/// Result of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExecutionResponse {
    /// Unique execution ID
    pub id: String,
    /// Execution status
    pub status: ExecutionStatus,
    /// Output content
    pub output: String,
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Additional metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    Pending,
    Running,
    Success,
    Failed,
    Timeout,
    Cancelled,
}

/// Execution history entry
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExecutionHistoryEntry {
    /// Execution ID
    pub id: String,
    /// Skill name
    pub skill: String,
    /// Tool name
    pub tool: String,
    /// Instance used
    pub instance: String,
    /// Status
    pub status: ExecutionStatus,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// When the execution started
    pub started_at: DateTime<Utc>,
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Output content (stdout/result)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

/// Request to search for skills/tools
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchRequest {
    /// Search query
    pub query: String,
    /// Maximum number of results
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    /// Filter by skill name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_filter: Option<String>,
    /// Include AI-generated examples in results
    #[serde(default)]
    pub include_examples: bool,
}

fn default_top_k() -> usize {
    5
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchResult {
    /// Result ID
    pub id: String,
    /// Skill name
    pub skill: String,
    /// Tool name
    pub tool: String,
    /// Content/description
    pub content: String,
    /// Relevance score (0.0 - 1.0)
    pub score: f32,
    /// Rerank score if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rerank_score: Option<f32>,
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchResponse {
    /// Search results
    pub results: Vec<SearchResult>,
    /// Query processing info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_info: Option<QueryInfo>,
    /// Total time in milliseconds
    pub duration_ms: u64,
}

/// Query processing information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct QueryInfo {
    /// Normalized query
    pub normalized: String,
    /// Detected intent
    pub intent: String,
    /// Confidence score
    pub confidence: f32,
}

/// Search configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchConfigResponse {
    /// Embedding provider
    pub embedding_provider: String,
    /// Embedding model
    pub embedding_model: String,
    /// Vector dimensions
    pub dimensions: usize,
    /// Vector store backend
    pub vector_backend: String,
    /// Whether hybrid search is enabled
    pub hybrid_search_enabled: bool,
    /// Whether reranking is enabled
    pub reranking_enabled: bool,
    /// Number of indexed documents
    pub indexed_documents: usize,
}

/// Update search configuration request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateSearchConfigRequest {
    /// Embedding provider (fastembed, openai, ollama)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_provider: Option<String>,
    /// Embedding model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_model: Option<String>,
    /// Vector backend (inmemory, qdrant)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_backend: Option<String>,
    /// Enable/disable hybrid search
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_hybrid: Option<bool>,
    /// Enable/disable reranking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_reranking: Option<bool>,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AppConfig {
    /// Default timeout in seconds
    pub default_timeout_secs: u64,
    /// Maximum concurrent executions
    pub max_concurrent_executions: usize,
    /// Whether to enable execution history
    pub enable_history: bool,
    /// Maximum history entries to keep
    pub max_history_entries: usize,
    /// Search configuration
    pub search: SearchConfigResponse,
}

/// Update application configuration request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateAppConfigRequest {
    /// Default timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_timeout_secs: Option<u64>,
    /// Maximum concurrent executions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_concurrent_executions: Option<usize>,
    /// Whether to enable execution history
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_history: Option<bool>,
    /// Maximum history entries to keep
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_history_entries: Option<usize>,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    /// Overall status
    pub status: String,
    /// Whether all components are healthy
    pub healthy: bool,
    /// Component statuses
    pub components: HashMap<String, ComponentHealth>,
    /// Server version
    pub version: String,
    /// Uptime in seconds
    pub uptime_secs: u64,
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Whether component is healthy
    pub healthy: bool,
    /// Optional status message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Version information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VersionResponse {
    /// Server version
    pub version: String,
    /// Build info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<String>,
    /// Git commit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    /// Rust version used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_version: Option<String>,
    /// Wasmtime version
    pub wasmtime_version: String,
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiError {
    /// Error code
    pub code: String,
    /// Human-readable message
    pub message: String,
    /// Additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn not_found(resource: &str) -> Self {
        Self::new("NOT_FOUND", format!("{} not found", resource))
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new("BAD_REQUEST", message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("INTERNAL_ERROR", message)
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new("VALIDATION_ERROR", message)
    }
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginationParams {
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: usize,
    /// Items per page
    #[serde(default = "default_per_page")]
    pub per_page: usize,
}

fn default_page() -> usize {
    1
}

fn default_per_page() -> usize {
    20
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginatedResponse<T> {
    /// Items for this page
    pub items: Vec<T>,
    /// Total number of items
    pub total: usize,
    /// Current page
    pub page: usize,
    /// Items per page
    pub per_page: usize,
    /// Total pages
    pub total_pages: usize,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: usize, page: usize, per_page: usize) -> Self {
        let total_pages = (total + per_page - 1) / per_page;
        Self {
            items,
            total,
            page,
            per_page,
            total_pages,
        }
    }
}

// =============================================================================
// Manifest Import Types
// =============================================================================

/// Request to import a manifest configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ImportManifestRequest {
    /// The manifest content (TOML format)
    pub content: String,
    /// Whether to merge with existing skills or replace
    #[serde(default)]
    pub merge: bool,
    /// Whether to install skills immediately or just validate
    #[serde(default)]
    pub install: bool,
}

/// Parsed skill from manifest
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ParsedSkill {
    /// Skill name
    pub name: String,
    /// Source location
    pub source: String,
    /// Runtime type
    pub runtime: String,
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Instances defined
    pub instances: Vec<ParsedInstance>,
    /// Docker configuration if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docker_config: Option<DockerConfig>,
}

/// Parsed instance from manifest
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ParsedInstance {
    /// Instance name
    pub name: String,
    /// Configuration keys (values hidden)
    pub config_keys: Vec<String>,
    /// Environment variable keys
    pub env_keys: Vec<String>,
    /// Whether it's the default instance
    pub is_default: bool,
}

/// Docker runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DockerConfig {
    /// Docker image
    pub image: String,
    /// Entrypoint command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<String>,
    /// Volume mounts
    #[serde(default)]
    pub volumes: Vec<String>,
    /// Working directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
    /// Memory limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
    /// CPU limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpus: Option<String>,
    /// Network mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
}

/// Response from parsing/importing a manifest
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ImportManifestResponse {
    /// Whether the import was successful
    pub success: bool,
    /// Parsed skills from the manifest
    pub skills: Vec<ParsedSkill>,
    /// Number of skills found
    pub skills_count: usize,
    /// Number of skills successfully installed (if install=true)
    pub installed_count: usize,
    /// Validation warnings
    #[serde(default)]
    pub warnings: Vec<String>,
    /// Errors encountered
    #[serde(default)]
    pub errors: Vec<String>,
}

/// Request to validate manifest content
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidateManifestRequest {
    /// The manifest content (TOML format)
    pub content: String,
}

/// Response from validating a manifest
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidateManifestResponse {
    /// Whether the manifest is valid
    pub valid: bool,
    /// Parsed skills (if valid)
    pub skills: Vec<ParsedSkill>,
    /// Validation errors
    #[serde(default)]
    pub errors: Vec<String>,
    /// Validation warnings
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Request to export current configuration as manifest
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExportManifestRequest {
    /// Format to export (toml, json)
    #[serde(default = "default_export_format")]
    pub format: String,
    /// Whether to include sensitive values (default: false)
    #[serde(default)]
    pub include_secrets: bool,
}

fn default_export_format() -> String {
    "toml".to_string()
}

/// Response with exported manifest
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExportManifestResponse {
    /// The manifest content
    pub content: String,
    /// Format used
    pub format: String,
    /// Number of skills included
    pub skills_count: usize,
}

// =============================================================================
// System Service Types
// =============================================================================

/// Status of a system service
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServiceStatus {
    /// Service name
    pub name: String,
    /// Whether the service is running
    pub running: bool,
    /// Process ID if running
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    /// Port the service is listening on
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    /// URL to access the service
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Error message if failed to start
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Response listing all system services
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServicesStatusResponse {
    /// List of service statuses
    pub services: Vec<ServiceStatus>,
}

/// Request to start a service
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StartServiceRequest {
    /// Service name to start
    pub service: String,
    /// Optional port to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
}

/// Response from starting a service
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StartServiceResponse {
    /// Whether the service was started successfully
    pub success: bool,
    /// Service status after starting
    pub status: ServiceStatus,
    /// Message about the operation
    pub message: String,
}

/// Request to stop a service
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StopServiceRequest {
    /// Service name to stop
    pub service: String,
}

// =============================================================================
// Vector DB Testing Types
// =============================================================================

/// Request to test search connection (quick validation)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TestConnectionRequest {
    /// Embedding provider (fastembed, openai, ollama)
    pub embedding_provider: String,
    /// Embedding model name
    pub embedding_model: String,
    /// Vector backend (inmemory, qdrant)
    pub vector_backend: String,
    /// Qdrant URL (if using Qdrant backend)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qdrant_url: Option<String>,
    /// Ollama URL (if using Ollama provider)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ollama_url: Option<String>,
}

/// Response from testing search connection
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TestConnectionResponse {
    /// Whether the test was successful
    pub success: bool,
    /// Embedding provider health status
    pub embedding_provider_status: ComponentHealth,
    /// Vector backend health status
    pub vector_backend_status: ComponentHealth,
    /// Test duration in milliseconds
    pub duration_ms: u128,
    /// Overall status message
    pub message: String,
}

/// Request to test full search pipeline (indexing + search)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TestPipelineRequest {
    /// Embedding provider (fastembed, openai, ollama)
    pub embedding_provider: String,
    /// Embedding model name
    pub embedding_model: String,
    /// Vector backend (inmemory, qdrant)
    pub vector_backend: String,
    /// Enable hybrid search (dense + sparse)
    pub enable_hybrid: bool,
    /// Enable cross-encoder reranking
    pub enable_reranking: bool,
    /// Qdrant URL (if using Qdrant backend)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qdrant_url: Option<String>,
}

/// Response from testing search pipeline
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TestPipelineResponse {
    /// Whether the test was successful
    pub success: bool,
    /// Indexing statistics
    pub index_stats: PipelineIndexStats,
    /// Search results from test query
    pub search_results: Vec<PipelineSearchResult>,
    /// Test duration in milliseconds
    pub duration_ms: u128,
    /// Overall status message
    pub message: String,
}

/// Indexing statistics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PipelineIndexStats {
    /// Number of documents indexed
    pub documents_indexed: usize,
    /// Indexing duration in milliseconds
    pub indexing_duration_ms: u64,
    /// Embedding generation duration in milliseconds
    pub embedding_duration_ms: u64,
}

/// Search result from pipeline
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PipelineSearchResult {
    /// Document ID
    pub id: String,
    /// Document content
    pub content: String,
    /// Similarity score (0.0 - 1.0)
    pub score: f32,
    /// Rerank score if reranking was enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rerank_score: Option<f32>,
    /// Document metadata
    pub metadata: DocumentMetadata,
}

/// Document metadata for search results
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentMetadata {
    /// Skill name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_name: Option<String>,
    /// Tool name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
}

// =============================================================================
// Agent Configuration Types
// =============================================================================

/// Agent runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentConfig {
    /// Agent runtime type
    pub runtime: AgentRuntime,
    /// Model configuration for the agent
    pub model_config: AgentModelConfig,
    /// Execution timeout in seconds
    pub timeout_secs: u64,
    /// Claude Code path (auto-detected if None)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claude_code_path: Option<String>,
}

/// Agent runtime type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AgentRuntime {
    /// Claude Code (Anthropic) - uses system installation
    ClaudeCode,
    /// Google Gemini
    Gemini,
    /// OpenAI GPT
    OpenAI,
    /// Custom agent implementation
    Custom,
}

/// Model configuration for agent
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentModelConfig {
    /// LLM provider (anthropic, openai, google)
    pub provider: String,
    /// Model name (e.g., claude-sonnet-4, gpt-4o, gemini-pro)
    pub model: String,
    /// Temperature (0.0 - 2.0)
    pub temperature: f32,
    /// Maximum tokens for responses
    pub max_tokens: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            runtime: AgentRuntime::ClaudeCode,
            model_config: AgentModelConfig {
                provider: "anthropic".to_string(),
                model: "claude-sonnet-4".to_string(),
                temperature: 0.7,
                max_tokens: 4096,
            },
            timeout_secs: 300,
            claude_code_path: None, // Auto-detect on first use
        }
    }
}

/// Response with agent configuration and available options
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetAgentConfigResponse {
    /// Current agent configuration
    pub config: AgentConfig,
    /// Available agent runtimes
    pub available_runtimes: Vec<RuntimeInfo>,
    /// Available models by provider
    pub available_models: HashMap<String, Vec<ModelInfo>>,
    /// Whether Claude Code is detected on system
    pub claude_code_detected: bool,
    /// Detected Claude Code version if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claude_code_version: Option<String>,
}

/// Information about an agent runtime
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RuntimeInfo {
    /// Runtime type
    pub runtime: AgentRuntime,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Supported LLM providers
    pub supported_providers: Vec<String>,
    /// Whether this runtime is available on the system
    pub available: bool,
}

/// Information about an LLM model
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModelInfo {
    /// Model ID (e.g., claude-sonnet-4)
    pub id: String,
    /// Display name
    pub name: String,
    /// Maximum context tokens
    pub max_tokens: usize,
    /// Whether the model supports tool use
    pub supports_tools: bool,
}

/// Request to update agent configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateAgentConfigRequest {
    /// Agent runtime (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<AgentRuntime>,
    /// Model configuration (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_config: Option<AgentModelConfig>,
    /// Timeout in seconds (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
    /// Claude Code path override (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claude_code_path: Option<String>,
}

/// Response from indexing operation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IndexResponse {
    /// Whether indexing was successful
    pub success: bool,
    /// Number of documents indexed
    pub documents_indexed: usize,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Status message
    pub message: String,
    /// Indexing statistics
    pub stats: IndexStats,
}

/// Indexing statistics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IndexStats {
    /// Number of documents added
    pub documents_added: usize,
    /// Number of documents updated
    pub documents_updated: usize,
    /// Total documents in index
    pub total_documents: usize,
    /// Index size in bytes (approximate)
    pub index_size_bytes: Option<usize>,
}

// ============================================================================
// Feedback API Types
// ============================================================================

/// Submit feedback request
#[derive(Debug, Clone, Deserialize)]
pub struct SubmitFeedbackRequest {
    /// The search query that produced this result
    pub query: String,
    /// The result ID that feedback is for
    pub result_id: String,
    /// The score of the result
    pub score: f32,
    /// The rank position in results (0-based)
    pub rank: usize,
    /// Type of feedback
    pub feedback_type: String, // "positive" or "negative"
    /// Optional reason for the feedback
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Optional comment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// Client type (optional, defaults to "http")
    #[serde(default = "default_client_type")]
    pub client_type: String,
}

fn default_client_type() -> String {
    "http".to_string()
}

/// Submit feedback response
#[derive(Debug, Clone, Serialize)]
pub struct SubmitFeedbackResponse {
    /// Success status
    pub success: bool,
    /// Feedback ID
    pub feedback_id: String,
    /// Message
    pub message: String,
}

/// Get feedback request (query parameters)
#[derive(Debug, Clone, Deserialize)]
pub struct GetFeedbackRequest {
    /// Filter by query text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    /// Filter by result ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_id: Option<String>,
    /// Filter by feedback type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_type: Option<String>,
    /// Limit results
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Offset for pagination
    #[serde(default)]
    pub offset: usize,
}

fn default_limit() -> usize {
    100
}

/// Get feedback response
#[derive(Debug, Clone, Serialize)]
pub struct GetFeedbackResponse {
    /// Feedback entries
    pub feedback: Vec<FeedbackEntry>,
    /// Total count (before pagination)
    pub total_count: usize,
    /// Limit used
    pub limit: usize,
    /// Offset used
    pub offset: usize,
}

/// Feedback entry for API responses
#[derive(Debug, Clone, Serialize)]
pub struct FeedbackEntry {
    /// Feedback ID
    pub id: String,
    /// Query text
    pub query: String,
    /// Result ID
    pub result_id: String,
    /// Result score
    pub score: f32,
    /// Result rank
    pub rank: usize,
    /// Feedback type
    pub feedback_type: String,
    /// Reason
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Comment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// Client type
    pub client_type: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

// Analytics Dashboard Types

/// Analytics overview response
#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsOverviewResponse {
    /// Total number of searches
    pub total_searches: usize,
    /// Total feedback submissions
    pub total_feedback: usize,
    /// Positive feedback count
    pub positive_feedback: usize,
    /// Negative feedback count
    pub negative_feedback: usize,
    /// Average search latency in milliseconds
    pub avg_latency_ms: f64,
    /// Average results per search
    pub avg_results: f64,
    /// Most recent searches
    pub recent_searches: Vec<SearchHistorySummary>,
}

/// Search history summary for dashboard
#[derive(Debug, Clone, Serialize)]
pub struct SearchHistorySummary {
    pub query: String,
    pub results_count: usize,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

/// Top queries response
#[derive(Debug, Clone, Serialize)]
pub struct TopQueriesResponse {
    pub queries: Vec<QueryStats>,
}

/// Query statistics
#[derive(Debug, Clone, Serialize)]
pub struct QueryStats {
    pub query: String,
    pub count: usize,
    pub avg_results: f64,
    pub avg_latency_ms: f64,
    pub positive_feedback: usize,
    pub negative_feedback: usize,
}

/// Feedback statistics response
#[derive(Debug, Clone, Serialize)]
pub struct FeedbackStatsResponse {
    /// Feedback by type
    pub by_type: Vec<FeedbackTypeCount>,
    /// Top positively rated results
    pub top_positive: Vec<ResultFeedbackSummary>,
    /// Top negatively rated results
    pub top_negative: Vec<ResultFeedbackSummary>,
}

/// Feedback count by type
#[derive(Debug, Clone, Serialize)]
pub struct FeedbackTypeCount {
    pub feedback_type: String,
    pub count: usize,
}

/// Result feedback summary
#[derive(Debug, Clone, Serialize)]
pub struct ResultFeedbackSummary {
    pub result_id: String,
    pub positive_count: usize,
    pub negative_count: usize,
    pub total_count: usize,
}

/// Search timeline data
#[derive(Debug, Clone, Serialize)]
pub struct SearchTimelineResponse {
    pub timeline: Vec<TimelineDataPoint>,
}

/// Timeline data point
#[derive(Debug, Clone, Serialize)]
pub struct TimelineDataPoint {
    pub timestamp: DateTime<Utc>,
    pub search_count: usize,
    pub avg_latency_ms: f64,
}
