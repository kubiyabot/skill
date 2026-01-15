//! Embedded web assets for the Skill Engine web UI
//!
//! This module embeds the compiled WASM web interface into the binary,
//! allowing the web UI to be served without external dependencies.
//!
//! Enable the `web-ui` feature to include the embedded assets.
//! Without this feature, the web UI will show a placeholder message.

use axum::{
    body::Body,
    http::{header, StatusCode},
    response::Response,
};

/// Embedded web UI assets (when web-ui feature is enabled)
#[cfg(feature = "web-ui")]
mod inner {
    use rust_embed::RustEmbed;

    #[derive(RustEmbed)]
    #[folder = "$CARGO_MANIFEST_DIR/../skill-web/dist"]
    pub struct Assets;
}

/// Serve a static asset from the embedded files
///
/// This function handles:
/// - Direct file serving with correct MIME types
/// - SPA fallback (serves index.html for client-side routing)
/// - 404 for truly missing files
#[cfg(feature = "web-ui")]
pub async fn serve_static(path: &str) -> Response {
    use rust_embed::Embed;

    // Normalize the path - remove leading slash if present
    let path = path.trim_start_matches('/');

    // Default to index.html for root path
    let path = if path.is_empty() { "index.html" } else { path };

    // Try to get the asset
    match inner::Assets::get(path) {
        Some(content) => {
            // Determine MIME type from file extension
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .header(header::CACHE_CONTROL, cache_control_header(path))
                .body(Body::from(content.data.into_owned()))
                .unwrap()
        }
        None => {
            // SPA fallback: for HTML routes (no extension or .html), serve index.html
            // This enables client-side routing to work correctly
            if is_spa_route(path) {
                if let Some(index) = inner::Assets::get("index.html") {
                    return Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                        .body(Body::from(index.data.into_owned()))
                        .unwrap();
                }
            }

            // Truly not found
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(header::CONTENT_TYPE, "text/plain")
                .body(Body::from("Not found"))
                .unwrap()
        }
    }
}

/// Serve a placeholder when web-ui feature is not enabled
#[cfg(not(feature = "web-ui"))]
pub async fn serve_static(_path: &str) -> Response {
    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Skill Engine Web UI</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            color: #e0e0e0;
        }
        .container {
            text-align: center;
            padding: 2rem;
            max-width: 600px;
        }
        h1 {
            color: #00d9ff;
            margin-bottom: 1rem;
        }
        p {
            line-height: 1.6;
            margin-bottom: 1.5rem;
        }
        code {
            background: #2a2a4a;
            padding: 0.2rem 0.5rem;
            border-radius: 4px;
            font-family: 'Monaco', 'Consolas', monospace;
        }
        pre {
            background: #2a2a4a;
            padding: 1rem;
            border-radius: 8px;
            text-align: left;
            overflow-x: auto;
        }
        a {
            color: #00d9ff;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🔧 Skill Engine</h1>
        <p>The web UI is not built into this binary.</p>
        <p>To enable the web UI, build with the <code>web-ui</code> feature:</p>
        <pre>
# Build the web UI first
cd crates/skill-web
trunk build --release

# Then build the CLI with web-ui feature
cargo build -p skill-cli --features skill-http/web-ui
        </pre>
        <p>The API is still available at <a href="/api/health">/api/health</a></p>
    </div>
</body>
</html>"#;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(html))
        .unwrap()
}

/// Check if a path should use SPA fallback (serve index.html)
#[cfg(feature = "web-ui")]
fn is_spa_route(path: &str) -> bool {
    // Has no file extension (likely a client-side route)
    let has_extension = path.contains('.') && !path.ends_with('/');

    // Or explicitly asking for HTML
    let is_html = path.ends_with(".html") || path.ends_with(".htm");

    // SPA routes are paths without extensions OR HTML files
    !has_extension || is_html
}

/// Generate appropriate cache control header based on file type
#[cfg(feature = "web-ui")]
fn cache_control_header(path: &str) -> &'static str {
    // WASM and JS files with hashes can be cached long-term
    if path.ends_with(".wasm") || (path.contains("-") && path.ends_with(".js")) {
        "public, max-age=31536000, immutable"
    }
    // CSS with hashes can also be cached long-term
    else if path.contains("-") && path.ends_with(".css") {
        "public, max-age=31536000, immutable"
    }
    // HTML should be revalidated
    else if path.ends_with(".html") || path.ends_with(".htm") {
        "no-cache, must-revalidate"
    }
    // Everything else gets short cache
    else {
        "public, max-age=3600"
    }
}

/// Check if the embedded assets are available
#[cfg(feature = "web-ui")]
pub fn has_assets() -> bool {
    use rust_embed::Embed;
    inner::Assets::get("index.html").is_some()
}

#[cfg(not(feature = "web-ui"))]
pub fn has_assets() -> bool {
    false
}

/// Get the list of all embedded assets (for debugging)
#[cfg(feature = "web-ui")]
pub fn list_assets() -> Vec<String> {
    use rust_embed::Embed;
    inner::Assets::iter().map(|s| s.to_string()).collect()
}

#[cfg(not(feature = "web-ui"))]
pub fn list_assets() -> Vec<String> {
    vec![]
}
