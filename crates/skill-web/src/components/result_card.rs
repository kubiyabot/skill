//! Search Result Card Component - Expandable card with feedback buttons

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::api::{Api, SubmitFeedbackRequest};
use crate::components::use_notifications;

#[derive(Properties, Clone, PartialEq)]
pub struct ResultCardProps {
    /// Result index (1-based)
    pub index: usize,
    /// Result ID (skill:tool)
    pub id: String,
    /// Skill name
    pub skill: String,
    /// Tool name
    pub tool: String,
    /// Content/description
    pub content: String,
    /// Relevance score
    pub score: f32,
    /// Optional rerank score
    pub rerank_score: Option<f32>,
    /// Search query that produced this result
    pub query: String,
}

#[function_component(ResultCard)]
pub fn result_card(props: &ResultCardProps) -> Html {
    let expanded = use_state(|| false);
    let feedback_submitted = use_state(|| None::<String>); // "positive" or "negative"
    let is_submitting = use_state(|| false);

    // API & notifications
    let api = use_memo((), |_| std::rc::Rc::new(Api::new()));
    let notifications = use_notifications();

    // Toggle expand/collapse
    let on_toggle = {
        let expanded = expanded.clone();
        Callback::from(move |_: web_sys::MouseEvent| {
            expanded.set(!*expanded);
        })
    };

    // Submit feedback handler
    let submit_feedback = {
        let props = props.clone();
        let api = api.clone();
        let feedback_submitted = feedback_submitted.clone();
        let is_submitting = is_submitting.clone();
        let notifications = notifications.clone();

        move |feedback_type: String| {
            if *is_submitting {
                return;
            }

            is_submitting.set(true);

            let api = api.clone();
            let feedback_submitted = feedback_submitted.clone();
            let is_submitting = is_submitting.clone();
            let notifications = notifications.clone();
            let props = props.clone();
            let _feedback_type_clone = feedback_type.clone();

            spawn_local(async move {
                let request = SubmitFeedbackRequest {
                    query: props.query,
                    result_id: props.id.clone(),
                    score: props.score,
                    rank: props.index - 1, // Convert to 0-based
                    feedback_type: feedback_type.clone(),
                    reason: None,
                    comment: None,
                    client_type: "http".to_string(),
                };

                match api.feedback.submit(&request).await {
                    Ok(_) => {
                        feedback_submitted.set(Some(feedback_type.clone()));
                        notifications.success(
                            "Feedback submitted",
                            format!("Thank you for your feedback on {}", props.tool),
                        );
                    }
                    Err(e) => {
                        notifications.error("Failed to submit feedback", format!("Error: {}", e));
                    }
                }

                is_submitting.set(false);
            });
        }
    };

    // Feedback button handlers
    let on_thumbs_up = {
        let submit_feedback = submit_feedback.clone();
        Callback::from(move |e: web_sys::MouseEvent| {
            e.stop_propagation();
            submit_feedback("positive".to_string());
        })
    };

    let on_thumbs_down = {
        let submit_feedback = submit_feedback.clone();
        Callback::from(move |e: web_sys::MouseEvent| {
            e.stop_propagation();
            submit_feedback("negative".to_string());
        })
    };

    // Determine card classes based on feedback
    let card_border_class = match &*feedback_submitted {
        Some(ft) if ft == "positive" => "border-green-500 dark:border-green-400",
        Some(ft) if ft == "negative" => "border-red-500 dark:border-red-400",
        _ => "border-gray-200 dark:border-gray-700 hover:border-primary-500 dark:hover:border-primary-400",
    };

    html! {
        <div class={classes!(
            "p-4", "bg-white", "dark:bg-gray-800", "rounded-lg", "border-2",
            "transition-all", "duration-200", "cursor-pointer",
            card_border_class
        )}
        onclick={on_toggle.clone()}
        >
            // Header
            <div class="flex items-start justify-between mb-3">
                <div class="flex items-center gap-3 flex-1">
                    // Rank badge
                    <span class="flex-shrink-0 w-8 h-8 rounded-full bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300 flex items-center justify-center text-sm font-bold">
                        { props.index }
                    </span>

                    // Title
                    <div class="flex-1 min-w-0">
                        <h4 class="font-semibold text-gray-900 dark:text-white truncate">
                            { &props.skill }
                        </h4>
                        <p class="text-sm text-gray-600 dark:text-gray-400 truncate">
                            { &props.tool }
                        </p>
                    </div>
                </div>

                // Right side: Scores and buttons
                <div class="flex items-center gap-2 ml-4">
                    // Score badge
                    <span class="px-2 py-1 rounded bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 text-xs font-mono whitespace-nowrap">
                        { format!("{:.3}", props.score) }
                    </span>

                    // Feedback buttons
                    <div class="flex items-center gap-1">
                        <button
                            class={classes!(
                                "p-1.5", "rounded", "transition-colors",
                                if feedback_submitted.as_ref() == Some(&"positive".to_string()) {
                                    "bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300"
                                } else {
                                    "hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400"
                                }
                            )}
                            onclick={on_thumbs_up}
                            disabled={*is_submitting || feedback_submitted.is_some()}
                            title="This result was helpful"
                        >
                            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                                <path d="M2 10.5a1.5 1.5 0 113 0v6a1.5 1.5 0 01-3 0v-6zM6 10.333v5.43a2 2 0 001.106 1.79l.05.025A4 4 0 008.943 18h5.416a2 2 0 001.962-1.608l1.2-6A2 2 0 0015.56 8H12V4a2 2 0 00-2-2 1 1 0 00-1 1v.667a4 4 0 01-.8 2.4L6.8 7.933a4 4 0 00-.8 2.4z" />
                            </svg>
                        </button>

                        <button
                            class={classes!(
                                "p-1.5", "rounded", "transition-colors",
                                if feedback_submitted.as_ref() == Some(&"negative".to_string()) {
                                    "bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300"
                                } else {
                                    "hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400"
                                }
                            )}
                            onclick={on_thumbs_down}
                            disabled={*is_submitting || feedback_submitted.is_some()}
                            title="This result was not helpful"
                        >
                            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                                <path d="M18 9.5a1.5 1.5 0 11-3 0v-6a1.5 1.5 0 013 0v6zM14 9.667v-5.43a2 2 0 00-1.105-1.79l-.05-.025A4 4 0 0011.055 2H5.64a2 2 0 00-1.962 1.608l-1.2 6A2 2 0 004.44 12H8v4a2 2 0 002 2 1 1 0 001-1v-.667a4 4 0 01.8-2.4l1.4-1.866a4 4 0 00.8-2.4z" />
                            </svg>
                        </button>
                    </div>

                    // Expand icon
                    <svg
                        class={classes!(
                            "w-5", "h-5", "text-gray-400", "transition-transform", "duration-200",
                            if *expanded { "rotate-180" } else { "" }
                        )}
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                    >
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                    </svg>
                </div>
            </div>

            // Content preview
            <p class={classes!(
                "text-sm", "text-gray-700", "dark:text-gray-300",
                if !*expanded { "line-clamp-2" } else { "" }
            )}>
                { &props.content }
            </p>

            // Expanded details
            if *expanded {
                <div class="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700 space-y-3"
                     onclick={|e: web_sys::MouseEvent| e.stop_propagation()}
                >
                    // Metadata
                    <div class="grid grid-cols-2 gap-4 text-sm">
                        <div>
                            <span class="text-gray-600 dark:text-gray-400">{ "Skill:" }</span>
                            <span class="ml-2 font-medium text-gray-900 dark:text-white">
                                { &props.skill }
                            </span>
                        </div>
                        <div>
                            <span class="text-gray-600 dark:text-gray-400">{ "Tool:" }</span>
                            <span class="ml-2 font-medium text-gray-900 dark:text-white">
                                { &props.tool }
                            </span>
                        </div>
                        <div>
                            <span class="text-gray-600 dark:text-gray-400">{ "Score:" }</span>
                            <span class="ml-2 font-mono text-gray-900 dark:text-white">
                                { format!("{:.6}", props.score) }
                            </span>
                        </div>
                        if let Some(rerank_score) = props.rerank_score {
                            <div>
                                <span class="text-gray-600 dark:text-gray-400">{ "Rerank Score:" }</span>
                                <span class="ml-2 font-mono text-gray-900 dark:text-white">
                                    { format!("{:.6}", rerank_score) }
                                </span>
                            </div>
                        }
                    </div>

                    // Full description
                    <div class="p-3 bg-gray-50 dark:bg-gray-900 rounded text-sm">
                        <p class="text-gray-700 dark:text-gray-300 whitespace-pre-wrap">
                            { &props.content }
                        </p>
                    </div>

                    // Action hint
                    <div class="text-xs text-gray-500 dark:text-gray-400 italic">
                        { "Click the card header to collapse" }
                    </div>
                </div>
            }

            // Feedback confirmation
            if let Some(feedback_type) = &*feedback_submitted {
                <div class={classes!(
                    "mt-3", "p-2", "rounded", "text-xs", "flex", "items-center", "gap-2",
                    if feedback_type == "positive" {
                        "bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-300"
                    } else {
                        "bg-red-50 dark:bg-red-900/20 text-red-700 dark:text-red-300"
                    }
                )}>
                    <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
                    </svg>
                    { format!("Feedback recorded: {}", if feedback_type == "positive" { "Helpful" } else { "Not helpful" }) }
                </div>
            }
        </div>
    }
}
