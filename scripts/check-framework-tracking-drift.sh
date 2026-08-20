#!/usr/bin/env bash
# guard-layer: source
# check-framework-tracking-drift.sh — T-2814.
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
# `check-cron-install-drift.sh` (T-2561). It carries the `# guard-layer: source`
# marker above (T-2802), so `run-guard-layer.sh` executes it: git plus the vendored
# framework dir are both in-tree, so it is safe to run anywhere. An earlier draft
# pointed at `.context/cron/ondemand-checks.conf`, a registry removed in T-2822 when
# the canary-status work was yielded to governance-canary-signal — their
# `crontab_declares` derives the same fact from the crontabs themselves.
#
# Two detection axes (T-2817)
# ---------------------------
# The two are complementary because each is blind where the other fires:
#
#   A. UNTRACKED  — a file is on disk but absent from git. Visible only in the checkout
#                   where the untracked file physically lives (i.e. the machine that
#                   created it).
#   B. DANGLING   — tracked framework code references "$FRAMEWORK_ROOT/<path>" and that
#                   path does not exist. Visible only where the file is MISSING — a clean
#                   clone, a fresh deploy, or a git worktree (which materialises tracked
#                   files only).
#
# Axis A alone reports a worktree as clean while `fw bvp` is broken in it, because the
# untracked lib/bvp.sh simply is not there to be noticed. That is not hypothetical: it is
# how T-2817 was found. Axis B catches the same defect from the consumer side, and is the
# axis that matters for anyone who did not create the drift.
#
# Firing policy
# -------------
#   FIRES (exit 1)  — an untracked file under a LOAD-BEARING subtree: bin/ lib/ policy/
#                     agents/. These are what break a clean clone.       [axis A]
#   FIRES (exit 1)  — a "$FRAMEWORK_ROOT/<path>" reference in tracked framework code that
#                     does not resolve on disk.                          [axis B]
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
        # T-2811: web/ was originally lumped with docs/ as "informational", on the
        # reasoning that a missing page is a gap rather than a broken install. That
        # is true of docs/ and it is false of most of web/, which this checker was
        # asked to prove the hard way: it reported this worktree CLEAN while
        # `fw serve` could not start in it.
        #
        #   web/**/*.py       — Flask registers every blueprint in create_app(), so
        #                       a missing module is ModuleNotFoundError before the
        #                       app binds a port. Not a degraded page: no app.
        #   web/templates/**  — base.html `{% include %}`s partials, so ONE missing
        #                       template is HTTP 500 on every route that extends it.
        #                       Found via `TemplateNotFound: _pins.html`, which took
        #                       Watchtower from "won't start" to "starts, serves 500".
        #
        # web/static/** stays informational, and that line is drawn from the same
        # evidence rather than from taste: a missing font or stylesheet renders an
        # ugly page, not a broken one. Promoting it too would fire on cosmetics, and
        # a check that fires on cosmetics is one people learn to skip — which is how
        # the original over-broad "informational" call survived until it cost the
        # whole app.
        "$FW_ROOT"/web/static/*) return 1 ;;
        "$FW_ROOT"/web/templates/*) return 0 ;;
        "$FW_ROOT"/web/*.py|"$FW_ROOT"/web/*/*.py|"$FW_ROOT"/web/*/*/*.py) return 0 ;;
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

# --- Axis B: dangling "$FRAMEWORK_ROOT/<path>" references (T-2817) -----------
#
# Scan shell sources under the framework root for the literal interpolation shape and
# test each resolved path. A reference carrying a FURTHER interpolation (a `${` after the
# leading $FRAMEWORK_ROOT) cannot be resolved statically, so it is skipped and counted
# rather than guessed — a false positive in a check that gates nothing just teaches people
# to ignore it.
DANGLING=""
DANGLING_COUNT=0
DANGLING_SKIPPED=0
REFS_CHECKED=0

# The anchor is deliberately NARROW: only a reference in SOURCE-or-EXECUTE position
# counts. A first pass matched every "$FRAMEWORK_ROOT/..." string and produced 47 hits of
# which ~44 were noise — bare $VAR interpolations, `path/to/script.sh` usage examples in
# help text, and the framework's own tests/ and .git/ which a vendored copy legitimately
# omits. Those are not broken installs, and a check that cries wolf 44 times out of 47 is
# a check nobody reads.
#
# A reference that is sourced or executed, by contrast, is load-bearing by construction:
# if it is missing the command fails at runtime. That is exactly the `. "$FRAMEWORK_ROOT/
# lib/bvp.sh"` shape that broke `fw bvp` here. Same philosophy as the sibling static
# checks (T-2666 anchors on a precise preceding-line shape, T-2672 on specific RPC method
# strings) — a narrow anchor with few false positives beats a broad one nobody trusts.
while IFS= read -r rel; do
    [ -n "$rel" ] || continue
    # A path still carrying a shell interpolation cannot be resolved statically.
    # Nor can a documentation placeholder: `lib/<name>.py` in a usage string is a
    # template for the reader to fill in, not a file anyone expects to exist.
    # T-2807 surfaced this — recovering 92 files brought in help text containing
    # `. "$FRAMEWORK_ROOT/lib/<name>.py"`, which the source-position anchor
    # matched correctly and which is still not a real reference. Both are
    # "cannot be resolved statically", so both are skipped and counted rather
    # than guessed; the alternative is one permanent false positive, and a check
    # that is never clean is a check nobody reads.
    case "$rel" in
        *'$'*|*'<'*|*'>'*)
            DANGLING_SKIPPED=$((DANGLING_SKIPPED + 1))
            continue
            ;;
    esac
    REFS_CHECKED=$((REFS_CHECKED + 1))
    [ -e "$FW_ROOT/$rel" ] && continue
    case "$DANGLING" in
        *"$rel"$'\n'*) continue ;;   # already reported
    esac
    DANGLING="${DANGLING}${rel}"$'\n'
    DANGLING_COUNT=$((DANGLING_COUNT + 1))
done <<EOF
$(grep -rhE '(^|[;&|]|[[:space:]])(\.|source|bash|sh|python3|python)[[:space:]]+"\$FRAMEWORK_ROOT/[^"]*"' \
    "$FW_ROOT/bin" "$FW_ROOT/lib" "$FW_ROOT/agents" 2>/dev/null \
  | grep -vE '^[[:space:]]*#' \
  | grep -oE '"\$FRAMEWORK_ROOT/[^"]*"' \
  | sed -e 's|^"\$FRAMEWORK_ROOT/||' -e 's|"$||' \
  | sort -u)
EOF

TOTAL_FIRING=$((FIRING_COUNT + DANGLING_COUNT))

if [ "$JSON" = "1" ]; then
    printf '{"ok":%s,"checked":%d,"firing_count":%d,"informational_count":%d,"firing":[' \
        "$([ "$TOTAL_FIRING" -eq 0 ] && echo true || echo false)" \
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
    printf '],"refs_checked":%d,"refs_skipped_dynamic":%d,"dangling_count":%d,"dangling":[' \
        "$REFS_CHECKED" "$DANGLING_SKIPPED" "$DANGLING_COUNT"
    first=1
    while IFS= read -r r; do
        [ -n "$r" ] || continue
        [ "$first" = "1" ] || printf ','
        first=0
        printf '"%s"' "$r"
    done <<EOF
$DANGLING
EOF
    printf ']}\n'
    [ "$TOTAL_FIRING" -eq 0 ] && exit 0 || exit 1
fi

if [ "$TOTAL_FIRING" -eq 0 ]; then
    if [ "$QUIET" != "1" ]; then
        echo "check-framework-tracking-drift: no load-bearing drift ($CHECKED file(s) scanned, $REFS_CHECKED reference(s) resolved, $INFO_COUNT informational)"
        if [ "$INFO_COUNT" -gt 0 ]; then
            echo "  ($INFO_COUNT untracked file(s) under docs/ or web/ — content drift, not clean-clone-breaking)"
        fi
        if [ "$DANGLING_SKIPPED" -gt 0 ]; then
            echo "  ($DANGLING_SKIPPED dynamic reference(s) skipped — not statically resolvable)"
        fi
    fi
    exit 0
fi

if [ "$FIRING_COUNT" -gt 0 ]; then
    echo "check-framework-tracking-drift: $FIRING_COUNT untracked load-bearing file(s) — a clean clone would be missing them:"
    while IFS= read -r f; do
        [ -n "$f" ] || continue
        printf '  UNTRACKED  %s\n' "$f"
    done <<EOF
$FIRING
EOF
fi

if [ "$DANGLING_COUNT" -gt 0 ]; then
    echo "check-framework-tracking-drift: $DANGLING_COUNT dangling reference(s) — tracked code points at paths that are not here:"
    while IFS= read -r r; do
        [ -n "$r" ] || continue
        printf '  DANGLING   %s\n' "\$FRAMEWORK_ROOT/$r"
    done <<EOF
$DANGLING
EOF
fi

if [ "$QUIET" != "1" ]; then
    echo ""
    echo "  $INFO_COUNT further untracked file(s) under docs/ or web/ (informational)."
    if [ "$DANGLING_SKIPPED" -gt 0 ]; then
        echo "  $DANGLING_SKIPPED dynamic reference(s) skipped (not statically resolvable)."
    fi
    echo ""
    echo "Remediation:"
    if [ "$FIRING_COUNT" -gt 0 ]; then
        echo "  UNTRACKED — the blanket \`.agentic-framework\` rule in .gitignore predates"
        echo "  vendoring: it is labelled \"Framework symlink (machine-specific)\" but the tree"
        echo "  is vendored and largely tracked. Narrow that rule, then add the files:"
        echo "    git add -f $FW_ROOT/bin $FW_ROOT/lib $FW_ROOT/policy $FW_ROOT/agents"
        echo "  Review before committing — confirm nothing machine-local or secret-bearing is"
        echo "  swept in (host paths, tokens, per-machine config)."
    fi
    if [ "$DANGLING_COUNT" -gt 0 ]; then
        echo "  DANGLING — this checkout is MISSING files that tracked framework code needs."
        echo "  You cannot fix it here by adding files: they were never committed, so there is"
        echo "  nothing to pull. Fix it in the checkout that still HAS them (run this check"
        echo "  there; it will report them as UNTRACKED), commit them, then update this one."
    fi
fi

exit 1
