#!/usr/bin/env bash
# comms-selftest.sh (T-2482) -- affirmative, staged, single-command proof that
# the core comms round-trip to a LIVE peer works RIGHT NOW, attributed to the
# link that broke.
#
# WHY: the codebase has 11 failure-detecting canaries + four observability arcs,
# all of which detect the substrate breaking AFTER THE FACT. The charter's core
# promise -- "discover each other, exchange durable messages" -- had no single
# on-demand proof that the FULL round-trip works, pinned to the broken stage.
# `agent-send.sh --to <id>` already composes discover + send + consume-confirm,
# but collapses to ONE exit code (can't say DISCOVER=PASS / SEND=PASS /
# CONSUME=FAIL), is negative-framed (fires a real turn, fails loud), and has no
# affirmative "all three green" report. The operator's recurring complaint --
# "why is there still no response?" -- is exactly that collapse into silence.
#
# This is a THIN staged wrapper over primitives that already exist:
#   STAGE 1  DISCOVER  <- scripts/diagnose-unconsumed.sh (presence + pty_session)
#   STAGE 2  SEND      <- scripts/agent-send.sh (posts a durable turn)
#   STAGE 3  CONSUME   <- agent-send.sh's receipt wait (peer acked?)
# and it turns agent-send's collapsed exit code into a per-stage PASS/FAIL
# breakdown so a failure is attributed to discover vs send vs consume.
#
# EXIT CODES (contract-stable):
#   0  round-trip proven   -- every run stage PASSed.
#   1  round-trip broken    -- a stage FAILed; the report names which.
#   2  tooling error        -- could not read a required signal (fail-closed:
#                             an un-probeable peer is never reported "proven").
#
# USAGE:
#   comms-selftest.sh --peer <agent_id> [--message <text>] [--cid <id>]
#                     [--hub <addr>] [--discover-only] [--json]
#
#   --peer           the peer agent_id to prove the round-trip against (required)
#   --message        override the synthetic proof-ping text
#   --cid            conversation_id to scope the send/receipt (optional)
#   --hub            hub address to probe (default: local hub)
#   --discover-only  run the side-effect-free DISCOVER stage alone (assert the
#                    peer is reachable + armed WITHOUT firing a turn)
#   --json           emit {ok, peer, stages:[{stage,status,detail}], verdict,
#                    broken_stage} instead of the human report
#
# The SEND stage is bounded by COMMS_SELFTEST_SEND_TIMEOUT (default 120s) so the
# prover never hangs: no receipt within the bound renders as CONSUME=FAIL(timeout).
#
# TEST HOOKS (PL-213 -- host-independent, no live hub; see test-comms-selftest.sh):
#   TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON=<file>  (inherited by diagnose-unconsumed.sh
#                                                 for the DISCOVER stage)
#   COMMS_SELFTEST_TEST_SEND_RC=<int>            canned agent-send.sh exit code,
#                                                 skips the real send (SEND+CONSUME)
#
# Origin: T-2468 purpose-review (4th re-issue) -> this build. The affirmative
# complement to the 11 failure-detecting canaries: "prove it works" vs "detect
# when it broke". Composes T-2479 (diagnose-unconsumed) + arc-003 agent-send.
set -uo pipefail

SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

peer="" message="" cid="" hub="" discover_only=0 want_json=0

tooling() { # <msg> -- exit 2, fail-closed
    if [ "$want_json" -eq 1 ]; then
        jq -cn --arg e "$1" '{ok:false, verdict:"tooling-error", error:$e}' 2>/dev/null \
            || printf '{"ok":false,"verdict":"tooling-error","error":"%s"}\n' "$1"
    else
        echo "comms-selftest: $1" >&2
    fi
    exit 2
}

while [ $# -gt 0 ]; do
    case "$1" in
        --peer)          peer="${2:-}"; shift 2 ;;
        --message)       message="${2:-}"; shift 2 ;;
        --cid)           cid="${2:-}"; shift 2 ;;
        --hub)           hub="${2:-}"; shift 2 ;;
        --discover-only) discover_only=1; shift ;;
        --json)          want_json=1; shift ;;
        -h|--help)
            sed -n '2,60p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
            exit 0 ;;
        *) echo "comms-selftest: unknown arg '$1'" >&2; exit 2 ;;
    esac
done

command -v jq >/dev/null 2>&1 || tooling "jq not found (required)"
[ -n "$peer" ] || tooling "--peer <agent_id> is required"

# stage state
disc_status="?"; disc_detail=""
send_status="skipped"; send_detail=""
cons_status="skipped"; cons_detail=""
broken=""

diag_class() { # runs diagnose-unconsumed for <peer>, echoes .class ("" on tooling)
    local dj
    dj="$(bash "$SELF_DIR/diagnose-unconsumed.sh" --peer "$peer" \
            ${cid:+--cid "$cid"} ${hub:+--hub "$hub"} --json 2>/dev/null || true)"
    printf '%s' "$dj" | jq -r '.class // empty' 2>/dev/null || true
}

# --- STAGE 1: DISCOVER -------------------------------------------------------
class="$(diag_class)"
[ -n "$class" ] || tooling "could not read peer presence (diagnose-unconsumed returned nothing)"
case "$class" in
    dead)
        disc_status="FAIL"; disc_detail="peer dead/absent — no live presence"; broken="DISCOVER" ;;
    unwakeable)
        disc_status="FAIL"; disc_detail="LIVE but NOT armed (no pty_session): nothing can ring its PTY"; broken="DISCOVER" ;;
    consumed|busy-or-manual|live-armed)
        disc_status="PASS"; disc_detail="peer is LIVE + armed" ;;
    *)
        tooling "unexpected diagnose class '$class'" ;;
esac

render_and_exit() { # <exit-code>
    local code="$1" verdict
    if [ "$code" -eq 0 ]; then verdict="round-trip-proven"; else verdict="broken"; fi
    if [ "$want_json" -eq 1 ]; then
        jq -cn --arg peer "$peer" \
            --arg ds "$disc_status" --arg dd "$disc_detail" \
            --arg ss "$send_status" --arg sd "$send_detail" \
            --arg cs "$cons_status" --arg cd "$cons_detail" \
            --arg verdict "$verdict" --arg broken "$broken" \
            '{ok:true, peer:$peer, verdict:$verdict,
              broken_stage:(if $broken=="" then null else $broken end),
              stages:[{stage:"DISCOVER",status:$ds,detail:$dd},
                      {stage:"SEND",status:$ss,detail:$sd},
                      {stage:"CONSUME",status:$cs,detail:$cd}]}'
    else
        echo "comms-selftest: round-trip proof to peer '$peer'"
        printf '  DISCOVER : %-7s — %s\n' "$disc_status" "$disc_detail"
        printf '  SEND     : %-7s — %s\n' "$send_status" "$send_detail"
        printf '  CONSUME  : %-7s — %s\n' "$cons_status" "$cons_detail"
        if [ "$code" -eq 0 ]; then
            if [ "$send_status" = "skipped" ]; then
                echo "  → DISCOVER-ONLY: peer reachable + armed (no turn fired)."
            else
                echo "  → PROVEN: the full discover→send→consume round-trip works."
            fi
        else
            echo "  → BROKEN at $broken. The link is not silently failing anymore — it is named."
        fi
    fi
    exit "$code"
}

# DISCOVER failed -> stop before a pointless send
[ "$disc_status" = "FAIL" ] && render_and_exit 1

# --discover-only -> side-effect-free health check, done
if [ "$discover_only" -eq 1 ]; then
    render_and_exit 0
fi

# --- STAGE 2 + 3: SEND + CONSUME --------------------------------------------
# The send is wrapped in an OWN bounded timeout so the prover NEVER hangs an
# operator: a receipt that never arrives renders as CONSUME=FAIL(timeout), not a
# wedged terminal. That "no response within bound" IS the "why is there still no
# response?" case the verb exists to surface.
send_timeout="${COMMS_SELFTEST_SEND_TIMEOUT:-120}"
msg="${message:-[comms-selftest] round-trip proof ping — your receipt is the proof, no action needed}"
if [ -n "${COMMS_SELFTEST_TEST_SEND_RC:-}" ]; then
    send_rc="$COMMS_SELFTEST_TEST_SEND_RC"
else
    timeout "$send_timeout" bash "$SELF_DIR/agent-send.sh" --to "$peer" --message "$msg" \
        ${cid:+--conversation-id "$cid"} >/dev/null 2>&1
    send_rc=$?
fi

case "$send_rc" in
    0)  # delivered AND acked
        send_status="PASS"; send_detail="durable turn written to the hub"
        cons_status="PASS"; cons_detail="peer posted a receipt (consumed)" ;;
    3)  # arc-003 RC3b: turn durably written, but no receipt arrived
        send_status="PASS"; send_detail="durable turn written to the hub"
        cons_status="FAIL"; broken="CONSUME"
        c2="$(diag_class)"
        cons_detail="no receipt — peer did not consume (diagnosis: ${c2:-unknown}, G-083)" ;;
    124) # our bounded timeout tripped: turn was posted, receipt never arrived in time
        send_status="PASS"; send_detail="durable turn written to the hub"
        cons_status="FAIL"; broken="CONSUME"
        c2="$(diag_class)"
        cons_detail="no receipt within ${send_timeout}s (timed out; peer busy/manual: ${c2:-unknown}, G-083)" ;;
    2)  # precondition (peer lost its pty between discover and send, or usage)
        send_status="FAIL"; send_detail="agent-send precondition failed (peer unarmed at send time / usage)"
        cons_status="n/a"; cons_detail="not reached"; broken="SEND" ;;
    *)  # any other non-zero
        send_status="FAIL"; send_detail="agent-send exited $send_rc"
        cons_status="n/a"; cons_detail="not reached"; broken="SEND" ;;
esac

[ -z "$broken" ] && render_and_exit 0
render_and_exit 1
