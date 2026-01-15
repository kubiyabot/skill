//! Analytics API client

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{ApiClient, ApiResult};

#[derive(Clone)]
pub struct AnalyticsApi {
    client: ApiClient,
}

impl AnalyticsApi {
    pub fn new(client: ApiClient) -> Self {
        Self { client }
    }

    /// Get analytics overview
    pub async fn get_overview(&self, days: u32) -> ApiResult<AnalyticsOverviewResponse> {
        self.client
            .get(&format!("/analytics/overview?days={}", days))
            .await
    }

    /// Get top queries
    pub async fn get_top_queries(&self, limit: usize, days: u32) -> ApiResult<TopQueriesResponse> {
        self.client
            .get(&format!(
                "/analytics/top-queries?limit={}&days={}",
                limit, days
            ))
            .await
    }

    /// Get feedback statistics
    pub async fn get_feedback_stats(&self, days: u32) -> ApiResult<FeedbackStatsResponse> {
        self.client
            .get(&format!("/analytics/feedback-stats?days={}", days))
            .await
    }

    /// Get search timeline
    pub async fn get_timeline(
        &self,
        days: u32,
        interval_hours: u32,
    ) -> ApiResult<SearchTimelineResponse> {
        self.client
            .get(&format!(
                "/analytics/timeline?days={}&interval_hours={}",
                days, interval_hours
            ))
            .await
    }
}

// Response types

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnalyticsOverviewResponse {
    pub total_searches: usize,
    pub total_feedback: usize,
    pub positive_feedback: usize,
    pub negative_feedback: usize,
    pub avg_latency_ms: f64,
    pub avg_results: f64,
    pub recent_searches: Vec<SearchHistorySummary>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchHistorySummary {
    pub query: String,
    pub results_count: usize,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TopQueriesResponse {
    pub queries: Vec<QueryStats>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueryStats {
    pub query: String,
    pub count: usize,
    pub avg_results: f64,
    pub avg_latency_ms: f64,
    pub positive_feedback: usize,
    pub negative_feedback: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeedbackStatsResponse {
    pub by_type: Vec<FeedbackTypeCount>,
    pub top_positive: Vec<ResultFeedbackSummary>,
    pub top_negative: Vec<ResultFeedbackSummary>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeedbackTypeCount {
    pub feedback_type: String,
    pub count: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResultFeedbackSummary {
    pub result_id: String,
    pub positive_count: usize,
    pub negative_count: usize,
    pub total_count: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchTimelineResponse {
    pub timeline: Vec<TimelineDataPoint>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimelineDataPoint {
    pub timestamp: DateTime<Utc>,
    pub search_count: usize,
    pub avg_latency_ms: f64,
}
