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

    /// Record a transport failure. Opens (or RE-arms) the circuit after
    /// threshold.
    ///
    /// T-2495: any failure at/over `FAILURE_THRESHOLD` re-stamps `opened_at`,
    /// restarting the cooldown. The previous `&& self.opened_at.is_none()`
    /// guard stamped only on the FIRST open, so a failed half-open probe
    /// (which runs precisely when `opened_at` is already `Some`) never
    /// re-armed the cooldown — `is_half_open()` then stayed true forever and
    /// `should_skip()` returned false forever, silently defeating the breaker.
    /// `record_failure` is only called on an *attempted* route (router.rs:
    /// 1511/1528) — sessions in the open (non-half-open) window are skipped
    /// without an attempt — so re-stamping cannot prematurely extend a
    /// still-open circuit; it only re-arms after a probe genuinely fails.
    fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        if self.consecutive_failures >= FAILURE_THRESHOLD {
            self.opened_at = Some(Instant::now());
        }
    }
}

/// Default hard cap on the session-circuit map. Env-tunable via
/// `TERMLINK_CIRCUIT_MAX_ENTRIES` (must parse `> 0`, else this default).
fn circuit_max_entries_from_env() -> usize {
    std::env::var("TERMLINK_CIRCUIT_MAX_ENTRIES")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(10_000)
}

/// Global circuit breaker registry for all sessions seen by the hub.
///
/// Thread-safe via internal `Mutex`. Keyed by session ID.
///
/// T-2676: the `session_id` key is per-routing-target and effectively
/// unbounded over the hub's lifetime — `record_failure` inserts via
/// `.entry().or_default()` and, before this fix, nothing ever removed an entry
/// outside the `#[cfg(test)] reset()`. A session that experienced a failed
/// route left a permanent `CircuitState` even after it deregistered, so the map
/// grew without bound (the same unbounded-peer/session-keyed-map class as the
/// T-2675 presence map). There is no single authoritative deregister site the
/// router owns for these targets, so rather than wire into multiple fragile
/// teardown paths (a missed one re-leaks), the map is bounded by a hard cap
/// with a SAFE eviction: only entries that are NOT actively blocking are
/// dropped. `should_skip` returns `true` only for an open circuit still inside
/// its cooldown; a closed or half-open (cooldown-elapsed) entry returns
/// `false`, observably identical to an absent key — so evicting those cannot
/// un-block a live session's breaker (the inverse of the T-2495 defeat).
pub struct CircuitBreakerRegistry {
    states: Mutex<HashMap<String, CircuitState>>,
    /// Hard cap; eviction (safe, non-actively-blocking-first) runs when the map
    /// would exceed it. Always `>= 1` (clamped in `with_cap`).
    max_entries: usize,
}

impl Default for CircuitBreakerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CircuitBreakerRegistry {
    pub fn new() -> Self {
        Self::with_cap(circuit_max_entries_from_env())
    }

    /// Construct with an explicit hard cap. Used by tests for deterministic,
    /// env-independent bounds (parallel tests must not race on env).
    pub fn with_cap(max_entries: usize) -> Self {
        Self {
            states: Mutex::new(HashMap::new()),
            max_entries: max_entries.max(1),
        }
    }

    /// T-2676: bound the map. Called after an insert that may have grown it past
    /// the cap. Two tiers, both SAFE (never removes an actively-blocking
    /// circuit, so a live breaker is never defeated):
    ///   1. Drop every entry that is NOT actively blocking (closed OR half-open
    ///      — both return `should_skip == false`, identical to absent). This
    ///      reclaims all recovered / dead / cooled-down cruft, which dominates
    ///      the leak.
    ///   2. Only if STILL over cap (pathological: >cap circuits simultaneously
    ///      open within cooldown — a fleet-wide outage), evict the oldest such
    ///      circuits by `opened_at` down to the cap to hold the memory bound.
    ///      All remaining entries are actively-open, so `opened_at` is `Some`.
    fn evict_if_over_cap(states: &mut HashMap<String, CircuitState>, cap: usize) {
        if states.len() <= cap {
            return;
        }
        states.retain(|_, s| s.is_open() && !s.is_half_open());
        if states.len() > cap {
            let excess = states.len() - cap;
            let mut by_age: Vec<(String, Instant)> = states
                .iter()
                .filter_map(|(k, s)| s.opened_at.map(|t| (k.clone(), t)))
                .collect();
            by_age.sort_by_key(|(_, t)| *t);
            for (k, _) in by_age.into_iter().take(excess) {
                states.remove(&k);
            }
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
        // T-2676: hold the map bound after this insert. Safe eviction only —
        // an actively-blocking circuit is never removed.
        Self::evict_if_over_cap(&mut states, self.max_entries);
    }

    /// Get the number of open circuits (for diagnostics).
    pub fn open_count(&self) -> usize {
        let states = self.states.lock().expect("circuit breaker lock poisoned");
        states.values().filter(|s| s.is_open() && !s.is_half_open()).count()
    }

    /// Total tracked entries (test-only observability for the T-2676 bound).
    #[cfg(test)]
    pub fn entry_count(&self) -> usize {
        let states = self.states.lock().expect("circuit breaker lock poisoned");
        states.len()
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
    fn t2676_cap_bounds_map_without_defeating_live_breaker() {
        // AC #2 (safety) + AC #3 (bound): a hard cap holds the session map, and
        // eviction NEVER removes an actively-blocking circuit. Reverting the
        // `evict_if_over_cap` call in `record_failure` makes the entry_count
        // assertion fail (map grows to 51) — the load-bearing property.
        let reg = CircuitBreakerRegistry::with_cap(3);

        // A live, actively-blocking circuit (open, within cooldown).
        for _ in 0..FAILURE_THRESHOLD {
            reg.record_failure("live");
        }
        assert!(reg.should_skip("live"), "live circuit is open");

        // Flood many distinct single-failure (closed) session_ids — the leak
        // vector (dead/transient sessions that never recover).
        for i in 0..50 {
            reg.record_failure(&format!("dead-{i}"));
        }

        // Bound held — NOT 51 entries.
        assert!(
            reg.entry_count() <= 3,
            "map bounded to cap (got {})",
            reg.entry_count()
        );
        // Safety: the live actively-blocking breaker survived every eviction.
        assert!(
            reg.should_skip("live"),
            "live breaker must survive eviction (AC #2 — inverse of T-2495 defeat)"
        );
        // The transient closed entries did not accumulate.
        assert!(!reg.should_skip("dead-0"));
    }

    #[test]
    fn t2676_tier2_evicts_oldest_actively_open_when_all_blocking() {
        // Pathological backstop: when MORE than `cap` circuits are
        // simultaneously open within cooldown (a fleet-wide outage), tier-2
        // evicts the oldest-by-opened_at down to the cap to hold the memory
        // bound. Driven directly on the private helper for determinism (the
        // public API can't reliably manufacture >cap simultaneously-open
        // circuits without timestamp-collision flakiness).
        let mut states: HashMap<String, CircuitState> = HashMap::new();
        let base = Instant::now();
        // Larger age_ms = further in the past = smaller Instant = "older".
        // All within COOLDOWN (elapsed << 60s) → actively blocking.
        for (name, age_ms) in [("oldest", 40u64), ("old", 30), ("new", 20), ("newest", 10)] {
            states.insert(
                name.to_string(),
                CircuitState {
                    consecutive_failures: FAILURE_THRESHOLD,
                    opened_at: Some(base - Duration::from_millis(age_ms)),
                },
            );
        }
        CircuitBreakerRegistry::evict_if_over_cap(&mut states, 2);
        assert_eq!(states.len(), 2, "held exactly to cap");
        assert!(
            states.contains_key("newest") && states.contains_key("new"),
            "the 2 newest (largest opened_at) survive"
        );
        assert!(
            !states.contains_key("oldest") && !states.contains_key("old"),
            "the 2 oldest evicted first"
        );
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

    /// T-2495 regression: a FAILED half-open probe must re-arm the cooldown.
    /// Before the fix, `record_failure` only stamped `opened_at` on the first
    /// open, so after the cooldown the circuit stayed half-open forever and
    /// `should_skip` returned false forever — silently defeating the breaker.
    #[test]
    fn failed_half_open_probe_re_arms_cooldown() {
        let reg = CircuitBreakerRegistry::new();

        // Open + backdate so the circuit is half-open (cooldown expired).
        {
            let mut states = reg.states.lock().expect("circuit breaker lock poisoned");
            states.insert(
                "sess-5".to_string(),
                CircuitState {
                    consecutive_failures: 3,
                    opened_at: Some(Instant::now() - COOLDOWN - Duration::from_secs(1)),
                },
            );
        }
        // Half-open → probe allowed.
        assert!(!reg.should_skip("sess-5"), "precondition: half-open allows a probe");

        // The probe FAILS → must re-arm: opened_at moves to ~now, cooldown restarts.
        reg.record_failure("sess-5");
        assert!(
            reg.should_skip("sess-5"),
            "a failed half-open probe must re-open the circuit (re-arm cooldown)"
        );
    }

    /// T-2495: same guarantee for the model breaker (shared `CircuitState`).
    #[test]
    fn model_failed_half_open_probe_re_arms_cooldown() {
        let mcb = ModelCircuitBreaker::new();
        {
            let mut states = mcb.states.lock().expect("model circuit breaker lock poisoned");
            states.insert(
                "opus".to_string(),
                CircuitState {
                    consecutive_failures: 3,
                    opened_at: Some(Instant::now() - COOLDOWN - Duration::from_secs(1)),
                },
            );
        }
        assert!(!mcb.should_skip("opus"), "precondition: half-open allows a probe");
        mcb.record_failure("opus");
        assert!(
            mcb.should_skip("opus"),
            "a failed half-open model probe must re-open the circuit"
        );
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
