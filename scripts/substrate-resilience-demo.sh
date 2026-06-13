#!/usr/bin/env bash
# scripts/substrate-resilience-demo.sh
#
# T-2223 (arc-parallel-substrate, T-2018 §6): self-contained operator-facing
# proof of the substrate's RELIABILITY + ANTIFRAGILITY properties
# (constitutional directives #2 + #1) under the offline-queue primitive (#5,
# T-2051) and exactly-once post-idempotency (#5, T-2049) — "what happens to a
# post when the hub is DOWN at the moment I send it?"
#
# The drain demo (T-2211) proves work-stealing, the cooperative-handoff demo
# (T-2212) proves directed assignment, and the lease-expiry demo (T-2214)
# proves worker-death auto-reclaim. None exercises the RESILIENCE family: a
# post issued while the hub is unreachable must NOT be silently dropped — it is
# durably enqueued, auto-drains on the next post once the hub returns, and a
# replay of the same client_msg_id is absorbed exactly-once (no double-append).
# Without this, a single hub blip would lose in-flight posts — the opposite of
# reliable. This demo proves the substrate buffers and heals instead.
#
# The proof is in the assertions — every step hard-checks the PROPERTY claimed
# (PL-213), covering both the durability path AND the exactly-once path:
#
#   1. POS: hub up — post P1 is delivered                        (baseline)
#   --  hub STOPS (simulated blip) --
#   2. POS: post P2 (with client_msg_id) is QUEUED, not dropped  (never-silent-drop)
#   3. POS: the outbound queue depth is exactly 1                (durable buffer)
#   --  hub RESTARTS --
#   4. POS: next post P3 auto-drains the queue ("Drained 1")     (heal-on-reconnect)
#   5. POS: topic now holds exactly 3 work posts (P1,P2,P3)      (queued post landed once)
#   6. POS: replaying P2's client_msg_id is absorbed            (exactly-once / dedup)
#   7. POS: topic STILL holds exactly 3 work posts              (no double-append)
#   8. POS: outbound queue is fully drained (depth 0)            (clean steady state)
#
# Fully ISOLATED — runs a throwaway UDS-only hub on a temp runtime_dir with a
# temp identity/queue dir, so it never touches the operator's live hub or the
# real ~/.termlink/outbound.sqlite. Composes ONLY shipped verbs — no new
# primitive, no hub change:
#   hub start / hub stop / channel create / channel post (--client-msg-id)
#   channel queue-status / channel info
#
# Exit codes:
#   0  PASS — every property assertion held
#   1  FAIL — an assertion failed (post dropped, queue not drained, or a
#             replay double-appended)
#   2  usage / missing dependency (termlink, jq)
#
# Usage:
#   substrate-resilience-demo.sh [--topic NAME] [--json] [--keep] [--help]

set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
JSON_MODE=0
KEEP=0
TOPIC="substrate-resilience-demo"

die2() { echo "substrate-resilience-demo: $*" >&2; exit 2; }

while [ $# -gt 0 ]; do
  case "$1" in
    --topic) TOPIC="$2"; shift 2;;
    --json)  JSON_MODE=1; shift;;
    --keep)  KEEP=1; shift;;
    -h|--help) sed -n '2,49p' "$0" | sed 's/^# \{0,1\}//'; exit 0;;
    *) die2 "unknown arg: $1";;
  esac
done

command -v "$TERMLINK" >/dev/null 2>&1 || die2 "termlink not on PATH (set TERMLINK_BIN)"
command -v jq >/dev/null 2>&1 || die2 "jq required"

# ── Isolation: throwaway identity dir (→ outbound.sqlite) + runtime dir (→ hub.sock). ──
DEMO_DIR="$(mktemp -d "${TMPDIR:-/tmp}/tl-resilience-demo.XXXXXX")" || die2 "mktemp failed"
RT="$DEMO_DIR/rt"
mkdir -p "$RT"
export TERMLINK_IDENTITY_DIR="$DEMO_DIR"
export TERMLINK_RUNTIME_DIR="$RT"
SOCK="$RT/hub.sock"

PASS=0; FAIL=0
STEPS_JSON="[]"
log() { [ "$JSON_MODE" -eq 1 ] || echo "$@"; }

record() {
  local name="$1" ok="$2" detail="$3"
  if [ "$ok" -eq 1 ]; then PASS=$((PASS+1)); log "  [PASS] $name — $detail"
  else FAIL=$((FAIL+1)); log "  [FAIL] $name — $detail"; fi
  STEPS_JSON=$(jq -c --arg n "$name" --argjson ok "$ok" --arg d "$detail" \
    '. + [{step:$n, pass:($ok==1), detail:$d}]' <<<"$STEPS_JSON")
}

hub_stop() { "$TERMLINK" hub stop >/dev/null 2>&1 || true; }

cleanup() {
  hub_stop
  if [ "$KEEP" -eq 1 ]; then
    log "# --keep: demo dir retained at $DEMO_DIR"
  else
    rm -rf "$DEMO_DIR" 2>/dev/null || true
  fi
  return 0
}
trap cleanup EXIT

# Wait for the UDS hub socket to appear (or vanish). $1=appear|vanish, $2=secs
wait_sock() {
  local want="$1" secs="${2:-8}" i=0
  while [ "$i" -lt $(( secs * 10 )) ]; do
    if [ "$want" = "appear" ]; then [ -S "$SOCK" ] && return 0
    else [ -S "$SOCK" ] || return 0; fi
    sleep 0.1; i=$((i+1))
  done
  return 1
}

hub_start() {
  # UDS-only (no --tcp) → zero port conflict with any live hub on this host.
  "$TERMLINK" hub start >"$DEMO_DIR/hub.log" 2>&1 &
  wait_sock appear 8 || { echo "hub failed to start; log:" >&2; cat "$DEMO_DIR/hub.log" >&2; return 1; }
}

topic_count() {
  "$TERMLINK" channel info "$TOPIC" --json 2>/dev/null | jq -r '.count // 0'
}
queue_depth() {
  "$TERMLINK" channel queue-status --json 2>/dev/null | jq -r '.pending // 0'
}

log "=== substrate RESILIENCE demo (T-2223) ==="
log "# isolated: identity_dir=$DEMO_DIR runtime_dir=$RT topic=$TOPIC"

# ── Setup: start isolated hub + create topic ──────────────────────────────────
hub_start || die2 "could not start isolated hub"
"$TERMLINK" channel create "$TOPIC" --retention messages:50 >/dev/null 2>&1 \
  || "$TERMLINK" channel create "$TOPIC" >/dev/null 2>&1 \
  || die2 "channel create failed for $TOPIC"

# ── Step 1 (POS): baseline delivery while hub is up ───────────────────────────
out=$("$TERMLINK" channel post "$TOPIC" --msg-type work --payload "p1-baseline" --json 2>&1)
OFF1=$(jq -r '.delivered.offset // empty' <<<"$out" 2>/dev/null)
if [ -n "$OFF1" ]; then record "baseline-delivered" 1 "P1 delivered at offset $OFF1 (hub up)"
else record "baseline-delivered" 0 "P1 not delivered while hub up: $out"; fi

# ── Hub blip: STOP the hub ────────────────────────────────────────────────────
hub_stop
if wait_sock vanish 8; then log "# hub stopped (socket gone) — simulating blip"
else log "# WARN: hub socket lingered after stop"; fi

# ── Step 2 (POS): post during blip is QUEUED, not dropped ─────────────────────
CID="resilience-demo-cid-0001"
out=$("$TERMLINK" channel post "$TOPIC" --msg-type work --payload "p2-during-blip" \
        --client-msg-id "$CID" --json 2>&1)
if jq -e '.queued.queue_id' >/dev/null 2>&1 <<<"$out"; then
  QID=$(jq -r '.queued.queue_id' <<<"$out")
  record "queued-not-dropped" 1 "P2 durably queued (queue_id=$QID) — not silent-dropped"
else
  record "queued-not-dropped" 0 "P2 was NOT queued during blip (silent drop?): $out"
fi

# ── Step 3 (POS): queue depth is exactly 1 ────────────────────────────────────
DEPTH=$(queue_depth)
if [ "$DEPTH" = "1" ]; then record "queue-depth-1" 1 "outbound queue holds exactly 1 pending post"
else record "queue-depth-1" 0 "expected queue depth 1, got '$DEPTH'"; fi

# ── Hub returns: RESTART ──────────────────────────────────────────────────────
hub_start || die2 "could not restart isolated hub"

# ── Step 4 (POS): next post auto-drains the queue ─────────────────────────────
out=$("$TERMLINK" channel post "$TOPIC" --msg-type work --payload "p3-after-recovery" --json 2>&1)
OFF3=$(jq -r '.delivered.offset // empty' <<<"$out" 2>/dev/null)
if grep -qiE "Drained 1 queued post" <<<"$out" && [ -n "$OFF3" ]; then
  record "auto-drain-on-reconnect" 1 "queue auto-drained (P2 flushed) AND P3 delivered at $OFF3"
elif [ -n "$OFF3" ]; then
  record "auto-drain-on-reconnect" 1 "P3 delivered at $OFF3 (drain verified via depth/count below)"
else
  record "auto-drain-on-reconnect" 0 "post after recovery did not deliver: $out"
fi

# ── Step 5 (POS): topic holds exactly 3 work posts (P1, drained-P2, P3) ───────
CNT_AFTER_DRAIN=$(topic_count)
if [ "$CNT_AFTER_DRAIN" = "3" ]; then
  record "queued-post-landed-once" 1 "topic count=3 — P2 flushed and landed exactly once"
else
  record "queued-post-landed-once" 0 "expected topic count 3 after drain, got '$CNT_AFTER_DRAIN'"
fi

# ── Step 6+7 (POS): replay P2's client_msg_id is absorbed exactly-once ────────
out=$("$TERMLINK" channel post "$TOPIC" --msg-type work --payload "p2-replay" \
        --client-msg-id "$CID" --json 2>&1)
REPLAY_OK=$?
CNT_AFTER_REPLAY=$(topic_count)
if [ "$REPLAY_OK" -eq 0 ] && [ "$CNT_AFTER_REPLAY" = "$CNT_AFTER_DRAIN" ]; then
  record "replay-absorbed-exactly-once" 1 "replay of CID=$CID accepted; topic count unchanged ($CNT_AFTER_REPLAY) — deduped"
else
  record "replay-absorbed-exactly-once" 0 "replay double-appended or failed: rc=$REPLAY_OK count $CNT_AFTER_DRAIN->$CNT_AFTER_REPLAY: $out"
fi

# ── Step 8 (POS): outbound queue fully drained ───────────────────────────────
DEPTH_END=$(queue_depth)
if [ "$DEPTH_END" = "0" ]; then record "queue-drained-clean" 1 "outbound queue depth 0 — clean steady state"
else record "queue-drained-clean" 0 "expected queue depth 0 at end, got '$DEPTH_END'"; fi

TOTAL=$((PASS+FAIL))
if [ "$JSON_MODE" -eq 1 ]; then
  jq -nc --argjson pass "$PASS" --argjson fail "$FAIL" --argjson total "$TOTAL" \
     --argjson steps "$STEPS_JSON" --arg topic "$TOPIC" \
     '{ok:($fail==0), verdict:(if $fail==0 then "PASS" else "FAIL" end),
       topic:$topic, passed:$pass, failed:$fail, total:$total, steps:$steps}'
else
  log ""
  if [ "$FAIL" -eq 0 ]; then
    log "RESILIENCE DEMO PASS — $PASS/$TOTAL property assertions green"
    log "  (post during blip queued not dropped, auto-drained on reconnect, replay deduped exactly-once)"
  else
    log "RESILIENCE DEMO FAIL — $FAIL/$TOTAL assertion(s) failed"
  fi
fi
[ "$FAIL" -eq 0 ] && exit 0 || exit 1
