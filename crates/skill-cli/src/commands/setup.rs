//! Interactive setup wizard for search and RAG configuration
//!
//! Provides a user-friendly way to configure embedding providers,
//! search settings, and model management.

use anyhow::{Context, Result};
use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use skill_runtime::SearchConfig;
use std::fs;
use std::path::PathBuf;

/// Valid FastEmbed model names
const FASTEMBED_MODELS: &[(&str, usize)] = &[
    ("all-minilm", 384),
    ("allminilm", 384),
    ("minilm", 384),
    ("bge-small", 384),
    ("bgesmall", 384),
    ("bge-small-en", 384),
    ("bge-base", 768),
    ("bgebase", 768),
    ("bge-base-en", 768),
    ("bge-large", 1024),
    ("bgelarge", 1024),
    ("bge-large-en", 1024),
];

/// Valid OpenAI model names
const OPENAI_MODELS: &[(&str, usize)] = &[
    ("text-embedding-ada-002", 1536),
    ("text-embedding-3-small", 1536),
    ("text-embedding-3-large", 3072),
];

/// Validate model name for a given provider
/// Returns (is_valid, optional_dimensions)
fn validate_model_for_provider(provider: &str, model: &str) -> Result<(bool, Option<usize>)> {
    let model_lower = model.to_lowercase();

    match provider {
        "fastembed" => {
            for (name, dims) in FASTEMBED_MODELS {
                if model_lower == *name {
                    return Ok((true, Some(*dims)));
                }
            }
            Err(anyhow::anyhow!(
                "Unknown FastEmbed model: '{}'\nSupported models: all-minilm, bge-small, bge-base, bge-large",
                model
            ))
        }
        "openai" => {
            for (name, dims) in OPENAI_MODELS {
                if model_lower == *name {
                    return Ok((true, Some(*dims)));
                }
            }
            Err(anyhow::anyhow!(
                "Unknown OpenAI model: '{}'\nSupported models: text-embedding-ada-002, text-embedding-3-small, text-embedding-3-large",
                model
            ))
        }
        "ollama" => {
            // Ollama accepts any model name - users must know their model
            Ok((true, None))
        }
        _ => {
            Err(anyhow::anyhow!("Unknown provider: {}", provider))
        }
    }
}

/// Get the path to the search configuration file
fn get_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Failed to get home directory")?;
    Ok(home.join(".skill-engine").join("search.toml"))
}

/// Load existing configuration or return default
fn load_config() -> Result<SearchConfig> {
    let config_path = get_config_path()?;
    if config_path.exists() {
        SearchConfig::from_toml_file(&config_path)
    } else {
        Ok(SearchConfig::default())
    }
}

/// Save configuration to file
fn save_config(config: &SearchConfig) -> Result<()> {
    let config_path = get_config_path()?;

    // Ensure directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let toml_str = toml::to_string_pretty(config)?;
    fs::write(&config_path, &toml_str)?;

    println!("{} Configuration saved to {}", "✓".green(), config_path.display());
    Ok(())
}

/// Display current configuration
fn show_config(config: &SearchConfig) -> Result<()> {
    let config_path = get_config_path()?;

    println!();
    println!("{}", "Search Configuration".bold().underline());
    println!();

    if config_path.exists() {
        println!("  {} {}", "Config file:".dimmed(), config_path.display());
    } else {
        println!("  {} {}", "Config file:".dimmed(), "(using defaults)".yellow());
    }
    println!();

    println!("{}", "Embedding Provider".bold());
    println!("  {} {}", "Provider:".cyan(), config.embedding.provider);
    println!("  {} {}", "Model:".cyan(), config.embedding.model);
    println!("  {} {}", "Dimensions:".cyan(), config.embedding.dimensions);

    if config.embedding.provider == "openai" {
        let has_key = config.embedding.openai_api_key.is_some() ||
                      std::env::var("OPENAI_API_KEY").is_ok();
        println!("  {} {}", "API Key:".cyan(),
            if has_key { "configured".green().to_string() } else { "not set".red().to_string() });
    }

    if config.embedding.provider == "ollama" {
        let host = config.embedding.ollama_host.as_deref().unwrap_or("http://localhost:11434");
        println!("  {} {}", "Host:".cyan(), host);
    }
    println!();

    println!("{}", "Retrieval Settings".bold());
    println!("  {} {}", "Hybrid Search:".cyan(),
        if config.retrieval.enable_hybrid { "enabled".green() } else { "disabled".dimmed() });
    println!("  {} {}", "First Stage K:".cyan(), config.retrieval.first_stage_k);
    println!("  {} {}", "Final K:".cyan(), config.retrieval.final_k);
    println!();

    println!("{}", "Reranker".bold());
    println!("  {} {}", "Enabled:".cyan(),
        if config.reranker.enabled { "yes".green() } else { "no".dimmed() });
    if config.reranker.enabled {
        println!("  {} {}", "Model:".cyan(), config.reranker.model);
        println!("  {} {}", "Max Documents:".cyan(), config.reranker.max_documents);
    }
    println!();

    println!("{}", "Backend".bold());
    println!("  {} {}", "Type:".cyan(), format!("{:?}", config.backend.backend_type).to_lowercase());
    println!();

    Ok(())
}

/// Run the interactive setup wizard
async fn interactive_wizard(mut config: SearchConfig) -> Result<SearchConfig> {
    let theme = ColorfulTheme::default();

    println!();
    println!("{}", "Skill Engine Search Setup".bold().cyan());
    println!("{}", "=".repeat(40).dimmed());
    println!();
    println!("This wizard will help you configure semantic search for tool discovery.");
    println!("Your configuration will be saved to ~/.skill-engine/search.toml");
    println!();

    // Step 1: Select embedding provider
    println!("{}", "Step 1: Embedding Provider".bold());
    println!();

    let providers = vec![
        "FastEmbed (Recommended) - Local, no API key, fast",
        "OpenAI - Cloud-based, requires API key, high quality",
        "Ollama - Local, requires Ollama server, customizable",
    ];

    let current_provider_idx = match config.embedding.provider.as_str() {
        "fastembed" => 0,
        "openai" => 1,
        "ollama" => 2,
        _ => 0,
    };

    let provider_selection = Select::with_theme(&theme)
        .with_prompt("Select embedding provider")
        .items(&providers)
        .default(current_provider_idx)
        .interact()?;

    match provider_selection {
        0 => {
            config.embedding.provider = "fastembed".to_string();

            // FastEmbed model selection
            // Note: all-minilm is most compatible across platforms
            // BGE models may have ONNX compatibility issues on some systems (macOS/ARM)
            let models = vec![
                "all-minilm (Recommended) - Most compatible, fast, 384 dimensions",
                "bge-small - Better quality, 384 dimensions (may have issues on some platforms)",
                "bge-base - High quality, 768 dimensions (may have issues on some platforms)",
                "bge-large - Best quality, 1024 dimensions (may have issues on some platforms)",
            ];

            let current_model_idx = match config.embedding.model.as_str() {
                "all-minilm" | "allminilm" | "minilm" => 0,
                "bge-small" | "bgesmall" | "bge-small-en" => 1,
                "bge-base" | "bgebase" | "bge-base-en" => 2,
                "bge-large" | "bgelarge" | "bge-large-en" => 3,
                _ => 0,
            };

            let model_selection = Select::with_theme(&theme)
                .with_prompt("Select FastEmbed model")
                .items(&models)
                .default(current_model_idx)
                .interact()?;

            config.embedding.model = match model_selection {
                0 => "all-minilm".to_string(),
                1 => "bge-small".to_string(),
                2 => "bge-base".to_string(),
                3 => "bge-large".to_string(),
                _ => "all-minilm".to_string(),
            };

            config.embedding.dimensions = match model_selection {
                0 | 1 => 384,
                2 => 768,
                3 => 1024,
                _ => 384,
            };

            println!("{} FastEmbed configured with {} model", "✓".green(), config.embedding.model.cyan());
        }
        1 => {
            config.embedding.provider = "openai".to_string();

            // Check for API key
            let has_key = std::env::var("OPENAI_API_KEY").is_ok();

            if !has_key {
                println!();
                println!("{} OPENAI_API_KEY not found in environment", "!".yellow());
                println!("  Set it with: {}", "export OPENAI_API_KEY=sk-...".cyan());
                println!();
            }

            // OpenAI model selection
            let models = vec![
                "text-embedding-ada-002 (Default) - Fast, cost-effective",
                "text-embedding-3-small - Newer, better quality",
                "text-embedding-3-large - Best quality, higher cost",
            ];

            let model_selection = Select::with_theme(&theme)
                .with_prompt("Select OpenAI model")
                .items(&models)
                .default(0)
                .interact()?;

            config.embedding.model = match model_selection {
                0 => "text-embedding-ada-002".to_string(),
                1 => "text-embedding-3-small".to_string(),
                2 => "text-embedding-3-large".to_string(),
                _ => "text-embedding-ada-002".to_string(),
            };

            config.embedding.dimensions = match model_selection {
                0 => 1536,
                1 => 1536,
                2 => 3072,
                _ => 1536,
            };

            println!("{} OpenAI configured with {} model", "✓".green(), config.embedding.model.cyan());
        }
        2 => {
            config.embedding.provider = "ollama".to_string();

            // Ollama host configuration
            let default_host = config.embedding.ollama_host.as_deref()
                .unwrap_or("http://localhost:11434");

            let host: String = Input::with_theme(&theme)
                .with_prompt("Ollama server URL")
                .default(default_host.to_string())
                .interact_text()?;

            config.embedding.ollama_host = Some(host);

            // Ollama model selection
            let model: String = Input::with_theme(&theme)
                .with_prompt("Ollama embedding model")
                .default(config.embedding.model.clone())
                .interact_text()?;

            config.embedding.model = model;

            // Dimensions (user must know their model's dimensions)
            let dims: String = Input::with_theme(&theme)
                .with_prompt("Embedding dimensions (depends on model)")
                .default(config.embedding.dimensions.to_string())
                .interact_text()?;

            config.embedding.dimensions = dims.parse().unwrap_or(384);

            println!("{} Ollama configured with {} model", "✓".green(), config.embedding.model.cyan());
        }
        _ => {}
    }

    println!();

    // Step 2: Advanced settings
    println!("{}", "Step 2: Advanced Settings".bold());
    println!();

    let configure_advanced = Confirm::with_theme(&theme)
        .with_prompt("Configure advanced settings? (hybrid search, reranking)")
        .default(false)
        .interact()?;

    if configure_advanced {
        // Hybrid search
        let enable_hybrid = Confirm::with_theme(&theme)
            .with_prompt("Enable hybrid search? (combines vector + keyword search)")
            .default(config.retrieval.enable_hybrid)
            .interact()?;

        config.retrieval.enable_hybrid = enable_hybrid;

        if enable_hybrid {
            println!("  {} Hybrid search improves recall for keyword-heavy queries", "i".blue());
        }

        // Reranking
        let enable_rerank = Confirm::with_theme(&theme)
            .with_prompt("Enable reranking? (improves precision, slightly slower)")
            .default(config.reranker.enabled)
            .interact()?;

        config.reranker.enabled = enable_rerank;

        if enable_rerank {
            let rerank_models = vec![
                "ms-marco-MiniLM-L-6-v2 (Default) - Fast, good quality",
                "ms-marco-TinyBERT-L-2-v2 - Fastest, smaller model",
                "bge-reranker-base - High quality reranking",
            ];

            let rerank_selection = Select::with_theme(&theme)
                .with_prompt("Select reranker model")
                .items(&rerank_models)
                .default(0)
                .interact()?;

            config.reranker.model = match rerank_selection {
                0 => "ms-marco-MiniLM-L-6-v2".to_string(),
                1 => "ms-marco-TinyBERT-L-2-v2".to_string(),
                2 => "BAAI/bge-reranker-base".to_string(),
                _ => "ms-marco-MiniLM-L-6-v2".to_string(),
            };
        }

        // Number of results
        let final_k: String = Input::with_theme(&theme)
            .with_prompt("Number of results to return")
            .default(config.retrieval.final_k.to_string())
            .interact_text()?;

        config.retrieval.final_k = final_k.parse().unwrap_or(5);
    }

    println!();

    // Step 3: Test configuration
    println!("{}", "Step 3: Verify Configuration".bold());
    println!();

    let test_config = Confirm::with_theme(&theme)
        .with_prompt("Test the configuration now? (downloads models if needed)")
        .default(true)
        .interact()?;

    if test_config {
        test_embedding_provider(&config).await?;
    }

    Ok(config)
}

/// Test the embedding provider configuration
async fn test_embedding_provider(config: &SearchConfig) -> Result<()> {
    use skill_runtime::SearchPipeline;

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.cyan} {msg}")
        .unwrap());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    pb.set_message("Initializing embedding provider...");

    // Try to create the pipeline
    let pipeline_result = SearchPipeline::from_config(config.clone()).await;

    match pipeline_result {
        Ok(pipeline) => {
            pb.set_message("Testing embedding generation...");

            // Try to generate an embedding
            let test_docs = vec![skill_runtime::IndexDocument {
                id: "test".to_string(),
                content: "Test document for configuration verification".to_string(),
                metadata: skill_runtime::DocumentMetadata::default(),
            }];

            match pipeline.index_documents(test_docs).await {
                Ok(_) => {
                    pb.finish_with_message(format!("{} Configuration test passed!", "✓".green()));
                    println!();
                    println!("  {} Embedding provider is working correctly", "✓".green());
                    println!("  {} Model downloaded and cached", "✓".green());
                }
                Err(e) => {
                    pb.finish_with_message(format!("{} Test failed", "✗".red()));
                    println!();
                    println!("  {} Error: {}", "✗".red(), e);
                    println!();
                    println!("  Check your configuration and try again.");
                }
            }
        }
        Err(e) => {
            pb.finish_with_message(format!("{} Initialization failed", "✗".red()));
            println!();
            println!("  {} Error: {}", "✗".red(), e);

            if config.embedding.provider == "openai" {
                println!();
                println!("  {} Make sure OPENAI_API_KEY is set correctly", "!".yellow());
            } else if config.embedding.provider == "ollama" {
                println!();
                println!("  {} Make sure Ollama server is running at {}", "!".yellow(),
                    config.embedding.ollama_host.as_deref().unwrap_or("http://localhost:11434"));
            }
        }
    }

    Ok(())
}

/// Execute the setup command
pub async fn execute(
    show: bool,
    reset: bool,
    provider: Option<&str>,
    model: Option<&str>,
    hybrid: Option<bool>,
    rerank: Option<bool>,
) -> Result<()> {
    // Load existing config or default
    let mut config = load_config()?;

    // Handle --show flag
    if show {
        return show_config(&config);
    }

    // Handle --reset flag
    if reset {
        let config_path = get_config_path()?;
        if config_path.exists() {
            fs::remove_file(&config_path)?;
            println!("{} Configuration reset to defaults", "✓".green());
        } else {
            println!("{} No configuration file to reset", "!".yellow());
        }
        return Ok(());
    }

    // Handle non-interactive options
    let has_options = provider.is_some() || model.is_some() || hybrid.is_some() || rerank.is_some();

    if has_options {
        // Non-interactive mode
        if let Some(p) = provider {
            match p.to_lowercase().as_str() {
                "fastembed" | "openai" | "ollama" => {
                    config.embedding.provider = p.to_lowercase();
                    println!("{} Provider set to {}", "✓".green(), p.cyan());
                }
                _ => {
                    return Err(anyhow::anyhow!(
                        "Unknown provider '{}'. Use: fastembed, openai, ollama", p
                    ));
                }
            }
        }

        if let Some(m) = model {
            // Validate model name based on provider
            let (valid, dimensions) = validate_model_for_provider(&config.embedding.provider, m)?;
            if !valid {
                return Err(anyhow::anyhow!(
                    "Invalid model '{}' for provider '{}'", m, config.embedding.provider
                ));
            }
            config.embedding.model = m.to_string();
            if let Some(dims) = dimensions {
                config.embedding.dimensions = dims;
            }
            println!("{} Model set to {}", "✓".green(), m.cyan());

            // Warn about potential BGE compatibility issues
            if config.embedding.provider == "fastembed" && m.to_lowercase().starts_with("bge") {
                println!("{} Note: BGE models may have ONNX compatibility issues on some platforms (macOS/ARM).",
                    "!".yellow());
                println!("  If you encounter errors, try using 'all-minilm' instead.");
            }
        }

        if let Some(h) = hybrid {
            config.retrieval.enable_hybrid = h;
            println!("{} Hybrid search {}", "✓".green(),
                if h { "enabled".green() } else { "disabled".dimmed() });
        }

        if let Some(r) = rerank {
            config.reranker.enabled = r;
            println!("{} Reranking {}", "✓".green(),
                if r { "enabled".green() } else { "disabled".dimmed() });
        }

        // Save configuration
        save_config(&config)?;

        return Ok(());
    }

    // Interactive mode
    config = interactive_wizard(config).await?;

    // Confirm and save
    println!();
    let save = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Save this configuration?")
        .default(true)
        .interact()?;

    if save {
        save_config(&config)?;
        println!();
        println!("{} Setup complete! Run {} to search for tools.",
            "✓".green().bold(),
            "skill find <query>".cyan());
    } else {
        println!("{} Configuration not saved", "!".yellow());
    }

    Ok(())
}
