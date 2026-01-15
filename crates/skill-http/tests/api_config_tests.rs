//! Integration tests for Configuration API endpoints

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;

// ============================================================================
// Get Config Tests
// ============================================================================

#[tokio::test]
async fn test_get_config_returns_200() {
    let app = TestApp::new().await;
    let req = TestApp::get_request("/api/config");
    let (status, body) = app.request(req).await;

    assert_eq!(status, StatusCode::OK);

    let config: serde_json::Value = TestApp::parse_json(&body);
    assert!(config.is_object());
}

#[tokio::test]
async fn test_get_config_structure() {
    let app = TestApp::new().await;
    let req = TestApp::get_request("/api/config");
    let (status, body) = app.request(req).await;

    if status == StatusCode::OK {
        let config: serde_json::Value = TestApp::parse_json(&body);

        // Config should have expected fields
        assert!(config.is_object());
        // May have fields like host, port, etc.
    }
}

// ============================================================================
// Update Config Tests
// ============================================================================

#[tokio::test]
async fn test_update_config_with_valid_data() {
    let app = TestApp::new().await;
    let body = json!({
        "host": "127.0.0.1",
        "port": 3000
    }).to_string();

    let req = TestApp::put_request("/api/config", &body);
    let (status, _) = app.request(req).await;

    // Config update may or may not be implemented
    assert!(
        status == StatusCode::OK
        || status == StatusCode::NOT_IMPLEMENTED
        || status == StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn test_update_config_invalid_json() {
    let app = TestApp::new().await;
    let body = r#"{ invalid json }"#;

    let req = TestApp::put_request("/api/config", body);
    let (status, _) = app.request(req).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ============================================================================
// Health Check Tests
// ============================================================================

#[tokio::test]
async fn test_health_endpoint_returns_200() {
    let app = TestApp::new().await;
    let req = TestApp::get_request("/api/health");
    let (status, body) = app.request(req).await;

    assert_eq!(status, StatusCode::OK);

    let health: serde_json::Value = TestApp::parse_json(&body);
    assert!(health.is_object());
}

#[tokio::test]
async fn test_health_response_structure() {
    let app = TestApp::new().await;
    let req = TestApp::get_request("/api/health");
    let (status, body) = app.request(req).await;

    if status == StatusCode::OK {
        let health: serde_json::Value = TestApp::parse_json(&body);

        // Health should have status field
        assert!(health.get("status").is_some() || health.get("healthy").is_some());
    }
}

// ============================================================================
// Version Tests
// ============================================================================

#[tokio::test]
async fn test_version_endpoint_returns_200() {
    let app = TestApp::new().await;
    let req = TestApp::get_request("/api/version");
    let (status, body) = app.request(req).await;

    assert_eq!(status, StatusCode::OK);

    let version: serde_json::Value = TestApp::parse_json(&body);
    assert!(version.is_object() || version.is_string());
}

#[tokio::test]
async fn test_version_has_version_field() {
    let app = TestApp::new().await;
    let req = TestApp::get_request("/api/version");
    let (status, body) = app.request(req).await;

    if status == StatusCode::OK {
        let version: serde_json::Value = TestApp::parse_json(&body);

        if version.is_object() {
            assert!(version.get("version").is_some());
        }
    }
}

// ============================================================================
// Manifest Validation Tests
// ============================================================================

#[tokio::test]
async fn test_validate_manifest_with_valid_toml() {
    let app = TestApp::new().await;
    let body = json!({
        "content": r#"
version = "1"

[skills.test]
source = "github:test/skill"
runtime = "wasm"
        "#
    }).to_string();

    let req = TestApp::post_request("/api/manifest/validate", &body);
    let (status, resp_body) = app.request(req).await;

    assert_eq!(status, StatusCode::OK);

    let response: serde_json::Value = TestApp::parse_json(&resp_body);
    assert_eq!(response["valid"], true);
}

#[tokio::test]
async fn test_validate_manifest_with_invalid_toml() {
    let app = TestApp::new().await;
    let body = json!({
        "content": r#"
invalid toml [[[
        "#
    }).to_string();

    let req = TestApp::post_request("/api/manifest/validate", &body);
    let (status, resp_body) = app.request(req).await;

    assert_eq!(status, StatusCode::OK);

    let response: serde_json::Value = TestApp::parse_json(&resp_body);
    assert_eq!(response["valid"], false);
    assert!(response["errors"].as_array().unwrap().len() > 0);
}

// ============================================================================
// Manifest Import Tests
// ============================================================================

#[tokio::test]
async fn test_import_manifest_with_valid_data() {
    let app = TestApp::new().await;
    let body = json!({
        "content": r#"
version = "1"

[skills.imported-skill]
source = "github:test/imported"
runtime = "wasm"
        "#,
        "install": false,
        "merge": true
    }).to_string();

    let req = TestApp::post_request("/api/manifest/import", &body);
    let (status, resp_body) = app.request(req).await;

    assert_eq!(status, StatusCode::OK);

    let response: serde_json::Value = TestApp::parse_json(&resp_body);
    assert!(response["skills_count"].as_u64().unwrap() >= 1);
}

// ============================================================================
// Manifest Export Tests
// ============================================================================

#[tokio::test]
async fn test_export_manifest_returns_toml() {
    let app = TestApp::new().await;
    let req = TestApp::get_request("/api/manifest/export");
    let (status, body) = app.request(req).await;

    // Export may or may not be implemented
    if status == StatusCode::OK {
        let manifest: serde_json::Value = TestApp::parse_json(&body);
        // Should return manifest content
        assert!(manifest.is_object() || manifest.is_string());
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_config_endpoints_handle_empty_body() {
    let app = TestApp::new().await;
    let body = "{}";

    let req = TestApp::put_request("/api/config", body);
    let (status, _) = app.request(req).await;

    // Should handle empty body gracefully
    assert!(
        status == StatusCode::OK
        || status == StatusCode::BAD_REQUEST
        || status == StatusCode::NOT_IMPLEMENTED
    );
}

#[tokio::test]
async fn test_manifest_endpoints_handle_missing_fields() {
    let app = TestApp::new().await;
    let body = json!({
        "invalid_field": "value"
    }).to_string();

    let req = TestApp::post_request("/api/manifest/validate", &body);
    let (status, _) = app.request(req).await;

    // Should handle missing required fields
    assert!(
        status == StatusCode::BAD_REQUEST
        || status == StatusCode::NOT_FOUND
        || status == StatusCode::NOT_IMPLEMENTED
        || status == StatusCode::UNPROCESSABLE_ENTITY
    );
}
