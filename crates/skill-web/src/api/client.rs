//! HTTP API client for the skill-http backend

use gloo_net::http::Request;
use serde::{de::DeserializeOwned, Serialize};
use std::rc::Rc;

use super::error::{ApiError, ApiErrorResponse, ApiResult};

/// API client for making requests to the backend
#[derive(Clone)]
pub struct ApiClient {
    base_url: Rc<str>,
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::local()
    }
}

impl ApiClient {
    /// Create a new API client with the given base URL
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().into(),
        }
    }

    /// Create a client pointing to the default local server (proxied through Trunk)
    pub fn local() -> Self {
        Self::new("/api")
    }

    /// Create a client pointing to a specific host
    pub fn with_host(host: &str, port: u16) -> Self {
        Self::new(format!("http://{}:{}/api", host, port))
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Build a full URL from a path
    fn url(&self, path: &str) -> String {
        if path.starts_with('/') {
            format!("{}{}", self.base_url, path)
        } else {
            format!("{}/{}", self.base_url, path)
        }
    }

    /// Handle response and parse JSON or error
    async fn handle_response<T: DeserializeOwned>(
        response: gloo_net::http::Response,
    ) -> ApiResult<T> {
        let status = response.status();

        if response.ok() {
            response
                .json()
                .await
                .map_err(|e| ApiError::Deserialization(e.to_string()))
        } else {
            // Try to parse error response
            let error = match response.json::<ApiErrorResponse>().await {
                Ok(err_resp) => err_resp.into(),
                Err(_) => ApiError::from_status(status, response.status_text()),
            };
            Err(error)
        }
    }

    /// Handle response that returns no body (204 No Content)
    async fn handle_empty_response(response: gloo_net::http::Response) -> ApiResult<()> {
        let status = response.status();

        if response.ok() {
            Ok(())
        } else {
            let error = match response.json::<ApiErrorResponse>().await {
                Ok(err_resp) => err_resp.into(),
                Err(_) => ApiError::from_status(status, response.status_text()),
            };
            Err(error)
        }
    }

    /// Make a GET request
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> ApiResult<T> {
        let url = self.url(path);

        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        Self::handle_response(response).await
    }

    /// Make a GET request with query parameters
    pub async fn get_with_query<T: DeserializeOwned, Q: Serialize>(
        &self,
        path: &str,
        query: &Q,
    ) -> ApiResult<T> {
        let query_string = serde_urlencoded::to_string(query)
            .map_err(|e| ApiError::Serialization(e.to_string()))?;

        let url = if query_string.is_empty() {
            self.url(path)
        } else {
            format!("{}?{}", self.url(path), query_string)
        };

        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        Self::handle_response(response).await
    }

    /// Make a POST request with JSON body
    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> ApiResult<T> {
        let url = self.url(path);

        let response = Request::post(&url)
            .header("Content-Type", "application/json")
            .json(body)
            .map_err(|e| ApiError::Serialization(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        Self::handle_response(response).await
    }

    /// Make a POST request without expecting a response body
    pub async fn post_no_response<B: Serialize>(&self, path: &str, body: &B) -> ApiResult<()> {
        let url = self.url(path);

        let response = Request::post(&url)
            .header("Content-Type", "application/json")
            .json(body)
            .map_err(|e| ApiError::Serialization(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        Self::handle_empty_response(response).await
    }

    /// Make a PUT request with JSON body
    pub async fn put<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> ApiResult<T> {
        let url = self.url(path);

        let response = Request::put(&url)
            .header("Content-Type", "application/json")
            .json(body)
            .map_err(|e| ApiError::Serialization(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        Self::handle_response(response).await
    }

    /// Make a PATCH request with JSON body
    pub async fn patch<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> ApiResult<T> {
        let url = self.url(path);

        let response = Request::patch(&url)
            .header("Content-Type", "application/json")
            .json(body)
            .map_err(|e| ApiError::Serialization(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        Self::handle_response(response).await
    }

    /// Make a DELETE request
    pub async fn delete(&self, path: &str) -> ApiResult<()> {
        let url = self.url(path);

        let response = Request::delete(&url)
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        Self::handle_empty_response(response).await
    }

    /// Make a DELETE request expecting a response body
    pub async fn delete_with_response<T: DeserializeOwned>(&self, path: &str) -> ApiResult<T> {
        let url = self.url(path);

        let response = Request::delete(&url)
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        Self::handle_response(response).await
    }
}
