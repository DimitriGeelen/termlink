#!/usr/bin/env bash
# guard-layer: source
# T-2812 — unregistered local modifications to vendored framework code.
#
# The framework is vendored, and a WHOLESALE vendor event (`fw upgrade`,
# `bootstrap-replace`, `fw update`) replaces the tree roughly every two months.
# Any local fix upstream does not carry is deleted by that routine operation, in
# silence. This check answers "what have we changed since the last vendor event
# that nobody has registered?" — which, before `.vendor-divergence.yaml` existed,
# could only be answered by reading 124 commit messages.
#
# It was found the hard way: T-2687 was about to close as complete, 12/12
# fixtures green, with its fix to vendored `lib/pickup.sh` unfiled and one
# `fw update` from deletion — while a re-vendor was being proposed on another
# branch.
#
# WHAT IT CHECKS. Commits touching `.agentic-framework/` since the
# `last_vendor_event` recorded in `.vendor-divergence.yaml`, minus:
#   * commits whose task id is registered under `divergences:`
#   * commits whose task id is registered under `not_divergence:` (recovery of
#     files that exist upstream, ports OF upstream fixes, committed artifacts)
#   * the vendor events themselves
# Whatever remains is a local change to framework code that nobody has decided
# about. That is the firing set.
#
# WHY THE BASELINE IS READ FROM THE REGISTER rather than detected. The first
# implementation grepped commit subjects for vendor-event keywords and picked a
# commit saying "re-vendor RECOMMENDED" — recommended, not performed — as the
# baseline, which silently shortened the at-risk window from 12 commits to 4. A
# heuristic that can quietly understate the answer is worse than a declared
# constant someone has to update deliberately.
#
# Exit codes: 0 = every local change is registered, 1 = unregistered change(s),
# 2 = tooling error. FAIL-CLOSED: a missing or unparseable register exits 2,
# never 0 — reporting "clean" because the register could not be read is the
# failure mode this whole file is about.
#
# Usage: bash scripts/check-vendor-divergence.sh [--json] [--quiet]
# Test seam (PL-213): VENDOR_DIVERGENCE_REGISTER / VENDOR_DIVERGENCE_REPO.
# Fixtures: bash tests/vendor-divergence-fixtures.sh

set -uo pipefail

JSON=0
QUIET=0
while [ $# -gt 0 ]; do
    case "$1" in
        --json)  JSON=1 ;;
        --quiet) QUIET=1 ;;
        -h|--help) sed -n '3,42p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "check-vendor-divergence: unknown flag: $1" >&2; exit 2 ;;
    esac
    shift
done

command -v python3 >/dev/null 2>&1 || {
    echo "check-vendor-divergence: python3 not found — cannot verify (fail-closed)" >&2
    exit 2
}

REPO="${VENDOR_DIVERGENCE_REPO:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
REGISTER="${VENDOR_DIVERGENCE_REGISTER:-$REPO/.vendor-divergence.yaml}"

cd "$REPO" 2>/dev/null || {
    echo "check-vendor-divergence: cannot enter repo $REPO (fail-closed)" >&2
    exit 2
}

REGISTER="$REGISTER" CHECK_JSON="$JSON" CHECK_QUIET="$QUIET" python3 - <<'PY'
import json, os, re, subprocess, sys

try:
    import yaml
except ImportError:
    sys.stderr.write("check-vendor-divergence: PyYAML not installed (fail-closed)\n")
    sys.exit(2)

REG   = os.environ["REGISTER"]
JSON  = os.environ.get("CHECK_JSON") == "1"
QUIET = os.environ.get("CHECK_QUIET") == "1"

if not os.path.isfile(REG):
    sys.stderr.write("check-vendor-divergence: register not found at %s — cannot verify "
                     "(fail-closed)\n" % REG)
    sys.exit(2)
try:
    reg = yaml.safe_load(open(REG)) or {}
except yaml.YAMLError as e:
    sys.stderr.write("check-vendor-divergence: register does not parse: %s (fail-closed)\n"
                     % str(e).split("\n")[0])
    sys.exit(2)

ev = (reg.get("last_vendor_event") or {}).get("commit")
if not ev:
    sys.stderr.write("check-vendor-divergence: register has no last_vendor_event.commit "
                     "(fail-closed)\n")
    sys.exit(2)

known = set()
for d in (reg.get("divergences") or []):
    if d.get("task"):
        known.add(d["task"])
for grp in (reg.get("not_divergence") or []):
    for t in (grp.get("tasks") or []):
        known.add(t)

def git(*a):
    r = subprocess.run(("git",) + a, capture_output=True, text=True)
    return r.stdout, r.returncode

out, rc = git("log", "--pretty=format:%h\t%ad\t%s", "--date=short",
              "%s..HEAD" % ev, "--", ".agentic-framework/")
if rc != 0:
    sys.stderr.write("check-vendor-divergence: git log failed for baseline %s "
                     "(fail-closed)\n" % ev)
    sys.exit(2)

# A vendor event is itself not local divergence. Matched narrowly and ONLY to
# exclude — never to pick the baseline, which is declared.
VENDOR_RE = re.compile(r"\b(fw upgrade|fw update|bootstrap-replace|de-vendor|vendor .*refresh)\b", re.I)

unregistered, scanned = [], 0
for line in out.splitlines():
    if not line.strip():
        continue
    scanned += 1
    sha, date, subj = (line.split("\t", 2) + ["", ""])[:3]
    if VENDOR_RE.search(subj):
        continue
    m = re.match(r"\s*(T-\d+)", subj)
    task = m.group(1) if m else None
    if task and task in known:
        continue
    unregistered.append({"commit": sha, "date": date, "task": task or "(untagged)",
                         "subject": subj[:100]})

if JSON:
    print(json.dumps({
        "ok": not unregistered,
        "baseline": ev,
        "scanned": scanned,
        "registered": sorted(known),
        "unregistered_count": len(unregistered),
        "unregistered": unregistered,
    }, indent=2))
    sys.exit(1 if unregistered else 0)

if not unregistered:
    if not QUIET:
        print("check-vendor-divergence: %d commit(s) touch vendored code since %s, all "
              "registered" % (scanned, ev))
    sys.exit(0)

print("check-vendor-divergence: %d unregistered local change(s) to vendored framework code"
      % len(unregistered))
print("  (baseline: last vendor event %s; %d commit(s) scanned)" % (ev, scanned))
print()
for u in unregistered:
    print("  %s  %s  %s" % (u["commit"], u["date"], u["subject"]))
print()
if not QUIET:
    print("Each of these is deleted by the next `fw upgrade` / re-vendor unless upstream")
    print("carries it. Decide per commit and record it in .vendor-divergence.yaml:")
    print("  divergences:    a real local fix — file it upstream, set status accordingly")
    print("  not_divergence: recovery of untracked files, a port OF an upstream fix, or a")
    print("                  committed artifact — a re-vendor costs nothing")
sys.exit(1)
PY
