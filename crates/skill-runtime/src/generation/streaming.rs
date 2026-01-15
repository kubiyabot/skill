//! SSE streaming event types for AI generation
//!
//! Provides typed events for real-time streaming of generation progress,
//! compatible with Server-Sent Events (SSE) protocol.

use serde::{Deserialize, Serialize};
use std::time::Duration;

// =============================================================================
// Generation Events
// =============================================================================

/// Events emitted during AI-powered example generation
///
/// These events follow the SSE (Server-Sent Events) format and can be
/// streamed to CLI or MCP clients in real-time.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GenerationEvent {
    /// Generation process has started for a tool
    Started {
        /// Name of the tool being processed
        tool_name: String,
        /// Total number of tools to process
        total_tools: usize,
        /// Current tool index (1-based)
        current_index: usize,
    },

    /// LLM is processing/thinking
    Thinking {
        /// Current thought/reasoning step
        thought: String,
    },

    /// Performing an intermediate search
    Searching {
        /// Search query being executed
        query: String,
    },

    /// Search results received
    SearchResult {
        /// Tool names found
        tools: Vec<String>,
        /// Number of results
        count: usize,
    },

    /// An example has been generated
    Example {
        /// The generated example
        example: GeneratedExample,
    },

    /// Validation result for an example
    Validation {
        /// Whether the example passed validation
        valid: bool,
        /// Validation errors (if any)
        errors: Vec<String>,
        /// Index of the example being validated
        example_index: usize,
    },

    /// Progress update
    Progress {
        /// Current item being processed
        current: usize,
        /// Total items to process
        total: usize,
        /// Completion percentage (0.0 - 100.0)
        percent: f32,
        /// Optional message
        message: Option<String>,
    },

    /// Tool generation completed
    ToolCompleted {
        /// Tool name
        tool_name: String,
        /// Number of examples generated
        examples_generated: usize,
        /// Number of valid examples
        valid_examples: usize,
        /// Duration in milliseconds
        duration_ms: u64,
    },

    /// All generation completed
    Completed {
        /// Total examples generated across all tools
        total_examples: usize,
        /// Total valid examples
        total_valid: usize,
        /// Total tools processed
        total_tools: usize,
        /// Total duration in milliseconds
        duration_ms: u64,
    },

    /// An error occurred
    Error {
        /// Error message
        message: String,
        /// Whether the error is recoverable
        recoverable: bool,
        /// Optional tool name context
        tool_name: Option<String>,
    },

    /// Agent reasoning step (for self-ask patterns)
    AgentStep {
        /// The reasoning step
        step: AgentStep,
    },
}

impl GenerationEvent {
    /// Create a started event
    pub fn started(tool_name: impl Into<String>, total_tools: usize, current_index: usize) -> Self {
        Self::Started {
            tool_name: tool_name.into(),
            total_tools,
            current_index,
        }
    }

    /// Create a thinking event
    pub fn thinking(thought: impl Into<String>) -> Self {
        Self::Thinking {
            thought: thought.into(),
        }
    }

    /// Create a progress event
    pub fn progress(current: usize, total: usize, message: Option<String>) -> Self {
        let percent = if total > 0 {
            (current as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        Self::Progress {
            current,
            total,
            percent,
            message,
        }
    }

    /// Create an error event
    pub fn error(message: impl Into<String>, recoverable: bool) -> Self {
        Self::Error {
            message: message.into(),
            recoverable,
            tool_name: None,
        }
    }

    /// Create an error event with tool context
    pub fn tool_error(message: impl Into<String>, tool_name: impl Into<String>, recoverable: bool) -> Self {
        Self::Error {
            message: message.into(),
            recoverable,
            tool_name: Some(tool_name.into()),
        }
    }

    /// Create a completed event
    pub fn completed(total_examples: usize, total_valid: usize, total_tools: usize, duration: Duration) -> Self {
        Self::Completed {
            total_examples,
            total_valid,
            total_tools,
            duration_ms: duration.as_millis() as u64,
        }
    }

    /// Format as SSE data line
    pub fn to_sse_data(&self) -> String {
        format!("data: {}\n\n", serde_json::to_string(self).unwrap_or_default())
    }

    /// Format as SSE with event type
    pub fn to_sse(&self) -> String {
        let event_type = match self {
            Self::Started { .. } => "started",
            Self::Thinking { .. } => "thinking",
            Self::Searching { .. } => "searching",
            Self::SearchResult { .. } => "search_result",
            Self::Example { .. } => "example",
            Self::Validation { .. } => "validation",
            Self::Progress { .. } => "progress",
            Self::ToolCompleted { .. } => "tool_completed",
            Self::Completed { .. } => "completed",
            Self::Error { .. } => "error",
            Self::AgentStep { .. } => "agent_step",
        };
        format!(
            "event: {}\ndata: {}\n\n",
            event_type,
            serde_json::to_string(self).unwrap_or_default()
        )
    }
}

// =============================================================================
// Generated Example
// =============================================================================

/// A generated usage example for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedExample {
    /// Full command string (e.g., "skill run k8s:apply --file=deploy.yaml")
    pub command: String,

    /// Human-readable explanation of what the command does
    pub explanation: String,

    /// Model confidence score (0.0 - 1.0)
    #[serde(default)]
    pub confidence: f32,

    /// Whether the example passed schema validation
    #[serde(default)]
    pub validated: bool,

    /// Optional use case category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Parameter values used in this example
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

impl GeneratedExample {
    /// Create a new generated example
    pub fn new(command: impl Into<String>, explanation: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            explanation: explanation.into(),
            confidence: 0.0,
            validated: false,
            category: None,
            parameters: None,
        }
    }

    /// Set confidence score
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Mark as validated
    pub fn with_validated(mut self, validated: bool) -> Self {
        self.validated = validated;
        self
    }

    /// Set category
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }
}

// =============================================================================
// Agent Reasoning Steps
// =============================================================================

/// A step in the agent's reasoning process (self-ask-with-search pattern)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    /// Step number
    pub step_number: usize,

    /// The agent's current thought
    pub thought: String,

    /// Optional follow-up question
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_up_question: Option<String>,

    /// Search results for this step
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_results: Option<Vec<SearchResultRef>>,

    /// Whether this is the final answer step
    pub is_final: bool,

    /// Final answer (if is_final = true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_answer: Option<String>,
}

impl AgentStep {
    /// Create a thinking step
    pub fn thinking(step_number: usize, thought: impl Into<String>) -> Self {
        Self {
            step_number,
            thought: thought.into(),
            follow_up_question: None,
            search_results: None,
            is_final: false,
            final_answer: None,
        }
    }

    /// Create a follow-up question step
    pub fn follow_up(step_number: usize, thought: impl Into<String>, question: impl Into<String>) -> Self {
        Self {
            step_number,
            thought: thought.into(),
            follow_up_question: Some(question.into()),
            search_results: None,
            is_final: false,
            final_answer: None,
        }
    }

    /// Create a final answer step
    pub fn final_answer(step_number: usize, thought: impl Into<String>, answer: impl Into<String>) -> Self {
        Self {
            step_number,
            thought: thought.into(),
            follow_up_question: None,
            search_results: None,
            is_final: true,
            final_answer: Some(answer.into()),
        }
    }

    /// Add search results to this step
    pub fn with_search_results(mut self, results: Vec<SearchResultRef>) -> Self {
        self.search_results = Some(results);
        self
    }
}

/// A reference to a search result (lightweight for streaming)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultRef {
    /// Tool name (e.g., "kubernetes:apply")
    pub tool_name: String,
    /// Brief description
    pub description: String,
    /// Relevance score
    pub score: f32,
}

// =============================================================================
// Stream Builder
// =============================================================================

/// Builder for creating generation event streams
pub struct GenerationStreamBuilder {
    tool_name: String,
    total_tools: usize,
    current_index: usize,
}

impl GenerationStreamBuilder {
    /// Create a new stream builder
    pub fn new(tool_name: impl Into<String>, total_tools: usize, current_index: usize) -> Self {
        Self {
            tool_name: tool_name.into(),
            total_tools,
            current_index,
        }
    }

    /// Get the started event
    pub fn started(&self) -> GenerationEvent {
        GenerationEvent::started(&self.tool_name, self.total_tools, self.current_index)
    }

    /// Create a thinking event
    pub fn thinking(&self, thought: impl Into<String>) -> GenerationEvent {
        GenerationEvent::thinking(thought)
    }

    /// Create an example event
    pub fn example(&self, example: GeneratedExample) -> GenerationEvent {
        GenerationEvent::Example { example }
    }

    /// Create a validation event
    pub fn validation(&self, valid: bool, errors: Vec<String>, example_index: usize) -> GenerationEvent {
        GenerationEvent::Validation {
            valid,
            errors,
            example_index,
        }
    }

    /// Create a tool completed event
    pub fn tool_completed(&self, examples_generated: usize, valid_examples: usize, duration: Duration) -> GenerationEvent {
        GenerationEvent::ToolCompleted {
            tool_name: self.tool_name.clone(),
            examples_generated,
            valid_examples,
            duration_ms: duration.as_millis() as u64,
        }
    }

    /// Create an error event
    pub fn error(&self, message: impl Into<String>, recoverable: bool) -> GenerationEvent {
        GenerationEvent::tool_error(message, &self.tool_name, recoverable)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_event_serialization() {
        let event = GenerationEvent::started("kubernetes:apply", 10, 1);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"started\""));
        assert!(json.contains("\"tool_name\":\"kubernetes:apply\""));
        assert!(json.contains("\"total_tools\":10"));

        // Deserialize back
        let parsed: GenerationEvent = serde_json::from_str(&json).unwrap();
        if let GenerationEvent::Started { tool_name, total_tools, current_index } = parsed {
            assert_eq!(tool_name, "kubernetes:apply");
            assert_eq!(total_tools, 10);
            assert_eq!(current_index, 1);
        } else {
            panic!("Expected Started event");
        }
    }

    #[test]
    fn test_thinking_event() {
        let event = GenerationEvent::thinking("Analyzing parameter schema...");
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"thinking\""));
        assert!(json.contains("Analyzing parameter schema"));
    }

    #[test]
    fn test_progress_event() {
        let event = GenerationEvent::progress(5, 10, Some("Processing tools".to_string()));
        if let GenerationEvent::Progress { current, total, percent, message } = event {
            assert_eq!(current, 5);
            assert_eq!(total, 10);
            assert!((percent - 50.0).abs() < 0.01);
            assert_eq!(message, Some("Processing tools".to_string()));
        } else {
            panic!("Expected Progress event");
        }
    }

    #[test]
    fn test_error_event() {
        let event = GenerationEvent::tool_error("Connection timeout", "k8s:apply", true);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"error\""));
        assert!(json.contains("\"recoverable\":true"));
        assert!(json.contains("\"tool_name\":\"k8s:apply\""));
    }

    #[test]
    fn test_generated_example() {
        let example = GeneratedExample::new(
            "skill run k8s:apply --file=deploy.yaml",
            "Apply a Kubernetes deployment manifest"
        )
        .with_confidence(0.95)
        .with_validated(true)
        .with_category("deployment");

        assert_eq!(example.command, "skill run k8s:apply --file=deploy.yaml");
        assert!((example.confidence - 0.95).abs() < 0.01);
        assert!(example.validated);
        assert_eq!(example.category, Some("deployment".to_string()));
    }

    #[test]
    fn test_agent_step() {
        let step = AgentStep::follow_up(
            1,
            "I need to find tools for container deployment",
            "What tools handle Kubernetes deployments?"
        );

        assert_eq!(step.step_number, 1);
        assert!(!step.is_final);
        assert!(step.follow_up_question.is_some());
        assert!(step.final_answer.is_none());

        let final_step = AgentStep::final_answer(
            3,
            "Based on my search, I recommend using kubernetes:apply",
            "Use kubernetes:apply with --file flag to deploy your manifest"
        );

        assert!(final_step.is_final);
        assert!(final_step.final_answer.is_some());
    }

    #[test]
    fn test_sse_format() {
        let event = GenerationEvent::thinking("Processing...");
        let sse = event.to_sse();

        assert!(sse.starts_with("event: thinking\n"));
        assert!(sse.contains("data: "));
        assert!(sse.ends_with("\n\n"));
    }

    #[test]
    fn test_stream_builder() {
        let builder = GenerationStreamBuilder::new("docker:build", 5, 2);

        let started = builder.started();
        if let GenerationEvent::Started { tool_name, total_tools, current_index } = started {
            assert_eq!(tool_name, "docker:build");
            assert_eq!(total_tools, 5);
            assert_eq!(current_index, 2);
        }

        let example = GeneratedExample::new("skill run docker:build .", "Build Docker image");
        let event = builder.example(example);
        assert!(matches!(event, GenerationEvent::Example { .. }));
    }

    #[test]
    fn test_completed_event() {
        let event = GenerationEvent::completed(50, 45, 10, Duration::from_secs(30));
        if let GenerationEvent::Completed { total_examples, total_valid, total_tools, duration_ms } = event {
            assert_eq!(total_examples, 50);
            assert_eq!(total_valid, 45);
            assert_eq!(total_tools, 10);
            assert_eq!(duration_ms, 30000);
        } else {
            panic!("Expected Completed event");
        }
    }
}
