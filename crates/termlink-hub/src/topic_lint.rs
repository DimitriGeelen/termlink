//! Topic↔role soft-lint at emit (T-1300, per T-1297 GO § Spike 3).
//!
//! Rules live in `<runtime_dir>/topic_roles.yaml` if present, else
//! [`Rules::defaults`] supplies the 10 prefix rules + 4 exempt categories
//! that cover 95% of the current 125-topic catalog. SIGHUP triggers a reload.
//! Lint is **soft**: warnings dual-write to bus topic `routing:lint` but
//! never reject the emit.

use std::path::Path;
use std::sync::{Arc, RwLock};

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};

/// One prefix rule. A topic matches if it starts with `prefix` followed by
/// either `.`, `:`, or end-of-string. Roles list the session-roles allowed
/// to emit topics under that prefix.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Rule {
    pub prefix: String,
    #[serde(default)]
    pub roles: Vec<String>,
    /// If true, the rule passes whenever the caller's session-role *is*
    /// part of the topic prefix's product family — used for `oauth.*`,
    /// `<project>.*` style rules where the role-name equals the prefix.
    /// V1 implementation: caller's roles must contain the rule's `prefix`
    /// verbatim. Stub for Build C / T-1301.
    #[serde(default)]
    pub roles_from_originator_role: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Rules {
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub exempt_prefixes: Vec<String>,
}

impl Rules {
    /// Built-in defaults from T-1297 § Spike 3. Cover 95% of 125 topics.
    pub fn defaults() -> Self {
        let r = |prefix: &str, roles: &[&str]| Rule {
            prefix: prefix.to_string(),
            roles: roles.iter().map(|s| s.to_string()).collect(),
            roles_from_originator_role: false,
        };
        let r_origin = |prefix: &str| Rule {
            prefix: prefix.to_string(),
            roles: Vec::new(),
            roles_from_originator_role: true,
        };
        Rules {
            rules: vec![
                r("framework", &["framework", "pickup"]),
                r("channel", &["framework"]),
                r("pickup", &["framework", "pickup"]),
                r("learning", &["framework"]),
                r("inception", &["framework"]),
                r("claude.md", &["framework"]),
                r("gap", &["framework"]),
                r("peer", &["framework"]),
                r("infra", &["ring20-management", "infrastructure"]),
                r("outage", &["ring20-management", "infrastructure"]),
                r_origin("oauth"),
                r_origin("task"),
            ],
            exempt_prefixes: vec![
                "agent.".into(),
                "session.".into(),
                "worker.".into(),
                "test.".into(),
                "help.".into(),
                "channel.delivery".into(),
            ],
        }
    }

    /// Parse YAML at `path`. Unknown keys tolerated (forward-compat).
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("topic_lint: read {}", path.display()))?;
        let rules: Rules = serde_yaml::from_str(&raw)
            .with_context(|| format!("topic_lint: parse {}", path.display()))?;
        Ok(rules)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LintOutcome {
    Pass,
    ExemptMatch,
    NoMatchingRule,
    Warn {
        rule_prefix: String,
        expected_roles: Vec<String>,
        actual_roles: Vec<String>,
    },
}

/// Test whether `topic` starts with `prefix` followed by a separator (`.` or
/// `:`) or end-of-string. Pure prefix-with-boundary semantics — `framework`
/// matches `framework:pickup` and `framework.gap` but not `frameworks.x`.
fn topic_has_prefix(topic: &str, prefix: &str) -> bool {
    if let Some(rest) = topic.strip_prefix(prefix) {
        rest.is_empty()
            || rest.starts_with('.')
            || rest.starts_with(':')
    } else {
        false
    }
}

/// Pure lint function. No I/O.
pub fn lint(topic: &str, caller_roles: &[String], rules: &Rules) -> LintOutcome {
    // Exempt prefixes win first — operational topics never warn.
    for ex in &rules.exempt_prefixes {
        // Exempt entries already include the trailing `.` or full token.
        if topic == ex.trim_end_matches('.') || topic.starts_with(ex.as_str()) {
            return LintOutcome::ExemptMatch;
        }
    }

    // Find the most-specific (longest) matching rule.
    let mut best: Option<&Rule> = None;
    for rule in &rules.rules {
        if topic_has_prefix(topic, &rule.prefix)
            && best
                .as_ref()
                .map(|b| rule.prefix.len() > b.prefix.len())
                .unwrap_or(true)
        {
            best = Some(rule);
        }
    }

    let Some(rule) = best else {
        return LintOutcome::NoMatchingRule;
    };

    let expected: Vec<String> = if rule.roles_from_originator_role {
        vec![rule.prefix.clone()]
    } else {
        rule.roles.clone()
    };

    let role_match = expected.iter().any(|r| caller_roles.iter().any(|c| c == r));
    if role_match {
        LintOutcome::Pass
    } else {
        LintOutcome::Warn {
            rule_prefix: rule.prefix.clone(),
            expected_roles: expected,
            actual_roles: caller_roles.to_vec(),
        }
    }
}

/// Process-global Rules state, mutated by SIGHUP reloads.
static RULES: std::sync::OnceLock<Arc<RwLock<Rules>>> = std::sync::OnceLock::new();

/// Path the reload handler re-reads from on SIGHUP. None = use defaults.
static RULES_PATH: std::sync::OnceLock<Option<std::path::PathBuf>> = std::sync::OnceLock::new();

/// Initialise the lint engine. Called once by the hub server at startup.
/// If `<runtime_dir>/topic_roles.yaml` exists it is parsed; otherwise the
/// built-in defaults are installed. Either way the global state is set so
/// later [`current_rules`] calls succeed.
pub fn init(runtime_dir: &Path) {
    let path = runtime_dir.join("topic_roles.yaml");
    let (rules, used_path) = if path.is_file() {
        match Rules::load_from_path(&path) {
            Ok(r) => {
                tracing::info!(
                    file = %path.display(),
                    rule_count = r.rules.len(),
                    exempt_count = r.exempt_prefixes.len(),
                    "topic_lint: loaded rules from file"
                );
                (r, Some(path.clone()))
            }
            Err(e) => {
                tracing::warn!(
                    file = %path.display(),
                    error = %e,
                    "topic_lint: failed to parse rule file — falling back to defaults"
                );
                (Rules::defaults(), Some(path.clone()))
            }
        }
    } else {
        tracing::info!(
            file = %path.display(),
            "topic_lint: rule file absent — using defaults"
        );
        (Rules::defaults(), None)
    };
    let _ = RULES.set(Arc::new(RwLock::new(rules)));
    let _ = RULES_PATH.set(used_path);
}

/// Snapshot current rules (cheap clone of the inner Arc-RwLock guard's data).
pub fn current_rules() -> Rules {
    RULES
        .get()
        .and_then(|r| r.read().ok().map(|g| g.clone()))
        .unwrap_or_else(Rules::defaults)
}

/// Reload rules from the file recorded at [`init`] time. Used by the SIGHUP
/// task. On parse failure the previous rules stay in place.
pub fn reload() {
    let Some(Some(path)) = RULES_PATH.get() else {
        tracing::info!("topic_lint: SIGHUP — no file path recorded; nothing to reload");
        return;
    };
    match Rules::load_from_path(path) {
        Ok(new_rules) => {
            if let Some(slot) = RULES.get()
                && let Ok(mut g) = slot.write()
            {
                *g = new_rules;
                tracing::info!(file = %path.display(), "topic_lint: reloaded rules");
            }
        }
        Err(e) => {
            tracing::warn!(
                file = %path.display(),
                error = %e,
                "topic_lint: reload failed — keeping previous rules"
            );
        }
    }
}

/// Spawn a SIGHUP watcher that calls [`reload`] on every signal. Idempotent.
pub fn spawn_sighup_watcher() {
    use tokio::signal::unix::{signal, SignalKind};
    tokio::spawn(async move {
        let mut sig = match signal(SignalKind::hangup()) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(error = %e, "topic_lint: SIGHUP watcher failed to install");
                return;
            }
        };
        tracing::info!("topic_lint: SIGHUP watcher installed");
        while sig.recv().await.is_some() {
            reload();
        }
    });
}

/// Build the JSON payload posted to bus topic `routing:lint` when lint warns.
/// Payload shape is documented for downstream subscribers (Watchtower, etc).
pub fn warning_payload(
    method: &str,
    topic: &str,
    from: Option<&str>,
    rule_prefix: &str,
    expected_roles: &[String],
    actual_roles: &[String],
) -> Value {
    json!({
        "type": "routing.lint.warning",
        "method": method,
        "topic": topic,
        "from": from,
        "rule_prefix": rule_prefix,
        "expected_roles": expected_roles,
        "actual_roles": actual_roles,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roles(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn defaults_match_framework_pickup_for_role_framework() {
        let rules = Rules::defaults();
        let outcome = lint("framework:pickup", &roles(&["framework"]), &rules);
        assert_eq!(outcome, LintOutcome::Pass);
    }

    #[test]
    fn defaults_warn_on_framework_pickup_for_role_product() {
        let rules = Rules::defaults();
        let outcome = lint("framework:pickup", &roles(&["product"]), &rules);
        match outcome {
            LintOutcome::Warn {
                rule_prefix,
                expected_roles,
                actual_roles,
            } => {
                assert_eq!(rule_prefix, "framework");
                assert!(expected_roles.contains(&"framework".to_string()));
                assert_eq!(actual_roles, vec!["product".to_string()]);
            }
            other => panic!("expected Warn, got {other:?}"),
        }
    }

    #[test]
    fn exempt_prefix_returns_exempt_regardless_of_role() {
        let rules = Rules::defaults();
        assert_eq!(
            lint("agent.request", &roles(&["product"]), &rules),
            LintOutcome::ExemptMatch
        );
        assert_eq!(
            lint("agent.response", &[], &rules),
            LintOutcome::ExemptMatch
        );
    }

    #[test]
    fn yaml_loader_parses_sample_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("topic_roles.yaml");
        std::fs::write(
            &path,
            r#"
rules:
  - prefix: "framework"
    roles: [framework, pickup]
  - prefix: "infra"
    roles: [ring20-management]
  - prefix: "oauth"
    roles_from_originator_role: true
exempt_prefixes:
  - "agent."
  - "session."
"#,
        )
        .unwrap();
        let r = Rules::load_from_path(&path).unwrap();
        assert_eq!(r.rules.len(), 3);
        assert_eq!(r.rules[0].prefix, "framework");
        assert_eq!(r.rules[0].roles, vec!["framework", "pickup"]);
        assert!(r.rules[2].roles_from_originator_role);
        assert_eq!(r.exempt_prefixes, vec!["agent.", "session."]);
    }

    #[test]
    fn caller_with_no_roles_warns_on_non_exempt_topic() {
        let rules = Rules::defaults();
        match lint("framework:pickup", &[], &rules) {
            LintOutcome::Warn { actual_roles, .. } => assert!(actual_roles.is_empty()),
            other => panic!("expected Warn, got {other:?}"),
        }
    }

    #[test]
    fn caller_with_no_roles_exempt_topic_passes_as_exempt() {
        let rules = Rules::defaults();
        assert_eq!(
            lint("worker.done", &[], &rules),
            LintOutcome::ExemptMatch
        );
    }

    #[test]
    fn no_matching_rule_returns_no_matching_rule() {
        let rules = Rules::defaults();
        // Random unmapped topic with no exempt prefix
        assert_eq!(
            lint("zzz-unmapped-topic", &roles(&["framework"]), &rules),
            LintOutcome::NoMatchingRule
        );
    }

    #[test]
    fn topic_prefix_boundary_rejects_lookalike() {
        let rules = Rules::defaults();
        // "frameworks" should NOT match "framework" prefix — boundary required
        assert_eq!(
            lint("frameworkz.x", &roles(&["framework"]), &rules),
            LintOutcome::NoMatchingRule
        );
    }

    #[test]
    fn most_specific_rule_wins() {
        // Add an overlapping more-specific rule
        let mut rules = Rules::defaults();
        rules.rules.push(Rule {
            prefix: "framework.gap".into(),
            roles: vec!["audit".into()],
            roles_from_originator_role: false,
        });
        // Generic role=framework would Pass under "framework" but not under
        // "framework.gap" which expects role=audit.
        match lint("framework.gap", &roles(&["framework"]), &rules) {
            LintOutcome::Warn { rule_prefix, .. } => assert_eq!(rule_prefix, "framework.gap"),
            other => panic!("expected Warn from most-specific rule, got {other:?}"),
        }
    }

    #[test]
    fn yaml_tolerates_unknown_keys() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("topic_roles.yaml");
        std::fs::write(
            &path,
            r#"
version: 2
description: "some forward-compat thing"
rules:
  - prefix: "framework"
    roles: [framework]
exempt_prefixes: ["agent."]
"#,
        )
        .unwrap();
        let r = Rules::load_from_path(&path).unwrap();
        assert_eq!(r.rules.len(), 1);
    }

    #[test]
    fn hot_reload_via_repeated_load_reflects_file_changes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("topic_roles.yaml");
        std::fs::write(
            &path,
            "rules:\n  - prefix: \"framework\"\n    roles: [framework]\n",
        )
        .unwrap();
        let r1 = Rules::load_from_path(&path).unwrap();
        assert_eq!(r1.rules.len(), 1);
        std::fs::write(
            &path,
            "rules:\n  - prefix: \"framework\"\n    roles: [framework]\n  - prefix: \"infra\"\n    roles: [ops]\n",
        )
        .unwrap();
        let r2 = Rules::load_from_path(&path).unwrap();
        assert_eq!(r2.rules.len(), 2);
    }

    #[test]
    fn warning_payload_serializes_expected_fields() {
        let p = warning_payload(
            "event.broadcast",
            "framework:pickup",
            Some("session-abc"),
            "framework",
            &roles(&["framework", "pickup"]),
            &roles(&["product"]),
        );
        assert_eq!(p["type"], "routing.lint.warning");
        assert_eq!(p["method"], "event.broadcast");
        assert_eq!(p["topic"], "framework:pickup");
        assert_eq!(p["from"], "session-abc");
        assert_eq!(p["rule_prefix"], "framework");
    }
}
