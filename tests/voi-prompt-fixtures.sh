#!/usr/bin/env bash
# Fixtures for scripts/voi-prompt.sh (T-3175).
#
# The load-bearing case is the FULL CYCLE: ask once, snooze, stay silent for exactly N
# runs, then re-ask — and say that it is a re-ask. That is the operator's requirement
# ("record the choice so I'm not asked every time, then after 5 or 10 runs ask again"),
# and a mutant that breaks expiry must turn the snooze permanent.
set -uo pipefail
S="${S:-scripts/voi-prompt.sh}"
TMP="$(mktemp -d)"; trap 'rm -rf -- "$TMP"' EXIT
PASS=0; FAIL=0
ok(){ PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad(){ FAIL=$((FAIL+1)); printf '  FAIL %s\n       %s\n' "$1" "${2:-}"; }
rc_is(){ if [ "$2" = "$3" ]; then ok "$1"; else bad "$1" "expected rc=$2 got rc=$3"; fi; }

D="$TMP/tasks"; ST="$TMP/store.yaml"; mkdir -p "$D"
mk(){ mkdir -p "$D"; { echo "---"; echo "id: $1"; echo "name: \"t\""; echo "workflow_type: ${2:-inception}";
      echo "created: ${3:-2026-10-01T00:00:00Z}"; [ -n "${4:-}" ] && echo "$4"; echo "---"; echo b; } > "$D/$1.md"; }
chk(){ FW_VOI_TASKS_DIR="$D" FW_VOI_STORE="$ST" bash "$S" --check "$@" 2>&1; }
chkrc(){ FW_VOI_TASKS_DIR="$D" FW_VOI_STORE="$ST" bash "$S" --check >/dev/null 2>&1; echo $?; }
rec(){ FW_VOI_TASKS_DIR="$D" FW_VOI_STORE="$ST" bash "$S" --record "$@" 2>&1; }
# pend <id> -> "yes" if <id> is in the PENDING array (not merely mentioned in suppressed)
pend(){ FW_VOI_TASKS_DIR="$D" FW_VOI_STORE="$ST" bash "$S" --check --json 2>/dev/null \
  > "$TMP/.p" ; python3 -c "
import json,sys
try: d=json.load(open('$TMP/.p'))
except Exception: print('err'); sys.exit()
print('yes' if any(x['id']=='$1' for x in d.get('pending',[])) else 'no')
"; }
pend_of(){ python3 -c "
import json,sys
try: d=json.load(open('$1'))
except Exception: print('err'); sys.exit()
print('yes' if any(x['id']=='$2' for x in d.get('pending',[])) else 'no')
"; }
recrc(){ FW_VOI_TASKS_DIR="$D" FW_VOI_STORE="$ST" bash "$S" --record "$@" >/dev/null 2>&1; echo $?; }

echo "== Case 1: it asks, and only about what needs asking =="
mk T-100 inception 2026-10-01 "voi_score: 0.5"
mk T-101 inception 2026-10-01 ""
mk T-102 inception 2026-10-01 "voi_score: 0.85"
mk T-103 build     2026-10-01 ""
mk T-104 inception 2026-01-01 "voi_score: 0.5"
rc_is "pending work -> rc 1" 1 "$(chkrc)"
o="$(chk --json)"
printf '%s' "$o" | grep -q '"pending_count": 2' && ok "asks about exactly the 2 that need it" || bad "pending_count" "$o"
printf '%s' "$o" | grep -q 'T-100' && printf '%s' "$o" | grep -q 'T-101' && ok "  the default AND the absent one" || bad "which pending" "$o"
printf '%s' "$o" | grep -q 'T-102' && bad "a considered value must not be asked about" "$o" || ok "a considered value is never asked about"
printf '%s' "$o" | grep -q 'T-103' && bad "non-inception asked about" "$o" || ok "non-inception is never asked about"
printf '%s' "$o" | grep -q 'T-104' && bad "pre-cutoff task asked about" "$o" || ok "pre-cutoff task is grandfathered"

echo "== Case 2: THE CYCLE — snooze, silence for exactly N, then re-ask =="
# --snooze 3 => silent for runs 1,2,3 and re-asked at run 4 (askable_at_run = R+N+1)
rm -f "$ST"; rec --task T-100 --choice snooze --snooze 3 >/dev/null 2>&1
rec --task T-101 --choice waive --reason "throwaway spike" >/dev/null 2>&1
[ "$(pend T-100)" = "no" ] && ok "run 1 after snooze: silent about T-100" || bad "snooze not honoured at run 1" "$(cat $TMP/.p)"
[ "$(pend T-100)" = "no" ] && ok "run 2: still silent" || bad "run2" ""
[ "$(pend T-100)" = "no" ] && ok "run 3: still silent" || bad "run3" ""
r4="$(pend T-100)"
[ "$r4" = "yes" ] && ok "run 4: RE-ASKS after the 3-run snooze expires" || bad "did not re-ask" "$(cat $TMP/.p)"
grep -q 're-ask' "$TMP/.p" && ok "  and says it is a RE-ASK, not a fresh question" || bad "re-ask wording" ""
[ "$(pend_of "$TMP/.p" T-101)" = "no" ] && ok "the permanent waiver stays silent across all runs" || bad "waiver re-asked" ""

echo "== Case 3: the two scopes are different requests =="
rm -f "$ST"
rec --global --choice snooze --snooze 2 >/dev/null 2>&1
# snooze N recorded at run R is askable at R+N+1, so N=2 -> silent for runs 1,2; expires at run 3
rm -f "$ST"; rec --global --choice snooze --snooze 2 >/dev/null 2>&1
o="$(chk --json)"                       # run 1: silent
printf '%s' "$o" | grep -q '"global_suppressed": true' && ok "global snooze silences everything" || bad "global snooze" "$o"
printf '%s' "$o" | grep -q '"pending_count": 0' && ok "  nothing pending while globally snoozed" || bad "global pending" "$o"
rc_is "  and rc is 0, so it is genuinely quiet" 0 "$(chkrc)"   # run 2: silent
o="$(chk --json)"                       # run 3: expiry
printf '%s' "$o" | grep -q 'EXPIRED' && ok "global snooze EXPIRES and says so" || bad "global expiry" "$o"

echo "== Case 4: a permanent waiver still needs a reason (carried from T-3174) =="
rm -f "$ST"
rc_is "waive with no --reason is refused" 2 "$(recrc --task T-100 --choice waive)"
rc_is "waive with an empty --reason is refused" 2 "$(recrc --task T-100 --choice waive --reason '')"
rc_is "waive WITH a reason is accepted" 0 "$(recrc --task T-100 --choice waive --reason 'not worth ranking')"
rc_is "set with no --value is refused" 2 "$(recrc --task T-100 --choice set)"
rc_is "an unknown choice is refused" 2 "$(recrc --task T-100 --choice maybe)"
rc_is "--record with neither --task nor --global is refused" 2 "$(recrc --choice snooze)"

echo "== Case 5: the store records who/when and keeps history =="
rm -f "$ST"
rec --task T-100 --choice snooze --snooze 5 >/dev/null 2>&1
rec --task T-100 --choice set --value 0.7 >/dev/null 2>&1
grep -q 'recorded_at' "$ST" && ok "store records WHEN a decision was made" || bad "recorded_at" ""
grep -q 'recorded_at_run' "$ST" && ok "store records at WHICH run" || bad "recorded_at_run" ""
h=$(python3 -c "import yaml;print(len(yaml.safe_load(open('$ST'))['history']))" 2>/dev/null || echo 0)
[ "$h" = "2" ] && ok "history is append-only (2 entries after 2 decisions)" || bad "history append-only" "len=$h"
cur=$(python3 -c "import yaml;print(yaml.safe_load(open('$ST'))['decisions']['task:T-100']['choice'])" 2>/dev/null)
[ "$cur" = "set" ] && ok "current state is the LATEST decision, not the first" || bad "current state" "$cur"

echo "== Case 6: fails open, never blocks =="
printf 'not: [valid\n' > "$TMP/bad.yaml"
FW_VOI_TASKS_DIR="$D" FW_VOI_STORE="$TMP/bad.yaml" bash "$S" --check >/dev/null 2>&1
rc_is "unparseable store -> rc 0, nothing pending" 0 "$?"
FW_VOI_TASKS_DIR="$TMP/nope" FW_VOI_STORE="$TMP/s2.yaml" bash "$S" --check >/dev/null 2>&1
rc_is "absent tasks dir -> rc 0" 0 "$?"
printf -- '---\nbroken [\n---\n' > "$D/T-999.md"
FW_VOI_TASKS_DIR="$D" FW_VOI_STORE="$TMP/s3.yaml" bash "$S" --check >/dev/null 2>&1
rc=$?; [ "$rc" = "0" ] || [ "$rc" = "1" ] && ok "a malformed task file never crashes the check" || bad "malformed task" "rc=$rc"
rm -f "$D/T-999.md"

echo "== Case 7: --quiet and --status =="
rm -f "$ST"; mk T-200 inception 2026-10-01 "voi_score: 0.9"
o="$(FW_VOI_TASKS_DIR="$TMP/empty" FW_VOI_STORE="$ST" bash "$S" --check --quiet 2>&1)"
[ -z "$o" ] && ok "--quiet is silent when there is nothing to ask" || bad "--quiet" "$o"
rec --task T-300 --choice snooze --snooze 4 >/dev/null 2>&1
o="$(FW_VOI_STORE="$ST" bash "$S" --status 2>&1)"
printf '%s' "$o" | grep -q 'run(s) left' && ok "--status shows how many runs remain on a snooze" || bad "status" "$o"
grep -q '^# guard-layer: source' "$S" && ok "carries the guard-layer marker" || bad "marker" ""

echo "== Case 8: MUTANT — break expiry, the snooze becomes permanent =="
M="$TMP/mutant.sh"
sed 's|            left = e.get("askable_at_run", 0) - run|            left = 999999|' "$S" > "$M"
rm -f "$ST"; mk T-400 inception 2026-10-01 "voi_score: 0.5"
FW_VOI_TASKS_DIR="$D" FW_VOI_STORE="$ST" bash "$S" --record --task T-400 --choice snooze --snooze 1 >/dev/null 2>&1
for _ in 1 2 3; do FW_VOI_TASKS_DIR="$D" FW_VOI_STORE="$ST" bash "$M" --check --json >/dev/null 2>&1; done
FW_VOI_TASKS_DIR="$D" FW_VOI_STORE="$ST" bash "$M" --check --json > "$TMP/.m" 2>&1
[ "$(pend_of "$TMP/.m" T-400)" = "no" ] && ok "mutant never re-asks — snooze became permanent (the defect)" || bad "mutant re-asked" "$(cat $TMP/.m)"
FW_VOI_TASKS_DIR="$D" FW_VOI_STORE="$ST" bash "$S" --check --json > "$TMP/.r" 2>&1
[ "$(pend_of "$TMP/.r" T-400)" = "yes" ] && ok "  ...while the real script DOES re-ask (expiry is load-bearing)" || bad "real script did not re-ask" "$(cat $TMP/.r)"

printf '\n%s\n' "voi-prompt fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
