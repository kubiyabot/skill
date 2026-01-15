//! Feedback API client

use serde::{Deserialize, Serialize};
use super::{ApiClient, ApiResult};

#[derive(Clone)]
pub struct FeedbackApi {
    client: ApiClient,
}

impl FeedbackApi {
    pub fn new(client: ApiClient) -> Self {
        Self { client }
    }

    /// Submit feedback for a search result
    pub async fn submit(&self, request: &SubmitFeedbackRequest) -> ApiResult<SubmitFeedbackResponse> {
        self.client.post("/feedback", request).await
    }

    /// Get feedback with filters
    pub async fn get(&self, request: &GetFeedbackRequest) -> ApiResult<GetFeedbackResponse> {
        let mut query_params = vec![];

        if let Some(query) = &request.query {
            query_params.push(format!("query={}", urlencoding::encode(query)));
        }
        if let Some(result_id) = &request.result_id {
            query_params.push(format!("result_id={}", urlencoding::encode(result_id)));
        }
        if let Some(feedback_type) = &request.feedback_type {
            query_params.push(format!("feedback_type={}", urlencoding::encode(feedback_type)));
        }
        query_params.push(format!("limit={}", request.limit));
        query_params.push(format!("offset={}", request.offset));

        let query_string = query_params.join("&");
        let url = format!("/feedback?{}", query_string);

        self.client.get(&url).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitFeedbackRequest {
    pub query: String,
    pub result_id: String,
    pub score: f32,
    pub rank: usize,
    pub feedback_type: String, // "positive" or "negative"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub client_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitFeedbackResponse {
    pub success: bool,
    pub feedback_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct GetFeedbackRequest {
    pub query: Option<String>,
    pub result_id: Option<String>,
    pub feedback_type: Option<String>,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetFeedbackResponse {
    pub feedback: Vec<FeedbackEntry>,
    pub total_count: usize,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackEntry {
    pub id: String,
    pub query: String,
    pub result_id: String,
    pub score: f32,
    pub rank: usize,
    pub feedback_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub client_type: String,
    pub timestamp: String,
}
