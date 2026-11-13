//! T-3764: every `origin`/`basis` pair an adapter is told to send by the shared parity
//! fixtures (`tests/fixtures/b25-parity/*.json`) is accepted by the meter's AL-1
//! `check_origin` table (06 AL-1, FCW-4). The r3 fixture once carried basis `watchdog`,
//! which the table rejects; a watchdog line is the same SI-1 line as the doorbell's
//! (termlink-session handler.rs), so the basis is `doorbell`.
//! Each pair goes through the real meter (`intake` of a `prompt_submit`) and must leave
//! `origin_inconsistent` at 0. Negative control: the old pair (automated, watchdog) is
//! counted inconsistent by the same path, so the check can fail.
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use termlink_sidecar::meter::*;

const A: &str = "agent-t3764";

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

/// Feed one prompt_submit with `origin`/`basis` to a fresh meter; true when the
/// meter's table accepts the pair (no FCW-4 count).
fn meter_accepts(origin: &str, basis: &str) -> bool {
    let now = Arc::new(AtomicU64::new(1_000_000));
    let (n1, n2) = (now.clone(), now.clone());
    let m = Meter::new(A, Box::new(NoS1), Box::new(NoLedger))
        .with_clocks(Arc::new(move || n1.load(Ordering::SeqCst)), Arc::new(move || n2.load(Ordering::SeqCst)));
    let v = json!({"schema": SCHEMA, "harness": "fake", "agent_id": A, "session_id": "s",
                   "event": "prompt_submit", "at_ms": 1, "origin": origin, "basis": basis});
    let intake = m.intake(v.to_string().as_bytes());
    assert_eq!(intake, Intake::Applied, "{origin}/{basis}: the report itself is well formed");
    m.counters().origin_inconsistent == 0
}

/// Every object anywhere in `v` carrying both `origin` and `basis` as strings.
fn pairs(v: &Value, out: &mut Vec<(String, String)>) {
    match v {
        Value::Object(o) => {
            if let (Some(Value::String(a)), Some(Value::String(b))) = (o.get("origin"), o.get("basis")) {
                out.push((a.clone(), b.clone()));
            }
            o.values().for_each(|x| pairs(x, out));
        }
        Value::Array(a) => a.iter().for_each(|x| pairs(x, out)),
        _ => {}
    }
}

/// The pairs the meter rejects among those in `doc`.
fn rejected(doc: &Value) -> Vec<(String, String)> {
    let mut p = Vec::new();
    pairs(doc, &mut p);
    p.sort();
    p.dedup();
    p.into_iter().filter(|(o, b)| !meter_accepts(o, b)).collect()
}

#[test]
fn t3764_every_fixture_basis_is_in_the_meter_table() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/b25-parity");
    let mut files = 0;
    let mut seen = 0;
    for e in std::fs::read_dir(&dir).expect("parity fixture dir") {
        let path = e.unwrap().path();
        if path.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        files += 1;
        let doc: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).expect("fixture parses");
        let mut p = Vec::new();
        pairs(&doc, &mut p);
        seen += p.len();
        let bad = rejected(&doc);
        assert!(bad.is_empty(), "{}: bases the meter's check_origin table rejects (FCW-4): {bad:?}", path.display());
    }
    // Vacuity guard: the scan reached both fixtures and the r3 origin cases.
    assert!(files >= 2, "found {files} fixture files under {}", dir.display());
    assert!(seen >= 8, "only {seen} origin/basis pairs found");
}

#[test]
fn t3764_negative_control_watchdog_basis_is_rejected() {
    // The pre-T-3764 r3 case, verbatim.
    let old = json!({"cases": [{"name": "watchdog-line-is-automated", "state": "held-by-other-silent",
        "hook": "prompt", "origin": "automated", "basis": "watchdog", "expect": "ignored"}]});
    assert_eq!(rejected(&old), vec![("automated".to_string(), "watchdog".to_string())]);
    // And the same line as it is now reported passes.
    assert!(meter_accepts("automated", "doorbell"));
}
