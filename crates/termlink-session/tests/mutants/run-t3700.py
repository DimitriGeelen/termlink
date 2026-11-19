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
     "    if (now_ms - p.ts_ms).abs() > PROOF_WINDOW_MS {", "    if false {"),
    ("m3-bind-by-name", "a bind admits any reader naming the bound agent (ruling E broken)", IC,
     "        let ok = reader.strength == Strength::Proven && reader.agent == b.agent && reader.key_fp.as_deref() == Some(b.key_fp.as_str());",
     "        let ok = reader.agent == b.agent;"),
    ("m4-hub-unreadable-is-empty", "E-169.5 control: an unreadable consumer list answers as a list", IC,
     "        if let Some(e) = st.load_error.as_ref().or(st.persist_error.as_ref()) {\n            return Err(e.clone());\n        }",
     "        let _ = (&st.load_error, &st.persist_error);"),
    ("m5-sidecar-malformed-is-one", "a malformed hub answer is shown as one consumer", SC,
     "        return unknown(\"the hub's answer holds no consumer list\".into());",
     "        return Verdict { line: \"one (proven)\".into(), others: vec![], bound: None };"),
    ("m6-acks-not-recorded", "E-66.5-style control: an ack (receipt) does not record its poster", CH,
     "    if msg_type == \"receipt\" && crate::inbox_consumers::applies(&topic) {",
     "    if false && msg_type == \"receipt\" && crate::inbox_consumers::applies(&topic) {"),
    ("m7-reads-not-recorded", "E-66.6 control: only acks and claims record; a read-only consumer is missed", CH,
     "    if let Some(r) = consumer_gate(&id, params, \"channel.subscribe\", \"cursor\", crate::inbox_consumers::Action::Read, peer_addr) {",
     "    if let Some(r) = None::<RpcResponse> {"),
    ("m8-no-eviction-row", "the 17th reader evicts silently", IC,
     "                        \"event\": \"inbox-consumer-evicted\", \"topic\": topic, \"agent\": gone.agent,",
     "                        \"event\": \"x\", \"topic\": topic, \"agent\": gone.agent,"),
    ("m9-hub-instance-ignored", "V-12: the hub keys consumers by agent only (a second copy merges)", IC,
     "    match list.iter_mut().find(|e| e.agent == reader.agent && e.instance == reader.instance) {",
     "    match list.iter_mut().find(|e| e.agent == reader.agent) {"),
    ("m10-sidecar-instance-ignored", "V-12: the sidecar treats its own agent at any instance as itself", SC,
     "        let is_me = seen.agent == me_agent && (seen.instance == me_instance || seen.instance == ACK_INSTANCE);",
     "        let is_me = seen.agent == me_agent;"),
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
    ("m15-liveness-reads-content", "liveness reads the main's inbox content (a consumer of it) instead of first_at", LV,
     "        Ok(n) => match hub.call(method::CHANNEL_FIRST_AT, json!({\"topic\": inbox, \"from_offset\": n.drained_up_to})) {",
     "        Ok(n) => match hub.call(method::CHANNEL_FIRST_AT, json!({\"topic\": \"-\", \"from_offset\": n.drained_up_to})) {"),
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
