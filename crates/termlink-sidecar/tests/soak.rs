//! Soak tests, run on demand, that no stored message is lost or handed over twice over a long run.
//! B13 soak (X5): H-16 and H-17 (step 3 section 9).
//!
//!   cargo test -p termlink-sidecar --release soak_ -- --ignored --nocapture
//!
//! H-16: "no stored message is lost and none is handed over twice" over 1000
//! randomised kill points across the store (the write sequence and every chain
//! append), the claim and the record (the deliver kill points). In-process, no hub:
//! the receive path's network half is covered elsewhere; what a crash can break is
//! the store, the claim and the record, and those are exercised here directly.
//! `SOAK_KILLS` and `SOAK_SEED` override the defaults (1000, 20261012).
//!
//! H-17: "every listed error ends visibly": the 17 FC rows of step 3 section 5.6,
//! each with a test and a control that went red (B12's matrix).

use serde_json::json;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use termlink_session::agent_identity::Identity;
use termlink_sidecar::deliver::{self, Deliverer, KillPoint, Pulled};
use termlink_sidecar::policy::BuiltIn;
use termlink_sidecar::store::{self, ChainStatus, PutOutcome, Step, Store, StoreError};
use termlink_sidecar::testkit::Estate;
use termlink_sidecar::wire::{Msg, Roster};

const B: &str = "agent-b";
const SESSION: &str = "soak-session";

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn chance(&mut self, pct: u64) -> bool {
        self.below(100) < pct
    }
}

struct Life {
    store: Arc<Store>,
    d: Deliverer,
}

fn open(e: &Estate, ka: &Identity, transcript: &Path) -> Life {
    let store = Arc::new(Store::open(&e.store_root(), B).unwrap());
    let mut r = Roster::new();
    r.add(ka.verifying_key());
    let d = Deliverer::new(B, store.clone(), Box::new(BuiltIn::new(r)), None);
    d.register_session(SESSION, transcript).unwrap();
    Life { store, d }
}

fn append(transcript: &Path, recs: &[serde_json::Value]) {
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(transcript).unwrap();
    for r in recs {
        writeln!(f, "{r}").unwrap();
    }
}

/// The harness after a tool call: tool_use, tool_result, optionally the hook's
/// frame as a harness record, then the model's next turn.
fn tool_call(transcript: &Path, tool: &str, frame: Option<&str>) {
    let mut recs = vec![
        json!({"type": "assistant", "message": {"content": [{"type": "tool_use", "id": tool}]}}),
        json!({"type": "user", "message": {"content": [{"type": "tool_result", "tool_use_id": tool, "content": "ok"}]}}),
    ];
    if let Some(f) = frame {
        recs.push(json!({"type": "attachment", "attachment": {"type": "hook_additional_context", "content": [f], "toolUseID": tool}}));
    }
    recs.push(json!({"type": "assistant", "message": {"content": [{"type": "text", "text": "next"}]}}));
    append(transcript, &recs);
}

fn is_kill(err: &str) -> bool {
    err.contains(deliver::KILLED) || err.contains("failpoint")
}

#[derive(Default, Debug)]
struct Counts {
    kills_store_put: u64,
    kills_store_other: u64,
    kills_deliver: BTreeMap<String, u64>,
    plain_restarts: u64,
    injected: u64,
    frames: u64,
    frames_dropped_by_harness: u64,
    finalize_calls: u64,
}

impl Counts {
    fn kills(&self) -> u64 {
        self.kills_store_put + self.kills_store_other + self.kills_deliver.values().sum::<u64>()
    }
}

#[test]
#[ignore = "soak: 1000 kill points; run with --release -- --ignored"]
fn soak_h16_no_stored_message_lost_none_handed_over_twice() {
    let target: u64 = std::env::var("SOAK_KILLS").ok().and_then(|s| s.parse().ok()).unwrap_or(1000);
    let seed: u64 = std::env::var("SOAK_SEED").ok().and_then(|s| s.parse().ok()).unwrap_or(20261012);
    let mut rng = Rng(seed | 1);
    let e = Estate::create_without_hub().unwrap();
    let ka = e.identity("a").unwrap();
    let transcript: PathBuf = e.root().join("transcript.jsonl");
    std::fs::write(&transcript, "").unwrap();
    let mut life = Some(open(&e, &ka, &transcript));
    let mut c = Counts::default();
    let mut injected: BTreeMap<String, Msg> = BTreeMap::new();
    let mut unconfirmed: Vec<String> = vec![]; // put never returned Stored/Duplicate: the sender still chases it
    let mut tool_n = 0u64;
    let crash = |life: &mut Option<Life>| {
        store::disarm();
        deliver::disarm_kill();
        *life = None;
        *life = Some(open(&e, &ka, &transcript));
    };
    let mut iterations = 0u64;
    while c.kills() < target {
        iterations += 1;
        assert!(iterations < target * 200, "the soak is not reaching its kill points: {c:?}");
        let l = life.as_ref().unwrap();
        // A store kill point may be armed for any operation (every chain append
        // goes through the store); it is disarmed again if it did not fire.
        let store_kill = if rng.chance(25) { Some(Step::ALL[rng.below(Step::ALL.len() as u64) as usize]) } else { None };
        if let Some(s) = store_kill {
            store::arm(s);
        }
        let op = rng.below(10);
        let mut killed = false;
        if op < 3 {
            // A message arrives (or the sender retries one it never saw stored).
            let undelivered = injected.len() as u64 - c.frames.min(injected.len() as u64);
            let id = if !unconfirmed.is_empty() && rng.chance(60) {
                unconfirmed[rng.below(unconfirmed.len() as u64) as usize].clone()
            } else if undelivered < 40 {
                let n = c.injected;
                c.injected += 1;
                let m = Msg {
                    from: ka.fingerprint().to_owned(),
                    from_agent: "agent-a".into(),
                    from_instance: "inst-a".into(),
                    to_agent: B.into(),
                    to_instance: "inst-b".into(),
                    conv: format!("soak-{}", n % 25),
                    msg_id: format!("m{n:06}"),
                    seq: n,
                    priority: if rng.chance(20) { "7".into() } else { "0".into() },
                    ts_ms: n as i64,
                    body: format!("message {n}"),
                };
                injected.insert(m.msg_id.clone(), m.clone());
                unconfirmed.push(m.msg_id.clone());
                m.msg_id
            } else {
                store::disarm();
                continue;
            };
            let m = injected[&id].clone();
            match l.store.put(&m.from, &m.conv, &m.msg_id, &m.payload_bytes()) {
                Ok(PutOutcome::Stored { .. }) | Ok(PutOutcome::Duplicate { .. }) => unconfirmed.retain(|x| *x != id),
                Ok(PutOutcome::Conflict { .. }) => panic!("conflict on an identical retry of {id}"),
                Err(StoreError::Failpoint(_)) => {
                    c.kills_store_put += 1;
                    killed = true;
                }
                Err(other) => panic!("put {id}: {other}"),
            }
        } else if op < 7 {
            // A tool call ends: the hook pulls.
            let dk = if store_kill.is_none() && rng.chance(30) {
                let k = [KillPoint::AfterClaim, KillPoint::AfterRecord, KillPoint::AfterClaimed][rng.below(3) as usize];
                deliver::arm_kill(k);
                Some(k)
            } else {
                None
            };
            tool_n += 1;
            let tool = format!("toolu_{tool_n}");
            match l.d.pull(SESSION, "post-tool-use", Some(&tool)) {
                Ok(Pulled::Frame { frame, .. }) => {
                    c.frames += 1;
                    // Mostly the harness writes the frame; sometimes the hook's
                    // output is lost (killed hook, dropped output).
                    if rng.chance(85) {
                        tool_call(&transcript, &tool, Some(&frame.text));
                    } else {
                        c.frames_dropped_by_harness += 1;
                        tool_call(&transcript, &tool, None);
                    }
                }
                Ok(Pulled::Nothing { .. }) => tool_call(&transcript, &tool, None),
                Err(err) if is_kill(&err) => {
                    // The hook got no answer: no frame, the turn goes on.
                    tool_call(&transcript, &tool, None);
                    match dk {
                        Some(k) if err.contains(deliver::KILLED) => *c.kills_deliver.entry(format!("{k:?}")).or_default() += 1,
                        _ => c.kills_store_other += 1,
                    }
                    killed = true;
                }
                Err(err) => panic!("pull: {err}"),
            }
        } else if op < 9 {
            // The finalizer looks at one attempt.
            let names = l.store.states_with_prefix("delivery-").unwrap();
            if !names.is_empty() {
                let nonce = names[rng.below(names.len() as u64) as usize]["delivery-".len()..].to_owned();
                let dk = if store_kill.is_none() && rng.chance(30) {
                    let k = [KillPoint::BeforeHandedOver, KillPoint::BeforeAbsent][rng.below(2) as usize];
                    deliver::arm_kill(k);
                    Some(k)
                } else {
                    None
                };
                c.finalize_calls += 1;
                match l.d.finalize(&nonce, false) {
                    Ok(_) => {}
                    Err(err) if is_kill(&err) => {
                        match dk {
                            Some(k) if err.contains(deliver::KILLED) => *c.kills_deliver.entry(format!("{k:?}")).or_default() += 1,
                            _ => c.kills_store_other += 1,
                        }
                        killed = true;
                    }
                    Err(err) => panic!("finalize {nonce}: {err}"),
                }
            }
        } else {
            c.plain_restarts += 1;
            killed = true; // a restart with no kill point: recovery runs anyway
        }
        store::disarm();
        deliver::disarm_kill();
        if killed {
            crash(&mut life);
        }
    }

    // Drain: no more kill points. The sender retries what it never saw stored;
    // the sidecar sweeps open attempts; every tool call now carries its frame.
    let l = life.as_ref().unwrap();
    for id in unconfirmed.clone() {
        let m = &injected[&id];
        l.store.put(&m.from, &m.conv, &m.msg_id, &m.payload_bytes()).unwrap();
    }
    for round in 0.. {
        assert!(round < 500, "drain does not converge");
        l.d.sweep(store::now_ms(), u64::MAX);
        tool_n += 1;
        let tool = format!("toolu_{tool_n}");
        match l.d.pull(SESSION, "post-tool-use", Some(&tool)).unwrap() {
            Pulled::Frame { frame, .. } => {
                c.frames += 1;
                tool_call(&transcript, &tool, Some(&frame.text));
            }
            Pulled::Nothing { .. } => {
                tool_call(&transcript, &tool, None);
                if l.d.open_attempts().is_empty() && l.d.pending().is_empty() {
                    break;
                }
            }
        }
    }

    // H-16 checks.
    let text = std::fs::read_to_string(&transcript).unwrap();
    let mut seen: BTreeMap<String, u64> = BTreeMap::new();
    for line in text.lines() {
        if !line.contains("hook_additional_context") {
            continue;
        }
        for id in injected.keys() {
            if line.contains(&format!("msg_id={id}")) {
                *seen.entry(id.clone()).or_default() += 1;
            }
        }
    }
    let mut lost = vec![];
    let mut twice = vec![];
    let mut stored_twice = vec![];
    let mut convs: Vec<String> = injected.values().map(|m| m.conv.clone()).collect();
    convs.sort();
    convs.dedup();
    for conv in &convs {
        assert!(matches!(l.store.verify_chain(conv).unwrap(), ChainStatus::Intact { .. }), "chain {conv} broken");
    }
    for (id, m) in &injected {
        let lines = l.store.chain_lines(&m.conv).unwrap();
        let count = |ev: &str| lines.iter().filter(|x| x.msg_id == *id && x.event == ev && x.peer_ts_ms.is_none()).count();
        if count("STORED") != 1 {
            stored_twice.push((id.clone(), count("STORED")));
        }
        if count("HANDED_OVER") != 1 {
            lost.push(id.clone());
        }
        if seen.get(id).copied().unwrap_or(0) > 1 {
            twice.push((id.clone(), seen[id]));
        }
    }
    let orphans_closed: usize = convs
        .iter()
        .map(|conv| {
            l.store.chain_lines(conv).unwrap().iter().filter(|x| x.event == "ABSENT" && x.reason.as_deref().is_some_and(|r| r.contains("orphan"))).count()
        })
        .sum();
    let absent: usize = convs.iter().map(|conv| l.store.chain_lines(conv).unwrap().iter().filter(|x| x.event == "ABSENT").count()).sum();
    println!(
        "SOAK-H16 {}",
        json!({
            "seed": seed, "kill_points": c.kills(), "store_write_sequence_kills": c.kills_store_put,
            "store_chain_append_kills_in_deliver": c.kills_store_other, "deliver_kills": c.kills_deliver,
            "plain_restarts": c.plain_restarts, "messages": injected.len(), "frames": c.frames,
            "frames_dropped_by_harness": c.frames_dropped_by_harness, "finalize_calls": c.finalize_calls,
            "attempts_closed_absent": absent, "orphan_claims_closed": orphans_closed,
            "lost_after_stored": lost.len(), "handed_over_twice": twice.len(), "stored_not_once": stored_twice.len(),
        })
    );
    assert!(lost.is_empty(), "lost after STORED: {lost:?}");
    assert!(twice.is_empty(), "handed over twice: {twice:?}");
    assert!(stored_twice.is_empty(), "STORED not exactly once: {stored_twice:?}");
    assert!(c.kills() >= target);
}

#[test]
#[ignore = "soak companion; run with --release -- --ignored"]
fn soak_h17_every_listed_error_ends_visibly() {
    // The 17 rows of step 3 section 5.6 are the FC rows; B12 gave each a test and
    // a control that turned it red. Hold the matrix to that.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let matrix = std::fs::read_to_string(root.join("../../docs/reports/arc011-ph1-fc-matrix.md")).unwrap();
    let tests: String = std::fs::read_dir(root.join("tests"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "rs"))
        .map(|e| std::fs::read_to_string(e.path()).unwrap())
        .collect();
    for n in 1..=17 {
        let fc = format!("| FC-{n} |");
        let rows: Vec<&str> = matrix.lines().filter(|l| l.starts_with(&fc)).collect();
        assert_eq!(rows.len(), 2, "FC-{n}: expected its rule row and its mutation-run row");
        assert!(rows.iter().any(|r| r.trim_end().ends_with("| killed |")), "FC-{n}: control not seen red");
        let run = rows.iter().find(|r| r.contains("killed")).unwrap();
        let test = run.split('`').nth(3).unwrap();
        assert!(tests.contains(&format!("fn {test}(")), "FC-{n}: test {test} does not exist");
        let patch = run.split('`').nth(1).unwrap();
        assert!(root.join(format!("tests/mutants/{patch}.patch")).is_file(), "FC-{n}: control {patch} missing");
    }
    assert!(matrix.contains("PASS: all 17 mutants turned their test red"));
}
