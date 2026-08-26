#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# check-vacuous-verification.sh — verification legs that cannot fail.
#
# ORIGIN: 832-Workflow-designer, agent-chat-arc offset 471. They found an AC
# asserting "this fixture invents no runtime semantics" carried by
# `grep -qvE "conditionExpression|capability|..."`. `-v` selects NON-matching
# lines and `-q` exits 0 if ANY line was selected, so it asserts "at least one
# line lacks these tokens" — true of essentially every multi-line file. Green in
# both worlds, so it never had a chance to fail.
#
# ── CORRECTION 2026-08-26 (same day) ────────────────────────────────────────
# The claim below that the T-1427 leg "could not fail" is WRONG under this
# project's gate, and the correction is worth more than the original finding.
#
# The gate runs each line as `if ( eval "$cmd" ); then` under `set -euo pipefail`
# (agents/task-create/update-task.sh:1215, documented at :1100). pipefail
# propagates a failing left-hand command through a pipe, so:
#
#     a | b     pipefail RESCUES   — status is non-zero if a failed
#     a ; b     pipefail CANNOT    — separate commands; status is b's alone
#
# Every leg this script was written for is the PIPE form, and every one of them
# fails correctly under the real runner. The poisoned control that "proved"
# otherwise was run in a bare shell the gate never uses — an instrument
# measuring the wrong environment.
#
# THE SCRIPT IS KEPT because `grep -qv` is still a wrong-semantics smell worth
# flagging, and because a leg may be run outside the gate. But it reports a
# SMELL, not a proven vacuity, and this header says so rather than letting the
# original overclaim stand.
#
# The real defect class here is a SEMICOLON or a CAPTURE discarding the
# preceding status — 832's `out=$(cmd); echo "$out" | grep -q PAT`. Confirmed
# under our gate: a command exiting 1 while printing the matched string PASSES.
# Two scanners disagree on how many we have (40/23 vs 34/21), so that population
# is UNRESOLVED and no number is published here.
# ────────────────────────────────────────────────────────────────────────────
# MEASURED HERE, live tree, 2026-08-26: 10 executable legs matched, 9 in
# completed tasks and ONE LIVE, in T-1427:
#
#     cargo build --workspace --release 2>&1 | tail -3 | grep -qv "error"
#
# Proven vacuous rather than argued — a poisoned control whose output contained
# "error: could not compile" and exited 1 still passed the leg with rc 0,
# because one of the three tailed lines lacked the word. Replaced with plain
# `cargo build --workspace --release`, which exits 101 on the same input.
#
# TWO DEFECTS, ONE IDIOM. Both forms found here also PIPE a build command into
# grep, which discards the build's own exit status and substitutes a text
# heuristic. `cmd | tail -N | grep -q ...` can only ever assert something about
# cmd's OUTPUT, never about whether cmd SUCCEEDED. Where a command already exits
# non-zero on failure, the honest leg is the bare command.
#
# WHAT THIS ASSERTS. Executable lines inside the FIRST `## Verification` block
# of each live task file. Comment and blank lines are excluded because the gate
# does not run them. Stale worktrees under .claude/worktrees/ are excluded
# because their blocks are not run by this project's gate — counting them is the
# shape, not the consequence.
#
# --self-test proves the scanner can see dirt before you trust a clean run.
#
# Exit: 0 clean | 1 vacuous leg(s) found | 2 self-test failed
set -uo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null || echo /opt/termlink)" || exit 1

# grep with BOTH -q and -v in any order/bundling.
VACUOUS_RE='grep[[:space:]]+(-[a-zA-Z]*q[a-zA-Z]*v[a-zA-Z]*|-[a-zA-Z]*v[a-zA-Z]*q[a-zA-Z]*)([[:space:]]|$)|grep[[:space:]]+-q[[:space:]]+-v|grep[[:space:]]+-v[[:space:]]+-q'

scan() {  # $1 = root dir to scan
  local root="$1"
  while IFS= read -r f; do
    awk -v FILE="$f" '
      /^## Verification[[:space:]]*$/ { if (!seen) { seen=1; inblk=1 }; next }
      inblk && /^## / { inblk=0 }
      inblk {
        line=$0
        sub(/^[[:space:]]+/, "", line)
        if (line == "" || line ~ /^#/) next
        print FILE ":" NR "\t" line
      }
    ' "$f"
  done < <(find "$root" -name '*.md' -path '*/.tasks/*' -not -path '*/.claude/worktrees/*' 2>/dev/null) \
    | grep -E "$VACUOUS_RE" || true
}

if [ "${1:-}" = "--self-test" ]; then
  # Prove the scanner can see dirt. A clean report from an instrument that
  # cannot see its subject is indistinguishable from one that looked.
  _t=$(mktemp -d) || exit 2
  mkdir -p "$_t/.tasks/active"
  printf '## Verification\n\ncargo build 2>&1 | tail -3 | grep -qv "error"\n' \
    > "$_t/.tasks/active/poisoned.md"
  printf '## Verification\n\ncargo build --workspace\n' \
    > "$_t/.tasks/active/clean.md"
  _hits=$(scan "$_t")
  rm -rf "$_t"
  _n=$(printf '%s' "$_hits" | grep -c . || true)
  if [ "$_n" = "1" ] && printf '%s' "$_hits" | grep -q poisoned; then
    echo "self-test: PASS — scanner detected the poisoned leg and not the clean one"
    exit 0
  fi
  echo "self-test: FAIL — expected exactly 1 hit (poisoned.md), got $_n"
  printf '%s\n' "$_hits"
  exit 2
fi

all_hits=$(scan .)
hits=$(printf '%s' "$all_hits" | grep '/.tasks/active/' || true)
past=$(printf '%s' "$all_hits" | grep '/.tasks/completed/' || true)
n=$(printf '%s' "$hits" | grep -c . || true)
n_past=$(printf '%s' "$past" | grep -c . || true)

echo "check-vacuous-verification: scanning live ## Verification blocks"
echo "  PREDICATE: executable lines only (comments and blanks excluded — the gate"
echo "             does not run them), first ## Verification block per file,"
echo "             .claude/worktrees/ excluded (not run by this project's gate)."
echo "  Run --self-test to confirm the scanner can see a planted defect."
echo ""

if [ "$n_past" != "0" ]; then
  echo "  ADVISORY: $n_past occurrence(s) in .tasks/completed/. Those blocks are"
  echo "            never re-run, so they are not a live failure — but they are"
  echo "            the copy-paste source for new ones. Not counted as failures."
  echo ""
fi

if [ "$n" = "0" ]; then
  echo "  clean: no grep -qv / -vq in any ACTIVE task's verification leg"
  exit 0
fi

echo "  $n vacuous leg(s) — grep -q with -v exits 0 when ANY line fails to match,"
echo "  so these are green whether or not the property holds:"
echo ""
printf '%s\n' "$hits" | sed 's/^/    /'
echo ""
echo "  Fix: for 'contains none of these', use  ! grep -qE 'a|b'"
echo "       for 'this command succeeded', use the bare command — piping it into"
echo "       grep discards its exit status and asserts about output instead."
exit 1
