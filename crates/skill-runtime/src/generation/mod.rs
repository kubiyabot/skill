//! AI-powered example generation for skill tools
//!
//! This module provides infrastructure for generating synthetic usage examples
//! using LLMs, with real-time streaming feedback via SSE events.

mod streaming;
mod llm_provider;
mod validator;
mod example_generator;
mod evaluation;
mod fixtures;

#[cfg(test)]
mod integration_tests;

pub use streaming::*;
pub use evaluation::*;
pub use fixtures::*;

// Re-export commonly used types
pub use streaming::{GenerationEvent, GeneratedExample, AgentStep};
pub use llm_provider::{
    LlmProvider, LlmResponse, LlmChunk, TokenUsage,
    ChatMessage, CompletionRequest, create_llm_provider,
};
pub use validator::{ExampleValidator, ValidationResult, ParsedCommand};
pub use example_generator::{ExampleGenerator, GeneratorConfig};

#[cfg(feature = "ollama")]
pub use llm_provider::ollama::OllamaProvider;

#[cfg(feature = "openai")]
pub use llm_provider::openai::OpenAIProvider;
