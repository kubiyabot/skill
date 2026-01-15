//! Settings page with full API integration
//!
//! Provides configuration for:
//! - Appearance (theme)
//! - Execution defaults (timeout, metadata, history)
//! - Search pipeline (embedding provider, vector store, hybrid search, reranking)
//! - Data management (import/export, clear history)

use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::api::{
    Api, AppConfig, SearchConfigResponse, UpdateSearchConfigRequest,
};
use crate::components::card::Card;
use crate::components::{use_import_config_modal, use_notifications, ImportConfigModal, Tooltip};
use crate::store::ui::{UiAction, UiStore};

/// Settings state
#[derive(Clone, PartialEq)]
struct SettingsState {
    // Appearance
    theme: String,
    // Execution
    default_timeout_secs: u64,
    max_concurrent_executions: usize,
    include_metadata: bool,
    enable_history: bool,
    max_history_entries: usize,
    // Search
    embedding_provider: String,
    embedding_model: String,
    vector_backend: String,
    ollama_url: Option<String>,
    qdrant_url: Option<String>,
    hybrid_search_enabled: bool,
    reranking_enabled: bool,
    indexed_documents: usize,
    // Advanced embedding model override
    use_advanced_model: bool,
    // Agent
    agent_runtime: String,
    agent_provider: String,
    agent_model: String,
    agent_temperature: f32,
    agent_max_tokens: usize,
    agent_timeout_secs: u64,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            default_timeout_secs: 30,
            max_concurrent_executions: 10,
            include_metadata: false,
            enable_history: true,
            max_history_entries: 1000,
            embedding_provider: "fastembed".to_string(),
            embedding_model: "all-minilm".to_string(),
            vector_backend: "file".to_string(),
            ollama_url: Some("http://localhost:11434".to_string()),
            qdrant_url: Some("http://localhost:6333".to_string()),
            hybrid_search_enabled: true,
            reranking_enabled: false,
            indexed_documents: 0,
            use_advanced_model: false,
            agent_runtime: "claude-code".to_string(),
            agent_provider: "anthropic".to_string(),
            agent_model: "claude-sonnet-4".to_string(),
            agent_temperature: 0.7,
            agent_max_tokens: 4096,
            agent_timeout_secs: 300,
        }
    }
}

impl SettingsState {
    fn from_config(config: &AppConfig) -> Self {
        // Check if model is a non-standard value (advanced)
        let model = &config.search.embedding_model;
        let provider = &config.search.embedding_provider;
        let is_standard_model = Self::is_standard_model(provider, model);

        Self {
            theme: "system".to_string(),
            default_timeout_secs: config.default_timeout_secs,
            max_concurrent_executions: config.max_concurrent_executions,
            include_metadata: false,
            enable_history: config.enable_history,
            max_history_entries: config.max_history_entries,
            embedding_provider: provider.clone(),
            embedding_model: model.clone(),
            vector_backend: config.search.vector_backend.clone(),
            ollama_url: Some("http://localhost:11434".to_string()),
            qdrant_url: Some("http://localhost:6333".to_string()),
            hybrid_search_enabled: config.search.hybrid_search_enabled,
            reranking_enabled: config.search.reranking_enabled,
            indexed_documents: config.search.indexed_documents,
            use_advanced_model: !is_standard_model,
            // Agent defaults (not user-configurable anymore)
            agent_runtime: "claude-code".to_string(),
            agent_provider: "anthropic".to_string(),
            agent_model: "claude-sonnet-4".to_string(),
            agent_temperature: 0.7,
            agent_max_tokens: 4096,
            agent_timeout_secs: 300,
        }
    }

    fn is_standard_model(provider: &str, model: &str) -> bool {
        match provider {
            "fastembed" => matches!(model, "all-minilm" | "bge-small" | "bge-base" | "bge-large"),
            "openai" => matches!(model, "text-embedding-ada-002" | "text-embedding-3-small" | "text-embedding-3-large"),
            "ollama" => matches!(model, "nomic-embed-text" | "mxbai-embed-large" | "all-minilm"),
            _ => false,
        }
    }

    fn get_default_model(provider: &str) -> &'static str {
        match provider {
            "fastembed" => "all-minilm",
            "openai" => "text-embedding-3-small",
            "ollama" => "nomic-embed-text",
            _ => "all-minilm",
        }
    }
}

/// Test result for vector DB testing
#[derive(Clone, PartialEq)]
struct TestResult {
    success: bool,
    message: String,
    duration_ms: u128,
    details: Option<String>,
}

/// Settings page component
#[function_component(SettingsPage)]
pub fn settings_page() -> Html {
    let (_, ui_dispatch) = use_store::<UiStore>();
    let notifications = use_notifications();
    let import_modal = use_import_config_modal();

    // API client
    let api = use_memo((), |_| Rc::new(Api::new()));

    // State
    let settings = use_state(SettingsState::default);
    let loading = use_state(|| true);
    let saving = use_state(|| false);
    let error = use_state(|| Option::<String>::None);
    let has_changes = use_state(|| false);

    // Vector DB testing state
    let test_connection_loading = use_state(|| false);
    let test_pipeline_loading = use_state(|| false);
    let test_result = use_state(|| Option::<TestResult>::None);

    // Load settings on mount
    {
        let api = api.clone();
        let settings = settings.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                match api.config.get().await {
                    Ok(config) => {
                        settings.set(SettingsState::from_config(&config));
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(e.to_string()));
                        loading.set(false);
                    }
                }
            });
        });
    }

    // Theme change handler
    let on_theme_change = {
        let settings = settings.clone();
        let has_changes = has_changes.clone();
        let ui_dispatch = ui_dispatch.clone();
        Callback::from(move |value: String| {
            let mut new_settings = (*settings).clone();
            new_settings.theme = value.clone();
            settings.set(new_settings);
            has_changes.set(true);
            // Apply theme immediately
            ui_dispatch.apply(UiAction::SetDarkMode(value == "dark"));
        })
    };

    // Timeout change handler
    let on_timeout_change = {
        let settings = settings.clone();
        let has_changes = has_changes.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            if let Ok(value) = input.value().parse::<u64>() {
                let mut new_settings = (*settings).clone();
                new_settings.default_timeout_secs = value;
                settings.set(new_settings);
                has_changes.set(true);
            }
        })
    };

    // Max concurrent change handler
    let on_max_concurrent_change = {
        let settings = settings.clone();
        let has_changes = has_changes.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            if let Ok(value) = input.value().parse::<usize>() {
                let mut new_settings = (*settings).clone();
                new_settings.max_concurrent_executions = value;
                settings.set(new_settings);
                has_changes.set(true);
            }
        })
    };

    // History entries change handler
    let on_history_entries_change = {
        let settings = settings.clone();
        let has_changes = has_changes.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            if let Ok(value) = input.value().parse::<usize>() {
                let mut new_settings = (*settings).clone();
                new_settings.max_history_entries = value;
                settings.set(new_settings);
                has_changes.set(true);
            }
        })
    };

    // Enable history toggle
    let on_enable_history_toggle = {
        let settings = settings.clone();
        let has_changes = has_changes.clone();
        Callback::from(move |_: MouseEvent| {
            let mut new_settings = (*settings).clone();
            new_settings.enable_history = !new_settings.enable_history;
            settings.set(new_settings);
            has_changes.set(true);
        })
    };

    // Include metadata toggle
    let on_include_metadata_toggle = {
        let settings = settings.clone();
        let has_changes = has_changes.clone();
        Callback::from(move |_: MouseEvent| {
            let mut new_settings = (*settings).clone();
            new_settings.include_metadata = !new_settings.include_metadata;
            settings.set(new_settings);
            has_changes.set(true);
        })
    };

    // Embedding provider change
    let on_embedding_provider_change = {
        let settings = settings.clone();
        let has_changes = has_changes.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let mut new_settings = (*settings).clone();
            let new_provider = select.value();
            new_settings.embedding_provider = new_provider.clone();

            // Auto-set default model for new provider if not in advanced mode
            if !new_settings.use_advanced_model {
                new_settings.embedding_model = SettingsState::get_default_model(&new_provider).to_string();
            }

            settings.set(new_settings);
            has_changes.set(true);
        })
    };

    // Vector backend change
    let on_vector_backend_change = {
        let settings = settings.clone();
        let has_changes = has_changes.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let mut new_settings = (*settings).clone();
            new_settings.vector_backend = select.value();
            settings.set(new_settings);
            has_changes.set(true);
        })
    };

    // Embedding model dropdown change (for standard models)
    let on_embedding_model_select = {
        let settings = settings.clone();
        let has_changes = has_changes.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let mut new_settings = (*settings).clone();
            new_settings.embedding_model = select.value();
            settings.set(new_settings);
            has_changes.set(true);
        })
    };

    // Embedding model text input change (for advanced/custom models)
    let on_embedding_model_change = {
        let settings = settings.clone();
        let has_changes = has_changes.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let mut new_settings = (*settings).clone();
            new_settings.embedding_model = input.value();
            settings.set(new_settings);
            has_changes.set(true);
        })
    };

    // Toggle advanced model mode
    let on_advanced_model_toggle = {
        let settings = settings.clone();
        let has_changes = has_changes.clone();
        Callback::from(move |_: MouseEvent| {
            let mut new_settings = (*settings).clone();
            new_settings.use_advanced_model = !new_settings.use_advanced_model;

            // If switching to standard mode, set default model for current provider
            if !new_settings.use_advanced_model {
                new_settings.embedding_model = SettingsState::get_default_model(&new_settings.embedding_provider).to_string();
            }

            settings.set(new_settings);
            has_changes.set(true);
        })
    };

    // Ollama URL change
    let on_ollama_url_change = {
        let settings = settings.clone();
        let has_changes = has_changes.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let mut new_settings = (*settings).clone();
            new_settings.ollama_url = Some(input.value());
            settings.set(new_settings);
            has_changes.set(true);
        })
    };

    // Qdrant URL change
    let on_qdrant_url_change = {
        let settings = settings.clone();
        let has_changes = has_changes.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let mut new_settings = (*settings).clone();
            new_settings.qdrant_url = Some(input.value());
            settings.set(new_settings);
            has_changes.set(true);
        })
    };

    // Hybrid search toggle
    let on_hybrid_toggle = {
        let settings = settings.clone();
        let has_changes = has_changes.clone();
        Callback::from(move |_: MouseEvent| {
            let mut new_settings = (*settings).clone();
            new_settings.hybrid_search_enabled = !new_settings.hybrid_search_enabled;
            settings.set(new_settings);
            has_changes.set(true);
        })
    };

    // Reranking toggle
    let on_reranking_toggle = {
        let settings = settings.clone();
        let has_changes = has_changes.clone();
        Callback::from(move |_: MouseEvent| {
            let mut new_settings = (*settings).clone();
            new_settings.reranking_enabled = !new_settings.reranking_enabled;
            settings.set(new_settings);
            has_changes.set(true);
        })
    };

    // Save changes handler
    let on_save = {
        let api = api.clone();
        let settings = settings.clone();
        let saving = saving.clone();
        let has_changes = has_changes.clone();
        let notifications = notifications.clone();

        Callback::from(move |_: MouseEvent| {
            let current_settings = (*settings).clone();
            saving.set(true);

            let api = api.clone();
            let saving = saving.clone();
            let has_changes = has_changes.clone();
            let notifications = notifications.clone();

            spawn_local(async move {
                // Update app config
                let app_result = api
                    .config
                    .update(&crate::api::UpdateAppConfigRequest {
                        default_timeout_secs: Some(current_settings.default_timeout_secs),
                        max_concurrent_executions: Some(current_settings.max_concurrent_executions),
                        enable_history: Some(current_settings.enable_history),
                        max_history_entries: Some(current_settings.max_history_entries),
                    })
                    .await;

                // Update search config
                let search_result = api
                    .config
                    .update_search_config(&UpdateSearchConfigRequest {
                        embedding_provider: Some(current_settings.embedding_provider),
                        embedding_model: None,
                        vector_backend: Some(current_settings.vector_backend),
                        enable_hybrid: Some(current_settings.hybrid_search_enabled),
                        enable_reranking: Some(current_settings.reranking_enabled),
                    })
                    .await;

                saving.set(false);

                match (app_result, search_result) {
                    (Ok(_), Ok(_)) => {
                        has_changes.set(false);
                        notifications.success("Settings Saved", "Your settings have been updated");
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        notifications.error("Save Failed", &e.to_string());
                    }
                }
            });
        })
    };

    // Reset to defaults handler
    let on_reset = {
        let settings = settings.clone();
        let has_changes = has_changes.clone();
        let notifications = notifications.clone();

        Callback::from(move |_: MouseEvent| {
            settings.set(SettingsState {
                theme: "system".to_string(),
                default_timeout_secs: 30,
                max_concurrent_executions: 4,
                include_metadata: false,
                enable_history: true,
                max_history_entries: 1000,
                embedding_provider: "fastembed".to_string(),
                embedding_model: "all-minilm".to_string(),
                vector_backend: "file".to_string(),
                ollama_url: Some("http://localhost:11434".to_string()),
                qdrant_url: Some("http://localhost:6333".to_string()),
                hybrid_search_enabled: false,
                reranking_enabled: false,
                indexed_documents: 0,
                use_advanced_model: false,
                // Agent defaults
                agent_runtime: "claude-code".to_string(),
                agent_provider: "anthropic".to_string(),
                agent_model: "claude-sonnet-4".to_string(),
                agent_temperature: 0.7,
                agent_max_tokens: 4096,
                agent_timeout_secs: 300,
            });
            has_changes.set(true);
            notifications.info("Settings Reset", "Settings have been reset to defaults");
        })
    };

    // Test connection handler (quick validation)
    let on_test_connection = {
        let api = api.clone();
        let settings = settings.clone();
        let test_connection_loading = test_connection_loading.clone();
        let test_result = test_result.clone();
        let notifications = notifications.clone();

        Callback::from(move |_: MouseEvent| {
            test_connection_loading.set(true);
            test_result.set(None);

            let api = api.clone();
            let settings = (*settings).clone();
            let test_connection_loading = test_connection_loading.clone();
            let test_result = test_result.clone();
            let notifications = notifications.clone();

            spawn_local(async move {
                let request = crate::api::TestConnectionRequest {
                    embedding_provider: settings.embedding_provider,
                    embedding_model: settings.embedding_model,
                    vector_backend: settings.vector_backend,
                    qdrant_url: settings.qdrant_url.clone(),
                    ollama_url: settings.ollama_url.clone(),
                };

                match api.search.test_connection(&request).await {
                    Ok(response) => {
                        let details = format!(
                            "Embedding: {} | Backend: {}",
                            if response.embedding_provider_status.healthy { "✓" } else { "✗" },
                            if response.vector_backend_status.healthy { "✓" } else { "✗" }
                        );

                        let success = response.success;
                        let duration_ms = response.duration_ms;
                        let message = response.message.clone();

                        test_result.set(Some(TestResult {
                            success,
                            message: message.clone(),
                            duration_ms,
                            details: Some(details),
                        }));

                        if success {
                            notifications.success(
                                "Connection Test Passed",
                                &format!("All components healthy ({}ms)", duration_ms)
                            );
                        } else {
                            notifications.error("Connection Test Failed", &message);
                        }
                    }
                    Err(e) => {
                        notifications.error("Test Failed", &format!("Error: {}", e));
                        test_result.set(Some(TestResult {
                            success: false,
                            message: format!("Error: {}", e),
                            duration_ms: 0,
                            details: None,
                        }));
                    }
                }
                test_connection_loading.set(false);
            });
        })
    };

    // Test pipeline handler (full test with indexing)
    let on_test_pipeline = {
        let api = api.clone();
        let settings = settings.clone();
        let test_pipeline_loading = test_pipeline_loading.clone();
        let test_result = test_result.clone();
        let notifications = notifications.clone();

        Callback::from(move |_: MouseEvent| {
            test_pipeline_loading.set(true);
            test_result.set(None);

            let api = api.clone();
            let settings = (*settings).clone();
            let test_pipeline_loading = test_pipeline_loading.clone();
            let test_result = test_result.clone();
            let notifications = notifications.clone();

            spawn_local(async move {
                let request = crate::api::TestPipelineRequest {
                    embedding_provider: settings.embedding_provider,
                    embedding_model: settings.embedding_model,
                    vector_backend: settings.vector_backend,
                    enable_hybrid: settings.hybrid_search_enabled,
                    enable_reranking: settings.reranking_enabled,
                    qdrant_url: settings.qdrant_url.clone(),
                };

                match api.search.test_pipeline(&request).await {
                    Ok(response) => {
                        let details = format!(
                            "Indexed {} docs | Found {} results",
                            response.index_stats.documents_indexed,
                            response.search_results.len()
                        );

                        let success = response.success;
                        let duration_ms = response.duration_ms;
                        let message = response.message.clone();

                        test_result.set(Some(TestResult {
                            success,
                            message: message.clone(),
                            duration_ms,
                            details: Some(details),
                        }));

                        if success {
                            notifications.success(
                                "Pipeline Test Passed",
                                &format!("Pipeline working correctly ({}ms)", duration_ms)
                            );
                        } else {
                            notifications.error("Pipeline Test Failed", &message);
                        }
                    }
                    Err(e) => {
                        notifications.error("Test Failed", &format!("Error: {}", e));
                        test_result.set(Some(TestResult {
                            success: false,
                            message: format!("Error: {}", e),
                            duration_ms: 0,
                            details: None,
                        }));
                    }
                }
                test_pipeline_loading.set(false);
            });
        })
    };

    // Import config handler
    let on_import_click = {
        let import_modal = import_modal.clone();
        Callback::from(move |_: MouseEvent| {
            import_modal.open();
        })
    };

    // Reload settings after import
    let on_config_imported = {
        let api = api.clone();
        let settings = settings.clone();
        let notifications = notifications.clone();
        Callback::from(move |count: usize| {
            let api = api.clone();
            let settings = settings.clone();
            spawn_local(async move {
                if let Ok(config) = api.config.get().await {
                    settings.set(SettingsState::from_config(&config));
                }
            });
        })
    };

    // Clear history handler
    let on_clear_history = {
        let api = api.clone();
        let notifications = notifications.clone();
        Callback::from(move |_: MouseEvent| {
            let api = api.clone();
            let notifications = notifications.clone();

            // Show confirmation
            if !web_sys::window()
                .and_then(|w| w.confirm_with_message("Are you sure you want to clear all execution history? This cannot be undone.").ok())
                .unwrap_or(false)
            {
                return;
            }

            spawn_local(async move {
                match api.executions.clear_history().await {
                    Ok(_) => {
                        notifications.success("History Cleared", "Execution history cleared successfully");
                    }
                    Err(e) => {
                        notifications.error("Clear Failed", &format!("Failed to clear history: {}", e));
                    }
                }
            });
        })
    };

    // Current settings
    let current = (*settings).clone();

    html! {
        <div class="space-y-6 animate-fade-in">
            // Import Config Modal
            <ImportConfigModal on_imported={on_config_imported} />

            // Page header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">
                        { "Settings" }
                    </h1>
                    <p class="text-gray-500 dark:text-gray-400 mt-1">
                        { "Configure your Skill Engine preferences" }
                    </p>
                </div>
                if *has_changes {
                    <div class="flex items-center gap-2 px-3 py-1 bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-300 rounded-full text-sm">
                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                        </svg>
                        { "Unsaved changes" }
                    </div>
                }
            </div>

            // Error alert
            if let Some(err) = (*error).clone() {
                <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
                    <div class="flex items-center gap-3">
                        <svg class="w-5 h-5 text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                        <p class="text-sm text-red-700 dark:text-red-300">{ err }</p>
                    </div>
                </div>
            }

            // Loading state
            if *loading {
                <div class="space-y-6">
                    { for (0..4).map(|_| html! { <SettingsCardSkeleton /> }) }
                </div>
            } else {
                // Appearance settings
                <Card title="Appearance">
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                { "Theme" }
                            </label>
                            <div class="flex gap-4">
                                { for ["light", "dark", "system"].iter().map(|t| {
                                    let is_selected = current.theme == *t;
                                    let on_change = on_theme_change.clone();
                                    let value = t.to_string();

                                    html! {
                                        <button
                                            onclick={Callback::from(move |_: MouseEvent| on_change.emit(value.clone()))}
                                            class={classes!(
                                                "flex", "items-center", "gap-2", "px-4", "py-2", "rounded-lg", "border", "cursor-pointer", "transition-colors",
                                                if is_selected {
                                                    "border-primary-500 bg-primary-50 dark:bg-primary-900/30"
                                                } else {
                                                    "border-gray-200 dark:border-gray-700 hover:border-gray-300"
                                                }
                                            )}
                                        >
                                            <span class="capitalize">{ *t }</span>
                                        </button>
                                    }
                                }) }
                            </div>
                        </div>
                    </div>
                </Card>

                // Execution settings
                <Card title="Execution">
                    <div class="space-y-4">
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                    { "Default timeout (seconds)" }
                                </label>
                                <input
                                    type="number"
                                    class="input w-full"
                                    value={current.default_timeout_secs.to_string()}
                                    min="1"
                                    max="300"
                                    oninput={on_timeout_change}
                                />
                                <p class="text-xs text-gray-500 mt-1">
                                    { "Maximum time a skill can run (1-300 seconds)" }
                                </p>
                            </div>

                            <div>
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                    { "Max concurrent executions" }
                                </label>
                                <input
                                    type="number"
                                    class="input w-full"
                                    value={current.max_concurrent_executions.to_string()}
                                    min="1"
                                    max="16"
                                    oninput={on_max_concurrent_change}
                                />
                                <p class="text-xs text-gray-500 mt-1">
                                    { "Number of skills that can run in parallel (1-16)" }
                                </p>
                            </div>
                        </div>

                        <div class="space-y-3 pt-2">
                            <ToggleSwitch
                                label="Include execution metadata by default"
                                description="Attach timing and environment info to results"
                                checked={current.include_metadata}
                                on_toggle={on_include_metadata_toggle}
                            />

                            <ToggleSwitch
                                label="Enable execution history"
                                description="Track and store execution history for analysis"
                                checked={current.enable_history}
                                on_toggle={on_enable_history_toggle}
                            />
                        </div>
                    </div>
                </Card>

                // Search pipeline settings
                <Card title="Search Pipeline">
                    <div class="space-y-4">
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div>
                                <label class="flex items-center text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                    { "Embedding Provider" }
                                    <Tooltip text="Converts text into numerical vectors for semantic search. FastEmbed runs locally, OpenAI uses API, Ollama is self-hosted." />
                                </label>
                                <select
                                    class="input w-full"
                                    value={current.embedding_provider.clone()}
                                    onchange={on_embedding_provider_change}
                                >
                                    <option value="fastembed" selected={current.embedding_provider == "fastembed"}>
                                        { "FastEmbed (Local)" }
                                    </option>
                                    <option value="openai" selected={current.embedding_provider == "openai"}>
                                        { "OpenAI" }
                                    </option>
                                    <option value="ollama" selected={current.embedding_provider == "ollama"}>
                                        { "Ollama" }
                                    </option>
                                </select>
                                <p class="text-xs text-gray-500 mt-1">
                                    { "FastEmbed runs locally with no API key required" }
                                </p>
                            </div>

                            <div>
                                <label class="flex items-center text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                    { "Vector Store" }
                                    <Tooltip text="Database for storing and searching document embeddings. File-based is persistent and fast. InMemory is faster but data is lost on restart. Qdrant requires Docker." />
                                </label>
                                <select
                                    class="input w-full"
                                    value={current.vector_backend.clone()}
                                    onchange={on_vector_backend_change}
                                >
                                    <option value="file" selected={current.vector_backend == "file"}>
                                        { "File-based (Persistent)" }
                                    </option>
                                    <option value="memory" selected={current.vector_backend == "memory"}>
                                        { "In-Memory" }
                                    </option>
                                    <option value="qdrant" selected={current.vector_backend == "qdrant"}>
                                        { "Qdrant (Docker)" }
                                    </option>
                                </select>
                                <p class="text-xs text-gray-500 mt-1">
                                    { "File-based stores vectors locally with persistence. In-memory is fastest but temporary." }
                                </p>
                            </div>
                        </div>

                        // Embedding Model selection
                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                                    { "Embedding Model" }
                                </label>
                                <button
                                    type="button"
                                    class="text-xs text-primary-600 dark:text-primary-400 hover:underline"
                                    onclick={on_advanced_model_toggle}
                                >
                                    { if current.use_advanced_model { "Use Standard Models" } else { "Advanced (Custom Model)" } }
                                </button>
                            </div>

                            if current.use_advanced_model {
                                // Advanced: Free-form text input
                                <input
                                    type="text"
                                    class="input w-full font-mono text-sm"
                                    value={current.embedding_model.clone()}
                                    oninput={on_embedding_model_change}
                                    placeholder={
                                        match current.embedding_provider.as_str() {
                                            "fastembed" => "all-minilm",
                                            "openai" => "text-embedding-3-small",
                                            "ollama" => "nomic-embed-text",
                                            _ => "model-name"
                                        }
                                    }
                                />
                                <p class="text-xs text-gray-500 mt-1">
                                    { "Enter a custom embedding model name" }
                                </p>
                            } else {
                                // Standard: Dropdown with predefined models
                                <select
                                    class="input w-full"
                                    value={current.embedding_model.clone()}
                                    onchange={on_embedding_model_select}
                                >
                                    { match current.embedding_provider.as_str() {
                                        "fastembed" => html! {
                                            <>
                                                <option value="all-minilm" selected={current.embedding_model == "all-minilm"}>
                                                    { "all-MiniLM (384 dims) - Recommended" }
                                                </option>
                                                <option value="bge-small" selected={current.embedding_model == "bge-small"}>
                                                    { "BGE-Small (384 dims)" }
                                                </option>
                                                <option value="bge-base" selected={current.embedding_model == "bge-base"}>
                                                    { "BGE-Base (768 dims)" }
                                                </option>
                                                <option value="bge-large" selected={current.embedding_model == "bge-large"}>
                                                    { "BGE-Large (1024 dims)" }
                                                </option>
                                            </>
                                        },
                                        "openai" => html! {
                                            <>
                                                <option value="text-embedding-3-small" selected={current.embedding_model == "text-embedding-3-small"}>
                                                    { "text-embedding-3-small (1536 dims) - Recommended" }
                                                </option>
                                                <option value="text-embedding-3-large" selected={current.embedding_model == "text-embedding-3-large"}>
                                                    { "text-embedding-3-large (3072 dims)" }
                                                </option>
                                                <option value="text-embedding-ada-002" selected={current.embedding_model == "text-embedding-ada-002"}>
                                                    { "text-embedding-ada-002 (1536 dims) - Legacy" }
                                                </option>
                                            </>
                                        },
                                        "ollama" => html! {
                                            <>
                                                <option value="nomic-embed-text" selected={current.embedding_model == "nomic-embed-text"}>
                                                    { "nomic-embed-text - Recommended" }
                                                </option>
                                                <option value="mxbai-embed-large" selected={current.embedding_model == "mxbai-embed-large"}>
                                                    { "mxbai-embed-large" }
                                                </option>
                                                <option value="all-minilm" selected={current.embedding_model == "all-minilm"}>
                                                    { "all-minilm" }
                                                </option>
                                            </>
                                        },
                                        _ => html! {
                                            <option value="all-minilm">{ "all-minilm" }</option>
                                        }
                                    }}
                                </select>
                                <p class="text-xs text-gray-500 mt-1">
                                    { "Select from recommended models for " }
                                    <span class="capitalize">{ current.embedding_provider.clone() }</span>
                                </p>
                            }
                        </div>

                        // Conditional Ollama URL input (only shown when provider = "ollama")
                        if current.embedding_provider == "ollama" {
                            <div>
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                    { "Ollama Server URL" }
                                </label>
                                <input
                                    type="text"
                                    class="input w-full font-mono text-sm"
                                    value={current.ollama_url.clone().unwrap_or_else(|| "http://localhost:11434".to_string())}
                                    oninput={on_ollama_url_change}
                                    placeholder="http://localhost:11434"
                                />
                                <p class="text-xs text-gray-500 mt-1">
                                    { "URL of your Ollama server instance" }
                                </p>
                            </div>
                        }

                        // Conditional Qdrant URL input (only shown when backend = "qdrant")
                        if current.vector_backend == "qdrant" {
                            <div>
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                    { "Qdrant Server URL" }
                                </label>
                                <input
                                    type="text"
                                    class="input w-full font-mono text-sm"
                                    value={current.qdrant_url.clone().unwrap_or_else(|| "http://localhost:6333".to_string())}
                                    oninput={on_qdrant_url_change}
                                    placeholder="http://localhost:6333"
                                />
                                <p class="text-xs text-gray-500 mt-1">
                                    { "URL of your Qdrant server instance (requires Docker)" }
                                </p>
                            </div>
                        }

                        <div class="p-3 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
                            <div class="flex items-center justify-between text-sm">
                                <span class="text-gray-600 dark:text-gray-400">{ "Indexed Documents" }</span>
                                <span class="font-mono text-gray-900 dark:text-white">{ current.indexed_documents }</span>
                            </div>
                        </div>

                        // Vector DB Testing Section
                        <div class="border-t border-gray-200 dark:border-gray-700 pt-4 mt-4">
                            <h4 class="text-sm font-medium text-gray-900 dark:text-white mb-3">
                                { "Connection Testing" }
                            </h4>
                            <p class="text-xs text-gray-600 dark:text-gray-400 mb-3">
                                { "Test your embedding provider and vector backend configuration before saving." }
                            </p>

                            <div class="flex gap-2">
                                <button
                                    class="btn btn-secondary text-sm"
                                    onclick={on_test_connection}
                                    disabled={*test_connection_loading || *loading}
                                >
                                    if *test_connection_loading {
                                        <span class="flex items-center gap-2">
                                            <span class="animate-spin">{ "⟳" }</span>
                                            { "Testing..." }
                                        </span>
                                    } else {
                                        { "Quick Test" }
                                    }
                                </button>

                                <button
                                    class="btn btn-secondary text-sm"
                                    onclick={on_test_pipeline}
                                    disabled={*test_pipeline_loading || *loading}
                                >
                                    if *test_pipeline_loading {
                                        <span class="flex items-center gap-2">
                                            <span class="animate-spin">{ "⟳" }</span>
                                            { "Testing..." }
                                        </span>
                                    } else {
                                        { "Full Pipeline Test" }
                                    }
                                </button>
                            </div>

                            // Test result display
                            if let Some(result) = &*test_result {
                                <div class={classes!(
                                    "mt-3", "p-3", "rounded", "text-sm",
                                    if result.success {
                                        "bg-success-50 dark:bg-success-900/20 border border-success-200 dark:border-success-800"
                                    } else {
                                        "bg-error-50 dark:bg-error-900/20 border border-error-200 dark:border-error-800"
                                    }
                                )}>
                                    <div class="flex items-start gap-2">
                                        if result.success {
                                            <svg class="w-4 h-4 text-success-500 flex-shrink-0 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                                            </svg>
                                        } else {
                                            <svg class="w-4 h-4 text-error-500 flex-shrink-0 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                            </svg>
                                        }
                                        <div class="flex-1">
                                            <p class={classes!(
                                                "font-medium",
                                                if result.success { "text-success-700 dark:text-success-300" }
                                                else { "text-error-700 dark:text-error-300" }
                                            )}>
                                                { &result.message }
                                            </p>
                                            if let Some(details) = &result.details {
                                                <p class="text-xs mt-1 text-gray-600 dark:text-gray-400">
                                                    { details }
                                                </p>
                                            }
                                            <p class="text-xs mt-1 text-gray-500 dark:text-gray-500">
                                                { format!("Completed in {}ms", result.duration_ms) }
                                            </p>
                                        </div>
                                    </div>
                                </div>
                            }
                        </div>

                        <div class="space-y-3 pt-2">
                            <ToggleSwitch
                                label="Enable Hybrid Search"
                                description="Combines semantic (AI-based) and keyword (BM25) search for better results across different query types"
                                checked={current.hybrid_search_enabled}
                                on_toggle={on_hybrid_toggle}
                            />

                            <ToggleSwitch
                                label="Enable Reranking"
                                description="Re-orders results using cross-encoder model for better accuracy. Slower but more precise."
                                checked={current.reranking_enabled}
                                on_toggle={on_reranking_toggle}
                            />
                        </div>
                    </div>
                </Card>

                // Data settings
                <Card title="Data Management">
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                { "History retention" }
                            </label>
                            <div class="flex items-center gap-2">
                                <input
                                    type="number"
                                    class="input w-32"
                                    value={current.max_history_entries.to_string()}
                                    min="100"
                                    max="10000"
                                    oninput={on_history_entries_change}
                                />
                                <span class="text-gray-500 dark:text-gray-400">{ "executions" }</span>
                            </div>
                            <p class="text-xs text-gray-500 mt-1">
                                { "Older entries will be automatically removed (100-10,000)" }
                            </p>
                        </div>

                        <div class="flex flex-wrap gap-3 pt-2">
                            <button class="btn btn-secondary" onclick={on_clear_history}>
                                <svg class="w-4 h-4 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                </svg>
                                { "Clear History" }
                            </button>
                            <button class="btn btn-secondary">
                                <svg class="w-4 h-4 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                                </svg>
                                { "Export Config" }
                            </button>
                            <button class="btn btn-secondary" onclick={on_import_click}>
                                <svg class="w-4 h-4 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                                </svg>
                                { "Import Config" }
                            </button>
                        </div>
                    </div>
                </Card>

                // About section
                <Card title="About">
                    <div class="space-y-3">
                        <div class="flex justify-between py-1">
                            <span class="text-gray-500 dark:text-gray-400">{ "Version" }</span>
                            <span class="font-mono text-gray-900 dark:text-white">{ "0.2.2" }</span>
                        </div>
                        <div class="flex justify-between py-1">
                            <span class="text-gray-500 dark:text-gray-400">{ "Build Date" }</span>
                            <span class="font-mono text-gray-900 dark:text-white">{ "2025-12-22" }</span>
                        </div>
                        <div class="flex justify-between py-1">
                            <span class="text-gray-500 dark:text-gray-400">{ "Embedding Model" }</span>
                            <span class="font-mono text-gray-900 dark:text-white">{ &current.embedding_model }</span>
                        </div>
                    </div>
                    <div class="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
                        <a
                            href="https://github.com/your-repo/skill-engine"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="btn btn-secondary"
                        >
                            <svg class="w-4 h-4 mr-2" fill="currentColor" viewBox="0 0 24 24">
                                <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                            </svg>
                            { "View on GitHub" }
                        </a>
                    </div>
                </Card>

                // Save buttons
                <div class="flex justify-end gap-4 sticky bottom-6 bg-gray-50 dark:bg-gray-900 py-4 -mx-6 px-6 border-t border-gray-200 dark:border-gray-800">
                    <button
                        class="btn btn-secondary"
                        onclick={on_reset}
                        disabled={*saving}
                    >
                        { "Reset to Defaults" }
                    </button>
                    <button
                        class="btn btn-primary"
                        onclick={on_save}
                        disabled={!*has_changes || *saving}
                    >
                        if *saving {
                            <svg class="animate-spin -ml-1 mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24">
                                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                            </svg>
                            { "Saving..." }
                        } else {
                            { "Save Changes" }
                        }
                    </button>
                </div>
            }
        </div>
    }
}

/// Toggle switch component props
#[derive(Properties, PartialEq)]
struct ToggleSwitchProps {
    label: &'static str,
    description: &'static str,
    checked: bool,
    on_toggle: Callback<MouseEvent>,
}

/// Toggle switch component
#[function_component(ToggleSwitch)]
fn toggle_switch(props: &ToggleSwitchProps) -> Html {
    html! {
        <div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
            <div>
                <p class="text-sm font-medium text-gray-900 dark:text-white">
                    { props.label }
                </p>
                <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                    { props.description }
                </p>
            </div>
            <button
                type="button"
                role="switch"
                aria-checked={props.checked.to_string()}
                onclick={props.on_toggle.clone()}
                class={classes!(
                    "relative", "inline-flex", "h-6", "w-11", "flex-shrink-0",
                    "cursor-pointer", "rounded-full", "border-2", "border-transparent",
                    "transition-colors", "duration-200", "ease-in-out",
                    "focus:outline-none", "focus:ring-2", "focus:ring-primary-500", "focus:ring-offset-2",
                    if props.checked { "bg-primary-600" } else { "bg-gray-200 dark:bg-gray-600" }
                )}
            >
                <span
                    class={classes!(
                        "pointer-events-none", "inline-block", "h-5", "w-5",
                        "transform", "rounded-full", "bg-white", "shadow",
                        "ring-0", "transition", "duration-200", "ease-in-out",
                        if props.checked { "translate-x-5" } else { "translate-x-0" }
                    )}
                />
            </button>
        </div>
    }
}

/// Skeleton loader for settings cards
#[function_component(SettingsCardSkeleton)]
fn settings_card_skeleton() -> Html {
    html! {
        <div class="card p-6 animate-pulse">
            <div class="h-5 w-32 bg-gray-200 dark:bg-gray-700 rounded mb-4"></div>
            <div class="space-y-4">
                <div class="h-10 w-full bg-gray-200 dark:bg-gray-700 rounded"></div>
                <div class="h-10 w-3/4 bg-gray-200 dark:bg-gray-700 rounded"></div>
            </div>
        </div>
    }
}
