//! Accuracy evaluation metrics for AI-generated examples
//!
//! Provides metrics for measuring the quality, accuracy, and diversity
//! of generated examples against tool schemas.

use std::collections::HashMap;
use crate::skill_md::ToolDocumentation;
use super::streaming::GeneratedExample;
use super::validator::ExampleValidator;

// =============================================================================
// Accuracy Metrics
// =============================================================================

/// Comprehensive accuracy metrics for a batch of generated examples
#[derive(Debug, Clone, Default)]
pub struct AccuracyMetrics {
    /// Total number of examples generated
    pub total_generated: usize,

    /// Number that passed schema validation
    pub schema_valid: usize,

    /// Number with all required parameters present
    pub required_params_present: usize,

    /// Number with correct parameter types
    pub type_correct: usize,

    /// Number with non-empty explanations
    pub has_explanation: usize,

    /// Diversity score (Jaccard-based, 0.0-1.0)
    pub diversity_score: f32,

    /// Per-tool breakdown
    pub per_tool: HashMap<String, ToolMetrics>,

    /// Validation errors by type
    pub error_breakdown: HashMap<String, usize>,
}

impl AccuracyMetrics {
    /// Create new empty metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate schema validation rate (0.0-1.0)
    pub fn validation_rate(&self) -> f32 {
        if self.total_generated == 0 {
            return 0.0;
        }
        self.schema_valid as f32 / self.total_generated as f32
    }

    /// Calculate required parameter compliance rate (0.0-1.0)
    pub fn param_compliance_rate(&self) -> f32 {
        if self.total_generated == 0 {
            return 0.0;
        }
        self.required_params_present as f32 / self.total_generated as f32
    }

    /// Calculate type correctness rate (0.0-1.0)
    pub fn type_correctness_rate(&self) -> f32 {
        if self.total_generated == 0 {
            return 0.0;
        }
        self.type_correct as f32 / self.total_generated as f32
    }

    /// Calculate explanation coverage rate (0.0-1.0)
    pub fn explanation_rate(&self) -> f32 {
        if self.total_generated == 0 {
            return 0.0;
        }
        self.has_explanation as f32 / self.total_generated as f32
    }

    /// Calculate overall quality score (weighted average, 0.0-1.0)
    pub fn overall_quality(&self) -> f32 {
        let weights = [
            (self.validation_rate(), 0.4),       // Schema validation is most important
            (self.param_compliance_rate(), 0.25), // Required params
            (self.type_correctness_rate(), 0.15), // Type correctness
            (self.explanation_rate(), 0.1),       // Has explanation
            (self.diversity_score, 0.1),          // Diversity
        ];

        weights.iter().map(|(rate, weight)| rate * weight).sum()
    }

    /// Check if metrics meet minimum quality threshold
    pub fn meets_threshold(&self, threshold: f32) -> bool {
        self.validation_rate() >= threshold
    }

    /// Add metrics for a tool
    pub fn add_tool_metrics(&mut self, tool_name: &str, metrics: ToolMetrics) {
        self.total_generated += metrics.total_generated;
        self.schema_valid += metrics.schema_valid;
        self.required_params_present += metrics.required_params_present;
        self.type_correct += metrics.type_correct;
        self.has_explanation += metrics.has_explanation;

        // Aggregate error breakdown
        for (error_type, count) in &metrics.error_breakdown {
            *self.error_breakdown.entry(error_type.clone()).or_insert(0) += count;
        }

        self.per_tool.insert(tool_name.to_string(), metrics);
    }

    /// Format as a summary string
    pub fn summary(&self) -> String {
        format!(
            "Accuracy Metrics:\n\
             - Total Generated: {}\n\
             - Schema Valid: {} ({:.1}%)\n\
             - Param Compliance: {:.1}%\n\
             - Type Correct: {:.1}%\n\
             - Has Explanation: {:.1}%\n\
             - Diversity: {:.2}\n\
             - Overall Quality: {:.2}",
            self.total_generated,
            self.schema_valid,
            self.validation_rate() * 100.0,
            self.param_compliance_rate() * 100.0,
            self.type_correctness_rate() * 100.0,
            self.explanation_rate() * 100.0,
            self.diversity_score,
            self.overall_quality()
        )
    }
}

/// Metrics for a single tool's generated examples
#[derive(Debug, Clone, Default)]
pub struct ToolMetrics {
    /// Tool name
    pub tool_name: String,

    /// Total examples generated for this tool
    pub total_generated: usize,

    /// Examples that passed validation
    pub schema_valid: usize,

    /// Examples with all required params
    pub required_params_present: usize,

    /// Examples with correct types
    pub type_correct: usize,

    /// Examples with non-empty explanations
    pub has_explanation: usize,

    /// Error types for this tool
    pub error_breakdown: HashMap<String, usize>,

    /// Average confidence score
    pub avg_confidence: f32,
}

impl ToolMetrics {
    /// Create new metrics for a tool
    pub fn new(tool_name: &str) -> Self {
        Self {
            tool_name: tool_name.to_string(),
            ..Default::default()
        }
    }

    /// Calculate validation rate
    pub fn validation_rate(&self) -> f32 {
        if self.total_generated == 0 {
            return 0.0;
        }
        self.schema_valid as f32 / self.total_generated as f32
    }

    /// Calculate type correctness rate
    pub fn type_correctness_rate(&self) -> f32 {
        if self.total_generated == 0 {
            return 0.0;
        }
        self.type_correct as f32 / self.total_generated as f32
    }

    /// Calculate required param compliance rate
    pub fn param_compliance_rate(&self) -> f32 {
        if self.total_generated == 0 {
            return 0.0;
        }
        self.required_params_present as f32 / self.total_generated as f32
    }
}

// =============================================================================
// Accuracy Evaluator
// =============================================================================

/// Evaluator for measuring accuracy of generated examples
pub struct AccuracyEvaluator {
    validator: ExampleValidator,
}

impl AccuracyEvaluator {
    /// Create a new evaluator
    pub fn new() -> Self {
        Self {
            validator: ExampleValidator::new(),
        }
    }

    /// Create with strict validation
    pub fn strict() -> Self {
        Self {
            validator: ExampleValidator::strict(),
        }
    }

    /// Evaluate a batch of examples for a single tool
    pub fn evaluate_tool(
        &self,
        tool: &ToolDocumentation,
        examples: &[GeneratedExample],
    ) -> ToolMetrics {
        let mut metrics = ToolMetrics::new(&tool.name);
        metrics.total_generated = examples.len();

        let mut total_confidence = 0.0;

        for example in examples {
            // Check for explanation
            if !example.explanation.trim().is_empty() {
                metrics.has_explanation += 1;
            }

            // Validate example
            let validation = self.validator.validate_example(example, tool);

            if validation.valid {
                metrics.schema_valid += 1;
            }

            // Check required params (more specific than full validation)
            let parsed = self.validator.parse_command(&example.command);
            if let Ok(parsed) = parsed {
                let has_all_required = tool.parameters.iter()
                    .filter(|p| p.required)
                    .all(|p| parsed.has_param(&p.name));

                if has_all_required {
                    metrics.required_params_present += 1;
                }

                // Type checking would require running validation on each param
                // For now, count valid examples as type-correct
                if validation.valid {
                    metrics.type_correct += 1;
                }
            }

            // Track errors
            for error in &validation.errors {
                let error_type = categorize_error(error);
                *metrics.error_breakdown.entry(error_type).or_insert(0) += 1;
            }

            total_confidence += example.confidence;
        }

        if !examples.is_empty() {
            metrics.avg_confidence = total_confidence / examples.len() as f32;
        }

        metrics
    }

    /// Evaluate examples for multiple tools
    pub fn evaluate_batch(
        &self,
        tools: &[ToolDocumentation],
        examples_by_tool: &HashMap<String, Vec<GeneratedExample>>,
    ) -> AccuracyMetrics {
        let mut metrics = AccuracyMetrics::new();

        for tool in tools {
            if let Some(examples) = examples_by_tool.get(&tool.name) {
                let tool_metrics = self.evaluate_tool(tool, examples);
                metrics.add_tool_metrics(&tool.name, tool_metrics);
            }
        }

        // Calculate diversity across all examples
        let all_examples: Vec<_> = examples_by_tool.values()
            .flat_map(|v| v.iter())
            .cloned()
            .collect();
        metrics.diversity_score = self.validator.calculate_diversity(&all_examples);

        metrics
    }

    /// Evaluate a single tool and return pass/fail with detailed results
    pub fn evaluate_with_threshold(
        &self,
        tool: &ToolDocumentation,
        examples: &[GeneratedExample],
        threshold: f32,
    ) -> (bool, ToolMetrics) {
        let metrics = self.evaluate_tool(tool, examples);
        let passes = metrics.validation_rate() >= threshold;
        (passes, metrics)
    }
}

impl Default for AccuracyEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Categorize an error message into a type
fn categorize_error(error: &str) -> String {
    let lower = error.to_lowercase();
    if lower.contains("required") || lower.contains("missing") {
        "missing_required".to_string()
    } else if lower.contains("type") || lower.contains("expected") {
        "type_mismatch".to_string()
    } else if lower.contains("parse") {
        "parse_error".to_string()
    } else if lower.contains("explanation") {
        "empty_explanation".to_string()
    } else {
        "other".to_string()
    }
}

// =============================================================================
// Performance Metrics
// =============================================================================

/// Performance metrics for generation
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    /// Total time for all generation (ms)
    pub total_time_ms: u64,

    /// Time per tool (ms)
    pub per_tool_time_ms: HashMap<String, u64>,

    /// Time to first event (ms)
    pub time_to_first_event_ms: Option<u64>,

    /// Events per second
    pub events_per_second: f32,

    /// Total events emitted
    pub total_events: usize,
}

impl PerformanceMetrics {
    /// Create new empty metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate average time per tool
    pub fn avg_time_per_tool(&self) -> u64 {
        if self.per_tool_time_ms.is_empty() {
            return 0;
        }
        let total: u64 = self.per_tool_time_ms.values().sum();
        total / self.per_tool_time_ms.len() as u64
    }

    /// Check if meets latency threshold
    pub fn meets_latency_threshold(&self, max_ms_per_tool: u64) -> bool {
        self.per_tool_time_ms.values().all(|&t| t <= max_ms_per_tool)
    }

    /// Format as summary string
    pub fn summary(&self) -> String {
        format!(
            "Performance Metrics:\n\
             - Total Time: {}ms\n\
             - Avg per Tool: {}ms\n\
             - Time to First Event: {:?}ms\n\
             - Events/sec: {:.1}\n\
             - Total Events: {}",
            self.total_time_ms,
            self.avg_time_per_tool(),
            self.time_to_first_event_ms,
            self.events_per_second,
            self.total_events
        )
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::fixtures::*;

    #[test]
    fn test_accuracy_metrics_calculation() {
        let mut metrics = AccuracyMetrics::new();
        metrics.total_generated = 10;
        metrics.schema_valid = 9;
        metrics.required_params_present = 10;
        metrics.type_correct = 8;
        metrics.has_explanation = 10;
        metrics.diversity_score = 0.75;

        assert!((metrics.validation_rate() - 0.9).abs() < 0.01);
        assert!((metrics.param_compliance_rate() - 1.0).abs() < 0.01);
        assert!((metrics.type_correctness_rate() - 0.8).abs() < 0.01);
        assert!(metrics.overall_quality() > 0.8);
    }

    #[test]
    fn test_empty_metrics() {
        let metrics = AccuracyMetrics::new();
        assert_eq!(metrics.validation_rate(), 0.0);
        assert_eq!(metrics.param_compliance_rate(), 0.0);
        assert_eq!(metrics.overall_quality(), 0.0);
    }

    #[test]
    fn test_meets_threshold() {
        let mut metrics = AccuracyMetrics::new();
        metrics.total_generated = 100;
        metrics.schema_valid = 95;

        assert!(metrics.meets_threshold(0.95));
        assert!(!metrics.meets_threshold(0.96));
    }

    #[test]
    fn test_tool_metrics() {
        let mut metrics = ToolMetrics::new("apply");
        metrics.total_generated = 5;
        metrics.schema_valid = 4;

        assert_eq!(metrics.tool_name, "apply");
        assert!((metrics.validation_rate() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_evaluator_with_valid_examples() {
        let evaluator = AccuracyEvaluator::new();
        let tool = kubernetes_apply_tool();

        let examples = vec![
            GeneratedExample::new(
                "skill run kubernetes:apply --file=deploy.yaml",
                "Apply deployment manifest"
            ).with_confidence(0.9),
            GeneratedExample::new(
                "skill run kubernetes:apply --file=service.yaml --namespace=prod",
                "Apply to production"
            ).with_confidence(0.85),
        ];

        let metrics = evaluator.evaluate_tool(&tool, &examples);

        assert_eq!(metrics.total_generated, 2);
        assert!(metrics.validation_rate() > 0.0);
        assert!(metrics.has_explanation > 0);
    }

    #[test]
    fn test_evaluator_with_invalid_examples() {
        let evaluator = AccuracyEvaluator::new();
        let tool = kubernetes_apply_tool();

        let examples = vec![
            // Missing required 'file' parameter
            GeneratedExample::new(
                "skill run kubernetes:apply --namespace=prod",
                "Missing file param"
            ),
            // Empty explanation
            GeneratedExample::new(
                "skill run kubernetes:apply --file=test.yaml",
                ""
            ),
        ];

        let metrics = evaluator.evaluate_tool(&tool, &examples);

        assert_eq!(metrics.total_generated, 2);
        // Both should fail - one missing required param, one empty explanation
        assert!(metrics.schema_valid < 2);
        assert_eq!(metrics.has_explanation, 1); // Only first has explanation
    }

    #[test]
    fn test_error_categorization() {
        assert_eq!(categorize_error("Missing required parameter: file"), "missing_required");
        assert_eq!(categorize_error("expected integer, got 'abc'"), "type_mismatch");
        assert_eq!(categorize_error("Failed to parse command"), "parse_error");
        assert_eq!(categorize_error("explanation is empty"), "empty_explanation");
        assert_eq!(categorize_error("unknown error"), "other");
    }

    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::new();
        metrics.total_time_ms = 5000;
        metrics.per_tool_time_ms.insert("apply".to_string(), 1000);
        metrics.per_tool_time_ms.insert("get".to_string(), 2000);
        metrics.total_events = 50;
        metrics.events_per_second = 10.0;

        assert_eq!(metrics.avg_time_per_tool(), 1500);
        assert!(metrics.meets_latency_threshold(2000));
        assert!(!metrics.meets_latency_threshold(1500));
    }

    #[test]
    fn test_batch_evaluation() {
        let evaluator = AccuracyEvaluator::new();

        let tools = vec![
            kubernetes_apply_tool(),
            simple_tool(),
        ];

        let mut examples_by_tool = HashMap::new();
        examples_by_tool.insert(
            "apply".to_string(),
            vec![GeneratedExample::new("skill run kubernetes:apply --file=test.yaml", "Test")],
        );
        examples_by_tool.insert(
            "list".to_string(),
            vec![GeneratedExample::new("skill run tool:list --type=pods", "List pods")],
        );

        let metrics = evaluator.evaluate_batch(&tools, &examples_by_tool);

        assert_eq!(metrics.total_generated, 2);
        assert_eq!(metrics.per_tool.len(), 2);
        assert!(metrics.diversity_score > 0.0);
    }
}
