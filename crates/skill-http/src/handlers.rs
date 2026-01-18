//! API request handlers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use skill_runtime::{instance::InstanceConfig, SkillExecutor};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::types::*;
use crate::AppState;

/// List all installed skills
pub async fn list_skills(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<SkillSummary>>, (StatusCode, Json<ApiError>)> {
    debug!("Listing skills (page={}, per_page={})", pagination.page, pagination.per_page);

    let skills = state.skills.read().await;
    let total = skills.len();

    let start = (pagination.page.saturating_sub(1)) * pagination.per_page;
    let items: Vec<SkillSummary> = skills
        .values()
        .skip(start)
        .take(pagination.per_page)
        .cloned()
        .collect();

    Ok(Json(PaginatedResponse::new(items, total, pagination.page, pagination.per_page)))
}

/// Get details for a specific skill
pub async fn get_skill(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<SkillDetail>, (StatusCode, Json<ApiError>)> {
    debug!("Getting skill: {}", name);

    let skills = state.skills.read().await;
    let skill = skills
        .get(&name)
        .ok_or_else(|| {
            (StatusCode::NOT_FOUND, Json(ApiError::not_found(&format!("Skill '{}'", name))))
        })?
        .clone();
    drop(skills);

    // Try to load tools from the skill
    let mut tools = Vec::new();
    let mut instances = Vec::new();
    let mut required_services = Vec::new();

    // Check manifest for skill definition
    let manifest = state.manifest.read().await;
    if let Some(manifest) = manifest.as_ref() {
        if let Some(skill_def) = manifest.get_skill(&name) {
            // Get instances from manifest
            for (inst_name, _inst_config) in &skill_def.instances {
                instances.push(InstanceInfo {
                    name: inst_name.clone(),
                    description: None,
                    is_default: inst_name == "default",
                    config_keys: vec![],
                });
            }

            // Get service requirements from manifest
            for service_req in &skill_def.services {
                let status = get_service_status(&state, &service_req.name, service_req.default_port).await;
                required_services.push(SkillServiceRequirement {
                    name: service_req.name.clone(),
                    description: service_req.description.clone(),
                    optional: service_req.optional,
                    default_port: service_req.default_port,
                    status,
                });
            }

            // Try to load tools from the skill source
            let source_path = if skill_def.source.starts_with("./") || skill_def.source.starts_with('/') {
                state.working_dir.join(&skill_def.source)
            } else {
                // For non-local sources, check if installed
                let home = dirs::home_dir().unwrap_or_default();
                home.join(".skill-engine").join("registry").join(&name)
            };

            // Try to load tools from SKILL.md first (works for all skill types)
            if source_path.exists() {
                use skill_runtime::skill_md::{parse_skill_md, find_skill_md};

                if let Some(skill_md_path) = find_skill_md(&source_path) {
                    if let Ok(skill_content) = parse_skill_md(&skill_md_path) {
                        tools = skill_content.tool_docs.into_iter().map(|(tool_name, tool_doc)| ToolInfo {
                            name: tool_name,
                            description: tool_doc.description,
                            parameters: tool_doc.parameters.into_iter().map(|p| ParameterInfo {
                                name: p.name,
                                param_type: p.param_type.to_string(),
                                description: p.description,
                                required: p.required,
                                default_value: p.default,
                            }).collect(),
                            streaming: false,
                        }).collect();
                    }
                }

                // Fallback: Try to load as WASM if SKILL.md parsing didn't work
                if tools.is_empty() {
                    if let Ok(component) = state.local_loader.load_skill(&source_path, &state.engine).await {
                        let instance_config = InstanceConfig::default();
                        // Create executor directly with the loaded component
                        if let Ok(executor) = SkillExecutor::from_component(
                            state.engine.clone(),
                            component,
                            name.clone(),
                            "default".to_string(),
                            instance_config,
                        ) {
                            if let Ok(tool_defs) = executor.get_tools().await {
                                tools = tool_defs.into_iter().map(|t| ToolInfo {
                                    name: t.name,
                                    description: t.description,
                                    parameters: t.parameters.into_iter().map(|p| ParameterInfo {
                                        name: p.name,
                                        param_type: format!("{:?}", p.param_type).to_lowercase(),
                                        description: p.description,
                                        required: p.required,
                                        default_value: p.default_value,
                                    }).collect(),
                                    streaming: t.streaming,
                                }).collect();
                            }
                        }
                    }
                }
            }
        }
    }
    drop(manifest);

    // If no instances found, add default
    if instances.is_empty() {
        instances.push(InstanceInfo {
            name: "default".to_string(),
            description: None,
            is_default: true,
            config_keys: vec![],
        });
    }

    // Update the cached skill summary with actual tools count
    if !tools.is_empty() {
        let mut skills = state.skills.write().await;
        if let Some(cached_skill) = skills.get_mut(&name) {
            cached_skill.tools_count = tools.len();
            cached_skill.instances_count = instances.len();
        }
    }

    // Build full detail response
    let detail = SkillDetail {
        summary: SkillSummary {
            tools_count: tools.len(),
            instances_count: instances.len(),
            required_services,
            ..skill
        },
        full_description: None,
        author: None,
        repository: None,
        license: None,
        tools,
        instances,
    };

    Ok(Json(detail))
}

/// Get the current status of a service
async fn get_service_status(state: &Arc<AppState>, service_name: &str, default_port: Option<u16>) -> ServiceStatus {
    let services = state.services.read().await;

    if let Some(service) = services.get(service_name) {
        // Service is tracked by the server
        let running = service.process.is_some() || check_port_in_use(service.port);
        ServiceStatus {
            name: service_name.to_string(),
            running,
            pid: None,
            port: Some(service.port),
            url: if running { Some(format!("http://127.0.0.1:{}", service.port)) } else { None },
            error: None,
        }
    } else if let Some(port) = default_port {
        // Check if service is running externally on default port
        let running = check_port_in_use(port);
        ServiceStatus {
            name: service_name.to_string(),
            running,
            pid: None,
            port: if running { Some(port) } else { None },
            url: if running { Some(format!("http://127.0.0.1:{}", port)) } else { None },
            error: None,
        }
    } else {
        // Unknown service, not running
        ServiceStatus {
            name: service_name.to_string(),
            running: false,
            pid: None,
            port: None,
            url: None,
            error: None,
        }
    }
}

/// Install a new skill
pub async fn install_skill(
    State(state): State<Arc<AppState>>,
    Json(request): Json<InstallSkillRequest>,
) -> Result<Json<InstallSkillResponse>, (StatusCode, Json<ApiError>)> {
    info!("Installing skill from: {}", request.source);

    // TODO: Implement actual skill installation using GitSkillLoader
    // For now, return a mock response

    let name = request.name.clone().unwrap_or_else(|| {
        // Extract name from source
        request.source
            .split('/')
            .last()
            .unwrap_or("unknown")
            .trim_end_matches(".git")
            .to_string()
    });

    // Add to skills list
    let skill = SkillSummary {
        name: name.clone(),
        version: "0.1.0".to_string(),
        description: format!("Installed from {}", request.source),
        source: request.source.clone(),
        runtime: "wasm".to_string(),
        tools_count: 0,
        instances_count: 1,
        last_used: None,
        execution_count: 0,
        required_services: Vec::new(),
    };

    state.skills.write().await.insert(name.clone(), skill);

    Ok(Json(InstallSkillResponse {
        success: true,
        name: Some(name),
        version: Some("0.1.0".to_string()),
        error: None,
        tools_count: 0,
    }))
}

/// Uninstall a skill
pub async fn uninstall_skill(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    info!("Uninstalling skill: {}", name);

    let mut skills = state.skills.write().await;
    if skills.remove(&name).is_some() {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, Json(ApiError::not_found(&format!("Skill '{}'", name)))))
    }
}

/// Execute a native skill (CLI commands like kubectl, docker, git, terraform)
async fn execute_native_skill(
    state: Arc<AppState>,
    skill_name: &str,
    tool_name: &str,
    instance_name: String,
    args: &HashMap<String, serde_json::Value>,
    start: Instant,
) -> Result<Json<ExecutionResponse>, (StatusCode, Json<ApiError>)> {
    use tokio::process::Command;

    let execution_id = Uuid::new_v4().to_string();

    // Convert JSON args to Vec<(String, String)>
    let parsed_args: Vec<(String, String)> = args.iter()
        .map(|(k, v)| {
            let value = match v {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Number(n) => n.to_string(),
                other => other.to_string().trim_matches('"').to_string(),
            };
            (k.clone(), value)
        })
        .collect();

    // Build the native command
    let command_str = build_native_command(skill_name, tool_name, &parsed_args)
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::internal(&format!("Failed to build command: {}", e))))
        })?;

    // Parse the command
    let parts: Vec<&str> = command_str.split_whitespace().collect();
    if parts.is_empty() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::internal("Empty command")),
        ));
    }

    let program = parts[0];
    let args = &parts[1..];

    // Execute the command
    let output = Command::new(program)
        .args(args)
        .output()
        .await
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::internal(&format!("Failed to execute command: {}", e))))
        })?;

    let duration_ms = start.elapsed().as_millis() as u64;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let success = output.status.success();

    let (status, error_msg) = if success {
        (ExecutionStatus::Success, None)
    } else {
        (ExecutionStatus::Failed, Some(stderr.clone()))
    };

    // Record in history
    let history_entry = ExecutionHistoryEntry {
        id: execution_id.clone(),
        skill: skill_name.to_string(),
        tool: tool_name.to_string(),
        instance: instance_name.clone(),
        status: status.clone(),
        duration_ms,
        started_at: Utc::now(),
        error: error_msg.clone(),
        output: Some(stdout.clone()),
    };

    // Save to in-memory cache
    state.execution_history.write().await.push(history_entry.clone());

    // Save to database (non-blocking)
    if let Some(db) = state.execution_history_db.read().await.as_ref() {
        let db = db.clone();
        let entry = history_entry.clone();
        tokio::spawn(async move {
            if let Err(e) = db.add_execution(&entry).await {
                tracing::warn!("Failed to save execution to database: {}", e);
            }
        });
    }

    // Update skill's last_used and execution_count
    let mut skills = state.skills.write().await;
    if let Some(skill) = skills.get_mut(skill_name) {
        skill.last_used = Some(Utc::now());
        skill.execution_count += 1;
    }
    drop(skills);

    let response = if success {
        ExecutionResponse {
            id: execution_id,
            status,
            output: stdout,
            error: None,
            duration_ms,
            metadata: HashMap::new(),
        }
    } else {
        ExecutionResponse {
            id: execution_id,
            status,
            output: stdout,
            error: error_msg,
            duration_ms,
            metadata: HashMap::new(),
        }
    };

    Ok(Json(response))
}

/// Build a native command from skill name, tool name, and arguments
fn build_native_command(
    skill_name: &str,
    tool_name: &str,
    args: &[(String, String)],
) -> anyhow::Result<String> {
    // Map skill name to base CLI command
    let base_command = match skill_name {
        "kubernetes" => "kubectl",
        "aws" => "aws",
        "docker" => "docker",
        "terraform" => "terraform",
        "helm" => "helm",
        "git" => "git",
        "postgres-native" => "psql",
        _ => skill_name,
    };

    let mut cmd_parts = vec![base_command.to_string()];

    // Add tool name as subcommand
    cmd_parts.push(tool_name.to_string());

    // Process arguments generically
    for (key, value) in args {
        if key == "arg" || key == "resource" || key.is_empty() {
            // Positional argument - just add the value
            cmd_parts.push(value.clone());
        } else if value == "true" {
            // Boolean flag
            if key.len() == 1 {
                cmd_parts.push(format!("-{}", key));
            } else {
                cmd_parts.push(format!("--{}", key));
            }
        } else if value == "false" {
            // Skip false boolean flags
            continue;
        } else if key.len() == 1 {
            // Short flag: -n value
            cmd_parts.push(format!("-{}", key));
            cmd_parts.push(value.clone());
        } else {
            // Long flag: --namespace value
            cmd_parts.push(format!("--{}", key));
            cmd_parts.push(value.clone());
        }
    }

    Ok(cmd_parts.join(" "))
}

/// Execute a tool
pub async fn execute_tool(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ExecutionRequest>,
) -> Result<Json<ExecutionResponse>, (StatusCode, Json<ApiError>)> {
    let start = Instant::now();
    let execution_id = Uuid::new_v4().to_string();
    let instance_name = request.instance.clone().unwrap_or_else(|| "default".to_string());

    info!(
        execution_id = %execution_id,
        skill = %request.skill,
        tool = %request.tool,
        instance = %instance_name,
        "Executing tool"
    );

    // Verify skill exists
    let skills = state.skills.read().await;
    if !skills.contains_key(&request.skill) {
        debug!("Skill '{}' not found in skills list", request.skill);
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::not_found(&format!("Skill '{}'", request.skill))),
        ));
    }
    debug!("Skill '{}' found in skills list", request.skill);
    drop(skills);

    // Get skill source from manifest
    let manifest = state.manifest.read().await;

    if manifest.is_none() {
        warn!("Manifest is None - skills will execute as WASM by default");
    }

    let skill_def = manifest.as_ref()
        .and_then(|m| {
            debug!("Getting skill '{}' from manifest", request.skill);
            m.get_skill(&request.skill)
        })
        .ok_or_else(|| {
            warn!("Skill '{}' not found in manifest", request.skill);
            (StatusCode::NOT_FOUND, Json(ApiError::not_found(&format!("Skill '{}' not in manifest", request.skill))))
        })?
        .clone();
    drop(manifest);

    // Check if this is a native skill
    use skill_runtime::SkillRuntime;
    debug!("Skill runtime: {:?}, checking if Native", skill_def.runtime);
    if skill_def.runtime == SkillRuntime::Native {
        debug!("Routing to native skill execution");
        return execute_native_skill(state.clone(), &request.skill, &request.tool, instance_name, &request.args, start).await;
    }

    // Determine source path
    let source_path = if skill_def.source.starts_with("./") || skill_def.source.starts_with('/') {
        state.working_dir.join(&skill_def.source)
    } else {
        let home = dirs::home_dir().unwrap_or_default();
        home.join(".skill-engine").join("registry").join(&request.skill)
    };

    // Load and execute the WASM skill
    let result = async {
        // load_skill returns a Component directly
        let component = state.local_loader.load_skill(&source_path, &state.engine).await
            .map_err(|e| format!("Failed to load skill: {}", e))?;

        // Build instance config with environment variables
        let mut instance_config = InstanceConfig::default();

        // Get service requirements from skill definition and inject running service URLs
        let mut service_urls: Vec<(String, String)> = Vec::new();
        for service_req in &skill_def.services {
            let status = get_service_status(&state, &service_req.name, service_req.default_port).await;
            if status.running {
                if let Some(url) = &status.url {
                    // Add to environment (for WASI-compatible access)
                    let env_key = format!("{}_URL", service_req.name.to_uppercase().replace("-", "_"));
                    instance_config.environment.insert(env_key.clone(), url.clone());

                    // Also prepare for argument injection (for JS WASM components)
                    let arg_key = format!("_{}_url", service_req.name.replace("-", "_"));
                    service_urls.push((arg_key, url.clone()));

                    debug!("Passing {} to skill execution", env_key);
                }
            }
        }

        let executor = SkillExecutor::from_component(
            state.engine.clone(),
            component,
            request.skill.clone(),
            instance_name.clone(),
            instance_config,
        ).map_err(|e| format!("Failed to create executor: {}", e))?;

        // Convert args to Vec<(String, String)>
        let mut args: Vec<(String, String)> = request.args.iter()
            .map(|(k, v)| {
                let value = match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                (k.clone(), value)
            })
            .collect();

        // Inject service URLs as arguments for WASM JS components
        // This works around the limitation that WASM JS components can't easily read env vars
        for (key, url) in service_urls {
            args.push((key, url));
        }

        let exec_result = executor.execute_tool(&request.tool, args).await
            .map_err(|e| format!("Execution failed: {}", e))?;

        Ok::<_, String>(exec_result)
    }.await;

    let duration_ms = start.elapsed().as_millis() as u64;

    let (status, output, error) = match result {
        Ok(exec_result) => {
            if exec_result.success {
                (ExecutionStatus::Success, exec_result.output, None)
            } else {
                (ExecutionStatus::Failed, exec_result.output, exec_result.error_message)
            }
        }
        Err(e) => {
            warn!(error = %e, "Tool execution failed");
            (ExecutionStatus::Failed, String::new(), Some(e))
        }
    };

    // Record in history
    let history_entry = ExecutionHistoryEntry {
        id: execution_id.clone(),
        skill: request.skill.clone(),
        tool: request.tool.clone(),
        instance: instance_name,
        status: status.clone(),
        duration_ms,
        started_at: Utc::now(),
        error: error.clone(),
        output: Some(output.clone()),
    };

    // Save to in-memory cache
    state.execution_history.write().await.push(history_entry.clone());

    // Save to database (non-blocking)
    if let Some(db) = state.execution_history_db.read().await.as_ref() {
        let db = db.clone();
        let entry = history_entry.clone();
        tokio::spawn(async move {
            if let Err(e) = db.add_execution(&entry).await {
                tracing::warn!("Failed to save execution to database: {}", e);
            }
        });
    }

    // Update skill's last_used and execution_count
    let mut skills = state.skills.write().await;
    if let Some(skill) = skills.get_mut(&request.skill) {
        skill.last_used = Some(Utc::now());
        skill.execution_count += 1;
    }

    Ok(Json(ExecutionResponse {
        id: execution_id,
        status,
        output,
        error,
        duration_ms,
        metadata: HashMap::new(),
    }))
}

/// List execution history
pub async fn list_executions(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<ExecutionHistoryEntry>>, (StatusCode, Json<ApiError>)> {
    debug!("Listing executions");

    let history = state.execution_history.read().await;
    let total = history.len();

    let start = (pagination.page.saturating_sub(1)) * pagination.per_page;
    let items: Vec<ExecutionHistoryEntry> = history
        .iter()
        .rev() // Most recent first
        .skip(start)
        .take(pagination.per_page)
        .cloned()
        .collect();

    Ok(Json(PaginatedResponse::new(items, total, pagination.page, pagination.per_page)))
}

/// Get a specific execution
pub async fn get_execution(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ExecutionHistoryEntry>, (StatusCode, Json<ApiError>)> {
    debug!("Getting execution: {}", id);

    // Try in-memory cache first
    let history = state.execution_history.read().await;
    if let Some(entry) = history.iter().find(|e| e.id == id).cloned() {
        return Ok(Json(entry));
    }
    drop(history);

    // If not in cache, try database
    if let Some(db) = state.execution_history_db.read().await.as_ref() {
        if let Ok(Some(entry)) = db.get_execution(&id).await {
            return Ok(Json(entry));
        }
    }

    Err((StatusCode::NOT_FOUND, Json(ApiError::not_found(&format!("Execution '{}'", id)))))
}

/// Clear all execution history
///
/// Permanently deletes all execution history entries from both memory and persistent storage.
/// This action cannot be undone.
pub async fn clear_execution_history(
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    info!("Clearing execution history");

    // Clear in-memory cache
    state.execution_history.write().await.clear();

    // Clear database
    if let Some(db) = state.execution_history_db.read().await.as_ref() {
        db.clear_all().await.map_err(|e| {
            error!("Failed to clear execution history database: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::internal("Failed to clear execution history database")),
            )
        })?;
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Semantic search for skills/tools
pub async fn semantic_search(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<ApiError>)> {
    let start = Instant::now();

    info!(query = %request.query, top_k = request.top_k, "Performing semantic search");

    // Get search pipeline
    let pipeline = {
        let pipeline_lock = state.search_pipeline.read().await;
        match pipeline_lock.as_ref() {
            Some(p) => p.clone(),
            None => {
                // Return empty results if pipeline not initialized
                warn!("Search pipeline not initialized - returning empty results");
                return Ok(Json(SearchResponse {
                    results: vec![],
                    query_info: Some(QueryInfo {
                        normalized: request.query.clone(),
                        intent: "search".to_string(),
                        confidence: 0.0,
                    }),
                    duration_ms: 0,
                }));
            }
        }
    };

    // Perform search
    let search_results = pipeline.search(&request.query, request.top_k).await.map_err(|e| {
        warn!("Search failed: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::internal(format!("Search failed: {}", e))),
        )
    })?;

    // Convert results
    let mut results: Vec<SearchResult> = search_results
        .into_iter()
        .map(|r| SearchResult {
            id: r.id,
            skill: r.metadata.skill_name.unwrap_or_default(),
            tool: r.metadata.tool_name.unwrap_or_default(),
            content: r.content,
            score: r.score,
            rerank_score: r.rerank_score,
        })
        .collect();

    // If filtering by skill, filter results
    if let Some(ref skill_filter) = request.skill_filter {
        results.retain(|r| r.skill == *skill_filter);
    }

    let duration_ms = start.elapsed().as_millis() as u64;

    // Log search to analytics database (non-blocking)
    let analytics_db = state.analytics_db.read().await.clone();
    if let Some(db) = analytics_db {
        let query = request.query.clone();
        let results_count = results.len();
        let avg_score = if !results.is_empty() {
            Some(results.iter().map(|r| r.score).sum::<f32>() / results.len() as f32)
        } else {
            None
        };
        let duration = duration_ms;

        // Spawn async task to log search
        tokio::spawn(async move {
            use crate::analytics::{SearchHistoryEntry};
            use chrono::Utc;
            use uuid::Uuid;

            let entry = SearchHistoryEntry {
                id: Uuid::new_v4(),
                query,
                top_k: request.top_k,
                results_count,
                avg_score,
                duration_ms: duration,
                client_type: "http".to_string(),
                client_id: None,
                session_id: None,
                timestamp: Utc::now(),
            };

            if let Err(e) = db.log_search(&entry).await {
                tracing::warn!("Failed to log search to analytics: {}", e);
            }
        });
    }

    Ok(Json(SearchResponse {
        results,
        query_info: Some(QueryInfo {
            normalized: request.query.clone(),
            intent: "search".to_string(),
            confidence: 0.9,
        }),
        duration_ms,
    }))
}

/// Get search configuration
pub async fn get_search_config(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SearchConfigResponse>, (StatusCode, Json<ApiError>)> {
    // Get indexed document count from search pipeline if available
    let indexed_documents = if let Some(ref pipeline) = *state.search_pipeline.read().await {
        pipeline.document_count().await.unwrap_or(0)
    } else {
        0
    };

    // Return current search configuration
    Ok(Json(SearchConfigResponse {
        embedding_provider: "fastembed".to_string(),
        embedding_model: "all-minilm".to_string(),
        dimensions: 384,
        vector_backend: "file".to_string(),
        hybrid_search_enabled: true,
        reranking_enabled: false,
        indexed_documents,
    }))
}

/// Update search configuration
pub async fn update_search_config(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<UpdateSearchConfigRequest>,
) -> Result<Json<SearchConfigResponse>, (StatusCode, Json<ApiError>)> {
    info!("Updating search configuration: {:?}", request);

    // TODO: Actually update the search configuration
    // For now, just return the updated config

    Ok(Json(SearchConfigResponse {
        embedding_provider: request.embedding_provider.unwrap_or_else(|| "fastembed".to_string()),
        embedding_model: request.embedding_model.unwrap_or_else(|| "BAAI/bge-small-en-v1.5".to_string()),
        dimensions: 384,
        vector_backend: request.vector_backend.unwrap_or_else(|| "inmemory".to_string()),
        hybrid_search_enabled: request.enable_hybrid.unwrap_or(false),
        reranking_enabled: request.enable_reranking.unwrap_or(false),
        indexed_documents: 0,
    }))
}

/// Get application configuration
pub async fn get_config(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AppConfig>, (StatusCode, Json<ApiError>)> {
    // Get indexed document count from search pipeline if available
    let indexed_documents = if let Some(ref pipeline) = *state.search_pipeline.read().await {
        pipeline.document_count().await.unwrap_or(0)
    } else {
        0
    };

    Ok(Json(AppConfig {
        default_timeout_secs: 30,
        max_concurrent_executions: 10,
        enable_history: true,
        max_history_entries: 1000,
        search: SearchConfigResponse {
            embedding_provider: "fastembed".to_string(),
            embedding_model: "all-minilm".to_string(),
            dimensions: 384,
            vector_backend: "file".to_string(),
            hybrid_search_enabled: true,
            reranking_enabled: false,
            indexed_documents,
        },
    }))
}

/// Update application configuration
pub async fn update_config(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<UpdateAppConfigRequest>,
) -> Result<Json<AppConfig>, (StatusCode, Json<ApiError>)> {
    info!("Updating app configuration: {:?}", request);

    // TODO: Actually persist configuration changes
    Ok(Json(AppConfig {
        default_timeout_secs: request.default_timeout_secs.unwrap_or(30),
        max_concurrent_executions: request.max_concurrent_executions.unwrap_or(10),
        enable_history: request.enable_history.unwrap_or(true),
        max_history_entries: request.max_history_entries.unwrap_or(1000),
        search: SearchConfigResponse {
            embedding_provider: "fastembed".to_string(),
            embedding_model: "BAAI/bge-small-en-v1.5".to_string(),
            dimensions: 384,
            vector_backend: "inmemory".to_string(),
            hybrid_search_enabled: false,
            reranking_enabled: false,
            indexed_documents: 0,
        },
    }))
}

/// Health check endpoint
pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HealthResponse>, (StatusCode, Json<ApiError>)> {
    let uptime = state.started_at.elapsed().as_secs();

    let mut components = HashMap::new();

    // Check skill engine
    components.insert(
        "skill_engine".to_string(),
        ComponentHealth {
            name: "Skill Engine".to_string(),
            healthy: true,
            message: None,
        },
    );

    // Check search pipeline (if initialized)
    components.insert(
        "search_pipeline".to_string(),
        ComponentHealth {
            name: "Search Pipeline".to_string(),
            healthy: true,
            message: Some("Using in-memory vector store".to_string()),
        },
    );

    let all_healthy = components.values().all(|c| c.healthy);

    Ok(Json(HealthResponse {
        status: if all_healthy { "healthy".to_string() } else { "degraded".to_string() },
        healthy: all_healthy,
        components,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: uptime,
    }))
}

/// Version information endpoint
pub async fn version_info() -> Json<VersionResponse> {
    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        build: option_env!("BUILD_DATE").map(String::from),
        commit: option_env!("GIT_COMMIT").map(String::from),
        rust_version: option_env!("RUST_VERSION").map(String::from),
        wasmtime_version: "26.0".to_string(),
    })
}

/// Fallback handler for 404
pub async fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(ApiError::not_found("Endpoint")),
    )
}

// =============================================================================
// Manifest Import/Export Handlers
// =============================================================================

/// Validate a manifest without importing
pub async fn validate_manifest(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<ValidateManifestRequest>,
) -> Result<Json<ValidateManifestResponse>, (StatusCode, Json<ApiError>)> {
    info!("Validating manifest ({} bytes)", request.content.len());

    // Parse the TOML content
    let parsed: Result<toml::Value, _> = toml::from_str(&request.content);

    match parsed {
        Ok(value) => {
            let (skills, warnings) = parse_manifest_skills(&value);

            Ok(Json(ValidateManifestResponse {
                valid: true,
                skills,
                errors: vec![],
                warnings,
            }))
        }
        Err(e) => {
            Ok(Json(ValidateManifestResponse {
                valid: false,
                skills: vec![],
                errors: vec![format!("TOML parse error: {}", e)],
                warnings: vec![],
            }))
        }
    }
}

/// Import a manifest configuration
pub async fn import_manifest(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ImportManifestRequest>,
) -> Result<Json<ImportManifestResponse>, (StatusCode, Json<ApiError>)> {
    info!(
        "Importing manifest ({} bytes, merge={}, install={})",
        request.content.len(),
        request.merge,
        request.install
    );

    // Parse the TOML content
    let parsed: Result<toml::Value, _> = toml::from_str(&request.content);

    match parsed {
        Ok(value) => {
            let (skills, warnings) = parse_manifest_skills(&value);
            let skills_count = skills.len();
            let mut installed_count = 0;
            let errors: Vec<String> = vec![];

            if request.install {
                // Add skills to the state
                let mut state_skills = state.skills.write().await;

                if !request.merge {
                    // Clear existing skills if not merging
                    state_skills.clear();
                }

                for skill in &skills {
                    let skill_summary = SkillSummary {
                        name: skill.name.clone(),
                        version: "0.1.0".to_string(),
                        description: skill.description.clone().unwrap_or_default(),
                        source: skill.source.clone(),
                        runtime: skill.runtime.clone(),
                        tools_count: 0,
                        instances_count: skill.instances.len(),
                        last_used: None,
                        execution_count: 0,
                        required_services: Vec::new(),
                    };

                    state_skills.insert(skill.name.clone(), skill_summary);
                    installed_count += 1;
                }
            }

            Ok(Json(ImportManifestResponse {
                success: true,
                skills,
                skills_count,
                installed_count,
                warnings,
                errors,
            }))
        }
        Err(e) => {
            Ok(Json(ImportManifestResponse {
                success: false,
                skills: vec![],
                skills_count: 0,
                installed_count: 0,
                warnings: vec![],
                errors: vec![format!("TOML parse error: {}", e)],
            }))
        }
    }
}

/// Export current configuration as manifest
pub async fn export_manifest(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ExportManifestRequest>,
) -> Result<Json<ExportManifestResponse>, (StatusCode, Json<ApiError>)> {
    info!("Exporting manifest (format={})", request.format);

    let skills = state.skills.read().await;
    let skills_count = skills.len();

    // Build TOML manifest
    let mut content = String::from("# Skill Engine Manifest\nversion = \"1\"\n\n");

    for skill in skills.values() {
        content.push_str(&format!(
            "[skills.{}]\n",
            skill.name.replace('-', "_")
        ));
        content.push_str(&format!("source = \"{}\"\n", skill.source));
        if !skill.description.is_empty() {
            content.push_str(&format!("description = \"{}\"\n", skill.description));
        }
        if skill.runtime != "wasm" {
            content.push_str(&format!("runtime = \"{}\"\n", skill.runtime));
        }
        content.push_str("\n[skills.");
        content.push_str(&skill.name.replace('-', "_"));
        content.push_str(".instances.default]\n");
        content.push_str("# Add configuration here\n\n");
    }

    let format = if request.format == "json" {
        // Convert to JSON if requested
        let json_content = serde_json::json!({
            "version": "1",
            "skills": skills.values().map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "source": s.source,
                    "description": s.description,
                    "runtime": s.runtime,
                })
            }).collect::<Vec<_>>()
        });
        content = serde_json::to_string_pretty(&json_content).unwrap_or_default();
        "json".to_string()
    } else {
        "toml".to_string()
    };

    Ok(Json(ExportManifestResponse {
        content,
        format,
        skills_count,
    }))
}

/// Parse skills from a TOML manifest value
fn parse_manifest_skills(value: &toml::Value) -> (Vec<ParsedSkill>, Vec<String>) {
    let mut skills = vec![];
    let mut warnings = vec![];

    if let Some(skills_table) = value.get("skills").and_then(|v| v.as_table()) {
        for (name, skill_value) in skills_table {
            if let Some(skill_table) = skill_value.as_table() {
                let source = skill_table
                    .get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                if source.is_empty() {
                    warnings.push(format!("Skill '{}' has no source defined", name));
                    continue;
                }

                let runtime = skill_table
                    .get("runtime")
                    .and_then(|v| v.as_str())
                    .unwrap_or("wasm")
                    .to_string();

                let description = skill_table
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                // Parse instances
                let mut instances = vec![];
                if let Some(instances_table) = skill_table.get("instances").and_then(|v| v.as_table()) {
                    let _instance_names: Vec<_> = instances_table.keys().collect();
                    for (idx, (instance_name, instance_value)) in instances_table.iter().enumerate() {
                        let mut config_keys = vec![];
                        let mut env_keys = vec![];

                        if let Some(inst_table) = instance_value.as_table() {
                            // Get config keys
                            if let Some(config) = inst_table.get("config").and_then(|v| v.as_table()) {
                                config_keys = config.keys().cloned().collect();
                            }
                            // Get env keys
                            if let Some(env) = inst_table.get("env").and_then(|v| v.as_table()) {
                                env_keys = env.keys().cloned().collect();
                            }
                        }

                        instances.push(ParsedInstance {
                            name: instance_name.clone(),
                            config_keys,
                            env_keys,
                            is_default: idx == 0 || instance_name == "default",
                        });
                    }
                }

                // If no instances defined, add a default one
                if instances.is_empty() {
                    instances.push(ParsedInstance {
                        name: "default".to_string(),
                        config_keys: vec![],
                        env_keys: vec![],
                        is_default: true,
                    });
                }

                // Parse docker config if present
                let docker_config = skill_table.get("docker").and_then(|v| v.as_table()).map(|docker| {
                    DockerConfig {
                        image: docker.get("image").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        entrypoint: docker.get("entrypoint").and_then(|v| v.as_str()).map(String::from),
                        volumes: docker
                            .get("volumes")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                            .unwrap_or_default(),
                        working_dir: docker.get("working_dir").and_then(|v| v.as_str()).map(String::from),
                        memory: docker.get("memory").and_then(|v| v.as_str()).map(String::from),
                        cpus: docker.get("cpus").and_then(|v| v.as_str()).map(String::from),
                        network: docker.get("network").and_then(|v| v.as_str()).map(String::from),
                    }
                });

                skills.push(ParsedSkill {
                    name: name.clone(),
                    source,
                    runtime,
                    description,
                    instances,
                    docker_config,
                });
            }
        }
    } else {
        warnings.push("No 'skills' section found in manifest".to_string());
    }

    (skills, warnings)
}

// =============================================================================
// System Service Handlers
// =============================================================================

use crate::types::{
    ServiceStatus, ServicesStatusResponse, StartServiceRequest, StartServiceResponse,
    StopServiceRequest,
};
use crate::server::TrackedService;

/// List all system services and their status
pub async fn list_services(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ServicesStatusResponse>, (axum::http::StatusCode, Json<ApiError>)> {
    let services = state.services.read().await;

    let mut service_statuses = vec![];

    // Check kubectl proxy status
    let kubectl_proxy_status = if let Some(service) = services.get("kubectl-proxy") {
        ServiceStatus {
            name: "kubectl-proxy".to_string(),
            running: service.process.is_some(),
            pid: None, // We don't track PID in Child in a clean way
            port: Some(service.port),
            url: Some(format!("http://127.0.0.1:{}", service.port)),
            error: None,
        }
    } else {
        // Check if kubectl proxy is running externally
        let port = 8001u16;
        let running = check_port_in_use(port);
        ServiceStatus {
            name: "kubectl-proxy".to_string(),
            running,
            pid: None,
            port: if running { Some(port) } else { None },
            url: if running { Some(format!("http://127.0.0.1:{}", port)) } else { None },
            error: None,
        }
    };
    service_statuses.push(kubectl_proxy_status);

    Ok(Json(ServicesStatusResponse {
        services: service_statuses,
    }))
}

/// Start a system service
pub async fn start_service(
    State(state): State<Arc<AppState>>,
    Json(request): Json<StartServiceRequest>,
) -> Result<Json<StartServiceResponse>, (axum::http::StatusCode, Json<ApiError>)> {
    match request.service.as_str() {
        "kubectl-proxy" => start_kubectl_proxy(state, request.port.unwrap_or(8001)).await,
        _ => Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(ApiError::bad_request(format!("Unknown service: {}", request.service))),
        )),
    }
}

/// Stop a system service
pub async fn stop_service(
    State(state): State<Arc<AppState>>,
    Json(request): Json<StopServiceRequest>,
) -> Result<Json<StartServiceResponse>, (axum::http::StatusCode, Json<ApiError>)> {
    match request.service.as_str() {
        "kubectl-proxy" => stop_kubectl_proxy(state).await,
        _ => Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(ApiError::bad_request(format!("Unknown service: {}", request.service))),
        )),
    }
}

/// Start kubectl proxy service
async fn start_kubectl_proxy(
    state: Arc<AppState>,
    port: u16,
) -> Result<Json<StartServiceResponse>, (axum::http::StatusCode, Json<ApiError>)> {
    // Check if already running
    {
        let services = state.services.read().await;
        if let Some(service) = services.get("kubectl-proxy") {
            if service.process.is_some() {
                return Ok(Json(StartServiceResponse {
                    success: true,
                    status: ServiceStatus {
                        name: "kubectl-proxy".to_string(),
                        running: true,
                        pid: None,
                        port: Some(service.port),
                        url: Some(format!("http://127.0.0.1:{}", service.port)),
                        error: None,
                    },
                    message: "kubectl proxy is already running".to_string(),
                }));
            }
        }
    }

    // Check if port is already in use (external kubectl proxy)
    if check_port_in_use(port) {
        return Ok(Json(StartServiceResponse {
            success: true,
            status: ServiceStatus {
                name: "kubectl-proxy".to_string(),
                running: true,
                pid: None,
                port: Some(port),
                url: Some(format!("http://127.0.0.1:{}", port)),
                error: None,
            },
            message: format!("kubectl proxy already running externally on port {}", port),
        }));
    }

    // Start kubectl proxy
    let result = std::process::Command::new("kubectl")
        .args(["proxy", "--port", &port.to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();

    match result {
        Ok(child) => {
            // Wait a moment for the proxy to start
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            // Check if it's actually running
            let running = check_port_in_use(port);

            if running {
                let mut services = state.services.write().await;
                services.insert("kubectl-proxy".to_string(), TrackedService {
                    name: "kubectl-proxy".to_string(),
                    process: Some(child),
                    port,
                });

                // Set environment variable for skills
                std::env::set_var("KUBECTL_PROXY_URL", format!("http://127.0.0.1:{}", port));

                Ok(Json(StartServiceResponse {
                    success: true,
                    status: ServiceStatus {
                        name: "kubectl-proxy".to_string(),
                        running: true,
                        pid: None,
                        port: Some(port),
                        url: Some(format!("http://127.0.0.1:{}", port)),
                        error: None,
                    },
                    message: format!("kubectl proxy started on port {}", port),
                }))
            } else {
                Ok(Json(StartServiceResponse {
                    success: false,
                    status: ServiceStatus {
                        name: "kubectl-proxy".to_string(),
                        running: false,
                        pid: None,
                        port: None,
                        url: None,
                        error: Some("kubectl proxy failed to start. Is kubectl installed and configured?".to_string()),
                    },
                    message: "kubectl proxy failed to start".to_string(),
                }))
            }
        }
        Err(e) => {
            Ok(Json(StartServiceResponse {
                success: false,
                status: ServiceStatus {
                    name: "kubectl-proxy".to_string(),
                    running: false,
                    pid: None,
                    port: None,
                    url: None,
                    error: Some(format!("Failed to start kubectl proxy: {}. Is kubectl installed?", e)),
                },
                message: format!("Failed to start kubectl proxy: {}", e),
            }))
        }
    }
}

/// Stop kubectl proxy service
async fn stop_kubectl_proxy(
    state: Arc<AppState>,
) -> Result<Json<StartServiceResponse>, (axum::http::StatusCode, Json<ApiError>)> {
    let mut services = state.services.write().await;

    if let Some(mut service) = services.remove("kubectl-proxy") {
        if let Some(mut child) = service.process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }

        // Unset environment variable
        std::env::remove_var("KUBECTL_PROXY_URL");

        Ok(Json(StartServiceResponse {
            success: true,
            status: ServiceStatus {
                name: "kubectl-proxy".to_string(),
                running: false,
                pid: None,
                port: None,
                url: None,
                error: None,
            },
            message: "kubectl proxy stopped".to_string(),
        }))
    } else {
        Ok(Json(StartServiceResponse {
            success: true,
            status: ServiceStatus {
                name: "kubectl-proxy".to_string(),
                running: false,
                pid: None,
                port: None,
                url: None,
                error: None,
            },
            message: "kubectl proxy was not running".to_string(),
        }))
    }
}

/// Check if a port is in use
fn check_port_in_use(port: u16) -> bool {
    std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok()
}

// =============================================================================
// Vector DB Testing Handlers
// =============================================================================

/// Test search connection (quick validation)
pub async fn test_search_connection(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<TestConnectionRequest>,
) -> Result<Json<TestConnectionResponse>, (StatusCode, Json<ApiError>)> {
    info!(
        "Testing search connection: provider={}, backend={}",
        request.embedding_provider, request.vector_backend
    );

    let start = Instant::now();

    // Test embedding provider connectivity
    let embedding_provider_status = test_embedding_provider(
        &request.embedding_provider,
        &request.embedding_model,
        request.ollama_url.as_deref(),
    )
    .await;

    // Test vector backend connectivity
    let vector_backend_status = test_vector_backend(
        &request.vector_backend,
        request.qdrant_url.as_deref(),
    )
    .await;

    let duration_ms = start.elapsed().as_millis();
    let success = embedding_provider_status.healthy && vector_backend_status.healthy;

    let message = if success {
        "All components are healthy".to_string()
    } else {
        let mut errors = Vec::new();
        if !embedding_provider_status.healthy {
            errors.push(format!(
                "Embedding provider: {}",
                embedding_provider_status.message.as_ref().unwrap_or(&"unhealthy".to_string())
            ));
        }
        if !vector_backend_status.healthy {
            errors.push(format!(
                "Vector backend: {}",
                vector_backend_status.message.as_ref().unwrap_or(&"unhealthy".to_string())
            ));
        }
        errors.join("; ")
    };

    Ok(Json(TestConnectionResponse {
        success,
        embedding_provider_status,
        vector_backend_status,
        duration_ms,
        message,
    }))
}

/// Test embedding provider connectivity
async fn test_embedding_provider(
    provider: &str,
    model: &str,
    ollama_url: Option<&str>,
) -> ComponentHealth {
    use skill_runtime::embeddings::{EmbeddingConfig, EmbeddingProviderFactory, EmbeddingProviderType};

    let provider_type: EmbeddingProviderType = provider.parse().unwrap_or(EmbeddingProviderType::FastEmbed);

    let config = EmbeddingConfig {
        provider: provider_type,
        model: Some(model.to_string()),
        api_key: std::env::var("OPENAI_API_KEY").ok(),
        base_url: ollama_url.map(String::from),
        batch_size: 100,
    };

    match EmbeddingProviderFactory::create(&config) {
        Ok(embedding_provider) => {
            // Try a simple embedding to verify it works
            match embedding_provider.embed_query("test").await {
                Ok(embedding) => {
                    debug!("Embedding provider test successful: {} dimensions", embedding.len());
                    ComponentHealth {
                        name: format!("{} ({})", embedding_provider.provider_name(), embedding_provider.model_name()),
                        healthy: true,
                        message: Some(format!("Connected successfully ({} dimensions)", embedding.len())),
                    }
                }
                Err(e) => {
                    warn!("Embedding provider test failed: {}", e);
                    ComponentHealth {
                        name: format!("{} / {}", provider, model),
                        healthy: false,
                        message: Some(format!("Failed to generate test embedding: {}", e)),
                    }
                }
            }
        }
        Err(e) => {
            warn!("Failed to create embedding provider: {}", e);
            ComponentHealth {
                name: format!("{} / {}", provider, model),
                healthy: false,
                message: Some(format!("Failed to initialize: {}", e)),
            }
        }
    }
}

/// Test vector backend connectivity
async fn test_vector_backend(backend: &str, _qdrant_url: Option<&str>) -> ComponentHealth {
    use skill_runtime::search_config::BackendType;

    let backend_type: BackendType = backend.parse().unwrap_or(BackendType::InMemory);

    match backend_type {
        BackendType::File => {
            // File backend is always available
            ComponentHealth {
                name: "File-based Vector Store".to_string(),
                healthy: true,
                message: Some("File-based backend is always available".to_string()),
            }
        }
        BackendType::InMemory => {
            // InMemory is always available
            ComponentHealth {
                name: "In-Memory Vector Store".to_string(),
                healthy: true,
                message: Some("In-memory backend is always available".to_string()),
            }
        }
        #[cfg(feature = "qdrant")]
        BackendType::Qdrant => {
            use skill_runtime::vector_store::{QdrantVectorStore, QdrantConfig};

            let url = qdrant_url.unwrap_or("http://localhost:6334");

            let config = QdrantConfig {
                url: url.to_string(),
                api_key: std::env::var("QDRANT_API_KEY").ok(),
                collection_name: "test_connection".to_string(),
                vector_size: 384,
                ..Default::default()
            };

            // Try to create a Qdrant client
            match QdrantVectorStore::new(config).await {
                Ok(_store) => {
                    debug!("Qdrant connection test successful");
                    ComponentHealth {
                        name: "Qdrant Vector Store".to_string(),
                        healthy: true,
                        message: Some(format!("Connected to Qdrant at {}", url)),
                    }
                }
                Err(e) => {
                    warn!("Qdrant connection test failed: {}", e);
                    ComponentHealth {
                        name: "Qdrant Vector Store".to_string(),
                        healthy: false,
                        message: Some(format!("Failed to connect to Qdrant: {}", e)),
                    }
                }
            }
        }
        #[cfg(not(feature = "qdrant"))]
        BackendType::Qdrant => {
            ComponentHealth {
                name: "Qdrant Vector Store".to_string(),
                healthy: false,
                message: Some("Qdrant feature not enabled in this build".to_string()),
            }
        }
    }
}

/// Test full search pipeline (indexing + search)
pub async fn test_search_pipeline(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<TestPipelineRequest>,
) -> Result<Json<TestPipelineResponse>, (StatusCode, Json<ApiError>)> {
    info!(
        "Testing full search pipeline: provider={}, backend={}, hybrid={}, reranking={}",
        request.embedding_provider, request.vector_backend, request.enable_hybrid, request.enable_reranking
    );

    let start = Instant::now();

    // Build search configuration
    use skill_runtime::search_config::{
        SearchConfig, BackendConfig, EmbeddingConfig as RuntimeEmbeddingConfig,
        RetrievalConfig, RerankerConfig, ContextConfig, QdrantConfig as RuntimeQdrantConfig,
        BackendType, IndexConfig, AiIngestionConfig,
    };

    let backend_type: BackendType = request.vector_backend.parse().unwrap_or(BackendType::InMemory);

    let config = SearchConfig {
        backend: BackendConfig {
            backend_type,
            ..Default::default()
        },
        embedding: RuntimeEmbeddingConfig {
            provider: request.embedding_provider.clone(),
            model: request.embedding_model.clone(),
            dimensions: 384, // Default for most models
            batch_size: 100,
            openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
            ollama_host: None,
        },
        retrieval: RetrievalConfig {
            enable_hybrid: request.enable_hybrid,
            ..Default::default()
        },
        reranker: RerankerConfig {
            enabled: request.enable_reranking,
            ..Default::default()
        },
        context: ContextConfig::default(),
        file: None, // Use default file config
        qdrant: if backend_type == BackendType::Qdrant {
            Some(RuntimeQdrantConfig {
                url: request.qdrant_url.unwrap_or_else(|| "http://localhost:6334".to_string()),
                api_key: std::env::var("QDRANT_API_KEY").ok(),
                collection: "skill_test_pipeline".to_string(),
                ..Default::default()
            })
        } else {
            None
        },
        index: IndexConfig::default(),
        ai_ingestion: AiIngestionConfig::default(),
    };

    // Create temporary pipeline
    use skill_runtime::search::{SearchPipeline, IndexDocument};
    use skill_runtime::vector_store::DocumentMetadata as RuntimeDocMetadata;

    let pipeline = match SearchPipeline::from_config(config).await {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to create search pipeline: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::internal(format!("Failed to create pipeline: {}", e))),
            ));
        }
    };

    // Create sample test documents
    let sample_docs = vec![
        IndexDocument {
            id: "test-1".to_string(),
            content: "List all Kubernetes pods in the default namespace".to_string(),
            metadata: RuntimeDocMetadata {
                skill_name: Some("kubernetes".to_string()),
                tool_name: Some("list-pods".to_string()),
                ..Default::default()
            },
        },
        IndexDocument {
            id: "test-2".to_string(),
            content: "Deploy a new application to Kubernetes cluster".to_string(),
            metadata: RuntimeDocMetadata {
                skill_name: Some("kubernetes".to_string()),
                tool_name: Some("apply".to_string()),
                ..Default::default()
            },
        },
        IndexDocument {
            id: "test-3".to_string(),
            content: "Get AWS S3 bucket list and configuration".to_string(),
            metadata: RuntimeDocMetadata {
                skill_name: Some("aws".to_string()),
                tool_name: Some("s3-list-buckets".to_string()),
                ..Default::default()
            },
        },
        IndexDocument {
            id: "test-4".to_string(),
            content: "Execute SQL query on PostgreSQL database".to_string(),
            metadata: RuntimeDocMetadata {
                skill_name: Some("postgres".to_string()),
                tool_name: Some("query".to_string()),
                ..Default::default()
            },
        },
        IndexDocument {
            id: "test-5".to_string(),
            content: "List all running Docker containers".to_string(),
            metadata: RuntimeDocMetadata {
                skill_name: Some("docker".to_string()),
                tool_name: Some("ps".to_string()),
                ..Default::default()
            },
        },
    ];

    let indexing_start = Instant::now();

    // Index documents
    let index_stats = match pipeline.index_documents(sample_docs).await {
        Ok(stats) => stats,
        Err(e) => {
            warn!("Failed to index documents: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::internal(format!("Failed to index documents: {}", e))),
            ));
        }
    };

    let indexing_duration_ms = indexing_start.elapsed().as_millis() as u64;

    // Perform test search
    let search_results = match pipeline.search("kubernetes pods", 3).await {
        Ok(results) => results,
        Err(e) => {
            warn!("Search failed: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::internal(format!("Search failed: {}", e))),
            ));
        }
    };

    let duration_ms = start.elapsed().as_millis();

    // Convert results to response format
    let pipeline_results: Vec<PipelineSearchResult> = search_results
        .into_iter()
        .map(|r| PipelineSearchResult {
            id: r.id,
            content: r.content,
            score: r.score,
            rerank_score: r.rerank_score,
            metadata: DocumentMetadata {
                skill_name: r.metadata.skill_name,
                tool_name: r.metadata.tool_name,
                tags: r.metadata.tags,
            },
        })
        .collect();

    let results_count = pipeline_results.len();

    Ok(Json(TestPipelineResponse {
        success: true,
        index_stats: PipelineIndexStats {
            documents_indexed: index_stats.documents_added,
            indexing_duration_ms,
            embedding_duration_ms: indexing_duration_ms, // Approximation
        },
        search_results: pipeline_results,
        duration_ms,
        message: format!(
            "Pipeline test completed: indexed {} documents, found {} results",
            index_stats.documents_added,
            results_count
        ),
    }))
}

// =============================================================================
// Agent Configuration Handlers
// =============================================================================

/// Get agent configuration
pub async fn get_agent_config(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<GetAgentConfigResponse>, (StatusCode, Json<ApiError>)> {
    info!("Getting agent configuration");

    // Detect Claude Code on system
    let (claude_code_detected, claude_code_version) = detect_claude_code().await;

    // Build configuration response
    let config = AgentConfig::default();

    let available_runtimes = vec![
        RuntimeInfo {
            runtime: AgentRuntime::ClaudeCode,
            name: "Claude Code".to_string(),
            description: "Anthropic's Claude with extended thinking and tool use via system installation".to_string(),
            supported_providers: vec!["anthropic".to_string()],
            available: claude_code_detected,
        },
        RuntimeInfo {
            runtime: AgentRuntime::Gemini,
            name: "Google Gemini".to_string(),
            description: "Google's multimodal AI with code execution".to_string(),
            supported_providers: vec!["google".to_string()],
            available: false, // Not yet implemented
        },
        RuntimeInfo {
            runtime: AgentRuntime::OpenAI,
            name: "OpenAI GPT".to_string(),
            description: "OpenAI's GPT models with function calling".to_string(),
            supported_providers: vec!["openai".to_string()],
            available: false, // Not yet implemented
        },
    ];

    let available_models = get_available_models();

    Ok(Json(GetAgentConfigResponse {
        config,
        available_runtimes,
        available_models,
        claude_code_detected,
        claude_code_version,
    }))
}

/// Update agent configuration
pub async fn update_agent_config(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<UpdateAgentConfigRequest>,
) -> Result<Json<AgentConfig>, (StatusCode, Json<ApiError>)> {
    info!("Updating agent configuration");

    let mut config = AgentConfig::default();

    // Apply updates
    if let Some(runtime) = request.runtime {
        config.runtime = runtime;
    }

    if let Some(model_config) = request.model_config {
        config.model_config = model_config;
    }

    if let Some(timeout_secs) = request.timeout_secs {
        config.timeout_secs = timeout_secs;
    }

    if let Some(claude_code_path) = request.claude_code_path {
        config.claude_code_path = Some(claude_code_path);
    }

    // TODO: Persist configuration to disk

    Ok(Json(config))
}

/// Detect Claude Code installation on the system
async fn detect_claude_code() -> (bool, Option<String>) {
    use tokio::process::Command;

    // Try to detect claude command
    match Command::new("claude")
        .arg("--version")
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            debug!("Detected Claude Code: {}", version);
            (true, Some(version))
        }
        _ => {
            // Try alternative paths
            let alternative_paths = vec![
                "/usr/local/bin/claude",
                "/opt/homebrew/bin/claude",
                "~/.local/bin/claude",
            ];

            for path in alternative_paths {
                match Command::new(path)
                    .arg("--version")
                    .output()
                    .await
                {
                    Ok(output) if output.status.success() => {
                        let version = String::from_utf8_lossy(&output.stdout)
                            .trim()
                            .to_string();
                        debug!("Detected Claude Code at {}: {}", path, version);
                        return (true, Some(version));
                    }
                    _ => continue,
                }
            }

            debug!("Claude Code not detected on system");
            (false, None)
        }
    }
}

/// Get available LLM models by provider
fn get_available_models() -> HashMap<String, Vec<ModelInfo>> {
    let mut models = HashMap::new();

    // Anthropic models
    models.insert(
        "anthropic".to_string(),
        vec![
            ModelInfo {
                id: "claude-sonnet-4".to_string(),
                name: "Claude Sonnet 4".to_string(),
                max_tokens: 200_000,
                supports_tools: true,
            },
            ModelInfo {
                id: "claude-opus-4".to_string(),
                name: "Claude Opus 4".to_string(),
                max_tokens: 200_000,
                supports_tools: true,
            },
            ModelInfo {
                id: "claude-haiku-3.5".to_string(),
                name: "Claude Haiku 3.5".to_string(),
                max_tokens: 200_000,
                supports_tools: true,
            },
        ],
    );

    // OpenAI models
    models.insert(
        "openai".to_string(),
        vec![
            ModelInfo {
                id: "gpt-4o".to_string(),
                name: "GPT-4o".to_string(),
                max_tokens: 128_000,
                supports_tools: true,
            },
            ModelInfo {
                id: "gpt-4-turbo".to_string(),
                name: "GPT-4 Turbo".to_string(),
                max_tokens: 128_000,
                supports_tools: true,
            },
            ModelInfo {
                id: "gpt-3.5-turbo".to_string(),
                name: "GPT-3.5 Turbo".to_string(),
                max_tokens: 16_000,
                supports_tools: true,
            },
        ],
    );

    // Google models
    models.insert(
        "google".to_string(),
        vec![
            ModelInfo {
                id: "gemini-2.0-flash-exp".to_string(),
                name: "Gemini 2.0 Flash (Experimental)".to_string(),
                max_tokens: 1_000_000,
                supports_tools: true,
            },
            ModelInfo {
                id: "gemini-1.5-pro".to_string(),
                name: "Gemini 1.5 Pro".to_string(),
                max_tokens: 2_000_000,
                supports_tools: true,
            },
            ModelInfo {
                id: "gemini-1.5-flash".to_string(),
                name: "Gemini 1.5 Flash".to_string(),
                max_tokens: 1_000_000,
                supports_tools: true,
            },
        ],
    );

    models
}

/// Index all skills into the search pipeline
pub async fn index_skills(
    State(state): State<Arc<AppState>>,
) -> Result<Json<IndexResponse>, (StatusCode, Json<ApiError>)> {
    info!("Starting skill indexing");

    let start = Instant::now();

    // Ensure pipeline is initialized
    {
        let pipeline = state.search_pipeline.read().await;
        if pipeline.is_none() {
            info!("Initializing search pipeline");
            drop(pipeline);
            if let Err(e) = state.initialize_search_pipeline().await {
                warn!("Failed to initialize search pipeline: {}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiError::internal(format!("Failed to initialize search pipeline: {}", e))),
                ));
            }
        }
    }

    // Get skills
    let skills = state.skills.read().await;
    if skills.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::validation("No skills loaded. Load skills first before indexing.")),
        ));
    }

    // Build documents from skills by loading tool information
    use skill_runtime::search::IndexDocument;
    use skill_runtime::vector_store::DocumentMetadata as RuntimeDocMetadata;

    let mut documents = Vec::new();
    let mut _loaded_tools = 0;

    // For each skill, we need to load the actual tools
    for (skill_name, skill_summary) in skills.iter() {
        // Try to load skill details to get tools
        let manifest = state.manifest.read().await;
        if let Some(manifest) = manifest.as_ref() {
            if let Some(skill_def) = manifest.skills.get(skill_name) {
                // Build source path
                let source_path = if skill_def.source.starts_with("./") || skill_def.source.starts_with('/') {
                    manifest.base_dir.join(&skill_def.source)
                } else {
                    let home = dirs::home_dir().unwrap_or_default();
                    home.join(".skill-engine").join("registry").join(skill_name)
                };

                // Load tools from SKILL.md if available
                if source_path.exists() {
                    use skill_runtime::skill_md::find_skill_md;
                    if let Some(skill_md_path) = find_skill_md(&source_path) {
                        if let Ok(skill_content) = skill_runtime::skill_md::parse_skill_md(&skill_md_path) {
                            // Create documents from tools (tool_docs is HashMap<String, ToolDocumentation>)
                            for (_tool_name, tool_doc) in skill_content.tool_docs {
                                let params_text = tool_doc.parameters.iter()
                                    .map(|p| format!("{}: {}", p.name, p.description))
                                    .collect::<Vec<_>>()
                                    .join(", ");

                                let content = format!(
                                    "{} - {} | {} | Parameters: {}",
                                    skill_name,
                                    tool_doc.name,
                                    tool_doc.description,
                                    params_text
                                );

                                documents.push(IndexDocument {
                                    id: format!("{}:{}", skill_name, tool_doc.name),
                                    content,
                                    metadata: RuntimeDocMetadata {
                                        skill_name: Some(skill_name.clone()),
                                        tool_name: Some(tool_doc.name.clone()),
                                        instance_name: None,
                                        category: Some(skill_summary.runtime.clone()),
                                        tags: vec![],
                                        custom: std::collections::HashMap::new(),
                                    },
                                });
                                _loaded_tools += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    let doc_count = documents.len();
    drop(skills);

    if doc_count == 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::validation("No tools found to index. Ensure skills have SKILL.md files.")),
        ));
    }

    // Index documents
    let pipeline = state.search_pipeline.read().await;
    let pipeline = pipeline.as_ref().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::internal("Search pipeline not initialized")),
        )
    })?;

    let pipeline_stats = pipeline.index_documents(documents).await.map_err(|e| {
        warn!("Failed to index documents: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::internal(format!("Failed to index documents: {}", e))),
        )
    })?;

    let duration_ms = start.elapsed().as_millis() as u64;

    info!(
        "Indexed {} documents in {}ms",
        doc_count, duration_ms
    );

    // Convert PipelineIndexStats to IndexStats
    use crate::types::IndexStats;
    let index_stats = IndexStats {
        documents_added: pipeline_stats.documents_added,
        documents_updated: pipeline_stats.documents_updated,
        total_documents: pipeline_stats.total_documents,
        index_size_bytes: pipeline_stats.index_size_bytes,
    };

    Ok(Json(IndexResponse {
        success: true,
        documents_indexed: doc_count,
        duration_ms,
        message: format!("Successfully indexed {} documents", doc_count),
        stats: index_stats,
    }))
}

// ============================================================================
// Feedback API Handlers
// ============================================================================

/// Submit feedback for a search result
pub async fn submit_feedback(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SubmitFeedbackRequest>,
) -> Result<Json<SubmitFeedbackResponse>, (StatusCode, Json<ApiError>)> {
    info!(
        query = %request.query,
        result_id = %request.result_id,
        feedback_type = %request.feedback_type,
        "Submitting feedback"
    );

    // Get analytics database
    let analytics_db = state.analytics_db.read().await;
    let db = match analytics_db.as_ref() {
        Some(db) => db.clone(),
        None => {
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiError::internal("Analytics database not initialized")),
            ));
        }
    };
    drop(analytics_db);

    // Validate feedback type
    use crate::analytics::FeedbackType;
    let feedback_type = FeedbackType::from_str(&request.feedback_type).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError::validation(format!(
                "Invalid feedback_type: {}. Must be 'positive' or 'negative'",
                request.feedback_type
            ))),
        )
    })?;

    // Create feedback entry
    use crate::analytics::SearchFeedbackEntry;
    use uuid::Uuid;
    let feedback_id = Uuid::new_v4();
    let entry = SearchFeedbackEntry {
        id: feedback_id,
        query: request.query.clone(),
        result_id: request.result_id.clone(),
        score: request.score,
        rank: request.rank,
        feedback_type,
        reason: request.reason.clone(),
        comment: request.comment.clone(),
        client_type: request.client_type.clone(),
        timestamp: chrono::Utc::now(),
    };

    // Log feedback to database
    db.log_feedback(&entry).await.map_err(|e| {
        error!("Failed to log feedback: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::internal(format!("Failed to save feedback: {}", e))),
        )
    })?;

    info!(feedback_id = %feedback_id, "Feedback logged successfully");

    Ok(Json(SubmitFeedbackResponse {
        success: true,
        feedback_id: feedback_id.to_string(),
        message: "Feedback submitted successfully".to_string(),
    }))
}

/// Get feedback with optional filters
pub async fn get_feedback(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(request): axum::extract::Query<GetFeedbackRequest>,
) -> Result<Json<GetFeedbackResponse>, (StatusCode, Json<ApiError>)> {
    info!(
        query = ?request.query,
        result_id = ?request.result_id,
        feedback_type = ?request.feedback_type,
        limit = request.limit,
        offset = request.offset,
        "Getting feedback"
    );

    // Get analytics database
    let analytics_db = state.analytics_db.read().await;
    let db = match analytics_db.as_ref() {
        Some(db) => db.clone(),
        None => {
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiError::internal("Analytics database not initialized")),
            ));
        }
    };
    drop(analytics_db);

    // Build filter
    use crate::analytics::{FeedbackFilter, FeedbackType};
    let feedback_type_filter = request
        .feedback_type
        .as_ref()
        .and_then(|ft| FeedbackType::from_str(ft));

    let filter = FeedbackFilter {
        query: request.query.clone(),
        result_id: request.result_id.clone(),
        feedback_type: feedback_type_filter,
        client_type: None,
        from_date: None,
        to_date: None,
        limit: Some(request.limit),
        offset: Some(request.offset),
    };

    // Get feedback from database
    let feedback_entries = db.get_feedback(&filter).await.map_err(|e| {
        error!("Failed to get feedback: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::internal(format!("Failed to retrieve feedback: {}", e))),
        )
    })?;

    // Convert to API response format
    use crate::types::FeedbackEntry;
    let feedback: Vec<FeedbackEntry> = feedback_entries
        .iter()
        .map(|entry| FeedbackEntry {
            id: entry.id.to_string(),
            query: entry.query.clone(),
            result_id: entry.result_id.clone(),
            score: entry.score,
            rank: entry.rank,
            feedback_type: entry.feedback_type.as_str().to_string(),
            reason: entry.reason.clone(),
            comment: entry.comment.clone(),
            client_type: entry.client_type.clone(),
            timestamp: entry.timestamp,
        })
        .collect();

    let total_count = feedback.len(); // Note: This is the count after filters, not total in DB

    info!(count = feedback.len(), "Retrieved feedback");

    Ok(Json(GetFeedbackResponse {
        feedback,
        total_count,
        limit: request.limit,
        offset: request.offset,
    }))
}

/// Get analytics overview
pub async fn get_analytics_overview(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<Json<AnalyticsOverviewResponse>, (StatusCode, Json<ApiError>)> {
    info!("Getting analytics overview");

    // Get analytics database
    let db = state
        .analytics_db
        .read()
        .await
        .as_ref()
        .ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiError::internal("Analytics database not available")),
            )
        })?
        .clone();

    // Parse days parameter (default 30)
    let days = params
        .get("days")
        .and_then(|d| d.parse::<u32>().ok())
        .unwrap_or(30);

    // Get overview stats
    let overview = db.get_overview(days).await.map_err(|e| {
        error!(error = %e, "Failed to get analytics overview");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::internal(format!(
                "Failed to get analytics overview: {}",
                e
            ))),
        )
    })?;

    // Get recent searches
    let recent_filter = crate::analytics::types::SearchHistoryFilter {
        limit: Some(10),
        ..Default::default()
    };
    let recent_entries = db.get_history(&recent_filter).await.map_err(|e| {
        error!(error = %e, "Failed to get recent searches");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::internal("Failed to get recent searches")),
        )
    })?;

    let recent_searches = recent_entries
        .into_iter()
        .map(|e| SearchHistorySummary {
            query: e.query,
            results_count: e.results_count,
            duration_ms: e.duration_ms,
            timestamp: e.timestamp,
        })
        .collect();

    Ok(Json(AnalyticsOverviewResponse {
        total_searches: overview.total_searches,
        total_feedback: overview.total_feedback,
        positive_feedback: overview.positive_feedback,
        negative_feedback: overview.negative_feedback,
        avg_latency_ms: overview.avg_latency_ms,
        avg_results: overview.avg_results,
        recent_searches,
    }))
}

/// Get top queries
pub async fn get_top_queries(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<Json<TopQueriesResponse>, (StatusCode, Json<ApiError>)> {
    info!("Getting top queries");

    let db = state
        .analytics_db
        .read()
        .await
        .as_ref()
        .ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiError::internal("Analytics database not available")),
            )
        })?
        .clone();

    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(10);
    let days = params
        .get("days")
        .and_then(|d| d.parse::<u32>().ok())
        .unwrap_or(30);

    let top_queries = db.get_top_queries(limit, days).await.map_err(|e| {
        error!(error = %e, "Failed to get top queries");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::internal("Failed to get top queries")),
        )
    })?;

    let queries = top_queries
        .into_iter()
        .map(|q| QueryStats {
            query: q.query,
            count: q.count,
            avg_results: q.avg_results,
            avg_latency_ms: q.avg_latency_ms,
            positive_feedback: q.positive_feedback,
            negative_feedback: q.negative_feedback,
        })
        .collect();

    Ok(Json(TopQueriesResponse { queries }))
}

/// Get feedback statistics
pub async fn get_feedback_statistics(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<Json<FeedbackStatsResponse>, (StatusCode, Json<ApiError>)> {
    info!("Getting feedback statistics");

    let db = state
        .analytics_db
        .read()
        .await
        .as_ref()
        .ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiError::internal("Analytics database not available")),
            )
        })?
        .clone();

    let days = params
        .get("days")
        .and_then(|d| d.parse::<u32>().ok())
        .unwrap_or(30);

    let stats = db.get_feedback_stats(days).await.map_err(|e| {
        error!(error = %e, "Failed to get feedback statistics");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::internal("Failed to get feedback statistics")),
        )
    })?;

    let by_type = stats
        .by_type
        .into_iter()
        .map(|(t, c)| FeedbackTypeCount {
            feedback_type: t,
            count: c,
        })
        .collect();

    let top_positive = stats
        .top_positive
        .into_iter()
        .map(|(id, count)| ResultFeedbackSummary {
            result_id: id,
            positive_count: count,
            negative_count: 0,
            total_count: count,
        })
        .collect();

    let top_negative = stats
        .top_negative
        .into_iter()
        .map(|(id, count)| ResultFeedbackSummary {
            result_id: id,
            positive_count: 0,
            negative_count: count,
            total_count: count,
        })
        .collect();

    Ok(Json(FeedbackStatsResponse {
        by_type,
        top_positive,
        top_negative,
    }))
}

/// Get search timeline
pub async fn get_search_timeline(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<Json<SearchTimelineResponse>, (StatusCode, Json<ApiError>)> {
    info!("Getting search timeline");

    let db = state
        .analytics_db
        .read()
        .await
        .as_ref()
        .ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiError::internal("Analytics database not available")),
            )
        })?
        .clone();

    let days = params
        .get("days")
        .and_then(|d| d.parse::<u32>().ok())
        .unwrap_or(7);
    let interval_hours = params
        .get("interval_hours")
        .and_then(|i| i.parse::<u32>().ok())
        .unwrap_or(24);

    let timeline_points = db.get_timeline(days, interval_hours).await.map_err(|e| {
        error!(error = %e, "Failed to get search timeline");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::internal("Failed to get search timeline")),
        )
    })?;

    let timeline = timeline_points
        .into_iter()
        .map(|p| TimelineDataPoint {
            timestamp: p.timestamp,
            search_count: p.search_count,
            avg_latency_ms: p.avg_latency_ms,
        })
        .collect();

    Ok(Json(SearchTimelineResponse { timeline }))
}

/// Serve the OpenAPI specification as JSON
pub async fn openapi_spec() -> impl IntoResponse {
    use axum::http::header;
    let spec = crate::openapi::generate_openapi_json();
    (
        [(header::CONTENT_TYPE, "application/json")],
        spec
    )
}
