//! Context compression for token-efficient tool retrieval output
//!
//! Reduces token usage by 60-70% while maintaining execution-ready
//! tool interfaces. Uses progressive expansion based on relevance rank.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tiktoken_rs::cl100k_base;

/// Compression strategy for tool contexts
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CompressionStrategy {
    /// Keep first sentence + parameter list
    Extractive,
    /// Structured format with placeholders
    #[default]
    TemplateBased,
    /// More detail for top results, less for lower ranks
    Progressive,
    /// No compression - full context
    None,
}

impl std::str::FromStr for CompressionStrategy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "extractive" => Ok(Self::Extractive),
            "template" | "template-based" | "templatebased" => Ok(Self::TemplateBased),
            "progressive" => Ok(Self::Progressive),
            "none" | "full" => Ok(Self::None),
            _ => anyhow::bail!("Unknown compression strategy: {}. Options: extractive, template, progressive, none", s),
        }
    }
}

/// Configuration for context compression
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Maximum tokens per tool (default: 200)
    pub max_tokens_per_tool: usize,
    /// Maximum total tokens across all tools (default: 800)
    pub max_total_tokens: usize,
    /// Include code examples (default: false)
    pub include_examples: bool,
    /// Include full parameter documentation (default: false)
    pub include_full_params: bool,
    /// Compression strategy
    pub strategy: CompressionStrategy,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            max_tokens_per_tool: 200,
            max_total_tokens: 800,
            include_examples: false,
            include_full_params: false,
            strategy: CompressionStrategy::default(),
        }
    }
}

impl CompressionConfig {
    /// Create config with a specific strategy
    pub fn with_strategy(strategy: CompressionStrategy) -> Self {
        Self {
            strategy,
            ..Default::default()
        }
    }

    /// Set max tokens per tool
    pub fn max_per_tool(mut self, tokens: usize) -> Self {
        self.max_tokens_per_tool = tokens;
        self
    }

    /// Set max total tokens
    pub fn max_total(mut self, tokens: usize) -> Self {
        self.max_total_tokens = tokens;
        self
    }

    /// Include examples in output
    pub fn with_examples(mut self) -> Self {
        self.include_examples = true;
        self
    }

    /// Include full parameter documentation
    pub fn with_full_params(mut self) -> Self {
        self.include_full_params = true;
        self
    }
}

/// A tool parameter with compressed documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type (e.g., "string", "number", "object")
    pub param_type: String,
    /// Whether the parameter is required
    pub required: bool,
    /// One-line description
    pub description: String,
}

/// Compressed tool context optimized for token efficiency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedToolContext {
    /// Tool identifier (skill@instance/tool_name)
    pub tool_id: String,
    /// One-line summary of the tool
    pub summary: String,
    /// Execution hint for the LLM
    pub execution_hint: Option<String>,
    /// Required and important parameters only
    pub parameters: Vec<ToolParameter>,
    /// Relevance score from search/reranking
    pub relevance_score: f32,
    /// Rank in the result set (1-indexed)
    pub rank: usize,
    /// Token count for this compressed context
    pub token_count: usize,
}

/// Result of context compression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionResult {
    /// Compressed tool contexts
    pub tools: Vec<CompressedToolContext>,
    /// Total token count
    pub total_tokens: usize,
    /// Original token count (before compression)
    pub original_tokens: usize,
    /// Compression ratio (compressed / original)
    pub compression_ratio: f32,
}

/// Input tool document for compression
#[derive(Debug, Clone)]
pub struct ToolDocument {
    /// Tool identifier (skill@instance/tool_name)
    pub tool_id: String,
    /// Tool name
    pub name: String,
    /// Full description
    pub description: String,
    /// Parameter definitions
    pub parameters: Vec<ToolParameterInput>,
    /// Optional code example
    pub example: Option<String>,
    /// Relevance score
    pub relevance_score: f32,
}

/// Input parameter definition
#[derive(Debug, Clone)]
pub struct ToolParameterInput {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: String,
    /// Whether required
    pub required: bool,
    /// Full description
    pub description: String,
}

/// Context compressor for reducing token usage
pub struct ContextCompressor {
    config: CompressionConfig,
    tokenizer: tiktoken_rs::CoreBPE,
}

impl ContextCompressor {
    /// Create a new context compressor with default config
    pub fn new() -> Result<Self> {
        Self::with_config(CompressionConfig::default())
    }

    /// Create with specific config
    pub fn with_config(config: CompressionConfig) -> Result<Self> {
        let tokenizer = cl100k_base()?;
        Ok(Self { config, tokenizer })
    }

    /// Count tokens in text
    pub fn count_tokens(&self, text: &str) -> usize {
        self.tokenizer.encode_ordinary(text).len()
    }

    /// Compress tool documents for efficient LLM context
    pub fn compress(&self, tools: Vec<ToolDocument>) -> CompressionResult {
        // Calculate original token count
        let original_tokens: usize = tools.iter()
            .map(|t| self.count_full_tool_tokens(t))
            .sum();

        let mut compressed_tools = Vec::with_capacity(tools.len());
        let mut total_tokens = 0;

        for (rank, tool) in tools.into_iter().enumerate() {
            let rank = rank + 1; // 1-indexed

            let compressed = match self.config.strategy {
                CompressionStrategy::Extractive => self.compress_extractive(&tool, rank),
                CompressionStrategy::TemplateBased => self.compress_template(&tool, rank),
                CompressionStrategy::Progressive => self.compress_progressive(&tool, rank),
                CompressionStrategy::None => self.no_compression(&tool, rank),
            };

            // Check token budget
            if total_tokens + compressed.token_count > self.config.max_total_tokens {
                break;
            }

            total_tokens += compressed.token_count;
            compressed_tools.push(compressed);
        }

        let compression_ratio = if original_tokens > 0 {
            total_tokens as f32 / original_tokens as f32
        } else {
            1.0
        };

        CompressionResult {
            tools: compressed_tools,
            total_tokens,
            original_tokens,
            compression_ratio,
        }
    }

    /// Count tokens for full uncompressed tool
    fn count_full_tool_tokens(&self, tool: &ToolDocument) -> usize {
        let mut text = format!("{}\n{}\n", tool.tool_id, tool.description);
        for param in &tool.parameters {
            text.push_str(&format!("{}: {} - {}\n", param.name, param.param_type, param.description));
        }
        if let Some(ref example) = tool.example {
            text.push_str(&format!("Example: {}\n", example));
        }
        self.count_tokens(&text)
    }

    /// Extractive compression: first sentence + parameters
    fn compress_extractive(&self, tool: &ToolDocument, rank: usize) -> CompressedToolContext {
        let summary = self.extract_first_sentence(&tool.description);
        let parameters = self.compress_parameters(&tool.parameters, false);

        let compressed = CompressedToolContext {
            tool_id: tool.tool_id.clone(),
            summary,
            execution_hint: None,
            parameters,
            relevance_score: tool.relevance_score,
            rank,
            token_count: 0, // Will be calculated
        };

        self.finalize_compressed(compressed)
    }

    /// Template-based compression with structured format
    fn compress_template(&self, tool: &ToolDocument, rank: usize) -> CompressedToolContext {
        let summary = self.extract_first_sentence(&tool.description);
        let parameters = self.compress_parameters(&tool.parameters, self.config.include_full_params);

        // Generate execution hint
        let param_names: Vec<&str> = tool.parameters.iter()
            .filter(|p| p.required)
            .map(|p| p.name.as_str())
            .collect();

        let execution_hint = if !param_names.is_empty() {
            Some(format!("Call with: {}", param_names.join(", ")))
        } else {
            Some("Call with no required parameters".to_string())
        };

        let compressed = CompressedToolContext {
            tool_id: tool.tool_id.clone(),
            summary,
            execution_hint,
            parameters,
            relevance_score: tool.relevance_score,
            rank,
            token_count: 0,
        };

        self.finalize_compressed(compressed)
    }

    /// Progressive compression: more detail for top results
    fn compress_progressive(&self, tool: &ToolDocument, rank: usize) -> CompressedToolContext {
        match rank {
            // Rank 1: Full parameters + example hint
            1 => {
                let summary = self.extract_first_two_sentences(&tool.description);
                let parameters = self.compress_parameters(&tool.parameters, true);

                let execution_hint = tool.example.as_ref().map(|ex| {
                    // Extract just the function call pattern
                    self.extract_call_pattern(ex)
                });

                let compressed = CompressedToolContext {
                    tool_id: tool.tool_id.clone(),
                    summary,
                    execution_hint,
                    parameters,
                    relevance_score: tool.relevance_score,
                    rank,
                    token_count: 0,
                };

                self.finalize_compressed(compressed)
            }
            // Rank 2-3: Required parameters only
            2 | 3 => {
                let summary = self.extract_first_sentence(&tool.description);
                let parameters = self.compress_parameters_required_only(&tool.parameters);

                let compressed = CompressedToolContext {
                    tool_id: tool.tool_id.clone(),
                    summary,
                    execution_hint: None,
                    parameters,
                    relevance_score: tool.relevance_score,
                    rank,
                    token_count: 0,
                };

                self.finalize_compressed(compressed)
            }
            // Rank 4+: Minimal - name + one-line
            _ => {
                let summary = self.extract_first_sentence(&tool.description);
                // Truncate summary to very short
                let summary = if summary.len() > 80 {
                    format!("{}...", &summary[..77])
                } else {
                    summary
                };

                let compressed = CompressedToolContext {
                    tool_id: tool.tool_id.clone(),
                    summary,
                    execution_hint: None,
                    parameters: Vec::new(), // No parameters for low-rank tools
                    relevance_score: tool.relevance_score,
                    rank,
                    token_count: 0,
                };

                self.finalize_compressed(compressed)
            }
        }
    }

    /// No compression - return full context
    fn no_compression(&self, tool: &ToolDocument, rank: usize) -> CompressedToolContext {
        let parameters: Vec<ToolParameter> = tool.parameters.iter()
            .map(|p| ToolParameter {
                name: p.name.clone(),
                param_type: p.param_type.clone(),
                required: p.required,
                description: p.description.clone(),
            })
            .collect();

        let compressed = CompressedToolContext {
            tool_id: tool.tool_id.clone(),
            summary: tool.description.clone(),
            execution_hint: tool.example.clone(),
            parameters,
            relevance_score: tool.relevance_score,
            rank,
            token_count: 0,
        };

        self.finalize_compressed(compressed)
    }

    /// Finalize by calculating token count and enforcing limits
    fn finalize_compressed(&self, mut ctx: CompressedToolContext) -> CompressedToolContext {
        let text = self.context_to_text(&ctx);
        let token_count = self.count_tokens(&text);

        // If over budget, truncate
        if token_count > self.config.max_tokens_per_tool {
            ctx.summary = self.truncate_to_tokens(&ctx.summary, 50);
            ctx.execution_hint = None;
            if ctx.parameters.len() > 3 {
                ctx.parameters.truncate(3);
            }
        }

        let final_text = self.context_to_text(&ctx);
        ctx.token_count = self.count_tokens(&final_text);
        ctx
    }

    /// Convert context to text for token counting
    fn context_to_text(&self, ctx: &CompressedToolContext) -> String {
        let mut text = format!("{}: {}", ctx.tool_id, ctx.summary);
        if let Some(ref hint) = ctx.execution_hint {
            text.push_str(&format!(" [{}]", hint));
        }
        for param in &ctx.parameters {
            text.push_str(&format!(" {}:{}", param.name, param.param_type));
        }
        text
    }

    /// Extract first sentence from text
    fn extract_first_sentence(&self, text: &str) -> String {
        let text = text.trim();
        if let Some(idx) = text.find(|c| c == '.' || c == '!' || c == '?') {
            let sentence = text[..=idx].trim().to_string();
            if sentence.len() < 200 {
                return sentence;
            }
        }
        // Fallback: truncate at word boundary
        if text.len() > 150 {
            let truncated = &text[..150];
            if let Some(idx) = truncated.rfind(' ') {
                return format!("{}...", &truncated[..idx]);
            }
        }
        text.to_string()
    }

    /// Extract first two sentences
    fn extract_first_two_sentences(&self, text: &str) -> String {
        let first = self.extract_first_sentence(text);
        let remaining = &text[first.len()..].trim_start();

        if remaining.is_empty() {
            return first;
        }

        let second = self.extract_first_sentence(remaining);
        if !second.is_empty() && first.len() + second.len() < 300 {
            format!("{} {}", first, second)
        } else {
            first
        }
    }

    /// Extract call pattern from example
    fn extract_call_pattern(&self, example: &str) -> String {
        // Look for function call patterns
        if let Some(start) = example.find('(') {
            if let Some(end) = example.find(')') {
                let call = &example[..=end];
                // Find the start of the function name
                let name_start = call.rfind(|c: char| c.is_whitespace() || c == '{' || c == ':')
                    .map(|i| i + 1)
                    .unwrap_or(0);
                return call[name_start..].to_string();
            }
        }
        // Fallback: truncate example
        if example.len() > 60 {
            format!("{}...", &example[..57])
        } else {
            example.to_string()
        }
    }

    /// Compress parameters list
    fn compress_parameters(&self, params: &[ToolParameterInput], include_full: bool) -> Vec<ToolParameter> {
        params.iter()
            .map(|p| {
                let description = if include_full {
                    p.description.clone()
                } else {
                    self.truncate_description(&p.description, 50)
                };

                ToolParameter {
                    name: p.name.clone(),
                    param_type: p.param_type.clone(),
                    required: p.required,
                    description,
                }
            })
            .collect()
    }

    /// Compress to required parameters only
    fn compress_parameters_required_only(&self, params: &[ToolParameterInput]) -> Vec<ToolParameter> {
        params.iter()
            .filter(|p| p.required)
            .map(|p| ToolParameter {
                name: p.name.clone(),
                param_type: p.param_type.clone(),
                required: true,
                description: self.truncate_description(&p.description, 40),
            })
            .collect()
    }

    /// Truncate description to word boundary
    fn truncate_description(&self, desc: &str, max_chars: usize) -> String {
        if desc.len() <= max_chars {
            return desc.to_string();
        }

        let truncated = &desc[..max_chars];
        if let Some(idx) = truncated.rfind(' ') {
            format!("{}...", &truncated[..idx])
        } else {
            format!("{}...", truncated)
        }
    }

    /// Truncate text to approximate token count
    fn truncate_to_tokens(&self, text: &str, max_tokens: usize) -> String {
        let tokens = self.tokenizer.encode_ordinary(text);
        if tokens.len() <= max_tokens {
            return text.to_string();
        }

        // Decode truncated tokens
        let truncated_tokens = &tokens[..max_tokens];
        match self.tokenizer.decode(truncated_tokens.to_vec()) {
            Ok(decoded) => {
                // Find last word boundary
                if let Some(idx) = decoded.rfind(' ') {
                    format!("{}...", &decoded[..idx])
                } else {
                    format!("{}...", decoded)
                }
            }
            Err(_) => {
                // Fallback to character truncation
                let char_estimate = max_tokens * 4;
                if text.len() > char_estimate {
                    format!("{}...", &text[..char_estimate])
                } else {
                    text.to_string()
                }
            }
        }
    }

    /// Get the config
    pub fn config(&self) -> &CompressionConfig {
        &self.config
    }
}

impl Default for ContextCompressor {
    fn default() -> Self {
        Self::new().expect("Failed to create default ContextCompressor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tool() -> ToolDocument {
        ToolDocument {
            tool_id: "kubernetes@default/list_pods".to_string(),
            name: "list_pods".to_string(),
            description: "List all pods in a Kubernetes namespace. Returns pod names, status, and resource usage. Use this to monitor cluster workloads.".to_string(),
            parameters: vec![
                ToolParameterInput {
                    name: "namespace".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "The Kubernetes namespace to query. Use 'default' for the default namespace or specify a custom namespace name.".to_string(),
                },
                ToolParameterInput {
                    name: "label_selector".to_string(),
                    param_type: "string".to_string(),
                    required: false,
                    description: "Optional label selector to filter pods. Format: 'key=value' or 'key in (v1,v2)'.".to_string(),
                },
            ],
            example: Some("list_pods(namespace='production', label_selector='app=web')".to_string()),
            relevance_score: 0.95,
        }
    }

    #[test]
    fn test_token_counting() {
        let compressor = ContextCompressor::new().unwrap();
        let tokens = compressor.count_tokens("Hello, world!");
        assert!(tokens > 0);
        assert!(tokens < 10);
    }

    #[test]
    fn test_first_sentence_extraction() {
        let compressor = ContextCompressor::new().unwrap();

        let text = "List all pods in a namespace. Returns pod names and status. Use for monitoring.";
        let first = compressor.extract_first_sentence(text);
        assert_eq!(first, "List all pods in a namespace.");

        let short = "Short text without period";
        let first = compressor.extract_first_sentence(short);
        assert_eq!(first, "Short text without period");
    }

    #[test]
    fn test_compress_extractive() {
        let compressor = ContextCompressor::with_config(
            CompressionConfig::with_strategy(CompressionStrategy::Extractive)
        ).unwrap();

        let result = compressor.compress(vec![sample_tool()]);

        assert_eq!(result.tools.len(), 1);
        let tool = &result.tools[0];
        assert!(tool.summary.contains("List all pods"));
        assert!(!tool.summary.contains("Returns pod names")); // Only first sentence
        assert!(tool.token_count > 0);
    }

    #[test]
    fn test_compress_progressive() {
        let compressor = ContextCompressor::with_config(
            CompressionConfig::with_strategy(CompressionStrategy::Progressive)
                .max_total(2000) // High limit to get all tools
        ).unwrap();

        let tools = vec![
            sample_tool(),
            {
                let mut t = sample_tool();
                t.tool_id = "kubernetes@default/get_deployment".to_string();
                t.relevance_score = 0.85;
                t
            },
            {
                let mut t = sample_tool();
                t.tool_id = "kubernetes@default/delete_pod".to_string();
                t.relevance_score = 0.75;
                t
            },
            {
                let mut t = sample_tool();
                t.tool_id = "kubernetes@default/create_service".to_string();
                t.relevance_score = 0.65;
                t
            },
        ];

        let result = compressor.compress(tools);

        // Rank 1 should have more detail
        assert!(result.tools[0].execution_hint.is_some());
        assert!(!result.tools[0].parameters.is_empty());

        // Rank 4 should have minimal detail
        assert!(result.tools[3].execution_hint.is_none());
        assert!(result.tools[3].parameters.is_empty());
    }

    #[test]
    fn test_compression_ratio() {
        let compressor = ContextCompressor::with_config(
            CompressionConfig::with_strategy(CompressionStrategy::TemplateBased)
        ).unwrap();

        let result = compressor.compress(vec![sample_tool()]);

        // Should achieve significant compression
        assert!(result.compression_ratio < 1.0);
        assert!(result.total_tokens < result.original_tokens);
    }

    #[test]
    fn test_token_budget_enforcement() {
        let compressor = ContextCompressor::with_config(
            CompressionConfig::default()
                .max_total(50) // Very small budget
        ).unwrap();

        let tools = vec![sample_tool(), sample_tool(), sample_tool()];
        let result = compressor.compress(tools);

        // Should not exceed budget
        assert!(result.total_tokens <= 50);
        // Might not include all tools
        assert!(result.tools.len() <= 3);
    }

    #[test]
    fn test_no_compression() {
        let compressor = ContextCompressor::with_config(
            CompressionConfig::with_strategy(CompressionStrategy::None)
                .max_total(10000)
        ).unwrap();

        let result = compressor.compress(vec![sample_tool()]);

        // Should have full description
        let tool = &result.tools[0];
        assert!(tool.summary.contains("Returns pod names"));
        // No compression has minimal compression ratio (some overhead in formatting)
        assert!(result.compression_ratio > 0.4); // Should keep most of the content
    }

    #[test]
    fn test_strategy_from_str() {
        assert_eq!("extractive".parse::<CompressionStrategy>().unwrap(), CompressionStrategy::Extractive);
        assert_eq!("template".parse::<CompressionStrategy>().unwrap(), CompressionStrategy::TemplateBased);
        assert_eq!("progressive".parse::<CompressionStrategy>().unwrap(), CompressionStrategy::Progressive);
        assert_eq!("none".parse::<CompressionStrategy>().unwrap(), CompressionStrategy::None);
        assert!("invalid".parse::<CompressionStrategy>().is_err());
    }

    #[test]
    fn test_serialize_compressed_context() {
        let compressor = ContextCompressor::new().unwrap();
        let result = compressor.compress(vec![sample_tool()]);

        // Should serialize to valid JSON
        let json = serde_json::to_string_pretty(&result).unwrap();
        assert!(json.contains("kubernetes@default/list_pods"));

        // Should deserialize back
        let deserialized: CompressionResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tools.len(), 1);
    }
}
