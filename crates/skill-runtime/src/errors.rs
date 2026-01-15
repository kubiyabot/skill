use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Component loading failed: {0}")]
    ComponentLoadError(String),

    #[error("Component validation failed: {0}")]
    ValidationError(String),

    #[error("Sandbox initialization failed: {0}")]
    SandboxError(String),

    #[error("Tool execution failed: {0}")]
    ExecutionError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Instance not found: {skill}/{instance}")]
    InstanceNotFound {
        skill: String,
        instance: String,
    },

    #[error("WASM runtime error: {0}")]
    WasmError(#[from] wasmtime::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("TOML deserialization error: {0}")]
    TomlError(#[from] toml::de::Error),
}

pub type Result<T> = std::result::Result<T, RuntimeError>;
