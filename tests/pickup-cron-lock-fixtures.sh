#!/usr/bin/env bash
# pickup-cron-lock-fixtures.sh — hermetic fixtures for scripts/check-pickup-cron-lock.sh
# (T-2870). Weighted toward the FIRING cases and the false-positive guards, per the
# repo rule that a register-driven check trivially green is not a check.
set -u

SCRIPT="$(cd "$(dirname "$0")/.." && pwd)/scripts/check-pickup-cron-lock.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

PASS=0
FAIL=0
assert_rc() { # desc expected_rc actual_rc
  if [ "$2" -eq "$3" ]; then PASS=$((PASS+1)); echo "ok - $1"
  else FAIL=$((FAIL+1)); echo "not ok - $1 (expected rc=$2 got rc=$3)"; fi
}
assert_grep() { # desc pattern file
  if grep -q "$2" "$3"; then PASS=$((PASS+1)); echo "ok - $1"
  else FAIL=$((FAIL+1)); echo "not ok - $1 (pattern '$2' absent)"; fi
}

run() { # dir → rc, output in $TMP/.out
  PICKUP_LOCK_CRON_DIR="$1" PICKUP_LOCK_PROJECT_ROOT="/opt/termlink" \
    bash "$SCRIPT" ${2:-} > "$TMP/.out" 2>&1
  echo $?
}

# ── Case 1: the real-world defect shape — QUOTED fw path, no flock. FIRES. ──
# This pins the matching-note property: `grep "fw pickup process"` misses this
# line (quote between fw and pickup); the check must not.
mkdir -p "$TMP/c1"
cat > "$TMP/c1/agentic-pickup-termlink" <<'EOF'
SHELL=/bin/bash
* * * * * root PROJECT_ROOT="/opt/termlink" "/opt/termlink/.agentic-framework/bin/fw" pickup process 2>&1 | logger -t agentic-pickup
EOF
rc=$(run "$TMP/c1")
assert_rc "c1: quoted-path unlocked line fires" 1 "$rc"
assert_grep "c1: names the file" "agentic-pickup-termlink" "$TMP/.out"
assert_grep "c1: names the reason" "NO flock" "$TMP/.out"

# ── Case 2: both lines locked on ONE shared path. CLEAN. ──
mkdir -p "$TMP/c2"
cat > "$TMP/c2/agentic-pickup-termlink" <<'EOF'
* * * * * root flock -n /var/lock/agentic-pickup-termlink.lock -c 'PROJECT_ROOT="/opt/termlink" /opt/termlink/.agentic-framework/bin/fw pickup process' 2>&1 | logger -t agentic-pickup
EOF
cat > "$TMP/c2/agentic-audit-termlink" <<'EOF'
*/15 * * * * root flock -n /var/lock/agentic-pickup-termlink.lock -c 'cd /opt/termlink && PROJECT_ROOT="/opt/termlink" /opt/termlink/.agentic-framework/bin/fw pickup process' 2>&1 | logger -t agentic-cron
EOF
rc=$(run "$TMP/c2")
assert_rc "c2: two lines, one shared lock — clean" 0 "$rc"
assert_grep "c2: reports 2 lines checked" "2 pickup cron line" "$TMP/.out"

# ── Case 3: both locked but on DIFFERENT lock paths. FIRES (no mutual exclusion). ──
mkdir -p "$TMP/c3"
cat > "$TMP/c3/agentic-pickup-termlink" <<'EOF'
* * * * * root flock -n /var/lock/pickup-a.lock -c '/opt/termlink/.agentic-framework/bin/fw pickup process'
EOF
cat > "$TMP/c3/agentic-audit-termlink" <<'EOF'
*/15 * * * * root flock -n /var/lock/pickup-b.lock -c '/opt/termlink/.agentic-framework/bin/fw pickup process'
EOF
rc=$(run "$TMP/c3")
assert_rc "c3: divergent lock paths fire" 1 "$rc"
assert_grep "c3: names the divergence" "DISTINCT lock paths" "$TMP/.out"

# ── Case 4: one locked, one not — mixed. FIRES on the unlocked one. ──
mkdir -p "$TMP/c4"
cat > "$TMP/c4/agentic-pickup-termlink" <<'EOF'
* * * * * root flock -n /var/lock/agentic-pickup-termlink.lock -c '/opt/termlink/.agentic-framework/bin/fw pickup process'
EOF
cat > "$TMP/c4/agentic-audit-termlink" <<'EOF'
*/15 * * * * root PROJECT_ROOT="/opt/termlink" "/opt/termlink/.agentic-framework/bin/fw" pickup process 2>&1 | logger -t agentic-cron
EOF
rc=$(run "$TMP/c4")
assert_rc "c4: mixed locked/unlocked fires" 1 "$rc"
assert_grep "c4: attributes the unlocked file" "agentic-audit-termlink: pickup line has NO flock" "$TMP/.out"

# ── Case 5: comment lines and other projects' pickup lines never fire. ──
mkdir -p "$TMP/c5"
cat > "$TMP/c5/agentic-audit-other" <<'EOF'
# a comment mentioning /opt/termlink pickup process must not count
* * * * * root /home/other/.agentic-framework/bin/fw pickup process
EOF
rc=$(run "$TMP/c5")
assert_rc "c5: comments + foreign-project lines are out of scope" 0 "$rc"
assert_grep "c5: reports nothing to serialise" "nothing to serialise" "$TMP/.out"

# ── Case 6: absent cron dir — informational clean (macOS/dev host). ──
rc=$(run "$TMP/does-not-exist")
assert_rc "c6: absent cron dir is informational clean" 0 "$rc"
assert_grep "c6: says so" "nothing to check" "$TMP/.out"

# ── Case 7: --json shapes on both verdicts. ──
rc=$(run "$TMP/c1" --json)
assert_rc "c7: json firing rc" 1 "$rc"
assert_grep "c7: json ok:false" '"ok": false' "$TMP/.out"
rc=$(run "$TMP/c2" --json)
assert_rc "c7: json clean rc" 0 "$rc"
assert_grep "c7: json ok:true" '"ok": true' "$TMP/.out"
assert_grep "c7: json counts locked" '"locked": 2' "$TMP/.out"

# ── Case 8: flock present but path unparseable — fires, never a silent pass. ──
mkdir -p "$TMP/c8"
cat > "$TMP/c8/agentic-pickup-termlink" <<'EOF'
* * * * * root flock -n -c '/opt/termlink/.agentic-framework/bin/fw pickup process'
EOF
rc=$(run "$TMP/c8")
assert_rc "c8: unparseable lock path fires" 1 "$rc"
assert_grep "c8: names the parse failure" "no lock path could be parsed" "$TMP/.out"

echo ""
echo "pickup-cron-lock fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
