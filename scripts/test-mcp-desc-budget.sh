#!/usr/bin/env bash
# test-mcp-desc-budget.sh (arc-005 mcp-slimming, T-2406) — anti-regrowth guard for
# MCP tool-description bloat.
#
# Every tool `description = "..."` string in crates/termlink-mcp/src/tools.rs is
# loaded into EVERY agent's context each session. Left ungoverned it creeps back
# (task-ID archaeology, PL cross-refs, param restatement). This guard reports the
# current budget and FAILS if any single description exceeds MAX_DESC_CEILING or the
# total exceeds TOTAL_DESC_CEILING — so `cargo test`/CI catches a regrowth.
#
# Ceilings start GENEROUS (pass at today's baseline) and are TIGHTENED as the
# mcp-slimming slices land (S1 worst-offenders → S2 600-1000 band → S3 long-tail).
# Lower them in lockstep with each slice's trims; a slice that trims but does not
# tighten the ceiling has not locked its win.
#
# Usage: test-mcp-desc-budget.sh [--report-only]
# Env:   MAX_DESC_CEILING (default 1560), TOTAL_DESC_CEILING (default 113000),
#        MCP_TOOLS_FILE (default crates/termlink-mcp/src/tools.rs)
#
# Ceiling history (each slice tightens after its trims land):
#   baseline (pre-S1): max 11751, total 156525  → ceilings 12000 / 160000
#   S1 (T-2406):       max  1546, total 133220  → ceilings  1600 / 135000
#   S2 (T-2407):       max  1546, total 112319  → ceilings  1560 / 113000
#                      (max unchanged — the 1546-char termlink_help tool is out of
#                       band; S2 trimmed the 600–999 band, reclaiming ~20.9KB total)

set -u
SELF_DIR="$(cd "$(dirname "$0")" && pwd)"
FILE="${MCP_TOOLS_FILE:-${SELF_DIR}/../crates/termlink-mcp/src/tools.rs}"
MAX_CEIL="${MAX_DESC_CEILING:-1560}"
TOTAL_CEIL="${TOTAL_DESC_CEILING:-113000}"
report_only=0; [ "${1:-}" = "--report-only" ] && report_only=1

[ -f "$FILE" ] || { echo "FATAL: MCP tools file not found: $FILE"; exit 2; }

# Measure with python (robust to escaped quotes inside descriptions).
read -r COUNT TOTAL MAX MAXLINE_DESC < <(python3 - "$FILE" <<'PY'
import re,sys
src=open(sys.argv[1],encoding='utf-8',errors='replace').read()
descs=re.findall(r'description\s*=\s*"((?:[^"\\]|\\.)*)"', src)
lens=[len(d) for d in descs]
n=len(lens); tot=sum(lens); mx=max(lens) if lens else 0
# a short label of the longest description (first 40 chars, spaces->_)
longest=max(descs,key=len) if descs else ""
lbl=re.sub(r'\s+','_',longest[:40]) or "-"
print(n, tot, mx, lbl)
PY
)

est_tokens=$(( TOTAL / 4 ))
echo "MCP tool-description budget (${FILE##*/}):"
echo "  tools:        $COUNT"
echo "  total bytes:  $TOTAL  (~${est_tokens} tokens, loaded per agent per session)"
echo "  max single:   $MAX   (ceiling $MAX_CEIL)   longest~ ${MAXLINE_DESC}"
echo "  total ceiling: $TOTAL_CEIL"

[ "$report_only" = "1" ] && exit 0

fail=0
if [ "$MAX" -gt "$MAX_CEIL" ]; then
    echo "FAIL: a tool description ($MAX chars) exceeds MAX_DESC_CEILING ($MAX_CEIL)."
    echo "      Trim it per docs/operations/mcp-description-policy.md, or (if intentional) raise the ceiling."
    fail=1
fi
if [ "$TOTAL" -gt "$TOTAL_CEIL" ]; then
    echo "FAIL: total description bytes ($TOTAL) exceed TOTAL_DESC_CEILING ($TOTAL_CEIL)."
    fail=1
fi

if [ "$fail" -eq 0 ]; then echo "RESULT: PASS (within budget)"; else echo "RESULT: FAIL"; exit 1; fi
