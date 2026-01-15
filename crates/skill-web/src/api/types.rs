//! API request and response types
//!
//! These types mirror the skill-http API types for serialization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Pagination
// ============================================================================

/// Pagination query parameters
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

impl PaginationParams {
    pub fn new(page: usize, per_page: usize) -> Self {
        Self { page, per_page }
    }
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
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

// ============================================================================
// Skills
// ============================================================================

/// Host service requirement for a skill
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillServiceRequirement {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub optional: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_port: Option<u16>,
    pub status: ServiceStatus,
}

/// Skill summary from API
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillSummary {
    pub name: String,
    pub version: String,
    pub description: String,
    pub source: String,
    pub runtime: String,
    pub tools_count: usize,
    pub instances_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<String>,
    pub execution_count: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_services: Vec<SkillServiceRequirement>,
}

/// Skill detail from API
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillDetail {
    #[serde(flatten)]
    pub summary: SkillSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    pub tools: Vec<ToolInfo>,
    pub instances: Vec<InstanceInfo>,
}

/// Tool information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ParameterInfo>,
    pub streaming: bool,
}

/// Parameter information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
}

/// Instance information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub is_default: bool,
    pub config_keys: Vec<String>,
}

/// Install skill request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallSkillRequest {
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_ref: Option<String>,
    #[serde(default)]
    pub force: bool,
}

/// Install skill response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallSkillResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub tools_count: usize,
}

// ============================================================================
// Execution
// ============================================================================

/// Execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub skill: String,
    pub tool: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    #[serde(default)]
    pub args: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
}

/// Execution response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionResponse {
    pub id: String,
    pub status: ExecutionStatus,
    pub output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub duration_ms: u64,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

/// Execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionHistoryEntry {
    pub id: String,
    pub skill: String,
    pub tool: String,
    pub instance: String,
    pub status: ExecutionStatus,
    pub duration_ms: u64,
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

// ============================================================================
// Search
// ============================================================================

/// Search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_filter: Option<String>,
    #[serde(default)]
    pub include_examples: bool,
}

fn default_top_k() -> usize {
    5
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub skill: String,
    pub tool: String,
    pub content: String,
    pub score: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rerank_score: Option<f32>,
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_info: Option<QueryInfo>,
    pub duration_ms: u64,
}

/// Query processing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryInfo {
    pub normalized: String,
    pub intent: String,
    pub confidence: f32,
}

// ============================================================================
// Configuration
// ============================================================================

/// Search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfigResponse {
    pub embedding_provider: String,
    pub embedding_model: String,
    pub dimensions: usize,
    pub vector_backend: String,
    pub hybrid_search_enabled: bool,
    pub reranking_enabled: bool,
    pub indexed_documents: usize,
}

/// Update search config request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateSearchConfigRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_backend: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_hybrid: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_reranking: Option<bool>,
}

/// Response from indexing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexResponse {
    pub success: bool,
    pub documents_indexed: usize,
    pub duration_ms: u64,
    pub message: String,
    pub stats: IndexStats,
}

/// Indexing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub documents_added: usize,
    pub documents_updated: usize,
    pub total_documents: usize,
    pub index_size_bytes: Option<usize>,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub default_timeout_secs: u64,
    pub max_concurrent_executions: usize,
    pub enable_history: bool,
    pub max_history_entries: usize,
    pub search: SearchConfigResponse,
}

/// Update app config request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateAppConfigRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_timeout_secs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_concurrent_executions: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_history: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_history_entries: Option<usize>,
}

// ============================================================================
// Health & Version
// ============================================================================

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub healthy: bool,
    pub components: HashMap<String, ComponentHealth>,
    pub version: String,
    pub uptime_secs: u64,
}

/// Component health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub healthy: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Version response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionResponse {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_version: Option<String>,
    pub wasmtime_version: String,
}

// =============================================================================
// Manifest Import Types
// =============================================================================

/// Request to import a manifest configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportManifestRequest {
    pub content: String,
    #[serde(default)]
    pub merge: bool,
    #[serde(default)]
    pub install: bool,
}

/// Parsed skill from manifest
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedSkill {
    pub name: String,
    pub source: String,
    pub runtime: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub instances: Vec<ParsedInstance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docker_config: Option<DockerConfig>,
}

/// Parsed instance from manifest
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedInstance {
    pub name: String,
    pub config_keys: Vec<String>,
    pub env_keys: Vec<String>,
    pub is_default: bool,
}

/// Docker runtime configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DockerConfig {
    pub image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<String>,
    #[serde(default)]
    pub volumes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpus: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
}

/// Response from importing a manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportManifestResponse {
    pub success: bool,
    pub skills: Vec<ParsedSkill>,
    pub skills_count: usize,
    pub installed_count: usize,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub errors: Vec<String>,
}

/// Request to validate manifest content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateManifestRequest {
    pub content: String,
}

/// Response from validating a manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateManifestResponse {
    pub valid: bool,
    pub skills: Vec<ParsedSkill>,
    #[serde(default)]
    pub errors: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

// ============================================================================
// System Services
// ============================================================================

/// Status of a system service
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub running: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Response listing all system services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesStatusResponse {
    pub services: Vec<ServiceStatus>,
}

/// Request to start a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartServiceRequest {
    pub service: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
}

/// Response from starting a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartServiceResponse {
    pub success: bool,
    pub status: ServiceStatus,
    pub message: String,
}

/// Request to stop a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopServiceRequest {
    pub service: String,
}

// ============================================================================
// Vector DB Testing Types
// ============================================================================

/// Request to test search connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConnectionRequest {
    pub embedding_provider: String,
    pub embedding_model: String,
    pub vector_backend: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qdrant_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ollama_url: Option<String>,
}

/// Response from testing search connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConnectionResponse {
    pub success: bool,
    pub embedding_provider_status: ComponentHealth,
    pub vector_backend_status: ComponentHealth,
    pub duration_ms: u128,
    pub message: String,
}

/// Request to test full search pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPipelineRequest {
    pub embedding_provider: String,
    pub embedding_model: String,
    pub vector_backend: String,
    pub enable_hybrid: bool,
    pub enable_reranking: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qdrant_url: Option<String>,
}

/// Response from testing search pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPipelineResponse {
    pub success: bool,
    pub index_stats: PipelineIndexStats,
    pub search_results: Vec<PipelineSearchResult>,
    pub duration_ms: u128,
    pub message: String,
}

/// Pipeline indexing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineIndexStats {
    pub documents_indexed: usize,
    pub indexing_duration_ms: u64,
    pub embedding_duration_ms: u64,
}

/// Search result from pipeline
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipelineSearchResult {
    pub id: String,
    pub content: String,
    pub score: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rerank_score: Option<f32>,
    pub metadata: DocumentMetadata,
}

/// Document metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

// ============================================================================
// Agent Configuration Types
// ============================================================================

/// Agent runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub runtime: AgentRuntime,
    pub model_config: AgentModelConfig,
    pub timeout_secs: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claude_code_path: Option<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            runtime: AgentRuntime::ClaudeCode,
            model_config: AgentModelConfig::default(),
            timeout_secs: 300,
            claude_code_path: None,
        }
    }
}

/// Agent runtime type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentRuntime {
    ClaudeCode,
    Gemini,
    #[serde(rename = "openai")]
    OpenAI,
    Custom,
}

/// Model configuration for agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentModelConfig {
    pub provider: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: usize,
}

impl Default for AgentModelConfig {
    fn default() -> Self {
        Self {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
        }
    }
}

/// Response with agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAgentConfigResponse {
    pub config: AgentConfig,
    pub available_runtimes: Vec<RuntimeInfo>,
    pub available_models: HashMap<String, Vec<ModelInfo>>,
    pub claude_code_detected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claude_code_version: Option<String>,
}

/// Information about an agent runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeInfo {
    pub runtime: AgentRuntime,
    pub name: String,
    pub description: String,
    pub supported_providers: Vec<String>,
    pub available: bool,
}

/// Information about an LLM model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub max_tokens: usize,
    pub supports_tools: bool,
}

/// Request to update agent configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateAgentConfigRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<AgentRuntime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_config: Option<AgentModelConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claude_code_path: Option<String>,
}
