//! HTTP Server implementation - REST API for skill invocation

use anyhow::Result;
use skill_runtime::{InstanceManager, LocalSkillLoader, SkillEngine, SkillManifest};
use skill_runtime::search::SearchPipeline;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::analytics::SearchAnalyticsDb;
use crate::execution_history::ExecutionHistoryDb;
use crate::routes::{create_app, create_app_with_ui};
use crate::types::{ExecutionHistoryEntry, ServiceStatus, SkillServiceRequirement, SkillSummary};

/// HTTP Server configuration
#[derive(Debug, Clone)]
pub struct HttpServerConfig {
    /// Host to bind to
    pub host: String,
    /// Port to listen on
    pub port: u16,
    /// Enable CORS for local development
    pub enable_cors: bool,
    /// Enable request tracing
    pub enable_tracing: bool,
    /// Enable embedded web UI
    pub enable_web_ui: bool,
    /// Working directory for skills
    pub working_dir: Option<PathBuf>,
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            enable_cors: true,
            enable_tracing: true,
            enable_web_ui: false,
            working_dir: None,
        }
    }
}

/// Tracked system service (like kubectl proxy)
pub struct TrackedService {
    /// Service name
    pub name: String,
    /// Process handle
    pub process: Option<std::process::Child>,
    /// Port being used
    pub port: u16,
}

/// Shared application state
pub struct AppState {
    /// Server start time for uptime tracking
    pub started_at: Instant,
    /// Installed skills (in-memory for now)
    pub skills: RwLock<HashMap<String, SkillSummary>>,
    /// Execution history (in-memory cache for fast access)
    pub execution_history: RwLock<Vec<ExecutionHistoryEntry>>,
    /// Execution history database (persistent storage)
    pub execution_history_db: RwLock<Option<Arc<ExecutionHistoryDb>>>,
    /// Server configuration
    pub config: HttpServerConfig,
    /// Skill engine for WASM execution
    pub engine: Arc<SkillEngine>,
    /// Skill manifest for configuration
    pub manifest: RwLock<Option<SkillManifest>>,
    /// Instance manager
    pub instance_manager: InstanceManager,
    /// Local skill loader
    pub local_loader: LocalSkillLoader,
    /// Working directory
    pub working_dir: PathBuf,
    /// Tracked background services (kubectl proxy, etc.)
    pub services: RwLock<HashMap<String, TrackedService>>,
    /// Search pipeline for semantic search
    pub search_pipeline: RwLock<Option<Arc<SearchPipeline>>>,
    /// Analytics database for search history and feedback
    pub analytics_db: RwLock<Option<Arc<SearchAnalyticsDb>>>,
}

impl AppState {
    /// Create new application state
    pub fn new(config: HttpServerConfig) -> Result<Self> {
        let working_dir = config.working_dir.clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let engine = Arc::new(SkillEngine::new()?);
        let instance_manager = InstanceManager::new()?;
        let local_loader = LocalSkillLoader::new()?;

        // Try to load manifest from working directory - find() searches for .skill-engine.toml
        let manifest = SkillManifest::find(&working_dir)
            .and_then(|path| SkillManifest::load(&path).ok());

        Ok(Self {
            started_at: Instant::now(),
            skills: RwLock::new(HashMap::new()),
            execution_history: RwLock::new(Vec::new()),
            execution_history_db: RwLock::new(None),
            config,
            engine,
            manifest: RwLock::new(manifest),
            instance_manager,
            local_loader,
            working_dir,
            services: RwLock::new(HashMap::new()),
            search_pipeline: RwLock::new(None),
            analytics_db: RwLock::new(None),
        })
    }

    /// Initialize search pipeline with default configuration
    pub async fn initialize_search_pipeline(&self) -> Result<()> {
        use skill_runtime::search_config::SearchConfig;

        let config = SearchConfig::default();
        let pipeline = SearchPipeline::from_config(config).await?;

        let mut search_pipeline = self.search_pipeline.write().await;
        *search_pipeline = Some(Arc::new(pipeline));

        Ok(())
    }

    /// Initialize analytics database
    pub async fn initialize_analytics_db(&self) -> Result<()> {
        let db_path = dirs::home_dir()
            .map(|p| p.join(".skill-engine/analytics.db"))
            .unwrap_or_else(|| PathBuf::from(".skill-engine/analytics.db"))
            .to_string_lossy()
            .to_string();

        let db = SearchAnalyticsDb::new(&db_path).await?;

        let mut analytics_db = self.analytics_db.write().await;
        *analytics_db = Some(Arc::new(db));

        info!("Analytics database initialized at: {}", db_path);

        Ok(())
    }

    /// Initialize execution history database
    pub async fn initialize_execution_history_db(&self) -> Result<()> {
        let db_path = dirs::home_dir()
            .map(|p| p.join(".skill-engine/execution-history.db"))
            .unwrap_or_else(|| PathBuf::from(".skill-engine/execution-history.db"))
            .to_string_lossy()
            .to_string();

        let db = ExecutionHistoryDb::new(&db_path).await?;

        // Load recent history into memory cache (for fast access)
        let recent_history = db.list_executions(1000, 0).await?;
        let mut history = self.execution_history.write().await;
        *history = recent_history;

        let mut execution_history_db = self.execution_history_db.write().await;
        *execution_history_db = Some(Arc::new(db));

        info!("Execution history database initialized at: {}", db_path);

        Ok(())
    }

    /// Initialize skills from manifest - loads all tools at startup
    pub async fn load_skills_from_manifest(&self) -> Result<()> {
        // First, collect skill info from manifest
        let skill_infos: Vec<_> = {
            let manifest = self.manifest.read().await;
            if let Some(manifest) = manifest.as_ref() {
                manifest.skills.iter().map(|(name, skill_def)| {
                    let instances_count = if skill_def.instances.is_empty() { 1 } else { skill_def.instances.len() };
                    let runtime_str = match skill_def.runtime {
                        skill_runtime::SkillRuntime::Wasm => "wasm",
                        skill_runtime::SkillRuntime::Docker => "docker",
                        skill_runtime::SkillRuntime::Native => "native",
                    };
                    let source_path = if skill_def.source.starts_with("./") || skill_def.source.starts_with('/') {
                        manifest.base_dir.join(&skill_def.source)
                    } else {
                        let home = dirs::home_dir().unwrap_or_default();
                        home.join(".skill-engine").join("registry").join(name)
                    };
                    (
                        name.clone(),
                        skill_def.description.clone().unwrap_or_default(),
                        skill_def.source.clone(),
                        runtime_str.to_string(),
                        instances_count,
                        skill_def.runtime == skill_runtime::SkillRuntime::Wasm,
                        source_path,
                        skill_def.services.clone(),
                    )
                }).collect()
            } else {
                vec![]
            }
        };

        // Now load tools for each skill (no locks held)
        let mut skills_to_insert = Vec::new();
        for (name, description, source, runtime, instances_count, is_wasm, source_path, services) in skill_infos {
            // Try to load tools count from SKILL.md first (works for all skill types)
            let tools_count = if source_path.exists() {
                use skill_runtime::skill_md::find_skill_md;
                if let Some(skill_md_path) = find_skill_md(&source_path) {
                    match skill_runtime::skill_md::parse_skill_md(&skill_md_path) {
                        Ok(skill_content) => skill_content.tool_docs.len(),
                        Err(_) => {
                            // Fallback to WASM loading for WASM skills
                            if is_wasm {
                                self.load_skill_tools_count(&name, &source_path).await
                            } else {
                                0
                            }
                        }
                    }
                } else if is_wasm {
                    self.load_skill_tools_count(&name, &source_path).await
                } else {
                    0
                }
            } else {
                0
            };

            // Convert service requirements to SkillServiceRequirement with initial status
            let required_services: Vec<SkillServiceRequirement> = services.iter().map(|s| {
                SkillServiceRequirement {
                    name: s.name.clone(),
                    description: s.description.clone(),
                    optional: s.optional,
                    default_port: s.default_port,
                    status: ServiceStatus {
                        name: s.name.clone(),
                        running: false,
                        pid: None,
                        port: s.default_port,
                        url: None,
                        error: None,
                    },
                }
            }).collect();

            let skill_summary = SkillSummary {
                name: name.clone(),
                version: "0.1.0".to_string(),
                description,
                source,
                runtime,
                tools_count,
                instances_count,
                last_used: None,
                execution_count: 0,
                required_services,
            };
            skills_to_insert.push((name, skill_summary));
        }

        // Insert all skills
        let mut skills = self.skills.write().await;
        for (name, summary) in skills_to_insert {
            skills.insert(name, summary);
        }
        info!("Loaded {} skills from manifest", skills.len());

        Ok(())
    }

    /// Load tools count for a skill
    async fn load_skill_tools_count(&self, name: &str, source_path: &PathBuf) -> usize {
        match self.local_loader.load_skill(source_path, &self.engine).await {
            Ok(component) => {
                let instance_config = skill_runtime::instance::InstanceConfig::default();
                match skill_runtime::SkillExecutor::from_component(
                    self.engine.clone(),
                    component,
                    name.to_string(),
                    "default".to_string(),
                    instance_config,
                ) {
                    Ok(executor) => {
                        match executor.get_tools().await {
                            Ok(tools) => {
                                info!(skill = %name, tools = tools.len(), "Loaded skill tools");
                                tools.len()
                            }
                            Err(e) => {
                                tracing::warn!(skill = %name, error = %e, "Failed to get tools");
                                0
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!(skill = %name, error = %e, "Failed to create executor");
                        0
                    }
                }
            }
            Err(e) => {
                tracing::warn!(skill = %name, error = %e, "Failed to load skill");
                0
            }
        }
    }
}

/// HTTP Server that exposes skills via REST API
pub struct HttpServer {
    config: HttpServerConfig,
}

impl HttpServer {
    /// Create a new HTTP server with default config
    pub fn new() -> Result<Self> {
        Ok(Self {
            config: HttpServerConfig::default(),
        })
    }

    /// Create a new HTTP server with custom config
    pub fn with_config(config: HttpServerConfig) -> Result<Self> {
        Ok(Self { config })
    }

    /// Run the HTTP server
    pub async fn run(&self) -> Result<()> {
        // Create application state
        let state = Arc::new(AppState::new(self.config.clone())?);

        // Initialize execution history database
        if let Err(e) = state.initialize_execution_history_db().await {
            tracing::warn!("Failed to initialize execution history database: {}", e);
        }

        // Initialize analytics database
        if let Err(e) = state.initialize_analytics_db().await {
            tracing::warn!("Failed to initialize analytics database: {}", e);
        }

        // Load skills from manifest
        state.load_skills_from_manifest().await?;

        // Build the application router based on mode
        let mut app = if self.config.enable_web_ui {
            create_app_with_ui(state)
        } else {
            create_app(state)
        };

        // Add CORS middleware if enabled
        if self.config.enable_cors {
            let cors = CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any);
            app = app.layer(cors);
        }

        // Add tracing middleware if enabled
        if self.config.enable_tracing {
            app = app.layer(TraceLayer::new_for_http());
        }

        // Bind and serve
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        info!(
            address = %addr,
            cors = self.config.enable_cors,
            tracing = self.config.enable_tracing,
            web_ui = self.config.enable_web_ui,
            "HTTP server starting"
        );

        if self.config.enable_web_ui {
            println!("Skill Engine Web UI available at http://{}", addr);
            println!("  Web interface: http://{}/", addr);
            println!("  API endpoints: http://{}/api/...", addr);
        } else {
            println!("Skill Engine HTTP API listening on http://{}", addr);
            println!("  API endpoints: http://{}/api/...", addr);
            println!("  Health check:  http://{}/api/health", addr);
        }

        axum::serve(listener, app).await?;

        Ok(())
    }
}

impl Default for HttpServer {
    fn default() -> Self {
        Self::new().expect("Failed to create default HttpServer")
    }
}
