#!/usr/bin/env bash
# T-2151 — substrate end-to-end smoke verifier.
#
# Proves the substrate's canonical work-stealing pattern is healthy on
# this host in one command. Each stage is reported with PASS/FAIL so
# operators and CI both get a clear signal.
#
# Pattern exercised (matches docs/operations/substrate-orchestrator-recipe.md):
#
#   create-topic
#     → post one work envelope
#     → claim (as the "orchestrator" role)
#     → claim-transfer (T-2046, primitive #3) → "worker" role
#     → substrate-worker-loop.sh --claim-id (T-2150 adopted-claim mode)
#       (this is the regression gate for the composition path)
#     → verify active=0 + expired=0 (cursor advanced cleanly)
#
# Composition of:
#   - termlink channel create / post / claim / claim-transfer / claims-summary
#   - scripts/substrate-worker-loop.sh (T-2146 + T-2150)
#
# Exit codes:
#   0   substrate healthy — every stage PASSed
#   1   any stage FAILed — the failing stage and error are on stderr
#   2   usage / missing dependency (termlink, scripts/, jq)
#
# Usage:
#   substrate-smoke.sh [--hub addr] [--json] [--help]
#
# Pair with: /substrate (cold-start digest), /find-idle, /claims,
# scripts/substrate-orchestrator-loop.sh, scripts/substrate-worker-loop.sh.

set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKER_LOOP="${SCRIPT_DIR}/substrate-worker-loop.sh"

HUB=""
JSON_MODE=0

# Smoke identity labels — distinct so we exercise the transfer path.
SMOKE_ORCH_ID="substrate-smoke-orch"
SMOKE_WORKER_ID="substrate-smoke-worker"

# Topic name carries a timestamp + pid so concurrent smokes don't collide.
SMOKE_TOPIC="smoke:t2151-$(date -u +%Y%m%d-%H%M%S)-$$"

# Stage tracking — we accumulate then render at the end.
STAGES_PASSED=()
STAGES_FAILED=()
ERRORS_OUT=""
CLAIM_ID=""
OFFSET=""

usage() {
    cat <<'EOF'
Usage: substrate-smoke.sh [--hub addr] [--json] [--help]

Proves the substrate is healthy by running the full canonical
work-stealing pattern end-to-end on this host. Each stage is reported
with PASS/FAIL.

Stages:
  create          channel create on a fresh smoke topic
  post            one work envelope posted (cursor moves)
  claim           orchestrator-role claim succeeds
  transfer        claim-transfer to worker-role succeeds
  worker-loop     worker-loop --claim-id adopts, runs cmd, releases
  verify-clean    claims-summary shows active=0 expired=0

Options:
  --hub addr     Target hub. Default: local hub.
  --json         Emit one envelope {ok, stages, topic, claim_id, errors}
                 instead of human format. Pipe-safe.
  -h, --help     Print this help and exit 0.

Exit codes:
  0   every stage PASSed — substrate is healthy
  1   at least one stage FAILed — the stage and error are on stderr
  2   usage error or missing dependency

Examples:
  scripts/substrate-smoke.sh
  scripts/substrate-smoke.sh --hub 192.168.10.122:9100
  scripts/substrate-smoke.sh --json | jq '.ok'

See: docs/operations/substrate-getting-started.md (T-2149)
EOF
}

log() {
    if [ "$JSON_MODE" -eq 0 ]; then
        echo "substrate-smoke: $1" >&2
    fi
}

stage_pass() {
    STAGES_PASSED+=("$1")
    log "✓ $1"
}

stage_fail() {
    local stage="$1"
    local err="$2"
    STAGES_FAILED+=("$stage")
    ERRORS_OUT+="${stage}: ${err}\n"
    echo "substrate-smoke: FAIL at stage ${stage}: ${err}" >&2
    finalize 1
}

finalize() {
    local rc="$1"
    if [ "$JSON_MODE" -eq 1 ]; then
        local ok_str passed_json failed_json
        if [ "$rc" -eq 0 ]; then
            ok_str="true"
        else
            ok_str="false"
        fi
        if [ "${#STAGES_PASSED[@]}" -eq 0 ]; then
            passed_json="[]"
        else
            passed_json=$(printf '%s\n' "${STAGES_PASSED[@]}" | jq -R . | jq -s -c .)
        fi
        if [ "${#STAGES_FAILED[@]}" -eq 0 ]; then
            failed_json="[]"
        else
            failed_json=$(printf '%s\n' "${STAGES_FAILED[@]}" | jq -R . | jq -s -c .)
        fi
        printf '{"ok":%s,"topic":"%s","claim_id":"%s","stages_passed":%s,"stages_failed":%s,"errors":"%s"}\n' \
            "$ok_str" "$SMOKE_TOPIC" "$CLAIM_ID" \
            "$passed_json" "$failed_json" \
            "$(printf '%b' "$ERRORS_OUT" | sed 's/"/\\"/g' | tr -d '\n')"
    else
        if [ "$rc" -eq 0 ]; then
            echo "substrate smoke: PASS (topic=${SMOKE_TOPIC}, ${#STAGES_PASSED[@]} stages OK)"
        fi
    fi
    exit "$rc"
}

# ---- Arg parsing ---------------------------------------------------------

while [ $# -gt 0 ]; do
    case "$1" in
        --hub) HUB="$2"; shift 2 ;;
        --json) JSON_MODE=1; shift ;;
        -h|--help) usage; exit 0 ;;
        --) shift; break ;;
        *) echo "substrate-smoke: unknown flag: $1" >&2; exit 2 ;;
    esac
done

# ---- Dependency checks ---------------------------------------------------

command -v "$TERMLINK" >/dev/null 2>&1 || {
    echo "substrate-smoke: termlink binary not on PATH (set TERMLINK_BIN to override)" >&2
    exit 2
}
[ -x "$WORKER_LOOP" ] || {
    echo "substrate-smoke: worker-loop not found / not executable at ${WORKER_LOOP}" >&2
    exit 2
}
if [ "$JSON_MODE" -eq 1 ]; then
    command -v jq >/dev/null 2>&1 || {
        echo "substrate-smoke: --json requires jq" >&2
        exit 2
    }
fi

HUB_ARGS=()
if [ -n "$HUB" ]; then
    HUB_ARGS=(--hub "$HUB")
fi

log "smoke topic: $SMOKE_TOPIC"

# ---- Stage: create -------------------------------------------------------

if ! out=$("$TERMLINK" channel create "$SMOKE_TOPIC" --retention messages:100 "${HUB_ARGS[@]}" 2>&1); then
    stage_fail "create" "$out"
fi
stage_pass "create"

# ---- Stage: post ---------------------------------------------------------

if ! out=$("$TERMLINK" channel post "$SMOKE_TOPIC" --payload "substrate-smoke unit-of-work" --json "${HUB_ARGS[@]}" 2>&1); then
    stage_fail "post" "$out"
fi
# Parse offset out — accept both `"offset":N` and `offset=N` shapes.
OFFSET=$(echo "$out" | grep -o '"offset"[[:space:]]*:[[:space:]]*[0-9]*' \
        | sed 's/.*://;s/[[:space:]]//g' | head -n1)
if [ -z "$OFFSET" ]; then
    OFFSET=$(echo "$out" | grep -oE 'offset=[0-9]+' | sed 's/offset=//' | head -n1)
fi
if [ -z "$OFFSET" ]; then
    stage_fail "post" "could not parse offset from: $out"
fi
stage_pass "post (offset=$OFFSET)"

# ---- Stage: claim (orchestrator role) ------------------------------------

if ! out=$("$TERMLINK" channel claim "$SMOKE_TOPIC" "$OFFSET" \
           --claimer "$SMOKE_ORCH_ID" --ttl-ms 60000 --json "${HUB_ARGS[@]}" 2>&1); then
    stage_fail "claim" "$out"
fi
CLAIM_ID=$(echo "$out" | grep -o '"claim_id"[[:space:]]*:[[:space:]]*"[^"]*"' \
           | sed 's/.*"claim_id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/' | head -n1)
if [ -z "$CLAIM_ID" ]; then
    stage_fail "claim" "no claim_id in: $out"
fi
stage_pass "claim (claim_id=$CLAIM_ID)"

# ---- Stage: claim-transfer (orch → worker) -------------------------------

if ! out=$("$TERMLINK" channel claim-transfer \
           --claim-id "$CLAIM_ID" --to-owner "$SMOKE_WORKER_ID" \
           --by "$SMOKE_ORCH_ID" --reason "substrate-smoke transfer" \
           --json "${HUB_ARGS[@]}" 2>&1); then
    # Orchestrator still owns the claim — release it loudly so we don't leak.
    "$TERMLINK" channel release --claim-id "$CLAIM_ID" --claimer "$SMOKE_ORCH_ID" \
        --json "${HUB_ARGS[@]}" >/dev/null 2>&1 || true
    stage_fail "transfer" "$out"
fi
stage_pass "transfer"

# ---- Stage: worker-loop (adopted-claim mode, T-2150) ---------------------

# Use --hub when set; otherwise let worker-loop pick local default.
WORKER_HUB_ARGS=()
if [ -n "$HUB" ]; then
    WORKER_HUB_ARGS=(--hub "$HUB")
fi

# $TERMLINK_CLAIM_ID expands inside the spawned worker process, not here.
# shellcheck disable=SC2016
# --skip-preflight (T-2164): substrate-smoke is a MECHANICAL test (does
# claim → transfer → release work?), not a DEPLOY-CORRECTNESS check (is
# TERMLINK_RUNTIME_DIR off /tmp?). On CI/ephemeral hosts where /tmp is
# the runtime_dir, the T-2163 preflight gate would block this self-contained
# smoke from running. Bypass — deploy-correctness is covered by /preflight
# (T-2158) + the nightly cron canary (T-2160).
if ! out=$("$WORKER_LOOP" \
           --topic "$SMOKE_TOPIC" --offset "$OFFSET" \
           --claim-id "$CLAIM_ID" --claimer "$SMOKE_WORKER_ID" \
           --ttl-ms 30000 --skip-preflight \
           --cmd 'echo "smoke worker ok claim=$TERMLINK_CLAIM_ID"' \
           "${WORKER_HUB_ARGS[@]}" 2>&1); then
    # Worker now owns — attempt to release without ack so we don't leak.
    "$TERMLINK" channel release --claim-id "$CLAIM_ID" --claimer "$SMOKE_WORKER_ID" \
        --json "${HUB_ARGS[@]}" >/dev/null 2>&1 || true
    stage_fail "worker-loop" "$out"
fi
if ! echo "$out" | grep -q "adopting existing claim_id"; then
    stage_fail "worker-loop" "expected adopting-loud-log not seen in: $out"
fi
if ! echo "$out" | grep -q "worker ok"; then
    stage_fail "worker-loop" "expected release(--ack) loud-log not seen in: $out"
fi
stage_pass "worker-loop (adopted-claim path)"

# ---- Stage: verify-clean -------------------------------------------------

if ! out=$("$TERMLINK" channel claims-summary "$SMOKE_TOPIC" --json "${HUB_ARGS[@]}" 2>&1); then
    stage_fail "verify-clean" "$out"
fi
active=$(echo "$out" | grep -o '"active_count"[[:space:]]*:[[:space:]]*[0-9]*' \
         | sed 's/.*://;s/[[:space:]]//g' | head -n1)
expired=$(echo "$out" | grep -o '"expired_count"[[:space:]]*:[[:space:]]*[0-9]*' \
         | sed 's/.*://;s/[[:space:]]//g' | head -n1)
if [ "$active" != "0" ] || [ "$expired" != "0" ]; then
    stage_fail "verify-clean" "active=$active expired=$expired (expected 0/0) in: $out"
fi
stage_pass "verify-clean (active=0 expired=0)"

finalize 0
