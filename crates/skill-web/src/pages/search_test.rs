//! Search Testing Page - Test semantic search with live results
//!
//! Features:
//! - Live semantic search with query input
//! - Results display with scores and metadata
//! - Configuration toggle (use current settings vs custom)
//! - Search statistics (latency, results count)

use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::api::{Api, SearchRequest, SearchResponse};
use crate::components::card::Card;
use crate::components::result_card::ResultCard;
use crate::components::use_notifications;

#[derive(Clone, PartialEq)]
struct SearchStats {
    total_results: usize,
    latency_ms: u64,
    query: String,
}

#[function_component(SearchTestPage)]
pub fn search_test_page() -> Html {
    // State
    let query = use_state(String::new);
    let results = use_state(|| None::<SearchResponse>);
    let is_searching = use_state(|| false);
    let search_stats = use_state(|| None::<SearchStats>);
    let top_k = use_state(|| 10_usize);
    let is_indexing = use_state(|| false);

    // API & notifications
    let api = use_memo((), |_| Rc::new(Api::new()));
    let notifications = use_notifications();

    // Search handler
    let on_search = {
        let api = api.clone();
        let query = query.clone();
        let results = results.clone();
        let is_searching = is_searching.clone();
        let search_stats = search_stats.clone();
        let top_k = top_k.clone();
        let notifications = notifications.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            let query_text = (*query).clone();
            if query_text.trim().is_empty() {
                return;
            }

            is_searching.set(true);
            results.set(None);
            search_stats.set(None);

            let api = api.clone();
            let results = results.clone();
            let is_searching = is_searching.clone();
            let search_stats = search_stats.clone();
            let notifications = notifications.clone();
            let top_k_val = *top_k;

            spawn_local(async move {
                let start = js_sys::Date::now();

                let request = SearchRequest {
                    query: query_text.clone(),
                    top_k: top_k_val,
                    skill_filter: None,
                    include_examples: false,
                };

                match api.search.search(&request).await {
                    Ok(response) => {
                        let end = js_sys::Date::now();
                        let elapsed = (end - start) as u64;
                        let total = response.results.len();

                        search_stats.set(Some(SearchStats {
                            total_results: total,
                            latency_ms: elapsed,
                            query: query_text.clone(),
                        }));

                        results.set(Some(response));

                        notifications.success(
                            "Search completed",
                            format!("Found {} results in {}ms", total, elapsed),
                        );
                    }
                    Err(e) => {
                        notifications.error("Search failed", format!("Error: {}", e));
                    }
                }
                is_searching.set(false);
            });
        })
    };

    // Input change handler
    let on_input_change = {
        let query = query.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            query.set(input.value());
        })
    };

    // Enter key handler
    let on_key_press = {
        let api = api.clone();
        let query = query.clone();
        let results = results.clone();
        let is_searching = is_searching.clone();
        let search_stats = search_stats.clone();
        let top_k = top_k.clone();
        let notifications = notifications.clone();

        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                let query_text = (*query).clone();
                if query_text.trim().is_empty() {
                    return;
                }

                is_searching.set(true);
                results.set(None);
                search_stats.set(None);

                let api = api.clone();
                let results = results.clone();
                let is_searching = is_searching.clone();
                let search_stats = search_stats.clone();
                let notifications = notifications.clone();
                let top_k_val = *top_k;

                spawn_local(async move {
                    let start = js_sys::Date::now();

                    let request = SearchRequest {
                        query: query_text.clone(),
                        top_k: top_k_val,
                        skill_filter: None,
                        include_examples: false,
                    };

                    match api.search.search(&request).await {
                        Ok(response) => {
                            let end = js_sys::Date::now();
                            let elapsed = (end - start) as u64;
                            let total = response.results.len();

                            search_stats.set(Some(SearchStats {
                                total_results: total,
                                latency_ms: elapsed,
                                query: query_text.clone(),
                            }));

                            results.set(Some(response));

                            notifications.success(
                                "Search completed",
                                format!("Found {} results in {}ms", total, elapsed),
                            );
                        }
                        Err(e) => {
                            notifications.error("Search failed", format!("Error: {}", e));
                        }
                    }
                    is_searching.set(false);
                });
            }
        })
    };

    // Top K change handler
    let on_top_k_change = {
        let top_k = top_k.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            if let Ok(value) = select.value().parse::<usize>() {
                top_k.set(value);
            }
        })
    };

    // Index handler
    let on_index = {
        let api = api.clone();
        let is_indexing = is_indexing.clone();
        let notifications = notifications.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            is_indexing.set(true);

            let api = api.clone();
            let is_indexing = is_indexing.clone();
            let notifications = notifications.clone();

            spawn_local(async move {
                let start = js_sys::Date::now();

                match api.search.index().await {
                    Ok(response) => {
                        let end = js_sys::Date::now();
                        let elapsed = (end - start) as u64;

                        notifications.success(
                            "Indexing completed",
                            format!(
                                "Indexed {} documents in {}ms (server: {}ms)",
                                response.documents_indexed, elapsed, response.duration_ms
                            ),
                        );
                    }
                    Err(e) => {
                        notifications.error("Indexing failed", format!("Error: {}", e));
                    }
                }
                is_indexing.set(false);
            });
        })
    };

    html! {
        <div class="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
            <div class="max-w-6xl mx-auto space-y-6">
                // Header
                <div class="flex items-start justify-between">
                    <div>
                        <h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-1">
                            { "Semantic Search Testing" }
                        </h1>
                        <p class="text-sm text-gray-600 dark:text-gray-400">
                            { "Test semantic search with live results from your skill catalog" }
                        </p>
                    </div>
                    <button
                        class="btn btn-secondary flex items-center gap-2"
                        onclick={on_index}
                        disabled={*is_indexing}
                    >
                        if *is_indexing {
                            <span class="animate-spin">{ "⟳" }</span>
                            { "Indexing..." }
                        } else {
                            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                            </svg>
                            { "Re-index Skills" }
                        }
                    </button>
                </div>

                // Search Input Card
                <Card title="Search Query">
                    <div class="space-y-4">
                        <div class="flex gap-2">
                            <input
                                type="text"
                                class="input flex-1"
                                placeholder="Enter search query (e.g., 'list kubernetes pods')"
                                value={(*query).clone()}
                                oninput={on_input_change}
                                onkeypress={on_key_press}
                            />
                            <select
                                class="input w-24"
                                value={top_k.to_string()}
                                onchange={on_top_k_change}
                            >
                                <option value="5">{ "Top 5" }</option>
                                <option value="10" selected={*top_k == 10}>{ "Top 10" }</option>
                                <option value="20">{ "Top 20" }</option>
                                <option value="50">{ "Top 50" }</option>
                            </select>
                            <button
                                class="btn btn-primary px-6"
                                onclick={on_search}
                                disabled={*is_searching || query.trim().is_empty()}
                            >
                                if *is_searching {
                                    <span class="flex items-center gap-2">
                                        <span class="animate-spin">{ "⟳" }</span>
                                        { "Searching..." }
                                    </span>
                                } else {
                                    { "Search" }
                                }
                            </button>
                        </div>

                        // Search statistics
                        if let Some(stats) = &*search_stats {
                            <div class="p-3 bg-blue-50 dark:bg-blue-900/20 rounded border border-blue-200 dark:border-blue-800">
                                <div class="grid grid-cols-3 gap-4 text-sm">
                                    <div>
                                        <span class="text-gray-600 dark:text-gray-400">{ "Query:" }</span>
                                        <span class="ml-2 font-medium text-gray-900 dark:text-white">
                                            { &stats.query }
                                        </span>
                                    </div>
                                    <div>
                                        <span class="text-gray-600 dark:text-gray-400">{ "Results:" }</span>
                                        <span class="ml-2 font-medium text-gray-900 dark:text-white">
                                            { stats.total_results }
                                        </span>
                                    </div>
                                    <div>
                                        <span class="text-gray-600 dark:text-gray-400">{ "Latency:" }</span>
                                        <span class="ml-2 font-medium text-gray-900 dark:text-white">
                                            { format!("{}ms", stats.latency_ms) }
                                        </span>
                                    </div>
                                </div>
                            </div>
                        }
                    </div>
                </Card>

                // Results Card
                <Card title="Search Results">
                    if let Some(response) = &*results {
                        if response.results.is_empty() {
                            <div class="text-center py-8 text-gray-500 dark:text-gray-400">
                                <svg class="w-12 h-12 mx-auto mb-3 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                                </svg>
                                <p>{ "No results found for your query." }</p>
                            </div>
                        } else {
                            <div class="space-y-3">
                                { for response.results.iter().enumerate().map(|(idx, result)| {
                                    html! {
                                        <ResultCard
                                            index={idx + 1}
                                            id={format!("{}:{}", result.skill, result.tool)}
                                            skill={result.skill.clone()}
                                            tool={result.tool.clone()}
                                            content={result.content.clone()}
                                            score={result.score}
                                            rerank_score={None}
                                            query={(*query).clone()}
                                        />
                                    }
                                }) }
                            </div>
                        }
                    } else if *is_searching {
                        <div class="flex items-center justify-center py-8">
                            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-500"></div>
                        </div>
                    } else {
                        <div class="text-center py-8 text-gray-500 dark:text-gray-400">
                            <svg class="w-12 h-12 mx-auto mb-3 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                            </svg>
                            <p>{ "No results yet. Enter a query and click Search." }</p>
                        </div>
                    }
                </Card>
            </div>
        </div>
    }
}
