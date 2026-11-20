#!/usr/bin/env python3
"""T-3708 (arc-011 P-24 slice a, B-14, AL-29) mutants (FP 3.1.b): apply each, run the unit's tests, restore. Every
mutant must be killed. Design `docs/reports/T-3708-design.md` section 5 (slice-a subset) plus the build's own rules.

Run from the repo root:  python3 crates/termlink-session/tests/mutants/run-t3708.py [mutant-id ...]
Writes a table to stdout; exit 0 only if every mutant is killed and the unmutated suite passes.
"""
import os
import tempfile
import subprocess
import sys

F = "crates/termlink-sidecar/src/fields.rs"
H = "crates/termlink-sidecar/src/host.rs"
S = "crates/termlink-sidecar/src/store.rs"
P = "crates/termlink-sidecar/src/pull.rs"
A = "crates/termlink-sidecar/src/api.rs"
R = "crates/termlink-sidecar/src/revocation.rs"
PO = "crates/termlink-sidecar/src/policy.rs"
HR = "crates/termlink-hub/src/router.rs"

# (id, what, file, old, new)
MUTANTS = [
    ("m1-cached-without-stamp", "E-19/E-55 control: a field past its bound is served as fresh", F,
     "        Some(s) if now_mono.saturating_sub(s.mono_ms) <= bound_ms => {", "        Some(s) if true => {"),
    ("m2-heartbeat-is-surfacing", "E-56 control (arch 6.7.5): pending mail without a hand-over still reads CLEAR", F,
     "    if !surfaced {", "    if false && !surfaced {"),
    ("m3-idle-age-threshold", "E-57 control: an idle agent turns bad by age alone", F,
     "        \"bad\": !overdue.is_empty(),", "        \"bad\": !overdue.is_empty() || last.is_some_and(|t| age(t) > 24 * 3_600_000),"),
    ("m4-per-agent-yes", "E-58 control (r1 MAJOR 3): per_agent_accounts claims yes", H,
     "                (\"no (sidecars run under separate accounts; the agent sessions' accounts are not checked)\".to_string(), Some(false))",
     "                (\"yes\".to_string(), Some(false))"),
    ("m4b-root-host-separated", "E-58 control: a root host reads as separated", H,
     "            } else if uids.contains(&0) || !pairwise_disjoint(&by_agent) {", "            } else if !pairwise_disjoint(&by_agent) {"),
    ("m4c-shared-uid-separated", "r3 MINOR 13: a uid shared between agents reads as separate accounts", H,
     "            } else if uids.contains(&0) || !pairwise_disjoint(&by_agent) {", "            } else if uids.contains(&0) || uids.len() < by_agent.len() {"),
    ("m5-wall-time-deadlines", "E-59 control: deadline ages follow the wall clock (no jump correction)", F,
     "        let correction = j.total - j.at_write.get(&stamp_wall).copied().unwrap_or(0);",
     "        let correction = 0 * (j.total - j.at_write.get(&stamp_wall).copied().unwrap_or(0));"),
    ("m5b-written-total-ignored", "r3 MINOR 15/r2 MAJOR 1: every stamp is corrected by every jump, also those before it", F,
     "        let correction = j.total - j.at_write.get(&stamp_wall).copied().unwrap_or(0);",
     "        let correction = j.total;"),
    ("m5c-deliver-wall-deadline", "E-59: the receiver's hand-over deadline uses plain wall age", "crates/termlink-sidecar/src/deliver.rs",
     "            let age = crate::fields::age_ms(stored_at, now_ms).min(crate::fields::age_ms(ready_at, now_ms));",
     "            let age = now_ms.saturating_sub(stored_at.max(ready_at));"),
    ("m5d-deliver-orders-wall-stamps", "r2 MAJOR 2: the later of storing and readiness is picked by wall value", "crates/termlink-sidecar/src/deliver.rs",
     "            let age = crate::fields::age_ms(stored_at, now_ms).min(crate::fields::age_ms(ready_at, now_ms));",
     "            let age = crate::fields::age_ms(stored_at.max(ready_at), now_ms);"),
    ("m5e-stamps-not-registered", "r2 MAJOR 1: chain stamps are not registered when written", S,
     "            ts_ms: Some(crate::fields::stamp_now()),", "            ts_ms: Some(now_ms()),"),
    ("m6-unknown-counts-fresh", "FCW-121: an unknown field does not hold CLEAR", F,
     "        if f[\"state\"] != \"fresh\" {", "        if false {"),
    ("m7-failed-pass-stamps-hub", "FCW-120: a pass that could not reach the hub stamps `hub` fresh", P,
     "                self.revocation_outage(&format!(\"hub unreachable: {e}\"));",
     "                crate::fields::touch(crate::fields::PULL);\n                self.revocation_outage(&format!(\"hub unreachable: {e}\"));"),
    ("m8-restart-is-fresh", "06a 2.13.f: a field never measured since start reads fresh", F,
     "        None => json!({\"state\": \"unknown\", \"shown\": \"unknown (not measured since start)\", \"last_at_ms\": null, \"bound_ms\": bound_ms}),",
     "        None => json!({\"state\": \"fresh\", \"value\": value, \"at_ms\": 0, \"bound_ms\": bound_ms}),"),
    ("m9-switch-default-off", "design 3.0: the new keys are hidden by default", F,
     "        Err(_) => true,", "        Err(_) => false,"),
    ("m10-sent-handover-is-ours", "a recipient's hand-over of a message we SENT counts as our surfacing", F,
     "        .filter(|r| r.stage == \"HANDED_OVER\" && !rows.iter().any(|s| s.msg_id == r.msg_id && s.stage == \"SENT\"))",
     "        .filter(|r| r.stage == \"HANDED_OVER\")"),
    ("m11-v8-from-ready-only", "V-8: a message stored after READY is overdue counted from READY", F,
     "        let by_ready = ready_since.map(|r| age(*stored).min(age(r)))", "        let by_ready = ready_since.map(|r| age(r))"),
    ("m12-hook-time-overwritten", "V-8: a later hook call resets the first missed time", F,
     "        m.entry(id.clone()).or_insert_with(|| {", "        m.insert(id.clone(), {"),
    ("m27-pending-window-one-hour", "r1 MAJOR 2: mail pending for more than an hour drops out of CLEAR", F,
     "    let earliest = pending_now.iter().map(|(_, t)| age(*t)).chain(handed_in_window).max();",
     "    let earliest = pending_now.iter().map(|(_, t)| age(*t)).filter(|a| *a <= CLEAR_WINDOW_MS).chain(handed_in_window).max();"),
    ("m36-clear-wall-order", "r3 MINOR 7: CLEAR orders hand-overs and mail by raw wall value", F,
     "    let surfaced = earliest.is_none_or(|e| own_handovers(rows).any(|t| age(t) <= e));",
     "    let newest_pending_wall = pending_now.iter().map(|(_, t)| *t).min();\n    let surfaced = earliest.is_none_or(|e| own_handovers(rows).any(|t| age(t) <= e || newest_pending_wall.is_some_and(|m| t >= m)));"),
    ("m37-held-mail-pending", "r3 finding 2: mail held for the operator counts as pending surfacing", F,
     "        .filter(|id| !(rows.iter().any(|r| r.msg_id == *id && r.stage == \"HELD\") && !rows.iter().any(|r| r.msg_id == *id && r.stage == \"ADMITTED\")))",
     "        .filter(|id| !id.is_empty())"),
    ("m28-unknown-revocation-ok", "r1 MAJOR 5: an unknown revocation line reads fine", F,
     "    } else {\n        \"bad\"\n    }", "    } else {\n        \"ok\"\n    }"),
    ("m29-heartbeat-age-ignored", "r1 MAJOR 5: an old heartbeat file with fresh component ages reads fine", F,
     "                    if v[\"heartbeat_age_ms\"].as_u64().unwrap_or(u64::MAX) > COMPONENT_BAD_MS {",
     "                    if false {"),
    ("m30-consumers-stamped-on-error", "r1 MAJOR 4: a consumers check the hub did not answer stamps the field", "crates/termlink-sidecar/src/consumers.rs",
     "    !v.line.starts_with(\"unknown\")", "    let _ = v;\n    true"),
    ("m31-revocation-off-unstamped", "FCW-120: a switched-off revocation pass leaves the field unstamped (reads unknown, never CLEAR)", R,
     "            // T-3708 (FCW-120): \"off (PH-1 trust)\" is this pass's evaluation too; unstamped it would read unknown\n            crate::fields::touch(crate::fields::REVOCATION);\n",
     ""),
    ("m32-peer-time-in-rows", "r1 MAJOR 6: V-8 and CLEAR read the peer's clock", F,
     "                ts_ms: r.get(\"ts_ms\").and_then(Value::as_u64).unwrap_or(0),",
     "                ts_ms: r.get(\"peer_ts_ms\").and_then(Value::as_u64).or_else(|| r.get(\"ts_ms\").and_then(Value::as_u64)).unwrap_or(0),"),
    ("m33-hook-miss-in-memory", "r1 MAJOR 7: the hook-miss record is not kept", F,
     "    if m != old {", "    if false && m != old {"),
    ("m13-e127-ignores-narrowing", "E-127 reader: an unverified list past PN-13 fires even while narrowing", R,
     "        if now.saturating_sub(t) > PN13.as_millis() as u64 && !narrowing_now {",
     "        if now.saturating_sub(t) > PN13.as_millis() as u64 {"),
    ("m14-e127-ignores-held-time", "E-127 reader: a message stored before the key was held is flagged", R,
     " && age(*at) < age(**t));", ");"),
    ("m39-e127-wall-order", "r3 MINOR 4: E-127 compares raw wall stamps", R,
     " && age(*at) < age(**t));", " && *at > **t);"),
    ("m15-skew-last-not-median", "V-5: one slow round trip flags clock skew", H,
     "        let offset = o[o.len() / 2];", "        let offset = self.samples.back().map(|s| s.0).unwrap_or(o[0]);"),
    ("m16-skew-threshold-30s", "V-5: clock skew flagged only past 30 s", H,
     "        \"clock\": if a > SKEW_FLAG_MS {", "        \"clock\": if a > SKEW_UNUSABLE_MS {"),
    ("m17-ntp-guess", "V-5: with no tools the NTP state is claimed synchronised", H,
     "    }\n    \"unchecked\"\n}", "    }\n    \"synchronised (timedatectl)\"\n}"),
    ("m18-stale-revocation-ok", "AL-29: a stale revocation list is not a bad field", F,
     "    if line.starts_with(\"verified seq \") || line == \"off (PH-1 trust)\" {",
     "    if line.starts_with(\"verified seq \") || line == \"off (PH-1 trust)\" || line.starts_with(\"stale since \") {"),
    ("m19-second-consumer-ok", "AL-29: more than one consumer is not a bad field", F,
     "                } else if s != \"one (proven)\" {", "                } else if false {"),
    ("m20-downgrade-reason-never-stale", "carried T-3703 CC N4: a stale-gate downgrade names the allow-list", PO,
     "    if allowed(trust) && !stale_gate(true, trust, narrowing) {", "    if false {"),
    ("m21-no-skew-from-version", "V-5: an agent with an empty inbox is never measured for skew", P,
     "        crate::host::record_round_trip(before, after, v.get(\"hub_now_ms\").and_then(Value::as_u64));",
     "        let _ = (before, after);"),
    ("m22-hub-version-no-time", "V-5: the hub's version reply carries no clock", HR,
     "            \"hub_now_ms\": crate::channel::hub_now_ms(),\n", ""),
    ("m23-latency-not-logged", "E-55/E-134: status calls leave no latency line", A,
     "        writeln!(f, \"{line}\")", "        let _ = (&mut f, &line);\n        Ok(())"),
    ("m24-pass-not-stamped", "FCW-120: a successful pass does not stamp `hub`", P,
     "                crate::fields::touch(crate::fields::PULL);\n                self.observe_impersonation();",
     "                self.observe_impersonation();"),
    ("m25-hub-predates-is-fine", "no verified list (`hub predates`) does not hold CLEAR", F,
     "    } else if line.starts_with(\"unverified since \") || line == \"hub predates revocation list\" {",
     "    } else if line == \"hub predates revocation list\" {\n        \"ok\"\n    } else if line.starts_with(\"unverified since \") {"),
    ("m26-invariant-ignored", "a broken revocation invariant does not hold CLEAR", F,
     "            \"revocation_invariants\" if v[\"broken\"].as_array().is_some_and(|a| !a.is_empty()) => {",
     "            \"revocation_invariants\" if false => {"),
    ("m10b-status-calls-the-hub", "design mutant 10 (r2 MAJOR 5): status makes a hub call (blocks on a silent hub)", A,
     "                    let all_rows = if obj.contains_key(\"conv\") || obj.contains_key(\"msg_id\") {",
     "                    if let Some(s) = sender {\n                        let _ = s.peek_inbox(0, 1);\n                    }\n                    let all_rows = if obj.contains_key(\"conv\") || obj.contains_key(\"msg_id\") {"),
    ("m38-filtered-rows", "r3 finding 3: CLEAR over the rows a --conv request filtered", A,
     "                    let all_rows = if obj.contains_key(\"conv\") || obj.contains_key(\"msg_id\") {",
     "                    let all_rows = if false {"),
    ("m34-unanchored-pending-ignored", "r2 MAJOR 4: pending mail with nothing to measure from does not hold CLEAR", F,
     "            unanchored.push(json!(id));", "            if false {\n                unanchored.push(json!(id));\n            }"),
    # m35 (a failed held-since write is not reported) is not in the list: the tests run as root, so a store
    # write cannot be made to fail here; the branch is covered by review only (r2 MINOR 8 disposition).
]

CMDS = [
    ["cargo", "test", "-p", "termlink-sidecar", "--lib", "--", "fields::", "host::", "consumers::"],
    ["cargo", "test", "-p", "termlink-sidecar", "--test", "policy_tests", "t3708"],
    ["cargo", "test", "-p", "termlink-sidecar", "--test", "t3708_status"],
    ["cargo", "test", "-p", "termlink-sidecar", "--test", "t3708_clock"],
    ["cargo", "test", "-p", "termlink-sidecar", "--test", "t3708_revoff"],
    ["cargo", "test", "-p", "termlink-sidecar", "--test", "deadline_tests"],
    ["cargo", "build", "-p", "termlink"],
    ["cargo", "test", "-p", "termlink-sidecar", "--test", "t3708_live"],
]

SCRATCH_RT = tempfile.mkdtemp(prefix="t3708-mutants-rt.")


def env():
    e = dict(os.environ)
    e.pop("PROJECT_ROOT", None)
    # T-3822: never let a mutant write into a live hub's runtime dir
    e["TERMLINK_RUNTIME_DIR"] = SCRATCH_RT
    return e


def run():
    out = ""
    for c in CMDS:
        r = subprocess.run(c, env=env(), capture_output=True, text=True)
        out += r.stdout + r.stderr
        if r.returncode != 0:
            return r.returncode, out
    return 0, out


def main():
    rc, out = run()
    if rc != 0:
        print("unmutated suite FAILS; no mutant run\n" + out[-2000:])
        return 2
    rows, survivors = [], 0
    for mid, what, path, old, new in MUTANTS:
        if sys.argv[1:] and mid not in sys.argv[1:]:
            continue
        orig = open(path).read()
        if orig.count(old) != 1:
            rows.append((mid, what, "NOT APPLIED (pattern count %d)" % orig.count(old)))
            survivors += 1
            continue
        open(path, "w").write(orig.replace(old, new))
        try:
            rc, out = run()
        finally:
            open(path, "w").write(orig)
        if rc == 0:
            verdict = "SURVIVED"
            survivors += 1
        elif "error[E" in out or "could not compile" in out:
            verdict = "COMPILE ERROR (not a kill)"
            survivors += 1
        else:
            verdict = "killed"
        rows.append((mid, what, verdict))
    # A mutant killed after `cargo build -p termlink` leaves its mutated binary on disk (T-3700 lesson): rebuild clean.
    if subprocess.run(["cargo", "build", "-p", "termlink"], env=env(), capture_output=True).returncode != 0:
        print("WARNING: the clean termlink binary could not be rebuilt; run cargo build -p termlink")
    print("| mutant | what | verdict |\n|---|---|---|")
    for r in rows:
        print("| %s | %s | %s |" % r)
    print("\n%d/%d killed" % (len(rows) - survivors, len(rows)))
    return 0 if survivors == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
