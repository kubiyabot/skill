//! API route definitions

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::embedded;
use crate::handlers;
use crate::openapi::ApiDoc;
use crate::AppState;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Create the main API router
pub fn api_routes(state: Arc<AppState>) -> Router {
    Router::new()
        // Skills endpoints
        .route("/skills", get(handlers::list_skills))
        .route("/skills", post(handlers::install_skill))
        .route("/skills/:name", get(handlers::get_skill))
        .route("/skills/:name", delete(handlers::uninstall_skill))
        // Execution endpoints
        .route("/execute", post(handlers::execute_tool))
        .route("/executions", get(handlers::list_executions))
        .route("/executions", delete(handlers::clear_execution_history))
        .route("/executions/:id", get(handlers::get_execution))
        // Search endpoints
        .route("/search", post(handlers::semantic_search))
        .route("/search/config", get(handlers::get_search_config))
        .route("/search/config", put(handlers::update_search_config))
        .route("/search/index", post(handlers::index_skills))
        .route("/search/test-connection", post(handlers::test_search_connection))
        .route("/search/test-pipeline", post(handlers::test_search_pipeline))
        // Feedback endpoints
        .route("/feedback", post(handlers::submit_feedback))
        .route("/feedback", get(handlers::get_feedback))
        // Analytics endpoints
        .route("/analytics/overview", get(handlers::get_analytics_overview))
        .route("/analytics/top-queries", get(handlers::get_top_queries))
        .route("/analytics/feedback-stats", get(handlers::get_feedback_statistics))
        .route("/analytics/timeline", get(handlers::get_search_timeline))
        // Agent configuration endpoints
        .route("/agent/config", get(handlers::get_agent_config))
        .route("/agent/config", put(handlers::update_agent_config))
        // Configuration endpoints
        .route("/config", get(handlers::get_config))
        .route("/config", put(handlers::update_config))
        // Manifest import/export endpoints
        .route("/manifest/validate", post(handlers::validate_manifest))
        .route("/manifest/import", post(handlers::import_manifest))
        .route("/manifest/export", post(handlers::export_manifest))
        // System service endpoints
        .route("/services", get(handlers::list_services))
        .route("/services/start", post(handlers::start_service))
        .route("/services/stop", post(handlers::stop_service))
        // Health and version
        .route("/health", get(handlers::health_check))
        .route("/version", get(handlers::version_info))
        // Apply state to all routes
        .with_state(state)
}

/// Create the full application router with API prefix (API only, no web UI)
pub fn create_app(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/docs/api").url("/api/openapi.json", ApiDoc::openapi()))
        .nest("/api", api_routes(state))
        .fallback(handlers::not_found)
}

/// Create the full application router with embedded web UI
///
/// This router serves:
/// - `/api/*` - REST API endpoints
/// - `/docs/api` - Swagger UI for API documentation
/// - `/*` - Embedded web UI (SPA with client-side routing)
pub fn create_app_with_ui(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/docs/api").url("/api/openapi.json", ApiDoc::openapi()))
        .nest("/api", api_routes(state))
        .fallback(serve_static_handler)
}

/// Handler for serving static assets from embedded files
async fn serve_static_handler(
    uri: axum::http::Uri,
) -> impl axum::response::IntoResponse {
    let path = uri.path();
    embedded::serve_static(path).await
}
