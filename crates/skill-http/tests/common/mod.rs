//! Common test utilities for HTTP integration tests

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use chrono::Utc;
use skill_http::{types::*, AppState, HttpServerConfig};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tower::ServiceExt;

/// Test app state with mock data
pub struct TestApp {
    pub state: Arc<AppState>,
}

impl TestApp {
    /// Create a new test app with fixtures
    pub async fn new() -> Self {
        let config = HttpServerConfig {
            host: "127.0.0.1".to_string(),
            port: 3001,
            enable_cors: true,
            enable_tracing: false,
            enable_web_ui: false,
            working_dir: Some(test_fixtures_dir()),
        };

        let state = Arc::new(AppState::new(config).unwrap());

        // Pre-populate with test skills
        load_test_skills(&state).await;

        Self { state }
    }

    /// Make a request to the app and get response
    pub async fn request(&self, req: Request<Body>) -> (StatusCode, Vec<u8>) {
        let app = skill_http::routes::create_app(self.state.clone());
        let response = app.oneshot(req).await.unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        (status, body.to_vec())
    }

    /// Helper to create a GET request
    pub fn get_request(path: &str) -> Request<Body> {
        Request::builder()
            .method("GET")
            .uri(path)
            .body(Body::empty())
            .unwrap()
    }

    /// Helper to create a POST request with JSON body
    pub fn post_request(path: &str, body: &str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    /// Helper to create a DELETE request
    pub fn delete_request(path: &str) -> Request<Body> {
        Request::builder()
            .method("DELETE")
            .uri(path)
            .body(Body::empty())
            .unwrap()
    }

    /// Helper to create a PUT request with JSON body
    pub fn put_request(path: &str, body: &str) -> Request<Body> {
        Request::builder()
            .method("PUT")
            .uri(path)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    /// Parse JSON response
    pub fn parse_json<T: serde::de::DeserializeOwned>(body: &[u8]) -> T {
        serde_json::from_slice(body).expect("Failed to parse JSON response")
    }
}

/// Get the test fixtures directory path
pub fn test_fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Load test skills into the app state
async fn load_test_skills(state: &Arc<AppState>) {
    let mut skills = state.skills.write().await;

    // Add test skill
    skills.insert(
        "test-skill".to_string(),
        SkillSummary {
            name: "test-skill".to_string(),
            version: "0.1.0".to_string(),
            description: "A test skill for integration tests".to_string(),
            source: "local:./test-skill".to_string(),
            runtime: "wasm".to_string(),
            tools_count: 3,
            instances_count: 1,
            execution_count: 0,
            last_used: None,
            required_services: vec![],
        },
    );

    // Add another test skill
    skills.insert(
        "aws-skill".to_string(),
        SkillSummary {
            name: "aws-skill".to_string(),
            version: "1.0.0".to_string(),
            description: "AWS operations skill".to_string(),
            source: "github:test/aws-skill".to_string(),
            runtime: "wasm".to_string(),
            tools_count: 10,
            instances_count: 2,
            execution_count: 5,
            last_used: Some(Utc::now()),
            required_services: vec![],
        },
    );

    // Add kubernetes skill with service requirement
    skills.insert(
        "kubernetes-skill".to_string(),
        SkillSummary {
            name: "kubernetes-skill".to_string(),
            version: "2.0.0".to_string(),
            description: "Kubernetes management".to_string(),
            source: "github:test/k8s-skill".to_string(),
            runtime: "wasm".to_string(),
            tools_count: 15,
            instances_count: 1,
            execution_count: 0,
            last_used: None,
            required_services: vec![SkillServiceRequirement {
                name: "kubectl-proxy".to_string(),
                description: Some("Kubernetes API proxy".to_string()),
                optional: false,
                default_port: Some(8001),
                status: ServiceStatus {
                    name: "kubectl-proxy".to_string(),
                    running: false,
                    pid: None,
                    port: None,
                    url: None,
                    error: None,
                },
            }],
        },
    );
}

/// Create a mock skill summary for testing
pub fn mock_skill_summary(name: &str) -> SkillSummary {
    SkillSummary {
        name: name.to_string(),
        version: "0.1.0".to_string(),
        description: format!("Test skill {}", name),
        source: format!("local:./{}", name),
        runtime: "wasm".to_string(),
        tools_count: 5,
        instances_count: 1,
        execution_count: 0,
        last_used: None,
        required_services: vec![],
    }
}

/// Create a mock execution response for testing
pub fn mock_execution_response() -> ExecutionResponse {
    ExecutionResponse {
        id: "test-exec-123".to_string(),
        status: ExecutionStatus::Success,
        output: r#"{"result": "success", "data": [1, 2, 3]}"#.to_string(),
        error: None,
        duration_ms: 42,
        metadata: HashMap::new(),
    }
}
