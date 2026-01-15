//! Example Generator - Core component for AI-powered example synthesis
//!
//! Generates realistic usage examples from tool schemas using LLMs,
//! with streaming output and validation.

use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::{Context, Result};
use futures_util::Stream;
use tokio_stream::StreamExt;

use crate::skill_md::ToolDocumentation;
use crate::search_config::AiIngestionConfig;
use super::llm_provider::{LlmProvider, CompletionRequest};
use super::validator::ExampleValidator;
use super::streaming::{GenerationEvent, GeneratedExample, GenerationStreamBuilder};

/// Configuration for the example generator
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Number of examples to generate per tool
    pub examples_per_tool: usize,
    /// Whether to validate generated examples
    pub validate_examples: bool,
    /// Maximum retries for failed generation
    pub max_retries: usize,
    /// Timeout for generation
    pub timeout: Duration,
    /// Temperature for LLM generation
    pub temperature: f32,
    /// Maximum tokens for response
    pub max_tokens: u32,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            examples_per_tool: 5,
            validate_examples: true,
            max_retries: 2,
            timeout: Duration::from_secs(30),
            temperature: 0.7,
            max_tokens: 2048,
        }
    }
}

impl From<&AiIngestionConfig> for GeneratorConfig {
    fn from(config: &AiIngestionConfig) -> Self {
        Self {
            examples_per_tool: config.examples_per_tool,
            validate_examples: config.validate_examples,
            max_retries: 2,
            timeout: Duration::from_secs(config.timeout_secs),
            temperature: 0.7,
            max_tokens: 2048,
        }
    }
}

/// AI-powered example generator
pub struct ExampleGenerator {
    /// LLM provider for generation
    llm: Arc<dyn LlmProvider>,
    /// Example validator
    validator: ExampleValidator,
    /// Generator configuration
    config: GeneratorConfig,
}

impl ExampleGenerator {
    /// Create a new example generator
    pub fn new(llm: Arc<dyn LlmProvider>, config: GeneratorConfig) -> Self {
        Self {
            llm,
            validator: ExampleValidator::new(),
            config,
        }
    }

    /// Create from AI ingestion config
    pub fn from_config(llm: Arc<dyn LlmProvider>, config: &AiIngestionConfig) -> Self {
        Self::new(llm, GeneratorConfig::from(config))
    }

    /// Generate examples for a tool (non-streaming)
    pub async fn generate(&self, tool: &ToolDocumentation) -> Result<Vec<GeneratedExample>> {
        let mut results = Vec::new();
        let mut stream = Box::pin(self.generate_stream(tool, 1, 1));

        while let Some(event) = stream.next().await {
            if let GenerationEvent::Example { example } = event {
                results.push(example);
            }
        }

        Ok(results)
    }

    /// Generate examples for multiple tools (non-streaming)
    pub async fn generate_batch(
        &self,
        tools: &[ToolDocumentation],
    ) -> Result<Vec<(String, Vec<GeneratedExample>)>> {
        let mut results = Vec::new();

        for tool in tools {
            let examples = self.generate(tool).await?;
            results.push((tool.name.clone(), examples));
        }

        Ok(results)
    }

    /// Generate examples with streaming events
    pub fn generate_stream<'a>(
        &'a self,
        tool: &'a ToolDocumentation,
        current_index: usize,
        total_tools: usize,
    ) -> impl Stream<Item = GenerationEvent> + 'a {
        async_stream::stream! {
            let start_time = Instant::now();
            let builder = GenerationStreamBuilder::new(&tool.name, total_tools, current_index);

            // Emit started event
            yield builder.started();

            // Build the prompt
            yield builder.thinking(format!("Building prompt for {} parameters...", tool.parameters.len()));

            let prompt = self.build_prompt(tool);

            // Create completion request
            let request = CompletionRequest::with_system(
                SYSTEM_PROMPT,
                &prompt,
            )
            .temperature(self.config.temperature)
            .max_tokens(self.config.max_tokens);

            yield builder.thinking("Generating examples with LLM...");

            // Generate with retries
            let mut attempts = 0;
            let mut examples = Vec::new();

            loop {
                attempts += 1;

                match self.llm.complete(&request).await {
                    Ok(response) => {
                        yield builder.thinking("Parsing LLM response...");

                        // Parse examples from response
                        match self.parse_examples(&response.content) {
                            Ok(parsed) => {
                                examples = parsed;
                                break;
                            }
                            Err(e) => {
                                if attempts >= self.config.max_retries {
                                    yield builder.error(
                                        format!("Failed to parse response after {} attempts: {}", attempts, e),
                                        false,
                                    );
                                    return;
                                }
                                yield builder.thinking(format!("Retrying ({}/{}): {}", attempts, self.config.max_retries, e));
                            }
                        }
                    }
                    Err(e) => {
                        if attempts >= self.config.max_retries {
                            yield builder.error(
                                format!("LLM generation failed after {} attempts: {}", attempts, e),
                                false,
                            );
                            return;
                        }
                        yield builder.thinking(format!("Retrying ({}/{}): {}", attempts, self.config.max_retries, e));
                    }
                }
            }

            // Process and validate each example
            let total_examples = examples.len();
            let mut valid_count = 0;

            for (idx, mut example) in examples.into_iter().enumerate() {
                // Validate if enabled
                if self.config.validate_examples {
                    let validation = self.validator.validate_example(&example, tool);

                    yield builder.validation(
                        validation.valid,
                        validation.errors.clone(),
                        idx,
                    );

                    if validation.valid {
                        example.validated = true;
                        example.confidence = validation.confidence;
                        valid_count += 1;
                        yield builder.example(example);
                    }
                } else {
                    yield builder.example(example);
                    valid_count += 1;
                }

                // Progress update
                yield GenerationEvent::progress(
                    idx + 1,
                    total_examples,
                    Some(format!("Processed {}/{} examples", idx + 1, total_examples)),
                );
            }

            // Emit completion
            let duration = start_time.elapsed();
            yield builder.tool_completed(total_examples, valid_count, duration);
        }
    }

    /// Build the prompt for example generation
    fn build_prompt(&self, tool: &ToolDocumentation) -> String {
        let params_desc = self.format_parameters(tool);
        let existing_examples = self.format_existing_examples(tool);

        format!(
            r#"Generate {count} realistic CLI usage examples for the following tool:

## Tool Information
- **Name**: {name}
- **Description**: {description}

## Parameters
{parameters}

{existing}

## Requirements
1. Each example must use valid parameter values
2. Cover diverse use cases (common operations, edge cases, real-world scenarios)
3. Include a brief explanation for each example
4. Use the format: `skill run {name} [options]`

## Output Format
Return a JSON array with exactly {count} examples:
```json
[
  {{"command": "skill run {name} --param=value", "explanation": "Brief description of what this does"}},
  ...
]
```

Generate {count} diverse, realistic examples now:"#,
            count = self.config.examples_per_tool,
            name = tool.name,
            description = tool.description,
            parameters = params_desc,
            existing = existing_examples,
        )
    }

    /// Format parameters for the prompt
    fn format_parameters(&self, tool: &ToolDocumentation) -> String {
        if tool.parameters.is_empty() {
            return "No parameters defined.".to_string();
        }

        tool.parameters
            .iter()
            .map(|p| {
                let required = if p.required { " (required)" } else { "" };
                let default = p.default.as_ref()
                    .map(|d| format!(" [default: {}]", d))
                    .unwrap_or_default();
                let allowed = if !p.allowed_values.is_empty() {
                    format!(" [values: {}]", p.allowed_values.join(", "))
                } else {
                    String::new()
                };

                format!(
                    "- `--{name}` ({type}){required}{default}{allowed}: {desc}",
                    name = p.name,
                    type = format!("{:?}", p.param_type).to_lowercase(),
                    required = required,
                    default = default,
                    allowed = allowed,
                    desc = p.description,
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format existing examples (if any) to avoid duplicates
    fn format_existing_examples(&self, tool: &ToolDocumentation) -> String {
        if tool.examples.is_empty() {
            return String::new();
        }

        let examples = tool.examples
            .iter()
            .take(3)
            .map(|e| format!("- `{}`", e.code.lines().next().unwrap_or(&e.code)))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "\n## Existing Examples (do not duplicate)\n{}\n",
            examples
        )
    }

    /// Parse examples from LLM response
    fn parse_examples(&self, response: &str) -> Result<Vec<GeneratedExample>> {
        // Try to find JSON array in response
        let json_str = self.extract_json_array(response)?;

        // Parse the JSON
        let parsed: serde_json::Value = serde_json::from_str(&json_str)
            .with_context(|| format!("Failed to parse JSON: {}", &json_str[..json_str.len().min(100)]))?;

        let array = parsed.as_array()
            .context("Expected JSON array")?;

        let examples: Vec<GeneratedExample> = array
            .iter()
            .filter_map(|item| {
                let command = item.get("command")?.as_str()?;
                let explanation = item.get("explanation")?.as_str()?;

                Some(GeneratedExample::new(command, explanation))
            })
            .collect();

        if examples.is_empty() {
            anyhow::bail!("No valid examples found in response");
        }

        Ok(examples)
    }

    /// Extract JSON array from response text
    fn extract_json_array(&self, response: &str) -> Result<String> {
        // Try to find JSON array directly
        if let Some(start) = response.find('[') {
            if let Some(end) = response.rfind(']') {
                if end > start {
                    return Ok(response[start..=end].to_string());
                }
            }
        }

        // Try to find JSON in code block
        if let Some(start) = response.find("```json") {
            let after_marker = &response[start + 7..];
            if let Some(end) = after_marker.find("```") {
                let json_content = &after_marker[..end];
                if let Some(arr_start) = json_content.find('[') {
                    if let Some(arr_end) = json_content.rfind(']') {
                        return Ok(json_content[arr_start..=arr_end].to_string());
                    }
                }
            }
        }

        // Try to find any code block
        if let Some(start) = response.find("```") {
            let after_marker = &response[start + 3..];
            // Skip optional language identifier
            let content_start = after_marker.find('\n').unwrap_or(0) + 1;
            let after_newline = &after_marker[content_start..];
            if let Some(end) = after_newline.find("```") {
                let json_content = &after_newline[..end];
                if let Some(arr_start) = json_content.find('[') {
                    if let Some(arr_end) = json_content.rfind(']') {
                        return Ok(json_content[arr_start..=arr_end].to_string());
                    }
                }
            }
        }

        anyhow::bail!("Could not find JSON array in response")
    }

    /// Get the LLM provider name
    pub fn provider_name(&self) -> &str {
        self.llm.name()
    }

    /// Get the model name
    pub fn model_name(&self) -> &str {
        self.llm.model()
    }
}

/// System prompt for example generation
const SYSTEM_PROMPT: &str = r#"You are a CLI tool documentation expert who generates realistic usage examples.

Your task is to create diverse, practical examples that demonstrate various use cases for command-line tools.

Guidelines:
- Generate valid commands with proper parameter syntax
- Cover common use cases, edge cases, and real-world scenarios
- Include meaningful explanations that help users understand each example
- Use realistic parameter values (not placeholders like "value1", "example")
- Ensure examples are syntactically correct and would execute successfully

Output your examples as a JSON array with "command" and "explanation" fields."#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill_md::{ParameterDoc, ParameterType};

    fn create_test_tool() -> ToolDocumentation {
        ToolDocumentation {
            name: "apply".to_string(),
            description: "Apply a Kubernetes manifest".to_string(),
            usage: None,
            parameters: vec![
                ParameterDoc {
                    name: "file".to_string(),
                    param_type: ParameterType::String,
                    description: "Path to manifest file".to_string(),
                    required: true,
                    default: None,
                    allowed_values: vec![],
                },
                ParameterDoc {
                    name: "namespace".to_string(),
                    param_type: ParameterType::String,
                    description: "Target namespace".to_string(),
                    required: false,
                    default: Some("default".to_string()),
                    allowed_values: vec![],
                },
            ],
            examples: vec![],
        }
    }

    #[test]
    fn test_build_prompt() {
        // Create a mock provider for testing
        struct MockProvider;

        #[async_trait::async_trait]
        impl LlmProvider for MockProvider {
            fn name(&self) -> &str { "mock" }
            fn model(&self) -> &str { "test" }
            async fn complete(&self, _: &CompletionRequest) -> Result<super::super::llm_provider::LlmResponse> {
                unimplemented!()
            }
            async fn complete_stream(&self, _: &CompletionRequest) -> Result<Pin<Box<dyn Stream<Item = Result<super::super::llm_provider::LlmChunk>> + Send>>> {
                unimplemented!()
            }
        }

        let generator = ExampleGenerator::new(
            Arc::new(MockProvider),
            GeneratorConfig::default(),
        );
        let tool = create_test_tool();

        let prompt = generator.build_prompt(&tool);

        assert!(prompt.contains("apply"));
        assert!(prompt.contains("Kubernetes manifest"));
        assert!(prompt.contains("--file"));
        assert!(prompt.contains("--namespace"));
        assert!(prompt.contains("(required)"));
        assert!(prompt.contains("[default: default]"));
    }

    #[test]
    fn test_parse_examples_json() {
        struct MockProvider;
        #[async_trait::async_trait]
        impl LlmProvider for MockProvider {
            fn name(&self) -> &str { "mock" }
            fn model(&self) -> &str { "test" }
            async fn complete(&self, _: &CompletionRequest) -> Result<super::super::llm_provider::LlmResponse> {
                unimplemented!()
            }
            async fn complete_stream(&self, _: &CompletionRequest) -> Result<Pin<Box<dyn Stream<Item = Result<super::super::llm_provider::LlmChunk>> + Send>>> {
                unimplemented!()
            }
        }

        let generator = ExampleGenerator::new(
            Arc::new(MockProvider),
            GeneratorConfig::default(),
        );

        let response = r#"
Here are the examples:
[
  {"command": "skill run apply --file=deploy.yaml", "explanation": "Apply deployment"},
  {"command": "skill run apply --file=service.yaml --namespace=prod", "explanation": "Apply to prod"}
]
        "#;

        let examples = generator.parse_examples(response).unwrap();
        assert_eq!(examples.len(), 2);
        assert!(examples[0].command.contains("deploy.yaml"));
        assert!(examples[1].command.contains("namespace=prod"));
    }

    #[test]
    fn test_parse_examples_code_block() {
        struct MockProvider;
        #[async_trait::async_trait]
        impl LlmProvider for MockProvider {
            fn name(&self) -> &str { "mock" }
            fn model(&self) -> &str { "test" }
            async fn complete(&self, _: &CompletionRequest) -> Result<super::super::llm_provider::LlmResponse> {
                unimplemented!()
            }
            async fn complete_stream(&self, _: &CompletionRequest) -> Result<Pin<Box<dyn Stream<Item = Result<super::super::llm_provider::LlmChunk>> + Send>>> {
                unimplemented!()
            }
        }

        let generator = ExampleGenerator::new(
            Arc::new(MockProvider),
            GeneratorConfig::default(),
        );

        let response = r#"
Here are some examples:

```json
[
  {"command": "skill run test --param=value", "explanation": "Test command"}
]
```
        "#;

        let examples = generator.parse_examples(response).unwrap();
        assert_eq!(examples.len(), 1);
    }

    #[test]
    fn test_config_from_ai_ingestion() {
        let ai_config = AiIngestionConfig {
            enabled: true,
            examples_per_tool: 3,
            timeout_secs: 60,
            ..Default::default()
        };

        let config = GeneratorConfig::from(&ai_config);
        assert_eq!(config.examples_per_tool, 3);
        assert_eq!(config.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_extract_json_array_direct() {
        struct MockProvider;
        #[async_trait::async_trait]
        impl LlmProvider for MockProvider {
            fn name(&self) -> &str { "mock" }
            fn model(&self) -> &str { "test" }
            async fn complete(&self, _: &CompletionRequest) -> Result<super::super::llm_provider::LlmResponse> {
                unimplemented!()
            }
            async fn complete_stream(&self, _: &CompletionRequest) -> Result<Pin<Box<dyn Stream<Item = Result<super::super::llm_provider::LlmChunk>> + Send>>> {
                unimplemented!()
            }
        }

        let generator = ExampleGenerator::new(
            Arc::new(MockProvider),
            GeneratorConfig::default(),
        );

        let input = r#"[{"a": 1}]"#;
        let result = generator.extract_json_array(input).unwrap();
        assert_eq!(result, r#"[{"a": 1}]"#);
    }
}
