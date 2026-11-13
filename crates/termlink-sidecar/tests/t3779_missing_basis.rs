//! T-3779: a `prompt_submit` without a `basis` field (or with `basis: null`) carries basis
//! `absent` (04 17.5.c, 06 AL-1). Origin `unknown` + no basis is the honest "cannot decide"
//! report and must not count as FCW-4 `origin_inconsistent`. Negative controls: `human` or
//! `automated` with no basis, and `unknown` with a wrong explicit basis, still count; a basis of
//! the wrong JSON type is refused as FCW-2 (unparseable); a missing origin stays FCW-2.
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use termlink_sidecar::meter::*;

const A: &str = "agent-t3779";

struct NoS1;
impl S1Source for NoS1 {
    fn read(&self, _: &str) -> Result<S1Reply, String> {
        Err("unbound".into())
    }
}
struct NoLedger;
impl Ledger for NoLedger {
    fn owed(&self, _: &str, _: u64) -> Result<bool, String> {
        Ok(false)
    }
}

fn meter() -> Meter {
    let now = Arc::new(AtomicU64::new(1_000_000));
    let (n1, n2) = (now.clone(), now.clone());
    Meter::new(A, Box::new(NoS1), Box::new(NoLedger))
        .with_clocks(Arc::new(move || n1.load(Ordering::SeqCst)), Arc::new(move || n2.load(Ordering::SeqCst)))
}

/// One prompt_submit with `extra` merged in, to a fresh meter: (intake, recorded origin/basis, counters).
fn submit(extra: Value) -> (Intake, Option<(Origin, String)>, Counters) {
    let m = meter();
    let mut v = json!({"schema": SCHEMA, "harness": "fake", "agent_id": A, "session_id": "s",
                       "event": "prompt_submit", "at_ms": 1});
    for (k, x) in extra.as_object().unwrap() {
        v[k] = x.clone();
    }
    let i = m.intake(v.to_string().as_bytes());
    (i, m.last_prompt("s").map(|p| (p.origin, p.basis)), m.counters())
}

#[test]
fn t3779_unknown_without_basis_is_absent_not_inconsistent() {
    for extra in [json!({"origin": "unknown"}), json!({"origin": "unknown", "basis": null})] {
        let (i, p, c) = submit(extra.clone());
        assert_eq!(i, Intake::Applied, "{extra}");
        assert_eq!(p, Some((Origin::Unknown, "absent".into())), "{extra}");
        assert_eq!(c.origin_inconsistent, 0, "{extra}: an unknown report with no basis is consistent");
        assert_eq!(c.unparseable, 0, "{extra}");
    }
    // The explicit form AEF sends (ee51d25b7) is unchanged.
    let (_, p, c) = submit(json!({"origin": "unknown", "basis": "absent"}));
    assert_eq!((p, c.origin_inconsistent), (Some((Origin::Unknown, "absent".into())), 0));
}

#[test]
fn t3779_controls_still_count() {
    // FCW-4: a missing basis is `absent`, which only `unknown` may carry.
    for extra in [
        json!({"origin": "human"}),
        json!({"origin": "automated"}),
        json!({"origin": "unknown", "basis": "pty-input"}),
        json!({"origin": "unknown", "basis": ""}),
    ] {
        let (i, p, c) = submit(extra.clone());
        assert_eq!(i, Intake::Applied, "{extra}");
        assert_eq!(p, Some((Origin::Unknown, "absent".into())), "{extra}");
        assert_eq!(c.origin_inconsistent, 1, "{extra}: still FCW-4");
    }
    // FCW-2: wrong type, or no origin at all, is refused before anything is applied.
    for extra in [json!({"origin": "unknown", "basis": 7}), json!({"origin": "unknown", "basis": ["absent"]}), json!({"basis": "absent"})] {
        let (i, p, c) = submit(extra.clone());
        assert!(matches!(i, Intake::Ignored(_)), "{extra}: {i:?}");
        assert_eq!((p, c.unparseable, c.origin_inconsistent), (None, 1, 0), "{extra}");
    }
}

/// Every r3 parity case that is a prompt with an origin but no basis is accepted by the meter
/// when sent the way the fixture states it (no basis field). Today that is
/// `unknown-origin-is-ignored`; the count guards against the loop running over nothing.
#[test]
fn t3779_fixture_prompts_without_basis_are_consistent() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/b25-parity/r3-prompt-origin.json");
    let doc: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let mut n = 0;
    for case in doc["cases"].as_array().unwrap() {
        if case["hook"] != "prompt" || case.get("basis").is_some() {
            continue;
        }
        let Some(origin) = case["origin"].as_str() else { continue }; // no origin: FCW-2, tested above
        n += 1;
        let (i, _, c) = submit(json!({"origin": origin}));
        assert_eq!((i, c.origin_inconsistent), (Intake::Applied, 0), "{}", case["name"]);
    }
    assert!(n >= 1, "r3 has at least one prompt case with an origin and no basis");
}
