//! Integration tests for AI generation pipeline
//!
//! Tests the full flow from tool documentation through AI generation,
//! validation, and streaming events.

#![cfg(test)]

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio_stream::StreamExt;

use super::fixtures::*;
use super::evaluation::*;
use super::streaming::*;
use super::validator::*;
use super::example_generator::*;
use super::llm_provider::*;
use crate::skill_md::ToolDocumentation;

// =============================================================================
// Full Pipeline Tests
// =============================================================================

#[tokio::test]
async fn test_full_pipeline_with_mock_llm() {
    // Setup
    let provider = Arc::new(DeterministicMockProvider::new());
    let config = GeneratorConfig::default();
    let generator = ExampleGenerator::new(provider.clone(), config);
    let tool = kubernetes_apply_tool();

    // Generate examples
    let examples = generator.generate(&tool).await.unwrap();

    // Verify examples were generated
    assert!(!examples.is_empty(), "Should generate at least one example");
    assert!(examples.len() >= 3, "Should generate multiple examples");

    // Verify example structure
    for example in &examples {
        assert!(!example.command.is_empty(), "Command should not be empty");
        assert!(!example.explanation.is_empty(), "Explanation should not be empty");
        assert!(example.command.contains("apply"), "Command should contain tool name");
    }

    // Verify provider was called
    assert_eq!(provider.call_count(), 1, "Provider should be called once");
}

#[tokio::test]
async fn test_streaming_events_order() {
    // Setup
    let provider = Arc::new(DeterministicMockProvider::new());
    let config = GeneratorConfig::default();
    let generator = ExampleGenerator::new(provider, config);
    let tool = kubernetes_apply_tool();

    // Collect all events from stream
    let stream = generator.generate_stream(&tool, 1, 1);
    let events: Vec<GenerationEvent> = stream.collect().await;

    // Verify event order
    assert!(!events.is_empty(), "Should emit events");

    // First event should be Started
    assert!(
        matches!(&events[0], GenerationEvent::Started { .. }),
        "First event should be Started, got {:?}",
        &events[0]
    );

    // Should have Thinking events
    let thinking_count = events.iter()
        .filter(|e| matches!(e, GenerationEvent::Thinking { .. }))
        .count();
    assert!(thinking_count > 0, "Should have Thinking events");

    // Should have Example events
    let example_count = events.iter()
        .filter(|e| matches!(e, GenerationEvent::Example { .. }))
        .count();
    assert!(example_count > 0, "Should have Example events");

    // Last substantive event should be ToolCompleted
    let tool_completed = events.iter()
        .rfind(|e| matches!(e, GenerationEvent::ToolCompleted { .. }));
    assert!(tool_completed.is_some(), "Should have ToolCompleted event");
}

#[tokio::test]
async fn test_batch_generation() {
    // Setup
    let provider = Arc::new(DeterministicMockProvider::new());
    let config = GeneratorConfig::default();
    let generator = ExampleGenerator::new(provider.clone(), config);

    let tools = vec![
        kubernetes_apply_tool(),
        simple_tool(),
        docker_build_tool(),
    ];

    // Generate for batch
    let results = generator.generate_batch(&tools).await.unwrap();

    // Verify results
    assert_eq!(results.len(), 3, "Should have results for all tools");

    for (tool_name, examples) in &results {
        assert!(!examples.is_empty(), "Tool {} should have examples", tool_name);
    }

    // Provider should be called once per tool
    assert_eq!(provider.call_count(), 3, "Provider should be called once per tool");
}

#[tokio::test]
async fn test_validation_integration() {
    // Setup - use provider that returns some invalid examples
    let provider = Arc::new(DeterministicMockProvider::with_validation_errors());
    let config = GeneratorConfig {
        validate_examples: true,
        ..Default::default()
    };
    let generator = ExampleGenerator::new(provider, config);
    let tool = kubernetes_apply_tool();

    // Collect events to check validation
    let stream = generator.generate_stream(&tool, 1, 1);
    let events: Vec<GenerationEvent> = stream.collect().await;

    // Count validation events
    let validation_events: Vec<_> = events.iter()
        .filter_map(|e| {
            if let GenerationEvent::Validation { valid, errors, example_index } = e {
                Some((valid, errors, example_index))
            } else {
                None
            }
        })
        .collect();

    assert!(!validation_events.is_empty(), "Should have validation events");

    // Should have some valid and some invalid
    let valid_count = validation_events.iter().filter(|(v, _, _)| **v).count();
    let invalid_count = validation_events.iter().filter(|(v, _, _)| !**v).count();

    assert!(valid_count > 0, "Should have some valid examples");
    assert!(invalid_count > 0, "Should have some invalid examples (mock returns mixed results)");
}

#[tokio::test]
async fn test_error_handling() {
    // Setup - use failing provider
    let provider = Arc::new(FailingMockProvider::new("Simulated LLM error"));
    let config = GeneratorConfig {
        max_retries: 1,
        ..Default::default()
    };
    let generator = ExampleGenerator::new(provider, config);
    let tool = simple_tool();

    // Collect events
    let stream = generator.generate_stream(&tool, 1, 1);
    let events: Vec<GenerationEvent> = stream.collect().await;

    // Should have an error event
    let has_error = events.iter().any(|e| matches!(e, GenerationEvent::Error { .. }));
    assert!(has_error, "Should emit error event on failure");
}

// =============================================================================
// Accuracy Tests
// =============================================================================

#[tokio::test]
async fn test_schema_compliance() {
    let provider = Arc::new(DeterministicMockProvider::new());
    let config = GeneratorConfig::default();
    let generator = ExampleGenerator::new(provider, config);
    let evaluator = AccuracyEvaluator::new();

    // Test with multiple tools
    let tools = vec![
        kubernetes_apply_tool(),
        tool_with_constraints(),
        aws_s3_tool(),
    ];

    for tool in &tools {
        let examples = generator.generate(tool).await.unwrap();
        let metrics = evaluator.evaluate_tool(tool, &examples);

        assert!(
            metrics.validation_rate() >= 0.8, // Allow some flexibility with mocks
            "Tool {} should have >=80% validation rate, got {:.1}%",
            tool.name,
            metrics.validation_rate() * 100.0
        );
    }
}

#[tokio::test]
async fn test_required_parameter_coverage() {
    let provider = Arc::new(DeterministicMockProvider::new());
    let config = GeneratorConfig::default();
    let generator = ExampleGenerator::new(provider, config);
    let validator = ExampleValidator::new();
    let tool = kubernetes_apply_tool();

    let examples = generator.generate(&tool).await.unwrap();

    // Check each example for required params
    let required_params: Vec<_> = tool.parameters.iter()
        .filter(|p| p.required)
        .map(|p| &p.name)
        .collect();

    let mut examples_with_all_required = 0;

    for example in &examples {
        if let Ok(parsed) = validator.parse_command(&example.command) {
            let has_all = required_params.iter().all(|p| parsed.has_param(p));
            if has_all {
                examples_with_all_required += 1;
            }
        }
    }

    let coverage_rate = examples_with_all_required as f32 / examples.len() as f32;
    assert!(
        coverage_rate >= 0.8,
        "At least 80% of examples should have all required parameters, got {:.1}%",
        coverage_rate * 100.0
    );
}

#[tokio::test]
async fn test_parameter_type_correctness() {
    let provider = Arc::new(DeterministicMockProvider::new());
    let config = GeneratorConfig::default();
    let generator = ExampleGenerator::new(provider, config);
    let evaluator = AccuracyEvaluator::new();

    // Use a tool with typed parameters
    let tool = kubernetes_apply_tool();
    let examples = generator.generate(&tool).await.unwrap();
    let metrics = evaluator.evaluate_tool(&tool, &examples);

    // Type correctness should be high for validated examples
    assert!(
        metrics.type_correctness_rate() >= 0.7,
        "Type correctness should be >= 70%, got {:.1}%",
        metrics.type_correctness_rate() * 100.0
    );
}

#[tokio::test]
async fn test_example_diversity() {
    let provider = Arc::new(DeterministicMockProvider::new());
    let config = GeneratorConfig {
        examples_per_tool: 5, // Generate more for diversity testing
        ..Default::default()
    };
    let generator = ExampleGenerator::new(provider, config);
    let validator = ExampleValidator::new();
    let tool = kubernetes_apply_tool();

    let examples = generator.generate(&tool).await.unwrap();

    // Calculate diversity
    let diversity = validator.calculate_diversity(&examples);

    assert!(
        diversity >= 0.5,
        "Diversity score should be >= 0.5, got {:.2}",
        diversity
    );

    // Check for duplicate commands
    let commands: std::collections::HashSet<_> = examples.iter()
        .map(|e| &e.command)
        .collect();
    assert_eq!(
        commands.len(),
        examples.len(),
        "Should have no duplicate commands"
    );
}

#[tokio::test]
async fn test_explanation_quality() {
    let provider = Arc::new(DeterministicMockProvider::new());
    let config = GeneratorConfig::default();
    let generator = ExampleGenerator::new(provider, config);
    let tool = kubernetes_apply_tool();

    let examples = generator.generate(&tool).await.unwrap();

    // Check explanation quality
    let mut good_explanations = 0;
    for example in &examples {
        let explanation = &example.explanation;
        // Good explanation is non-empty and descriptive
        if !explanation.trim().is_empty() && explanation.len() > 10 {
            good_explanations += 1;
        }
    }

    let quality_rate = good_explanations as f32 / examples.len() as f32;
    assert!(
        quality_rate >= 0.8,
        "At least 80% should have quality explanations, got {:.1}%",
        quality_rate * 100.0
    );
}

// =============================================================================
// Performance Benchmark Tests
// =============================================================================

#[tokio::test]
async fn bench_generation_latency() {
    let provider = Arc::new(DeterministicMockProvider::new());
    let config = GeneratorConfig::default();
    let generator = ExampleGenerator::new(provider, config);
    let tool = kubernetes_apply_tool();

    let start = Instant::now();
    let _examples = generator.generate(&tool).await.unwrap();
    let duration = start.elapsed();

    println!("Generation latency: {:?}", duration);

    // With mock provider, should be very fast
    assert!(
        duration < Duration::from_secs(1),
        "Generation should complete in < 1s with mock, took {:?}",
        duration
    );
}

#[tokio::test]
async fn bench_streaming_throughput() {
    let provider = Arc::new(DeterministicMockProvider::new());
    let config = GeneratorConfig::default();
    let generator = ExampleGenerator::new(provider, config);
    let tool = kubernetes_apply_tool();

    let start = Instant::now();
    let stream = generator.generate_stream(&tool, 1, 1);
    let events: Vec<GenerationEvent> = stream.collect().await;
    let duration = start.elapsed();

    let events_per_sec = events.len() as f32 / duration.as_secs_f32();

    println!("Streaming throughput: {} events in {:?} ({:.1} events/sec)",
        events.len(), duration, events_per_sec);

    // Should emit multiple events quickly
    assert!(events.len() >= 5, "Should emit at least 5 events");
    assert!(events_per_sec > 10.0, "Should emit > 10 events/sec with mock");
}

#[tokio::test]
async fn bench_batch_throughput() {
    let provider = Arc::new(DeterministicMockProvider::new());
    let config = GeneratorConfig::default();
    let generator = ExampleGenerator::new(provider, config);

    let tools = vec![
        kubernetes_apply_tool(),
        simple_tool(),
        tool_with_constraints(),
        aws_s3_tool(),
        docker_build_tool(),
    ];

    let start = Instant::now();
    let results = generator.generate_batch(&tools).await.unwrap();
    let duration = start.elapsed();

    let tools_per_sec = tools.len() as f32 / duration.as_secs_f32();

    println!("Batch throughput: {} tools in {:?} ({:.1} tools/sec)",
        tools.len(), duration, tools_per_sec);

    assert_eq!(results.len(), tools.len());
    assert!(duration < Duration::from_secs(5), "Batch should complete in < 5s");
}

#[tokio::test]
async fn bench_with_simulated_latency() {
    // Test with simulated network latency
    let provider = Arc::new(DeterministicMockProvider::new().with_delay(100)); // 100ms delay
    let config = GeneratorConfig::default();
    let generator = ExampleGenerator::new(provider, config);
    let tool = simple_tool();

    let start = Instant::now();
    let _examples = generator.generate(&tool).await.unwrap();
    let duration = start.elapsed();

    println!("Generation with 100ms latency: {:?}", duration);

    // Should account for latency but still be reasonable
    assert!(duration >= Duration::from_millis(100), "Should respect latency");
    assert!(duration < Duration::from_secs(2), "Should still complete quickly");
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[tokio::test]
async fn test_empty_tool_parameters() {
    let provider = Arc::new(DeterministicMockProvider::new());
    let config = GeneratorConfig::default();
    let generator = ExampleGenerator::new(provider, config);

    let tool = ToolDocumentation {
        name: "empty".to_string(),
        description: "A tool with no parameters".to_string(),
        usage: None,
        parameters: vec![],
        examples: vec![],
    };

    // Should still generate examples
    let examples = generator.generate(&tool).await.unwrap();
    assert!(!examples.is_empty(), "Should generate examples even for parameterless tool");
}

#[tokio::test]
async fn test_tool_with_many_parameters() {
    let provider = Arc::new(DeterministicMockProvider::new());
    let config = GeneratorConfig::default();
    let generator = ExampleGenerator::new(provider, config);

    // Kubernetes tool has many parameters
    let tool = kubernetes_apply_tool();
    assert!(tool.parameters.len() >= 5, "Test requires tool with many params");

    let examples = generator.generate(&tool).await.unwrap();
    assert!(!examples.is_empty());

    // Prompt should include all parameters
    // (implicitly tested through successful generation)
}

#[tokio::test]
async fn test_concurrent_generation() {
    let provider = Arc::new(DeterministicMockProvider::new());
    let config = GeneratorConfig::default();
    let generator = Arc::new(ExampleGenerator::new(provider.clone(), config));

    let tools = vec![
        kubernetes_apply_tool(),
        simple_tool(),
        docker_build_tool(),
    ];

    // Generate concurrently
    let handles: Vec<_> = tools.iter()
        .map(|tool| {
            let gen = generator.clone();
            let t = tool.clone();
            tokio::spawn(async move {
                gen.generate(&t).await
            })
        })
        .collect();

    // Wait for all
    let results: Vec<_> = futures_util::future::join_all(handles).await;

    for result in results {
        let examples = result.unwrap().unwrap();
        assert!(!examples.is_empty(), "Each concurrent generation should succeed");
    }
}

// =============================================================================
// Integration with AccuracyMetrics
// =============================================================================

#[tokio::test]
async fn test_comprehensive_accuracy_report() {
    let provider = Arc::new(DeterministicMockProvider::new());
    let config = GeneratorConfig::default();
    let generator = ExampleGenerator::new(provider, config);
    let evaluator = AccuracyEvaluator::new();

    let tools = vec![
        kubernetes_apply_tool(),
        simple_tool(),
        tool_with_constraints(),
    ];

    // Generate examples for all tools
    let mut examples_by_tool = HashMap::new();
    for tool in &tools {
        let examples = generator.generate(tool).await.unwrap();
        examples_by_tool.insert(tool.name.clone(), examples);
    }

    // Evaluate batch
    let metrics = evaluator.evaluate_batch(&tools, &examples_by_tool);

    // Print summary for debugging
    println!("\n{}", metrics.summary());

    // Assertions
    assert!(metrics.total_generated > 0, "Should have generated examples");
    assert!(metrics.validation_rate() >= 0.7, "Should have good validation rate");
    assert!(metrics.diversity_score > 0.0, "Should have diversity score");
    assert!(metrics.per_tool.len() == tools.len(), "Should have metrics per tool");

    // Overall quality should be reasonable
    assert!(
        metrics.overall_quality() >= 0.5,
        "Overall quality should be >= 50%, got {:.1}%",
        metrics.overall_quality() * 100.0
    );
}
