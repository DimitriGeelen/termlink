#!/usr/bin/env bash
# guard-layer: source
# check-fabric-card-parse.sh — every .fabric/components/*.yaml must parse.
#
# WHY THIS EXISTS, and it is not tidiness. `fw audit`'s fabric section reports a CRASH
# as a PASS. Reported by 001-CashWeb (agent-chat-arc 570), reproduced here on a second
# instance with the mechanism located:
#
#   .agentic-framework/agents/audit/audit.sh:2106   drift_result=$(python3 -c "..." 2>&1)
#                                          :2113   fabric_registered=$(echo "$drift_result" | awk '{print $1}')
#                                          :2122   pass "Fabric: $fabric_registered registered card(s)"
#
# Three independent things have to go wrong together and all three do:
#   1. `2>&1` folds the traceback INTO the value instead of letting it reach the operator
#   2. the assignment discards python's exit code, so a crash is indistinguishable from
#      a successful run (PL-361: never collapse an exit code you did not inspect)
#   3. `pass` is unconditional — nothing between the capture and the verdict can refuse
#
# Result, measured here verbatim:
#
#   [PASS] Fabric: Traceback ... registered card(s)
#
# and the three sibling fabric checks (registered cards, edgeless cards, drift) vanish
# from the run with no indication they did not execute, while the PASS counter
# increments. A tree with one broken card scores GREENER than a healthy one, and the
# audit runs pre-push, so the incentive points hardest the wrong way at exactly the
# moment you would most want it pointing right.
#
# WHAT THIS CHECK IS AND IS NOT. It cannot fix the audit — that is vendored code and
# G-062 forbids patching it (reported upstream instead). It closes the PRECONDITION: if
# no card is ever unparseable, the audit never reaches the branch that lies. That is a
# compensating control, not a fix, and calling it a fix would be the same class of error
# as the bug.
#
# The framework already implements the right doctrine 30 lines up in the same run:
#   [WARN] ... NOT EVALUATED: candidate set empty
#   "A PASS here would assert coverage the check does not have (T-3105)."
# So this is not "the audit should be more careful" — the doctrine is written, worded,
# and shipped, and the fabric branch violates it in the strongest available way: it did
# not merely fail to measure, it crashed, and it reported PASS.
#
# Exit: 0 every card parses · 1 at least one does not · 2 could not run (no verdict)
set -uo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null || echo /opt/termlink)" || exit 1

DIR="${FABRIC_COMPONENTS_DIR:-.fabric/components}"

if [ "${1:-}" = "--self-test" ]; then
  # A guard for a false-green must itself be proven able to go red. Plants a genuinely
  # unparseable card and a valid one in a scratch corpus and asserts BOTH verdicts —
  # detection alone would be satisfied by a check that fails on everything.
  t=$(mktemp -d) || exit 2
  trap 'rm -rf "$t"' EXIT INT TERM HUP
  mkdir -p "$t/good" "$t/bad"
  printf 'name: ok\nlocation: scripts/x.sh\n' > "$t/good/a.yaml"
  printf 'name: ok\nlocation: scripts/x.sh\n' > "$t/bad/a.yaml"
  # `a: {b` raises ParserError. NOTE: the shape first reported on the rail,
  # `{unquoted: F2273, x}`, PARSES FINE — a fixture built from it would prove nothing.
  printf 'name: broken\nbad: {unclosed\n' > "$t/bad/b.yaml"
  fail=0
  FABRIC_COMPONENTS_DIR="$t/good" bash "$0" >/dev/null 2>&1
  [ $? -eq 0 ] || { echo "self-test: FAIL clean corpus did not exit 0"; fail=1; }
  FABRIC_COMPONENTS_DIR="$t/bad" bash "$0" >/dev/null 2>&1
  [ $? -eq 1 ] || { echo "self-test: FAIL unparseable card not detected"; fail=1; }
  FABRIC_COMPONENTS_DIR="$t/does-not-exist" bash "$0" >/dev/null 2>&1
  [ $? -eq 2 ] || { echo "self-test: FAIL missing dir must be NO VERDICT (2), not clean"; fail=1; }
  [ "$fail" = "0" ] || exit 2
  echo "self-test: PASS — clean corpus 0, broken card 1, missing dir 2 (three distinct verdicts)"
  exit 0
fi

command -v python3 >/dev/null 2>&1 || { echo "check-fabric-card-parse: no python3 — no verdict"; exit 2; }
[ -d "$DIR" ] || { echo "check-fabric-card-parse: $DIR not found — NO VERDICT (not 'clean')"; exit 2; }

# The exit code is the verdict and stderr is NOT folded into stdout — the two things
# audit.sh:2106 gets wrong. A traceback here reaches the operator instead of becoming
# the value of a variable that gets printed after the word PASS.
out=$(python3 - "$DIR" <<'PY'
import glob, os, sys, yaml
d = sys.argv[1]
cards = sorted(glob.glob(os.path.join(d, "*.yaml")))
bad = []
for p in cards:
    try:
        with open(p, encoding="utf-8") as f:
            yaml.safe_load(f)
    except Exception as e:
        bad.append((p, type(e).__name__, str(e).splitlines()[0][:100]))
print(f"TOTAL {len(cards)}")
for p, t, m in bad:
    print(f"BAD {p} {t}: {m}")
sys.exit(1 if bad else 0)
PY
)
rc=$?

if [ "$rc" -ge 2 ]; then
  echo "check-fabric-card-parse: scanner itself failed (rc=$rc) — NO VERDICT"
  printf '%s\n' "$out"
  exit 2
fi

total=$(printf '%s\n' "$out" | awk '/^TOTAL /{print $2}')
nbad=$(printf '%s\n' "$out" | grep -c '^BAD ' || true)

echo "check-fabric-card-parse: ${total:-0} card(s) in $DIR"
echo "  PREDICATE: every card must yaml.safe_load. An unparseable card makes"
echo "             \`fw audit\` print '[PASS] Fabric: Traceback ... registered card(s)'"
echo "             and silently drop its three sibling fabric checks (audit.sh:2106-2122)."
echo "             This closes the PRECONDITION; it does not fix the audit."

if [ "$nbad" -gt 0 ]; then
  echo ""
  printf '%s\n' "$out" | grep '^BAD ' | sed 's/^BAD /  UNPARSEABLE /'
  echo ""
  echo "  Fix the card. Until then the daily audit reports the fabric section as PASSING."
  exit 1
fi

echo "  all parse."
exit 0
