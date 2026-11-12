#!/usr/bin/env python3
"""T-3686 mutant list (negative controls, FP 3.1.b). Each mutant patches the source, runs the
named tests, and must be KILLED (tests fail). Source is always restored. Exit 0 = all killed,
1 = a mutant survived, 2 = tooling (patch target not found / build broke for another reason)."""
import os, subprocess, sys

ROOT = "/opt/termlink"
os.environ.setdefault("TMPDIR", "/var/tmp")
LEASE = "crates/termlink-session/src/main_lease.rs"
SEAM = "crates/termlink-session/src/test_seam.rs"
CLI = "crates/termlink-cli/src/commands/main_lease.rs"
UNIT = ["cargo", "test", "-p", "termlink-session", "-q", "--lib", "--", "main_lease", "test_seam"]
INTEG = ["cargo", "test", "-p", "termlink", "-q", "--test", "main_lease_integration"]

MUTANTS = [
    ("M1 gate removed (seams obeyed outside an estate)", LEASE,
     "        if !in_estate() {\n            return Err(SeamRefused { vars: set });\n        }\n", "", [UNIT, INTEG]),
    ("M2 estate test always true", SEAM,
     "    in_estate(home.as_deref(), &crate::discovery::runtime_dir())\n",
     "    let _ = in_estate(home.as_deref(), &crate::discovery::runtime_dir());\n    true\n", [INTEG]),
    ("M3 runtime dir not checked (HOME alone decides)", SEAM,
     "        (Some(a), Some(b)) => a == b,\n        _ => false,",
     "        (Some(_), _) => true,\n        _ => false,", [UNIT]),
    ("M4 marker magic not checked", SEAM,
     ".map(|s| s.lines().next() == Some(ESTATE_MARKER_MAGIC))", ".map(|_| true)", [UNIT]),
    ("M5 seam detection by one exact name instead of prefix", SEAM,
     ".filter(|k| k.starts_with(MAIN_SEAM_PREFIX))", '.filter(|k| k == "TERMLINK_MAIN_TEST_LEASE_MS")', [UNIT]),
    ("M6 CLI swallows the refusal (silent defaults)", CLI,
     "        Err(e) => return unknown(session.as_deref(), e.to_string(), json),\n    };\n    let Some(project)",
     "        Err(_) => Timing::default(),\n    };\n    let Some(project)", [INTEG]),
]


def run(cmd):
    """rc 0 = pass, 1 = a test FAILED (kill), 2 = did not compile/run (tooling, not a kill)."""
    p = subprocess.run(cmd, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    if p.returncode == 0:
        return 0
    return 1 if ("test result: FAILED" in p.stdout or "panicked" in p.stdout) else 2


def main():
    survived, tooling = [], []
    for name, rel, old, new, tests in MUTANTS:
        path = os.path.join(ROOT, rel)
        src = open(path).read()
        if src.count(old) != 1:
            print(f"TOOLING {name}: patch target found {src.count(old)} times in {rel}")
            tooling.append(name)
            continue
        try:
            open(path, "w").write(src.replace(old, new))
            rcs = [run(t) for t in tests]
        finally:
            open(path, "w").write(src)
        if 2 in rcs:
            print(f"TOOLING {name}: build or run error (rc={rcs})")
            tooling.append(name)
            continue
        killed = any(rc == 1 for rc in rcs)
        print(f"{'KILLED ' if killed else 'SURVIVED'} {name} (rc={rcs})")
        if not killed:
            survived.append(name)
    print(f"summary: {len(MUTANTS) - len(survived) - len(tooling)} killed, {len(survived)} survived, {len(tooling)} tooling")
    sys.exit(2 if tooling else 1 if survived else 0)


if __name__ == "__main__":
    main()
