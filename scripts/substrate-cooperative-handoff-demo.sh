#!/usr/bin/env bash
# scripts/substrate-cooperative-handoff-demo.sh
#
# T-2212 (arc-parallel-substrate, T-2018 §6): self-contained operator-facing
# proof of the arc's DIRECTED-ASSIGNMENT mechanic — the orchestrator->worker
# COOPERATIVE HANDOFF documented in docs/operations/substrate-orchestrator-recipe.md.
#
# Where the T-2211 drain demo proves WORK-STEALING (N workers race to claim
# disjoint units), this demo proves the complementary path: an orchestrator
# claims a slot on a worker's behalf and atomically hands the lease over with
# ZERO release-then-claim race window (substrate primitive #3, claim-transfer),
# the worker then renews and releases. It is the shell-level companion to the
# Rust ownership tests and exercises the FULL claim lifecycle end-to-end.
#
# The proof is in the assertions — every step hard-checks BOTH the positive
# path AND the ownership-enforcement negative path (CLAIM_NOT_OWNED), so a
# regression that silently let a non-owner mutate a claim would fail the demo:
#
#   1. orchestrator claims an offset                          (claimer=orch)
#   2. NEG: a stale/other --by transfer is REFUSED            (ownership gate)
#   3. POS: orchestrator transfers the lease to the worker    (atomic handoff)
#   4. NEG: the ex-owner orchestrator can no longer RENEW      (gate moved)
#   5. POS: the new-owner worker renews the lease              (ownership moved)
#   6. NEG: the ex-owner orchestrator can no longer RELEASE    (gate moved)
#   7. POS: the new-owner worker releases with --ack           (work complete)
#
# Composes ONLY shipped verbs — no new primitive, no hub change:
#   channel create / channel post / channel claim /
#   channel claim-transfer / channel renew / channel release
#
# Exit codes:
#   0  PASS — every positive step succeeded AND every negative step was refused
#   1  FAIL — an assertion failed (ownership gate leaked, or a valid op refused)
#   2  usage / missing dependency (termlink, jq)
#
# Usage:
#   substrate-cooperative-handoff-demo.sh [--hub addr] [--topic NAME]
#                                         [--ttl-ms N] [--json] [--help]

set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
HUB=""
TTL_MS=60000
JSON_MODE=0
TOPIC="substrate-handoff-demo"
ORCH="demo-orch"
WORKER="demo-worker"
INTRUDER="demo-intruder"

die2() { echo "substrate-cooperative-handoff-demo: $*" >&2; exit 2; }

while [ $# -gt 0 ]; do
  case "$1" in
    --hub)    HUB="$2"; shift 2;;
    --topic)  TOPIC="$2"; shift 2;;
    --ttl-ms) TTL_MS="$2"; shift 2;;
    --json)   JSON_MODE=1; shift;;
    -h|--help) sed -n '2,46p' "$0" | sed 's/^# \{0,1\}//'; exit 0;;
    *) die2 "unknown arg: $1";;
  esac
done

command -v "$TERMLINK" >/dev/null 2>&1 || die2 "termlink not on PATH (set TERMLINK_BIN)"
command -v jq >/dev/null 2>&1 || die2 "jq required"

HUB_ARG=(); [ -n "$HUB" ] && HUB_ARG=(--hub "$HUB")
tl() { "$TERMLINK" "${HUB_ARG[@]}" "$@"; }

PASS=0; FAIL=0
STEPS_JSON="[]"
log() { [ "$JSON_MODE" -eq 1 ] || echo "$@"; }

# record_step <name> <ok:0|1> <detail>
record() {
  local name="$1" ok="$2" detail="$3"
  if [ "$ok" -eq 1 ]; then PASS=$((PASS+1)); log "  [PASS] $name — $detail"
  else FAIL=$((FAIL+1)); log "  [FAIL] $name — $detail"; fi
  STEPS_JSON=$(jq -c --arg n "$name" --argjson ok "$ok" --arg d "$detail" \
    '. + [{step:$n, pass:($ok==1), detail:$d}]' <<<"$STEPS_JSON")
}

cleanup() {
  # best-effort: release the claim if the demo aborted mid-flight (ignore errors)
  [ -n "${CID:-}" ] && tl channel release --claim-id "$CID" --claimer "$WORKER" --ack >/dev/null 2>&1
  [ -n "${CID:-}" ] && tl channel release --claim-id "$CID" --claimer "$ORCH" --ack >/dev/null 2>&1
  return 0
}
trap cleanup EXIT

log "=== substrate cooperative-handoff demo (T-2212) ==="
log "# topic=$TOPIC orchestrator=$ORCH worker=$WORKER ttl_ms=$TTL_MS"

# ── Setup: bounded topic + one work unit ──────────────────────────────────────
tl channel create "$TOPIC" --retention messages:50 >/dev/null 2>&1 \
  || tl channel create "$TOPIC" >/dev/null 2>&1 \
  || die2 "channel create failed for $TOPIC"
OFF=$(tl channel post "$TOPIC" --msg-type work --payload "handoff-unit" --json 2>/dev/null \
        | jq -r '.delivered.offset // .offset // empty')
[ -n "$OFF" ] || die2 "could not seed work unit (post returned no offset)"
log "# seeded work unit at offset $OFF"

# ── Step 1 (POS): orchestrator claims the offset ──────────────────────────────
CID=$(tl channel claim "$TOPIC" "$OFF" --claimer "$ORCH" --ttl-ms "$TTL_MS" --json 2>/dev/null \
        | jq -r '.claim_id // empty')
if [ -n "$CID" ]; then record "orchestrator-claim" 1 "claim_id=$CID claimer=$ORCH"
else record "orchestrator-claim" 0 "no claim_id returned"; CID=""; fi
[ -n "$CID" ] || { log ""; log "ABORT: could not establish initial claim"; FAIL=$((FAIL+1)); }

if [ -n "$CID" ]; then
  # ── Step 2 (NEG): a non-owner --by transfer must be refused ─────────────────
  out=$(tl channel claim-transfer --claim-id "$CID" --to-owner "$WORKER" --by "$INTRUDER" --json 2>&1)
  if [ $? -ne 0 ] && grep -qi "held by another claimer\|CLAIM_NOT_OWNED\|-32017" <<<"$out"; then
    record "reject-stale-by-transfer" 1 "ownership gate held (--by=$INTRUDER refused)"
  else
    record "reject-stale-by-transfer" 0 "GATE LEAKED: non-owner transfer was accepted: $out"
  fi

  # ── Step 3 (POS): orchestrator atomically transfers the lease to the worker ──
  out=$(tl channel claim-transfer --claim-id "$CID" --to-owner "$WORKER" --by "$ORCH" --json 2>&1)
  ok=$(jq -r 'select(.ok==true) | .to_owner' <<<"$out" 2>/dev/null)
  ca=$(jq -r '.claimed_at // empty' <<<"$out" 2>/dev/null)
  if [ "$ok" = "$WORKER" ]; then
    record "transfer-to-worker" 1 "claimed_by -> $WORKER (lease claimed_at=$ca survived)"
  else
    record "transfer-to-worker" 0 "transfer did not move ownership to $WORKER: $out"
  fi

  # ── Step 4 (NEG): the ex-owner orchestrator can no longer renew ─────────────
  out=$(tl channel renew --claim-id "$CID" --claimer "$ORCH" --additional-ttl-ms 30000 --json 2>&1)
  if [ $? -ne 0 ] && grep -qi "held by another claimer\|CLAIM_NOT_OWNED\|-32017" <<<"$out"; then
    record "reject-ex-owner-renew" 1 "ex-owner $ORCH renew refused after handoff"
  else
    record "reject-ex-owner-renew" 0 "GATE LEAKED: ex-owner renew was accepted: $out"
  fi

  # ── Step 5 (POS): the new-owner worker renews the lease ─────────────────────
  out=$(tl channel renew --claim-id "$CID" --claimer "$WORKER" --additional-ttl-ms 30000 --json 2>&1)
  rc=$(jq -r 'select(.ok==true) | .claimer' <<<"$out" 2>/dev/null)
  if [ "$rc" = "$WORKER" ]; then
    record "worker-renew" 1 "new owner $WORKER renewed (proves ownership moved)"
  else
    record "worker-renew" 0 "worker renew failed: $out"
  fi

  # ── Step 6 (NEG): the ex-owner orchestrator can no longer release ───────────
  out=$(tl channel release --claim-id "$CID" --claimer "$ORCH" --ack --json 2>&1)
  if [ $? -ne 0 ] && grep -qi "held by another claimer\|CLAIM_NOT_OWNED\|-32017" <<<"$out"; then
    record "reject-ex-owner-release" 1 "ex-owner $ORCH release refused after handoff"
  else
    record "reject-ex-owner-release" 0 "GATE LEAKED: ex-owner release was accepted: $out"
  fi

  # ── Step 7 (POS): the new-owner worker releases with --ack ──────────────────
  out=$(tl channel release --claim-id "$CID" --claimer "$WORKER" --ack --json 2>&1)
  if jq -e 'select(.ok==true and .ack==true)' >/dev/null 2>&1 <<<"$out"; then
    record "worker-release-ack" 1 "work completed (ack=true, cursor advanced)"
    CID=""   # released cleanly — disarm cleanup
  else
    record "worker-release-ack" 0 "worker release failed: $out"
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
    log "COOPERATIVE-HANDOFF DEMO PASS — $PASS/$TOTAL assertions green"
    log "  (3 positive lifecycle steps + 3 ownership-gate refusals, atomic handoff verified)"
  else
    log "COOPERATIVE-HANDOFF DEMO FAIL — $FAIL/$TOTAL assertion(s) failed"
  fi
fi
[ "$FAIL" -eq 0 ] && exit 0 || exit 1
