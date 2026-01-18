use thiserror::Error;

/// Errors that can occur during skill runtime operations
#[derive(Error, Debug)]
pub enum RuntimeError {
    /// Failed to load a WASM component
    #[error("Component loading failed: {0}")]
    ComponentLoadError(String),

    /// Component validation failed
    #[error("Component validation failed: {0}")]
    ValidationError(String),

    /// Sandbox initialization or operation failed
    #[error("Sandbox initialization failed: {0}")]
    SandboxError(String),

    /// Tool execution failed during runtime
    #[error("Tool execution failed: {0}")]
    ExecutionError(String),

    /// Configuration parsing or validation error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Requested skill instance was not found
    #[error("Instance not found: {skill}/{instance}")]
    InstanceNotFound {
        /// Skill name
        skill: String,
        /// Instance name
        instance: String,
    },

    /// WASM runtime error from Wasmtime
    #[error("WASM runtime error: {0}")]
    WasmError(#[from] wasmtime::Error),

    /// I/O operation error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// TOML deserialization error
    #[error("TOML deserialization error: {0}")]
    TomlError(#[from] toml::de::Error),
}

/// Result type alias using RuntimeError
pub type Result<T> = std::result::Result<T, RuntimeError>;
