//! T-1406: Regression guard — no new direct callers of legacy primitives.
//!
//! During the T-1166 bake window, any new in-repo direct caller of
//! `event.broadcast`, `inbox.list`, `inbox.status`, `inbox.clear`, `file.send`,
//! or `file.receive` increases legacy traffic and slows the gate. This test
//! walks `crates/**/src/**/*.rs` and fails if any source file outside the
//! allowlist contains one of these method strings as a quoted literal at a
//! caller-shaped use-site.
//!
//! Allowlisted files (intentional callers — fallback paths or definitions):
//! - `crates/termlink-hub/src/router.rs` — the router that handles them
//! - `crates/termlink-hub/src/rpc_audit.rs` — the legacy method definition list
//! - `crates/termlink-cli/src/commands/events.rs` — `cmd_broadcast` legacy fallback (T-1401)
//! - `crates/termlink-cli/src/commands/infrastructure.rs` — `fw doctor` legacy fallback (T-1400)
//! - `crates/termlink-mcp/src/tools.rs` — MCP broadcast + doctor legacy fallbacks (T-1400 / T-1403)
//! - `crates/termlink-session/src/inbox_channel.rs` — channel-first wrapper, legacy fallback path
//!
//! Skipped use-sites (definitionally not callers):
//! - Comment lines (`//`, `///`, `//!`, leading `*` inside block comments)
//! - Constant declarations (`pub const X: &str = "..."`)
//! - Match-arm patterns (`"..." =>`, `| "..."`)
//! - Code inside `#[cfg(test)]` modules, `#[test]` fns, and `#[tokio::test]` fns
//!
//! Test files (`tests/`, `_test.rs`, `benches/`) are skipped wholesale — they
//! may legitimately exercise the legacy path to assert backward-compat behaviour.
//!
//! Any new hit is a regression. The migration recipe in
//! `docs/migrations/T-1166-retire-legacy-primitives.md` shows the channel-based
//! replacement.

use std::path::{Path, PathBuf};

const LEGACY_METHODS: &[&str] = &[
    "event.broadcast",
    "inbox.list",
    "inbox.status",
    "inbox.clear",
    "file.send",
    "file.receive",
];

const ALLOWLIST: &[&str] = &[
    "crates/termlink-hub/src/router.rs",
    "crates/termlink-hub/src/rpc_audit.rs",
    "crates/termlink-cli/src/commands/events.rs",
    "crates/termlink-cli/src/commands/infrastructure.rs",
    "crates/termlink-mcp/src/tools.rs",
    "crates/termlink-session/src/inbox_channel.rs",
];

#[test]
fn no_new_direct_legacy_callers() {
    let workspace_root = workspace_root();
    let crates_dir = workspace_root.join("crates");
    let mut violations: Vec<String> = Vec::new();

    walk_rs(&crates_dir, &mut |path| {
        let rel = path.strip_prefix(&workspace_root).unwrap_or(path);
        let rel_str = rel.to_string_lossy().replace('\\', "/");

        // Skip test files and benches wholesale.
        if rel_str.contains("/tests/") || rel_str.contains("/benches/") {
            return;
        }
        if rel_str.ends_with("_test.rs") {
            return;
        }

        // Skip allowlisted source files.
        if ALLOWLIST.iter().any(|a| rel_str == *a) {
            return;
        }

        let body = match std::fs::read_to_string(path) {
            Ok(b) => b,
            Err(_) => return,
        };
        scan_file(&rel_str, &body, &mut violations);
    });

    assert!(
        violations.is_empty(),
        "T-1406: in-repo direct callers of legacy primitives found outside allowlist:\n\n{}\n\n\
         If this is an intentional fallback path that ships during the T-1166 bake \
         window, add the file to ALLOWLIST in this test (and reference the migration \
         task that established the fallback). Otherwise, migrate to the channel-based \
         replacement — see docs/migrations/T-1166-retire-legacy-primitives.md.",
        violations.join("\n"),
    );
}

#[test]
fn allowlist_entries_exist() {
    // Defensive: if a path in ALLOWLIST is renamed or deleted, the guard would
    // silently lose coverage. Fail loudly when an allowlist entry no longer
    // points at a real file.
    let root = workspace_root();
    let mut missing: Vec<&str> = Vec::new();
    for entry in ALLOWLIST {
        if !root.join(entry).is_file() {
            missing.push(entry);
        }
    }
    assert!(
        missing.is_empty(),
        "T-1406: allowlist contains paths that no longer exist:\n  {}",
        missing.join("\n  "),
    );
}

/// Predicate-level smoke test for the line-classifier. Catches future
/// regressions in `is_caller_line` without needing a full filesystem walk.
#[test]
fn classifier_recognises_known_non_callers() {
    // Comments
    assert!(!is_caller_line(r#"// "event.broadcast" is the legacy method"#));
    assert!(!is_caller_line(r#"/// uses "event.broadcast" as msg_type"#));
    assert!(!is_caller_line(r#"//! refers to "inbox.status""#));
    assert!(!is_caller_line(r#"   * doc continuation "file.send""#));

    // Constant declarations
    assert!(!is_caller_line(r#"pub const EVENT_BROADCAST: &str = "event.broadcast";"#));
    assert!(!is_caller_line(r#"const X: &str = "inbox.list";"#));

    // Match arms
    assert!(!is_caller_line(r#"        | "event.broadcast""#));
    assert!(!is_caller_line(r#"        "inbox.status" => handle_inbox_status(id),"#));

    // Real callers — must be flagged
    assert!(is_caller_line(r#"client.call("event.broadcast", id, params)"#));
    assert!(is_caller_line(r#"rpc_call(&sock, "inbox.status", json!({}))"#));
    assert!(is_caller_line(r#"let req = json!({"method": "event.broadcast", ...});"#));
}

fn scan_file(rel: &str, body: &str, violations: &mut Vec<String>) {
    let mut in_test_block = TestBlockTracker::new();
    for (lineno, line) in body.lines().enumerate() {
        in_test_block.step(line);
        if in_test_block.inside_test() {
            continue;
        }
        if !is_caller_line(line) {
            continue;
        }
        for method in LEGACY_METHODS {
            let needle = format!("\"{method}\"");
            if line.contains(&needle) {
                violations.push(format!(
                    "{rel}:{}: {}",
                    lineno + 1,
                    line.trim()
                ));
            }
        }
    }
}

/// Returns true if `line` looks like a use-site that could be a JSON-RPC
/// caller. Skips comments, constant declarations, and match-arm patterns.
fn is_caller_line(line: &str) -> bool {
    let trimmed = line.trim_start();

    // Comment lines.
    if trimmed.starts_with("//") || trimmed.starts_with("*") {
        return false;
    }

    // Constant declarations: `pub const X: &str = "..."` or `const X: &str = "..."`.
    if (trimmed.starts_with("pub const ") || trimmed.starts_with("const "))
        && trimmed.contains(": &str =")
    {
        return false;
    }

    // Match-arm patterns: lines whose non-trivial content is a single string
    // literal followed by `=>` or preceded by `|`.
    let stripped = trimmed.trim_end_matches(|c: char| c.is_whitespace() || c == ',');
    if stripped.starts_with('|') {
        return false;
    }
    // `"foo" =>` shape — the line is a match arm head.
    if stripped.starts_with('"')
        && (stripped.ends_with("=>") || stripped.contains("\" =>"))
        && !stripped.contains(".call(")
        && !stripped.contains("rpc_call(")
        && !stripped.contains("\"method\":")
    {
        return false;
    }

    true
}

/// Tracks whether the current line is inside a `#[cfg(test)]` mod or
/// `#[test]` / `#[tokio::test]` fn. Heuristic but reliable for our codebase:
/// when we see one of these attributes, the next opening brace begins a
/// test-only block; we count braces until that block closes.
struct TestBlockTracker {
    armed: bool,
    depth: u32,
}

impl TestBlockTracker {
    fn new() -> Self {
        Self {
            armed: false,
            depth: 0,
        }
    }

    fn inside_test(&self) -> bool {
        self.depth > 0
    }

    fn step(&mut self, line: &str) {
        let trimmed = line.trim_start();

        // Arm on test-attribute lines. Multiple attributes can stack so
        // staying armed across them is fine.
        if trimmed.starts_with("#[cfg(test)]")
            || trimmed.starts_with("#[test]")
            || trimmed.starts_with("#[tokio::test]")
        {
            self.armed = true;
            return;
        }

        // If we're armed and this line opens a block, start tracking depth.
        if self.armed {
            let opens = line.matches('{').count() as u32;
            let closes = line.matches('}').count() as u32;
            if opens > 0 {
                self.depth += opens;
                self.depth = self.depth.saturating_sub(closes);
                self.armed = false;
            }
            // If armed but no `{` yet (e.g. multi-line fn signature),
            // stay armed.
            return;
        }

        // Plain depth tracking while inside a test block.
        if self.depth > 0 {
            let opens = line.matches('{').count() as u32;
            let closes = line.matches('}').count() as u32;
            self.depth += opens;
            self.depth = self.depth.saturating_sub(closes);
        }
    }
}

fn walk_rs(dir: &Path, f: &mut dyn FnMut(&Path)) {
    let entries = match std::fs::read_dir(dir) {
        Ok(it) => it,
        Err(_) => return,
    };
    for ent in entries.flatten() {
        let p = ent.path();
        if p.is_dir() {
            walk_rs(&p, f);
        } else if p.extension().is_some_and(|e| e == "rs") {
            f(&p);
        }
    }
}

fn workspace_root() -> PathBuf {
    // termlink-hub's CARGO_MANIFEST_DIR is `<workspace>/crates/termlink-hub`.
    // Walk up until we find a Cargo.toml that owns a `crates/` sibling — that's
    // the workspace root.
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if p.join("crates").is_dir() && p.join("Cargo.toml").is_file() {
            return p;
        }
        if !p.pop() {
            panic!("could not locate workspace root from CARGO_MANIFEST_DIR");
        }
    }
}
