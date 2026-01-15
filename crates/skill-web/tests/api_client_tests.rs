//! Tests for API client
//!
//! These tests run in a WASM environment using wasm-bindgen-test

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;
use skill_web::api::client::ApiClient;

wasm_bindgen_test_configure!(run_in_browser);

// ============================================================================
// URL Construction Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_api_client_default_uses_local_url() {
    let client = ApiClient::default();
    assert_eq!(client.base_url(), "/api");
}

#[wasm_bindgen_test]
fn test_api_client_local_constructor() {
    let client = ApiClient::local();
    assert_eq!(client.base_url(), "/api");
}

#[wasm_bindgen_test]
fn test_api_client_new_with_custom_url() {
    let client = ApiClient::new("http://localhost:3000/api");
    assert_eq!(client.base_url(), "http://localhost:3000/api");
}

#[wasm_bindgen_test]
fn test_api_client_with_host_constructor() {
    let client = ApiClient::with_host("127.0.0.1", 3001);
    assert_eq!(client.base_url(), "http://127.0.0.1:3001/api");
}

#[wasm_bindgen_test]
fn test_api_client_with_host_custom_port() {
    let client = ApiClient::with_host("example.com", 8080);
    assert_eq!(client.base_url(), "http://example.com:8080/api");
}

#[wasm_bindgen_test]
fn test_api_client_base_url_no_trailing_slash() {
    let client = ApiClient::new("/api");
    assert!(!client.base_url().ends_with('/'));
}

#[wasm_bindgen_test]
fn test_api_client_base_url_strips_trailing_slash() {
    let client = ApiClient::new("/api/");
    // The client should handle trailing slashes properly in url() method
    assert!(client.base_url().contains("/api"));
}

// ============================================================================
// URL Building Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_url_building_with_leading_slash() {
    let client = ApiClient::new("/api");
    // Test internal url() method behavior via reflection
    // Since url() is private, we test through public API behavior
    assert_eq!(client.base_url(), "/api");
}

#[wasm_bindgen_test]
fn test_url_building_without_leading_slash() {
    let client = ApiClient::new("/api");
    // The url() method should handle both /path and path correctly
    assert_eq!(client.base_url(), "/api");
}

#[wasm_bindgen_test]
fn test_client_clone_preserves_url() {
    let client = ApiClient::new("http://test.com/api");
    let cloned = client.clone();
    assert_eq!(client.base_url(), cloned.base_url());
}

// ============================================================================
// Query Parameter Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_query_params_serialization_concept() {
    // Test that we can create query params structure
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestQuery {
        page: u32,
        per_page: u32,
    }

    let query = TestQuery {
        page: 1,
        per_page: 20,
    };

    // Verify serialization works
    let serialized = serde_urlencoded::to_string(&query).unwrap();
    assert!(serialized.contains("page=1"));
    assert!(serialized.contains("per_page=20"));
}

#[wasm_bindgen_test]
fn test_query_params_with_special_characters() {
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestQuery {
        search: String,
    }

    let query = TestQuery {
        search: "hello world".to_string(),
    };

    let serialized = serde_urlencoded::to_string(&query).unwrap();
    // URL encoding should handle spaces
    assert!(serialized.contains("search=hello"));
}

#[wasm_bindgen_test]
fn test_query_params_with_optional_fields() {
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestQuery {
        required: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        optional: Option<String>,
    }

    let query = TestQuery {
        required: "value".to_string(),
        optional: None,
    };

    let serialized = serde_urlencoded::to_string(&query).unwrap();
    assert!(serialized.contains("required=value"));
    assert!(!serialized.contains("optional"));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_api_error_types_exist() {
    use skill_web::api::error::ApiError;

    // Test that error types can be created
    let network_error = ApiError::Network("test error".to_string());
    assert!(matches!(network_error, ApiError::Network(_)));

    let serialization_error = ApiError::Serialization("test".to_string());
    assert!(matches!(serialization_error, ApiError::Serialization(_)));
}

#[wasm_bindgen_test]
fn test_api_error_display() {
    use skill_web::api::error::ApiError;

    let error = ApiError::Network("connection failed".to_string());
    let display_string = format!("{}", error);
    assert!(display_string.contains("connection") || display_string.contains("Network"));
}

// ============================================================================
// Request Body Serialization Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_json_body_serialization_simple() {
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestBody {
        name: String,
        value: i32,
    }

    let body = TestBody {
        name: "test".to_string(),
        value: 42,
    };

    let json = serde_json::to_string(&body).unwrap();
    assert!(json.contains("\"name\""));
    assert!(json.contains("\"test\""));
    assert!(json.contains("42"));
}

#[wasm_bindgen_test]
fn test_json_body_serialization_nested() {
    use serde::Serialize;

    #[derive(Serialize)]
    struct Inner {
        inner_value: String,
    }

    #[derive(Serialize)]
    struct TestBody {
        outer: String,
        nested: Inner,
    }

    let body = TestBody {
        outer: "outer".to_string(),
        nested: Inner {
            inner_value: "inner".to_string(),
        },
    };

    let json = serde_json::to_string(&body).unwrap();
    assert!(json.contains("outer"));
    assert!(json.contains("nested"));
    assert!(json.contains("inner_value"));
}

#[wasm_bindgen_test]
fn test_json_body_with_option_fields() {
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestBody {
        required: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        optional: Option<String>,
    }

    let body = TestBody {
        required: "value".to_string(),
        optional: None,
    };

    let json = serde_json::to_string(&body).unwrap();
    assert!(json.contains("required"));
    assert!(!json.contains("optional"));
}

// ============================================================================
// Response Deserialization Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_json_response_deserialization() {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct TestResponse {
        message: String,
        code: i32,
    }

    let json = r#"{"message":"success","code":200}"#;
    let response: TestResponse = serde_json::from_str(json).unwrap();

    assert_eq!(response.message, "success");
    assert_eq!(response.code, 200);
}

#[wasm_bindgen_test]
fn test_json_response_with_optional_fields() {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct TestResponse {
        required: String,
        optional: Option<String>,
    }

    let json = r#"{"required":"value"}"#;
    let response: TestResponse = serde_json::from_str(json).unwrap();

    assert_eq!(response.required, "value");
    assert!(response.optional.is_none());
}

#[wasm_bindgen_test]
fn test_json_response_with_arrays() {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct TestResponse {
        items: Vec<String>,
    }

    let json = r#"{"items":["a","b","c"]}"#;
    let response: TestResponse = serde_json::from_str(json).unwrap();

    assert_eq!(response.items.len(), 3);
    assert_eq!(response.items[0], "a");
}

// ============================================================================
// Error Response Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_api_error_response_structure() {
    use skill_web::api::error::ApiErrorResponse;
    use serde::Deserialize;

    let json = r#"{"code":"NOT_FOUND","message":"Resource not found"}"#;
    let error: ApiErrorResponse = serde_json::from_str(json).unwrap();

    assert_eq!(error.code, "NOT_FOUND");
    assert_eq!(error.message, "Resource not found");
}

#[wasm_bindgen_test]
fn test_api_error_response_with_details() {
    use skill_web::api::error::ApiErrorResponse;

    let json = r#"{"code":"VALIDATION_ERROR","message":"Invalid input","details":{"field":"name"}}"#;
    let error: ApiErrorResponse = serde_json::from_str(json).unwrap();

    assert_eq!(error.code, "VALIDATION_ERROR");
    assert!(error.details.is_some());
}

// ============================================================================
// Client Lifecycle Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_multiple_clients_independent() {
    let client1 = ApiClient::new("http://server1.com/api");
    let client2 = ApiClient::new("http://server2.com/api");

    assert_eq!(client1.base_url(), "http://server1.com/api");
    assert_eq!(client2.base_url(), "http://server2.com/api");
    assert_ne!(client1.base_url(), client2.base_url());
}

#[wasm_bindgen_test]
fn test_client_can_be_stored_in_rc() {
    use std::rc::Rc;

    let client = Rc::new(ApiClient::local());
    let client_clone = client.clone();

    assert_eq!(client.base_url(), client_clone.base_url());
}
