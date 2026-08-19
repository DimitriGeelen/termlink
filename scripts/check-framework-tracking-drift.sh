#!/usr/bin/env bash
# check-framework-tracking-drift.sh — T-2689.
#
# Reports files present under the vendored framework directory that are UNREACHABLE
# FROM GIT, i.e. present on this host's disk but absent from version control.
#
# Why this exists
# ---------------
# `.gitignore` carries a blanket `.agentic-framework` rule, added when that path was a
# machine-specific symlink. The framework is now VENDORED, and ~1565 files under it are
# tracked (including `bin/fw`). Ignore rules do not apply to already-tracked files, so
# those keep working and nothing looks broken — but every framework file added AFTER the
# rule landed is silently untrackable: `git add -A` skips it and `git status` never
# mentions it, because ignored files are not reported.
#
# The result is invisible drift between the framework that runs and the framework that
# is recoverable. The casualty that surfaced this: `lib/bvp.sh` and the whole `policy/`
# directory (`value-drivers.yaml`, `bvp-scoring-rubric.md`) are untracked, so a clean
# clone has a tracked `bin/fw` that routes `fw bvp` to a library that is not there — and
# the §ACD sovereignty-gated driver weights exist only on one host's disk.
#
# This is a DEPLOY-TIME / ad-hoc check, not a cron canary — the same tier as
# `check-cron-install-drift.sh` (T-2561). Register it in
# `.context/cron/ondemand-checks.conf` if it ever grows a heartbeat.
#
# Firing policy
# -------------
#   FIRES (exit 1)  — an untracked file under a LOAD-BEARING subtree: bin/ lib/ policy/
#                     agents/. These are what break a clean clone.
#   informational   — untracked files under docs/ or web/ (bulk content; a missing doc
#                     is a gap, not a broken install). Counted, never firing.
#   excluded        — __pycache__/, *.pyc, *.pyo, .git/ (generated; tracking them would
#                     swamp the signal).
#
# Usage:
#   bash scripts/check-framework-tracking-drift.sh [--json] [--quiet] [--root DIR]
#
# Exit codes:
#   0 — no load-bearing drift (informational drift may still be reported)
#   1 — at least one untracked load-bearing file
#   2 — tooling error (not a git repo, framework dir absent)
#
# Test seam (PL-213): --root points the scan at a fixture tree; the git query is run
# from that tree's repository, so fixtures need only be a git repo with some files.

set -uo pipefail

FW_ROOT=".agentic-framework"
JSON=0
QUIET=0

usage() {
    sed -n '2,/^set -uo pipefail$/p' "$0" | sed 's/^# \{0,1\}//' | head -n -2
    exit 0
}

while [ $# -gt 0 ]; do
    case "$1" in
        --json) JSON=1 ;;
        --quiet) QUIET=1 ;;
        --root)
            shift
            [ $# -ge 1 ] || { echo "check-framework-tracking-drift: --root requires a value" >&2; exit 2; }
            FW_ROOT="$1"
            ;;
        -h|--help) usage ;;
        *) echo "check-framework-tracking-drift: unknown flag: $1" >&2; exit 2 ;;
    esac
    shift
done

if [ ! -d "$FW_ROOT" ]; then
    echo "check-framework-tracking-drift: framework dir not found: $FW_ROOT" >&2
    exit 2
fi

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "check-framework-tracking-drift: not inside a git work tree" >&2
    exit 2
fi

# Load-bearing subtrees: an untracked file here breaks a clean clone.
is_load_bearing() {
    case "$1" in
        "$FW_ROOT"/bin/*|"$FW_ROOT"/lib/*|"$FW_ROOT"/policy/*|"$FW_ROOT"/agents/*) return 0 ;;
        *) return 1 ;;
    esac
}

# Tracked paths as an associative-array set.
#
# Deliberately NOT `printf '%s\n' "$list" | grep -qxF "$f"`: under `pipefail`, grep -q
# exits on first match and closes the pipe while printf is still writing, so the
# PIPELINE status is 141 (SIGPIPE) and the membership test reports "absent" for paths
# that are present. That is this repo's own L-387 footgun, and it inverted this check's
# result — every tracked file was reported UNTRACKED. The array lookup has no pipeline,
# no subprocess per file, and turns an O(files x tracked) scan into O(files).
declare -A TRACKED
while IFS= read -r t; do
    [ -n "$t" ] || continue
    TRACKED["$t"]=1
done < <(git ls-files "$FW_ROOT" 2>/dev/null || true)

FIRING=""
FIRING_COUNT=0
INFO_COUNT=0
CHECKED=0

while IFS= read -r f; do
    [ -n "$f" ] || continue
    CHECKED=$((CHECKED + 1))
    if [ -n "${TRACKED[$f]:-}" ]; then
        continue
    fi
    if is_load_bearing "$f"; then
        FIRING="${FIRING}${f}"$'\n'
        FIRING_COUNT=$((FIRING_COUNT + 1))
    else
        INFO_COUNT=$((INFO_COUNT + 1))
    fi
done <<EOF
$(find "$FW_ROOT" -type f \
    -not -path '*/__pycache__/*' \
    -not -name '*.pyc' \
    -not -name '*.pyo' \
    -not -path '*/.git/*' \
    2>/dev/null | sort)
EOF

if [ "$JSON" = "1" ]; then
    printf '{"ok":%s,"checked":%d,"firing_count":%d,"informational_count":%d,"firing":[' \
        "$([ "$FIRING_COUNT" -eq 0 ] && echo true || echo false)" \
        "$CHECKED" "$FIRING_COUNT" "$INFO_COUNT"
    first=1
    while IFS= read -r f; do
        [ -n "$f" ] || continue
        [ "$first" = "1" ] || printf ','
        first=0
        printf '"%s"' "$f"
    done <<EOF
$FIRING
EOF
    printf ']}\n'
    [ "$FIRING_COUNT" -eq 0 ] && exit 0 || exit 1
fi

if [ "$FIRING_COUNT" -eq 0 ]; then
    if [ "$QUIET" != "1" ]; then
        echo "check-framework-tracking-drift: no load-bearing drift ($CHECKED file(s) scanned, $INFO_COUNT informational)"
        if [ "$INFO_COUNT" -gt 0 ]; then
            echo "  ($INFO_COUNT untracked file(s) under docs/ or web/ — content drift, not clean-clone-breaking)"
        fi
    fi
    exit 0
fi

echo "check-framework-tracking-drift: $FIRING_COUNT untracked load-bearing file(s) — a clean clone would be missing them:"
while IFS= read -r f; do
    [ -n "$f" ] || continue
    printf '  UNTRACKED  %s\n' "$f"
done <<EOF
$FIRING
EOF

if [ "$QUIET" != "1" ]; then
    echo ""
    echo "  $INFO_COUNT further untracked file(s) under docs/ or web/ (informational)."
    echo ""
    echo "Remediation:"
    echo "  The blanket \`.agentic-framework\` rule in .gitignore predates vendoring — it is"
    echo "  labelled \"Framework symlink (machine-specific)\" but the tree is vendored and"
    echo "  largely tracked. Narrow that rule, then add the load-bearing files:"
    echo "    git add -f $FW_ROOT/bin $FW_ROOT/lib $FW_ROOT/policy $FW_ROOT/agents"
    echo "  Review before committing — confirm nothing machine-local or secret-bearing is"
    echo "  swept in (host paths, tokens, per-machine config)."
fi

exit 1
