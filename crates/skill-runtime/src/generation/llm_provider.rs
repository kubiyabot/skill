//! LLM Provider abstraction for AI-powered generation
//!
//! Provides a unified interface for multiple LLM providers (Ollama, OpenAI, Anthropic)
//! with streaming support.

use anyhow::Result;
use async_trait::async_trait;
use std::pin::Pin;
use futures_util::Stream;

use crate::search_config::{AiIngestionConfig, AiProvider};

/// Response from an LLM completion
#[derive(Debug, Clone)]
pub struct LlmResponse {
    /// Generated text content
    pub content: String,
    /// Model that generated the response
    pub model: String,
    /// Token usage statistics (if available)
    pub usage: Option<TokenUsage>,
    /// Completion finish reason
    pub finish_reason: Option<String>,
}

/// Token usage statistics
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    /// Prompt tokens used
    pub prompt_tokens: u32,
    /// Completion tokens generated
    pub completion_tokens: u32,
    /// Total tokens
    pub total_tokens: u32,
}

/// A chunk from streaming completion
#[derive(Debug, Clone)]
pub struct LlmChunk {
    /// Delta text content
    pub delta: String,
    /// Whether this is the final chunk
    pub is_final: bool,
}

/// Chat message for multi-turn conversations
#[derive(Debug, Clone)]
pub struct ChatMessage {
    /// Role (system, user, assistant)
    pub role: String,
    /// Message content
    pub content: String,
}

impl ChatMessage {
    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

/// LLM completion request
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    /// Messages for chat completion
    pub messages: Vec<ChatMessage>,
    /// Temperature (0.0-1.0)
    pub temperature: Option<f32>,
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Stop sequences
    pub stop: Option<Vec<String>>,
}

impl CompletionRequest {
    /// Create a new completion request with a single user prompt
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            messages: vec![ChatMessage::user(prompt)],
            temperature: None,
            max_tokens: None,
            stop: None,
        }
    }

    /// Create a request with system prompt and user message
    pub fn with_system(system: impl Into<String>, user: impl Into<String>) -> Self {
        Self {
            messages: vec![
                ChatMessage::system(system),
                ChatMessage::user(user),
            ],
            temperature: None,
            max_tokens: None,
            stop: None,
        }
    }

    /// Set temperature
    pub fn temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp.clamp(0.0, 2.0));
        self
    }

    /// Set max tokens
    pub fn max_tokens(mut self, max: u32) -> Self {
        self.max_tokens = Some(max);
        self
    }

    /// Add stop sequences
    pub fn stop(mut self, sequences: Vec<String>) -> Self {
        self.stop = Some(sequences);
        self
    }
}

/// Trait for LLM providers
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Get provider name
    fn name(&self) -> &str;

    /// Get model name
    fn model(&self) -> &str;

    /// Generate a completion (non-streaming)
    async fn complete(&self, request: &CompletionRequest) -> Result<LlmResponse>;

    /// Generate a streaming completion
    async fn complete_stream(
        &self,
        request: &CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<LlmChunk>> + Send>>>;
}

// =============================================================================
// Ollama Provider
// =============================================================================

#[cfg(feature = "ollama")]
pub mod ollama {
    use super::*;
    use ollama_rs::generation::completion::request::GenerationRequest;
    use ollama_rs::generation::chat::request::ChatMessageRequest;
    use ollama_rs::generation::chat::ChatMessage as OllamaMessage;
    use ollama_rs::Ollama;

    /// Ollama LLM provider for local model inference
    pub struct OllamaProvider {
        client: Ollama,
        model: String,
    }

    impl OllamaProvider {
        /// Create a new Ollama provider
        pub fn new(host: &str, model: &str) -> Result<Self> {
            // Parse host URL
            let url = url::Url::parse(host)
                .with_context(|| format!("Invalid Ollama host URL: {}", host))?;

            let host_str = url.host_str().unwrap_or("localhost");
            let port = url.port().unwrap_or(11434);

            let client = Ollama::new(format!("http://{}", host_str), port);

            Ok(Self {
                client,
                model: model.to_string(),
            })
        }

        /// Create from config
        pub fn from_config(config: &AiIngestionConfig) -> Result<Self> {
            let model = config.get_model().to_string();
            Self::new(&config.ollama.host, &model)
        }
    }

    #[async_trait]
    impl LlmProvider for OllamaProvider {
        fn name(&self) -> &str {
            "ollama"
        }

        fn model(&self) -> &str {
            &self.model
        }

        async fn complete(&self, request: &CompletionRequest) -> Result<LlmResponse> {
            // Convert messages to Ollama format
            let messages: Vec<OllamaMessage> = request
                .messages
                .iter()
                .map(|m| {
                    let role = match m.role.as_str() {
                        "system" => ollama_rs::generation::chat::MessageRole::System,
                        "user" => ollama_rs::generation::chat::MessageRole::User,
                        "assistant" => ollama_rs::generation::chat::MessageRole::Assistant,
                        _ => ollama_rs::generation::chat::MessageRole::User,
                    };
                    OllamaMessage::new(role, m.content.clone())
                })
                .collect();

            let mut chat_request = ChatMessageRequest::new(self.model.clone(), messages);

            // Apply options
            if let Some(temp) = request.temperature {
                let options = ollama_rs::generation::options::GenerationOptions::default()
                    .temperature(temp as f64);
                chat_request = chat_request.options(options);
            }

            let response = self.client.send_chat_messages(chat_request).await
                .context("Ollama chat request failed")?;

            let content = response.message.map(|m| m.content).unwrap_or_default();

            Ok(LlmResponse {
                content,
                model: self.model.clone(),
                usage: None, // Ollama doesn't provide token counts in basic response
                finish_reason: Some("stop".to_string()),
            })
        }

        async fn complete_stream(
            &self,
            request: &CompletionRequest,
        ) -> Result<Pin<Box<dyn Stream<Item = Result<LlmChunk>> + Send>>> {
            use futures_util::StreamExt;
            use tokio_stream::wrappers::ReceiverStream;

            let messages: Vec<OllamaMessage> = request
                .messages
                .iter()
                .map(|m| {
                    let role = match m.role.as_str() {
                        "system" => ollama_rs::generation::chat::MessageRole::System,
                        "user" => ollama_rs::generation::chat::MessageRole::User,
                        "assistant" => ollama_rs::generation::chat::MessageRole::Assistant,
                        _ => ollama_rs::generation::chat::MessageRole::User,
                    };
                    OllamaMessage::new(role, m.content.clone())
                })
                .collect();

            let mut chat_request = ChatMessageRequest::new(self.model.clone(), messages);

            if let Some(temp) = request.temperature {
                let options = ollama_rs::generation::options::GenerationOptions::default()
                    .temperature(temp as f64);
                chat_request = chat_request.options(options);
            }

            let (tx, rx) = tokio::sync::mpsc::channel::<Result<LlmChunk>>(100);

            // Clone for the async task
            let client = self.client.clone();

            tokio::spawn(async move {
                let mut stream = match client.send_chat_messages_stream(chat_request).await {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = tx.send(Err(anyhow::anyhow!("Stream error: {}", e))).await;
                        return;
                    }
                };

                while let Some(chunk_result) = stream.next().await {
                    match chunk_result {
                        Ok(chunk) => {
                            let content = chunk.message.map(|m| m.content).unwrap_or_default();
                            let is_final = chunk.done;

                            if tx.send(Ok(LlmChunk {
                                delta: content,
                                is_final,
                            })).await.is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(Err(anyhow::anyhow!("Chunk error: {}", e))).await;
                            break;
                        }
                    }
                }
            });

            Ok(Box::pin(ReceiverStream::new(rx)))
        }
    }
}

// =============================================================================
// OpenAI Provider
// =============================================================================

#[cfg(feature = "openai")]
pub mod openai {
    use super::*;
    use async_openai::{
        types::{
            ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
            ChatCompletionRequestUserMessage, ChatCompletionRequestAssistantMessage,
            CreateChatCompletionRequestArgs,
        },
        Client,
    };

    /// OpenAI LLM provider
    pub struct OpenAIProvider {
        client: Client<async_openai::config::OpenAIConfig>,
        model: String,
    }

    impl OpenAIProvider {
        /// Create a new OpenAI provider
        pub fn new(model: &str) -> Result<Self> {
            // Uses OPENAI_API_KEY from environment by default
            let client = Client::new();
            Ok(Self {
                client,
                model: model.to_string(),
            })
        }

        /// Create with custom API key
        pub fn with_api_key(api_key: &str, model: &str) -> Result<Self> {
            let config = async_openai::config::OpenAIConfig::new().with_api_key(api_key);
            let client = Client::with_config(config);
            Ok(Self {
                client,
                model: model.to_string(),
            })
        }

        /// Create from config
        pub fn from_config(config: &AiIngestionConfig) -> Result<Self> {
            let model = config.get_model().to_string();

            // Check for API key in environment
            if let Some(ref env_var) = config.openai.api_key_env {
                if let Ok(key) = std::env::var(env_var) {
                    return Self::with_api_key(&key, &model);
                }
            }

            // Fallback to default OPENAI_API_KEY
            Self::new(&model)
        }
    }

    #[async_trait]
    impl LlmProvider for OpenAIProvider {
        fn name(&self) -> &str {
            "openai"
        }

        fn model(&self) -> &str {
            &self.model
        }

        async fn complete(&self, request: &CompletionRequest) -> Result<LlmResponse> {
            let messages: Vec<ChatCompletionRequestMessage> = request
                .messages
                .iter()
                .map(|m| match m.role.as_str() {
                    "system" => ChatCompletionRequestMessage::System(
                        ChatCompletionRequestSystemMessage {
                            content: async_openai::types::ChatCompletionRequestSystemMessageContent::Text(m.content.clone()),
                            name: None,
                        }
                    ),
                    "assistant" => ChatCompletionRequestMessage::Assistant(
                        ChatCompletionRequestAssistantMessage {
                            content: Some(async_openai::types::ChatCompletionRequestAssistantMessageContent::Text(m.content.clone())),
                            name: None,
                            tool_calls: None,
                            refusal: None,
                            audio: None,
                        }
                    ),
                    _ => ChatCompletionRequestMessage::User(
                        ChatCompletionRequestUserMessage {
                            content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(m.content.clone()),
                            name: None,
                        }
                    ),
                })
                .collect();

            let mut builder = CreateChatCompletionRequestArgs::default();
            builder.model(&self.model).messages(messages);

            if let Some(temp) = request.temperature {
                builder.temperature(temp);
            }
            if let Some(max) = request.max_tokens {
                builder.max_completion_tokens(max);
            }
            if let Some(ref stop) = request.stop {
                builder.stop(stop.clone());
            }

            let req = builder.build()?;
            let response = self.client.chat().create(req).await?;

            let choice = response.choices.first()
                .context("No completion choices returned")?;

            let content = choice.message.content.clone().unwrap_or_default();

            let usage = response.usage.map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            });

            Ok(LlmResponse {
                content,
                model: response.model,
                usage,
                finish_reason: choice.finish_reason.as_ref().map(|r| format!("{:?}", r)),
            })
        }

        async fn complete_stream(
            &self,
            request: &CompletionRequest,
        ) -> Result<Pin<Box<dyn Stream<Item = Result<LlmChunk>> + Send>>> {
            use futures_util::StreamExt;
            use tokio_stream::wrappers::ReceiverStream;

            let messages: Vec<ChatCompletionRequestMessage> = request
                .messages
                .iter()
                .map(|m| match m.role.as_str() {
                    "system" => ChatCompletionRequestMessage::System(
                        ChatCompletionRequestSystemMessage {
                            content: async_openai::types::ChatCompletionRequestSystemMessageContent::Text(m.content.clone()),
                            name: None,
                        }
                    ),
                    "assistant" => ChatCompletionRequestMessage::Assistant(
                        ChatCompletionRequestAssistantMessage {
                            content: Some(async_openai::types::ChatCompletionRequestAssistantMessageContent::Text(m.content.clone())),
                            name: None,
                            tool_calls: None,
                            refusal: None,
                            audio: None,
                        }
                    ),
                    _ => ChatCompletionRequestMessage::User(
                        ChatCompletionRequestUserMessage {
                            content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(m.content.clone()),
                            name: None,
                        }
                    ),
                })
                .collect();

            let mut builder = CreateChatCompletionRequestArgs::default();
            builder.model(&self.model).messages(messages);

            if let Some(temp) = request.temperature {
                builder.temperature(temp);
            }
            if let Some(max) = request.max_tokens {
                builder.max_completion_tokens(max);
            }

            let req = builder.build()?;
            let (tx, rx) = tokio::sync::mpsc::channel::<Result<LlmChunk>>(100);

            let client = self.client.clone();

            tokio::spawn(async move {
                let mut stream = match client.chat().create_stream(req).await {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = tx.send(Err(anyhow::anyhow!("Stream error: {}", e))).await;
                        return;
                    }
                };

                while let Some(result) = stream.next().await {
                    match result {
                        Ok(response) => {
                            if let Some(choice) = response.choices.first() {
                                let delta = choice.delta.content.clone().unwrap_or_default();
                                let is_final = choice.finish_reason.is_some();

                                if tx.send(Ok(LlmChunk { delta, is_final })).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(Err(anyhow::anyhow!("Chunk error: {}", e))).await;
                            break;
                        }
                    }
                }
            });

            Ok(Box::pin(ReceiverStream::new(rx)))
        }
    }
}

// =============================================================================
// Provider Factory
// =============================================================================

use std::sync::Arc;

/// Create an LLM provider from configuration
pub fn create_llm_provider(config: &AiIngestionConfig) -> Result<Arc<dyn LlmProvider>> {
    match config.provider {
        #[cfg(feature = "ollama")]
        AiProvider::Ollama => {
            let provider = ollama::OllamaProvider::from_config(config)?;
            Ok(Arc::new(provider))
        }
        #[cfg(not(feature = "ollama"))]
        AiProvider::Ollama => {
            anyhow::bail!("Ollama support not enabled. Rebuild with --features ollama")
        }

        #[cfg(feature = "openai")]
        AiProvider::OpenAi => {
            let provider = openai::OpenAIProvider::from_config(config)?;
            Ok(Arc::new(provider))
        }
        #[cfg(not(feature = "openai"))]
        AiProvider::OpenAi => {
            anyhow::bail!("OpenAI support not enabled. Rebuild with --features openai")
        }

        AiProvider::Anthropic => {
            // Anthropic uses OpenAI-compatible API for most operations
            // For now, we'll return an error suggesting to use a different provider
            anyhow::bail!(
                "Anthropic provider not yet implemented. Use 'ollama' or 'openai' instead. \
                You can use Claude models through OpenRouter with the 'openai' provider."
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_creation() {
        let system = ChatMessage::system("You are a helpful assistant");
        assert_eq!(system.role, "system");

        let user = ChatMessage::user("Hello");
        assert_eq!(user.role, "user");

        let assistant = ChatMessage::assistant("Hi there!");
        assert_eq!(assistant.role, "assistant");
    }

    #[test]
    fn test_completion_request() {
        let req = CompletionRequest::new("Test prompt")
            .temperature(0.7)
            .max_tokens(1000)
            .stop(vec!["###".to_string()]);

        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.messages[0].role, "user");
        assert_eq!(req.temperature, Some(0.7));
        assert_eq!(req.max_tokens, Some(1000));
        assert!(req.stop.is_some());
    }

    #[test]
    fn test_completion_request_with_system() {
        let req = CompletionRequest::with_system(
            "You are a CLI expert",
            "How do I list files?"
        );

        assert_eq!(req.messages.len(), 2);
        assert_eq!(req.messages[0].role, "system");
        assert_eq!(req.messages[1].role, "user");
    }

    #[test]
    fn test_temperature_clamping() {
        let req = CompletionRequest::new("test").temperature(5.0);
        assert_eq!(req.temperature, Some(2.0));

        let req = CompletionRequest::new("test").temperature(-1.0);
        assert_eq!(req.temperature, Some(0.0));
    }
}
