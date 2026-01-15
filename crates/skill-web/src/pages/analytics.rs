//! Analytics Dashboard Page - Visualize search and feedback data
//!
//! Features:
//! - Overview statistics (searches, feedback, latency)
//! - Top queries with feedback counts
//! - Recent search history
//! - Feedback statistics by type and result
//! - Time range selector (7, 30, 90 days)

use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::api::analytics::{
    AnalyticsOverviewResponse, FeedbackStatsResponse, TopQueriesResponse,
};
use crate::api::Api;
use crate::components::card::Card;
use crate::components::use_notifications;

#[derive(Clone, PartialEq)]
enum TimeRange {
    Week,
    Month,
    Quarter,
}

impl TimeRange {
    fn to_days(&self) -> u32 {
        match self {
            TimeRange::Week => 7,
            TimeRange::Month => 30,
            TimeRange::Quarter => 90,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            TimeRange::Week => "Last 7 Days",
            TimeRange::Month => "Last 30 Days",
            TimeRange::Quarter => "Last 90 Days",
        }
    }
}

#[function_component(AnalyticsPage)]
pub fn analytics_page() -> Html {
    // State
    let time_range = use_state(|| TimeRange::Month);
    let overview = use_state(|| None::<AnalyticsOverviewResponse>);
    let top_queries = use_state(|| None::<TopQueriesResponse>);
    let feedback_stats = use_state(|| None::<FeedbackStatsResponse>);
    let is_loading = use_state(|| false);

    // API & notifications
    let api = use_memo((), |_| Rc::new(Api::new()));
    let notifications = use_notifications();

    // Load data effect
    {
        let api = api.clone();
        let time_range = time_range.clone();
        let overview = overview.clone();
        let top_queries = top_queries.clone();
        let feedback_stats = feedback_stats.clone();
        let is_loading = is_loading.clone();
        let notifications = notifications.clone();

        use_effect_with((*time_range).clone(), move |range| {
            let days = range.to_days();
            is_loading.set(true);

            let api = api.clone();
            let overview = overview.clone();
            let top_queries = top_queries.clone();
            let feedback_stats = feedback_stats.clone();
            let is_loading = is_loading.clone();
            let notifications = notifications.clone();

            spawn_local(async move {
                // Load all analytics data sequentially (WASM doesn't have tokio::try_join!)
                match api.analytics.get_overview(days).await {
                    Ok(ov) => overview.set(Some(ov)),
                    Err(e) => {
                        notifications.error("Failed to load overview", format!("Error: {}", e));
                        is_loading.set(false);
                        return;
                    }
                }

                match api.analytics.get_top_queries(10, days).await {
                    Ok(tq) => top_queries.set(Some(tq)),
                    Err(e) => {
                        notifications.error("Failed to load top queries", format!("Error: {}", e));
                    }
                }

                match api.analytics.get_feedback_stats(days).await {
                    Ok(fs) => feedback_stats.set(Some(fs)),
                    Err(e) => {
                        notifications.error("Failed to load feedback stats", format!("Error: {}", e));
                    }
                }

                is_loading.set(false);
            });

            || ()
        });
    }

    // Time range selector callback
    let on_range_change = time_range.clone();

    html! {
        <div class="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
            <div class="max-w-7xl mx-auto space-y-6">
                // Header
                <div class="flex items-center justify-between">
                    <div>
                        <h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-1">
                            { "Search Analytics" }
                        </h1>
                        <p class="text-sm text-gray-600 dark:text-gray-400">
                            { "Monitor search performance and user feedback" }
                        </p>
                    </div>

                    // Time range selector
                    <div class="flex gap-2">
                        {[TimeRange::Week, TimeRange::Month, TimeRange::Quarter].iter().map(|range| {
                            let is_active = &*time_range == range;
                            let range_clone = range.clone();
                            let time_range_setter = on_range_change.clone();
                            let on_click = Callback::from(move |_: web_sys::MouseEvent| {
                                time_range_setter.set(range_clone.clone());
                            });

                            html! {
                                <button
                                    class={classes!(
                                        "px-4", "py-2", "rounded-lg", "text-sm", "font-medium", "transition-colors",
                                        if is_active {
                                            "bg-primary-500 text-white"
                                        } else {
                                            "bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
                                        }
                                    )}
                                    onclick={on_click}
                                >
                                    { range.label() }
                                </button>
                            }
                        }).collect::<Html>()}
                    </div>
                </div>

                if *is_loading {
                    <div class="flex items-center justify-center py-12">
                        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-500"></div>
                    </div>
                } else {
                    <>
                        // Overview Cards
                        if let Some(ov) = &*overview {
                            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                                // Total Searches
                                <Card>
                                    <div class="p-6">
                                        <div class="flex items-center justify-between mb-2">
                                            <p class="text-sm font-medium text-gray-600 dark:text-gray-400">
                                                { "Total Searches" }
                                            </p>
                                            <svg class="w-5 h-5 text-blue-500" fill="currentColor" viewBox="0 0 20 20">
                                                <path fill-rule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" clip-rule="evenodd" />
                                            </svg>
                                        </div>
                                        <p class="text-3xl font-bold text-gray-900 dark:text-white">
                                            { ov.total_searches }
                                        </p>
                                        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                                            { format!("Avg {:.1} results", ov.avg_results) }
                                        </p>
                                    </div>
                                </Card>

                                // Total Feedback
                                <Card>
                                    <div class="p-6">
                                        <div class="flex items-center justify-between mb-2">
                                            <p class="text-sm font-medium text-gray-600 dark:text-gray-400">
                                                { "Total Feedback" }
                                            </p>
                                            <svg class="w-5 h-5 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                                                <path d="M2 10.5a1.5 1.5 0 113 0v6a1.5 1.5 0 01-3 0v-6zM6 10.333v5.43a2 2 0 001.106 1.79l.05.025A4 4 0 008.943 18h5.416a2 2 0 001.962-1.608l1.2-6A2 2 0 0015.56 8H12V4a2 2 0 00-2-2 1 1 0 00-1 1v.667a4 4 0 01-.8 2.4L6.8 7.933a4 4 0 00-.8 2.4z" />
                                            </svg>
                                        </div>
                                        <p class="text-3xl font-bold text-gray-900 dark:text-white">
                                            { ov.total_feedback }
                                        </p>
                                        <p class="text-xs text-green-600 dark:text-green-400 mt-1">
                                            { format!("{} positive", ov.positive_feedback) }
                                        </p>
                                    </div>
                                </Card>

                                // Average Latency
                                <Card>
                                    <div class="p-6">
                                        <div class="flex items-center justify-between mb-2">
                                            <p class="text-sm font-medium text-gray-600 dark:text-gray-400">
                                                { "Avg Latency" }
                                            </p>
                                            <svg class="w-5 h-5 text-purple-500" fill="currentColor" viewBox="0 0 20 20">
                                                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-12a1 1 0 10-2 0v4a1 1 0 00.293.707l2.828 2.829a1 1 0 101.415-1.415L11 9.586V6z" clip-rule="evenodd" />
                                            </svg>
                                        </div>
                                        <p class="text-3xl font-bold text-gray-900 dark:text-white">
                                            { format!("{:.0}ms", ov.avg_latency_ms) }
                                        </p>
                                        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                                            { "Response time" }
                                        </p>
                                    </div>
                                </Card>

                                // Feedback Rate
                                <Card>
                                    <div class="p-6">
                                        <div class="flex items-center justify-between mb-2">
                                            <p class="text-sm font-medium text-gray-600 dark:text-gray-400">
                                                { "Feedback Rate" }
                                            </p>
                                            <svg class="w-5 h-5 text-orange-500" fill="currentColor" viewBox="0 0 20 20">
                                                <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                                            </svg>
                                        </div>
                                        <p class="text-3xl font-bold text-gray-900 dark:text-white">
                                            {
                                                if ov.total_searches > 0 {
                                                    format!("{:.1}%", (ov.total_feedback as f64 / ov.total_searches as f64) * 100.0)
                                                } else {
                                                    "0%".to_string()
                                                }
                                            }
                                        </p>
                                        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                                            { "Of searches" }
                                        </p>
                                    </div>
                                </Card>
                            </div>
                        }

                        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                            // Top Queries
                            <Card title="Top Queries">
                                if let Some(tq) = &*top_queries {
                                    if tq.queries.is_empty() {
                                        <div class="text-center py-8 text-gray-500 dark:text-gray-400">
                                            <p>{ "No queries yet" }</p>
                                        </div>
                                    } else {
                                        <div class="space-y-3">
                                            { for tq.queries.iter().enumerate().map(|(idx, query)| {
                                                let positive_pct = if query.count > 0 {
                                                    (query.positive_feedback as f64 / query.count as f64) * 100.0
                                                } else {
                                                    0.0
                                                };

                                                html! {
                                                    <div class="p-4 bg-gray-50 dark:bg-gray-800 rounded border border-gray-200 dark:border-gray-700">
                                                        <div class="flex items-start justify-between mb-2">
                                                            <div class="flex items-center gap-2 flex-1">
                                                                <span class="flex-shrink-0 w-6 h-6 rounded-full bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300 flex items-center justify-center text-xs font-bold">
                                                                    { idx + 1 }
                                                                </span>
                                                                <p class="font-medium text-gray-900 dark:text-white truncate">
                                                                    { &query.query }
                                                                </p>
                                                            </div>
                                                            <span class="flex-shrink-0 px-2 py-1 rounded bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 text-xs font-medium">
                                                                { format!("{}x", query.count) }
                                                            </span>
                                                        </div>

                                                        <div class="grid grid-cols-3 gap-4 text-xs">
                                                            <div>
                                                                <span class="text-gray-600 dark:text-gray-400">{ "Results:" }</span>
                                                                <span class="ml-1 font-medium text-gray-900 dark:text-white">
                                                                    { format!("{:.1}", query.avg_results) }
                                                                </span>
                                                            </div>
                                                            <div>
                                                                <span class="text-gray-600 dark:text-gray-400">{ "Latency:" }</span>
                                                                <span class="ml-1 font-medium text-gray-900 dark:text-white">
                                                                    { format!("{:.0}ms", query.avg_latency_ms) }
                                                                </span>
                                                            </div>
                                                            <div>
                                                                <span class="text-gray-600 dark:text-gray-400">{ "Positive:" }</span>
                                                                <span class="ml-1 font-medium text-green-600 dark:text-green-400">
                                                                    { format!("{:.0}%", positive_pct) }
                                                                </span>
                                                            </div>
                                                        </div>
                                                    </div>
                                                }
                                            }) }
                                        </div>
                                    }
                                }
                            </Card>

                            // Recent Searches
                            <Card title="Recent Searches">
                                if let Some(ov) = &*overview {
                                    if ov.recent_searches.is_empty() {
                                        <div class="text-center py-8 text-gray-500 dark:text-gray-400">
                                            <p>{ "No recent searches" }</p>
                                        </div>
                                    } else {
                                        <div class="space-y-2">
                                            { for ov.recent_searches.iter().map(|search| {
                                                let timestamp = search.timestamp.format("%Y-%m-%d %H:%M").to_string();

                                                html! {
                                                    <div class="p-3 bg-gray-50 dark:bg-gray-800 rounded border border-gray-200 dark:border-gray-700">
                                                        <div class="flex items-center justify-between mb-1">
                                                            <p class="text-sm font-medium text-gray-900 dark:text-white truncate">
                                                                { &search.query }
                                                            </p>
                                                            <span class="text-xs text-gray-500 dark:text-gray-400 ml-2">
                                                                { timestamp }
                                                            </span>
                                                        </div>
                                                        <div class="flex items-center gap-4 text-xs text-gray-600 dark:text-gray-400">
                                                            <span>{ format!("{} results", search.results_count) }</span>
                                                            <span>{ format!("{}ms", search.duration_ms) }</span>
                                                        </div>
                                                    </div>
                                                }
                                            }) }
                                        </div>
                                    }
                                }
                            </Card>
                        </div>

                        // Feedback Statistics
                        if let Some(fs) = &*feedback_stats {
                            <Card title="Feedback Statistics">
                                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                                    // Top Positive Results
                                    <div>
                                        <h4 class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-3">
                                            { "Top Positive Results" }
                                        </h4>
                                        if fs.top_positive.is_empty() {
                                            <p class="text-sm text-gray-500 dark:text-gray-400">{ "No positive feedback yet" }</p>
                                        } else {
                                            <div class="space-y-2">
                                                { for fs.top_positive.iter().take(5).map(|result| {
                                                    html! {
                                                        <div class="flex items-center justify-between p-2 bg-green-50 dark:bg-green-900/20 rounded">
                                                            <span class="text-sm text-gray-900 dark:text-white truncate">
                                                                { &result.result_id }
                                                            </span>
                                                            <span class="text-sm font-medium text-green-600 dark:text-green-400">
                                                                { format!("+{}", result.positive_count) }
                                                            </span>
                                                        </div>
                                                    }
                                                }) }
                                            </div>
                                        }
                                    </div>

                                    // Top Negative Results
                                    <div>
                                        <h4 class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-3">
                                            { "Top Negative Results" }
                                        </h4>
                                        if fs.top_negative.is_empty() {
                                            <p class="text-sm text-gray-500 dark:text-gray-400">{ "No negative feedback yet" }</p>
                                        } else {
                                            <div class="space-y-2">
                                                { for fs.top_negative.iter().take(5).map(|result| {
                                                    html! {
                                                        <div class="flex items-center justify-between p-2 bg-red-50 dark:bg-red-900/20 rounded">
                                                            <span class="text-sm text-gray-900 dark:text-white truncate">
                                                                { &result.result_id }
                                                            </span>
                                                            <span class="text-sm font-medium text-red-600 dark:text-red-400">
                                                                { format!("-{}", result.negative_count) }
                                                            </span>
                                                        </div>
                                                    }
                                                }) }
                                            </div>
                                        }
                                    </div>
                                </div>
                            </Card>
                        }
                    </>
                }
            </div>
        </div>
    }
}
