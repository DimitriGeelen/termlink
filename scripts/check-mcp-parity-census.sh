#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# check-mcp-parity-census.sh (T-2747, herdr rank 13 — coverage census for the MCP/CLI parity suite)
#
# WHY: `crates/termlink-mcp/tests/parity.rs` asserts that an MCP tool and its CLI verb
# produce the same structured output. It covers 24 tools. There are 260. The other 236 are
# not passing — they are UNEXAMINED, and nothing anywhere distinguished those two states.
#
# That is the T-2680 lesson one layer up. The charter-drift canary once reported
# `{checked:214, live_off_charter:0}`, which read as "all 214 tools trace to the charter"
# when it had only ever looked for six known families. A suite that covers 9.2% of a
# surface and says nothing about the rest is the same shape: its green is a statement
# about the 24, and it gets read as a statement about the 260.
#
# The guard layer had no member that knows the two surfaces are supposed to correspond
# (`ls scripts/ | grep -i parit` → nothing). This adds one.
#
# WHAT THIS DOES NOT DO — it does not prevent drift, and it does not verify that any two
# implementations agree. It answers exactly one question: is every MCP tool either
# ASSERTED by a parity case, or ACKNOWLEDGED as not asserted with a cited reason? It
# converts *unexamined* into *acknowledged*. Whether an asserted pair actually matches is
# what parity.rs itself tests; whether an acknowledged tool should be asserted is a human
# call recorded in the allowlist.
#
# The value is the ratchet: every currently-uncovered tool is enumerated in the allowlist,
# so a NEW MCP tool added tomorrow is in neither set and fires on the next run. Today's gap
# is frozen and visible; tomorrow's requires a decision.
#
# COUNTING NOTE (T-2747). Three different counts of "`*_mcp` parallel helpers" were in
# circulation — 83 (T-1904 census), 68 (parity.rs header as of T-2683/T-2689), and 94
# (herdr worker 3) — with none asserted correct. Measured here: 79 distinct `fn *_mcp`
# names. They all disagree because the unit is not well defined: `fn to_json_mcp` alone
# occurs 26 times as a small helper redefined inside separate functions, and counting it
# as 26 "parallel implementations" is as defensible as counting it as one. This check
# therefore does not count helpers at all. It counts TOOLS — `name = "termlink_…"` inside
# a `#[tool(…)]` macro — which is a well-defined unit with exactly one meaning.
#
# Comment lines are stripped on both sides, in both directions. tools.rs contains a
# `// \`name = "termlink_help"\` literal` inside a doc comment that would otherwise inflate
# the tool count, and parity.rs discusses tool names in prose that would otherwise mark
# them covered without asserting anything about them.
#
# ALLOWLIST: `.context/checks/mcp-parity-census-allowlist` (git-tracked per T-2681), one
# `<tool_name>  # <reason>` per line. Entries are COUNTED AND REPORTED in the census — they
# never vanish from the total — but do not fire. This is a ledger of an open question, in
# the T-2483/T-2548 charter-drift tradition, not a permanent exemption.
#
# NOT a runtime cron canary — a source-level static check, sibling of
# check-alloc-sink-clamps.sh (T-2527), check-drain-sink-caps.sh (T-2531),
# check-silent-exit.sh (T-2666), check-busy-spin.sh (T-2672), check-platform-lock.sh
# (T-2693), check-error-code-emission.sh (T-2699) and check-version-derivation.sh (T-2746).
#
# EXIT CODES:
#   0  clean    -- every MCP tool is asserted or acknowledged.
#   1  firing   -- >=1 tool is neither.
#   2  tooling  -- missing dep / source files not found.
#
# USAGE:
#   check-mcp-parity-census.sh [--json] [--quiet] [--no-heartbeat]
#                              [--tools-file <f>] [--parity-file <f>] [--allowlist <f>]
#     --json          emit {ok, firing:[…], total, covered, acknowledged, unexamined,
#                     coverage_pct, scope}
#     --quiet         print only on firing (cron mode)
#     --no-heartbeat  skip the heartbeat touch (guard-layer runner invokes with this)
#
# Origin: T-2747 (herdr rank 13). Load-bearing proof: tests/mcp-parity-census-fixtures.sh,
# and deleting a covered tool's reference from parity.rs re-fires the check on that tool.
set -uo pipefail

WANT_JSON=0 QUIET=0 HEARTBEAT=1
TOOLS_FILE="crates/termlink-mcp/src/tools.rs"
PARITY_FILE="crates/termlink-mcp/tests/parity.rs"
ALLOWLIST="${MCP_PARITY_ALLOWLIST:-.context/checks/mcp-parity-census-allowlist}"

while [ $# -gt 0 ]; do
    case "$1" in
        --json) WANT_JSON=1; shift ;;
        --quiet) QUIET=1; shift ;;
        --no-heartbeat) HEARTBEAT=0; shift ;;
        --tools-file) shift; [ $# -gt 0 ] || { echo "check-mcp-parity-census: --tools-file needs a value" >&2; exit 2; }; TOOLS_FILE="$1"; shift ;;
        --parity-file) shift; [ $# -gt 0 ] || { echo "check-mcp-parity-census: --parity-file needs a value" >&2; exit 2; }; PARITY_FILE="$1"; shift ;;
        --allowlist) shift; [ $# -gt 0 ] || { echo "check-mcp-parity-census: --allowlist needs a value" >&2; exit 2; }; ALLOWLIST="$1"; shift ;;
        -h|--help) sed -n '2,68p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "check-mcp-parity-census: unknown arg '$1'" >&2; exit 2 ;;
    esac
done

command -v grep >/dev/null 2>&1 || { echo "check-mcp-parity-census: grep not found (required)" >&2; exit 2; }
[ -f "$TOOLS_FILE" ]  || { echo "check-mcp-parity-census: tools file not found: $TOOLS_FILE" >&2; exit 2; }
[ -f "$PARITY_FILE" ] || { echo "check-mcp-parity-census: parity file not found: $PARITY_FILE" >&2; exit 2; }

HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.mcp-parity-census-canary.heartbeat}"
if [ "$HEARTBEAT" -eq 1 ]; then
    touch "$HEARTBEAT_FILE" 2>/dev/null || true
fi

# strip_comments <file> — drop whole-line // comments before matching. Prose ABOUT a tool
# name is not a declaration of it and not an assertion about it (both directions matter).
strip_comments() { grep -vE '^[[:space:]]*//' "$1" 2>/dev/null || true; }

# The declared tool surface: `name = "termlink_…"` inside a #[tool(…)] macro.
TOOLS="$(strip_comments "$TOOLS_FILE" \
    | grep -oE 'name = "termlink_[a-z0-9_]+"' \
    | sed -E 's/name = "(.*)"/\1/' | sort -u)"

# Tools an assertion actually names.
COVERED="$(strip_comments "$PARITY_FILE" \
    | grep -oE '"termlink_[a-z0-9_]+"' \
    | tr -d '"' | sort -u)"

declare -A ALLOW=()
ack_listed=0
if [ -f "$ALLOWLIST" ]; then
    while IFS= read -r line; do
        line="${line%%#*}"
        line="$(printf '%s' "$line" | tr -d '[:space:]')"
        [ -n "$line" ] || continue
        ALLOW["$line"]=1
        ack_listed=$((ack_listed + 1))
    done < "$ALLOWLIST"
fi

total="$(printf '%s' "$TOOLS" | grep -c . || true)"
[ "${total:-0}" -gt 0 ] || { echo "check-mcp-parity-census: no tools found in $TOOLS_FILE — refusing to report a clean census over an empty set" >&2; exit 2; }

covered_n=0 ack_n=0 unexamined=""
while IFS= read -r tool; do
    [ -n "$tool" ] || continue
    if printf '%s\n' "$COVERED" | grep -qx "$tool"; then
        covered_n=$((covered_n + 1))
    elif [ -n "${ALLOW[$tool]:-}" ]; then
        ack_n=$((ack_n + 1))
    else
        unexamined="${unexamined}${tool}"$'\n'
    fi
done <<< "$TOOLS"

unexamined_n="$(printf '%s' "$unexamined" | grep -c . || true)"
pct=$(( covered_n * 1000 / total ))
pct_str="$((pct / 10)).$((pct % 10))"

emit_json() { # <ok> <firing-json>
    printf '{"ok":%s,"firing":%s,"total":%d,"covered":%d,"acknowledged":%d,"unexamined":%d,"coverage_pct":"%s","scope":"asserted-or-acknowledged; does NOT verify that an asserted pair agrees"}\n' \
        "$1" "$2" "$total" "$covered_n" "$ack_n" "$unexamined_n" "$pct_str"
}

firing_json='[]'
if [ "${unexamined_n:-0}" -gt 0 ] && command -v jq >/dev/null 2>&1; then
    firing_json="$(printf '%s' "$unexamined" | grep . | jq -Rsc 'split("\n") | map(select(length>0))' 2>/dev/null || echo '[]')"
fi

if [ "${unexamined_n:-0}" -eq 0 ]; then
    if [ "$WANT_JSON" -eq 1 ]; then
        emit_json true '[]'
    elif [ "$QUIET" -eq 0 ]; then
        echo "check-mcp-parity-census: clean — $total MCP tool(s): $covered_n asserted by parity.rs, $ack_n acknowledged, 0 unexamined."
        echo "  Parity coverage is ${pct_str}% — this check proves each tool is asserted OR acknowledged,"
        echo "  NOT that the two implementations agree (that is what parity.rs itself tests)."
    fi
    exit 0
fi

if [ "$WANT_JSON" -eq 1 ]; then
    emit_json false "$firing_json"
else
    echo "=== $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
    echo "check-mcp-parity-census: FIRING — $unexamined_n of $total MCP tool(s) are UNEXAMINED"
    echo "  (asserted by parity.rs: $covered_n = ${pct_str}%   acknowledged: $ack_n   unexamined: $unexamined_n)"
    printf '%s' "$unexamined" | grep . | head -40 | sed 's/^/  ↳ /'
    [ "$unexamined_n" -gt 40 ] && echo "  … and $((unexamined_n - 40)) more (use --json for the full list)"
    echo "  Unexamined is NOT the same as passing: nothing has ever compared these tools"
    echo "  against their CLI verbs. Fix: add a parity case, OR record the tool in"
    echo "  $ALLOWLIST with a reason saying why it is not asserted."
    echo "---"
fi
exit 1
