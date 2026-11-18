#!/usr/bin/env python3
"""T-3703 slice C (sidecar revocation pull) mutants (FP 3.1.b): apply each, run the slice A tests, restore. Every mutant must be killed.

Run from the repo root:  python3 crates/termlink-session/tests/mutants/run-t3703c.py [mutant-id ...]
Writes a table to stdout; exit 0 only if every mutant is killed and the unmutated suite passes.
"""
import os
import subprocess
import sys

REV = "crates/termlink-sidecar/src/revocation.rs"
PULL = "crates/termlink-sidecar/src/pull.rs"
POLICY = "crates/termlink-sidecar/src/policy.rs"

# (id, what, file, old, new)
MUTANTS = [
    ("c1-failed-pull-empty", "a failed pull is treated as an empty list (E-41.5, 04 6.2.6)", REV,
     "        self.publish(store);\n        let line = if why.starts_with(\"ROLLBACK\")",
     "        publish(store, &BTreeSet::new(), self.narrowing());\n        let line = if why.starts_with(\"ROLLBACK\")"),
    ("c2-any-seq", "a lower seq is accepted (E-40.5)", REV,
     "            if hseq < held.seq {", "            if false {"),
    ("c3-no-receive-check", "the receiver does not check the held list (E-39.5)", PULL,
     "        if crate::revocation::revoked(&self.store, &fp) {\n            let out",
     "        if false {\n            let out"),
    ("c4-leave-stored", "a message stored before the revocation is left STORED (E-39.6)", REV,
     "            if !is_revoked {\n                continue;\n            }\n            let done",
     "            if is_revoked || !is_revoked {\n                continue;\n            }\n            let done"),
    ("c5-no-pin-check", "the hub key is used without checking it against the pin", REV,
     "            (Some(k), Some(fp)) if ss::key_fingerprint(&k) == fp => k,", "            (Some(k), _) => k,"),
    ("c6-stale-reset", "a failed pull resets the stale timer (E-41 59:59 / 60:01)", REV,
     "    fn failed(&mut self, store: &Store, now: Instant, why: &str, tamper: bool) {\n",
     "    fn failed(&mut self, store: &Store, now: Instant, why: &str, tamper: bool) {\n        self.last_verified = Some(now);\n"),
    ("c7-start-verified", "a started sidecar is VERIFIED from its store (FCW-96)", REV,
     "            state: State::UnverifiedSinceStart,\n            held: None,", "            state: State::Verified,\n            held: None,"),
    ("c8-accept-fork", "the same seq with another digest is accepted", REV,
     "                    Err((format!(\"fork: seq {hseq} with another digest\"), true))", "                    Ok(())"),
    ("c9-no-hub-id", "a head naming another hub is accepted", REV,
     "        if head.get(\"hub_id\").and_then(J::as_str) != Some(self.hub_id.as_str()) {", "        if false {"),
    ("c10-no-load-verify", "the stored list is trusted on load (r1 M1)", REV,
     "                (Some(k), true) => match ss::parse_and_verify(text.as_bytes(), &k, SCHEMA) {",
     "                (Some(_k), true) => match ss::parse_strict(text.as_bytes()) {"),
    ("c11-no-chain", "a list that does not chain at held + 1 is accepted", REV,
     "                && l.get(\"prev_digest\").and_then(J::as_str) != Some(held.digest.as_str())",
     "                && false"),
    # round 2 (r2-dispositions.md)
    ("c12-handle-before-head", "messages are handled before the head is read (design 5.4, r2 OC 10)", PULL,
     "        self.pull_revocations(&version);\n        let topic",
     "        let _ = &version;\n        let topic"),
    ("c13-outage-not-counted", "a pass that cannot reach the hub is not a failed pull (r2 CC 1)", PULL,
     "        t.outage(&self.store, std::time::Instant::now(), why);", "        let _ = (&t, why);"),
    ("c14-no-stale-gate", "the stale state does not narrow urgent (DR-8 A, r2 CC 3)", POLICY,
     "    urgent && (!narrowing || trust == Trust::OwnProject)", "    let _ = (narrowing, trust);\n    urgent"),
    ("c15-no-fallback", "with no snapshot, nothing is revoked (r2 CC 9)", REV,
     "        .map(|l| keys_of(&entries_of(&l)))\n        .unwrap_or_default()", "        .map(|_l| BTreeSet::new())\n        .unwrap_or_default()"),
    ("c16-raise-all", "the entry names every key, not the new ones (r2 CC 10)", REV,
     ".is_some_and(|k| !named.contains(k))).collect();", ".is_some_and(|_k| true)).collect();"),
    ("c17-no-heal", "an unverifiable stored list keeps its floor and keys (r2 CC 7)", REV,
     "        if self.held_unverified {\n            return self.fetch", "        if false {\n            return self.fetch"),
    ("c18-sweep-open-attempt", "the sweep refuses a message whose attempt is open (r2 CC 9)", REV,
     "            if open {\n                continue;", "            if false && open {\n                continue;"),
]

CMDS = [
    ["cargo", "test", "-p", "termlink-sidecar", "--lib", "revocation"],
    ["cargo", "test", "-p", "termlink-sidecar", "--test", "t3703_revocation_sidecar"],
    ["cargo", "test", "-p", "termlink-sidecar", "--test", "policy_tests"],
]


def run():
    env = dict(os.environ)
    env.pop("PROJECT_ROOT", None)
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
    try:
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
    finally:
        pass
    print("| mutant | what | verdict |\n|---|---|---|")
    for r in rows:
        print("| %s | %s | %s |" % r)
    print("\n%d/%d killed" % (len(rows) - survivors, len(rows)))
    return 0 if survivors == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
