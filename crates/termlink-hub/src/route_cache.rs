use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Route cache — per-capability routing metadata with confidence, TTL, and lazy invalidation.
///
/// Sits between the bypass registry (Layer 1) and orchestrator.route (Layer 3) in the
/// T-233 layered capability discovery system (D-007). The cache stores routing metadata
/// (which specialist handles a capability), NOT execution logic.
///
/// 3-way lookup: hit+valid → CacheHit, expired/low-confidence → Stale, miss → CacheMiss.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RouteCache {
    pub entries: HashMap<String, RouteCacheEntry>,
}

/// A cached route: maps a capability slug to a specialist session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteCacheEntry {
    /// Capability slug (e.g., "commit-conventional", "test-rust").
    pub capability: String,
    /// Specialist session tag or display name.
    pub specialist: String,
    /// Confidence score (0.0–1.0). Decays over time without use.
    pub confidence: f64,
    /// Schema fields the specialist expects (for validation on lookup).
    #[serde(default)]
    pub request_schema: RequestSchema,
    /// How the route was learned.
    pub learned_from: LearnedFrom,
    /// ISO timestamp of last successful use.
    pub last_used: String,
    /// Number of successful dispatches via this route.
    pub hit_count: u64,
    /// Time-to-live in hours. After expiry, entry is treated as stale (not miss).
    pub ttl_hours: u64,
    /// ISO timestamp when the entry was created/last refreshed.
    pub created_at: String,
}

/// How a route was learned.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LearnedFrom {
    Orchestrator,
    Builtin,
}

/// Lightweight schema descriptor for cache validation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RequestSchema {
    #[serde(default)]
    pub required: Vec<String>,
    #[serde(default)]
    pub optional: Vec<String>,
}

/// Result of a cache lookup.
#[derive(Debug)]
pub enum CacheLookup<'a> {
    /// Valid hit: confidence >= threshold and not expired.
    Hit(&'a RouteCacheEntry),
    /// Entry exists but is expired or confidence is below threshold.
    Stale(&'a RouteCacheEntry),
    /// No entry for this capability.
    Miss,
}

/// Minimum confidence for a cache hit (below this → Stale).
const CONFIDENCE_THRESHOLD: f64 = 0.8;

/// Default TTL in hours (7 days).
const DEFAULT_TTL_HOURS: u64 = 168;

/// Confidence decay per week of non-use.
const CONFIDENCE_DECAY_PER_WEEK: f64 = 0.05;

impl RouteCache {
    /// Look up a capability slug in the cache.
    ///
    /// Returns Hit if the entry is valid (not expired, confidence >= 0.8),
    /// Stale if expired or low confidence, Miss if not found.
    pub fn lookup(&self, capability: &str) -> CacheLookup<'_> {
        match self.entries.get(capability) {
            Some(entry) => {
                let effective_confidence = entry.effective_confidence();
                let expired = entry.is_expired();

                if expired || effective_confidence < CONFIDENCE_THRESHOLD {
                    CacheLookup::Stale(entry)
                } else {
                    CacheLookup::Hit(entry)
                }
            }
            None => CacheLookup::Miss,
        }
    }

    /// Find entries matching a prefix (e.g., "commit" matches "commit-conventional").
    /// Returns stale lookup with the best matching entry, or Miss if no prefix matches.
    pub fn prefix_lookup(&self, prefix: &str) -> CacheLookup<'_> {
        let mut best: Option<&RouteCacheEntry> = None;
        for (slug, entry) in &self.entries {
            if slug.starts_with(prefix) || prefix.starts_with(slug.as_str()) {
                match best {
                    None => best = Some(entry),
                    Some(current) if entry.effective_confidence() > current.effective_confidence() => {
                        best = Some(entry);
                    }
                    _ => {}
                }
            }
        }
        match best {
            Some(entry) => CacheLookup::Stale(entry), // partial match always treated as stale
            None => CacheLookup::Miss,
        }
    }

    /// Record a successful route from orchestrator.route into the cache.
    pub fn record_route(
        &mut self,
        capability: &str,
        specialist: &str,
        schema: RequestSchema,
    ) {
        let now = now_iso();
        let entry = self
            .entries
            .entry(capability.to_string())
            .or_insert_with(|| RouteCacheEntry {
                capability: capability.to_string(),
                specialist: specialist.to_string(),
                confidence: 1.0,
                request_schema: schema.clone(),
                learned_from: LearnedFrom::Orchestrator,
                last_used: now.clone(),
                hit_count: 0,
                ttl_hours: DEFAULT_TTL_HOURS,
                created_at: now.clone(),
            });
        // Refresh existing entry
        entry.specialist = specialist.to_string();
        entry.confidence = 1.0;
        entry.request_schema = schema;
        entry.last_used = now.clone();
        entry.created_at = now;
    }

    /// Record a successful dispatch via a cached route. Increments hit count.
    pub fn record_hit(&mut self, capability: &str) {
        if let Some(entry) = self.entries.get_mut(capability) {
            entry.hit_count += 1;
            entry.last_used = now_iso();
            // Refresh confidence on use (capped at 1.0)
            entry.confidence = (entry.confidence + 0.1).min(1.0);
        }
    }

    /// Invalidate a specific capability entry (e.g., specialist rejected the request).
    /// Returns true if an entry was removed.
    pub fn invalidate(&mut self, capability: &str) -> bool {
        self.entries.remove(capability).is_some()
    }

    /// Invalidate all entries matching a specialist name (e.g., specialist went offline).
    /// Returns the number of entries removed.
    pub fn invalidate_specialist(&mut self, specialist: &str) -> usize {
        let before = self.entries.len();
        self.entries.retain(|_, e| e.specialist != specialist);
        before - self.entries.len()
    }

    /// Validate that a request's fields match the cached schema.
    /// Returns true if the request is compatible with the cached schema.
    pub fn schema_matches(entry: &RouteCacheEntry, request_fields: &[&str]) -> bool {
        // All required schema fields must be present in the request
        entry.request_schema.required.iter().all(|f| request_fields.contains(&f.as_str()))
    }

    // --- Persistence ---

    /// Load cache from the default path.
    pub fn load() -> Self {
        let path = cache_path();
        Self::load_from(&path)
    }

    /// Load from a specific path. Returns empty cache if file doesn't exist or is corrupt.
    pub fn load_from(path: &PathBuf) -> Self {
        match std::fs::read_to_string(path) {
            Ok(data) => match serde_json::from_str(&data) {
                Ok(cache) => cache,
                Err(e) => {
                    tracing::warn!(
                        path = %path.display(),
                        error = %e,
                        "Route cache corrupt — returning empty cache"
                    );
                    Self::default()
                }
            },
            Err(_) => Self::default(),
        }
    }

    /// Save cache to the default path using atomic write.
    pub fn save(&self) -> std::io::Result<()> {
        let path = cache_path();
        self.save_to(&path)
    }

    /// Save to a specific path using atomic write (temp file + rename).
    pub fn save_to(&self, path: &PathBuf) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        let tmp_path = path.with_extension("json.tmp");
        std::fs::write(&tmp_path, &data)?;
        std::fs::rename(&tmp_path, path)?;
        Ok(())
    }
}

impl RouteCacheEntry {
    /// Calculate effective confidence with decay based on time since last use.
    pub fn effective_confidence(&self) -> f64 {
        let weeks_since_use = weeks_since(&self.last_used);
        let decayed = self.confidence - (weeks_since_use * CONFIDENCE_DECAY_PER_WEEK);
        decayed.max(0.0)
    }

    /// Check if the entry has exceeded its TTL.
    pub fn is_expired(&self) -> bool {
        let hours_since_created = hours_since(&self.created_at);
        hours_since_created > self.ttl_hours as f64
    }
}

/// Default cache file path.
pub fn cache_path() -> PathBuf {
    termlink_session::discovery::runtime_dir().join("route-cache.json")
}

fn now_iso() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    // Simple ISO-ish format: enough for ordering and human readability
    
    time_from_epoch(secs)
}

fn time_from_epoch(secs: u64) -> String {
    // Minimal UTC timestamp without pulling in chrono
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Days since epoch to Y-M-D (simplified Gregorian)
    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let months_days: &[i64] = if is_leap(y) {
        &[31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        &[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 1;
    for &md in months_days {
        if remaining < md {
            break;
        }
        remaining -= md;
        m += 1;
    }
    let d = remaining + 1;

    format!("{y:04}-{m:02}-{d:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// Parse an ISO timestamp and return weeks since then (fractional).
fn weeks_since(iso: &str) -> f64 {
    let epoch_secs = parse_iso_epoch(iso);
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if epoch_secs == 0 || epoch_secs > now_secs {
        return 0.0;
    }
    (now_secs - epoch_secs) as f64 / (7.0 * 24.0 * 3600.0)
}

/// Parse an ISO timestamp and return hours since then (fractional).
fn hours_since(iso: &str) -> f64 {
    let epoch_secs = parse_iso_epoch(iso);
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if epoch_secs == 0 || epoch_secs > now_secs {
        return 0.0;
    }
    (now_secs - epoch_secs) as f64 / 3600.0
}

/// Minimal ISO 8601 parser → epoch seconds. Returns 0 on failure.
fn parse_iso_epoch(iso: &str) -> u64 {
    // Expected: "YYYY-MM-DDTHH:MM:SSZ"
    let parts: Vec<&str> = iso.split(['T', '-', ':', 'Z'].as_ref()).collect();
    if parts.len() < 6 {
        return 0;
    }
    let y: i64 = parts[0].parse().unwrap_or(0);
    let m: u64 = parts[1].parse().unwrap_or(0);
    let d: u64 = parts[2].parse().unwrap_or(0);
    let hh: u64 = parts[3].parse().unwrap_or(0);
    let mm: u64 = parts[4].parse().unwrap_or(0);
    let ss: u64 = parts[5].parse().unwrap_or(0);

    if y < 1970 || m == 0 || m > 12 || d == 0 || d > 31 {
        return 0;
    }

    // Days from epoch to start of year
    let mut days: u64 = 0;
    for yr in 1970..y {
        days += if is_leap(yr) { 366 } else { 365 };
    }
    let months_days: &[u64] = if is_leap(y) {
        &[31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        &[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    for md in &months_days[..(m as usize - 1)] {
        days += md;
    }
    days += d - 1;

    days * 86400 + hh * 3600 + mm * 60 + ss
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(capability: &str, specialist: &str, confidence: f64) -> RouteCacheEntry {
        RouteCacheEntry {
            capability: capability.to_string(),
            specialist: specialist.to_string(),
            confidence,
            request_schema: RequestSchema {
                required: vec!["files".to_string(), "message_type".to_string()],
                optional: vec!["body".to_string()],
            },
            learned_from: LearnedFrom::Orchestrator,
            last_used: now_iso(),
            hit_count: 3,
            ttl_hours: DEFAULT_TTL_HOURS,
            created_at: now_iso(),
        }
    }

    #[test]
    fn route_cache_hit() {
        let mut cache = RouteCache::default();
        cache.entries.insert(
            "commit-conventional".to_string(),
            make_entry("commit-conventional", "git-specialist", 0.95),
        );

        match cache.lookup("commit-conventional") {
            CacheLookup::Hit(entry) => {
                assert_eq!(entry.specialist, "git-specialist");
                assert_eq!(entry.hit_count, 3);
            }
            other => panic!("Expected Hit, got {:?}", std::mem::discriminant(&other)),
        }
    }

    #[test]
    fn route_cache_miss() {
        let cache = RouteCache::default();
        assert!(matches!(cache.lookup("nonexistent"), CacheLookup::Miss));
    }

    #[test]
    fn route_cache_stale_low_confidence() {
        let mut cache = RouteCache::default();
        cache.entries.insert(
            "test-rust".to_string(),
            make_entry("test-rust", "test-specialist", 0.3),
        );

        assert!(matches!(cache.lookup("test-rust"), CacheLookup::Stale(_)));
    }

    #[test]
    fn route_cache_stale_expired() {
        let mut cache = RouteCache::default();
        let mut entry = make_entry("docker-build", "docker-specialist", 0.95);
        // Set created_at to 8 days ago (past 7-day TTL)
        entry.created_at = "2020-01-01T00:00:00Z".to_string();
        cache.entries.insert("docker-build".to_string(), entry);

        assert!(matches!(cache.lookup("docker-build"), CacheLookup::Stale(_)));
    }

    #[test]
    fn confidence_decay() {
        let mut entry = make_entry("test", "specialist", 1.0);
        // Set last_used to ~2 weeks ago
        entry.last_used = "2020-01-01T00:00:00Z".to_string();
        let effective = entry.effective_confidence();
        // Should be significantly decayed (many weeks since 2020)
        assert!(effective < 0.5, "Confidence should have decayed: {effective}");
    }

    #[test]
    fn confidence_decay_recent_use() {
        let entry = make_entry("test", "specialist", 0.95);
        // last_used is now_iso() — effectively zero decay
        let effective = entry.effective_confidence();
        assert!(
            (effective - 0.95).abs() < 0.01,
            "Recent use should have near-zero decay: {effective}"
        );
    }

    #[test]
    fn record_route_creates_entry() {
        let mut cache = RouteCache::default();
        cache.record_route(
            "cargo-test",
            "test-specialist",
            RequestSchema {
                required: vec!["crate".to_string()],
                optional: vec![],
            },
        );

        assert!(cache.entries.contains_key("cargo-test"));
        let entry = &cache.entries["cargo-test"];
        assert_eq!(entry.specialist, "test-specialist");
        assert_eq!(entry.confidence, 1.0);
        assert_eq!(entry.hit_count, 0);
    }

    #[test]
    fn record_route_refreshes_existing() {
        let mut cache = RouteCache::default();
        cache.entries.insert(
            "cargo-test".to_string(),
            make_entry("cargo-test", "old-specialist", 0.5),
        );

        cache.record_route(
            "cargo-test",
            "new-specialist",
            RequestSchema::default(),
        );

        let entry = &cache.entries["cargo-test"];
        assert_eq!(entry.specialist, "new-specialist");
        assert_eq!(entry.confidence, 1.0);
    }

    #[test]
    fn record_hit_increments() {
        let mut cache = RouteCache::default();
        cache.entries.insert(
            "test".to_string(),
            make_entry("test", "specialist", 0.85),
        );

        cache.record_hit("test");

        let entry = &cache.entries["test"];
        assert_eq!(entry.hit_count, 4); // was 3
        assert!(entry.confidence >= 0.85); // refreshed
    }

    #[test]
    fn invalidate_single() {
        let mut cache = RouteCache::default();
        cache.entries.insert(
            "test".to_string(),
            make_entry("test", "specialist", 0.9),
        );

        assert!(cache.invalidate("test"));
        assert!(!cache.entries.contains_key("test"));
    }

    #[test]
    fn invalidate_specialist_removes_all() {
        let mut cache = RouteCache::default();
        cache.entries.insert(
            "a".to_string(),
            make_entry("a", "git-specialist", 0.9),
        );
        cache.entries.insert(
            "b".to_string(),
            make_entry("b", "git-specialist", 0.9),
        );
        cache.entries.insert(
            "c".to_string(),
            make_entry("c", "other-specialist", 0.9),
        );

        let removed = cache.invalidate_specialist("git-specialist");
        assert_eq!(removed, 2);
        assert_eq!(cache.entries.len(), 1);
        assert!(cache.entries.contains_key("c"));
    }

    #[test]
    fn prefix_lookup_matches() {
        let mut cache = RouteCache::default();
        cache.entries.insert(
            "commit-conventional".to_string(),
            make_entry("commit-conventional", "git-specialist", 0.9),
        );

        // "commit" should match "commit-conventional"
        match cache.prefix_lookup("commit") {
            CacheLookup::Stale(entry) => {
                assert_eq!(entry.specialist, "git-specialist");
            }
            other => panic!("Expected Stale (partial match), got {:?}", std::mem::discriminant(&other)),
        }

        // No match
        assert!(matches!(cache.prefix_lookup("docker"), CacheLookup::Miss));
    }

    #[test]
    fn schema_matches_validates() {
        let entry = make_entry("test", "specialist", 0.9);
        // Required: files, message_type
        assert!(RouteCache::schema_matches(&entry, &["files", "message_type", "body"]));
        assert!(!RouteCache::schema_matches(&entry, &["files"])); // missing message_type
    }

    #[test]
    fn persistence_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("route-cache.json");

        let mut cache = RouteCache::default();
        cache.record_route(
            "cargo-test",
            "test-specialist",
            RequestSchema {
                required: vec!["crate".to_string()],
                optional: vec!["filter".to_string()],
            },
        );
        cache.record_route(
            "commit-conventional",
            "git-specialist",
            RequestSchema {
                required: vec!["files".to_string()],
                optional: vec![],
            },
        );

        cache.save_to(&path).unwrap();

        let loaded = RouteCache::load_from(&path);
        assert_eq!(loaded.entries.len(), 2);
        assert_eq!(loaded.entries["cargo-test"].specialist, "test-specialist");
        assert_eq!(loaded.entries["commit-conventional"].specialist, "git-specialist");
    }

    #[test]
    fn load_missing_file_returns_empty() {
        let path = PathBuf::from("/tmp/nonexistent-route-cache-12345.json");
        let cache = RouteCache::load_from(&path);
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn load_corrupt_file_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("route-cache.json");
        std::fs::write(&path, "not valid json {{{").unwrap();

        let cache = RouteCache::load_from(&path);
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn iso_timestamp_round_trip() {
        let ts = now_iso();
        let epoch = parse_iso_epoch(&ts);
        assert!(epoch > 0, "Should parse our own timestamps");

        let now_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // Should be within 2 seconds
        assert!((now_epoch as i64 - epoch as i64).unsigned_abs() < 2);
    }
}
