#!/usr/bin/env bash
# check-env-var-docs.sh (T-2220) — Level-C prevention for env-var-name doc drift.
#
# Sibling to check-error-code-docs.sh (T-2217). Scans every TERMLINK_* env var
# CITED in operator docs (CLAUDE.md, docs/operations/*.md, .claude/commands/*.md)
# against the UNION of TERMLINK_* names actually REFERENCED in the implementation
# surfaces (crates/, scripts/, systemd-templates/). Any doc-cited var with no
# implementation surface is drift — an operator who sets it gets silent no-op,
# or a copy-pasted notify-hook gate that never fires (origin: T-2219, PL-217).
#
# A doc token ending in '_' is treated as a glob-prefix mention (e.g. the prose
# "TERMLINK_WATCH_*" captures as TERMLINK_WATCH_) and is OK if any impl var
# starts with it.
#
# Exit: 0 = clean, 1 = drift found, 2 = tooling error. Pass --json for machine output.
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || { echo "check-env-var-docs: not a git repo" >&2; exit 2; }
cd "$ROOT"

JSON_MODE="${1:-}" python3 - <<'PYEOF'
import os, re, subprocess, sys, json

def gg(patterns):
    r = subprocess.run(["git", "grep", "-hoE", "TERMLINK_[A-Z_]+", "--", *patterns],
                       capture_output=True, text=True)
    # git grep exits 1 when no matches — not an error for us
    return set(r.stdout.split())

DOC_PATHS  = ["CLAUDE.md", "docs/operations/*.md", ".claude/commands/*.md", ":!.claude/worktrees/*"]
IMPL_PATHS = ["crates/*", "scripts/*", "systemd-templates/*", ":!.claude/worktrees/*"]

docs = gg(DOC_PATHS)
impl = gg(IMPL_PATHS)

# Legitimately doc-only env vars (none currently). Add with a justifying comment.
ALLOW = set()

mismatches = []
for tok in sorted(docs):
    if tok in impl:
        continue
    if tok.endswith("_") and any(i.startswith(tok) for i in impl):
        continue  # glob-prefix doc mention
    if tok in ALLOW:
        continue
    mismatches.append(tok)

json_mode = os.environ.get("JSON_MODE") == "--json"
if json_mode:
    print(json.dumps({"ok": not mismatches, "mismatches": mismatches,
                      "doc_tokens": len(docs), "impl_tokens": len(impl)}))
else:
    if mismatches:
        print("env-var doc lint: DRIFT — doc-cited TERMLINK_* vars with no implementation surface:")
        for m in mismatches:
            loc = subprocess.run(
                ["git", "grep", "-nE", m + r"\b", "--", *DOC_PATHS],
                capture_output=True, text=True).stdout.splitlines()
            print(f"  {m}   {loc[0] if loc else '(citation not located)'}")
        print(f"\nScanned {len(docs)} doc-cited env-vars against {len(impl)} impl env-vars "
              "(crates/+scripts/+systemd-templates/).")
        print("Fix: correct the doc to the name the source actually reads, or add a "
              "justified entry to ALLOW in this script.")
    else:
        print(f"env-var doc lint: CLEAN — all {len(docs)} doc-cited TERMLINK_* vars have "
              f"an implementation surface ({len(impl)} impl vars scanned)")

sys.exit(1 if mismatches else 0)
PYEOF
