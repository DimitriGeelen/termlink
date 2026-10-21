//! Runs AEF's shared worker-contract conformance vectors (wc/0.3, CC-01..CC-25) against the hub
//! backend (T-3517). The vector file is AEF's and is never committed here: set
//! `TERMLINK_WC_VECTORS` to its path, or run `scripts/run-wc-vectors.sh`, which fetches it into a
//! temp dir at a pinned AEF commit.
//!
//! Judging follows AEF's local runner (`run_local.py`): `given` seeds the store, `steps` run in
//! order, `then` judges the final record and admissible histories, and `variants` branch from
//! the shared steps. Every case lands in exactly one of pass / fail / not-implemented / n/a.
//! A case is never pass unless its steps ran; a step the backend cannot express is not-implemented.

#[path = "wc_conformance/hub.rs"]
mod hub;

use hub::{CAPABILITIES, Fail, Hub};
use regex::Regex;
use serde_json::{Map, Value};

/// Capabilities whose absence may make a `subject: capability` case n/a (vocabulary, §6.6 N2).
const NA_CAPABILITIES: [&str; 4] = ["answer_channel", "registry_epoch", "caller_chosen_ids", "real_worker"];

/// `Yes` / `No` are definite; `Unknown` means the backend does not expose what the expectation names.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tri {
    Yes,
    No,
    Unknown,
}

fn match_value(actual: &Value, expected: &Value) -> bool {
    if let Some(o) = expected.as_object() {
        if o.len() == 1 && o.contains_key("present") {
            return !actual.is_null() == o["present"].as_bool().unwrap_or(false);
        }
        if let Some(alts) = o.get("any_of").and_then(Value::as_array) {
            return alts.iter().any(|e| match_value(actual, e));
        }
        let Some(a) = actual.as_object() else { return false };
        return o.iter().all(|(k, v)| match_value(a.get(k).unwrap_or(&Value::Null), v));
    }
    actual == expected
}

fn tri_all(parts: impl Iterator<Item = Tri>) -> Tri {
    let mut unknown = false;
    for p in parts {
        match p {
            Tri::Yes => return Tri::Yes,
            Tri::Unknown => unknown = true,
            Tri::No => {}
        }
    }
    if unknown { Tri::Unknown } else { Tri::No }
}

/// AEF's `match_expect`, tri-state: a key the result does not carry is Unknown, not No.
fn match_expect(res: &Map<String, Value>, exp: &Value) -> Tri {
    if let Some(alts) = exp.get("any_of").and_then(Value::as_array) {
        return tri_all(alts.iter().map(|e| match_expect(res, e)));
    }
    for key in ["accepted", "fence_reject", "error", "refused", "response", "no_transition", "ok"] {
        if let Some(want) = exp.get(key) {
            if res.get(key) != Some(want) {
                return Tri::No;
            }
        }
    }
    if let Some(n) = exp.get("notices") {
        match res.get("notices") {
            None => return Tri::Unknown,
            Some(a) if a != n => return Tri::No,
            Some(_) => {}
        }
    }
    if let Some(values) = exp.get("values").and_then(Value::as_object) {
        let rv = res.get("values").and_then(Value::as_object);
        for (k, want) in values {
            if let Some(base) = k.strip_suffix("_matches") {
                let Some(actual) = rv.and_then(|v| v.get(base)) else { return Tri::Unknown };
                let Some(s) = actual.as_str() else { return Tri::No };
                let pattern = want.as_str().unwrap_or_default();
                let anchored = Regex::new(&format!("^(?:{pattern})$")).expect("vector regex");
                if !anchored.is_match(s) {
                    return Tri::No;
                }
            } else {
                let Some(actual) = rv.and_then(|v| v.get(k)) else { return Tri::Unknown };
                if !match_value(actual, want) {
                    return Tri::No;
                }
            }
        }
    }
    Tri::Yes
}

fn check_step(hub: &mut Hub, step: &Value) -> Result<(), Fail> {
    let res = hub.apply(step)?;
    let exp = &step["expect"];
    let op = step["op"].as_str().unwrap_or_default();
    if op == "concurrent" {
        let results = res.get("results").and_then(Value::as_array).cloned().unwrap_or_default();
        let want = step["args"]["count"].as_u64().unwrap_or(0) as usize;
        if results.len() != want {
            return Err(Fail::Mismatch(format!("concurrent: {} results for {want} writers", results.len())));
        }
        let one = &exp["exactly_one"];
        let rest = json_any(&exp["rest_any_of"]);
        let mut winners = 0;
        for r in &results {
            let m = r.as_object().cloned().unwrap_or_default();
            match match_expect(&m, one) {
                Tri::Yes => winners += 1,
                Tri::Unknown => return Err(Fail::Missing("concurrent: winner check needs a surface the backend lacks".into())),
                Tri::No => {
                    if match_expect(&m, &rest) != Tri::Yes {
                        return Err(Fail::Mismatch(format!("concurrent: loser result {r} not admissible")));
                    }
                }
            }
        }
        if winners != 1 {
            return Err(Fail::Mismatch(format!("concurrent: {winners} writers matched exactly_one")));
        }
        return Ok(());
    }
    match match_expect(&res, exp) {
        Tri::Yes => Ok(()),
        Tri::No => Err(Fail::Mismatch(format!("step {op}: expected {exp}, got {}", Value::Object(res)))),
        Tri::Unknown => Err(Fail::Missing(format!("step {op}: expectation names a surface the backend does not expose: {exp}"))),
    }
}

fn json_any(list: &Value) -> Value {
    serde_json::json!({ "any_of": list.clone() })
}

fn check_then(before: &Value, after: &Value, then: &Value) -> Result<(), Fail> {
    let attempts = &after["attempts"];
    let before_attempts = &before["attempts"];
    if let Some(finals) = then.get("final").and_then(Value::as_object) {
        for (key, want) in finals {
            let Some(got) = attempts.get(key) else {
                return Err(Fail::Mismatch(format!("final: attempt {key} missing")));
            };
            for (field, value) in want.as_object().cloned().unwrap_or_default() {
                let Some(actual) = got.get(&field) else {
                    return Err(Fail::Missing(format!("final {key}.{field}: the backend does not expose it")));
                };
                if !match_value(actual, &value) {
                    return Err(Fail::Mismatch(format!("final {key}.{field}: expected {value}, got {actual}")));
                }
            }
        }
    }
    if let Some(adm) = then.get("admissible_histories").and_then(Value::as_object) {
        for (key, seqs) in adm {
            let hist = attempts.get(key).and_then(|a| a.get("history")).cloned().unwrap_or(Value::Null);
            let ok = seqs.as_array().is_some_and(|s| s.contains(&hist));
            if !ok {
                return Err(Fail::Mismatch(format!("history {key}: {hist} not in admissible {seqs}")));
            }
        }
    }
    if let Some(keys) = then.get("history_unchanged").and_then(Value::as_array) {
        for key in keys.iter().filter_map(Value::as_str) {
            let b = before_attempts.get(key).and_then(|a| a.get("history"));
            let a = attempts.get(key).and_then(|a| a.get("history"));
            if b != a {
                return Err(Fail::Mismatch(format!("history of {key} changed")));
            }
        }
    }
    if let Some(keys) = then.get("fields_unchanged").and_then(Value::as_array) {
        for key in keys.iter().filter_map(Value::as_str) {
            if before_attempts.get(key) != attempts.get(key) {
                return Err(Fail::Mismatch(format!("fields of {key} changed")));
            }
        }
    }
    if then.get("record_unchanged").and_then(Value::as_bool) == Some(true) && before_attempts != attempts {
        return Err(Fail::Mismatch("record changed".into()));
    }
    const HANDLED: [&str; 7] = ["final", "admissible_histories", "history_unchanged", "fields_unchanged", "record_unchanged", "judged_by", "variants"];
    for (key, value) in then.as_object().cloned().unwrap_or_default() {
        if HANDLED.contains(&key.as_str()) {
            continue;
        }
        let Some(actual) = after.get(&key) else {
            return Err(Fail::Missing(format!("then.{key}: the backend does not expose it")));
        };
        if !match_value(actual, &value) {
            return Err(Fail::Mismatch(format!("then.{key}: expected {value}, got {actual}")));
        }
    }
    Ok(())
}

/// One branch of a case: its own given (merged over the case's), steps and then.
struct Run {
    name: Option<String>,
    requires: Vec<String>,
    given: Value,
    steps: Vec<Value>,
    then: Value,
}

fn runs_of(vec: &Value) -> Vec<Run> {
    let steps_of = |v: &Value| v.as_array().cloned().unwrap_or_default();
    let Some(variants) = vec.get("variants").and_then(Value::as_array) else {
        return vec![Run {
            name: None,
            requires: vec![],
            given: vec["given"].clone(),
            steps: steps_of(&vec["steps"]),
            then: vec["then"].clone(),
        }];
    };
    variants
        .iter()
        .map(|var| {
            let mut given = vec["given"].as_object().cloned().unwrap_or_default();
            for (k, v) in var.get("given").and_then(Value::as_object).cloned().unwrap_or_default() {
                given.insert(k, v);
            }
            let mut steps = steps_of(&vec["steps"]);
            steps.extend(steps_of(&var["steps"]));
            Run {
                name: var["name"].as_str().map(str::to_string),
                requires: var["requires"].as_array().map_or(vec![], |r| r.iter().filter_map(Value::as_str).map(str::to_string).collect()),
                given: Value::Object(given),
                steps,
                then: var["then"].clone(),
            }
        })
        .collect()
}

#[derive(Debug)]
enum Verdict {
    Pass,
    Fail(String),
    NotImpl(String),
    Na(String),
}

fn run_variant(run: &Run) -> Verdict {
    let mut hub = match Hub::reset(&run.given) {
        Ok(h) => h,
        Err(Fail::Mismatch(m)) => return Verdict::Fail(m),
        Err(Fail::Missing(m)) => return Verdict::NotImpl(m),
    };
    let before = match hub.snapshot() {
        Ok(s) => s,
        Err(Fail::Mismatch(m)) => return Verdict::Fail(m),
        Err(Fail::Missing(m)) => return Verdict::NotImpl(m),
    };
    for step in &run.steps {
        match check_step(&mut hub, step) {
            Ok(()) => {}
            Err(Fail::Mismatch(m)) => return Verdict::Fail(m),
            Err(Fail::Missing(m)) => return Verdict::NotImpl(m),
        }
    }
    let after = match hub.snapshot() {
        Ok(s) => s,
        Err(Fail::Mismatch(m)) => return Verdict::Fail(m),
        Err(Fail::Missing(m)) => return Verdict::NotImpl(m),
    };
    match check_then(&before, &after, &run.then) {
        Ok(()) => Verdict::Pass,
        Err(Fail::Mismatch(m)) => Verdict::Fail(m),
        Err(Fail::Missing(m)) => Verdict::NotImpl(m),
    }
}

/// Status of one case, as AEF's `run_vector` computes it.
fn run_case(vec: &Value) -> (String, String) {
    let app = &vec["applicability"];
    let requires: Vec<&str> = app["requires"].as_array().map_or(vec![], |r| r.iter().filter_map(Value::as_str).collect());
    let n_a_allowed = app["n_a_allowed"].as_bool().unwrap_or(false);
    let lacking: Vec<&str> = requires.iter().copied().filter(|c| !CAPABILITIES.contains(c)).collect();
    if !lacking.is_empty() {
        if n_a_allowed && lacking.iter().all(|c| NA_CAPABILITIES.contains(c)) {
            return ("n/a".into(), format!("capability not offered: {}", lacking.join(", ")));
        }
        return ("fail".into(), format!("N2: lacks {} on a case that may not be n/a", lacking.join(", ")));
    }
    let mut statuses: Vec<(String, Verdict)> = Vec::new();
    for run in runs_of(vec) {
        let label = run.name.clone().unwrap_or_else(|| "main".into());
        if run.requires.iter().any(|c| !CAPABILITIES.contains(&c.as_str())) {
            statuses.push((label, Verdict::Na("variant requires a capability not offered".into())));
            continue;
        }
        statuses.push((label, run_variant(&run)));
    }
    let any = |f: fn(&Verdict) -> bool| statuses.iter().any(|(_, v)| f(v));
    let all_na = statuses.iter().all(|(_, v)| matches!(v, Verdict::Na(_)));
    let detail = |f: fn(&Verdict) -> Option<&String>| {
        statuses
            .iter()
            .filter_map(|(label, v)| f(v).map(|m| format!("[{label}] {m}")))
            .collect::<Vec<_>>()
            .join(" | ")
    };
    if any(|v| matches!(v, Verdict::Fail(_))) {
        return ("fail".into(), detail(|v| if let Verdict::Fail(m) = v { Some(m) } else { None }));
    }
    if any(|v| matches!(v, Verdict::NotImpl(_))) {
        return ("not-implemented".into(), detail(|v| if let Verdict::NotImpl(m) = v { Some(m) } else { None }));
    }
    if all_na {
        return if n_a_allowed {
            ("n/a".into(), detail(|v| if let Verdict::Na(m) = v { Some(m) } else { None }))
        } else {
            ("fail".into(), "N2: no variant applies to a transition-subject case".into())
        };
    }
    ("pass".into(), String::new())
}

#[test]
fn wc_vectors_against_hub_backend() {
    let Ok(path) = std::env::var("TERMLINK_WC_VECTORS") else {
        eprintln!(
            "SKIP: TERMLINK_WC_VECTORS is unset, so no vectors were run. \
             Run scripts/run-wc-vectors.sh (it fetches AEF's vectors.yaml into a temp dir)."
        );
        return;
    };
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let yaml: serde_yaml::Value = serde_yaml::from_str(&text).unwrap_or_else(|e| panic!("parse {path}: {e}"));
    let doc: Value = serde_json::to_value(yaml).expect("vectors as JSON");
    let vectors = doc["vectors"].as_array().cloned().unwrap_or_default();
    assert_eq!(vectors.len(), 25, "fail-closed: expected the 25 wc/0.3 cases, found {}", vectors.len());

    // TERMLINK_WC_CASE=CC-NN runs one case (reproducing a finding); the summary then covers that case only.
    let only = std::env::var("TERMLINK_WC_CASE").ok();
    let (mut pass, mut fail, mut ni, mut na) = (0, 0, 0, 0);
    println!("backend: hub (termlink-bus worker_contract, S1-S3), vectors: {path}");
    for vec in &vectors {
        let id = vec["id"].as_str().unwrap_or("?");
        if only.as_deref().is_some_and(|o| o != id) {
            continue;
        }
        let title = vec["title"].as_str().unwrap_or("");
        let (status, detail) = run_case(vec);
        match status.as_str() {
            "pass" => pass += 1,
            "fail" => fail += 1,
            "not-implemented" => ni += 1,
            _ => na += 1,
        }
        println!("  {id:6} {status:16} {title}");
        if !detail.is_empty() {
            println!("         {detail}");
        }
    }
    let ran = pass + fail + ni + na;
    println!("summary: {ran} cases: pass={pass} fail={fail} not-implemented={ni} n/a={na}");
    assert_eq!(fail, 0, "{fail} case(s) failed against the hub backend; see the table above");
}
