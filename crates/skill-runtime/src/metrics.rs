use std::sync::atomic::{AtomicU64, Ordering};

/// Performance metrics for skill execution
pub struct ExecutionMetrics {
    /// Duration in milliseconds of the most recent cold start
    pub cold_start_ms: AtomicU64,
    /// Duration in milliseconds of the most recent warm start
    pub warm_start_ms: AtomicU64,
    /// Total number of skill executions
    pub total_executions: AtomicU64,
    /// Total number of failed executions
    pub failed_executions: AtomicU64,
}

impl ExecutionMetrics {
    /// Creates a new ExecutionMetrics instance with all counters initialized to zero
    pub fn new() -> Self {
        Self {
            cold_start_ms: AtomicU64::new(0),
            warm_start_ms: AtomicU64::new(0),
            total_executions: AtomicU64::new(0),
            failed_executions: AtomicU64::new(0),
        }
    }

    /// Records the duration of a cold start execution
    pub fn record_cold_start(&self, duration_ms: u64) {
        self.cold_start_ms.store(duration_ms, Ordering::Relaxed);
    }

    /// Records the duration of a warm start execution
    pub fn record_warm_start(&self, duration_ms: u64) {
        self.warm_start_ms.store(duration_ms, Ordering::Relaxed);
    }

    /// Records an execution attempt and updates failure count if unsuccessful
    pub fn record_execution(&self, success: bool) {
        self.total_executions.fetch_add(1, Ordering::Relaxed);
        if !success {
            self.failed_executions.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Returns the most recent cold start duration in milliseconds
    pub fn get_cold_start_ms(&self) -> u64 {
        self.cold_start_ms.load(Ordering::Relaxed)
    }

    /// Returns the most recent warm start duration in milliseconds
    pub fn get_warm_start_ms(&self) -> u64 {
        self.warm_start_ms.load(Ordering::Relaxed)
    }

    /// Returns the total number of executions
    pub fn get_total_executions(&self) -> u64 {
        self.total_executions.load(Ordering::Relaxed)
    }

    /// Returns the total number of failed executions
    pub fn get_failed_executions(&self) -> u64 {
        self.failed_executions.load(Ordering::Relaxed)
    }

    /// Calculates and returns the success rate as a percentage (0.0 to 100.0)
    pub fn get_success_rate(&self) -> f64 {
        let total = self.get_total_executions();
        if total == 0 {
            return 0.0;
        }
        let failed = self.get_failed_executions();
        ((total - failed) as f64 / total as f64) * 100.0
    }
}

impl Default for ExecutionMetrics {
    fn default() -> Self {
        Self::new()
    }
}
