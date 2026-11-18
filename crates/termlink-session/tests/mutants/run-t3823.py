#!/usr/bin/env python3
"""T-3823 (P-31 code follow-ups) mutants (FP 3.1.b): apply each, run the slice A tests, restore. Every mutant must be killed.

Run from the repo root:  python3 crates/termlink-session/tests/mutants/run-t3823.py [mutant-id ...]
Writes a table to stdout; exit 0 only if every mutant is killed and the unmutated suite passes.
"""
import os
import tempfile
import subprocess
import sys

HUBREV = "crates/termlink-hub/src/revocation.rs"
KC = "crates/termlink-hub/src/key_card.rs"
SCREV = "crates/termlink-sidecar/src/revocation.rs"
PULL = "crates/termlink-sidecar/src/pull.rs"

# (id, what, file, old, new)
MUTANTS = [
    ("d1-no-digest-in-stamp", "the hub list cache ignores the file digest (same-length rewrite served stale)", HUBREV,
     "    Some((len, t, ss::hex(&Sha256::digest(&bytes))))", "    let _ = &bytes;\n    Some((len, t, String::new()))"),
    ("d2-fallback-cache-stale", "the sidecar fallback cache ignores the stored list's stamp", SCREV,
     "    if let Some((st, keys)) = cache.get(store.dir())\n        && *st == stamp\n",
     "    if let Some((_st, keys)) = cache.get(store.dir())\n"),
    ("d3-no-baseline", "a fresh sidecar raises an entry for every existing card", PULL,
     "            if !baseline {\n                let detail = format!(\"key card for agent", "            if true {\n                let detail = format!(\"key card for agent"),
    ("d4-no-mode-entry", "a change of the check setting raises nothing", PULL,
     "                if let Some(w) = &was {", "                if let Some(w) = None::<&String> {"),
    ("d5-outside-edit-weakens", "an outside edit of the setting can weaken the check", KC,
     "            return if strictness(edited) > strictness(baseline) { edited } else { baseline };",
     "            let _ = baseline;\n            return edited;"),
    ("d6-own-key-capped", "the card's own key can be pushed into other by the cap", KC,
     "    let s = if own_key || e.signers.contains_key(signer_fp)", "    let s = if (own_key && false) || e.signers.contains_key(signer_fp)"),
]

CMDS = [
    ["cargo", "test", "-p", "termlink-hub", "--lib", "t3823"],
    ["cargo", "test", "-p", "termlink-hub", "--lib", "key_card"],
    ["cargo", "test", "-p", "termlink-sidecar", "--lib", "t3823"],
    ["cargo", "build", "-p", "termlink"],
    ["cargo", "test", "-p", "termlink-sidecar", "--test", "t3704_impersonation", "t3823"],
]


SCRATCH_RT = tempfile.mkdtemp(prefix="t3823-mutants-rt.")


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
