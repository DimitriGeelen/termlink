use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// 3-layer template cache for specialist interaction patterns.
///
/// Part of the T-233 progressive learning system (D-007):
/// - **Layer 1:** Agent-local cache — per-agent learned templates
/// - **Layer 2:** Shared registry — promoted templates available to all agents
/// - **Layer 3:** Specialist canonical — source of truth (not stored here, pulled on miss)
///
/// Lookup order: agent-local → shared → miss (triggers specialist round-trip).
/// Promotion: 5 uses + 0 corrections → auto-promote from Layer 1 to Layer 2.
/// Invalidation: schema hash mismatch on use → discard + return miss.

/// Promotion threshold: uses required before a local template is promoted to shared.
pub const PROMOTION_THRESHOLD: u64 = 5;

/// A cached template entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateEntry {
    /// Specialist that provided this template.
    pub specialist: String,
    /// Format identifier (e.g., "specialist/report-v2").
    pub format_id: String,
    /// Monotonic version counter from the specialist.
    pub version: u32,
    /// Hash of the schema's required fields (content-addressed invalidation).
    pub schema_hash: String,
    /// The cached template data (JSON Schema or exemplar).
    pub template: serde_json::Value,
    /// Number of successful uses of this template.
    pub hit_count: u64,
    /// Number of specialist corrections received while using this template.
    pub correction_count: u64,
    /// ISO timestamp of last successful use.
    pub last_used: String,
    /// ISO timestamp when learned/cached.
    pub learned_at: String,
}

/// Result of a template cache lookup.
#[derive(Debug)]
pub enum TemplateLookup<'a> {
    /// Found in agent-local cache (Layer 1).
    LocalHit(&'a TemplateEntry),
    /// Found in shared registry (Layer 2).
    SharedHit(&'a TemplateEntry),
    /// No cached template — need specialist round-trip (Layer 3).
    Miss,
}

/// The 3-layer template cache.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateCache {
    /// Layer 1: Agent-local templates, keyed by format_id.
    pub local: HashMap<String, TemplateEntry>,
    /// Layer 2: Shared templates (promoted from local), keyed by format_id.
    pub shared: HashMap<String, TemplateEntry>,
}

impl TemplateCache {
    /// Look up a template by format_id, with optional schema hash validation.
    ///
    /// If `current_schema_hash` is provided, entries with mismatched hashes
    /// are treated as stale and skipped (returns Miss for invalidation).
    pub fn lookup<'a>(
        &'a self,
        format_id: &str,
        current_schema_hash: Option<&str>,
    ) -> TemplateLookup<'a> {
        // Layer 1: agent-local
        if let Some(entry) = self.local.get(format_id) {
            if let Some(hash) = current_schema_hash {
                if entry.schema_hash != hash {
                    // Schema mismatch — stale, treat as miss
                    return TemplateLookup::Miss;
                }
            }
            return TemplateLookup::LocalHit(entry);
        }

        // Layer 2: shared
        if let Some(entry) = self.shared.get(format_id) {
            if let Some(hash) = current_schema_hash {
                if entry.schema_hash != hash {
                    return TemplateLookup::Miss;
                }
            }
            return TemplateLookup::SharedHit(entry);
        }

        TemplateLookup::Miss
    }

    /// Record a template learned from a specialist interaction (Layer 1).
    pub fn record_template(
        &mut self,
        format_id: &str,
        specialist: &str,
        version: u32,
        schema_hash: &str,
        template: serde_json::Value,
    ) {
        let now = now_iso();
        self.local.insert(
            format_id.to_string(),
            TemplateEntry {
                specialist: specialist.to_string(),
                format_id: format_id.to_string(),
                version,
                schema_hash: schema_hash.to_string(),
                template,
                hit_count: 0,
                correction_count: 0,
                last_used: now.clone(),
                learned_at: now,
            },
        );
    }

    /// Record a successful use of a cached template. Returns true if promoted to shared.
    pub fn record_hit(&mut self, format_id: &str) -> bool {
        if let Some(entry) = self.local.get_mut(format_id) {
            entry.hit_count += 1;
            entry.last_used = now_iso();

            // Check promotion threshold
            if entry.hit_count >= PROMOTION_THRESHOLD && entry.correction_count == 0 {
                // Promote to shared (copy, don't move — agent keeps local copy)
                if !self.shared.contains_key(format_id) {
                    let mut shared_entry = entry.clone();
                    shared_entry.hit_count = 0; // reset for shared tracking
                    self.shared.insert(format_id.to_string(), shared_entry);
                    return true;
                }
            }
            return false;
        }

        // Also track hits on shared templates
        if let Some(entry) = self.shared.get_mut(format_id) {
            entry.hit_count += 1;
            entry.last_used = now_iso();
        }

        false
    }

    /// Record a specialist correction while using a template.
    /// Resets the promotion counter for that template.
    pub fn record_correction(&mut self, format_id: &str) {
        if let Some(entry) = self.local.get_mut(format_id) {
            entry.correction_count += 1;
        }
    }

    /// Invalidate a template by format_id (e.g., schema hash mismatch detected).
    /// Removes from both local and shared layers.
    /// Returns true if anything was removed.
    pub fn invalidate(&mut self, format_id: &str) -> bool {
        let local_removed = self.local.remove(format_id).is_some();
        let shared_removed = self.shared.remove(format_id).is_some();
        local_removed || shared_removed
    }

    /// Invalidate all templates from a specific specialist.
    /// Returns the number of entries removed.
    pub fn invalidate_specialist(&mut self, specialist: &str) -> usize {
        let before = self.local.len() + self.shared.len();
        self.local.retain(|_, e| e.specialist != specialist);
        self.shared.retain(|_, e| e.specialist != specialist);
        let after = self.local.len() + self.shared.len();
        before - after
    }

    // --- Persistence ---

    /// Load cache from the default path.
    pub fn load() -> Self {
        let path = cache_path();
        Self::load_from(&path)
    }

    /// Load from a specific path. Returns empty cache on missing/corrupt file.
    pub fn load_from(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(data) => match serde_json::from_str(&data) {
                Ok(cache) => cache,
                Err(e) => {
                    tracing::warn!(
                        path = %path.display(),
                        error = %e,
                        "Template cache corrupt — returning empty cache"
                    );
                    Self::default()
                }
            },
            Err(_) => Self::default(),
        }
    }

    /// Save cache using atomic write (temp file + rename).
    pub fn save(&self) -> std::io::Result<()> {
        let path = cache_path();
        self.save_to(&path)
    }

    /// Save to a specific path.
    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
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

/// Compute a schema hash from a JSON Schema value.
/// Uses the sorted required field names as the hash input.
pub fn compute_schema_hash(schema: &serde_json::Value) -> String {
    use std::collections::BTreeSet;

    let mut fields = BTreeSet::new();

    // Collect required fields
    if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
        for f in required {
            if let Some(s) = f.as_str() {
                fields.insert(format!("req:{s}"));
            }
        }
    }

    // Collect property names and types
    if let Some(props) = schema.get("properties").and_then(|p| p.as_object()) {
        for (name, def) in props {
            let typ = def.get("type").and_then(|t| t.as_str()).unwrap_or("any");
            fields.insert(format!("prop:{name}:{typ}"));
        }
    }

    // Simple hash: join sorted fields and compute a digest
    let input: String = fields.into_iter().collect::<Vec<_>>().join("|");

    // Use a simple FNV-1a-style hash (no external dep needed)
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in input.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

/// Default cache file path.
pub fn cache_path() -> PathBuf {
    termlink_session::discovery::runtime_dir().join("template-cache.json")
}

fn now_iso() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let days = secs / 86400;
    let tod = secs % 86400;
    let h = tod / 3600;
    let m = (tod % 3600) / 60;
    let s = tod % 60;
    let mut y = 1970i64;
    let mut rem = days as i64;
    loop {
        let diy = if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 { 366 } else { 365 };
        if rem < diy { break; }
        rem -= diy;
        y += 1;
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let md: &[i64] = if leap {
        &[31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        &[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut mo = 1;
    for &dm in md {
        if rem < dm { break; }
        rem -= dm;
        mo += 1;
    }
    let d = rem + 1;
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "required": ["title", "findings"],
            "properties": {
                "title": {"type": "string"},
                "findings": {"type": "array"},
                "severity": {"type": "string"}
            }
        })
    }

    #[test]
    fn template_cache_local_hit() {
        let mut cache = TemplateCache::default();
        let schema = sample_schema();
        let hash = compute_schema_hash(&schema);

        cache.record_template("report-v2", "audit-specialist", 1, &hash, schema.clone());

        match cache.lookup("report-v2", None) {
            TemplateLookup::LocalHit(entry) => {
                assert_eq!(entry.specialist, "audit-specialist");
                assert_eq!(entry.version, 1);
            }
            _ => panic!("Expected LocalHit"),
        }
    }

    #[test]
    fn template_cache_shared_hit() {
        let mut cache = TemplateCache::default();
        let schema = sample_schema();
        let hash = compute_schema_hash(&schema);

        // Insert directly into shared (simulating promotion)
        cache.shared.insert(
            "report-v2".to_string(),
            TemplateEntry {
                specialist: "audit-specialist".to_string(),
                format_id: "report-v2".to_string(),
                version: 1,
                schema_hash: hash,
                template: schema,
                hit_count: 10,
                correction_count: 0,
                last_used: now_iso(),
                learned_at: now_iso(),
            },
        );

        match cache.lookup("report-v2", None) {
            TemplateLookup::SharedHit(entry) => {
                assert_eq!(entry.specialist, "audit-specialist");
            }
            _ => panic!("Expected SharedHit"),
        }
    }

    #[test]
    fn template_cache_miss() {
        let cache = TemplateCache::default();
        assert!(matches!(cache.lookup("nonexistent", None), TemplateLookup::Miss));
    }

    #[test]
    fn template_cache_schema_hash_invalidation() {
        let mut cache = TemplateCache::default();
        let schema = sample_schema();
        let hash = compute_schema_hash(&schema);

        cache.record_template("report-v2", "specialist", 1, &hash, schema);

        // Lookup with matching hash → hit
        assert!(matches!(cache.lookup("report-v2", Some(&hash)), TemplateLookup::LocalHit(_)));

        // Lookup with different hash → miss (stale)
        assert!(matches!(cache.lookup("report-v2", Some("different-hash")), TemplateLookup::Miss));
    }

    #[test]
    fn template_cache_promotion() {
        let mut cache = TemplateCache::default();
        let schema = sample_schema();
        let hash = compute_schema_hash(&schema);

        cache.record_template("report-v2", "specialist", 1, &hash, schema);

        // 4 hits — not yet promoted
        for _ in 0..4 {
            assert!(!cache.record_hit("report-v2"));
        }
        assert!(cache.shared.is_empty());

        // 5th hit — promoted!
        assert!(cache.record_hit("report-v2"));
        assert!(cache.shared.contains_key("report-v2"));
        assert_eq!(cache.shared["report-v2"].specialist, "specialist");
        assert_eq!(cache.shared["report-v2"].hit_count, 0); // reset for shared
    }

    #[test]
    fn template_cache_correction_blocks_promotion() {
        let mut cache = TemplateCache::default();
        let schema = sample_schema();
        let hash = compute_schema_hash(&schema);

        cache.record_template("report-v2", "specialist", 1, &hash, schema);

        // 3 hits, then a correction
        for _ in 0..3 {
            cache.record_hit("report-v2");
        }
        cache.record_correction("report-v2");

        // 2 more hits (total 5) — should NOT promote because correction_count > 0
        for _ in 0..2 {
            assert!(!cache.record_hit("report-v2"));
        }
        assert!(cache.shared.is_empty());
    }

    #[test]
    fn template_cache_invalidate() {
        let mut cache = TemplateCache::default();
        cache.record_template("a", "specialist", 1, "hash-a", serde_json::json!({}));

        // Promote to shared too
        for _ in 0..5 {
            cache.record_hit("a");
        }
        assert!(cache.shared.contains_key("a"));

        // Invalidate removes from both layers
        assert!(cache.invalidate("a"));
        assert!(!cache.local.contains_key("a"));
        assert!(!cache.shared.contains_key("a"));
    }

    #[test]
    fn template_cache_invalidate_specialist() {
        let mut cache = TemplateCache::default();
        cache.record_template("a", "git-specialist", 1, "h1", serde_json::json!({}));
        cache.record_template("b", "git-specialist", 1, "h2", serde_json::json!({}));
        cache.record_template("c", "other-specialist", 1, "h3", serde_json::json!({}));

        let removed = cache.invalidate_specialist("git-specialist");
        assert_eq!(removed, 2);
        assert_eq!(cache.local.len(), 1);
        assert!(cache.local.contains_key("c"));
    }

    #[test]
    fn schema_hash_deterministic() {
        let schema = sample_schema();
        let h1 = compute_schema_hash(&schema);
        let h2 = compute_schema_hash(&schema);
        assert_eq!(h1, h2);
    }

    #[test]
    fn schema_hash_different_schemas() {
        let s1 = sample_schema();
        let s2 = serde_json::json!({
            "type": "object",
            "required": ["name", "value"],
            "properties": {
                "name": {"type": "string"},
                "value": {"type": "number"}
            }
        });
        assert_ne!(compute_schema_hash(&s1), compute_schema_hash(&s2));
    }

    #[test]
    fn persistence_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("template-cache.json");

        let mut cache = TemplateCache::default();
        cache.record_template(
            "report-v2",
            "audit-specialist",
            1,
            "abc123",
            serde_json::json!({"type": "object"}),
        );
        // Promote
        for _ in 0..5 {
            cache.record_hit("report-v2");
        }

        cache.save_to(&path).unwrap();

        let loaded = TemplateCache::load_from(&path);
        assert_eq!(loaded.local.len(), 1);
        assert_eq!(loaded.shared.len(), 1);
        assert_eq!(loaded.local["report-v2"].specialist, "audit-specialist");
        assert_eq!(loaded.shared["report-v2"].specialist, "audit-specialist");
    }

    #[test]
    fn load_missing_returns_empty() {
        let cache = TemplateCache::load_from(Path::new("/tmp/nonexistent-template-cache-99999.json"));
        assert!(cache.local.is_empty());
        assert!(cache.shared.is_empty());
    }

    #[test]
    fn local_takes_precedence_over_shared() {
        let mut cache = TemplateCache::default();

        // Shared has v1
        cache.shared.insert(
            "report".to_string(),
            TemplateEntry {
                specialist: "old".to_string(),
                format_id: "report".to_string(),
                version: 1,
                schema_hash: "hash-v1".to_string(),
                template: serde_json::json!({"v": 1}),
                hit_count: 0,
                correction_count: 0,
                last_used: now_iso(),
                learned_at: now_iso(),
            },
        );

        // Local has v2 (agent-specific override)
        cache.record_template("report", "new", 2, "hash-v2", serde_json::json!({"v": 2}));

        match cache.lookup("report", None) {
            TemplateLookup::LocalHit(entry) => {
                assert_eq!(entry.version, 2);
                assert_eq!(entry.specialist, "new");
            }
            _ => panic!("Expected LocalHit (local should take precedence)"),
        }
    }
}
