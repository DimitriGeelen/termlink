#!/usr/bin/env bash
# Fixtures for scripts/check-human-ac-steps-heading.sh (T-2859).
#
# Weighted toward the FIRING cases and the false-positive guards: a
# heading-form check is trivially green on a clean corpus, and a green check
# that cannot go red is not a check.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$ROOT/scripts/check-human-ac-steps-heading.sh"
PASS=0; FAIL=0
ok(){ if [ "$1" = "$2" ]; then PASS=$((PASS+1)); else FAIL=$((FAIL+1)); echo "  FAIL: $3 (want=$1 got=$2)"; fi; }

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
mk(){ mkdir -p "$TMP/$1"; }

# 1. canonical heading -> clean
mk c1; printf '  **Steps:**\n  1. do a thing\n' > "$TMP/c1/T-1.md"
bash "$CHECK" --tasks-dir "$TMP/c1" >/dev/null 2>&1; ok 0 "$?" "canonical heading is clean"

# 2. parenthesized variant -> FIRES (the T-1696/T-2858 class)
mk c2; printf '  **Steps (copy-paste):**\n  1. do a thing\n' > "$TMP/c2/T-2.md"
bash "$CHECK" --tasks-dir "$TMP/c2" >/dev/null 2>&1; ok 1 "$?" "parenthesized heading fires"

# 3. same-line content -> FIRES (the T-2522 class: renderer discards the rest)
mk c3; printf '  **Steps:** Choose one:\n  - a\n' > "$TMP/c3/T-3.md"
bash "$CHECK" --tasks-dir "$TMP/c3" >/dev/null 2>&1; ok 1 "$?" "same-line content fires"

# 4. trailing whitespace only -> clean (cosmetic, renderer unaffected)
mk c4; printf '  **Steps:**   \n  1. x\n' > "$TMP/c4/T-4.md"
bash "$CHECK" --tasks-dir "$TMP/c4" >/dev/null 2>&1; ok 0 "$?" "trailing whitespace is not a finding"

# 5. template example inside an HTML comment -> never fires
mk c5; printf '<!--\n  **Steps (example):** 1. open the page\n-->\n  **Steps:**\n  1. x\n' > "$TMP/c5/T-5.md"
bash "$CHECK" --tasks-dir "$TMP/c5" >/dev/null 2>&1; ok 0 "$?" "commented template example never fires"

# 6. allowlist suppresses a firing heading
mk c6; printf '  **Steps (copy-paste):**\n  1. x\n' > "$TMP/c6/T-6.md"
printf 'T-6.md::1  # acknowledged for test\n' > "$TMP/allow"
bash "$CHECK" --tasks-dir "$TMP/c6" --allowlist "$TMP/allow" >/dev/null 2>&1; ok 0 "$?" "allowlist suppresses firing"

# 7. allowlisted entry is still COUNTED and reported
out="$(bash "$CHECK" --tasks-dir "$TMP/c6" --allowlist "$TMP/allow" --json 2>/dev/null)"
printf '%s' "$out" > "$TMP/j.out"
grep -q '"acknowledged_count": 1' "$TMP/j.out"; ok 0 "$?" "allowlisted entry is counted in json"

# 8. removing the allowlist line re-fires (the ratchet)
: > "$TMP/allow_empty"
bash "$CHECK" --tasks-dir "$TMP/c6" --allowlist "$TMP/allow_empty" >/dev/null 2>&1; ok 1 "$?" "empty allowlist re-fires"

# 9. fail-closed: missing tasks dir is 2, never 0
bash "$CHECK" --tasks-dir "$TMP/does-not-exist" >/dev/null 2>&1; ok 2 "$?" "missing tasks dir exits 2"

# 10. fail-closed: a corpus with zero task files is 2, never a vacuous clean
mk c10; bash "$CHECK" --tasks-dir "$TMP/c10" >/dev/null 2>&1; ok 2 "$?" "empty corpus exits 2"

# 11. fail-closed: unreadable allowlist is 2
mk c11; printf '  **Steps:**\n  1. x\n' > "$TMP/c11/T-11.md"
printf 'x\n' > "$TMP/badallow"; chmod 000 "$TMP/badallow"
if [ "$(id -u)" -eq 0 ]; then PASS=$((PASS+1)); else
  bash "$CHECK" --tasks-dir "$TMP/c11" --allowlist "$TMP/badallow" >/dev/null 2>&1; ok 2 "$?" "unreadable allowlist exits 2"
fi
chmod 644 "$TMP/badallow" 2>/dev/null || true

# 12. json carries the firing detail a human can act on
out="$(bash "$CHECK" --tasks-dir "$TMP/c3" --json 2>/dev/null)"; printf '%s' "$out" > "$TMP/j3.out"
grep -q '"firing_count": 1' "$TMP/j3.out"; ok 0 "$?" "json reports firing_count"
grep -q '"line": 1' "$TMP/j3.out"; ok 0 "$?" "json reports the line number"
grep -q 'Choose one' "$TMP/j3.out"; ok 0 "$?" "json echoes the offending heading"

# 13. the real tree scans clean (PL-219 control)
bash "$CHECK" >/dev/null 2>&1; ok 0 "$?" "real .tasks/active scans clean"

echo "human-ac-steps-heading fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
