use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

mod auth;
mod commands;
mod config;

#[derive(Parser)]
#[command(name = "skill")]
#[command(about = "Skill Engine - Universal WASM plugin system", long_about = None)]
#[command(version)]
struct Cli {
    /// Path to skill manifest file (default: auto-detect .skill-engine.toml)
    #[arg(short = 'm', long = "manifest", global = true)]
    manifest: Option<std::path::PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a skill from registry, Git, or local path
    ///
    /// Examples:
    ///   skill install ./skill.wasm              # Local WASM file
    ///   skill install github:user/repo          # GitHub shorthand
    ///   skill install github:user/repo@v1.0.0   # Specific version
    ///   skill install https://github.com/u/r   # Full URL
    Install {
        /// Skill source: local path, Git URL, or shorthand (github:user/repo)
        source: String,

        /// Instance name for this installation
        #[arg(short = 'i', long)]
        instance: Option<String>,

        /// Force re-clone for Git sources (ignore cache)
        #[arg(short = 'f', long)]
        force: bool,

        /// Generate AI-powered examples after installation
        #[arg(long)]
        enhance: bool,
    },

    /// Run a skill tool
    ///
    /// Examples:
    ///   skill run aws:s3-list bucket=mybucket
    ///   skill run aws@prod:s3-upload bucket=b key=k file=f
    ///   skill run ./path/to/skill.wasm hello name=World
    ///   skill run github:user/repo:tool arg=value
    ///   skill run aws:list --config region=eu-west-1
    Run {
        /// Skill name or name@instance:tool
        skill: String,

        /// Tool name (if not in skill spec)
        tool: Option<String>,

        /// Inline config overrides (key=value pairs, override instance config)
        #[arg(short = 'c', long = "config", value_parser = parse_key_val)]
        config: Vec<(String, String)>,

        /// Tool arguments
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Execute skill with pass-through arguments (like docker exec)
    ///
    /// Passes arguments directly to the skill's main entry point,
    /// bypassing the tool-based execution model.
    ///
    /// Examples:
    ///   skill exec aws -- s3 ls s3://mybucket
    ///   skill exec aws@prod -- ec2 describe-instances
    ///   skill exec ./local-skill -- --help
    Exec {
        /// Skill name or name@instance
        skill: String,

        /// Inline config overrides (key=value pairs)
        #[arg(short = 'c', long = "config", value_parser = parse_key_val)]
        config: Vec<(String, String)>,

        /// Arguments to pass to the skill (after --)
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// List installed skills
    #[command(alias = "ls")]
    List {
        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },

    /// Remove a skill
    #[command(alias = "rm")]
    Remove {
        /// Skill name
        skill: String,

        /// Instance name
        #[arg(short = 'i', long)]
        instance: Option<String>,

        /// Force removal without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Configure a skill
    Config {
        /// Skill name
        skill: String,

        /// Instance name
        #[arg(short = 'i', long)]
        instance: Option<String>,

        #[command(subcommand)]
        action: Option<ConfigAction>,
    },

    /// Initialize a new skill project
    Init {
        /// Project name
        name: Option<String>,

        /// Template to use
        #[arg(short = 't', long)]
        template: Option<String>,

        /// List available templates
        #[arg(long)]
        list: bool,
    },

    /// Start HTTP and MCP server
    Serve {
        /// Skill to serve (if empty, serves all)
        skill: Option<String>,

        /// Port to bind to
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Use HTTP streaming transport instead of stdio (for web clients)
        #[arg(long)]
        http: bool,

        /// Start web interface (trunk serve on port 8080)
        #[arg(long)]
        with_web: bool,
    },

    /// Show skill information
    Info {
        /// Skill name
        skill: String,
    },

    /// Search for skills in registry
    Search {
        /// Search query
        query: String,
    },

    /// Find tools semantically using AI-powered vector search
    Find {
        /// Natural language query describing what you want to do
        query: String,

        /// Number of top results to show
        #[arg(short = 'n', long, default_value = "5")]
        top_k: Option<usize>,

        /// Embedding provider to use (fastembed, ollama, openai)
        #[arg(short = 'p', long, default_value = "fastembed")]
        provider: String,

        /// Model name (optional, uses provider default if not specified)
        #[arg(long)]
        model: Option<String>,

        /// Output format (rich, json, compact)
        #[arg(short = 'f', long, default_value = "rich")]
        format: String,
    },

    /// Enhance skills with AI-generated examples
    ///
    /// Uses LLMs to generate realistic usage examples for tool discovery.
    /// Requires AI ingestion to be enabled in configuration.
    ///
    /// Examples:
    ///   skill enhance kubernetes        # Enhance a specific skill
    ///   skill enhance --all             # Enhance all installed skills
    ///   skill enhance --all --stream    # With streaming progress
    Enhance {
        /// Skill name to enhance
        skill: Option<String>,

        /// Enhance all installed skills
        #[arg(long)]
        all: bool,

        /// Stream progress to terminal (shows thinking, examples)
        #[arg(long)]
        stream: bool,

        /// Number of examples to generate per tool
        #[arg(short = 'n', long, default_value = "5")]
        examples: usize,
    },

    /// Configure search and RAG settings
    ///
    /// Interactive wizard to configure embedding providers, search settings,
    /// and model management for semantic tool discovery.
    ///
    /// Examples:
    ///   skill setup                    # Interactive wizard
    ///   skill setup --show             # Show current configuration
    ///   skill setup --provider openai  # Set provider non-interactively
    ///   skill setup --reset            # Reset to defaults
    Setup {
        /// Show current configuration
        #[arg(long)]
        show: bool,

        /// Reset to default configuration
        #[arg(long)]
        reset: bool,

        /// Set embedding provider (fastembed, openai, ollama)
        #[arg(short = 'p', long)]
        provider: Option<String>,

        /// Set embedding model
        #[arg(long)]
        model: Option<String>,

        /// Enable hybrid search (BM25 + vector)
        #[arg(long)]
        hybrid: Option<bool>,

        /// Enable reranking for better precision
        #[arg(long)]
        rerank: Option<bool>,
    },

    /// Generate SKILL.md documentation template
    #[command(name = "init-skill")]
    InitSkill {
        /// Skill name for the template
        skill_name: String,

        /// Output path for SKILL.md (default: ./SKILL.md)
        #[arg(short = 'o', long)]
        output: Option<String>,

        /// Tool names to include in template (comma-separated)
        #[arg(short = 't', long, value_delimiter = ',')]
        tools: Option<Vec<String>>,

        /// Generate from installed skill (introspection)
        #[arg(long)]
        from_installed: bool,
    },

    /// Integrate with Claude Code
    ///
    /// Configure Claude Code to use Skill Engine as an MCP server,
    /// giving Claude access to all your installed skills.
    ///
    /// Examples:
    ///   skill claude setup              # Add to project .mcp.json
    ///   skill claude setup --global     # Add to global Claude config
    ///   skill claude status             # Check integration status
    ///   skill claude remove             # Remove integration
    Claude {
        #[command(subcommand)]
        action: ClaudeAction,
    },

    /// Upgrade skill CLI to the latest version
    ///
    /// Downloads and installs the latest release from GitHub.
    ///
    /// Examples:
    ///   skill upgrade                   # Upgrade to latest version
    ///   skill upgrade --check           # Check for updates without installing
    ///   skill upgrade --force           # Force reinstall current version
    Upgrade {
        /// Check for updates without installing
        #[arg(short = 'c', long)]
        check: bool,

        /// Force reinstall even if already on latest version
        #[arg(short = 'f', long)]
        force: bool,
    },

    /// Start the web interface with embedded UI
    ///
    /// Launches a local web server with the Skill Engine web interface.
    /// The UI is embedded in the binary - no external files required.
    ///
    /// Examples:
    ///   skill web                       # Start on localhost:3000
    ///   skill web --port 8080           # Custom port
    ///   skill web --open                # Open browser automatically
    ///   skill web --host 0.0.0.0        # Listen on all interfaces
    Web {
        /// Port to run the web server on
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Open browser automatically after starting
        #[arg(short, long)]
        open: bool,
    },

    /// Authenticate with external services (OAuth2, API keys, etc.)
    ///
    /// Examples:
    ///   skill auth login github           # OAuth2 Device Flow
    ///   skill auth login aws              # AWS IAM credentials
    ///   skill auth login openai           # API key
    ///   skill auth status                 # Check auth status
    ///   skill auth logout github          # Remove credentials
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },
}

#[derive(Subcommand)]
enum ClaudeAction {
    /// Configure Claude Code to use Skill Engine MCP server
    Setup {
        /// Add to global Claude config instead of project .mcp.json
        #[arg(short = 'g', long)]
        global: bool,

        /// Custom server name in .mcp.json (default: skill-engine)
        #[arg(short = 'n', long)]
        name: Option<String>,

        /// Custom path to skill binary
        #[arg(short = 'b', long)]
        binary: Option<String>,
    },

    /// Check Claude Code integration status
    Status,

    /// Remove Skill Engine from Claude Code configuration
    Remove {
        /// Remove from global config instead of project .mcp.json
        #[arg(short = 'g', long)]
        global: bool,

        /// Server name to remove (default: skill-engine)
        #[arg(short = 'n', long)]
        name: Option<String>,
    },

    /// Generate Claude Agent Skills from installed skills
    ///
    /// Creates SKILL.md, TOOLS.md, and scripts for Claude Agent Skills
    /// that integrate with Skill Engine. Supports dual-mode execution:
    /// MCP tools (preferred) or standalone scripts.
    ///
    /// Examples:
    ///   skill claude generate                    # Generate all skills
    ///   skill claude generate --skill kubernetes # Generate specific skill
    ///   skill claude generate --dry-run          # Preview without writing
    ///   skill claude generate --output ./skills  # Custom output directory
    Generate {
        /// Specific skill to generate (if not specified, generates all)
        #[arg(short = 's', long)]
        skill: Option<String>,

        /// Output directory (default: ~/.claude/skills)
        #[arg(short = 'o', long)]
        output: Option<std::path::PathBuf>,

        /// Force overwrite existing files
        #[arg(short = 'f', long)]
        force: bool,

        /// Dry run - show what would be generated without writing
        #[arg(long)]
        dry_run: bool,

        /// Skip generating scripts (only generate SKILL.md and TOOLS.md)
        #[arg(long)]
        no_scripts: bool,

        /// Generate for project-level Claude config (current directory)
        #[arg(short = 'p', long)]
        project: bool,
    },
}

#[derive(Subcommand)]
enum AuthAction {
    /// Login to an authentication provider
    Login {
        /// Provider name (github, google, aws, openai, anthropic, etc.)
        provider: String,

        /// Associate credentials with a specific skill
        #[arg(short = 's', long)]
        skill: Option<String>,

        /// Instance name for the credentials
        #[arg(short = 'i', long)]
        instance: Option<String>,

        /// OAuth2 scopes to request (comma-separated)
        #[arg(long, value_delimiter = ',')]
        scopes: Option<Vec<String>>,
    },

    /// Show authentication status
    Status {
        /// Filter by provider
        provider: Option<String>,
    },

    /// Logout from a provider (revoke credentials)
    Logout {
        /// Provider name
        provider: String,

        /// Skill name
        #[arg(short = 's', long)]
        skill: Option<String>,

        /// Instance name
        #[arg(short = 'i', long)]
        instance: Option<String>,
    },

    /// List available authentication providers
    Providers,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,

    /// Set a configuration value
    Set {
        /// Key=value pairs
        #[arg(value_parser = parse_key_val)]
        pairs: Vec<(String, String)>,
    },

    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },
}

fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing - IMPORTANT: Write to stderr for MCP stdio compatibility
    // For MCP stdio mode, we must never write to stdout as it's reserved for JSON-RPC
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    // Load manifest if specified or auto-detect
    let manifest = commands::manifest::load_manifest(cli.manifest.as_deref())?;

    let result = match cli.command {
        Commands::Install { source, instance, force, enhance } => {
            commands::install::execute(&source, instance.as_deref(), force, enhance).await
        }
        Commands::Run { skill, tool, config, args } => {
            commands::run::execute(&skill, tool.as_deref(), &config, &args, manifest.as_ref()).await
        }
        Commands::Exec { skill, config, args } => {
            commands::exec::execute(&skill, &config, &args, manifest.as_ref()).await
        }
        Commands::List { format } => {
            commands::list::execute(&format, manifest.as_ref()).await
        }
        Commands::Remove { skill, instance, force } => {
            commands::remove::execute(&skill, instance.as_deref(), force).await
        }
        Commands::Config { skill, instance, action } => {
            commands::config::execute(&skill, instance.as_deref(), action).await
        }
        Commands::Init { name, template, list } => {
            commands::init::execute(name.as_deref(), template.as_deref(), list).await
        }
        Commands::Serve { skill, port, host, http, with_web } => {
            commands::serve::execute(skill.as_deref(), &host, port, http, with_web).await
        }
        Commands::Info { skill } => {
            commands::info::execute(&skill, manifest.as_ref()).await
        }
        Commands::Search { query } => {
            commands::search::execute(&query).await
        }
        Commands::Find { query, top_k, provider, model, format } => {
            commands::find::execute(&query, top_k, &provider, model.as_deref(), &format).await
        }
        Commands::Enhance { skill, all, stream, examples } => {
            commands::enhance::execute(skill.as_deref(), all, stream, examples).await
        }
        Commands::Setup { show, reset, provider, model, hybrid, rerank } => {
            commands::setup::execute(show, reset, provider.as_deref(), model.as_deref(), hybrid, rerank).await
        }
        Commands::InitSkill { skill_name, output, tools, from_installed } => {
            if from_installed {
                commands::init_skill::generate_from_skill(&skill_name, output.as_deref()).await
            } else {
                commands::init_skill::execute(&skill_name, output.as_deref(), tools)
            }
        }
        Commands::Claude { action } => {
            match action {
                ClaudeAction::Setup { global, name, binary } => {
                    commands::claude::setup(global, name.as_deref(), binary.as_deref()).await
                }
                ClaudeAction::Status => {
                    commands::claude::status().await
                }
                ClaudeAction::Remove { global, name } => {
                    commands::claude::remove(global, name.as_deref()).await
                }
                ClaudeAction::Generate { skill, output, force, dry_run, no_scripts, project } => {
                    commands::claude::generate(skill, output, force, dry_run, no_scripts, project).await
                }
            }
        }
        Commands::Upgrade { check, force } => {
            commands::upgrade::execute(force, check).await
        }
        Commands::Auth { action } => {
            match action {
                AuthAction::Login { provider, skill, instance, scopes } => {
                    auth::login(&provider, skill.as_deref(), instance.as_deref(), scopes).await
                }
                AuthAction::Status { provider } => {
                    auth::status(provider.as_deref()).await
                }
                AuthAction::Logout { provider, skill, instance } => {
                    auth::logout(&provider, skill.as_deref(), instance.as_deref()).await
                }
                AuthAction::Providers => {
                    auth::providers().await
                }
            }
        }
        Commands::Web { port, host, open } => {
            commands::web::execute(&host, port, open).await
        }
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        // Print the error chain for debugging
        for cause in e.chain().skip(1) {
            eprintln!("  {} {}", "Caused by:".dimmed(), cause);
        }
        std::process::exit(1);
    }

    Ok(())
}
