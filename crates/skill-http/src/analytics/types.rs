//! Type definitions for analytics data

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Search history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistoryEntry {
    pub id: Uuid,
    pub query: String,
    pub top_k: usize,
    pub results_count: usize,
    pub avg_score: Option<f32>,
    pub duration_ms: u64,
    pub client_type: String,
    pub client_id: Option<String>,
    pub session_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Search feedback entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFeedbackEntry {
    pub id: Uuid,
    pub query: String,
    pub result_id: String,
    pub score: f32,
    pub rank: usize,
    pub feedback_type: FeedbackType,
    pub reason: Option<String>,
    pub comment: Option<String>,
    pub client_type: String,
    pub timestamp: DateTime<Utc>,
}

/// Type of feedback
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FeedbackType {
    Positive,
    Negative,
}

impl FeedbackType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FeedbackType::Positive => "positive",
            FeedbackType::Negative => "negative",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "positive" => Some(FeedbackType::Positive),
            "negative" => Some(FeedbackType::Negative),
            _ => None,
        }
    }
}

/// Filter for querying search history
#[derive(Debug, Clone, Default)]
pub struct SearchHistoryFilter {
    pub client_type: Option<String>,
    pub client_id: Option<String>,
    pub session_id: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Filter for querying feedback
#[derive(Debug, Clone, Default)]
pub struct FeedbackFilter {
    pub query: Option<String>,
    pub result_id: Option<String>,
    pub feedback_type: Option<FeedbackType>,
    pub client_type: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}
