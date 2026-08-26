#!/usr/bin/env bash
# guard-layer: source
# (no --no-heartbeat: these checks have no heartbeat companion to skip. The
#  marker IS the invocation the runner uses, so declaring a flag we do not
#  accept made check-receiver-ack-lag ERROR under the layer — a contract
#  asserting more than the mechanism, in the machine-readable direction.)
# check-receiver-ack-lag.sh — subscribers that never acknowledge.
#
# T-2838 item 5, receiver half. The spike's argument for hub-side enforcement was
# one sentence: "a client that simply declines to ack is indistinguishable from
# one that crashed." Both look like silence.
#
# WHAT ALREADY EXISTED, and why this is the missing piece rather than a duplicate:
#
#   check-unconfirmed-delivery-freshness.sh   SENDER side. Reads this host's own
#                                             ~/.termlink/awaiting_ack.sqlite
#                                             obligation rows (T-2287/T-2295) and
#                                             fires on rows outstanding too long.
#                                             It knows what WE sent and nobody
#                                             confirmed.
#
#   this script                               RECEIVER side. Reads the hub's own
#                                             receipt aggregate via
#                                             `channel ack-status` and fires on
#                                             senders whose read frontier is
#                                             falling behind the topic.
#
# 47 check-*.sh scripts existed before this one and NONE called ack-status. The
# sender side has had a canary since T-2295; the receiver side has had none, so
# "nobody is reading this topic" was observable only by a human running the verb.
#
# IT IS THE FIRST CONSUMER OF THE T-2838 ITEM 2 FIX. Before 6f42f2b0d,
# ack-status counted receipt envelopes in its own frontier, so acking raised the
# offset it was chasing and `lag: 0` was unreachable. A canary built on the old
# behaviour would have fired forever. It now uses latest_content_offset, so lag
# reaching 0 is a real state.
#
# THE IDENTITY CAVEAT, STATED IN THE OUTPUT AND NOT ONLY HERE. ack-status rows
# are keyed by identity FINGERPRINT. Where many agents share one keypair, their
# rows collapse into one and this check measures a host, not an agent. That is
# T-2838 item 1 and it is not fixed by this script — the output says how many
# distinct identities it actually saw so a reader can judge the resolution of
# the answer rather than assume it.
#
# Exit: 0 all senders within threshold | 1 at least one is behind
#       2 could not run (no verdict) — never rendered as healthy
set -uo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null || echo /opt/termlink)" || exit 1

TL="${TERMLINK_BIN:-./target/release/termlink}"
command -v "$TL" >/dev/null 2>&1 || TL=termlink
THRESHOLD="${LAG_THRESHOLD:-25}"
TOPICS_DEFAULT="agent-chat-arc framework:pickup"

usage() {
  cat <<'EOF'
check-receiver-ack-lag.sh [--topics "a b"] [--threshold N] [--self-test]

Fires when a sender on a topic has an ack frontier further than N content
envelopes behind the topic head, or has never acked at all.
EOF
}

TOPICS="$TOPICS_DEFAULT"
while [ $# -gt 0 ]; do
  case "$1" in
    --topics)    TOPICS="${2:-}"; shift 2 ;;
    --threshold) THRESHOLD="${2:-}"; shift 2 ;;
    --self-test) SELFTEST=1; shift ;;
    -h|--help)   usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

if [ "${SELFTEST:-0}" = "1" ]; then
  # The classifier must separate three states that all "look like silence":
  #   caught up | behind | never acked
  # A canary that cannot tell "never acked" from "caught up" is the false-green
  # shape this whole arc is about, so it is planted explicitly.
  classify() {  # $1=lag $2=up_to $3=threshold
    if [ "$2" = "null" ]; then echo "NEVER-ACKED"
    elif [ "$1" -gt "$3" ] 2>/dev/null; then echo "BEHIND"
    else echo "OK"; fi
  }
  fail=0
  [ "$(classify 0 3 25)" = "OK" ]           || { echo "self-test: FAIL caught-up misread"; fail=1; }
  [ "$(classify 99 3 25)" = "BEHIND" ]      || { echo "self-test: FAIL behind misread"; fail=1; }
  [ "$(classify 4 null 25)" = "NEVER-ACKED" ] || { echo "self-test: FAIL never-acked misread as OK"; fail=1; }
  # never-acked must NOT be rescued by a small lag — that is the exact collapse
  [ "$(classify 0 null 25)" = "NEVER-ACKED" ] || { echo "self-test: FAIL never-acked with lag 0 read as OK"; fail=1; }
  [ "$fail" = "0" ] && { echo "self-test: PASS — caught-up, behind and never-acked are three distinct verdicts"; exit 0; }
  exit 2
fi

command -v jq >/dev/null 2>&1 || { echo "check-receiver-ack-lag: jq not in PATH — no verdict"; exit 2; }

echo "check-receiver-ack-lag: receiver-side ack frontiers (threshold ${THRESHOLD})"
echo "  PREDICATE: reads the hub's own receipt aggregate via 'channel ack-status'."
echo "             Rows are keyed by identity FINGERPRINT — where agents share a"
echo "             keypair their rows collapse and this measures a HOST, not an"
echo "             agent (T-2838 item 1). Distinct-identity counts are printed"
echo "             per topic so the resolution of the answer is visible."
echo ""

rc=0
saw_any=0
for topic in $TOPICS; do
  out="$("$TL" channel ack-status "$topic" --json 2>/dev/null || true)"
  if [ -z "$out" ] || ! printf '%s' "$out" | jq -e 'type=="array"' >/dev/null 2>&1; then
    echo "  $topic: could not read ack-status — NO VERDICT for this topic"
    rc=2
    continue
  fi
  saw_any=1
  n_ident=$(printf '%s' "$out" | jq -r 'length')
  echo "  $topic  ($n_ident distinct identity row(s))"
  while IFS=$'\t' read -r sid lag upto; do
    [ -n "$sid" ] || continue
    short="${sid:0:16}"
    if [ "$upto" = "null" ]; then
      printf '    NEVER-ACKED  %s  lag=%s\n' "$short" "$lag"
      rc=1
    elif [ "$lag" -gt "$THRESHOLD" ] 2>/dev/null; then
      printf '    BEHIND       %s  lag=%s (up_to=%s)\n' "$short" "$lag" "$upto"
      rc=1
    else
      printf '    ok           %s  lag=%s\n' "$short" "$lag"
    fi
  done < <(printf '%s' "$out" | jq -r '.[] | [.sender_id, (.lag|tostring), (.up_to|tostring)] | @tsv')
  echo ""
done

if [ "$saw_any" = "0" ]; then
  echo "  no topic produced a reading — NO VERDICT. This is not 'all healthy'."
  exit 2
fi
[ "$rc" = "0" ] && echo "  all senders within threshold."
exit "$rc"
