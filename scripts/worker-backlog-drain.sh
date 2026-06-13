#!/usr/bin/env bash
# scripts/worker-backlog-drain.sh
#
# T-2205 (T-2018 §6 arc-parallel-substrate): worker-side companion to
# scripts/orchestrator-backlog-drain.sh (T-2204). AEF integrators clone
# both scripts as a complete substrate consumer kit.
#
# Polls for claims held by THIS worker on the work-queue topic, reads
# each claimed work-unit envelope, renders the operator brief, and
# (optionally) releases the claim. Three modes:
#
#   --dry-run    (default) print held claims + intended action; no hub writes
#   --live       print held claims + WAIT for operator ack via stdin
#                ("ack" → release --ack, "retry" → release no-ack,
#                 "skip" → leave claim alone for now)
#   --auto-noop  release --ack immediately on every claim WITHOUT doing
#                the work — substrate smoke-test only, never production
#
# Substrate primitives exercised:
#   #1 claim (READ via claims-summary)   discover held claims
#   #1 release                            cooperative completion
#   agent presence (#11) implied by /be-reachable advertise upstream

set -euo pipefail

# ── defaults ────────────────────────────────────────────────────────────────
QUEUE_TOPIC="work-queue"
MODE="dry-run"
WORKER_ID=""
ONCE=0
POLL_INTERVAL=15
HUB=""

usage() {
  cat <<EOF
worker-backlog-drain.sh — substrate parallel-worker poller (T-2205)

Usage: $0 [--dry-run|--live|--auto-noop] [options]

Modes:
  --dry-run                (default) print held claims + intended action
  --live                   print + prompt operator (stdin) for ack/retry/skip
  --auto-noop              release --ack every claim without doing the work
                           (substrate smoke-test only; never production)

Options:
  --queue-topic NAME       work-queue topic (default: work-queue)
  --worker-id ID           override worker identity (default: be-reachable state)
  --once                   one poll pass and exit (default: poll forever)
  --poll-interval SECS     poll cadence in seconds (default: 15)
  --hub ADDR               hub override (default: local)
  -h, --help               show this help

Exit codes:
  0    poll pass completed (or interactive session exited cleanly)
  1    a release call failed
  2    flag error / worker identity unresolved

Pair with: scripts/orchestrator-backlog-drain.sh (T-2204)
Recipe:    docs/operations/substrate-orchestrator-recipe.md
EOF
}

# ── flag parse ──────────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)        MODE="dry-run"; shift ;;
    --live)           MODE="live"; shift ;;
    --auto-noop)      MODE="auto-noop"; shift ;;
    --queue-topic)    QUEUE_TOPIC="$2"; shift 2 ;;
    --worker-id)      WORKER_ID="$2"; shift 2 ;;
    --once)           ONCE=1; shift ;;
    --poll-interval)  POLL_INTERVAL="$2"; shift 2 ;;
    --hub)            HUB="$2"; shift 2 ;;
    -h|--help)        usage; exit 0 ;;
    *)                echo "unknown flag: $1" >&2; usage >&2; exit 2 ;;
  esac
done

# ── worker identity ─────────────────────────────────────────────────────────
if [ -z "$WORKER_ID" ]; then
  if [ -f ~/.termlink/be-reachable.state ] && command -v jq >/dev/null 2>&1; then
    WORKER_ID="$(jq -r .agent_id ~/.termlink/be-reachable.state 2>/dev/null || true)"
  fi
fi
[ -z "$WORKER_ID" ] && WORKER_ID="${TERMLINK_AGENT_ID:-}"

if [ -z "$WORKER_ID" ]; then
  echo "ERROR: cannot resolve worker identity." >&2
  echo "  Set TERMLINK_AGENT_ID, pass --worker-id, or run '/be-reachable start'." >&2
  exit 2
fi

if ! command -v termlink >/dev/null 2>&1; then
  echo "ERROR: termlink required on PATH." >&2
  exit 2
fi

echo "# worker-backlog-drain.sh — T-2205"
echo "# mode=$MODE worker_id=$WORKER_ID queue_topic=$QUEUE_TOPIC poll_interval=${POLL_INTERVAL}s once=$ONCE"
echo

# ── poll pass ───────────────────────────────────────────────────────────────
poll_pass() {
  echo "# === poll pass $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="

  # Step 1: discover all active claims on the queue topic, filter to mine
  # Use `channel claims` (the LIST verb, T-2037) not `channel claims-summary`
  # (the AGGREGATE verb) — the summary returns counts only, not per-claim rows.
  local claims_json my_claims count
  claims_json="$(timeout 5 termlink channel claims "$QUEUE_TOPIC" --json 2>/dev/null || echo '{"claims":[]}')"
  my_claims="$(echo "$claims_json" | jq -c --arg w "$WORKER_ID" '[.claims[]? | select(.claimer == $w)]' 2>/dev/null || echo '[]')"
  count="$(echo "$my_claims" | jq 'length' 2>/dev/null || echo 0)"

  echo "# claims held by $WORKER_ID on $QUEUE_TOPIC: $count"
  if [ -z "$count" ] || [ "$count" = "0" ]; then
    return 0
  fi
  echo

  # Step 2: for each claim, read the envelope at the offset and render brief
  local i=0
  while [ "$i" -lt "$count" ]; do
    local claim_obj offset claim_id claimed_until ttl_remaining env_json payload
    local task_id classification ac_count dispatched_by now_ms
    claim_obj="$(echo "$my_claims" | jq -c ".[$i]")"
    offset="$(echo "$claim_obj" | jq -r '.offset // empty')"
    claim_id="$(echo "$claim_obj" | jq -r '.claim_id // empty')"
    claimed_until="$(echo "$claim_obj" | jq -r '.claimed_until // empty')"
    # Compute ttl_remaining locally — server returns absolute claimed_until, not delta
    if [ -n "$claimed_until" ]; then
      now_ms="$(date +%s%3N)"
      ttl_remaining=$(( claimed_until - now_ms ))
    else
      ttl_remaining="?"
    fi

    # Read the envelope at this offset. `subscribe` is a follow-stream; bound it
    # with `head -n1` (closes the pipe → SIGPIPE) and a hard `timeout 5` ceiling.
    env_json="$(timeout 5 termlink channel subscribe "$QUEUE_TOPIC" \
                   --cursor "$offset" --limit 1 --json 2>/dev/null \
                   | head -n1 || echo '{}')"
    # Envelope payload is base64-encoded under .payload_b64. Decode then jq.
    local payload_b64
    payload_b64="$(echo "$env_json" | jq -r '.payload_b64 // empty' 2>/dev/null || echo "")"
    if [ -n "$payload_b64" ]; then
      payload="$(echo "$payload_b64" | base64 -d 2>/dev/null || echo '{}')"
    else
      payload="$(echo "$env_json" | jq -r '.payload // "{}"' 2>/dev/null || echo '{}')"
    fi
    task_id="$(echo "$payload" | jq -r '.task_id // "?"' 2>/dev/null || echo "?")"
    classification="$(echo "$payload" | jq -r '.classification // "?"' 2>/dev/null || echo "?")"
    ac_count="$(echo "$payload" | jq -r '.ac_count // "?"' 2>/dev/null || echo "?")"
    dispatched_by="$(echo "$payload" | jq -r '.dispatched_by // "?"' 2>/dev/null || echo "?")"

    echo "CLAIM #$((i+1))/$count"
    echo "  offset=$offset claim_id=$claim_id ttl_remaining=${ttl_remaining}ms"
    echo "  unit: task=$task_id classification=$classification ac_count=$ac_count dispatched_by=$dispatched_by"
    if [ "$classification" = "closure-ready" ]; then
      echo "  brief: run .tasks/active/${task_id}-*.md Verification block; if all pass,"
      echo "         commit + 'fw task update $task_id --status work-completed'."
    else
      echo "  brief: $ac_count unchecked Agent AC(s). Read .tasks/active/${task_id}-*.md,"
      echo "         satisfy each AC, commit, then 'fw task update $task_id --status work-completed'."
    fi

    case "$MODE" in
      dry-run)
        echo "  action: [DRY-RUN] would prompt operator (no hub writes this mode)"
        ;;
      auto-noop)
        echo "  action: [AUTO-NOOP] releasing without doing work (substrate smoke-test only)"
        if ! termlink channel release --claim-id "$claim_id" --claimer "$WORKER_ID" --ack 2>&1 | tail -3; then
          echo "  WARN: release failed for claim_id=$claim_id" >&2
        fi
        ;;
      live)
        echo "  action: [LIVE] prompt for operator decision"
        echo "          ack    → release --ack (work done)"
        echo "          retry  → release no-ack (return for retry)"
        echo "          skip   → leave claim alone (continue to next)"
        echo -n "  decision> "
        local decision=""
        if read -r decision; then
          case "$decision" in
            ack)
              termlink channel release --claim-id "$claim_id" --claimer "$WORKER_ID" --ack 2>&1 | tail -3
              ;;
            retry)
              termlink channel release --claim-id "$claim_id" --claimer "$WORKER_ID" 2>&1 | tail -3
              ;;
            skip|"")
              echo "  (skipped, claim still held)"
              ;;
            *)
              echo "  unknown decision '$decision' — skipping"
              ;;
          esac
        else
          echo "  (stdin closed — skipping)"
        fi
        ;;
    esac
    echo
    i=$((i+1))
  done
}

# ── main loop ───────────────────────────────────────────────────────────────
if [ "$ONCE" = "1" ]; then
  poll_pass
  exit 0
fi

trap 'echo; echo "# SIGINT — exiting"; exit 0' INT TERM

while true; do
  poll_pass
  sleep "$POLL_INTERVAL"
done
