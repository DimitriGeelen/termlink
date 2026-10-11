#!/usr/bin/env bash
# guard-layer: source warn  # arc-011 PH-0 standing controls are EXPECTED red on today's system until PH-1/PH-2 land the status surface (T-3396); a red here is the exit criterion 6.2.1.c(2), not a regression
#
# T-3396 — the six standing controls of arc-011 PH-0 (step 3 §6.2.1.a (4), exit 6.2.1.c (2); R-69, GP-12).
# Each control reproduces ONE real incident class in a disposable, simulated estate (temp dirs, canned
# state, throw-away `sleep` processes) and asks TODAY's status surface the question the requirement
# demands it answer. The status call must NAME the cause (§7.2.d); today it does not, so each control is
# RED for a stated reason. Nothing here touches the live hub, a real inbox, a real key or a real session.
#
#   SC-1 INC-a   mail stored, receipted, never surfaced   -> must say "STORED <n> min ago, not handed over"
#   SC-2 INC-d   signing key filed under another agent    -> must say "signing key found under <other>"
#   SC-3 INC-c   two receivers on one inbox (GP-12 f.2)   -> must say "inbox consumed by two identities"
#   SC-4 GP-12 f.3  agent outside the waker               -> must say "receipts are signed as <X>"
#   SC-5 GP-12 f.4  one cooperating process dead          -> must say "<component> last seen <n> s ago: dead"
#   SC-6 INC-b   stray second hub / wrong runtime dir     -> must say "answering hub id <X>; declared <Y>"
#
# HOW EACH TURNS GREEN (the phase that makes today's status surface say it):
#   SC-1 PH-2 (R-44 surfacing deadline + R-53 "never CLEAR while deaf")  SC-2 PH-2 (start-up key check, R-53)
#   SC-3 PH-2 (store lock + hub-side consumer visibility, R-49)          SC-4 PH-2 (adapter/identity check per tick)
#   SC-5 PH-2 (30 s component heartbeat, R-8.e / R-17)                   SC-6 PH-2 (hub-id check, FX-2, T-3345 id)
# When a control's phase ships, the status surface starts emitting the phrase and the control goes green by
# itself; once all six are green, drop `warn` from the marker above so the suite gates as FAIL (R-69.e).
#
# Usage:  bash tests/standing-controls.sh              run against today's status surface (exit 1 = red as expected)
#         bash tests/standing-controls.sh --self-check prove every control CAN go green against a stub that
#                                                      says the required phrase (kill-check: red is not vacuous)
# Exit:   0 all six green · 1 at least one red · 2 tooling (fail-closed; never a verdict)
# Seams (PL-213): SC_STATUS_CMD overrides the status command (receives SC_SCENARIO, SC_AGENT, SC_NOTIFY_DIR).
set -uo pipefail
PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
API="$PROJECT_ROOT/scripts/notify-sidecar-api.sh"
MODE=run
case "${1:-}" in --self-check) MODE=self ;; "") : ;; *) echo "usage: $0 [--self-check]" >&2; exit 2 ;; esac
[ -r "$API" ] || { echo "TOOLING: cannot read $API" >&2; exit 2; }
command -v python3 >/dev/null && command -v sleep >/dev/null || { echo "TOOLING: python3/sleep missing" >&2; exit 2; }
TMP=$(mktemp -d) || { echo "TOOLING: mktemp failed" >&2; exit 2; }
PIDS=()
cleanup() { for p in "${PIDS[@]:-}"; do [ -n "$p" ] && kill "$p" 2>/dev/null; done; rm -rf "$TMP"; }
trap cleanup EXIT

now_ms() { echo $(( $(date +%s) * 1000 )); }
ago_ms() { echo $(( $(date +%s) * 1000 - $1 * 1000 )); }
spawn() { sleep 300 & PIDS+=("$!"); echo "$!"; }   # disposable stand-in process

# --- stub for --self-check: the "fixed" status surface, saying exactly the §7.2.d phrases ---
STUB="$TMP/stub-status.sh"
cat > "$STUB" <<'EOF'
#!/usr/bin/env bash
case "$SC_SCENARIO" in
 sc1) echo "STORED 10 min ago, not handed over: hook absent" ;;
 sc2) echo "signing key found under other-agent; none under this agent's name" ;;
 sc3) echo "inbox consumed by two identities: A1, B1" ;;
 sc4) echo "receipts are signed as X1; this agent is A1" ;;
 sc5) echo "supervisor last seen 600 s ago: dead" ;;
 sc6) echo "answering hub id H-stray; declared H-main" ;;
esac
EOF

run_status() {   # scenario agent notify-dir -> today's status output (stdout+stderr), hub probe stubbed
    local sc="$1" ag="$2" nd="$3" cmd
    if [ "$MODE" = self ]; then cmd="bash $STUB"
    elif [ -n "${SC_STATUS_CMD:-}" ]; then cmd="$SC_STATUS_CMD"
    else cmd="bash $API status --agent-id $ag --notify-dir $nd"; fi
    SC_SCENARIO="$sc" SC_AGENT="$ag" SC_NOTIFY_DIR="$nd" SIDECAR_API_TEST_HUB_RC=0 \
        SIDECAR_API_TEST_FQDN=sc-host SIDECAR_API_TEST_IP=10.0.0.1 \
        timeout 20 $cmd 2>&1
}

RED=0; GREEN=0; N=0
control() {   # id label required-regex reason-if-red output
    N=$((N+1))
    if printf '%s' "$5" | grep -qiE "$3"; then GREEN=$((GREEN+1)); printf '  GREEN %s %s\n' "$1" "$2"
    else RED=$((RED+1)); printf '  RED   %s %s\n        reason: %s\n' "$1" "$2" "$4"; fi
}
mkagent() {   # dir agent -> a healthy-looking sidecar: fresh heartbeat, live pid
    mkdir -p "$1"; now_ms > "$1/$2.heartbeat"; spawn > "$1/$2.pid"
}

echo "standing controls (T-3396, arc-011 PH-0) — mode: $MODE"

# SC-1 INC-a: mail arrived 10 min ago, receipted, sidecar heartbeat fresh, nothing handed to the agent
d="$TMP/sc1"; mkagent "$d" A1
printf 'pending=3\nlast_mail_topic=dm:A1:B1\nlast_mail_ts=%s\n' "$(ago_ms 600)" > "$d/A1.flag"
out=$(run_status sc1 A1 "$d")
control SC-1 "INC-a unsurfaced mail" 'stored.*not handed over' \
    "status reports a pending count only; it never says mail was STORED and NOT HANDED OVER, so a fresh heartbeat reads as healthy" "$out"

# SC-2 INC-d: the signing key sits under another agent's name; the sidecar refuses 32,205 confirmations
d="$TMP/sc2"; mkagent "$d" A1
mkdir -p "$d/keys"; : > "$d/keys/other-agent.key"          # key filed under the wrong name; none for A1
echo 32205 > "$d/A1.refusals"
out=$(run_status sc2 A1 "$d")
control SC-2 "INC-d misfiled key, 32,205 refusals" 'signing key found under' \
    "status does not look for the agent's key at all and does not surface the refusal count; heartbeat stays ALIVE" "$out"

# SC-3 INC-c: a second identity consumes the same inbox topic
d="$TMP/sc3"; mkagent "$d" A1; mkagent "$d" B1
printf 'consumer=A1\ntopic=dm:shared\n' > "$d/A1.consumer"; printf 'consumer=B1\ntopic=dm:shared\n' > "$d/B1.consumer"
out=$(run_status sc3 A1 "$d")
control SC-3 "INC-c two receivers on one inbox" 'consumed by two identities' \
    "status is per-agent and blind to a second live consumer of the same inbox" "$out"

# SC-4 GP-12 failure 3: agent is outside the waker; its receipts are signed under a different identity
d="$TMP/sc4"; mkagent "$d" A1
printf 'signed_as=X1\n' > "$d/A1.last-receipt"
out=$(run_status sc4 A1 "$d")
control SC-4 "agent outside the waker (receipt identity mismatch)" 'receipts are signed as' \
    "status never compares the receipt-signing identity with the agent's own" "$out"

# SC-5 GP-12 failure 4: sidecar alive, supervisor dead with a stale beat
d="$TMP/sc5"; mkagent "$d" A1
echo 999999 > "$d/.supervisor.pid"; ago_ms 600 > "$d/.supervisor.heartbeat"
out=$(run_status sc5 A1 "$d")
control SC-5 "one cooperating process dead" 'last seen [0-9]+ ?s ago: dead' \
    "status prints supervisor heartbeat_age as a bare figure with no component verdict, and the listener line stays ALIVE" "$out"

# SC-6 INC-b: a stray second hub answers from the default runtime dir; the declared hub is another
d="$TMP/sc6"; mkagent "$d" A1
mkdir -p "$d/hub-main" "$d/hub-stray"; spawn > "$d/hub-main/hub.pid"; spawn > "$d/hub-stray/hub.pid"
echo H-main > "$d/declared-hub-id"; echo H-stray > "$d/hub-stray/hub-id"
out=$(run_status sc6 A1 "$d")
control SC-6 "INC-b stray second hub" 'answering hub id' \
    "status has no hub-id check (no declared id, no answering id); only the on-demand preflight Check 7 sees two hubs" "$out"

echo
if [ "$MODE" = self ]; then
    echo "self-check: $GREEN/$N"
    [ "$GREEN" -eq "$N" ] && exit 0 || { echo "self-check FAILED: a control cannot go green" >&2; exit 1; }
fi
echo "  green: $GREEN   red: $RED   (expected red until PH-2; see header for the phase per control)"
[ "$RED" -eq 0 ]
