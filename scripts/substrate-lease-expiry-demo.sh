#!/usr/bin/env bash
# scripts/substrate-lease-expiry-demo.sh
#
# T-2214 (arc-parallel-substrate, T-2018 §6): self-contained operator-facing
# proof of the substrate's ANTIFRAGILITY property (constitutional directive #1)
# under the claim primitive (#1) — "what happens when a worker grabs a unit
# then DIES still holding the claim?"
#
# The drain demo (T-2211) proves work-stealing under live contention and the
# cooperative-handoff demo (T-2212) proves directed assignment. Neither proves
# the SELF-HEALING path: a claim's lease auto-expires when its owner stops
# renewing (the proxy for worker death / crash / network partition), the slot
# reopens to a DIFFERENT worker, and the lapsed original owner can no longer
# resurrect the claim. Without this, a single crashed worker would wedge its
# unit forever — the opposite of antifragile. This demo proves the substrate
# heals instead.
#
# The proof is in the assertions — every step hard-checks BOTH a positive path
# AND a negative (refusal) path, so a regression that let a lapsed owner mutate
# a reclaimed slot, or that failed to reopen an expired slot, fails the demo:
#
#   1. POS: worker-A claims an offset with a SHORT lease         (claimer=A)
#   2. NEG: worker-B's claim of the same offset is REFUSED       (exclusive while live)
#   --  wait out the lease (A "dies": stops renewing)  --
#   3. POS: worker-B claims the SAME offset successfully         (auto-reclaim after lapse)
#   4. NEG: the lapsed owner A can no longer RENEW               (lease gone)
#   5. NEG: the lapsed owner A can no longer RELEASE             (lease gone)
#   6. POS: the new owner B releases with --ack                  (work completes cleanly)
#
# Composes ONLY shipped verbs — no new primitive, no hub change:
#   channel create / channel post / channel claim / channel renew / channel release
#
# Exit codes:
#   0  PASS — every positive step succeeded AND every negative step was refused
#   1  FAIL — an assertion failed (slot didn't reopen, or a lapsed owner leaked through)
#   2  usage / missing dependency (termlink, jq)
#
# Usage:
#   substrate-lease-expiry-demo.sh [--hub addr] [--topic NAME]
#                                  [--ttl-ms N] [--json] [--help]

set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
HUB=""
TTL_MS=2000            # short by design — the demo waits this out to force expiry
JSON_MODE=0
TOPIC="substrate-lease-expiry-demo"
A="demo-worker-a"      # the worker that "dies" (stops renewing)
B="demo-worker-b"      # the worker that picks up the reopened slot

die2() { echo "substrate-lease-expiry-demo: $*" >&2; exit 2; }

while [ $# -gt 0 ]; do
  case "$1" in
    --hub)    HUB="$2"; shift 2;;
    --topic)  TOPIC="$2"; shift 2;;
    --ttl-ms) TTL_MS="$2"; shift 2;;
    --json)   JSON_MODE=1; shift;;
    -h|--help) sed -n '2,49p' "$0" | sed 's/^# \{0,1\}//'; exit 0;;
    *) die2 "unknown arg: $1";;
  esac
done

command -v "$TERMLINK" >/dev/null 2>&1 || die2 "termlink not on PATH (set TERMLINK_BIN)"
command -v jq >/dev/null 2>&1 || die2 "jq required"

HUB_ARG=(); [ -n "$HUB" ] && HUB_ARG=(--hub "$HUB")
tl() { "$TERMLINK" "${HUB_ARG[@]}" "$@"; }

# lease-lapsed error vocabulary (empirically observed, T-2214):
#   contested-while-live : "already claimed by another worker"
#   lapsed renew/release : "not found (never existed, released, or expired)"
LAPSED_RE="not found|never existed|released, or expired|expired|CLAIM_NOT_FOUND|-32016"
CONTESTED_RE="already claimed|held by another|CLAIM_ALREADY_HELD|-32015"

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

cleanup() {
  [ -n "${BCID:-}" ] && tl channel release --claim-id "$BCID" --claimer "$B" --ack >/dev/null 2>&1
  return 0
}
trap cleanup EXIT

WAIT=$(( TTL_MS / 1000 + 2 ))   # lease TTL + buffer, in whole seconds

log "=== substrate lease-expiry resilience demo (T-2214) ==="
log "# topic=$TOPIC worker-a=$A worker-b=$B ttl_ms=$TTL_MS wait=${WAIT}s"

# ── Setup: bounded topic + one work unit ──────────────────────────────────────
tl channel create "$TOPIC" --retention messages:50 >/dev/null 2>&1 \
  || tl channel create "$TOPIC" >/dev/null 2>&1 \
  || die2 "channel create failed for $TOPIC"
OFF=$(tl channel post "$TOPIC" --msg-type work --payload "expiry-unit" --json 2>/dev/null \
        | jq -r '.delivered.offset // .offset // empty')
[ -n "$OFF" ] || die2 "could not seed work unit (post returned no offset)"
log "# seeded work unit at offset $OFF"

# ── Step 1 (POS): worker-A claims the offset with a short lease ───────────────
ACID=$(tl channel claim "$TOPIC" "$OFF" --claimer "$A" --ttl-ms "$TTL_MS" --json 2>/dev/null \
        | jq -r '.claim_id // empty')
if [ -n "$ACID" ]; then record "worker-a-claim" 1 "claim_id=$ACID claimer=$A ttl_ms=$TTL_MS"
else record "worker-a-claim" 0 "no claim_id returned"; fi

if [ -n "$ACID" ]; then
  # ── Step 2 (NEG): worker-B's claim is refused while A's lease is live ───────
  out=$(tl channel claim "$TOPIC" "$OFF" --claimer "$B" --ttl-ms 5000 --json 2>&1)
  if [ $? -ne 0 ] && grep -qiE "$CONTESTED_RE" <<<"$out"; then
    record "reject-contested-claim" 1 "exclusivity held — B refused while A's lease live"
  else
    record "reject-contested-claim" 0 "GATE LEAKED: B claimed a live-leased offset: $out"
  fi

  # ── worker-A "dies": stops renewing. Wait out the lease. ────────────────────
  log "# worker-a stops renewing (simulated death); waiting ${WAIT}s for lease to lapse..."
  sleep "$WAIT"

  # ── Step 3 (POS): worker-B claims the SAME offset after expiry ──────────────
  out=$(tl channel claim "$TOPIC" "$OFF" --claimer "$B" --ttl-ms 30000 --json 2>&1)
  BCID=$(jq -r 'select(.ok==true) | .claim_id // empty' <<<"$out" 2>/dev/null)
  if [ -n "$BCID" ]; then
    record "worker-b-reclaim-after-expiry" 1 "slot reopened — B claimed offset $OFF (claim_id=$BCID)"
  else
    record "worker-b-reclaim-after-expiry" 0 "slot did NOT reopen after lease lapse: $out"
    BCID=""
  fi

  # ── Step 4 (NEG): the lapsed owner A can no longer renew ────────────────────
  out=$(tl channel renew --claim-id "$ACID" --claimer "$A" --additional-ttl-ms 30000 --json 2>&1)
  if [ $? -ne 0 ] && grep -qiE "$LAPSED_RE" <<<"$out"; then
    record "reject-lapsed-renew" 1 "lapsed owner $A renew refused (lease gone)"
  else
    record "reject-lapsed-renew" 0 "GATE LEAKED: lapsed owner renew accepted: $out"
  fi

  # ── Step 5 (NEG): the lapsed owner A can no longer release ──────────────────
  out=$(tl channel release --claim-id "$ACID" --claimer "$A" --ack --json 2>&1)
  if [ $? -ne 0 ] && grep -qiE "$LAPSED_RE" <<<"$out"; then
    record "reject-lapsed-release" 1 "lapsed owner $A release refused (lease gone)"
  else
    record "reject-lapsed-release" 0 "GATE LEAKED: lapsed owner release accepted: $out"
  fi

  # ── Step 6 (POS): the new owner B releases cleanly ──────────────────────────
  if [ -n "$BCID" ]; then
    out=$(tl channel release --claim-id "$BCID" --claimer "$B" --ack --json 2>&1)
    if jq -e 'select(.ok==true and .ack==true)' >/dev/null 2>&1 <<<"$out"; then
      record "worker-b-release-ack" 1 "reopened work completed by B (ack=true)"
      BCID=""   # released cleanly — disarm cleanup
    else
      record "worker-b-release-ack" 0 "new-owner release failed: $out"
    fi
  else
    record "worker-b-release-ack" 0 "skipped — B never acquired the reopened slot"
  fi
fi

TOTAL=$((PASS+FAIL))
if [ "$JSON_MODE" -eq 1 ]; then
  jq -nc --argjson pass "$PASS" --argjson fail "$FAIL" --argjson total "$TOTAL" \
     --argjson steps "$STEPS_JSON" --arg topic "$TOPIC" \
     '{ok:($fail==0), verdict:(if $fail==0 then "PASS" else "FAIL" end),
       topic:$topic, passed:$pass, failed:$fail, total:$total, steps:$steps}'
else
  log ""
  if [ "$FAIL" -eq 0 ]; then
    log "LEASE-EXPIRY RESILIENCE DEMO PASS — $PASS/$TOTAL assertions green"
    log "  (3 positive + 3 negative: slot auto-reopened after lease lapse, lapsed owner locked out)"
  else
    log "LEASE-EXPIRY RESILIENCE DEMO FAIL — $FAIL/$TOTAL assertion(s) failed"
  fi
fi
[ "$FAIL" -eq 0 ] && exit 0 || exit 1
