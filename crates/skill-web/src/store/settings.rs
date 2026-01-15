//! Settings state store
//!
//! Persisted to localStorage for cross-session settings.

use serde::{Deserialize, Serialize};
use yewdux::prelude::*;

/// Theme options
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    #[default]
    System,
}

impl Theme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::System => "system",
        }
    }
}

/// Output format options
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum OutputFormat {
    #[default]
    Json,
    Raw,
    Formatted,
}

impl OutputFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Raw => "raw",
            Self::Formatted => "formatted",
        }
    }
}

/// Embedding provider options
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum EmbeddingProvider {
    #[default]
    FastEmbed,
    OpenAI,
    Ollama,
}

impl EmbeddingProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FastEmbed => "fastembed",
            Self::OpenAI => "openai",
            Self::Ollama => "ollama",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::FastEmbed => "FastEmbed (Local)",
            Self::OpenAI => "OpenAI",
            Self::Ollama => "Ollama",
        }
    }
}

/// Vector store backend options
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum VectorBackend {
    #[default]
    InMemory,
    Qdrant,
}

impl VectorBackend {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InMemory => "inmemory",
            Self::Qdrant => "qdrant",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::InMemory => "In-Memory",
            Self::Qdrant => "Qdrant",
        }
    }
}

/// Search pipeline settings
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SearchSettings {
    /// Embedding provider
    pub embedding_provider: EmbeddingProvider,
    /// Embedding model name
    pub embedding_model: String,
    /// Vector store backend
    pub vector_backend: VectorBackend,
    /// Enable hybrid search (BM25 + Vector)
    pub enable_hybrid: bool,
    /// Enable reranking (Cross-encoder)
    pub enable_reranking: bool,
    /// Qdrant URL (if using Qdrant)
    pub qdrant_url: Option<String>,
    /// Ollama URL (if using Ollama)
    pub ollama_url: Option<String>,
}

impl Default for SearchSettings {
    fn default() -> Self {
        Self {
            embedding_provider: EmbeddingProvider::FastEmbed,
            embedding_model: "BAAI/bge-small-en-v1.5".to_string(),
            vector_backend: VectorBackend::InMemory,
            enable_hybrid: false,
            enable_reranking: false,
            qdrant_url: None,
            ollama_url: Some("http://localhost:11434".to_string()),
        }
    }
}

/// API connection settings
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApiSettings {
    /// Base URL for the API
    pub base_url: String,
    /// Request timeout in seconds
    pub timeout_secs: u32,
}

impl Default for ApiSettings {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:3000".to_string(),
            timeout_secs: 30,
        }
    }
}

/// Settings store state
#[derive(Clone, Debug, PartialEq, Store, Serialize, Deserialize)]
#[store(storage = "local", storage_tab_sync)]
pub struct SettingsStore {
    /// UI theme
    pub theme: Theme,
    /// Default execution timeout in seconds
    pub default_timeout: u32,
    /// Default output format
    pub output_format: OutputFormat,
    /// Include metadata in execution output
    pub include_metadata: bool,
    /// History retention count
    pub history_retention: u32,
    /// Whether onboarding has been completed
    pub onboarding_completed: bool,
    /// Search pipeline settings
    pub search: SearchSettings,
    /// API connection settings
    pub api: ApiSettings,
    /// Keyboard shortcuts enabled
    pub keyboard_shortcuts: bool,
    /// Show tool parameters by default
    pub expand_parameters: bool,
    /// Auto-refresh interval in seconds (0 = disabled)
    pub auto_refresh_interval: u32,
}

impl Default for SettingsStore {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            default_timeout: 30,
            output_format: OutputFormat::Json,
            include_metadata: false,
            history_retention: 1000,
            onboarding_completed: false,
            search: SearchSettings::default(),
            api: ApiSettings::default(),
            keyboard_shortcuts: true,
            expand_parameters: false,
            auto_refresh_interval: 0,
        }
    }
}

impl SettingsStore {
    /// Check if the user needs to go through onboarding
    pub fn needs_onboarding(&self) -> bool {
        !self.onboarding_completed
    }

    /// Get the effective theme (resolving system preference)
    pub fn effective_theme(&self) -> Theme {
        // In a real implementation, we'd check the system preference here
        // For now, just return the stored theme
        self.theme.clone()
    }
}

/// Settings store actions
pub enum SettingsAction {
    SetTheme(Theme),
    SetDefaultTimeout(u32),
    SetOutputFormat(OutputFormat),
    SetIncludeMetadata(bool),
    SetHistoryRetention(u32),
    CompleteOnboarding,
    // Search settings
    SetEmbeddingProvider(EmbeddingProvider),
    SetEmbeddingModel(String),
    SetVectorBackend(VectorBackend),
    SetEnableHybrid(bool),
    SetEnableReranking(bool),
    SetQdrantUrl(Option<String>),
    SetOllamaUrl(Option<String>),
    // API settings
    SetApiBaseUrl(String),
    SetApiTimeout(u32),
    // UI preferences
    SetKeyboardShortcuts(bool),
    SetExpandParameters(bool),
    SetAutoRefreshInterval(u32),
    // Reset
    ResetToDefaults,
    ResetSearchSettings,
}

impl Reducer<SettingsStore> for SettingsAction {
    fn apply(self, mut store: std::rc::Rc<SettingsStore>) -> std::rc::Rc<SettingsStore> {
        let state = std::rc::Rc::make_mut(&mut store);

        match self {
            SettingsAction::SetTheme(theme) => {
                state.theme = theme;
            }
            SettingsAction::SetDefaultTimeout(timeout) => {
                state.default_timeout = timeout;
            }
            SettingsAction::SetOutputFormat(format) => {
                state.output_format = format;
            }
            SettingsAction::SetIncludeMetadata(include) => {
                state.include_metadata = include;
            }
            SettingsAction::SetHistoryRetention(count) => {
                state.history_retention = count;
            }
            SettingsAction::CompleteOnboarding => {
                state.onboarding_completed = true;
            }
            // Search settings
            SettingsAction::SetEmbeddingProvider(provider) => {
                state.search.embedding_provider = provider;
            }
            SettingsAction::SetEmbeddingModel(model) => {
                state.search.embedding_model = model;
            }
            SettingsAction::SetVectorBackend(backend) => {
                state.search.vector_backend = backend;
            }
            SettingsAction::SetEnableHybrid(enable) => {
                state.search.enable_hybrid = enable;
            }
            SettingsAction::SetEnableReranking(enable) => {
                state.search.enable_reranking = enable;
            }
            SettingsAction::SetQdrantUrl(url) => {
                state.search.qdrant_url = url;
            }
            SettingsAction::SetOllamaUrl(url) => {
                state.search.ollama_url = url;
            }
            // API settings
            SettingsAction::SetApiBaseUrl(url) => {
                state.api.base_url = url;
            }
            SettingsAction::SetApiTimeout(timeout) => {
                state.api.timeout_secs = timeout;
            }
            // UI preferences
            SettingsAction::SetKeyboardShortcuts(enable) => {
                state.keyboard_shortcuts = enable;
            }
            SettingsAction::SetExpandParameters(expand) => {
                state.expand_parameters = expand;
            }
            SettingsAction::SetAutoRefreshInterval(interval) => {
                state.auto_refresh_interval = interval;
            }
            // Reset
            SettingsAction::ResetToDefaults => {
                *state = SettingsStore::default();
            }
            SettingsAction::ResetSearchSettings => {
                state.search = SearchSettings::default();
            }
        }

        store
    }
}
