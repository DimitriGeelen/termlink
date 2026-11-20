#!/usr/bin/env python3
"""T-3700 (arc-011 P-12, B-2, AL-30) mutants (FP 3.1.b): apply each, run the unit's tests, restore. Every mutant
must be killed. Includes the E-66.5 / E-66.6 / E-169.5 controls.

Run from the repo root:  python3 crates/termlink-session/tests/mutants/run-t3700.py [mutant-id ...]
Writes a table to stdout; exit 0 only if every mutant is killed and the unmutated suite passes.
"""
import os
import tempfile
import subprocess
import sys

IC = "crates/termlink-hub/src/inbox_consumers.rs"
CH = "crates/termlink-hub/src/channel.rs"
SC = "crates/termlink-sidecar/src/consumers.rs"
LV = "crates/termlink-sidecar/src/liveness.rs"

# (id, what, file, old, new)
MUTANTS = [
    ("m1-skip-signature-check", "a read proof is accepted without verifying its signature", IC,
     "    if vk.verify(&bytes, &sig).is_err() {", "    if false && vk.verify(&bytes, &sig).is_err() {"),
    ("m2-ignore-ts-window", "a proof with a skewed timestamp still proves", IC,
     "    if now_ms.abs_diff(p.ts_ms) > PROOF_WINDOW_MS as u64 {", "    if false {"),
    ("m3-bind-by-name", "a bind admits any reader naming the bound agent (ruling E broken)", IC,
     "    reader.strength == Strength::Proven && reader.agent == b.agent && reader.key_fp.as_deref() == Some(b.key_fp.as_str())",
     "    reader.agent == b.agent"),
    ("m4-hub-unreadable-is-empty", "E-169.5 control: an unreadable consumer list answers as a list", IC,
     "        if let Some(e) = st.load_error.as_ref().or(st.persist_error.as_ref()) {\n            return Err(e.clone());\n        }",
     "        let _ = (&st.load_error, &st.persist_error);"),
    ("m5-sidecar-malformed-is-one", "a malformed hub answer is shown as one consumer", SC,
     "        return unknown(\"the hub's answer holds no consumer list\".into());",
     "        return Verdict { line: \"one (proven)\".into(), others: vec![], bound: None };"),
    ("m6-acks-not-recorded", "E-66.5-style control: an ack (receipt) does not record its poster", CH,
     "    if msg_type == \"receipt\"\n        && crate::inbox_consumers::applies(&topic)",
     "    if false && msg_type == \"receipt\"\n        && crate::inbox_consumers::applies(&topic)"),
    ("m7-reads-not-recorded", "E-66.6 control: only acks and claims record; a read-only consumer is missed", CH,
     "    let reader = match consumer_gate(&id, params, \"channel.subscribe\", \"cursor\", crate::inbox_consumers::Action::Read, peer_addr) {",
     "    let reader = match Ok::<Option<crate::inbox_consumers::Reader>, RpcResponse>(None) {"),
    ("m8-no-eviction-row", "the 17th reader evicts silently", IC,
     "\"event\": \"inbox-consumer-evicted\", \"topic\": topic, \"agent\": gone.agent,",
     "\"event\": \"x\", \"topic\": topic, \"agent\": gone.agent,"),
    ("m9-hub-instance-ignored", "V-12: the hub keys consumers without the instance (a second copy merges)", IC,
     ".find(|e| e.agent == reader.agent && e.instance == reader.instance && e.strength == reader.strength && e.key_fp == reader.key_fp)",
     ".find(|e| e.agent == reader.agent && e.strength == reader.strength && e.key_fp == reader.key_fp)"),
    ("m10-sidecar-instance-ignored", "V-12: the sidecar treats its own key at any instance as itself", SC,
     "            && (seen.instance == me_instance || seen.instance == ACK_INSTANCE);", "            ;"),
    ("m11-card-mismatch-proves", "a proof signed by a key not on the named card proves", IC,
     "        Ok(Some(k)) if &k == vk.as_bytes() => {}", "        Ok(Some(_)) => {}"),
    ("m12-revoked-proves", "a revoked key still proves", IC,
     "        Ok(keys) if keys.contains(&fp) => {", "        Ok(keys) if keys.contains(&fp) && false => {"),
    ("m13-bind-without-row", "a bind takes effect when its admin row could not be written", IC,
     "    log(serde_json::json!({\"event\": \"inbox-bind\", \"topic\": topic, \"agent\": agent, \"key_fp\": b.key_fp}))?;",
     "    let _ = log(serde_json::json!({\"event\": \"inbox-bind\", \"topic\": topic, \"agent\": agent, \"key_fp\": b.key_fp}));"),
    ("m14-attention-every-check", "a second consumer raises an entry on every check", SC,
     "        if let Err(e) = store.attention_coalesced(ATTENTION_KIND, &key, &detail) {",
     "        let _ = &key;\n        if let Err(e) = store.attention(ATTENTION_KIND, &detail) {"),
    ("m15-liveness-reads-content", "liveness reads the main's inbox content instead of first_at", LV,
     "        Ok(n) => match hub.call(method::CHANNEL_FIRST_AT, json!({\"topic\": inbox, \"from_offset\": n.drained_up_to})) {",
     "        Ok(n) => match hub.call(method::CHANNEL_FIRST_AT, json!({\"topic\": \"-\", \"from_offset\": n.drained_up_to})) {"),
    # round 1 additions
    ("m16-claim-gate-removed", "a claim on an inbox is neither recorded nor refused after a bind", CH,
     "    if let Err(refused) = consumer_gate(&id, params, \"channel.claim\", \"offset\", crate::inbox_consumers::Action::Claim, peer_addr) {",
     "    if let Err(refused) = Ok::<Option<crate::inbox_consumers::Reader>, RpcResponse>(None) {"),
    ("m17-bindings-fail-open", "unreadable bindings are treated as none (every bound inbox opens)", IC,
     "    // the bind is read under the same lock the bind verb writes under\n    let bindings = load_bindings(dir).map_err(bindings_refusal)?;",
     "    let bindings = load_bindings(dir).unwrap_or_default();"),
    ("m18-strength-merge", "r1 CC 1: a claimed reader merges into the proven entry of the same name and instance", IC,
     ".find(|e| e.agent == reader.agent && e.instance == reader.instance && e.strength == reader.strength && e.key_fp == reader.key_fp)",
     ".find(|e| e.agent == reader.agent && e.instance == reader.instance)"),
    ("m19-self-by-name", "r1 CC 1: the sidecar takes any entry under its name and instance for itself", SC,
     "            && seen.strength == \"proven\"\n            && seen.key_fp == me_fp\n", ""),
    ("m20-no-long-poll-recheck", "r1 CC 6: a long-poll that waited across a bind still delivers", CH,
     "    if let (Some(r), Some(topic)) = (reader.as_ref(), param_str(params, \"topic\")) {",
     "    if let (Some(r), Some(topic)) = (reader.as_ref().filter(|_| false), param_str(params, \"topic\")) {"),
    ("m21-liveness-fallback-any-error", "r1 CC 9 / r2 N3: liveness falls back to a content read on any coded first_at error", LV,
     "            Err((Some(code), m)) if code == error_code::SESSION_NOT_FOUND && m.contains(\"Missing 'target'\") => match oldest_by_subscribe(hub, inbox, n.drained_up_to) {",
     "            Err((Some(_), m)) if !m.contains(\"channel.first_at\") => match oldest_by_subscribe(hub, inbox, n.drained_up_to) {"),
    ("m22-ack-replay-proves", "r1 CC 7: a replayed old receipt proves", IC,
     "        Ok(_) if now_ms.abs_diff(post_ts_ms) > PROOF_WINDOW_MS as u64 => claimed(&agent, ACK_INSTANCE, \"stale ack\"),\n", ""),
    ("m23-corrupt-overwritten", "r1 CC 4: a corrupt table is not moved aside (saved over)", IC,
     "    match std::fs::rename(&path, &aside) {", "    match Ok::<(), std::io::Error>(()) {"),
    ("m24-stale-status-shown", "r1 CC 2: an old consumers line is shown as current", SC,
     "        (Some(v), Some(t)) if now_ms.saturating_sub(t) <= STALE_AFTER_MS => v,", "        (Some(v), Some(_)) => v,"),
    ("m25-unknown-topics-recorded", "r1 CC 3: reads of inbox topics that do not exist are recorded", CH,
     "    if !ic::applies(topic) || (!topic_exists(topic) && !ic::is_bound(&termlink_session::discovery::runtime_dir(), topic)) {",
     "    if !ic::applies(topic) {"),
    ("m27-full-table-clean-list", "r2 N1: an inbox missing from a full table is answered as a clean empty list", IC,
     "            && (!st.topics.contains_key(topic) || entered_after_first_drop)\n        {", "            && false\n        {"),
    ("m28-suppressed-count-lost", "r2 N5: the suppressed count is dropped when the row carrying it fails", IC,
     "    if log(row).is_ok() {\n        w.2 = 0;\n    }", "    let _ = log(row);\n    w.2 = 0;"),
    ("m29-bindings-unreadable-unbound", "r2 N2: unreadable bindings count as unbound for a missing topic", IC,
     "    load_bindings(dir).map(|b| b.contains_key(topic)).unwrap_or(true)", "    load_bindings(dir).map(|b| b.contains_key(topic)).unwrap_or(false)"),
    ("m30-reset-marker-not-saved", "r2 N4: the reset marker stays in memory until a record", IC,
     "            if let Err(e) = save_consumers(dir, &st) {\n                st.persist_error = Some(format!(\"consumer list not persisted: {e}\"));\n            }\n            st",
     "            st"),
    ("m31-gate-ignores-bind-before-create", "r3 CC 4: a bound inbox whose topic does not exist is read ungated", CH,
     "    if !ic::applies(topic) || (!topic_exists(topic) && !ic::is_bound(&termlink_session::discovery::runtime_dir(), topic)) {",
     "    if !ic::applies(topic) || !topic_exists(topic) {"),
    ("m32-ack-ignores-bind-before-create", "r3 CC 4: an ack on a bound, not-yet-created inbox is not gated", CH,
     "        && (topic_exists(&topic) || crate::inbox_consumers::is_bound(&termlink_session::discovery::runtime_dir(), &topic))",
     "        && topic_exists(&topic)"),
    ("m33-entered-after-drop-served", "r3 CC 1: an inbox entering after a drop is served as a clean list", IC,
     "        let entered_after_first_drop = st.topic_since.get(topic).is_some_and(|entered| *entered >= first);",
     "        let entered_after_first_drop = st.topic_since.get(topic).is_some_and(|entered| *entered >= first) && false;"),
    ("m26-full-table-evicts-active", "r1 CC 3: a full table evicts an inbox read inside the window", IC,
     "            Some((old, last)) if last < window_start => {", "            Some((old, _)) => {"),
]

CMDS = [
    ["cargo", "test", "-p", "termlink-hub", "--lib", "inbox_consumers"],
    ["cargo", "test", "-p", "termlink-sidecar", "--lib", "consumers"],
    ["cargo", "test", "-p", "termlink-sidecar", "--lib", "liveness"],
    ["cargo", "build", "-p", "termlink"],
    ["cargo", "test", "-p", "termlink-sidecar", "--test", "t3700_consumers"],
]

SCRATCH_RT = tempfile.mkdtemp(prefix="t3700-mutants-rt.")


def run():
    env = dict(os.environ)
    env.pop("PROJECT_ROOT", None)
    # T-3822: never let a mutant write into a live hub's runtime dir
    env["TERMLINK_RUNTIME_DIR"] = SCRATCH_RT
    out = ""
    for c in CMDS:
        r = subprocess.run(c, env=env, capture_output=True, text=True)
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
    # A mutant killed after `cargo build -p termlink` leaves its mutated hub binary on disk, and the L4 tests run that
    # binary without rebuilding it (found 2026-11-20: V-12 then failed on the clean tree). Rebuild it clean.
    env = dict(os.environ)
    env.pop("PROJECT_ROOT", None)
    if subprocess.run(["cargo", "build", "-p", "termlink"], env=env, capture_output=True).returncode != 0:
        print("WARNING: the clean termlink binary could not be rebuilt; run cargo build -p termlink")
    print("| mutant | what | verdict |\n|---|---|---|")
    for r in rows:
        print("| %s | %s | %s |" % r)
    print("\n%d/%d killed" % (len(rows) - survivors, len(rows)))
    return 0 if survivors == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
