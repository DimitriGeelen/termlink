#!/usr/bin/env bash
# check-verification-pipefail.sh — T-2818.
#
# Repo-wide auditor for the L-387 SIGPIPE shape inside task `## Verification` blocks.
#
# The defect
# ----------
# Under `set -o pipefail`, `cmd | grep -q PATTERN` exits 141 when the pattern MATCHES:
# `grep -q` exits on first match and closes the pipe, SIGPIPE kills the upstream, and
# pipefail propagates the upstream's status. The pipeline fails on success.
#
# The P-011 verification gate runs every `## Verification` line under `set -euo pipefail`,
# so a command in that shape blocks completion of a task whose verification actually
# passed.
#
# Safe rewrites:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
#     cmd > /tmp/.out 2>&1; grep -q "PATTERN" /tmp/.out
#
# Why a repo-wide check
# ---------------------
# The framework already ships the detector (lib/reviewer/static_scan.py::
# detect_l387_sigpipe_risk) and fires it as a NON-BLOCKING ADVISORY at `started-work`, one
# task at a time (T-2059). Nothing ever asked the repo-wide question — and the answer here
# was 150 findings across 188 task files.
#
# In isolation the failure direction looks safe: the gate blocks rather than waves through.
# In aggregate it is not. A gate that blocks incorrectly 150 times teaches the operator that
# P-011 failures are noise and that `--force` is the normal way past them, and a verification
# gate people routinely force is a verification gate that no longer verifies.
#
# This is a REVIEW list and a DEPLOY-TIME / ad-hoc check, not a cron canary — same tier as
# check-cron-install-drift.sh (T-2561) and the four static checks in CLAUDE.md.
#
# Single source of truth
# ----------------------
# The heuristic is NOT reimplemented here. This wraps the framework's own
# `detect_l387_sigpipe_risk`, which resolves the producer as the LAST pipeline stage before
# the terminal grep, exempts `echo`/`printf` upstreams (SIGPIPE-immune bounded buffer), and
# skips comment lines. Two copies of a subtle SIGPIPE heuristic would drift, and the one
# that drifts is the one that stops catching things.
#
# Usage:
#   bash scripts/check-verification-pipefail.sh [--json] [--quiet] [--active-only]
#                                               [--tasks-dir DIR] [--framework-root DIR]
#
# Exit codes:
#   0 — no findings
#   1 — at least one risky verification command
#   2 — tooling error (detector unimportable, tasks dir absent)
#
# FAIL-CLOSED: if the framework detector cannot be imported this exits 2, never 0. A
# detector that reports clean because it failed to load converts an unknown into a false
# assurance, which is worse than having no detector at all.

set -uo pipefail

TASKS_DIR=".tasks"
FRAMEWORK_ROOT=".agentic-framework"
JSON=0
QUIET=0
ACTIVE_ONLY=0

usage() { sed -n '2,/^set -uo pipefail$/p' "$0" | sed 's/^# \{0,1\}//' | head -n -2; exit 0; }

while [ $# -gt 0 ]; do
    case "$1" in
        --json) JSON=1 ;;
        --quiet) QUIET=1 ;;
        --active-only) ACTIVE_ONLY=1 ;;
        --tasks-dir)
            shift; [ $# -ge 1 ] || { echo "check-verification-pipefail: --tasks-dir requires a value" >&2; exit 2; }
            TASKS_DIR="$1" ;;
        --framework-root)
            shift; [ $# -ge 1 ] || { echo "check-verification-pipefail: --framework-root requires a value" >&2; exit 2; }
            FRAMEWORK_ROOT="$1" ;;
        -h|--help) usage ;;
        *) echo "check-verification-pipefail: unknown flag: $1" >&2; exit 2 ;;
    esac
    shift
done

if [ ! -d "$TASKS_DIR" ]; then
    echo "check-verification-pipefail: tasks dir not found: $TASKS_DIR" >&2
    exit 2
fi

if ! command -v python3 >/dev/null 2>&1; then
    echo "check-verification-pipefail: python3 not available" >&2
    exit 2
fi

OUT="$(python3 - "$FRAMEWORK_ROOT" "$TASKS_DIR" "$ACTIVE_ONLY" "$JSON" <<'PY'
import sys, os, json, glob

framework_root, tasks_dir, active_only, as_json = sys.argv[1], sys.argv[2], sys.argv[3] == "1", sys.argv[4] == "1"

sys.path.insert(0, framework_root)
try:
    from lib.reviewer import static_scan as ss
except Exception as e:
    # Fail-closed: never report clean because the detector would not load.
    sys.stderr.write(
        "check-verification-pipefail: cannot import the framework detector from "
        "%s/lib/reviewer/static_scan.py (%s)\n" % (framework_root, e))
    sys.stderr.write(
        "  Point --framework-root at a checkout that has it. If it is genuinely missing,\n"
        "  that is itself vendoring drift: bash scripts/check-framework-tracking-drift.sh\n")
    sys.exit(2)

pattern = os.path.join(tasks_dir, "active", "*.md") if active_only \
    else os.path.join(tasks_dir, "**", "*.md")
files = sorted(glob.glob(pattern, recursive=not active_only))

findings = []
scanned = 0
with_block = 0
for f in files:
    scanned += 1
    try:
        with open(f, encoding="utf8", errors="replace") as fh:
            text = fh.read()
    except Exception:
        continue
    verif = ss.extract_section(text, "Verification") or ""
    if not verif:
        continue
    with_block += 1
    for x in ss.detect_l387_sigpipe_risk(verif):
        findings.append({
            "file": f,
            "location": getattr(x, "location", ""),
            "evidence": (getattr(x, "evidence", "") or "").strip(),
        })

if as_json:
    print(json.dumps({
        "ok": not findings,
        "scanned": scanned,
        "with_verification_block": with_block,
        "finding_count": len(findings),
        "findings": findings,
    }))
else:
    for x in findings:
        print("  RISK  %s: %s" % (x["file"], x["evidence"][:120]))
    print("SUMMARY %d %d %d" % (len(findings), scanned, with_block))
PY
)"
RC=$?

if [ "$RC" = "2" ]; then
    exit 2
fi

if [ "$JSON" = "1" ]; then
    printf '%s\n' "$OUT"
    printf '%s' "$OUT" | grep -q '"ok": true' && exit 0
    exit 1
fi

SUMMARY="$(printf '%s\n' "$OUT" | sed -n 's/^SUMMARY //p')"
COUNT="$(printf '%s' "$SUMMARY" | cut -d' ' -f1)"
SCANNED="$(printf '%s' "$SUMMARY" | cut -d' ' -f2)"
WITH_BLOCK="$(printf '%s' "$SUMMARY" | cut -d' ' -f3)"
[ -n "$COUNT" ] || { echo "check-verification-pipefail: internal error (no summary)" >&2; exit 2; }

if [ "$COUNT" = "0" ]; then
    [ "$QUIET" = "1" ] || echo "check-verification-pipefail: clean ($SCANNED task file(s) scanned, $WITH_BLOCK with a Verification block)"
    exit 0
fi

echo "check-verification-pipefail: $COUNT risky verification command(s) across $WITH_BLOCK task file(s) with a Verification block:"
printf '%s\n' "$OUT" | grep '^  RISK  '

if [ "$QUIET" != "1" ]; then
    echo ""
    echo "Each of these exits 141 when the pattern MATCHES (grep -q closes the pipe ->"
    echo "SIGPIPE -> pipefail propagates 141), so P-011 blocks a task whose verification"
    echo "actually passed. Safe rewrites:"
    echo "    out=\$(cmd 2>&1); echo \"\$out\" | grep -q \"PATTERN\""
    echo "    cmd > /tmp/.out 2>&1; grep -q \"PATTERN\" /tmp/.out"
    echo ""
    echo "Scope note: completed tasks are included by default because they are the best"
    echo "evidence of how widespread the shape is. Use --active-only once you are fixing."
fi

exit 1
