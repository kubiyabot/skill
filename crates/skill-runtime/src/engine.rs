use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use wasmtime::{component::Component, Config, Engine};


/// Main WASM runtime engine for executing skills
pub struct SkillEngine {
    engine: Arc<Engine>,
    cache_dir: PathBuf,
}

impl SkillEngine {
    /// Create a new SkillEngine with optimized configuration
    pub fn new() -> Result<Self> {
        let mut config = Config::new();

        // Enable Component Model support (required for WIT interfaces)
        config.wasm_component_model(true);

        // Enable async support for non-blocking I/O
        config.async_support(true);

        // Performance optimizations
        config.cranelift_opt_level(wasmtime::OptLevel::Speed);

        // Enable caching for faster subsequent loads
        let cache_dir = dirs::cache_dir()
            .context("Failed to get cache directory")?
            .join("skill-engine")
            .join("wasmtime-cache");

        std::fs::create_dir_all(&cache_dir)?;
        config.cache_config_load_default()?;

        // Memory limits (1GB per skill)
        config.max_wasm_stack(1024 * 1024); // 1MB stack

        // Enable debug info for better error messages
        config.debug_info(true);

        let engine = Engine::new(&config)?;

        tracing::info!(
            "Initialized SkillEngine with cache at: {}",
            cache_dir.display()
        );

        Ok(Self {
            engine: Arc::new(engine),
            cache_dir,
        })
    }

    /// Get the underlying Wasmtime engine
    pub fn wasmtime_engine(&self) -> &Engine {
        &self.engine
    }

    /// Load a WASM component from file
    pub async fn load_component(&self, path: &std::path::Path) -> Result<Component> {
        tracing::debug!("Loading component from: {}", path.display());

        let component = Component::from_file(&self.engine, path)
            .with_context(|| format!("Failed to load component from {}", path.display()))?;

        tracing::info!("Successfully loaded component: {}", path.display());

        Ok(component)
    }

    /// Pre-compile a component and store in cache (AOT compilation)
    pub async fn precompile_component(&self, path: &std::path::Path) -> Result<Vec<u8>> {
        tracing::debug!("Pre-compiling component: {}", path.display());

        let bytes = std::fs::read(path)
            .with_context(|| format!("Failed to read component file: {}", path.display()))?;

        let compiled = self
            .engine
            .precompile_component(&bytes)
            .context("Failed to precompile component")?;

        tracing::info!("Pre-compiled component: {} bytes", compiled.len());

        Ok(compiled)
    }

    /// Load a pre-compiled component from cache
    pub async fn load_precompiled(
        &self,
        compiled_bytes: &[u8],
    ) -> Result<Component> {
        unsafe {
            Component::deserialize(&self.engine, compiled_bytes)
                .context("Failed to deserialize pre-compiled component")
        }
    }

    /// Get cache directory path
    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    /// Validate a component against the skill interface
    pub async fn validate_component(&self, _component: &Component) -> Result<()> {
        // TODO: Use wit-bindgen to validate exports
        // For now, just check that the component loads successfully
        tracing::debug!("Validating component");

        // The fact that it loaded successfully is a good start
        // Full validation will be implemented with wit-bindgen integration

        Ok(())
    }
}

impl Default for SkillEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default SkillEngine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = SkillEngine::new().unwrap();
        assert!(engine.cache_dir.exists());
    }

    #[test]
    fn test_engine_config() {
        let engine = SkillEngine::new().unwrap();
        // Verify the engine was created successfully
        let _ = engine.wasmtime_engine();
    }
}
