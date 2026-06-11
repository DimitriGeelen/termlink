#!/usr/bin/env bash
# T-2170 — substrate-preflight.sh regression test.
#
# Protects scripts/substrate-preflight.sh (T-2154) — the load-bearing
# deploy-time check that three loop scripts depend on:
#
#   substrate-orchestrator-loop.sh (T-2163)
#   substrate-worker-loop.sh       (T-2163)
#   substrate-worker-pickup.sh     (T-2166)
#
# All three gate on the preflight's exit-code contract:
#
#   exit 0 — PASS
#   exit 1 — WARN (>=1 medium, no high-fail)
#   exit 2 — FAIL (>=1 high-fail, substrate WILL silently misbehave)
#
# If a future edit silently flips a fail-classified check to return
# exit 1 instead of exit 2, every production install loses PL-021
# prevention with no signal. The exit-code mapping itself has no
# regression coverage today. This smoke fills that gap.
#
# Mechanism: spawn substrate-preflight.sh under controlled env
# (HOME override + TERMLINK_RUNTIME_DIR override) per scenario and
# assert exit code + envelope shape. Pure read; no network; no auth;
# no state mutation outside a process-private tmpdir.
#
# Symmetric companion to substrate-systemd-smoke.sh (T-2169):
#   T-2169 protects the install path (systemd templates)
#   T-2170 protects the deploy-time check itself (preflight script)
#
# Exit codes:
#   0   every stage PASSed (SKIPs allowed)
#   1   any stage FAILed
#   2   usage / missing dependency
#
# Usage:
#   substrate-preflight-smoke.sh [--json] [--help]

set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PREFLIGHT="${SCRIPT_DIR}/substrate-preflight.sh"

JSON_MODE=0
STAGES_PASSED=()
STAGES_FAILED=()
STAGES_SKIPPED=()
ERRORS_OUT=""

usage() {
    cat <<'EOF'
Usage: substrate-preflight-smoke.sh [--json] [--help]

Regression-tests scripts/substrate-preflight.sh by spawning it under
controlled env per scenario and asserting exit code + JSON envelope.

Stages:
  1. PASS path: runtime_dir off /tmp + hubs.toml present + no
     be-reachable.state → exit 0
  2. hubs.toml WARN path: empty HOME (no hubs.toml) → exit 1
  3. be-reachable WARN path: state file with dead pid → exit 1
  4. runtime_dir FAIL path: TERMLINK_RUNTIME_DIR=/tmp/* on volatile
     /tmp → exit 2 (SKIPs gracefully if /tmp not detected as volatile)
  5. --json envelope: shape verified via jq
  6. --quiet PASS path: empty-log canary (no output, exit 0)
  7. --quiet FAIL path: timestamp-framed output
  8. --help: exit 0
  9. Negative test: mutated preflight (FAIL → exit 1 swap) caught

Options:
  --json    Emit a machine-readable envelope
              {ok, stages_passed, stages_failed, stages_skipped, errors}
  --help    Show this help and exit

Exit codes:
  0  every stage PASSed (SKIPs are fine)
  1  any stage FAILed
  2  usage error / preflight script missing
EOF
}

# ── helpers ────────────────────────────────────────────────────────────

stage_pass() {
    STAGES_PASSED+=("$1")
    [ "$JSON_MODE" -eq 1 ] || echo "PASS  $1"
}

stage_fail() {
    local stage="$1"; shift
    STAGES_FAILED+=("$stage")
    local msg="$*"
    ERRORS_OUT="${ERRORS_OUT}${stage}: ${msg}"$'\n'
    [ "$JSON_MODE" -eq 1 ] || {
        echo "FAIL  $stage" >&2
        echo "      $msg" >&2
    }
}

stage_skip() {
    local stage="$1"; shift
    STAGES_SKIPPED+=("$stage")
    local reason="$*"
    [ "$JSON_MODE" -eq 1 ] || echo "SKIP  $stage — $reason"
}

# Run preflight with controlled env. Echoes "<rc>|<output>".
run_preflight() {
    local home_dir="$1" runtime_dir="$2" extra_flag="${3:-}"
    local out
    out=$(HOME="$home_dir" TERMLINK_RUNTIME_DIR="$runtime_dir" \
        bash "$PREFLIGHT" $extra_flag 2>&1)
    local rc=$?
    printf '%s\n---SMOKE-RC---\n%s' "$out" "$rc"
}

parse_rc() {
    # Reads "<out>---SMOKE-RC---<rc>" from stdin, prints rc to stdout.
    awk -v RS='---SMOKE-RC---' 'NR==2 { gsub(/^[ \n]+|[ \n]+$/, ""); print }'
}

parse_out() {
    # Reads "<out>---SMOKE-RC---<rc>" from stdin, prints out to stdout.
    awk -v RS='---SMOKE-RC---' 'NR==1 { print }'
}

# Detect whether /tmp is volatile per the same heuristic the preflight uses.
# Returns 0 if volatile, 1 if not, on stdout for diagnostic.
tmp_is_volatile() {
    if mount 2>/dev/null | grep -qE "^tmpfs on /tmp\b"; then
        echo "tmpfs mount"
        return 0
    fi
    if [ -r /usr/lib/tmpfiles.d/tmp.conf ] && \
       grep -qE '^[Dd][[:space:]]+/tmp([[:space:]]|$)' /usr/lib/tmpfiles.d/tmp.conf 2>/dev/null; then
        echo "tmpfiles.d D-rule (/usr/lib)"
        return 0
    fi
    for f in /etc/tmpfiles.d/*.conf; do
        [ -r "$f" ] || continue
        if grep -qE '^[Dd][[:space:]]+/tmp([[:space:]]|$)' "$f" 2>/dev/null; then
            echo "tmpfiles.d D-rule ($f)"
            return 0
        fi
    done
    echo "(not detected as volatile)"
    return 1
}

# ── parse args ─────────────────────────────────────────────────────────

while [ $# -gt 0 ]; do
    case "$1" in
        --json) JSON_MODE=1; shift ;;
        --help|-h) usage; exit 0 ;;
        *) echo "substrate-preflight-smoke.sh: unknown arg: $1" >&2; usage >&2; exit 2 ;;
    esac
done

# ── pre-flight: preflight script exists ────────────────────────────────

if [ ! -f "$PREFLIGHT" ]; then
    [ "$JSON_MODE" -eq 1 ] && printf '{"ok":false,"stages_passed":[],"stages_failed":["preflight.exists"],"stages_skipped":[],"errors":"%s not found"}\n' "$PREFLIGHT"
    [ "$JSON_MODE" -eq 1 ] || echo "FAIL  preflight.exists — $PREFLIGHT not found" >&2
    exit 2
fi
if [ ! -x "$PREFLIGHT" ]; then
    [ "$JSON_MODE" -eq 1 ] && printf '{"ok":false,"stages_passed":[],"stages_failed":["preflight.executable"],"stages_skipped":[],"errors":"%s not +x"}\n' "$PREFLIGHT"
    [ "$JSON_MODE" -eq 1 ] || echo "FAIL  preflight.executable — $PREFLIGHT not +x" >&2
    exit 2
fi

# Tmpdir for controlled HOME overrides. Cleaned at exit.
TMPDIR_ROOT=$(mktemp -d -t substrate-preflight-smoke-XXXXXX)
trap 'rm -rf "$TMPDIR_ROOT"' EXIT

# ── stage 1: PASS path ─────────────────────────────────────────────────

stage_pass_path() {
    local home="${TMPDIR_ROOT}/home-pass"
    mkdir -p "${home}/.termlink"
    # Real-shaped hubs.toml so the [hubs.NAME] regex matches.
    cat > "${home}/.termlink/hubs.toml" <<'EOF'
[hubs.smoke-test]
address = "127.0.0.1:65535"
EOF
    local rc
    rc=$(run_preflight "$home" "/var/lib/termlink" | parse_rc)
    if [ "$rc" = "0" ]; then
        stage_pass "preflight.pass_path"
    else
        stage_fail "preflight.pass_path" "expected exit 0, got $rc"
    fi
}

# ── stage 2: hubs.toml WARN path ───────────────────────────────────────

stage_hubs_toml_warn() {
    local home="${TMPDIR_ROOT}/home-no-hubs"
    mkdir -p "${home}/.termlink"   # but no hubs.toml inside
    local rc
    rc=$(run_preflight "$home" "/var/lib/termlink" | parse_rc)
    if [ "$rc" = "1" ]; then
        stage_pass "preflight.hubs_toml_warn"
    else
        stage_fail "preflight.hubs_toml_warn" "expected exit 1, got $rc"
    fi
}

# ── stage 3: be-reachable WARN path ────────────────────────────────────

stage_be_reachable_warn() {
    local home="${TMPDIR_ROOT}/home-stale-be"
    mkdir -p "${home}/.termlink"
    cat > "${home}/.termlink/hubs.toml" <<'EOF'
[hubs.smoke-test]
address = "127.0.0.1:65535"
EOF
    # PID 999999 almost certainly does not exist; if it does, kill -0 will
    # report alive — vanishingly rare on any real box but not impossible.
    cat > "${home}/.termlink/be-reachable.state" <<'EOF'
{"agent_id": "smoke-stale", "pid": 999999}
EOF
    if kill -0 999999 2>/dev/null; then
        stage_skip "preflight.be_reachable_warn" "PID 999999 happens to be alive on this host"
        return
    fi
    local rc
    rc=$(run_preflight "$home" "/var/lib/termlink" | parse_rc)
    if [ "$rc" = "1" ]; then
        stage_pass "preflight.be_reachable_warn"
    else
        stage_fail "preflight.be_reachable_warn" "expected exit 1, got $rc"
    fi
}

# ── stage 4: runtime_dir FAIL path ─────────────────────────────────────

stage_runtime_dir_fail() {
    local volatility_reason
    volatility_reason=$(tmp_is_volatile)
    local volatile_rc=$?
    if [ "$volatile_rc" -ne 0 ]; then
        stage_skip "preflight.runtime_dir_fail" "/tmp not detected as volatile on this host: $volatility_reason"
        return
    fi
    local home="${TMPDIR_ROOT}/home-tmp-runtime"
    mkdir -p "${home}/.termlink"
    cat > "${home}/.termlink/hubs.toml" <<'EOF'
[hubs.smoke-test]
address = "127.0.0.1:65535"
EOF
    local rc
    rc=$(run_preflight "$home" "/tmp/substrate-preflight-smoke-test" | parse_rc)
    if [ "$rc" = "2" ]; then
        stage_pass "preflight.runtime_dir_fail"
    else
        stage_fail "preflight.runtime_dir_fail" "expected exit 2 (volatile-/tmp detected as $volatility_reason), got $rc"
    fi
}

# ── stage 5: --json envelope shape ─────────────────────────────────────

stage_json_envelope() {
    if ! command -v jq >/dev/null 2>&1; then
        stage_skip "preflight.json_envelope" "jq not available"
        return
    fi
    local home="${TMPDIR_ROOT}/home-json"
    mkdir -p "${home}/.termlink"
    cat > "${home}/.termlink/hubs.toml" <<'EOF'
[hubs.smoke-test]
address = "127.0.0.1:65535"
EOF
    local out
    out=$(run_preflight "$home" "/var/lib/termlink" "--json" | parse_out)
    # Shape: {ok, exit_code, checks: [...], summary: {pass, warn, fail}}
    if echo "$out" | jq -e '.ok != null and .exit_code != null and (.checks | type == "array") and (.summary.pass != null) and (.summary.warn != null) and (.summary.fail != null)' >/dev/null 2>&1; then
        stage_pass "preflight.json_envelope"
    else
        stage_fail "preflight.json_envelope" "envelope shape mismatch: $(echo "$out" | head -c 200)"
    fi
}

# ── stage 6: --quiet PASS path silent ──────────────────────────────────

stage_quiet_pass_silent() {
    local home="${TMPDIR_ROOT}/home-quiet-pass"
    mkdir -p "${home}/.termlink"
    cat > "${home}/.termlink/hubs.toml" <<'EOF'
[hubs.smoke-test]
address = "127.0.0.1:65535"
EOF
    local result
    result=$(run_preflight "$home" "/var/lib/termlink" "--quiet")
    local rc; rc=$(printf '%s' "$result" | parse_rc)
    local out; out=$(printf '%s' "$result" | parse_out)
    if [ "$rc" = "0" ] && [ -z "$out" ]; then
        stage_pass "preflight.quiet_pass_silent"
    else
        stage_fail "preflight.quiet_pass_silent" "expected exit 0 + no output, got rc=$rc output=[$out]"
    fi
}

# ── stage 7: --quiet FAIL path framed ──────────────────────────────────

stage_quiet_fail_framed() {
    local volatility_reason
    volatility_reason=$(tmp_is_volatile)
    if [ $? -ne 0 ]; then
        stage_skip "preflight.quiet_fail_framed" "/tmp not detected as volatile on this host"
        return
    fi
    local home="${TMPDIR_ROOT}/home-quiet-fail"
    mkdir -p "${home}/.termlink"
    cat > "${home}/.termlink/hubs.toml" <<'EOF'
[hubs.smoke-test]
address = "127.0.0.1:65535"
EOF
    local result
    result=$(run_preflight "$home" "/tmp/substrate-preflight-smoke-test" "--quiet")
    local rc; rc=$(printf '%s' "$result" | parse_rc)
    local out; out=$(printf '%s' "$result" | parse_out)
    # Expect: exit 2 + output frame starting with "=== <ts> ===" line.
    if [ "$rc" = "2" ] && echo "$out" | grep -qE '^=== [0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z ===$'; then
        stage_pass "preflight.quiet_fail_framed"
    else
        stage_fail "preflight.quiet_fail_framed" "expected exit 2 + ts-framed output, got rc=$rc; first line: $(echo "$out" | head -n1)"
    fi
}

# ── stage 8: --help exits 0 ────────────────────────────────────────────

stage_help_exits_zero() {
    if bash "$PREFLIGHT" --help >/dev/null 2>&1; then
        stage_pass "preflight.help_exits_zero"
    else
        stage_fail "preflight.help_exits_zero" "--help exited non-zero"
    fi
}

# ── stage 9: negative test — mutated preflight caught ──────────────────

stage_negative_mutation_caught() {
    # Mutate a copy of preflight to weaken the contract: swap the
    # high-severity FAIL classification's exit-code from 2 down to 1.
    # The PASS-path smoke (stage 1) should still pass under this mutation,
    # but stage 4 (runtime_dir FAIL) MUST detect it (expected 2, got 1).
    local mutated="${TMPDIR_ROOT}/preflight-mutated.sh"
    cp "$PREFLIGHT" "$mutated"
    # Replace `EXIT_RC=2` (the high-fail upgrade path) with `EXIT_RC=1`.
    if ! grep -q 'EXIT_RC=2' "$mutated"; then
        stage_skip "preflight.negative_mutation_caught" "preflight source no longer has 'EXIT_RC=2' literal — mutation point moved"
        return
    fi
    sed -i 's/EXIT_RC=2/EXIT_RC=1/g' "$mutated"
    chmod +x "$mutated"

    local volatility_reason
    volatility_reason=$(tmp_is_volatile)
    if [ $? -ne 0 ]; then
        stage_skip "preflight.negative_mutation_caught" "/tmp not detected as volatile — cannot trigger FAIL path"
        return
    fi

    local home="${TMPDIR_ROOT}/home-mutation"
    mkdir -p "${home}/.termlink"
    cat > "${home}/.termlink/hubs.toml" <<'EOF'
[hubs.smoke-test]
address = "127.0.0.1:65535"
EOF
    # Run the MUTATED preflight in the FAIL scenario. With the mutation,
    # rc should be 1 instead of the contracted 2 — proving the smoke
    # would catch this class of regression.
    local rc
    rc=$(HOME="$home" TERMLINK_RUNTIME_DIR="/tmp/substrate-preflight-smoke-test" \
        bash "$mutated" 2>&1 >/dev/null; echo $?)
    # Re-run with the args structure used elsewhere to capture rc cleanly.
    local out
    out=$(HOME="$home" TERMLINK_RUNTIME_DIR="/tmp/substrate-preflight-smoke-test" \
        bash "$mutated" 2>&1)
    local mutated_rc=$?

    if [ "$mutated_rc" = "1" ]; then
        # The mutated preflight returns 1 where the contract says 2 —
        # this is exactly the regression the smoke is designed to catch.
        stage_pass "preflight.negative_mutation_caught"
    else
        stage_fail "preflight.negative_mutation_caught" "mutated preflight should have returned 1 (proving smoke would catch a 2→1 swap), got $mutated_rc"
    fi
}

# ── run all stages ─────────────────────────────────────────────────────

stage_pass_path
stage_hubs_toml_warn
stage_be_reachable_warn
stage_runtime_dir_fail
stage_json_envelope
stage_quiet_pass_silent
stage_quiet_fail_framed
stage_help_exits_zero
stage_negative_mutation_caught

# ── render ─────────────────────────────────────────────────────────────

passed_count=${#STAGES_PASSED[@]}
failed_count=${#STAGES_FAILED[@]}
skipped_count=${#STAGES_SKIPPED[@]}

if [ "$JSON_MODE" -eq 1 ]; then
    passed_json="[]"
    if [ "$passed_count" -gt 0 ]; then
        passed_json=$(printf '"%s",' "${STAGES_PASSED[@]}" | sed 's/,$//')
        passed_json="[${passed_json}]"
    fi
    failed_json="[]"
    if [ "$failed_count" -gt 0 ]; then
        failed_json=$(printf '"%s",' "${STAGES_FAILED[@]}" | sed 's/,$//')
        failed_json="[${failed_json}]"
    fi
    skipped_json="[]"
    if [ "$skipped_count" -gt 0 ]; then
        skipped_json=$(printf '"%s",' "${STAGES_SKIPPED[@]}" | sed 's/,$//')
        skipped_json="[${skipped_json}]"
    fi
    errors_json=$(printf '%s' "$ERRORS_OUT" \
        | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' \
        | awk 'BEGIN{ORS=""} {if(NR>1) print "\\n"; print}')
    ok="true"
    [ "$failed_count" -gt 0 ] && ok="false"
    printf '{"ok":%s,"stages_passed":%s,"stages_failed":%s,"stages_skipped":%s,"errors":"%s"}\n' \
        "$ok" "$passed_json" "$failed_json" "$skipped_json" "$errors_json"
else
    echo
    echo "Summary: ${passed_count} PASS / ${failed_count} FAIL / ${skipped_count} SKIP"
fi

[ "$failed_count" -eq 0 ] && exit 0
exit 1
