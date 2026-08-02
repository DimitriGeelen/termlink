#!/usr/bin/env bash
# check-charter-sentence-drift.sh (T-2484, G-019 charter-fork prevention)
#
# WHY: P1 (T-2470) shipped docs/CHARTER.md as "the single owned statement of what
# TermLink is". Its header CLAIMS "README and docs/ARCHITECTURE.md both quote the
# canonical sentence below; edit it here and the docs follow by reference." That
# claim is mechanically FALSE — markdown has no transclusion, so the canonical
# purpose sentence exists as THREE independent copies:
#   docs/CHARTER.md      (bold **…**, line-wrapped)
#   README.md            (plain single line)
#   docs/ARCHITECTURE.md (blockquote "> …")
# Nothing verified they stay identical. A human editing the sentence in CHARTER.md
# exactly as the charter instructs would silently strand two stale copies — the
# charter, anchor of the whole T-2468 arc, was not load-bearing for its own
# consistency claim (G-019: fix why the framework was blind, not just the instance).
#
# This is the 13th canary (empty log = healthy). It extracts the canonical sentence
# from all three files, NORMALIZES away the per-file decorations (bold markers,
# blockquote prefixes, line-wrapping / whitespace), and FIRES if the three do not
# agree. Sibling of the P8 charter-drift canary (T-2483): P8 guards the TOOL SURFACE
# against off-charter breadth; THIS guards the CHARTER SENTENCE against its three
# copies forking. Mirrors the doc-set-drift idiom (check-preflight-doc-set-drift.sh,
# T-2188).
#
# EXTRACTION: the sentence is bounded by a unique anchor phrase and a terminal word.
# We deliberately extract by anchor rather than hardcoding the full sentence — a
# hardcoded copy would just be a 4th fork to drift. Truncating at the FIRST
# "machines" after the anchor (pure bash param-expansion, no PCRE dependency) avoids
# the greedy-match trap: README carries a SECOND sentence that also ends in
# "…machines." so a greedy `.*machines\.` would over-capture.
#
# EXIT CODES:
#   0  in-sync    -- all three files carry an identical (normalized) sentence.
#   1  drift      -- >=1 file's sentence diverges (diagnostic table on stderr).
#   2  tooling    -- a source file is missing or the sentence can't be extracted
#                    (fail-closed — never a false "in-sync").
#
# USAGE:
#   check-charter-sentence-drift.sh [--quiet] [--no-heartbeat] [--help]
#     --quiet         print only on firing (cron mode); in-sync prints nothing
#     --no-heartbeat  skip the heartbeat touch (meta-canary invokes with this)
#
# TEST HOOK (PL-213 -- host-independent):
#   CHARTER_SENTENCE_REPO_ROOT=<dir>   read the three files from <dir> instead of the
#                                      real repo (the test harness stages fixtures).
#
# Origin: T-2468 purpose review (6th re-issue) -> this build. /canaries auto-discovers
# the log. Pair with the 12 canaries in CLAUDE.md — all follow "empty-log = healthy".
set -u

QUIET=0
HEARTBEAT=1
for arg in "$@"; do
    case "$arg" in
        --quiet) QUIET=1 ;;
        --no-heartbeat) HEARTBEAT=0 ;;
        -h|--help)
            sed -n '2,46p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
            exit 0 ;;
        *) echo "check-charter-sentence-drift: unknown arg '$arg'" >&2; exit 2 ;;
    esac
done

# T-1723 heartbeat: prove this canary ran, even on drift/error cycles. Placed BEFORE
# any extraction so a missing surface can't silently swallow the heartbeat.
HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.charter-sentence-drift-canary.heartbeat}"
if [ "$HEARTBEAT" = 1 ]; then
    mkdir -p "$(dirname "$HEARTBEAT_FILE")" 2>/dev/null || true
    touch -- "$HEARTBEAT_FILE" 2>/dev/null || true
fi

ROOT="${CHARTER_SENTENCE_REPO_ROOT:-.}"
ANCHOR="TermLink is a hub-mediated, durable append-log message bus"

# The three surfaces carrying the canonical sentence.
SURFACES=(
    "charter:docs/CHARTER.md"
    "readme:README.md"
    "architecture:docs/ARCHITECTURE.md"
)

# Extract + normalize the canonical sentence from one file.
#   __MISSING__  file unreadable
#   __NOMATCH__  anchor or terminal word absent (can't extract)
extract_sentence() {
    local path="$1" flat after before
    [ -r "$path" ] || { printf '__MISSING__'; return; }
    # Strip bold markers (**), strip leading blockquote prefix ("> " or ">"),
    # join lines, collapse runs of whitespace to one space.
    flat=$(sed -e 's/\*\*//g' -e 's/^> \{0,1\}//' "$path" 2>/dev/null | tr '\n' ' ' | tr -s ' ')
    case "$flat" in
        *"$ANCHOR"*) ;;
        *) printf '__NOMATCH__'; return ;;
    esac
    after="${flat#*"$ANCHOR"}"          # everything after the (unique) anchor
    # Must contain a "machines" terminus after the anchor.
    case "$after" in
        *machines*) ;;
        *) printf '__NOMATCH__'; return ;;
    esac
    before="${after%%machines*}"        # text up to the FIRST "machines"
    printf '%s' "${ANCHOR}${before}machines."
}

declare -a NAMES
declare -A SENT
errors=0
for entry in "${SURFACES[@]}"; do
    name="${entry%%:*}"; rel="${entry#*:}"
    s=$(extract_sentence "$ROOT/$rel")
    NAMES+=("$name")
    SENT[$name]="$s"
    case "$s" in
        __MISSING__|__NOMATCH__) errors=$((errors+1)) ;;
    esac
done

if [ "$errors" -gt 0 ]; then
    echo "check-charter-sentence-drift: tooling error — could not extract the sentence from one or more surfaces (fail-closed)" >&2
    for name in "${NAMES[@]}"; do
        printf '  %-13s %s\n' "$name" "${SENT[$name]}" >&2
    done
    exit 2
fi

# Unanimity: all normalized sentences must equal the first.
first="${SENT[${NAMES[0]}]}"
drift=0
for name in "${NAMES[@]}"; do
    [ "${SENT[$name]}" = "$first" ] || { drift=1; break; }
done

if [ "$drift" -eq 0 ]; then
    [ "$QUIET" -eq 1 ] || echo "check-charter-sentence-drift: in-sync — all ${#NAMES[@]} surfaces carry the identical canonical sentence."
    exit 0
fi

# firing — framed log entry (stdout) + diagnostic table (stderr)
echo "=== $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
echo "check-charter-sentence-drift: FIRING — the canonical purpose sentence has drifted across surfaces:"
for name in "${NAMES[@]}"; do
    marker=""
    [ "${SENT[$name]}" = "$first" ] || marker="  <-- DRIFT"
    printf '  %-13s %s%s\n' "$name" "${SENT[$name]}" "$marker"
done
echo "  docs/CHARTER.md is authoritative (human-blessed). Bring README.md + docs/ARCHITECTURE.md"
echo "  back into agreement with it. This is the P1 'single owned statement' forking — see docs/CHARTER.md."
echo "---"
exit 1
