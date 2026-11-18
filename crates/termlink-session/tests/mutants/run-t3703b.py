#!/usr/bin/env python3
"""T-3703 slice B (hub revocation list) mutants (FP 3.1.b): apply each, run the slice A tests, restore. Every mutant must be killed.

Run from the repo root:  python3 crates/termlink-session/tests/mutants/run-t3703b.py [mutant-id ...]
Writes a table to stdout; exit 0 only if every mutant is killed and the unmutated suite passes.
"""
import os
import tempfile
import subprocess
import sys

HUB = "crates/termlink-hub/src/revocation.rs"
AUDIT = "crates/termlink-hub/src/rpc_audit.rs"
CHAN = "crates/termlink-hub/src/channel.rs"

# (id, what, file, old, new)
MUTANTS = [
    ("b1-no-row-before", "hub.revoke runs without its intent row (E-167.5 / FCW-91)", AUDIT,
     '    "hub.revoke",\n', ''),
    ("b2-unsigned-request", "a self-request is not verified under the key it revokes (E-43 / FCW-90)", HUB,
     "    ss::verify(&pk, req, Some(REQUEST_SCHEMA)).map_err(|e| format!(\"request not signed by the key it revokes: {e} (FCW-90)\"))?;",
     "    let _ = ss::verify(&pk, req, Some(REQUEST_SCHEMA));"),
    ("b3-not-idempotent", "re-revoking a listed key issues a new seq", HUB,
     "        return Ok(Issued { seq: seq_of(&cur), digest: digest_of(&cur), already: true });",
     "        let _ = 0;"),
    ("b4-unchained", "prev_digest does not chain the versions", HUB,
     '        ("prev_digest".into(), J::Str(digest_of(cur))),', '        ("prev_digest".into(), J::Str(ZERO.into())),'),
    ("b5-no-post-check", "the hub post check ignores the revocation list (design 5.8)", CHAN,
     "                Ok(keys) if !keys.contains(&fp) => return None,", "                Ok(_) => return None,"),
    ("b6-restore-not-blocked", "a list behind the committed log is served (design 5.7)", HUB,
     '                block(format!("revocation list is at seq {fs}, behind the committed seq {s} (a restore?)"), &mut out);',
     '                out.push(format!("revocation list is at seq {fs}, behind the committed seq {s} (a restore?)"));'),
    ("b7-no-card-needed", "a self-request without a key card is accepted with the request's names (r1 finding 8)", HUB,
     '    let Some((agent, project)) = card_of(&pk_bytes)? else {\n        return Err(',
     '    let Some((agent, project)) = card_of(&pk_bytes)?.or(Some(("fx-x".to_string(), "p".to_string()))) else {\n        return Err('),
    ("b8-write-failure-ignored", "a failed list write is reported as success (FCW-92)", HUB,
     '    atomic_write(&list_path(dir), text.as_bytes())\n        .map_err(|e| format!("revocation list not written ({e}); the previous list stays (FCW-92)"))?;',
     '    let _ = atomic_write(&list_path(dir), text.as_bytes());'),
    # round 2 (r2-dispositions.md)
    ("b9-post-check-ignores-reconcile", "while reconciling, the post check misses keys the restore lost (r2 OC 1)", HUB,
     "    if reconcile_reason(dir).is_some() {\n        let log", "    if false {\n        let log"),
    ("b10-reissue-chains-to-file", "a reissue chains to the restored file, not the committed head (r2 CC 5)", HUB,
     "    let prev = if file_top >= logged_top {", "    let prev = if true {"),
    ("b11-torn-row-ignored", "a torn committed row does not block (r2 OC 6)", HUB,
     "    if torn > 0 {", "    if false {"),
    # round 3 (r3-dispositions.md)
    ("b12-torn-never-superseded", "a torn row keeps blocking after a reissue (r3 CC N1)", HUB,
     "                rows.push(r);\n                torn_since_last = 0;", "                rows.push(r);"),
]

CMDS = [
    ["cargo", "test", "-p", "termlink-hub", "--lib", "revocation"],
    ["cargo", "test", "-p", "termlink", "--test", "t3703_revocation"],
]


SCRATCH_RT = tempfile.mkdtemp(prefix="t3703-mutants-rt.")


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
