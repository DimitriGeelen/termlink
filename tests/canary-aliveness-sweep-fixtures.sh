#!/usr/bin/env bash
# guard-layer: source
#
# tests/canary-aliveness-sweep-fixtures.sh (T-2878)
#
# Fixtures for `check-canary-aliveness.sh --all` — the sweep that watches every
# cron-scheduled canary in one run.
#
# Weighted deliberately toward the FIRING cases and the false-positive guards.
# A sweep is trivially green on a healthy tree, and a green that cannot go red is
# not evidence (PL-328). Every assertion below therefore drives the sweep against
# a constructed fixture tree via CANARY_SWEEP_WORKING_DIR / CANARY_SWEEP_CRON_DIR
# (PL-213), so no live cron, no real canary and no host state is involved.
#
# Run: bash tests/canary-aliveness-sweep-fixtures.sh

set -u

SCRIPT="$(cd "$(dirname "$0")/.." && pwd)/scripts/check-canary-aliveness.sh"
PASS=0
FAIL=0

ok()   { PASS=$((PASS + 1)); printf '  ok %s\n' "$1"; }
bad()  { FAIL=$((FAIL + 1)); printf '  NOT OK %s\n     %s\n' "$1" "${2:-}"; }

# assert_rc <expected-rc> <label> -- runs sweep in the current fixture tree
assert_rc() {
    local want="$1" label="$2" got out
    out=$(CANARY_SWEEP_WORKING_DIR="$W" CANARY_SWEEP_CRON_DIR="$C" \
          bash "$SCRIPT" --all 2>&1)
    got=$?
    if [ "$got" = "$want" ]; then ok "$label (rc=$got)"
    else bad "$label" "expected rc=$want got rc=$got; output: $(printf '%s' "$out" | head -3 | tr '\n' ' ')"; fi
}

assert_out() {
    local pat="$1" label="$2" out
    out=$(CANARY_SWEEP_WORKING_DIR="$W" CANARY_SWEEP_CRON_DIR="$C" \
          bash "$SCRIPT" --all 2>&1)
    if grep -q -- "$pat" <<<"$out"; then ok "$label"
    else bad "$label" "pattern '$pat' absent from: $(printf '%s' "$out" | head -5 | tr '\n' ' ')"; fi
}

assert_not_out() {
    local pat="$1" label="$2" out
    out=$(CANARY_SWEEP_WORKING_DIR="$W" CANARY_SWEEP_CRON_DIR="$C" \
          bash "$SCRIPT" --all 2>&1)
    if grep -q -- "$pat" <<<"$out"; then bad "$label" "pattern '$pat' unexpectedly present"
    else ok "$label"; fi
}

# ── fixture tree helpers ──────────────────────────────────────────────────────
new_tree() {
    ROOT=$(mktemp -d)
    W="$ROOT/working"; C="$ROOT/cron"
    mkdir -p "$W" "$C"
}
cleanup() { [ -n "${ROOT:-}" ] && rm -rf "$ROOT"; }
trap cleanup EXIT

# schedule <name>  — write a crontab that names the canary's LOG, which is the
# predicate the sweep uses to decide a canary is genuinely cron-scheduled.
schedule() {
    printf '# fixture crontab\n0 7 * * * root bash scripts/check-%s.sh --quiet >> .context/working/.%s.log\n' \
        "$1" "$1" > "$C/$1.crontab"
}
fresh_hb() { : > "$W/.$1.heartbeat"; }
stale_hb() { : > "$W/.$1.heartbeat"; touch -d '10 days ago' "$W/.$1.heartbeat" 2>/dev/null \
                  || touch -t "$(date -v-10d +%Y%m%d%H%M 2>/dev/null)" "$W/.$1.heartbeat"; }
log_with() { printf '%s\n' "$2" > "$W/.$1.log"; }

echo "canary-aliveness sweep fixtures (T-2878)"

# ── Case 1: healthy tree — every scheduled canary has a fresh heartbeat ───────
echo "case 1: all scheduled canaries fresh"
new_tree
schedule alpha-canary; fresh_hb alpha-canary
schedule beta-canary;  fresh_hb beta-canary
assert_rc 0 "healthy sweep exits 0"
assert_out "all 2 cron-scheduled canaries alive" "healthy sweep names the count"
cleanup

# ── Case 2: STALE — the classic dead-cron case the meta-canary exists for ────
echo "case 2: a stale heartbeat fires"
new_tree
schedule alpha-canary; fresh_hb alpha-canary
schedule beta-canary;  stale_hb beta-canary
assert_rc 1 "stale heartbeat exits 1"
assert_out "\[STALE\] beta-canary" "stale entry names the canary"
assert_not_out "\[STALE\] alpha-canary" "fresh canary is not reported stale"
cleanup

# ── Case 3: NO-HEARTBEAT — the class a heartbeat-only walk cannot see ────────
# This is the T-2878 defect itself (hook-counter-integrity). A sweep that walked
# only heartbeats would report this tree fully alive while a scheduled canary sat
# unwatched. Deleting the log-walk in sweep_all turns this case green — that
# mutant is what proves the arm carries weight.
echo "case 3: cron-scheduled canary with a log but no heartbeat"
new_tree
schedule alpha-canary; fresh_hb alpha-canary
schedule gamma-canary; log_with gamma-canary "gamma-canary: FIRING — something real"
assert_rc 1 "heartbeat-less scheduled canary exits 1"
assert_out "\[NO-HEARTBEAT\] gamma-canary" "names the unwatchable canary"
assert_out "scripts/\*gamma\*" "fix hint strips the -canary suffix so the glob can match"
cleanup

# ── Case 4: false-positive guard — heartbeat with no crontab is EXCLUDED ─────
# The source-level static checks (alloc-sink, busy-spin, drain-sink, silent-exit)
# leave a heartbeat behind when run by hand but have no schedule. Reporting them
# STALE forever is the noise T-2826 removed from /canaries; re-introducing it
# here would make the sweep mostly false.
echo "case 4: unscheduled heartbeat is excluded, not reported dead"
new_tree
schedule alpha-canary; fresh_hb alpha-canary
stale_hb orphan-canary          # heartbeat exists, NO crontab names its log
assert_rc 0 "unscheduled stale heartbeat does not fire"
assert_out "1 unscheduled heartbeat(s) excluded" "exclusion is counted and reported"
assert_not_out "\[STALE\] orphan-canary" "unscheduled canary is not called stale"
cleanup

# ── Case 5: fail-closed on an empty corpus ───────────────────────────────────
# "0 dead out of 0" is vacuously true. A discovery path that silently stops
# matching must not read as a clean bill of health (T-2747 zero-census lesson).
echo "case 5: fail-closed"
new_tree
assert_rc 2 "empty working dir exits 2, not 0"
cleanup

new_tree
schedule alpha-canary; fresh_hb alpha-canary
rm -rf "$C"
assert_rc 2 "absent cron source dir exits 2"
cleanup

new_tree
schedule alpha-canary; fresh_hb alpha-canary
rm -rf "$W"
assert_rc 2 "absent working dir exits 2"
cleanup

# ── Case 6: heartbeats present but NONE scheduled — still fail-closed ────────
# Distinct from case 5: files exist, so a naive implementation would iterate,
# find nothing scheduled, and report "all alive" over an empty checked set.
echo "case 6: heartbeats present but none scheduled"
new_tree
fresh_hb orphan-a-canary
fresh_hb orphan-b-canary
assert_rc 2 "all-excluded corpus exits 2, never a vacuous clean"
cleanup

# ── Case 7: both classes at once are both reported ───────────────────────────
echo "case 7: STALE and NO-HEARTBEAT reported together"
new_tree
schedule alpha-canary; fresh_hb alpha-canary
schedule beta-canary;  stale_hb beta-canary
schedule gamma-canary; log_with gamma-canary "real finding"
assert_rc 1 "mixed tree exits 1"
assert_out "\[STALE\] beta-canary" "stale class present"
assert_out "\[NO-HEARTBEAT\] gamma-canary" "no-heartbeat class present"
assert_out "3 cron-scheduled canaries checked, 1 alive" "counts are truthful"
cleanup

# ── Case 8: --quiet suppresses only the healthy line, never a firing ─────────
echo "case 8: --quiet semantics"
new_tree
schedule alpha-canary; fresh_hb alpha-canary
out=$(CANARY_SWEEP_WORKING_DIR="$W" CANARY_SWEEP_CRON_DIR="$C" bash "$SCRIPT" --all --quiet 2>&1)
rc=$?
if [ "$rc" = 0 ] && [ -z "$out" ]; then ok "--quiet healthy prints nothing (rc=0)"
else bad "--quiet healthy prints nothing" "rc=$rc out='$out'"; fi

schedule beta-canary; stale_hb beta-canary
out=$(CANARY_SWEEP_WORKING_DIR="$W" CANARY_SWEEP_CRON_DIR="$C" bash "$SCRIPT" --all --quiet 2>&1)
rc=$?
if [ "$rc" = 1 ] && grep -q "STALE" <<<"$out"; then ok "--quiet still reports a firing"
else bad "--quiet still reports a firing" "rc=$rc out='$out'"; fi
cleanup

# ── Case 9: threshold is honoured ────────────────────────────────────────────
echo "case 9: --max-age-hours"
new_tree
schedule beta-canary; stale_hb beta-canary      # 10 days old
out=$(CANARY_SWEEP_WORKING_DIR="$W" CANARY_SWEEP_CRON_DIR="$C" \
      bash "$SCRIPT" --all --max-age-hours 480 2>&1); rc=$?
if [ "$rc" = 0 ]; then ok "10d heartbeat under a 480h threshold is alive"
else bad "480h threshold" "rc=$rc out=$(head -2 <<<"$out")"; fi
out=$(CANARY_SWEEP_WORKING_DIR="$W" CANARY_SWEEP_CRON_DIR="$C" \
      bash "$SCRIPT" --all --max-age-hours 24 2>&1); rc=$?
if [ "$rc" = 1 ]; then ok "10d heartbeat over a 24h threshold fires"
else bad "24h threshold" "rc=$rc out=$(head -2 <<<"$out")"; fi
cleanup

# ── Case 10: per-canary mode is unchanged (no regression) ───────────────────
# The sweep is additive. The original single-canary path must behave exactly as
# before, since 8 crontabs depend on it.
echo "case 10: single-canary mode still works"
new_tree
fresh_hb solo-canary
out=$(HEARTBEAT_FILE="$W/.solo-canary.heartbeat" CANARY_NAME="solo" \
      CANARY_PROBE_CMD="" bash "$SCRIPT" 2>&1); rc=$?
if [ "$rc" = 0 ] && grep -q "Canary alive" <<<"$out"; then ok "single-canary fresh heartbeat exits 0"
else bad "single-canary fresh" "rc=$rc out=$out"; fi

out=$(HEARTBEAT_FILE="$W/.absent-canary.heartbeat" CANARY_NAME="absent" \
      CANARY_PROBE_CMD="" bash "$SCRIPT" 2>&1); rc=$?
if [ "$rc" = 1 ] && grep -q "HEARTBEAT ABSENT" <<<"$out"; then ok "single-canary absent heartbeat exits 1"
else bad "single-canary absent" "rc=$rc out=$out"; fi
cleanup

echo
echo "canary-aliveness sweep fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
