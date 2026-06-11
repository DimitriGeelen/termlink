#!/usr/bin/env bash
# T-2152 — canonical substrate worker-side pickup loop.
#
# Companion to scripts/substrate-orchestrator-loop.sh (T-2148).
# Together they wire the full T-2018 §6 work-stealing pattern:
#
#     orchestrator                                worker (THIS script)
#     ============                                ====================
#     subscribe work-topic                        loop:
#       → find-idle worker                          poll agent inbox (--json)
#       → claim(offset)                             pick next unread dm:* topic
#       → claim-transfer(orch → worker)             read latest envelope (--resume)
#       → DM "claim=X topic=Y offset=Z" ───────►    decode payload_b64
#                                                    parse claim/topic/offset
#                                                    spawn substrate-worker-loop.sh
#                                                       --claim-id X --topic Y
#                                                       --offset Z (adopted mode)
#
# Composition of:
#   - termlink agent inbox --json     (T-1553)
#   - termlink channel subscribe --resume --limit 1 --json
#   - scripts/substrate-worker-loop.sh (T-2146 + T-2150 adopted-claim)
#
# Lifecycle (per envelope):
#   1. Poll inbox at --poll-ms cadence for any dm:* topic with unread > 0.
#   2. Fetch the next envelope (subscribe --resume advances the persistent
#      cursor — no re-reading on restart).
#   3. Decode payload_b64 → parse `claim=ID topic=T offset=N`.
#      Malformed payload → log + skip (e.g. chat-arc message, not dispatch).
#   4. Spawn substrate-worker-loop.sh in adopted-claim mode.
#   5. Wait for worker-loop to exit; surface its exit code in our log.
#   6. Loop until --max-claims (default 0 = unlimited) or signal.
#
# SIGTERM/SIGINT → exit 130 (pickup-loop convention).
#
# Exit codes:
#   0    --max-claims reached, loop exited cleanly
#   2    usage / missing flag / jq missing
#   3    --worker-id unresolved
#   4    preflight refused start (T-2166, e.g. TERMLINK_RUNTIME_DIR on /tmp)
#   130  SIGTERM/SIGINT
#
# Requires: jq (envelope parsing), base64 (payload decode).

set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKER_LOOP="${SCRIPT_DIR}/substrate-worker-loop.sh"
STATE_DIR="${TERMLINK_STATE_DIR:-${HOME}/.termlink}"
BE_REACHABLE_STATE="${STATE_DIR}/be-reachable.state"

WORKER_ID=""
CMD_TEMPLATE=""
HUB=""
POLL_MS=2000
MAX_CLAIMS=0
TEST_PARSE=""
SKIP_PREFLIGHT=0

CLAIMS_DISPATCHED=0
CURRENT_WORKER_PID=""

usage() {
    cat <<'EOF'
Usage: substrate-worker-pickup.sh --cmd 'CMD-TEMPLATE' [options]

Worker-side pickup loop — turns orchestrator DMs into substrate-worker-loop.sh
spawns. Pairs with scripts/substrate-orchestrator-loop.sh (T-2148).

Required:
  --cmd 'CMD'             Shell command to run for each claimed unit.
                          Receives TERMLINK_CLAIM_ID, TERMLINK_CLAIM_TOPIC,
                          TERMLINK_CLAIM_OFFSET, TERMLINK_CLAIMER in env.
                          Use single-quotes so the env vars are seen by the
                          spawned worker, not by this shell.

Options:
  --worker-id ID          Worker identity (claimer). Default: $TERMLINK_AGENT_ID
                          env, then ~/.termlink/be-reachable.state (T-1841).
                          Refuses if neither is set — no implicit claimer.
  --hub addr              Target hub. Default: local hub.
  --poll-ms N             Inbox poll cadence, ms. Default 2000.
                          Smaller is more responsive but heavier on the hub.
  --max-claims N          Stop after dispatching N claims (smoke / bounded
                          runs). Default 0 = unlimited.
  --test-parse 'PAYLOAD'  Diagnostic mode: parse the given payload as if it
                          were an orchestrator dispatch DM and print the
                          extracted claim/topic/offset, then exit. No hub
                          contact, no worker-loop spawn. Exit 0 if all three
                          fields parsed; exit 1 if any missing.
                          Use for verifying orchestrator-DM-format
                          compatibility without setting up a live workload.
  --skip-preflight        Skip the startup substrate-preflight.sh call
                          (CI/test paths where preflight is already known
                          clean). T-2166. Default: run preflight.
  -h, --help              Print this help and exit 0.

Exit codes:
  0    --max-claims reached, loop exited cleanly
  2    usage error / missing flag / jq missing
  3    --worker-id unresolved
  4    preflight refused start (T-2166, e.g. TERMLINK_RUNTIME_DIR on /tmp)
  130  SIGTERM/SIGINT

Requires: jq (envelope parsing), base64 (payload decode).

Examples:
  # Long-running worker accepting dispatches from any orchestrator:
  substrate-worker-pickup.sh --worker-id deploy-worker-a \
                             --cmd 'python3 /opt/myapp/run.py \
                                    --topic "$TERMLINK_CLAIM_TOPIC" \
                                    --offset "$TERMLINK_CLAIM_OFFSET"'

  # Bounded smoke run (1 claim then exit):
  substrate-worker-pickup.sh --cmd 'true' --max-claims 1

See: docs/operations/substrate-orchestrator-recipe.md (T-2124 master recipe)
EOF
}

die() {
    echo "substrate-worker-pickup.sh: $1" >&2
    exit "${2:-2}"
}

log() {
    echo "substrate-worker-pickup.sh: $1" >&2
}

resolve_worker_id() {
    if [ -n "$WORKER_ID" ]; then
        return 0
    fi
    if [ -n "${TERMLINK_AGENT_ID:-}" ]; then
        WORKER_ID="$TERMLINK_AGENT_ID"
        return 0
    fi
    if [ -r "$BE_REACHABLE_STATE" ]; then
        local id
        id=$(grep -o '"agent_id"[[:space:]]*:[[:space:]]*"[^"]*"' "$BE_REACHABLE_STATE" \
             | sed 's/.*"agent_id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/')
        if [ -n "$id" ]; then
            WORKER_ID="$id"
            return 0
        fi
    fi
    die "worker-id unresolved — pass --worker-id, set \$TERMLINK_AGENT_ID, or run /be-reachable first" 3
}

run_preflight() {
    # T-2166: run substrate-preflight.sh as a startup gate (mirror of T-2163
    # on substrate-worker-loop.sh + substrate-orchestrator-loop.sh).
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
        echo "substrate-worker-pickup.sh: preflight script not found at $preflight — continuing without check" >&2
        return 0
    fi

    pf_out=$("$preflight" 2>&1)
    pf_rc=$?

    case "$pf_rc" in
        0) return 0 ;;
        1)
            echo "substrate-worker-pickup.sh: WARNING — substrate-preflight reported warnings:" >&2
            echo "$pf_out" >&2
            echo "substrate-worker-pickup.sh: continuing despite WARN (use --skip-preflight to suppress)" >&2
            return 0
            ;;
        *)
            echo "substrate-worker-pickup.sh: substrate-preflight FAILED (exit $pf_rc) — refusing to start:" >&2
            echo "$pf_out" >&2
            echo "substrate-worker-pickup.sh: fix the failure above OR pass --skip-preflight if you accept the risk" >&2
            exit 4
            ;;
    esac
}

# ---- Arg parsing ---------------------------------------------------------

while [ $# -gt 0 ]; do
    case "$1" in
        --worker-id) WORKER_ID="$2"; shift 2 ;;
        --cmd) CMD_TEMPLATE="$2"; shift 2 ;;
        --hub) HUB="$2"; shift 2 ;;
        --poll-ms) POLL_MS="$2"; shift 2 ;;
        --max-claims) MAX_CLAIMS="$2"; shift 2 ;;
        --test-parse) TEST_PARSE="$2"; shift 2 ;;
        --skip-preflight) SKIP_PREFLIGHT=1; shift ;;
        -h|--help) usage; exit 0 ;;
        --) shift; break ;;
        *) die "unknown flag: $1" 2 ;;
    esac
done

# --- Helpers (need to be defined before --test-parse short-circuit) -------

# Parse `claim=X topic=Y offset=Z` from payload. Exports CLAIM_ID, TOPIC,
# OFFSET into the caller. Returns 0 on success, 1 if any field missing.
parse_dispatch() {
    local payload="$1"
    CLAIM_ID=$(echo "$payload" | grep -oE 'claim=[^ ]+' | sed 's/^claim=//' | head -n1)
    TOPIC=$(echo "$payload" | grep -oE 'topic=[^ ]+' | sed 's/^topic=//' | head -n1)
    OFFSET=$(echo "$payload" | grep -oE 'offset=[0-9]+' | sed 's/^offset=//' | head -n1)
    if [ -z "$CLAIM_ID" ] || [ -z "$TOPIC" ] || [ -z "$OFFSET" ]; then
        return 1
    fi
    return 0
}

# --- Diagnostic short-circuit: --test-parse mode --------------------------

if [ -n "$TEST_PARSE" ]; then
    if parse_dispatch "$TEST_PARSE"; then
        printf 'claim_id=%s\ntopic=%s\noffset=%s\n' "$CLAIM_ID" "$TOPIC" "$OFFSET"
        exit 0
    else
        printf 'parse-failed (missing field)\nclaim_id=%s\ntopic=%s\noffset=%s\n' \
            "$CLAIM_ID" "$TOPIC" "$OFFSET" >&2
        exit 1
    fi
fi

[ -n "$CMD_TEMPLATE" ] || die "Usage: --cmd required (see --help)" 2
command -v jq >/dev/null 2>&1 || die "Usage: jq is required (envelope parsing)" 2
command -v base64 >/dev/null 2>&1 || die "Usage: base64 is required (payload decode)" 2
[ -x "$WORKER_LOOP" ] || die "worker-loop not found / not executable at ${WORKER_LOOP}" 2

resolve_worker_id

HUB_ARGS=()
if [ -n "$HUB" ]; then
    HUB_ARGS=(--hub "$HUB")
fi

POLL_S=$(awk -v ms="$POLL_MS" 'BEGIN { printf "%.3f", ms/1000 }')

# ---- Step 0 — preflight gate (T-2166) -----------------------------------
# Catch PL-021 (volatile /tmp) before any inbox poll or worker-loop spawn.
# Silent on PASS; warn and continue on WARN; refuse to start on FAIL
# (exit 4). Bypass via --skip-preflight for CI/test paths where preflight
# is already clean. Mirror of T-2163 on substrate-worker-loop.sh +
# substrate-orchestrator-loop.sh — under systemd Restart=on-failure this
# turns a misconfigured host into a loud restart-loop instead of a
# silently-running supervisor that fails per envelope arrival.
run_preflight

# ---- Signal handling -----------------------------------------------------

cleanup() {
    log "shutting down"
    if [ -n "$CURRENT_WORKER_PID" ] && kill -0 "$CURRENT_WORKER_PID" 2>/dev/null; then
        log "killing in-flight worker pid=$CURRENT_WORKER_PID"
        kill "$CURRENT_WORKER_PID" 2>/dev/null || true
        wait "$CURRENT_WORKER_PID" 2>/dev/null || true
    fi
}

trap 'cleanup; exit 130' INT TERM

# ---- Helpers -------------------------------------------------------------

# Returns one dm:* topic with unread > 0, or empty.
next_unread_dm() {
    "$TERMLINK" agent inbox --json "${HUB_ARGS[@]}" 2>/dev/null \
        | jq -r '.[]? | select(.unread > 0 and (.topic | startswith("dm:"))) | .topic' \
        | head -n1
}

# Fetch the next envelope from a DM topic. --resume advances the cursor.
# Decoded payload printed on stdout (empty if none).
fetch_decoded_payload() {
    local topic="$1"
    local env_json
    env_json=$("$TERMLINK" channel subscribe "$topic" --resume --limit 1 --json \
               "${HUB_ARGS[@]}" 2>/dev/null) || return 0
    if [ -z "$env_json" ]; then
        return 0
    fi
    local p64
    p64=$(echo "$env_json" | jq -r '.payload_b64 // empty')
    if [ -z "$p64" ]; then
        return 0
    fi
    echo "$p64" | base64 -d 2>/dev/null
}

# ---- Main loop -----------------------------------------------------------

log "worker_id=$WORKER_ID poll=${POLL_MS}ms max_claims=${MAX_CLAIMS} (0=unlimited)"
log "polling inbox for dm:* dispatches — Ctrl-C to stop"

while true; do
    dm_topic=$(next_unread_dm)
    if [ -z "$dm_topic" ]; then
        sleep "$POLL_S"
        continue
    fi

    payload=$(fetch_decoded_payload "$dm_topic")
    if [ -z "$payload" ]; then
        # No envelope or empty payload — already consumed by --resume cursor.
        continue
    fi

    if ! parse_dispatch "$payload"; then
        log "non-dispatch DM on $dm_topic (skipped): $payload"
        continue
    fi

    log "dispatch received on $dm_topic: claim=$CLAIM_ID topic=$TOPIC offset=$OFFSET"

    # shellcheck disable=SC2086
    "$WORKER_LOOP" \
        --topic "$TOPIC" --offset "$OFFSET" \
        --claim-id "$CLAIM_ID" --claimer "$WORKER_ID" \
        --cmd "$CMD_TEMPLATE" \
        "${HUB_ARGS[@]}" &
    CURRENT_WORKER_PID=$!
    wait "$CURRENT_WORKER_PID"
    rc=$?
    CURRENT_WORKER_PID=""
    log "worker-loop exit=$rc for claim=$CLAIM_ID"

    CLAIMS_DISPATCHED=$(( CLAIMS_DISPATCHED + 1 ))
    if [ "$MAX_CLAIMS" -gt 0 ] && [ "$CLAIMS_DISPATCHED" -ge "$MAX_CLAIMS" ]; then
        log "--max-claims=$MAX_CLAIMS reached — exiting cleanly"
        exit 0
    fi
done
