//! Test fixtures for AI generation testing
//!
//! Provides realistic tool schemas, mock LLM responses, and test utilities
//! for comprehensive testing of the generation pipeline.

use std::collections::HashMap;
use std::pin::Pin;
use anyhow::Result;
use async_trait::async_trait;
use futures_util::Stream;

use crate::skill_md::{ToolDocumentation, ParameterDoc, ParameterType, CodeExample};
use super::llm_provider::{LlmProvider, LlmResponse, LlmChunk, CompletionRequest};

// =============================================================================
// Tool Fixtures
// =============================================================================

/// Create a complex Kubernetes tool fixture
pub fn kubernetes_apply_tool() -> ToolDocumentation {
    ToolDocumentation {
        name: "apply".to_string(),
        description: "Apply a configuration to a resource by file name or stdin. The resource name must be specified.".to_string(),
        usage: Some("skill run kubernetes:apply --file=<manifest.yaml> [--namespace=<ns>] [--dry-run]".to_string()),
        parameters: vec![
            ParameterDoc {
                name: "file".to_string(),
                param_type: ParameterType::String,
                description: "Path to the file that contains the configuration to apply".to_string(),
                required: true,
                default: None,
                allowed_values: vec![],
            },
            ParameterDoc {
                name: "namespace".to_string(),
                param_type: ParameterType::String,
                description: "If present, the namespace scope for this CLI request".to_string(),
                required: false,
                default: Some("default".to_string()),
                allowed_values: vec![],
            },
            ParameterDoc {
                name: "dry-run".to_string(),
                param_type: ParameterType::Boolean,
                description: "Preview the object that would be sent without actually sending it".to_string(),
                required: false,
                default: None,
                allowed_values: vec![],
            },
            ParameterDoc {
                name: "output".to_string(),
                param_type: ParameterType::String,
                description: "Output format".to_string(),
                required: false,
                default: None,
                allowed_values: vec!["json".to_string(), "yaml".to_string(), "wide".to_string()],
            },
            ParameterDoc {
                name: "wait".to_string(),
                param_type: ParameterType::Boolean,
                description: "Wait for resources to be ready".to_string(),
                required: false,
                default: None,
                allowed_values: vec![],
            },
            ParameterDoc {
                name: "timeout".to_string(),
                param_type: ParameterType::Integer,
                description: "Timeout in seconds for the operation".to_string(),
                required: false,
                default: Some("300".to_string()),
                allowed_values: vec![],
            },
        ],
        examples: vec![
            CodeExample {
                language: Some("bash".to_string()),
                code: "skill run kubernetes:apply --file=deployment.yaml".to_string(),
                description: Some("Apply a deployment manifest".to_string()),
            },
        ],
    }
}

/// Create a simple tool with minimal parameters
pub fn simple_tool() -> ToolDocumentation {
    ToolDocumentation {
        name: "list".to_string(),
        description: "List all resources of a given type".to_string(),
        usage: None,
        parameters: vec![
            ParameterDoc {
                name: "type".to_string(),
                param_type: ParameterType::String,
                description: "Resource type to list".to_string(),
                required: true,
                default: None,
                allowed_values: vec![],
            },
        ],
        examples: vec![],
    }
}

/// Create a tool with enum constraints
pub fn tool_with_constraints() -> ToolDocumentation {
    ToolDocumentation {
        name: "get".to_string(),
        description: "Display one or many resources".to_string(),
        usage: None,
        parameters: vec![
            ParameterDoc {
                name: "resource".to_string(),
                param_type: ParameterType::String,
                description: "Resource type".to_string(),
                required: true,
                default: None,
                allowed_values: vec![
                    "pods".to_string(),
                    "deployments".to_string(),
                    "services".to_string(),
                    "configmaps".to_string(),
                    "secrets".to_string(),
                ],
            },
            ParameterDoc {
                name: "output".to_string(),
                param_type: ParameterType::String,
                description: "Output format".to_string(),
                required: false,
                default: None,
                allowed_values: vec!["json".to_string(), "yaml".to_string(), "wide".to_string()],
            },
            ParameterDoc {
                name: "all-namespaces".to_string(),
                param_type: ParameterType::Boolean,
                description: "List across all namespaces".to_string(),
                required: false,
                default: None,
                allowed_values: vec![],
            },
        ],
        examples: vec![],
    }
}

/// Create an AWS S3 tool
pub fn aws_s3_tool() -> ToolDocumentation {
    ToolDocumentation {
        name: "s3-copy".to_string(),
        description: "Copy files between S3 buckets or between local and S3".to_string(),
        usage: None,
        parameters: vec![
            ParameterDoc {
                name: "source".to_string(),
                param_type: ParameterType::String,
                description: "Source path (local path or s3://bucket/key)".to_string(),
                required: true,
                default: None,
                allowed_values: vec![],
            },
            ParameterDoc {
                name: "destination".to_string(),
                param_type: ParameterType::String,
                description: "Destination path (local path or s3://bucket/key)".to_string(),
                required: true,
                default: None,
                allowed_values: vec![],
            },
            ParameterDoc {
                name: "recursive".to_string(),
                param_type: ParameterType::Boolean,
                description: "Copy recursively".to_string(),
                required: false,
                default: None,
                allowed_values: vec![],
            },
            ParameterDoc {
                name: "region".to_string(),
                param_type: ParameterType::String,
                description: "AWS region".to_string(),
                required: false,
                default: Some("us-east-1".to_string()),
                allowed_values: vec![],
            },
        ],
        examples: vec![],
    }
}

/// Create a Docker build tool
pub fn docker_build_tool() -> ToolDocumentation {
    ToolDocumentation {
        name: "build".to_string(),
        description: "Build an image from a Dockerfile".to_string(),
        usage: None,
        parameters: vec![
            ParameterDoc {
                name: "context".to_string(),
                param_type: ParameterType::String,
                description: "Build context directory".to_string(),
                required: true,
                default: None,
                allowed_values: vec![],
            },
            ParameterDoc {
                name: "tag".to_string(),
                param_type: ParameterType::String,
                description: "Name and optionally a tag (name:tag)".to_string(),
                required: false,
                default: None,
                allowed_values: vec![],
            },
            ParameterDoc {
                name: "file".to_string(),
                param_type: ParameterType::String,
                description: "Name of the Dockerfile".to_string(),
                required: false,
                default: Some("Dockerfile".to_string()),
                allowed_values: vec![],
            },
            ParameterDoc {
                name: "no-cache".to_string(),
                param_type: ParameterType::Boolean,
                description: "Do not use cache when building".to_string(),
                required: false,
                default: None,
                allowed_values: vec![],
            },
        ],
        examples: vec![],
    }
}

// =============================================================================
// Mock LLM Responses
// =============================================================================

/// Get a mock JSON response for a tool
pub fn mock_response_for_tool(tool_name: &str) -> String {
    match tool_name {
        "apply" => r#"[
            {"command": "skill run kubernetes:apply --file=deployment.yaml", "explanation": "Apply a deployment manifest to the cluster"},
            {"command": "skill run kubernetes:apply --file=service.yaml --namespace=production", "explanation": "Apply a service in the production namespace"},
            {"command": "skill run kubernetes:apply --file=configmap.yaml --dry-run=true", "explanation": "Preview applying a configmap without making changes"},
            {"command": "skill run kubernetes:apply --file=app.yaml --namespace=staging --output=json", "explanation": "Apply manifest to staging and output result as JSON"},
            {"command": "skill run kubernetes:apply --file=./manifests/full-stack.yaml --wait --timeout=120", "explanation": "Apply and wait up to 120 seconds for resources to be ready"}
        ]"#.to_string(),
        "list" => r#"[
            {"command": "skill run tool:list --type=pods", "explanation": "List all pods"},
            {"command": "skill run tool:list --type=services", "explanation": "List all services"},
            {"command": "skill run tool:list --type=deployments", "explanation": "List all deployments"}
        ]"#.to_string(),
        "get" => r#"[
            {"command": "skill run kubernetes:get --resource=pods", "explanation": "Get all pods in the default namespace"},
            {"command": "skill run kubernetes:get --resource=deployments --output=json", "explanation": "Get deployments as JSON"},
            {"command": "skill run kubernetes:get --resource=services --all-namespaces", "explanation": "Get services across all namespaces"},
            {"command": "skill run kubernetes:get --resource=configmaps --output=yaml", "explanation": "Get configmaps in YAML format"},
            {"command": "skill run kubernetes:get --resource=secrets --all-namespaces --output=wide", "explanation": "Get secrets with extended info"}
        ]"#.to_string(),
        "s3-copy" => r#"[
            {"command": "skill run aws:s3-copy --source=./local-file.txt --destination=s3://my-bucket/file.txt", "explanation": "Upload local file to S3"},
            {"command": "skill run aws:s3-copy --source=s3://bucket-a/data.json --destination=s3://bucket-b/data.json", "explanation": "Copy file between S3 buckets"},
            {"command": "skill run aws:s3-copy --source=./data/ --destination=s3://backup-bucket/data/ --recursive", "explanation": "Upload entire directory to S3"},
            {"command": "skill run aws:s3-copy --source=s3://bucket/file.csv --destination=./downloads/file.csv --region=eu-west-1", "explanation": "Download from EU region bucket"}
        ]"#.to_string(),
        "build" => r#"[
            {"command": "skill run docker:build --context=. --tag=myapp:latest", "explanation": "Build image from current directory"},
            {"command": "skill run docker:build --context=./app --tag=myapp:v1.0 --file=Dockerfile.prod", "explanation": "Build with custom Dockerfile"},
            {"command": "skill run docker:build --context=. --tag=test:ci --no-cache", "explanation": "Build without cache for CI"},
            {"command": "skill run docker:build --context=./backend --tag=api:latest", "explanation": "Build backend API image"}
        ]"#.to_string(),
        _ => r#"[
            {"command": "skill run tool:command --param=value", "explanation": "Example command"}
        ]"#.to_string(),
    }
}

/// Get a mock response with some invalid examples for testing validation
pub fn mock_response_with_errors(tool_name: &str) -> String {
    match tool_name {
        "apply" => r#"[
            {"command": "skill run kubernetes:apply --file=valid.yaml", "explanation": "Valid example"},
            {"command": "skill run kubernetes:apply --namespace=prod", "explanation": "Missing required file parameter"},
            {"command": "skill run kubernetes:apply --file=test.yaml", "explanation": ""},
            {"command": "skill run kubernetes:apply --file=good.yaml --output=json", "explanation": "Another valid example"}
        ]"#.to_string(),
        _ => mock_response_for_tool(tool_name),
    }
}

// =============================================================================
// Mock LLM Provider
// =============================================================================

/// A deterministic mock LLM provider for testing
pub struct DeterministicMockProvider {
    /// Pre-configured responses by tool name
    responses: HashMap<String, String>,
    /// Default response when tool not found
    default_response: String,
    /// Artificial delay in ms (for latency testing)
    delay_ms: u64,
    /// Call counter
    call_count: std::sync::atomic::AtomicUsize,
}

impl DeterministicMockProvider {
    /// Create a new mock provider
    pub fn new() -> Self {
        let mut responses = HashMap::new();
        responses.insert("apply".to_string(), mock_response_for_tool("apply"));
        responses.insert("list".to_string(), mock_response_for_tool("list"));
        responses.insert("get".to_string(), mock_response_for_tool("get"));
        responses.insert("s3-copy".to_string(), mock_response_for_tool("s3-copy"));
        responses.insert("build".to_string(), mock_response_for_tool("build"));

        Self {
            responses,
            default_response: r#"[{"command": "skill run tool --param=value", "explanation": "Generic example"}]"#.to_string(),
            delay_ms: 0,
            call_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Create a mock that returns errors for some tools
    pub fn with_validation_errors() -> Self {
        let mut provider = Self::new();
        provider.responses.insert("apply".to_string(), mock_response_with_errors("apply"));
        provider
    }

    /// Set artificial delay for latency testing
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    /// Add a custom response for a tool
    pub fn with_response(mut self, tool_name: &str, response: &str) -> Self {
        self.responses.insert(tool_name.to_string(), response.to_string());
        self
    }

    /// Get the number of calls made
    pub fn call_count(&self) -> usize {
        self.call_count.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Extract tool name from prompt
    fn extract_tool_name(&self, prompt: &str) -> String {
        // Look for "Name: <tool_name>" in the prompt
        for line in prompt.lines() {
            if line.starts_with("- **Name**:") || line.starts_with("Name:") {
                return line
                    .split(':')
                    .nth(1)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
            }
        }
        "unknown".to_string()
    }
}

impl Default for DeterministicMockProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmProvider for DeterministicMockProvider {
    fn name(&self) -> &str {
        "mock"
    }

    fn model(&self) -> &str {
        "deterministic-test"
    }

    async fn complete(&self, request: &CompletionRequest) -> Result<LlmResponse> {
        self.call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        // Apply delay if configured
        if self.delay_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.delay_ms)).await;
        }

        // Extract tool name from user message
        let empty = String::new();
        let user_message = request.messages.iter()
            .find(|m| m.role == "user")
            .map(|m| &m.content)
            .unwrap_or(&empty);

        let tool_name = self.extract_tool_name(user_message);

        let content = self.responses
            .get(&tool_name)
            .cloned()
            .unwrap_or_else(|| self.default_response.clone());

        Ok(LlmResponse {
            content,
            model: "deterministic-test".to_string(),
            usage: None,
            finish_reason: Some("stop".to_string()),
        })
    }

    async fn complete_stream(
        &self,
        request: &CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<LlmChunk>> + Send>>> {
        // For simplicity, just return the full response as one chunk
        let response = self.complete(request).await?;

        let stream = async_stream::stream! {
            yield Ok(LlmChunk {
                delta: response.content,
                is_final: true,
            });
        };

        Ok(Box::pin(stream))
    }
}

// =============================================================================
// Failing Mock Provider
// =============================================================================

/// A mock provider that always fails
pub struct FailingMockProvider {
    error_message: String,
}

impl FailingMockProvider {
    /// Create a new failing mock provider with a custom error message
    pub fn new(message: &str) -> Self {
        Self {
            error_message: message.to_string(),
        }
    }
}

#[async_trait]
impl LlmProvider for FailingMockProvider {
    fn name(&self) -> &str {
        "failing-mock"
    }

    fn model(&self) -> &str {
        "error-test"
    }

    async fn complete(&self, _request: &CompletionRequest) -> Result<LlmResponse> {
        anyhow::bail!("{}", self.error_message)
    }

    async fn complete_stream(
        &self,
        _request: &CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<LlmChunk>> + Send>>> {
        anyhow::bail!("{}", self.error_message)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kubernetes_tool_fixture() {
        let tool = kubernetes_apply_tool();
        assert_eq!(tool.name, "apply");
        assert!(!tool.parameters.is_empty());

        // Check required parameter
        let file_param = tool.parameters.iter().find(|p| p.name == "file").unwrap();
        assert!(file_param.required);
    }

    #[test]
    fn test_tool_with_constraints() {
        let tool = tool_with_constraints();
        let resource_param = tool.parameters.iter().find(|p| p.name == "resource").unwrap();
        assert!(!resource_param.allowed_values.is_empty());
        assert!(resource_param.allowed_values.contains(&"pods".to_string()));
    }

    #[test]
    fn test_mock_response_parsing() {
        let response = mock_response_for_tool("apply");
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&response).unwrap();
        assert_eq!(parsed.len(), 5);

        for example in &parsed {
            assert!(example.get("command").is_some());
            assert!(example.get("explanation").is_some());
        }
    }

    #[tokio::test]
    async fn test_deterministic_mock_provider() {
        let provider = DeterministicMockProvider::new();

        let request = CompletionRequest::with_system(
            "You are a CLI expert",
            "Generate examples for:\n- **Name**: apply\n- **Description**: Apply manifest"
        );

        let response = provider.complete(&request).await.unwrap();
        assert!(response.content.contains("deployment.yaml"));
        assert_eq!(provider.call_count(), 1);
    }

    #[tokio::test]
    async fn test_failing_provider() {
        let provider = FailingMockProvider::new("Test error");
        let request = CompletionRequest::new("test");

        let result = provider.complete(&request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Test error"));
    }
}
