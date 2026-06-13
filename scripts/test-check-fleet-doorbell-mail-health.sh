#!/usr/bin/env bash
# T-1831 — tests for check-fleet-doorbell-mail-health.sh.
#
# Covers:
#   T1 --help → exit 0 with usage on stdout
#   T2 unknown arg → exit 2
#   T3 --json with local-only --hubs-file → ok=true, verdict=pass (skip if no local hub)
#   T4 --json with unreachable-only --hubs-file → ok=false, unreachable>=1
#   T5 missing --hubs-file → exit 2 tooling error
#   T6 unreachable host declared transient (T-2225) → ok=true, no DRIFT, exit 0
#   T7 unreachable host NOT declared transient → still DRIFT (skip-leak guard)
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
SCRIPT="${SCRIPT:-scripts/check-fleet-doorbell-mail-health.sh}"

PASS=0; FAIL=0; SKIP=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }
skip() { echo "  SKIP: $*"; SKIP=$((SKIP + 1)); }

# Pre-flight: is the local hub up? (affects T3 only)
if "$TERMLINK" hub status >/dev/null 2>&1; then hub_up=1; else hub_up=0; fi

work="$(mktemp -d -t fleet-dm-canary-tests.XXXXXX)"
trap 'rm -rf "$work"' EXIT

# -------- T1: --help → exit 0, usage on stdout --------
echo "T1: --help → exit 0 with usage"
out="$(bash "$SCRIPT" --help 2>/dev/null)"
rc=$?
if [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -qF "Usage:"; then
    pass "T1: --help exit=0 with usage"
else
    fail "T1: exit=$rc out=$out"
fi

# -------- T2: unknown arg → exit 2 --------
echo "T2: unknown arg → exit 2"
if bash "$SCRIPT" --bogus >/dev/null 2>&1; then
    fail "T2: should have failed on --bogus"
else
    rc=$?
    if [ "$rc" -eq 2 ]; then pass "T2: exit=$rc"
    else fail "T2: expected 2, got $rc"; fi
fi

# -------- T3: --json with local-only --hubs-file → verdict=pass --------
echo "T3: --json with local-only --hubs-file → verdict=pass"
if [ "$hub_up" -ne 1 ]; then
    skip "T3: local hub not up"
else
    cat > "$work/hubs-local.toml" <<EOF
[hubs.local-test-canary]
address = "127.0.0.1:9100"
secret_file = "/var/lib/termlink/hub.secret"
EOF
    out="$(bash "$SCRIPT" --hubs-file "$work/hubs-local.toml" --json --no-heartbeat 2>/dev/null)"
    rc=$?
    ok_field="$(printf '%s' "$out" | jq -r '.ok // false' 2>/dev/null || echo "")"
    verdict="$(printf '%s' "$out" | jq -r '.profiles[0].verdict // ""' 2>/dev/null || echo "")"
    if [ "$rc" -eq 0 ] && [ "$ok_field" = "true" ] && [ "$verdict" = "pass" ]; then
        pass "T3: ok=true verdict=pass"
    else
        fail "T3: rc=$rc ok=$ok_field verdict=$verdict out=$out"
    fi
fi

# -------- T4: --json with unreachable-only --hubs-file → ok=false, unreachable>=1 --------
echo "T4: --json with unreachable --hubs-file → ok=false unreachable>=1"
cat > "$work/hubs-unreachable.toml" <<EOF
[hubs.unreachable-canary]
address = "127.0.0.1:6"
secret_file = "/tmp/nonexistent.hex"
EOF
out="$(bash "$SCRIPT" --hubs-file "$work/hubs-unreachable.toml" --json --no-heartbeat 2>/dev/null)"
rc=$?
ok_field="$(printf '%s' "$out" | jq -r '.ok' 2>/dev/null || echo "")"
unreachable="$(printf '%s' "$out" | jq -r '.summary.unreachable // 0' 2>/dev/null || echo "0")"
if [ "$rc" -eq 1 ] && [ "$ok_field" = "false" ] && [ "$unreachable" -ge 1 ]; then
    pass "T4: rc=1 ok=false unreachable=$unreachable"
else
    fail "T4: rc=$rc ok=$ok_field unreachable=$unreachable out=$out"
fi

# -------- T5: missing --hubs-file → exit 2 --------
echo "T5: missing --hubs-file → exit 2"
if bash "$SCRIPT" --hubs-file "$work/nonexistent.toml" --no-heartbeat >/dev/null 2>&1; then
    fail "T5: should have failed on missing file"
else
    rc=$?
    if [ "$rc" -eq 2 ]; then pass "T5: exit=$rc"
    else fail "T5: expected 2, got $rc"; fi
fi

# -------- T6 (T-2225): unreachable host declared transient → ok=true, no DRIFT --------
echo "T6: unreachable + declared-transient (env) → ok=true transient_skipped>=1 exit 0"
cat > "$work/hubs-transient.toml" <<EOF
[hubs.t2225-trans-canary]
address = "127.0.0.1:6"
secret_file = "/tmp/nonexistent.hex"
EOF
out="$(FLEET_DM_CANARY_TRANSIENT=t2225-trans-canary bash "$SCRIPT" --hubs-file "$work/hubs-transient.toml" --json --no-heartbeat 2>/dev/null)"
rc=$?
ok_field="$(printf '%s' "$out" | jq -r '.ok' 2>/dev/null || echo "")"
tskip="$(printf '%s' "$out" | jq -r '.summary.transient_skipped // 0' 2>/dev/null || echo "0")"
unreach="$(printf '%s' "$out" | jq -r '.summary.unreachable // 0' 2>/dev/null || echo "0")"
ptrans="$(printf '%s' "$out" | jq -r '.profiles[0].transient // false' 2>/dev/null || echo "")"
if [ "$rc" -eq 0 ] && [ "$ok_field" = "true" ] && [ "$tskip" -ge 1 ] && [ "$unreach" -eq 0 ] && [ "$ptrans" = "true" ]; then
    pass "T6: rc=0 ok=true transient_skipped=$tskip unreachable=$unreach"
else
    fail "T6: rc=$rc ok=$ok_field transient_skipped=$tskip unreachable=$unreach profile.transient=$ptrans out=$out"
fi

# -------- T7 (T-2225): unreachable host NOT declared transient → still DRIFT (regression guard) --------
echo "T7: unreachable + NOT transient → ok=false (skip must not leak to undeclared hosts)"
out="$(bash "$SCRIPT" --hubs-file "$work/hubs-transient.toml" --json --no-heartbeat 2>/dev/null)"
rc=$?
ok_field="$(printf '%s' "$out" | jq -r '.ok' 2>/dev/null || echo "")"
tskip="$(printf '%s' "$out" | jq -r '.summary.transient_skipped // 0' 2>/dev/null || echo "0")"
if [ "$rc" -eq 1 ] && [ "$ok_field" = "false" ] && [ "$tskip" -eq 0 ]; then
    pass "T7: rc=1 ok=false transient_skipped=0 (undeclared host still DRIFTs)"
else
    fail "T7: rc=$rc ok=$ok_field transient_skipped=$tskip out=$out"
fi

echo ""
echo "Results: $PASS pass / $FAIL fail / $SKIP skip"
[ "$FAIL" -eq 0 ]
