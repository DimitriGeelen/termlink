#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# check-unpaired-capture.sh — capture-form verification legs with no failure assertion.
#
# ORIGIN: 832-Workflow-designer (agent-chat-arc 481) found their CLAUDE.md
# recommending an idiom that discards the command's exit status. Ours does the
# same, at .tasks/templates/default.md:107. Filed upstream at
# framework:pickup 49; this is remedy (d) from that filing, built here because
# the template is upstream-owned and a local edit is one `fw upgrade` from
# deletion.
#
# THE DEFECT
#
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
#
# The capture is CORRECT for SIGPIPE (L-387: `cmd | grep -q PAT` exits 141 when
# grep matches and closes stdin while cmd is still writing — measured on a
# 146,366-byte page, 3/3 runs). It also puts cmd's exit status into a variable
# assignment, where it dies. The line is a two-command sequence, so the gate
# judges it on the LAST command:
#
#     a ; b   the line's status is b's alone       <- capture form
#     a | b   pipefail preserves a's failure       <- plain pipe
#
# So a command that FAILS while still printing PATTERN passes the leg.
#
# THE MITIGATION IS ALREADY IN OUR TEMPLATE, JUST NOT NAMED AS ONE. Its worked
# examples pair a positive marker with a negative one:
#
#     out=$(bats f 2>&1); echo "$out" | grep -q '^ok 1 ' && ! echo "$out" | grep -q '^not ok'
#
# An author who writes only the positive half has followed the template and
# produced a leg that cannot fail. THIS SCRIPT FLAGS EXACTLY THAT: a capture
# whose only assertion is a positive grep.
#
# WHAT IT DOES NOT FLAG
#   - captures paired with a negative assertion (`! ... grep`) — the correct form
#   - plain pipes (`cmd | grep -q`) — pipefail rescues those; I published a
#     retraction for claiming otherwise (chat-arc 485)
#   - completed tasks — their blocks never re-run; reported as advisory only
#
# Exit: 0 clean | 1 unpaired capture(s) in an ACTIVE task | 2 self-test failed
set -uo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null || echo /opt/termlink)" || exit 1

# Executable lines of the first ## Verification block.
legs() {
  local root="$1"
  find "$root" -name '*.md' -path '*/.tasks/*' -not -path '*/.claude/worktrees/*' 2>/dev/null \
  | while IFS= read -r f; do
      awk -v FILE="$f" '
        /^## Verification[[:space:]]*$/ { if (!seen) { seen=1; inblk=1 }; next }
        inblk && /^## / { inblk=0 }
        inblk {
          l=$0; sub(/^[[:space:]]+/,"",l)
          if (l=="" || l ~ /^#/) next
          print FILE "\t" NR "\t" l
        }
      ' "$f"
    done
}

# A leg is UNPAIRED when it captures into a variable, asserts with a positive
# grep, and carries no negative assertion anywhere on the line.
unpaired() {
  awk -F'\t' '
    {
      line = $3
      if (line !~ /=\$\(/)        next   # no capture -> exit status not discarded
      if (line !~ /grep[[:space:]]+-[a-zA-Z]*q/) next   # no -q assertion
      if (line ~ /![[:space:]]*(echo|grep|printf)/) next # paired with a negative
      if (line ~ /grep[[:space:]]+-[a-zA-Z]*v/)  next   # negative via -v
      print $0
    }
  '
}

if [ "${1:-}" = "--self-test" ]; then
  t=$(mktemp -d) || exit 2
  mkdir -p "$t/.tasks/active"
  # planted defect: positive-only capture.
  # shellcheck disable=SC2016  # the $( ) is literal fixture text, not an expansion
  printf '## Verification\n\nout=$(mycmd 2>&1); echo "$out" | grep -q "PASSED"\n' \
    > "$t/.tasks/active/unpaired.md"
  # planted REMEDY: the template's own paired form. Must NOT be flagged —
  # 832's point that a blind-only control cannot tell you your guard is inert.
  # shellcheck disable=SC2016  # literal fixture text, not an expansion
  printf '## Verification\n\nout=$(mycmd 2>&1); echo "$out" | grep -q "ok 1" && ! echo "$out" | grep -q "not ok"\n' \
    > "$t/.tasks/active/paired.md"
  # planted plain pipe: rescued by pipefail, must NOT be flagged
  printf '## Verification\n\ncargo build --release | grep -q Finished\n' \
    > "$t/.tasks/active/piped.md"

  hits=$(legs "$t" | unpaired)
  rm -rf "$t"
  n=$(printf '%s' "$hits" | grep -c . || true)
  if [ "$n" = "1" ] && printf '%s' "$hits" | grep -q unpaired; then
    echo "self-test: PASS — flagged the unpaired capture, spared the paired form and the plain pipe"
    exit 0
  fi
  echo "self-test: FAIL — expected exactly 1 hit (unpaired.md), got $n"
  printf '%s\n' "$hits"
  exit 2
fi

all=$(legs . | unpaired)
act=$(printf '%s' "$all" | grep '/.tasks/active/' || true)
past=$(printf '%s' "$all" | grep '/.tasks/completed/' || true)
n=$(printf '%s' "$act" | grep -c . || true)
n_past=$(printf '%s' "$past" | grep -c . || true)

echo "check-unpaired-capture: capture-form legs with no failure assertion"
echo "  PREDICATE: executable lines in the first ## Verification block; line must"
echo "             capture with \$( ) AND assert with grep -q AND carry no negative"
echo "             assertion. Plain pipes are excluded (pipefail rescues them)."
echo "             Worktrees excluded. Run --self-test to confirm it can see a"
echo "             planted defect AND spares the remedy."
echo ""
[ "$n_past" != "0" ] && {
  echo "  ADVISORY: $n_past in .tasks/completed/ — never re-run, not failures."
  echo ""
}

if [ "$n" = "0" ]; then
  echo "  clean: no unpaired capture in any ACTIVE task"
  exit 0
fi

echo "  $n ACTIVE unpaired capture(s). Each passes when the command FAILS but"
echo "  still prints the pattern:"
echo ""
printf '%s\n' "$act" | awk -F'\t' '{printf "    %s:%s\n        %s\n", $1, $2, substr($3,1,140)}'
echo ""
echo "  Fix: pair it, as .tasks/templates/default.md:127-128 already does —"
echo "       ... && ! echo \"\$out\" | grep -q '<failure marker>'"
exit 1
