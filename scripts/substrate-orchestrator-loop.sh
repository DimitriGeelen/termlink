#!/usr/bin/env bash
# T-2148 — canonical substrate orchestrator loop (hello-world dispatch side).
#
# Sibling of scripts/substrate-worker-loop.sh (T-2146). Together they cover
# both halves of the T-2018 §6 #1 CLAIM + #2 DISPATCH + #3 TRANSFER pattern:
#
#     subscribe → find-idle → claim → claim-transfer → DM → loop
#
# Composition of existing substrate verbs:
#   - termlink channel subscribe --resume    (work-queue stream)
#   - termlink agent find-idle               (primitive #2, T-2020/T-2045)
#   - termlink channel claim                 (primitive #1, T-2032)
#   - termlink channel claim-transfer        (primitive #3, T-2046)
#   - termlink agent contact                 (doorbell, T-1429)
#
# Read-side observability is unchanged — pair with /claims (T-2093),
# /governor (T-2095), and /substrate (T-2096) to inspect the orchestrator
# in flight. See docs/operations/substrate-orchestrator-recipe.md §
# "Canonical orchestrator pattern" for the prose this script implements.
#
# Usage:
#   substrate-orchestrator-loop.sh --work-topic T [--capability C]
#                                  [--orchestrator-id ID] [--ttl-ms N]
#                                  [--idle-poll-ms N] [--hub addr]
#                                  [--max-envelopes N]
#
# Lifecycle (per envelope on --work-topic):
#   1. subscribe --resume streams envelopes one at a time (cursor advances
#      automatically across orchestrator restarts).
#   2. find-idle worker matching --capability (block-with-backoff if none).
#   3. orchestrator claim(topic, offset). CLAIM_CONFLICT → skip envelope
#      (someone else got it).
#   4. claim-transfer to worker. On failure (e.g. worker became busy mid-step),
#      release the claim (without --ack) so the slot reopens and skip envelope
#      — no claim leaked.
#   5. fire-and-forget DM via `agent contact` so the worker picks up. If the
#      DM fails, the worker still discovers the claim via its inbox poll.
#   6. loop.
#
# SIGTERM/SIGINT → graceful shutdown — release any in-flight claim and exit 130.
#
# Exit codes:
#   0    --max-envelopes reached, loop exited cleanly
#   2    usage error / missing flag / jq missing
#   3    couldn't resolve --orchestrator-id (no flag, env, or be-reachable.state)
#   4    preflight refused start — TERMLINK_RUNTIME_DIR on /tmp etc. (T-2163)
#   130  SIGTERM/SIGINT received during dispatch
#
# Requires: jq (the script parses JSON envelopes from `channel subscribe --json`).
#
# Adapt this script: change the `find-idle` capability filter or the
# `agent contact` payload format to match your workers' expectations.
# The dispatch wiring is what matters.

set -u

# ---- Configuration -------------------------------------------------------

TERMLINK="${TERMLINK_BIN:-termlink}"
STATE_DIR="${TERMLINK_STATE_DIR:-${HOME}/.termlink}"
BE_REACHABLE_STATE="${STATE_DIR}/be-reachable.state"

WORK_TOPIC=""
CAPABILITY=""
ORCHESTRATOR_ID=""
TTL_MS=60000
IDLE_POLL_MS=5000
HUB=""
MAX_ENVELOPES=0   # 0 = unlimited
SKIP_PREFLIGHT=0

# Internal state for signal handling.
IN_FLIGHT_CLAIM_ID=""
SUBSCRIBE_PID=""

# ---- Helpers -------------------------------------------------------------

usage() {
    cat <<'EOF'
Usage: substrate-orchestrator-loop.sh --work-topic T [options]

Required:
  --work-topic T            Topic to subscribe to (envelopes = units of work).
                            Cursor advances via --resume across restarts.

Options:
  --capability C            Worker filter — only dispatch to workers
                            advertising this capability (default: any LIVE
                            worker). See `termlink agent find-idle --help`.
  --orchestrator-id ID      Orchestrator identity (claimer label). Default:
                            $TERMLINK_AGENT_ID env, then
                            ~/.termlink/be-reachable.state (T-1841).
                            Refuses if neither is set — no implicit identity.
  --ttl-ms N                Initial claim lease, ms. Default 60000.
                            (Worker can renew via channel renew.)
  --idle-poll-ms N          Find-idle backoff when no worker available, ms.
                            Default 5000.
  --hub addr                Target hub. Default: local hub.
  --max-envelopes N         Stop after dispatching N envelopes (smoke testing
                            / bounded runs). Default 0 = unlimited.
  --skip-preflight          Skip the startup substrate-preflight.sh call
                            (CI/test paths where preflight is already known
                            clean). T-2163. Default: run preflight.
  -h, --help                Print this help and exit 0.

Exit codes:
  0    --max-envelopes reached, loop exited cleanly
  2    usage error / missing flag / jq missing
  3    --orchestrator-id unresolved
  4    preflight refused start (T-2163, e.g. TERMLINK_RUNTIME_DIR on /tmp)
  130  SIGTERM/SIGINT received during dispatch

Requires: jq (parses JSON envelopes from channel subscribe --json)

Examples:
  # Dispatch every envelope on aef:deploy to any LIVE deploy-capable worker:
  substrate-orchestrator-loop.sh --work-topic aef:deploy \
                                 --capability deploy

  # Bounded smoke run (3 envelopes, 30s leases, 2s find-idle backoff):
  substrate-orchestrator-loop.sh --work-topic smoke:work-queue \
                                 --orchestrator-id smoke-orch \
                                 --ttl-ms 30000 --idle-poll-ms 2000 \
                                 --max-envelopes 3

See: docs/operations/substrate-orchestrator-recipe.md (T-2124 master recipe)
EOF
}

die() {
    echo "substrate-orchestrator-loop.sh: $1" >&2
    exit "${2:-2}"
}

log() {
    echo "substrate-orchestrator-loop.sh: $1" >&2
}

resolve_orchestrator_id() {
    if [ -n "$ORCHESTRATOR_ID" ]; then
        return 0
    fi
    if [ -n "${TERMLINK_AGENT_ID:-}" ]; then
        ORCHESTRATOR_ID="$TERMLINK_AGENT_ID"
        return 0
    fi
    if [ -r "$BE_REACHABLE_STATE" ]; then
        local id
        id=$(grep -o '"agent_id"[[:space:]]*:[[:space:]]*"[^"]*"' "$BE_REACHABLE_STATE" \
             | sed 's/.*"agent_id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/')
        if [ -n "$id" ]; then
            ORCHESTRATOR_ID="$id"
            return 0
        fi
    fi
    die "orchestrator-id unresolved — pass --orchestrator-id, set \$TERMLINK_AGENT_ID, or run /be-reachable first" 3
}

run_preflight() {
    # T-2163: run substrate-preflight.sh as a startup gate.
    #   exit 0 (PASS) → silent, continue
    #   exit 1 (WARN) → print body + WARN line to stderr, continue
    #   exit 2 (FAIL) → print body to stderr, refuse to start (exit 4)
    # Locates sibling script via $0 dirname so install-location doesn't matter.
    # Missing preflight script → warn + continue (defensive — don't block on
    # absent tooling; the canary covers the long-running drift case).
    [ "$SKIP_PREFLIGHT" -eq 1 ] && return 0

    local self_dir preflight pf_out pf_rc
    self_dir=$(cd "$(dirname "$0")" && pwd)
    preflight="$self_dir/substrate-preflight.sh"

    if [ ! -x "$preflight" ]; then
        echo "substrate-orchestrator-loop.sh: preflight script not found at $preflight — continuing without check" >&2
        return 0
    fi

    pf_out=$("$preflight" 2>&1)
    pf_rc=$?

    case "$pf_rc" in
        0) return 0 ;;
        1)
            echo "substrate-orchestrator-loop.sh: WARNING — substrate-preflight reported warnings:" >&2
            echo "$pf_out" >&2
            echo "substrate-orchestrator-loop.sh: continuing despite WARN (use --skip-preflight to suppress)" >&2
            return 0
            ;;
        *)
            echo "substrate-orchestrator-loop.sh: substrate-preflight FAILED (exit $pf_rc) — refusing to start:" >&2
            echo "$pf_out" >&2
            echo "substrate-orchestrator-loop.sh: fix the failure above OR pass --skip-preflight if you accept the risk" >&2
            exit 4
            ;;
    esac
}

# ---- Arg parsing ---------------------------------------------------------

while [ $# -gt 0 ]; do
    case "$1" in
        --work-topic) WORK_TOPIC="$2"; shift 2 ;;
        --capability) CAPABILITY="$2"; shift 2 ;;
        --orchestrator-id) ORCHESTRATOR_ID="$2"; shift 2 ;;
        --ttl-ms) TTL_MS="$2"; shift 2 ;;
        --idle-poll-ms) IDLE_POLL_MS="$2"; shift 2 ;;
        --hub) HUB="$2"; shift 2 ;;
        --max-envelopes) MAX_ENVELOPES="$2"; shift 2 ;;
        --skip-preflight) SKIP_PREFLIGHT=1; shift ;;
        -h|--help) usage; exit 0 ;;
        --) shift; break ;;
        *) die "unknown flag: $1" 2 ;;
    esac
done

[ -n "$WORK_TOPIC" ] || die "Usage: --work-topic required (see --help)" 2
command -v jq >/dev/null 2>&1 || die "Usage: jq is required (parses JSON envelopes from channel subscribe)" 2

resolve_orchestrator_id

# T-2163: preflight gate — catch PL-021 (volatile /tmp) BEFORE any hub call.
# Silent on PASS; warn-and-continue on WARN; refuse-to-start (exit 4) on FAIL.
# Bypass via --skip-preflight in CI/test paths where preflight is already clean.
run_preflight

HUB_ARGS=()
if [ -n "$HUB" ]; then
    HUB_ARGS=(--hub "$HUB")
fi

CAPABILITY_ARGS=()
if [ -n "$CAPABILITY" ]; then
    CAPABILITY_ARGS=(--capability "$CAPABILITY")
fi

IDLE_POLL_S=$(awk -v ms="$IDLE_POLL_MS" 'BEGIN { printf "%.3f", ms/1000 }')

# ---- Signal handling -----------------------------------------------------

cleanup() {
    log "shutting down"
    # Release any in-flight claim so it doesn't sit there until TTL lapse.
    if [ -n "$IN_FLIGHT_CLAIM_ID" ]; then
        log "releasing in-flight claim $IN_FLIGHT_CLAIM_ID"
        "$TERMLINK" channel release --json "${HUB_ARGS[@]}" \
            --claim-id "$IN_FLIGHT_CLAIM_ID" --claimer "$ORCHESTRATOR_ID" \
            >/dev/null 2>&1 || true
        IN_FLIGHT_CLAIM_ID=""
    fi
    # Stop the subscribe stream child.
    if [ -n "$SUBSCRIBE_PID" ] && kill -0 "$SUBSCRIBE_PID" 2>/dev/null; then
        kill "$SUBSCRIBE_PID" 2>/dev/null || true
        wait "$SUBSCRIBE_PID" 2>/dev/null || true
    fi
}

trap 'cleanup; exit 130' INT TERM

# ---- Find-idle with backoff ---------------------------------------------

find_idle_worker() {
    # Returns one agent_id on stdout, or empty if none found.
    "$TERMLINK" agent find-idle --json "${HUB_ARGS[@]}" \
        "${CAPABILITY_ARGS[@]}" --limit 1 2>/dev/null \
        | jq -r '.idle[0].agent_id // empty'
}

wait_for_idle_worker() {
    local worker=""
    while [ -z "$worker" ]; do
        worker=$(find_idle_worker)
        if [ -z "$worker" ]; then
            sleep "$IDLE_POLL_S"
        fi
    done
    echo "$worker"
}

# ---- Per-envelope dispatch ----------------------------------------------

dispatch_one() {
    local offset="$1"

    log "envelope offset=$offset — waiting for idle worker (capability=${CAPABILITY:-any})"
    local worker
    worker=$(wait_for_idle_worker)
    log "found idle worker: $worker"

    # Step 1 — claim
    local claim_out claim_id
    claim_out=$("$TERMLINK" channel claim --json "${HUB_ARGS[@]}" \
                --claimer "$ORCHESTRATOR_ID" --ttl-ms "$TTL_MS" \
                "$WORK_TOPIC" "$offset" 2>&1) || {
        log "claim failed at offset=$offset — skipping (CLAIM_CONFLICT or other): $claim_out"
        return 0
    }
    claim_id=$(echo "$claim_out" | jq -r '.claim_id // empty')
    if [ -z "$claim_id" ]; then
        log "claim returned no claim_id at offset=$offset — skipping: $claim_out"
        return 0
    fi
    IN_FLIGHT_CLAIM_ID="$claim_id"

    # Step 2 — atomically transfer to worker
    if ! "$TERMLINK" channel claim-transfer --json "${HUB_ARGS[@]}" \
         --claim-id "$claim_id" --to-owner "$worker" \
         --by "$ORCHESTRATOR_ID" --reason "orchestrator dispatch" \
         >/dev/null 2>&1; then
        log "claim-transfer failed for offset=$offset worker=$worker — releasing claim, skipping"
        "$TERMLINK" channel release --json "${HUB_ARGS[@]}" \
            --claim-id "$claim_id" --claimer "$ORCHESTRATOR_ID" \
            >/dev/null 2>&1 || true
        IN_FLIGHT_CLAIM_ID=""
        return 0
    fi
    # Ownership now belongs to the worker — orchestrator is done with this claim.
    IN_FLIGHT_CLAIM_ID=""

    # Step 3 — fire-and-forget DM. Worker also discovers via inbox poll if this fails.
    "$TERMLINK" agent contact "$worker" \
        --message "claim=$claim_id topic=$WORK_TOPIC offset=$offset" \
        >/dev/null 2>&1 || \
        log "DM to $worker failed (worker will still discover via inbox poll)"

    log "dispatched offset=$offset → claim=$claim_id → worker=$worker"
}

# ---- Main loop -----------------------------------------------------------

log "orchestrator=$ORCHESTRATOR_ID work_topic=$WORK_TOPIC capability=${CAPABILITY:-any} ttl=${TTL_MS}ms"
log "subscribing to $WORK_TOPIC --resume — Ctrl-C to stop"

# Subscribe in background to capture PID for cleanup. The substrate's --resume
# advances a persisted cursor so restart-from-this-point Just Works.
COUNT=0
while IFS= read -r envelope; do
    [ -z "$envelope" ] && continue
    offset=$(echo "$envelope" | jq -r '.offset // empty')
    if [ -z "$offset" ]; then
        log "malformed envelope (no .offset), skipping: $envelope"
        continue
    fi
    dispatch_one "$offset"
    COUNT=$(( COUNT + 1 ))
    if [ "$MAX_ENVELOPES" -gt 0 ] && [ "$COUNT" -ge "$MAX_ENVELOPES" ]; then
        log "--max-envelopes=$MAX_ENVELOPES reached — exiting cleanly"
        cleanup
        exit 0
    fi
done < <("$TERMLINK" channel subscribe "$WORK_TOPIC" --resume --json "${HUB_ARGS[@]}" 2>/dev/null & SUBSCRIBE_PID=$!; wait "$SUBSCRIBE_PID")

# Subscribe ended (hub closed stream, etc).
cleanup
exit 0
