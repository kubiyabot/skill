//! OpenAPI specification generation for the Skill HTTP API
//!
//! This module provides OpenAPI 3.1 documentation for all REST endpoints using utoipa.

use utoipa::OpenApi;

use crate::types::*;

/// OpenAPI documentation for the Skill Engine HTTP API
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Skill Engine API",
        version = "1.0.0",
        description = "REST API for managing and executing AI agent skills",
        license(
            name = "Apache-2.0",
            url = "https://www.apache.org/licenses/LICENSE-2.0"
        ),
        contact(
            name = "Skill Engine",
            url = "https://github.com/kubiyabot/skill"
        )
    ),
    servers(
        (url = "http://localhost:3000/api", description = "Local development server"),
        (url = "https://api.skill-engine.dev/api", description = "Production API")
    ),
    components(
        schemas(
            SkillSummary,
            SkillDetail,
            ToolInfo,
            ParameterInfo,
            InstanceInfo,
            InstallSkillRequest,
            InstallSkillResponse,
            ExecutionRequest,
            ExecutionResponse,
            ExecutionStatus,
            ExecutionHistoryEntry,
            SearchRequest,
            SearchResult,
            SearchResponse,
            QueryInfo,
            SearchConfigResponse,
            UpdateSearchConfigRequest,
            AppConfig,
            UpdateAppConfigRequest,
            HealthResponse,
            ComponentHealth,
            VersionResponse,
            ApiError,
            PaginationParams,
            ImportManifestRequest,
            ImportManifestResponse,
            ValidateManifestRequest,
            ValidateManifestResponse,
            ExportManifestRequest,
            ExportManifestResponse,
            ServiceStatus,
            ServicesStatusResponse,
            StartServiceRequest,
            StartServiceResponse,
            StopServiceRequest,
            SkillServiceRequirement,
            ParsedSkill,
            ParsedInstance,
            DockerConfig,
        )
    ),
    tags(
        (name = "skills", description = "Skill management operations"),
        (name = "execution", description = "Tool execution operations"),
        (name = "search", description = "Semantic search operations"),
        (name = "config", description = "Configuration management"),
        (name = "services", description = "System service management"),
        (name = "manifest", description = "Manifest import/export operations"),
        (name = "system", description = "System health and version"),
    )
)]
pub struct ApiDoc;

/// Generate the OpenAPI specification as JSON
pub fn generate_openapi_json() -> String {
    ApiDoc::openapi().to_pretty_json().expect("Failed to serialize OpenAPI spec")
}
