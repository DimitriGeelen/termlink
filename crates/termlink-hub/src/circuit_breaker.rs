use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Number of consecutive transport failures before opening the circuit.
const FAILURE_THRESHOLD: u32 = 3;

/// How long an open circuit stays open before allowing a probe (half-open).
const COOLDOWN: Duration = Duration::from_secs(60);

/// Per-session circuit breaker state.
#[derive(Debug, Clone)]
#[derive(Default)]
struct CircuitState {
    consecutive_failures: u32,
    opened_at: Option<Instant>,
}


impl CircuitState {
    /// Is the circuit open (skip this session)?
    fn is_open(&self) -> bool {
        self.opened_at.is_some()
    }

    /// Is the circuit in half-open state (cooldown expired, try one probe)?
    fn is_half_open(&self) -> bool {
        match self.opened_at {
            Some(t) => t.elapsed() >= COOLDOWN,
            None => false,
        }
    }

    /// Record a successful call — close the circuit.
    fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.opened_at = None;
    }

    /// Record a transport failure. Opens circuit after threshold.
    fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        if self.consecutive_failures >= FAILURE_THRESHOLD
            && self.opened_at.is_none() {
                self.opened_at = Some(Instant::now());
            }
    }
}

/// Global circuit breaker registry for all sessions seen by the hub.
///
/// Thread-safe via internal `Mutex`. Keyed by session ID.
pub struct CircuitBreakerRegistry {
    states: Mutex<HashMap<String, CircuitState>>,
}

impl Default for CircuitBreakerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CircuitBreakerRegistry {
    pub fn new() -> Self {
        Self {
            states: Mutex::new(HashMap::new()),
        }
    }

    /// Check if a session's circuit is open (should be skipped).
    /// Returns `false` for unknown sessions (closed by default).
    /// Returns `false` for half-open circuits (allow one probe).
    pub fn should_skip(&self, session_id: &str) -> bool {
        let states = self.states.lock().expect("circuit breaker lock poisoned");
        match states.get(session_id) {
            Some(state) => state.is_open() && !state.is_half_open(),
            None => false,
        }
    }

    /// Record a successful call to a session — closes the circuit.
    pub fn record_success(&self, session_id: &str) {
        let mut states = self.states.lock().expect("circuit breaker lock poisoned");
        if let Some(state) = states.get_mut(session_id) {
            state.record_success();
        }
    }

    /// Record a transport failure for a session.
    /// After `FAILURE_THRESHOLD` consecutive failures, opens the circuit.
    pub fn record_failure(&self, session_id: &str) {
        let mut states = self.states.lock().expect("circuit breaker lock poisoned");
        states
            .entry(session_id.to_string())
            .or_default()
            .record_failure();
    }

    /// Get the number of open circuits (for diagnostics).
    pub fn open_count(&self) -> usize {
        let states = self.states.lock().expect("circuit breaker lock poisoned");
        states.values().filter(|s| s.is_open() && !s.is_half_open()).count()
    }

    /// Reset all circuit breaker state (for testing).
    #[cfg(test)]
    pub fn reset(&self) {
        let mut states = self.states.lock().expect("circuit breaker lock poisoned");
        states.clear();
    }
}

/// Default model fallback chain: opus → sonnet → haiku.
pub const DEFAULT_MODEL_FALLBACK: &[&str] = &["opus", "sonnet", "haiku"];

/// Model-level circuit breaker registry.
///
/// Tracks model availability separately from session-level breakers.
/// When a model is unavailable (circuit open), the dispatch system
/// falls back to the next model in the fallback chain.
pub struct ModelCircuitBreaker {
    states: Mutex<HashMap<String, CircuitState>>,
}

impl Default for ModelCircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelCircuitBreaker {
    pub fn new() -> Self {
        Self {
            states: Mutex::new(HashMap::new()),
        }
    }

    /// Check if a model's circuit is open (should be skipped).
    pub fn should_skip(&self, model: &str) -> bool {
        let states = self.states.lock().expect("model circuit breaker lock poisoned");
        match states.get(model) {
            Some(state) => state.is_open() && !state.is_half_open(),
            None => false,
        }
    }

    /// Record a successful dispatch with this model.
    pub fn record_success(&self, model: &str) {
        let mut states = self.states.lock().expect("model circuit breaker lock poisoned");
        if let Some(state) = states.get_mut(model) {
            state.record_success();
        }
    }

    /// Record a failure for this model.
    pub fn record_failure(&self, model: &str) {
        let mut states = self.states.lock().expect("model circuit breaker lock poisoned");
        states
            .entry(model.to_string())
            .or_default()
            .record_failure();
    }

    /// Resolve the best available model from a fallback chain.
    ///
    /// Starting from `preferred`, walks the fallback chain and returns
    /// the first model whose circuit is not open. Returns None if all
    /// models in the chain are unavailable.
    pub fn resolve_model(&self, preferred: &str, fallback_chain: &[&str]) -> Option<String> {
        // Try preferred first
        if !self.should_skip(preferred) {
            return Some(preferred.to_string());
        }
        // Walk fallback chain
        for &model in fallback_chain {
            if model == preferred {
                continue; // already tried
            }
            if !self.should_skip(model) {
                return Some(model.to_string());
            }
        }
        None
    }

    /// Get the number of open model circuits (for diagnostics).
    pub fn open_count(&self) -> usize {
        let states = self.states.lock().expect("model circuit breaker lock poisoned");
        states.values().filter(|s| s.is_open() && !s.is_half_open()).count()
    }

    /// Reset all model circuit state (for testing).
    #[cfg(test)]
    pub fn reset(&self) {
        let mut states = self.states.lock().expect("model circuit breaker lock poisoned");
        states.clear();
    }
}

/// Global singleton for session-level circuit breakers.
static REGISTRY: std::sync::LazyLock<CircuitBreakerRegistry> =
    std::sync::LazyLock::new(CircuitBreakerRegistry::new);

/// Global singleton for model-level circuit breakers.
static MODEL_REGISTRY: std::sync::LazyLock<ModelCircuitBreaker> =
    std::sync::LazyLock::new(ModelCircuitBreaker::new);

/// Get the global circuit breaker registry.
pub fn global() -> &'static CircuitBreakerRegistry {
    &REGISTRY
}

/// Get the global model circuit breaker registry.
pub fn model_global() -> &'static ModelCircuitBreaker {
    &MODEL_REGISTRY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_by_default() {
        let reg = CircuitBreakerRegistry::new();
        assert!(!reg.should_skip("unknown-session"));
    }

    #[test]
    fn opens_after_threshold_failures() {
        let reg = CircuitBreakerRegistry::new();

        // 2 failures — still closed
        reg.record_failure("sess-1");
        reg.record_failure("sess-1");
        assert!(!reg.should_skip("sess-1"));

        // 3rd failure — opens
        reg.record_failure("sess-1");
        assert!(reg.should_skip("sess-1"));
    }

    #[test]
    fn success_closes_circuit() {
        let reg = CircuitBreakerRegistry::new();

        // Open the circuit
        for _ in 0..3 {
            reg.record_failure("sess-2");
        }
        assert!(reg.should_skip("sess-2"));

        // Success closes it
        reg.record_success("sess-2");
        assert!(!reg.should_skip("sess-2"));
    }

    #[test]
    fn success_resets_failure_count() {
        let reg = CircuitBreakerRegistry::new();

        // 2 failures, then success
        reg.record_failure("sess-3");
        reg.record_failure("sess-3");
        reg.record_success("sess-3");

        // 2 more failures — not yet threshold (counter reset)
        reg.record_failure("sess-3");
        reg.record_failure("sess-3");
        assert!(!reg.should_skip("sess-3"));

        // 3rd failure after reset — NOW opens
        reg.record_failure("sess-3");
        assert!(reg.should_skip("sess-3"));
    }

    #[test]
    fn half_open_after_cooldown() {
        let reg = CircuitBreakerRegistry::new();

        // Open the circuit with a backdated opened_at
        {
            let mut states = reg.states.lock().expect("circuit breaker lock poisoned");
            states.insert(
                "sess-4".to_string(),
                CircuitState {
                    consecutive_failures: 3,
                    opened_at: Some(Instant::now() - COOLDOWN - Duration::from_secs(1)),
                },
            );
        }

        // Cooldown expired — half-open, should NOT skip (allow probe)
        assert!(!reg.should_skip("sess-4"));
    }

    #[test]
    fn independent_sessions() {
        let reg = CircuitBreakerRegistry::new();

        // Open circuit for sess-a
        for _ in 0..3 {
            reg.record_failure("sess-a");
        }

        // sess-b should be unaffected
        assert!(reg.should_skip("sess-a"));
        assert!(!reg.should_skip("sess-b"));
    }

    #[test]
    fn open_count() {
        let reg = CircuitBreakerRegistry::new();

        for _ in 0..3 {
            reg.record_failure("x");
            reg.record_failure("y");
        }
        assert_eq!(reg.open_count(), 2);

        reg.record_success("x");
        assert_eq!(reg.open_count(), 1);
    }

    // --- Model circuit breaker tests ---

    #[test]
    fn model_breaker_closed_by_default() {
        let mcb = ModelCircuitBreaker::new();
        assert!(!mcb.should_skip("opus"));
        assert!(!mcb.should_skip("sonnet"));
        assert!(!mcb.should_skip("haiku"));
    }

    #[test]
    fn model_breaker_opens_after_failures() {
        let mcb = ModelCircuitBreaker::new();
        mcb.record_failure("opus");
        mcb.record_failure("opus");
        assert!(!mcb.should_skip("opus"));

        mcb.record_failure("opus");
        assert!(mcb.should_skip("opus"));
    }

    #[test]
    fn model_breaker_success_closes() {
        let mcb = ModelCircuitBreaker::new();
        for _ in 0..3 { mcb.record_failure("sonnet"); }
        assert!(mcb.should_skip("sonnet"));

        mcb.record_success("sonnet");
        assert!(!mcb.should_skip("sonnet"));
    }

    #[test]
    fn model_resolve_preferred_available() {
        let mcb = ModelCircuitBreaker::new();
        let result = mcb.resolve_model("opus", DEFAULT_MODEL_FALLBACK);
        assert_eq!(result, Some("opus".to_string()));
    }

    #[test]
    fn model_resolve_fallback_on_failure() {
        let mcb = ModelCircuitBreaker::new();
        // Open circuit for opus
        for _ in 0..3 { mcb.record_failure("opus"); }

        let result = mcb.resolve_model("opus", DEFAULT_MODEL_FALLBACK);
        assert_eq!(result, Some("sonnet".to_string()));
    }

    #[test]
    fn model_resolve_fallback_chain() {
        let mcb = ModelCircuitBreaker::new();
        // Open circuits for opus and sonnet
        for _ in 0..3 {
            mcb.record_failure("opus");
            mcb.record_failure("sonnet");
        }

        let result = mcb.resolve_model("opus", DEFAULT_MODEL_FALLBACK);
        assert_eq!(result, Some("haiku".to_string()));
    }

    #[test]
    fn model_resolve_all_unavailable() {
        let mcb = ModelCircuitBreaker::new();
        for _ in 0..3 {
            mcb.record_failure("opus");
            mcb.record_failure("sonnet");
            mcb.record_failure("haiku");
        }

        let result = mcb.resolve_model("opus", DEFAULT_MODEL_FALLBACK);
        assert_eq!(result, None);
    }

    #[test]
    fn model_resolve_independent_models() {
        let mcb = ModelCircuitBreaker::new();
        for _ in 0..3 { mcb.record_failure("opus"); }

        // sonnet should still be available
        assert!(!mcb.should_skip("sonnet"));
        assert!(mcb.should_skip("opus"));
    }

    #[test]
    fn model_breaker_open_count() {
        let mcb = ModelCircuitBreaker::new();
        for _ in 0..3 {
            mcb.record_failure("opus");
            mcb.record_failure("sonnet");
        }
        assert_eq!(mcb.open_count(), 2);

        mcb.record_success("opus");
        assert_eq!(mcb.open_count(), 1);
    }

    #[test]
    fn default_model_fallback_chain_order() {
        assert_eq!(DEFAULT_MODEL_FALLBACK, &["opus", "sonnet", "haiku"]);
    }
}
