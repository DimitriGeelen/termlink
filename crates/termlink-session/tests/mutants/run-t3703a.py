#!/usr/bin/env python3
"""T-3703 slice A (AL-20) mutants (FP 3.1.b): apply each, run the slice A tests, restore. Every mutant must be killed.

Run from the repo root:  python3 crates/termlink-session/tests/mutants/run-t3703a.py
Writes a table to stdout; exit 0 only if every mutant is killed and the unmutated suite passes.
"""
import os
import subprocess
import sys

SRC = "crates/termlink-session/src/signed_state.rs"

MUTANTS = [
    ("m1-unsorted-members", "verify over the parse order, not the canonical order (E-42.5)",
     "    m.sort_by(|a, b| a.0.encode_utf16().cmp(b.0.encode_utf16()));\n", "\n"),
    ("m2-accept-fraction", "a fraction or exponent is not refused as a number (V-NEG-1)",
     "        if matches!(self.peek(), Some(b'.' | b'e' | b'E')) {\n            return Err(Refused::Number(start));",
     "        if false {\n            return Err(Refused::Number(start));"),
    ("m3-keep-duplicate", "a duplicate key is kept instead of refused (V-NEG-2)",
     "            if m.iter().any(|(x, _)| *x == k) {", "            if false {"),
    ("m4-bare-digest", "sign and verify over the bare digest, no domain prefix (V-NEG-3)",
     "    format!(\"{schema}\\n{digest_hex}\").into_bytes()", "    { let _ = schema; digest_hex.as_bytes().to_vec() }"),
    ("m5-trust-written-digest", "the written digest is trusted, not recomputed (FCW-89)",
     "        if recomputed != written {\n            return Err(Refused::DigestMismatch);\n        }\n    }",
     "        if false {\n            return Err(Refused::DigestMismatch);\n        }\n        let recomputed = written.to_string();\n        return key.verify_strict(&signing_input(schema, &recomputed), &Signature::from_bytes(&unhex_lower(obj.get(\"signature\").and_then(J::as_str).unwrap_or(\"\")).and_then(|b| b.try_into().ok()).ok_or(Refused::Field(\"signature\"))?)).map_err(|_| Refused::BadSignature);\n    }"),
    ("m6-range", "an integer above 2^53-1 is accepted (V-NEG-4)",
     "        if mag > MAX_INT {\n            return Err(Refused::Number(start));\n        }\n        Ok(J::Int",
     "        if false {\n            return Err(Refused::Number(start));\n        }\n        Ok(J::Int"),
]

CMD = ["cargo", "test", "-p", "termlink-session", "--test", "t3703_signed_state"]


def run():
    env = dict(os.environ)
    env.pop("PROJECT_ROOT", None)
    r = subprocess.run(CMD, env=env, capture_output=True, text=True)
    return r.returncode, r.stdout + r.stderr


def main():
    orig = open(SRC).read()
    rc, out = run()
    if rc != 0:
        print("unmutated suite FAILS; no mutant run\n" + out[-2000:])
        return 2
    rows, survivors = [], 0
    try:
        for mid, what, old, new in MUTANTS:
            if orig.count(old) != 1:
                rows.append((mid, what, "NOT APPLIED (pattern count %d)" % orig.count(old)))
                survivors += 1
                continue
            open(SRC, "w").write(orig.replace(old, new))
            rc, out = run()
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
        open(SRC, "w").write(orig)
    print("| mutant | what | verdict |\n|---|---|---|")
    for r in rows:
        print("| %s | %s | %s |" % r)
    print("\n%d/%d killed" % (len(rows) - survivors, len(rows)))
    return 0 if survivors == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
