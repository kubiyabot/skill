//! System services API operations

use super::client::ApiClient;
use super::error::ApiResult;
use super::types::{
    ServiceStatus, ServicesStatusResponse, StartServiceRequest, StartServiceResponse,
    StopServiceRequest,
};

/// Services API client
#[derive(Clone)]
pub struct ServicesApi {
    client: ApiClient,
}

impl ServicesApi {
    /// Create a new services API client
    pub fn new(client: ApiClient) -> Self {
        Self { client }
    }

    /// List all system services and their status
    pub async fn list(&self) -> ApiResult<Vec<ServiceStatus>> {
        let response: ServicesStatusResponse = self.client.get("/services").await?;
        Ok(response.services)
    }

    /// Start a service
    pub async fn start(&self, service: &str, port: Option<u16>) -> ApiResult<StartServiceResponse> {
        let request = StartServiceRequest {
            service: service.to_string(),
            port,
        };
        self.client.post("/services/start", &request).await
    }

    /// Stop a service
    pub async fn stop(&self, service: &str) -> ApiResult<StartServiceResponse> {
        let request = StopServiceRequest {
            service: service.to_string(),
        };
        self.client.post("/services/stop", &request).await
    }

    /// Check if kubectl proxy is running
    pub async fn is_kubectl_proxy_running(&self) -> ApiResult<bool> {
        let services = self.list().await?;
        Ok(services.iter().any(|s| s.name == "kubectl-proxy" && s.running))
    }

    /// Start kubectl proxy
    pub async fn start_kubectl_proxy(&self, port: Option<u16>) -> ApiResult<StartServiceResponse> {
        self.start("kubectl-proxy", port).await
    }

    /// Stop kubectl proxy
    pub async fn stop_kubectl_proxy(&self) -> ApiResult<StartServiceResponse> {
        self.stop("kubectl-proxy").await
    }
}
