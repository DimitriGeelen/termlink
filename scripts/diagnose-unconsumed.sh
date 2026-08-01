#!/usr/bin/env bash
# diagnose-unconsumed.sh (T-2479, G-083 loud consumption-contract)
#
# WHY: agent-send.sh already rings + waits for a msg_type=receipt and returns
# exit 3 when no receipt arrives. But that not-acked path is SILENT about WHY —
# a peer that is LIVE + armed (metadata.pty_session set) but busy or in
# manual-accept mode never posts a receipt, and the sender gets a bare "not
# acked" with no classification and no remediation. G-083: LIVE != listening.
# This verb turns "rung but unconsumed" into a classified, actionable failure.
#
# It reads the load-bearing signals (the SAME source the T-2387 waker-liveness
# canary uses):
#   - peer presence + status (LIVE?) + metadata.pty_session (armed?)  <- agent-listeners.sh --json
#   - a receipt on the dm-topic (was it consumed after all?)          <- wake-confirm.sh
# and classifies into one of four states, each with a loud remediation line.
#
# CLASSES / EXIT CODES (contract-stable):
#   0  consumed        -- a receipt exists; the wake WAS consumed (healthy)
#   1  busy-or-manual  -- peer LIVE + armed but no receipt: the alarming G-083
#                         case. Message is durably written; the session is not
#                         consuming (busy on its own turn, or manual-accept mode
#                         left the injected text unsubmitted).
#   2  unwakeable      -- peer LIVE but NOT armed (no pty_session): peers can DM
#                         it durably but nothing rings its PTY (T-2380 breakpoint #2).
#   3  dead            -- peer presence stale/absent: the process is gone.
#   4  tooling error   -- could not read a required signal.
#
# USAGE:
#   diagnose-unconsumed.sh --peer <agent_id> [--topic <dm-topic>] [--cid <cid>]
#                          [--hub <addr>] [--live-receipt-check] [--json]
#
#   --peer                the recipient agent_id whose consumption we diagnose (required)
#   --topic               the dm-topic the wake was posted to (for the receipt check)
#   --cid                 conversation_id to scope the receipt match (offset-aware)
#   --hub                 hub address to probe (default: local hub)
#   --live-receipt-check  actually poll wake-confirm.sh for a late receipt before
#                         concluding busy-or-manual (needs --topic + --cid)
#   --json                emit {class, exit, peer, remediation} instead of text
#
# TEST HOOKS (PL-213 -- host-independent unit tests, see test-diagnose-unconsumed.sh):
#   TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON=<file>  canned agent-listeners.sh --json envelope
#   TERMLINK_DIAGNOSE_TEST_RECEIPT_JSON=<file>   canned wake-confirm.sh JSON ({consumed:bool})
#
# Origin: T-2476 (P2 inception, GO'd) -> this build. Sibling of the T-2387
# waker-liveness canary (that finds DEAD/unarmed wakers on a schedule; this
# diagnoses ONE unconsumed send synchronously, at the moment it fails).
set -euo pipefail

SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

peer="" topic="" cid="" hub="" live_receipt_check=0 want_json=0
# Presence "LIVE" already encodes freshness in agent-listeners.sh; STALE_SECS is a
# secondary guard so a LIVE row with an implausibly old age still reads as dead.
STALE_SECS="${TERMLINK_DIAGNOSE_STALE_SECS:-180}"

while [ $# -gt 0 ]; do
    case "$1" in
        --peer)  peer="${2:-}"; shift 2 ;;
        --topic) topic="${2:-}"; shift 2 ;;
        --cid)   cid="${2:-}"; shift 2 ;;
        --hub)   hub="${2:-}"; shift 2 ;;
        --live-receipt-check) live_receipt_check=1; shift ;;
        --json)  want_json=1; shift ;;
        -h|--help)
            sed -n '2,60p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
            exit 0 ;;
        *) echo "diagnose-unconsumed: unknown arg '$1'" >&2; exit 4 ;;
    esac
done

if [ -z "$peer" ]; then
    echo "diagnose-unconsumed: --peer <agent_id> is required" >&2
    exit 4
fi

# --- read presence (test hook wins; else live) ------------------------------
if [ -n "${TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON:-}" ]; then
    presence_json="$(cat "$TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON" 2>/dev/null || true)"
else
    hub_args=(); [ -n "$hub" ] && hub_args=(--hub "$hub")
    presence_json="$(bash "$SELF_DIR/agent-listeners.sh" --json "${hub_args[@]}" 2>/dev/null || true)"
fi
if [ -z "$presence_json" ]; then
    echo "diagnose-unconsumed: could not read presence (agent-listeners.sh --json returned nothing)" >&2
    exit 4
fi

# --- classify via python (robust JSON) --------------------------------------
# Emits: CLASS<TAB>ARMED<TAB>STATUS<TAB>AGE   (ARMED/STATUS/AGE informational)
read -r class armed status age < <(
    PEER="$peer" STALE="$STALE_SECS" python3 - "$presence_json" <<'PY'
import sys, json, os
peer = os.environ["PEER"]; stale = int(os.environ["STALE"])
try:
    d = json.loads(sys.argv[1])
except Exception:
    print("ERROR\t-\t-\t-"); sys.exit(0)
ls = d.get("listeners", d) if isinstance(d, dict) else d
if not isinstance(ls, list):
    print("ERROR\t-\t-\t-"); sys.exit(0)
row = next((l for l in ls if isinstance(l, dict) and l.get("agent_id") == peer), None)
if row is None:
    print("dead\t-\tabsent\t-"); sys.exit(0)
status = row.get("status", "?")
age = row.get("age_secs", None)
pty = row.get("pty_session")
armed = "yes" if (pty not in (None, "", False)) else "no"
if status != "LIVE":
    print(f"dead\t{armed}\t{status}\t{age}"); sys.exit(0)
if isinstance(age, (int, float)) and age > stale:
    print(f"dead\t{armed}\t{status}\t{age}"); sys.exit(0)
if armed == "no":
    print(f"unwakeable\t{armed}\t{status}\t{age}"); sys.exit(0)
# LIVE + armed -> receipt check decides consumed vs busy-or-manual
print(f"live-armed\t{armed}\t{status}\t{age}")
PY
)

if [ "$class" = "ERROR" ]; then
    echo "diagnose-unconsumed: presence JSON was unparseable" >&2
    exit 4
fi

# --- for LIVE+armed, a receipt (if any) distinguishes consumed vs busy/manual
if [ "$class" = "live-armed" ]; then
    receipt_json=""
    if [ -n "${TERMLINK_DIAGNOSE_TEST_RECEIPT_JSON:-}" ]; then
        receipt_json="$(cat "$TERMLINK_DIAGNOSE_TEST_RECEIPT_JSON" 2>/dev/null || true)"
    elif [ "$live_receipt_check" -eq 1 ] && [ -n "$topic" ] && [ -n "$cid" ]; then
        hub_args=(); [ -n "$hub" ] && hub_args=(--hub "$hub")
        receipt_json="$(bash "$SELF_DIR/wake-confirm.sh" --topic "$topic" --cid "$cid" \
                        --timeout 5 "${hub_args[@]}" 2>/dev/null || true)"
    fi
    if [ -n "$receipt_json" ] && printf '%s' "$receipt_json" | jq -e '.consumed==true' >/dev/null 2>&1; then
        class="consumed"
    else
        class="busy-or-manual"
    fi
fi

# --- render + exit ----------------------------------------------------------
case "$class" in
    consumed)
        code=0
        headline="CONSUMED — peer '$peer' acked (receipt present); the wake was received."
        remediation="No action needed." ;;
    busy-or-manual)
        code=1
        headline="BUSY-OR-MANUAL — peer '$peer' is LIVE + armed but posted NO receipt: LIVE != listening (G-083)."
        remediation="The message IS durably written on the topic; the session is not consuming it. Likely busy on its own turn, or in manual-accept mode (injected text sits UNSUBMITTED). Remediation: have the peer consume manually (/check-arc respond), OR verify auto-accept is armed (IS_SANDBOX=1 --dangerously-skip-permissions), OR relaunch it through the T-2388 armed launcher: bash scripts/tl-claude.sh start --reachable --agent-id $peer -- --resume" ;;
    unwakeable)
        code=2
        headline="UNWAKEABLE — peer '$peer' is LIVE but NOT armed (no pty_session): nothing can ring its PTY (T-2380 breakpoint #2)."
        remediation="Peers can DM it durably but no wake reaches it. Relaunch armed: bash scripts/tl-claude.sh start --reachable --agent-id $peer -- --resume (running headless claudes cannot be retrofitted, PL-237)." ;;
    dead)
        code=3
        headline="DEAD — peer '$peer' has no fresh presence (status=$status, age=${age}s): the process is gone."
        remediation="Nothing is running to consume the wake. Relaunch the peer, then re-send. Confirm with: /peers --all" ;;
    *)
        echo "diagnose-unconsumed: internal classification error ('$class')" >&2
        exit 4 ;;
esac

if [ "$want_json" -eq 1 ]; then
    jq -cn --arg class "$class" --argjson exit "$code" --arg peer "$peer" \
        --arg status "$status" --arg armed "$armed" --arg age "${age}" \
        --arg remediation "$remediation" \
        '{class:$class, exit:$exit, peer:$peer, status:$status, armed:$armed, age_secs:$age, remediation:$remediation}'
else
    echo "diagnose-unconsumed: $headline"
    echo "  ↳ $remediation"
fi
exit "$code"
