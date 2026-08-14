#!/usr/bin/env bash
# stuck-claims-check-fixtures.sh (T-2710)
#
# Hermetic proof for scripts/check-stuck-claims-freshness.sh — no live hub, no
# network, no host state. Feeds canned `channel claims-summary --all
# --only-stuck --json` envelopes through the check's TERMLINK_STUCK_CLAIMS_TEST_JSON
# seam (PL-213) and asserts the exit-code contract on every branch.
#
# WHY THIS EXISTS. The canary shipped with that test seam in T-2556 and nothing
# ever used it. A seam nothing exercises is dormant tooling (PL-168) — and the
# specific consequence here was that the canary's HEALTHY path had never once
# been asserted. It fired daily for ~62 days on latched debris (T-2709) and no
# test could have noticed, because no test ran it at all.
#
# The load-bearing fixture is 7. It pins the T-2709 semantic at the canary
# layer: a topic can carry a large `expired_count` and still be healthy, because
# expired rows are cumulative history that is reaped only on re-claim of the
# same (topic, offset). The canary must key on the hub's `stuck_count` /
# `potentially_stuck` verdict, NEVER on `expired_count` — re-introducing the
# latter anywhere in this script re-creates a guard that can never go green.
#
# Exit 0 = all fixtures pass; exit 1 = a fixture assertion failed.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-stuck-claims-freshness.sh"
[ -f "$CHECK" ] || { echo "FAIL: check not found at $CHECK" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "FAIL: jq required" >&2; exit 1; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
FIX="$SCRATCH/summary.json"

pass=0; fail=0
ok()  { pass=$((pass+1)); echo "  ok: $1"; }
bad() { fail=$((fail+1)); echo "  FAIL: $1" >&2; }
assert_rc() { [ "$2" -eq "$3" ] && ok "$1 (rc=$3)" || bad "$1 — expected rc=$2 got rc=$3"; }
assert_eq() { [ "$2" = "$3" ] && ok "$1 ($3)" || bad "$1 — expected '$2' got '$3'"; }

# --no-heartbeat: a fixture run must never refresh the real cron heartbeat, or
# it would mask a dead cron from the T-1723 meta-canary.
run() {
    TERMLINK_STUCK_CLAIMS_TEST_JSON="$FIX" bash "$CHECK" --no-heartbeat >/dev/null 2>&1
    echo $?
}
run_out() {
    TERMLINK_STUCK_CLAIMS_TEST_JSON="$FIX" bash "$CHECK" --no-heartbeat 2>&1
}
run_json() {
    TERMLINK_STUCK_CLAIMS_TEST_JSON="$FIX" bash "$CHECK" --no-heartbeat --json 2>/dev/null
}
run_quiet() {
    TERMLINK_STUCK_CLAIMS_TEST_JSON="$FIX" bash "$CHECK" --no-heartbeat --quiet 2>/dev/null
}

echo "stuck-claims-check fixtures (T-2710):"

# --- fixture 1: healthy fleet — the path that had never been asserted ---
cat > "$FIX" <<'EOF'
{"ok":true,"topic_count":770,"stuck_count":0,"shown":0,"only_stuck":true,"topics":[]}
EOF
assert_rc "healthy fleet (stuck_count 0) exits 0" 0 "$(run)"
assert_eq "healthy envelope reports ok:true" "true" "$(run_json | jq -r '.ok')"
assert_eq "healthy envelope carries topic_count" "770" "$(run_json | jq -r '.topic_count')"
assert_eq "healthy envelope reports 0 stuck" "0" "$(run_json | jq -r '.stuck_count')"

# --quiet on a healthy cycle must print NOTHING — that is what makes
# "empty log = healthy" true for the cron redirect.
assert_eq "quiet healthy cycle prints nothing" "" "$(run_quiet)"

# --- fixture 2: genuinely stuck topic fires ---
cat > "$FIX" <<'EOF'
{"ok":true,"topic_count":12,"stuck_count":1,"shown":1,"only_stuck":true,
 "topics":[{"ok":true,"topic":"work-q1","active_count":1,"expired_count":0,
            "oldest_active_age_ms":95000,"potentially_stuck":true}]}
EOF
assert_rc "a stuck topic fires" 1 "$(run)"
assert_eq "firing envelope reports ok:false" "false" "$(run_json | jq -r '.ok')"
assert_eq "firing envelope names the topic" "work-q1" "$(run_json | jq -r '.stuck[0].topic')"
case "$(run_out)" in
    *work-q1*) ok "firing output names the topic" ;;
    *) bad "firing output should name the stuck topic" ;;
esac

# --quiet must still speak when firing — a silent canary is a dead canary.
case "$(run_quiet)" in
    *work-q1*) ok "quiet mode still reports a firing topic" ;;
    *) bad "quiet mode must not suppress a real firing" ;;
esac

# --- fixture 3: malformed envelope is a TOOLING error (2), never healthy (0) ---
cat > "$FIX" <<'EOF'
{"ok":true,"topics":[]}
EOF
assert_rc "envelope with no stuck_count exits 2 (tooling)" 2 "$(run)"

# --- fixture 4: unparseable JSON is tooling, not healthy ---
printf 'not json at all\n' > "$FIX"
assert_rc "unparseable JSON exits 2 (tooling)" 2 "$(run)"

# --- fixture 5: empty output is tooling, not healthy ---
: > "$FIX"
assert_rc "empty output exits 2 (tooling)" 2 "$(run)"

# --- fixture 6: per-topic fetch errors warn but do NOT fire on their own ---
cat > "$FIX" <<'EOF'
{"ok":true,"topic_count":5,"stuck_count":0,"shown":1,"only_stuck":true,
 "topics":[{"ok":false,"topic":"unreadable","error":"timeout"}]}
EOF
assert_rc "a fetch error alone does not fire" 0 "$(run)"
assert_eq "fetch errors are counted" "1" "$(run_json | jq -r '.fetch_errors')"
case "$(run_out)" in
    *mask*) ok "fetch-error warning says it could mask a stuck topic" ;;
    *) bad "a fetch error should warn that it may mask a stuck topic" ;;
esac

# --- fixture 7 (LOAD-BEARING, T-2709): expired_count is history, not a fault ---
# 81 expired rows, nothing held (active_count 0), and the hub says NOT stuck
# because the last lease lapsed ~62 days ago. This is the exact shape that had
# this canary firing daily forever. If anyone re-introduces an `expired_count`
# test into the firing gate, this fixture goes red.
cat > "$FIX" <<'EOF'
{"ok":true,"topic_count":770,"stuck_count":0,"shown":0,"only_stuck":true,
 "topics":[{"ok":true,"topic":"substrate-drain-demo","active_count":0,
            "expired_count":81,"oldest_active_age_ms":null,
            "potentially_stuck":false}]}
EOF
assert_rc "ancient expired claims (81, none held) do NOT fire" 0 "$(run)"
assert_eq "ancient expired claims report ok:true" "true" "$(run_json | jq -r '.ok')"
assert_eq "ancient expired claims yield an empty stuck list" "0" "$(run_json | jq -r '.stuck | length')"

# ...and the same topic DOES fire once the hub flags it, proving the canary
# tracks the hub's verdict rather than ignoring expired topics wholesale.
cat > "$FIX" <<'EOF'
{"ok":true,"topic_count":770,"stuck_count":1,"shown":1,"only_stuck":true,
 "topics":[{"ok":true,"topic":"substrate-drain-demo","active_count":0,
            "expired_count":81,"oldest_active_age_ms":null,
            "potentially_stuck":true}]}
EOF
assert_rc "the same topic fires when the hub flags it stuck" 1 "$(run)"

# --- fixture 8: several stuck topics are all reported ---
cat > "$FIX" <<'EOF'
{"ok":true,"topic_count":30,"stuck_count":2,"shown":2,"only_stuck":true,
 "topics":[{"ok":true,"topic":"a","active_count":1,"expired_count":0,
            "oldest_active_age_ms":90000,"potentially_stuck":true},
           {"ok":true,"topic":"b","active_count":2,"expired_count":1,
            "oldest_active_age_ms":120000,"potentially_stuck":true}]}
EOF
assert_rc "multiple stuck topics fire" 1 "$(run)"
assert_eq "all stuck topics are listed" "2" "$(run_json | jq -r '.stuck | length')"
assert_eq "stuck_count matches" "2" "$(run_json | jq -r '.stuck_count')"

# --- fixture 9 (T-2709): a half-inert check must NOT report a bare "healthy" ---
# Against a hub predating T-2709 the abandoned-claim arm cannot fire at all, so
# stuck_count 0 means "we could not look", not "nothing is wrong". Non-firing
# (a capability gap is not a stuck claim, per PL-219) but it must be SAID.
cat > "$FIX" <<'EOF'
{"ok":true,"topic_count":770,"stuck_count":0,"shown":0,"only_stuck":true,
 "expired_arm_inert":true,"topics":[]}
EOF
assert_rc "a degraded (half-inert) check still exits 0" 0 "$(run)"
case "$(run_out)" in
    *DEGRADED*) ok "degraded run says so instead of a bare 'healthy'" ;;
    *) bad "a half-inert check must not report plain healthy" ;;
esac
case "$(run_out)" in
    *newest_expired_at_ms*) ok "degraded note names the missing field" ;;
    *) bad "degraded note should name newest_expired_at_ms" ;;
esac

# The flag must not leak into the healthy path when the hub DOES serve it.
cat > "$FIX" <<'EOF'
{"ok":true,"topic_count":770,"stuck_count":0,"shown":0,"only_stuck":true,
 "expired_arm_inert":false,"topics":[]}
EOF
case "$(run_out)" in
    *DEGRADED*) bad "a fully-capable hub must not be reported degraded" ;;
    *) ok "fully-capable hub reports plain healthy" ;;
esac

# An older CLI omits the field entirely — absent must read as "not inert",
# never as degraded, or every pre-T-2709 CLI would cry wolf.
cat > "$FIX" <<'EOF'
{"ok":true,"topic_count":770,"stuck_count":0,"shown":0,"only_stuck":true,"topics":[]}
EOF
case "$(run_out)" in
    *DEGRADED*) bad "absent expired_arm_inert must not be read as degraded" ;;
    *) ok "absent expired_arm_inert reads as not-degraded" ;;
esac

echo
echo "stuck-claims-check fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ] || exit 1
