#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
#
# check-verification-heading-shadow (T-2877) — does the P-011 gate actually see
# this task's verification commands?
#
# `extract_verification_block` takes the FIRST `^## Verification` match. A line
# at column 0 that merely LOOKS like that heading therefore shadows the real one,
# and the gate runs whatever follows the counterfeit. T-2873 carried an orphaned
# template block — a stray `-->` with NO opening `<!--` anywhere above it, whose
# wrapped fragment left "## Verification` instead of a Human AC here..." at column
# 0, twenty-five lines above the genuine heading. The gate extracted template
# prose ("1. Open https://example.com/dashboard in browser") and the task's five
# real verification commands were unreachable.
#
# Both failure directions are bad and neither is loud: the prose either evals as
# bash (the T-2990/T-2991 hazard) or is refused as unparseable, reporting a
# nonsense cause. "No commands to run" and "all commands passed" are the same
# output — the T-2831 lesson, one layer further in.
#
# WHAT IT ASSERTS: that what the gate will EXECUTE is command-shaped rather than
# markdown prose. It calls the framework's own `extract_verification_block`
# rather than reimplementing the heading rule — two copies of a subtle rule drift,
# and the copy that drifts is the one that quietly stops catching things (T-2818;
# `update-task.sh`'s own T-2921 comment makes the same argument about its former
# inline copy).
#
# SCOPE — read a green narrowly (T-2680). It detects a COUNTERFEIT/SHADOWED
# heading. It does NOT verify that a task's verification is adequate, that its
# commands test its acceptance criteria, or that a task has any verification at
# all: a task with an empty `## Verification` passes this check and still gates on
# nothing. That gap is stated on every output path deliberately.
#
# WHY THIS PREDICATE, measured over 2596 task files before it was written:
#   - ">1 occurrence of ^## Verification" fires on 23 files, of which 22 extract
#     correctly (their first heading IS the genuine one). Wrong 22 times in 23.
#   - "an orphaned --> with no opener" fires on 14 files, of which 1 mis-extracts.
#   - "the extracted block contains markdown prose" fires on exactly 1 file, and
#     it is a true positive. Zero false positives corpus-wide.
# A guard that is wrong 22 times out of 23 is the fatigue shape T-2818 documented
# from the other direction, so the first two are reported as diagnosis on an
# already-firing file and never as the firing gate itself.
#
# Exit 0 clean / 1 firing / 2 tooling. Fail-closed: a missing tasks dir, an
# unsourceable extractor, or a corpus of zero task files exit 2, never 0.

set -uo pipefail

TASKS_DIR="${TASKS_DIR_OVERRIDE:-}"
ALLOWLIST=""
JSON=0
QUIET=0

usage() {
  sed -n '2,48p' "$0"
  exit 0
}

while [ $# -gt 0 ]; do
  case "$1" in
    --tasks-dir) TASKS_DIR="${2:-}"; shift 2 ;;
    --allowlist) ALLOWLIST="${2:-}"; shift 2 ;;
    --json)      JSON=1; shift ;;
    --quiet)     QUIET=1; shift ;;
    --no-heartbeat) shift ;;
    -h|--help)   usage ;;
    *) echo "check-verification-heading-shadow: unknown argument: $1" >&2; exit 2 ;;
  esac
done

REPO_ROOT="${REPO_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
[ -n "$TASKS_DIR" ] || TASKS_DIR="$REPO_ROOT/.tasks"
[ -n "$ALLOWLIST" ] || ALLOWLIST="$REPO_ROOT/.context/checks/verification-heading-shadow-allowlist"

die_tooling() {
  if [ "$JSON" = "1" ]; then
    printf '{"ok": false, "error": "tooling", "detail": "%s"}\n' "$1"
  else
    echo "check-verification-heading-shadow: TOOLING ERROR — $1" >&2
  fi
  exit 2
}

[ -d "$TASKS_DIR" ] || die_tooling "tasks dir not found: $TASKS_DIR"

# Source the framework's own extractor. Never reimplement the heading rule.
EXTRACTOR="${VERIFICATION_PORT_LIB:-$REPO_ROOT/.agentic-framework/lib/verification-port.sh}"
[ -f "$EXTRACTOR" ] || die_tooling "extractor not found: $EXTRACTOR (cannot assert what the gate would run)"
# shellcheck disable=SC1090
source "$EXTRACTOR" 2>/dev/null || die_tooling "could not source extractor: $EXTRACTOR"
declare -F extract_verification_block >/dev/null 2>&1 \
  || die_tooling "extract_verification_block not defined after sourcing $EXTRACTOR"

# Allowlist: bare filename, repo-relative path, or absolute path all match.
declare -A ACK=()
ACK_COUNT=0
if [ -f "$ALLOWLIST" ]; then
  while IFS= read -r line; do
    line="${line%%#*}"
    line="$(printf '%s' "$line" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
    [ -z "$line" ] && continue
    ACK["$(basename "$line")"]=1
    ACK_COUNT=$((ACK_COUNT + 1))
  done < "$ALLOWLIST"
fi

# Markdown-prose shapes that can never be a shell command. Measured: zero
# false positives across 2596 task files.
PROSE_RE='^[[:space:]]*[0-9]+\.[[:space:]]|^[[:space:]]*-[[:space:]]\[[ x]\]|^[[:space:]]*\*\*[A-Za-z]'

scanned=0
with_block=0
firing=0
acked=0
FIRING_LINES=""
JSON_ROWS=""

shopt -s nullglob
FILES=("$TASKS_DIR"/active/*.md "$TASKS_DIR"/completed/*.md)
shopt -u nullglob

[ "${#FILES[@]}" -gt 0 ] || die_tooling "no task files found under $TASKS_DIR (a zero-file corpus is never a clean census)"

for f in "${FILES[@]}"; do
  scanned=$((scanned + 1))
  blk="$(extract_verification_block "$f" 2>/dev/null)" || continue
  [ -z "$blk" ] && continue
  with_block=$((with_block + 1))
  bad="$(printf '%s\n' "$blk" | grep -nE "$PROSE_RE" || true)"
  [ -z "$bad" ] && continue

  base="$(basename "$f")"
  if [ -n "${ACK[$base]:-}" ]; then
    acked=$((acked + 1))
    continue
  fi
  firing=$((firing + 1))

  # Diagnosis (never the firing gate — see the header note on over-fire rates).
  dupes=$(grep -c '^## Verification' "$f" || true)
  orphan="no"
  depth=0
  while IFS= read -r ln; do
    o=$(printf '%s' "$ln" | grep -o -- '<!--' | wc -l)
    c=$(printf '%s' "$ln" | grep -o -- '-->' | wc -l)
    depth=$((depth + o - c))
    if [ "$depth" -lt 0 ]; then orphan="yes"; depth=0; fi
  done < "$f"

  first_bad="$(printf '%s\n' "$bad" | head -1)"
  FIRING_LINES="${FIRING_LINES}  ${base}
      the gate would execute markdown prose, not commands
      first offending extracted line: ${first_bad}
      diagnosis: '^## Verification' headings=${dupes}; orphaned '-->' with no opener=${orphan}
"
  JSON_ROWS="${JSON_ROWS}{\"file\": \"${base}\", \"heading_count\": ${dupes}, \"orphan_close_comment\": \"${orphan}\"},"
done

JSON_ROWS="${JSON_ROWS%,}"

if [ "$JSON" = "1" ]; then
  ok=true; [ "$firing" -gt 0 ] && ok=false
  printf '{"ok": %s, "scanned": %d, "with_block": %d, "firing_count": %d, "acknowledged_count": %d, "firing": [%s], "scope": "detects a counterfeit/shadowed ## Verification heading; does NOT verify that a task has adequate verification, or any at all"}\n' \
    "$ok" "$scanned" "$with_block" "$firing" "$acked" "$JSON_ROWS"
  [ "$firing" -gt 0 ] && exit 1
  exit 0
fi

if [ "$firing" -gt 0 ]; then
  echo "check-verification-heading-shadow: FIRING — ${firing} task(s) whose P-011 gate is pointed at prose"
  printf '%s' "$FIRING_LINES"
  echo "  scanned ${scanned} task file(s), ${with_block} with a non-empty extracted block, ${acked} acknowledged."
  echo "  SCOPE: detects a counterfeit/shadowed heading. It does NOT verify that a task's"
  echo "         verification is adequate, or that it has any — an empty block passes here."
  echo "  Remediation: the counterfeit heading is almost always an orphaned template block."
  echo "         Delete it (or re-indent the wrapped fragment so it is not at column 0), then"
  echo "         re-run the extractor to confirm the real commands come back."
  exit 1
fi

if [ "$QUIET" != "1" ]; then
  echo "check-verification-heading-shadow: clean — ${scanned} task file(s) scanned, ${with_block} with a non-empty extracted block, 0 pointed at prose, ${acked} acknowledged."
  echo "  SCOPE: detects a counterfeit/shadowed '## Verification' heading. It does NOT verify"
  echo "         that a task's verification is adequate, that its commands test its ACs, or"
  echo "         that a task has any verification at all — an empty block passes this check."
fi
exit 0
