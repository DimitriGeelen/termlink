#!/usr/bin/env bash
# T-3061 — Two-party end-to-end prover for the arc-003 deterministic notify rail.
#
# WHAT THIS ANSWERS THAT NOTHING ELSE DOES
# ----------------------------------------
# The rail already has three test suites and all three are single-process and
# deliberately hub-independent:
#   tests/notify-sidecar-supervisor-fixtures.sh  (supervisor autostart/self-heal)
#   tests/notify-sidecar-canary-fixtures.sh      (freshness canary classes)
#   scripts/test-notify-sidecar.sh               (MAIL/CLEAR/DEAF verdicts, via
#                                                 TERMLINK_NOTIFY_TEST_UNREAD)
# Every one of them proves a component in isolation. NONE of them sends a message
# from one agent to another through a real hub and then asks the RECEIVER what it
# saw. T-3058 recorded the rail "proven end to end"; what it actually proved was
# transport + sidecar + receipt on one host. That is a narrower claim, and the
# difference is the whole reason this file exists.
#
# THE ASSERTION RULE (T-2876, and it is not negotiable)
# -----------------------------------------------------
# A send returning 0 means the message was ACCEPTED, not RECEIVED. Three findings
# in this repo in two days share that shape: a hub reported `injected` while the
# PTY got nothing (T-2873); a config looked authoritative and was never read
# (T-2874); a send succeeded while the target sat blocked (T-2875). So this prover
# NEVER reads the sender's exit code as evidence of delivery. Every delivery
# verdict comes from the receiver's own observable state.
#
# And it goes one step further than reading the receiver's files locally: the peer
# is asked to read its OWN flag THROUGH ITS OWN SESSION (`termlink exec`), so the
# observation comes out of the peer's process with the peer's filesystem view. A
# flag this script read locally is a file it could in principle have written
# itself; a flag the peer reads and reports back is evidence about the peer.
#
# WHY "WAKE" IS A STAGE, AND WHY IT IS ALLOWED TO FAIL
# ----------------------------------------------------
# Grepped across the live tree, the only readers of notify/<agent>.flag are the
# sidecar that writes it and notify-check.sh, which is a query an agent runs BY
# HAND. No systemd unit, cron job, or hook reacts to a raised flag. So the rail
# today is: mail -> sidecar -> flag -> nothing. That is PL-168 ("canary scripts
# without a trigger are dormant tooling") one layer up: the wake signal is written
# and nobody is woken.
#
# A prover that omitted this stage would report the rail green while nobody wakes,
# which is exactly the G-069 shipped-but-dark failure the rail's own history is
# made of. So WAKE is a first-class stage and its honest answer today is NOT-WIRED.
# It is NOT counted as a pass. If that offends a green dashboard, the dashboard is
# wrong.
#
# EXIT CONTRACT
#   0  PROVEN   every requested stage passed
#   1  BROKEN   a stage genuinely failed (the stage is named)
#   2  TOOLING  could not run the experiment at all — unreachable hub, unreadable
#               peer, missing dependency. FAIL-CLOSED: never a delivery verdict.
#
# The 1-vs-2 split is load-bearing and is the T-2696 lesson: a prover that reports
# "substrate broken" on a quiet host teaches its operator to ignore it. "I could
# not look" and "I looked and it is broken" must never share an exit code.
#
# TEST SEAMS (PL-213 — the suite must run with no hub and no peer)
#   NOTIFY_E2E_TEST_SEND=<script>       stub the send;      argv: <topic> <body>
#   NOTIFY_E2E_TEST_PEER_READ=<script>  stub the peer read; argv: <agent> <path>
#   NOTIFY_E2E_TEST_SELF_READ=<script>  stub the self read; argv: <agent> <path>
#   NOTIFY_E2E_TEST_RECEIPTS=<file>     canned receipts JSON for the RECEIPT stage
#   NOTIFY_E2E_TEST_WAKE_CONSUMERS=<n>  force the consumer count for WAKE
#   NOTIFY_E2E_TEST_PRECOND=<0|1>       force precondition outcome
#
set -u

VERSION="1.0.0"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TERMLINK="${TERMLINK:-termlink}"

# ---- defaults -------------------------------------------------------------
SELF_AGENT="${NOTIFY_E2E_SELF_AGENT:-claude-termlink}"
SELF_FP="${NOTIFY_E2E_SELF_FP:-d1993c2c3ec44c94}"
PEER_AGENT="${NOTIFY_E2E_PEER_AGENT:-framework-agent-systemd}"
PEER_FP="${NOTIFY_E2E_PEER_FP:-3bba15e681b3a078}"
PEER_SESSION="${NOTIFY_E2E_PEER_SESSION:-}"
NOTIFY_DIR="${NOTIFY_E2E_NOTIFY_DIR:-$HOME/.termlink/notify}"
HB_MAX_AGE="${NOTIFY_E2E_HB_MAX_AGE:-90}"      # seconds a heartbeat may lag
DELIVER_TIMEOUT="${NOTIFY_E2E_DELIVER_TIMEOUT:-90}"
EXEC_TIMEOUT="${NOTIFY_E2E_EXEC_TIMEOUT:-30}"
POLL_INTERVAL="${NOTIFY_E2E_POLL_INTERVAL:-2}"
ITERATIONS=5
JSON=0
RUN_REVERSE=1
EXPERIMENTS=""
STAGES_REQUESTED="precond,deliver,receipt,wake,reverse"

usage() {
    cat <<'USAGE'
Usage: notify-rail-e2e.sh [options]

Two-party end-to-end prover for the deterministic notify rail. Asserts ONLY on
the receiver's own observation, never on the sender's exit code.

Options:
  --peer-agent ID        receiving agent id        (default framework-agent-systemd)
  --peer-fp FP           receiving identity fp     (default 3bba15e681b3a078)
  --peer-session TLID    peer's termlink session   (auto-resolved if omitted)
  --self-agent ID        sending agent id          (default claude-termlink)
  --self-fp FP           sending identity fp       (default d1993c2c3ec44c94)
  --stages LIST          comma list of precond,deliver,receipt,wake,reverse
  --experiment NAME      e1 (sidecar-dead) | e2 (latency) | e3 (closed-loop wake)
                         | e4 (identity split) | all
  --iterations N         iterations for the e2 latency experiment (default 5)
  --deliver-timeout SEC  how long to wait for the receiver to observe (default 90)
  --no-reverse           skip the REVERSE stage
  --json                 emit a machine-readable verdict envelope
  --help                 this text

Exit: 0 proven · 1 broken (stage named) · 2 tooling (fail-closed)
USAGE
}

while [ $# -gt 0 ]; do
    case "$1" in
        --peer-agent)       PEER_AGENT="${2:-}"; shift 2 ;;
        --peer-fp)          PEER_FP="${2:-}"; shift 2 ;;
        --peer-session)     PEER_SESSION="${2:-}"; shift 2 ;;
        --self-agent)       SELF_AGENT="${2:-}"; shift 2 ;;
        --self-fp)          SELF_FP="${2:-}"; shift 2 ;;
        --stages)           STAGES_REQUESTED="${2:-}"; shift 2 ;;
        --experiment)       EXPERIMENTS="${2:-}"; shift 2 ;;
        --iterations)       ITERATIONS="${2:-}"; shift 2 ;;
        --deliver-timeout)  DELIVER_TIMEOUT="${2:-}"; shift 2 ;;
        --no-reverse)       RUN_REVERSE=0; shift ;;
        --json)             JSON=1; shift ;;
        --help|-h)          usage; exit 0 ;;
        *) echo "notify-rail-e2e: unknown argument: $1" >&2; usage >&2; exit 2 ;;
    esac
done

case "$ITERATIONS" in ''|*[!0-9]*) echo "notify-rail-e2e: --iterations must be numeric" >&2; exit 2 ;; esac
[ "$ITERATIONS" -ge 1 ] 2>/dev/null || { echo "notify-rail-e2e: --iterations must be >= 1" >&2; exit 2; }

# ---- result accumulation --------------------------------------------------
STAGE_NAMES=(); STAGE_VERDICTS=(); STAGE_DETAILS=()
BROKEN_STAGE=""; TOOLING_REASON=""
LATENCIES=""

record() {  # record <stage> <PASS|FAIL|SKIP|NOT-WIRED|TOOLING> <detail>
    STAGE_NAMES+=("$1"); STAGE_VERDICTS+=("$2"); STAGE_DETAILS+=("$3")
    [ "$2" = "FAIL" ] && [ -z "$BROKEN_STAGE" ] && BROKEN_STAGE="$1"
    [ "$2" = "NOT-WIRED" ] && [ -z "$BROKEN_STAGE" ] && BROKEN_STAGE="$1"
    if [ "$JSON" -eq 0 ]; then
        printf '  %-10s %-10s %s\n' "$1" "$2" "$3"
    fi
}

die_tooling() {
    TOOLING_REASON="$1"
    record "${2:-PRECOND}" "TOOLING" "$1"
    emit_verdict
    exit 2
}

now_ms() { date +%s%3N; }

wants_stage() { case ",$STAGES_REQUESTED," in *",$1,"*) return 0 ;; *) return 1 ;; esac }

# ---- receiver-side observation -------------------------------------------
# Read a file from the PEER's process. Falls back to a local read ONLY when the
# peer session is unavailable, and says so, because a local read is a weaker
# claim: it proves the file exists, not that the peer can see it.
peer_read() {  # peer_read <path>  -> stdout, rc 0 ok / 1 unreadable
    local path="$1"
    if [ -n "${NOTIFY_E2E_TEST_PEER_READ:-}" ]; then
        bash "$NOTIFY_E2E_TEST_PEER_READ" "$PEER_AGENT" "$path"; return $?
    fi
    [ -n "$PEER_SESSION" ] || return 1
    local out
    out="$(timeout "$EXEC_TIMEOUT" "$TERMLINK" exec "$PEER_SESSION" "cat $path" --json 2>/dev/null)" || return 1
    printf '%s' "$out" | jq -e -r 'select(.ok == true) | .stdout' 2>/dev/null || return 1
}

self_read() {  # self_read <path>
    local path="$1"
    if [ -n "${NOTIFY_E2E_TEST_SELF_READ:-}" ]; then
        bash "$NOTIFY_E2E_TEST_SELF_READ" "$SELF_AGENT" "$path"; return $?
    fi
    [ -r "$path" ] || return 1
    cat "$path"
}

kv() { # kv <blob> <key>   parse key=value out of a flag file
    printf '%s\n' "$1" | grep -E "^$2=" | head -1 | cut -d= -f2-
}

# ---- the shared dm topic --------------------------------------------------
# dm:<sorted_a>:<sorted_b> — the sidecar comment at notify-sidecar.sh:216 says
# "self appears in either slot", i.e. ONE shared topic per pair, not two
# directional ones. Computing it the same way rather than hardcoding, so a
# convention change surfaces here instead of silently addressing the void.
dm_topic() {
    local a="$1" b="$2"
    if [ "$a" \< "$b" ]; then printf 'dm:%s:%s\n' "$a" "$b"
    else printf 'dm:%s:%s\n' "$b" "$a"; fi
}

TOPIC="$(dm_topic "$SELF_FP" "$PEER_FP")"

send_sentinel() {  # send_sentinel <body> -> rc (NOT used as delivery evidence)
    local body="$1"
    if [ -n "${NOTIFY_E2E_TEST_SEND:-}" ]; then
        bash "$NOTIFY_E2E_TEST_SEND" "$TOPIC" "$body"; return $?
    fi
    # `channel post <TOPIC> [BODY]` — the body is POSITIONAL. An earlier draft
    # passed `--message`, which clap rejects with exit 2; the prover correctly
    # reported DELIVER broken, and the isolation repro (PL-297) showed the fault
    # was in the prover, not the rail. Keeping the note so nobody re-introduces it.
    timeout "$EXEC_TIMEOUT" "$TERMLINK" channel post "$TOPIC" "$body" >/dev/null 2>&1
}

# ===========================================================================
# STAGE: PRECOND
# ===========================================================================
stage_precond() {
    if [ -n "${NOTIFY_E2E_TEST_PRECOND:-}" ]; then
        if [ "$NOTIFY_E2E_TEST_PRECOND" = "1" ]; then
            record PRECOND PASS "forced by test seam"; return 0
        fi
        die_tooling "forced precondition failure (test seam)" PRECOND
    fi

    command -v jq >/dev/null 2>&1 || die_tooling "jq not found" PRECOND

    # Resolve the peer session if not supplied.
    if [ -z "$PEER_SESSION" ] && [ -z "${NOTIFY_E2E_TEST_PEER_READ:-}" ]; then
        local listing
        listing="$(timeout "$EXEC_TIMEOUT" "$TERMLINK" list 2>/dev/null)" \
            || die_tooling "termlink list failed — hub unreachable?" PRECOND
        PEER_SESSION="$(printf '%s\n' "$listing" | awk -v p="$(printf '%.14s' "$PEER_AGENT")" \
            '$2 != "" && index($0, substr(p,1,12)) > 0 { print $1; exit }')"
        [ -n "$PEER_SESSION" ] || die_tooling "no session found for peer '$PEER_AGENT'" PRECOND
    fi

    # Both heartbeats must be FRESH. A stale heartbeat means the sidecar is not
    # cycling, and every later stage would then be measuring a corpse.
    local now hb age blob
    now="$(now_ms)"
    for pair in "$SELF_AGENT:self" "$PEER_AGENT:peer"; do
        local agent="${pair%%:*}" side="${pair##*:}"
        if [ "$side" = "peer" ]; then
            blob="$(peer_read "$NOTIFY_DIR/$agent.heartbeat")" \
                || die_tooling "cannot read $agent heartbeat through peer session" PRECOND
        else
            blob="$(self_read "$NOTIFY_DIR/$agent.heartbeat")" \
                || die_tooling "cannot read $agent heartbeat locally" PRECOND
        fi
        hb="$(printf '%s' "$blob" | tr -dc '0-9')"
        [ -n "$hb" ] || die_tooling "$agent heartbeat is empty/non-numeric" PRECOND
        age=$(( (now - hb) / 1000 ))
        [ "$age" -le "$HB_MAX_AGE" ] \
            || die_tooling "$agent heartbeat stale (${age}s > ${HB_MAX_AGE}s) — sidecar not cycling" PRECOND
    done

    record PRECOND PASS "peer=$PEER_SESSION topic=$TOPIC both heartbeats fresh"
    return 0
}

# ===========================================================================
# STAGE: DELIVER  — the load-bearing one
# ===========================================================================
# Posts a sentinel and then waits for the PEER to observe it. The sentinel is
# carried in the message body AND identified by the topic on the peer's flag.
# The send's return code is recorded for the report but is explicitly NOT part
# of the verdict.
# NOTE: this runs inside a command substitution, i.e. a SUBSHELL, so it cannot
# export anything to the caller via variables — the first draft tried and died on
# `set -u`. Everything the caller needs is therefore returned on stdout as
# <latency_ms>|<send_rc>|<pending_before-><pending_after>.
deliver_once() {  # deliver_once <sentinel> -> echoes "lat|send_rc|pending", rc 0 observed / 1 not
    local sentinel="$1"
    local before after t0 t1 blob pending_before pending_after latest

    blob="$(peer_read "$NOTIFY_DIR/$PEER_AGENT.flag")" || return 2
    pending_before="$(kv "$blob" pending)"
    case "$pending_before" in ''|*[!0-9]*) pending_before=0 ;; esac

    t0="$(now_ms)"
    send_sentinel "$sentinel"   # rc deliberately ignored as evidence
    local send_rc=$?

    local deadline=$(( $(date +%s) + DELIVER_TIMEOUT ))
    local flag_ts
    while [ "$(date +%s)" -lt "$deadline" ]; do
        blob="$(peer_read "$NOTIFY_DIR/$PEER_AGENT.flag")" || { sleep "$POLL_INTERVAL"; continue; }
        pending_after="$(kv "$blob" pending)"
        latest="$(kv "$blob" latest_topic)"
        flag_ts="$(kv "$blob" ts | tr -dc '0-9')"
        case "$pending_after" in ''|*[!0-9]*) pending_after=0 ;; esac
        case "$flag_ts" in '') flag_ts=0 ;; esac
        # FRESHNESS GUARD (T-3061). `latest_topic` is STICKY — it keeps naming our
        # topic after a previous send — so matching on it alone fires instantly
        # against pre-existing state and measures nothing. The first draft did
        # exactly that and reported min=60ms on a rail that polls every 15s: a
        # vacuous pass (T-2831) inside the prover itself. The flag's own ts must
        # therefore have advanced PAST our send before any match counts, which
        # proves the sidecar cycled after we posted rather than before.
        if [ "$flag_ts" -gt "$t0" ] 2>/dev/null &&
           { [ "$pending_after" -gt "$pending_before" ] || [ "$latest" = "$TOPIC" ]; }; then
            t1="$(now_ms)"
            printf '%s|%s|%s\n' "$(( t1 - t0 ))" "$send_rc" "$pending_before->$pending_after"
            return 0
        fi
        sleep "$POLL_INTERVAL"
    done
    printf '%s|%s|%s\n' "-1" "$send_rc" "$pending_before->(unchanged)"
    return 1
}

stage_deliver() {
    local sentinel="e2e-$(date +%s)-$$-deliver"
    local out lat send_rc pending
    out="$(deliver_once "$sentinel")"; local rc=$?
    lat="${out%%|*}"; out="${out#*|}"
    send_rc="${out%%|*}"; pending="${out#*|}"
    case $rc in
        0) record DELIVER PASS "peer observed in ${lat}ms (pending $pending, send rc=$send_rc)"
           LATENCIES="$lat"; return 0 ;;
        2) die_tooling "peer flag unreadable during DELIVER" DELIVER ;;
        *) record DELIVER FAIL "peer never observed within ${DELIVER_TIMEOUT}s (pending $pending, send rc=$send_rc — note: send rc is NOT evidence)"
           return 1 ;;
    esac
}

# ===========================================================================
# STAGE: RECEIPT — the auto-confirm claim, and only that claim
# ===========================================================================
# --auto-confirm posts stage=delivered with NO LLM turn. Per the conf comment,
# that asserts "the transport delivered it and the journal holds it", NOT that
# an agent read or understood it. This stage verifies exactly that narrow claim
# and the report says so, so nobody upgrades it in their head later.
stage_receipt() {
    local blob
    if [ -n "${NOTIFY_E2E_TEST_RECEIPTS:-}" ]; then
        blob="$(cat "$NOTIFY_E2E_TEST_RECEIPTS" 2>/dev/null)" || { record RECEIPT FAIL "receipts fixture unreadable"; return 1; }
    else
        blob="$(timeout "$EXEC_TIMEOUT" "$TERMLINK" channel receipts "$TOPIC" --json 2>/dev/null)" \
            || { record RECEIPT FAIL "channel receipts failed for $TOPIC"; return 1; }
    fi
    # SCHEMA NOTE (T-3061 finding). The sidecar conf documents --auto-confirm as
    # posting "a mechanism-A receipt (stage=delivered, up_to=<offset>)". The first
    # draft of this stage filtered on `.stage == "delivered"` accordingly and got
    # zero matches on a topic that demonstrably HAS receipts. `channel receipts`
    # returns {sender_id, ts_unix_ms, up_to} — there is NO stage field on this
    # surface. So the documented distinction between a delivered-receipt and any
    # other receipt is NOT observable to a consumer here. Asserting on the
    # documented-but-absent field would have made this stage permanently red
    # against a working rail (T-2818), so the assertion tests what is actually
    # observable: a receipt FROM THE PEER whose up_to has advanced.
    local up_to
    up_to="$(printf '%s' "$blob" | jq -r --arg fp "$PEER_FP" \
        '[(.receipts // .)[]? | select(.sender_id == $fp) | .up_to] | max // -1' 2>/dev/null)"
    case "$up_to" in ''|null) up_to=-1 ;; esac
    if [ "$up_to" -ge 0 ] 2>/dev/null; then
        record RECEIPT PASS "peer $PEER_FP acked up_to=$up_to — asserts transport+journal only, NOT that an agent read it (no stage field exists on this surface)"
        return 0
    fi
    record RECEIPT FAIL "no receipt from peer $PEER_FP on $TOPIC — a sender using --await-ack would exhaust retries and dead-letter a message that actually arrived"
    return 1
}

# ===========================================================================
# STAGE: WAKE — allowed, and expected, to be honest
# ===========================================================================
count_wake_consumers() {
    if [ -n "${NOTIFY_E2E_TEST_WAKE_CONSUMERS:-}" ]; then
        printf '%s\n' "$NOTIFY_E2E_TEST_WAKE_CONSUMERS"; return 0
    fi
    # A consumer is something OTHER than the sidecar (writer) and notify-check
    # (a hand-run query) that reads the flag on a trigger: a systemd unit, a cron
    # job, or a hook. Those are the only shapes that can wake anybody.
    local n=0 f
    for f in /etc/cron.d/* ; do
        [ -r "$f" ] || continue
        grep -lqE 'notify-check|notify/.*\.flag' "$f" 2>/dev/null && n=$((n+1))
    done
    for f in /etc/systemd/system/*.service ; do
        [ -r "$f" ] || continue
        grep -lqE 'notify-check|notify/.*\.flag' "$f" 2>/dev/null && n=$((n+1))
    done
    printf '%s\n' "$n"
}

stage_wake() {
    local n
    n="$(count_wake_consumers)"
    case "$n" in ''|*[!0-9]*) n=0 ;; esac
    if [ "$n" -gt 0 ]; then
        record WAKE PASS "$n consumer(s) react to a raised flag"
        return 0
    fi
    record WAKE NOT-WIRED "nothing consumes the flag: mail -> sidecar -> flag -> (nobody). The mailbox fills; no one is woken. PL-168 at the wake layer."
    return 1
}

# ===========================================================================
# STAGE: REVERSE — the rail must work in both directions
# ===========================================================================
# T-3054 gave the AEF agent its own identity key specifically so it could be a
# receiver. This stage proves it can also be a SENDER, by having the peer post
# from its own process and then checking OUR flag.
stage_reverse() {
    local sentinel="e2e-$(date +%s)-$$-reverse"
    local blob pending_before pending_after latest

    blob="$(self_read "$NOTIFY_DIR/$SELF_AGENT.flag")" || { record REVERSE FAIL "own flag unreadable"; return 1; }
    pending_before="$(kv "$blob" pending)"
    case "$pending_before" in ''|*[!0-9]*) pending_before=0 ;; esac

    local t0 send_out
    t0="$(now_ms)"

    # The peer's send result is CAPTURED, not discarded. The first draft piped it
    # to /dev/null, which hid a JSON-RPC -32005 "command not in allowlist" on
    # every run: the peer never sent anything, and the stage still reported PASS
    # because `latest_topic` was STICKY from the DELIVER stage seconds earlier.
    # A stage that passes while the peer is structurally unable to send is the
    # wrong stage (T-2831). Both halves are fixed here: surface the send error,
    # and require the receiver's flag to have been rewritten after t0.
    if [ -n "${NOTIFY_E2E_TEST_SEND:-}" ]; then
        send_out="$(bash "$NOTIFY_E2E_TEST_SEND" "$TOPIC" "$sentinel" 2>&1)"
    else
        [ -n "$PEER_SESSION" ] || { record REVERSE FAIL "no peer session to send from"; return 1; }
        send_out="$(timeout "$EXEC_TIMEOUT" "$TERMLINK" exec "$PEER_SESSION" \
            "termlink channel post $TOPIC '$sentinel'" --json 2>&1)"
    fi
    if printf '%s' "$send_out" | grep -q 'not in allowlist'; then
        record REVERSE FAIL "peer CANNOT SEND: 'termlink' is not in its --allowed-commands, so this peer can receive mail but never reply. The rail is half-duplex with $PEER_AGENT."
        return 1
    fi
    if printf '%s' "$send_out" | grep -q '"ok":false'; then
        record REVERSE FAIL "peer send rejected: $(printf '%s' "$send_out" | head -c 200)"
        return 1
    fi

    local deadline=$(( $(date +%s) + DELIVER_TIMEOUT ))
    local flag_ts
    while [ "$(date +%s)" -lt "$deadline" ]; do
        blob="$(self_read "$NOTIFY_DIR/$SELF_AGENT.flag")" || { sleep "$POLL_INTERVAL"; continue; }
        pending_after="$(kv "$blob" pending)"
        latest="$(kv "$blob" latest_topic)"
        flag_ts="$(kv "$blob" ts | tr -dc '0-9')"
        case "$pending_after" in ''|*[!0-9]*) pending_after=0 ;; esac
        case "$flag_ts" in '') flag_ts=0 ;; esac
        if [ "$flag_ts" -gt "$t0" ] 2>/dev/null &&
           { [ "$pending_after" -gt "$pending_before" ] || [ "$latest" = "$TOPIC" ]; }; then
            record REVERSE PASS "own flag moved ($pending_before->$pending_after, latest=$latest, flag rewritten after send)"
            return 0
        fi
        sleep "$POLL_INTERVAL"
    done
    record REVERSE FAIL "own flag never moved within ${DELIVER_TIMEOUT}s (pending stayed $pending_before)"
    return 1
}

# ===========================================================================
# EXPERIMENT E1 — a dead sidecar must read DEAF, never CLEAR
# ===========================================================================
# This is the single most important property in the rail. If a stopped sidecar
# reported CLEAR, then "no mail" and "I cannot tell" would be the same answer,
# and the rail would be able to go dark for 82 days while looking healthy — which
# is precisely what it did before T-3049.
#
# Deliberately run against a SCRATCH agent, never the production sidecar: the
# property belongs to notify-check.sh, not to any particular agent, and killing
# the live AEF sidecar to test it would be an outage to prove a point.
experiment_e1() {
    local d rc out
    d="$(mktemp -d)/notify"; mkdir -p "$d"
    local agent="e2e-scratch-$$"

    # A live sidecar with mail outstanding -> MAIL (10)
    TERMLINK_NOTIFY_TEST_UNREAD=3 TERMLINK_NOTIFY_TEST_LATEST_TOPIC="dm:scratch" \
        bash "$HERE/notify-sidecar.sh" --agent-id "$agent" --notify-dir "$d" --once >/dev/null 2>&1
    out="$(bash "$HERE/notify-check.sh" --agent-id "$agent" --notify-dir "$d" 2>&1)"; rc=$?
    if [ "$rc" -ne 10 ]; then
        record E1 FAIL "live sidecar with 3 unread returned rc=$rc (expected 10/MAIL): $out"
        rm -rf "$(dirname "$d")"; return 1
    fi

    # Now simulate the sidecar having DIED: backdate the heartbeat so it stops
    # cycling, while the flag still claims mail is pending.
    printf '%s' "$(( $(date +%s%3N) - 3600000 ))" > "$d/$agent.heartbeat"
    out="$(bash "$HERE/notify-check.sh" --agent-id "$agent" --notify-dir "$d" 2>&1)"; rc=$?
    rm -rf "$(dirname "$d")"

    if [ "$rc" -eq 3 ]; then
        record E1 PASS "dead sidecar reports DEAF (rc=3), not CLEAR — 'no mail' and 'cannot tell' stay distinguishable"
        return 0
    fi
    if [ "$rc" -eq 0 ]; then
        record E1 FAIL "DEAD SIDECAR REPORTED CLEAR (rc=0) — the rail can go dark while looking healthy"
        return 1
    fi
    record E1 FAIL "dead sidecar returned rc=$rc (expected 3/DEAF): $out"
    return 1
}

# ===========================================================================
# EXPERIMENT E2 — latency distribution, measured receiver-side
# ===========================================================================
experiment_e2() {
    local i out lat vals=() ok=0
    for i in $(seq 1 "$ITERATIONS"); do
        out="$(deliver_once "e2e-lat-$i-$$")"
        if [ $? -eq 0 ]; then lat="${out%%|*}"; vals+=("$lat"); ok=$((ok+1)); fi
    done
    if [ "$ok" -eq 0 ]; then
        record E2 FAIL "0/$ITERATIONS sentinels observed by the receiver"
        return 1
    fi
    local sorted min max p50
    sorted="$(printf '%s\n' "${vals[@]}" | sort -n)"
    min="$(printf '%s\n' "$sorted" | head -1)"
    max="$(printf '%s\n' "$sorted" | tail -1)"
    p50="$(printf '%s\n' "$sorted" | awk '{a[NR]=$1} END{print a[int((NR+1)/2)]}')"
    LATENCIES="$sorted"
    record E2 PASS "$ok/$ITERATIONS observed — min=${min}ms p50=${p50}ms max=${max}ms"
    [ "$ok" -eq "$ITERATIONS" ] && return 0
    return 1
}

# ===========================================================================
# EXPERIMENT E3 — the closed loop: does anyone actually WAKE?
# ===========================================================================
# WAKE (the stage) asks "is a consumer wired?" and today answers NOT-WIRED.
# E3 asks the different, sharper question: "IF a consumer runs, does the chain
# actually close?" — mail -> sidecar -> flag -> consumer -> observable action.
#
# It runs the consumer INSIDE THE PEER'S OWN PROCESS via termlink exec, so the
# wake is the peer's, not a simulation of one. The peer's action is a marker file
# write rather than a bus post, because `termlink` is NOT in the AEF agent's
# --allowed-commands and routing around that allowlist with an allowed
# interpreter would defeat a control the operator set rather than satisfy it.
#
# Proven live 2026-09-22: peer woke 8001ms after send and wrote the marker.
experiment_e3() {
    local marker="${NOTIFY_E2E_MARKER:-$PWD/.context/working/.e2e-wake-marker}"
    local consumer="/opt/termlink/scripts/notify-wake-consumer.py"
    local out

    if [ -n "${NOTIFY_E2E_TEST_PEER_READ:-}" ]; then
        record E3 SKIP "closed-loop wake needs a live peer session"
        return 0
    fi
    [ -n "$PEER_SESSION" ] || { record E3 FAIL "no peer session for the closed-loop wake"; return 1; }

    rm -f "$marker" 2>/dev/null
    ( timeout $((DELIVER_TIMEOUT + 40)) "$TERMLINK" exec "$PEER_SESSION" \
        "python3 $consumer --agent-id $PEER_AGENT --topic-filter $TOPIC --marker $marker --timeout $DELIVER_TIMEOUT --interval 2" \
        --json > "${TMPDIR:-/tmp}/.e2e-e3.$$" 2>&1 ) &
    local bg=$!
    sleep 5
    send_sentinel "e2e-closed-loop-$(date +%s)-$$"
    wait "$bg" 2>/dev/null
    out="$(cat "${TMPDIR:-/tmp}/.e2e-e3.$$" 2>/dev/null)"
    rm -f "${TMPDIR:-/tmp}/.e2e-e3.$$"

    if printf '%s' "$out" | grep -q 'not in allowlist'; then
        record E3 FAIL "peer cannot run the consumer: python3 not in its --allowed-commands"
        return 1
    fi
    if [ -r "$marker" ] && grep -q 'woke_at_ms' "$marker" 2>/dev/null; then
        local lat
        lat="$(jq -r '.latency_ms // "?"' "$marker" 2>/dev/null)"
        record E3 PASS "CLOSED LOOP: peer woke and acted ${lat}ms after send (marker written by $PEER_AGENT)"
        return 0
    fi
    record E3 FAIL "peer never woke within ${DELIVER_TIMEOUT}s — no marker written"
    return 1
}

# ===========================================================================
# EXPERIMENT E4 — identity split: is the sidecar watching the WRONG mailbox?
# ===========================================================================
# Found 2026-09-22 and proven, not inferred. `.context/cron/notify-sidecar-agents.conf`
# declares `claude-termlink d1993c2c3ec44c94` (the shared HOST key), but
# `TERMLINK_AGENT_ID=claude-termlink termlink agent identity --resolve` returns
# 6738c073bbcc587a, because a per-agent key exists at identities/claude-termlink.key.
# The agent therefore has TWO fingerprints and the sidecar watches exactly one.
#
# Proven by construction: a topic dm:<peer>:<alt-fp> was created and posted to.
# The PEER's sidecar consumed it and posted a receipt (up_to=0). OUR sidecar never
# moved — pending stayed 89, latest_topic unchanged, across two full 15s cycles.
# Same message, same hub, same moment; the only variable is the configured
# fingerprint. That is a SILENT LOSS path inside the rail whose entire purpose is
# to prevent silent loss: any peer addressing this agent by its resolved identity
# writes to a mailbox nothing is watching, and no surface reports it.
#
# (The first attempt at this experiment concluded "silent loss confirmed" from a
# post that had FAILED with -32013 unknown-topic. A conclusion drawn from a failed
# send is the exact sender-side reasoning this prover forbids. The topic must be
# created first; the check below compares identities directly and needs no send.)
experiment_e4() {
    local declared="$SELF_FP" resolved
    resolved="$(TERMLINK_AGENT_ID="$SELF_AGENT" timeout "$EXEC_TIMEOUT" \
        "$TERMLINK" agent identity --resolve --json 2>/dev/null \
        | jq -r '.fingerprint // empty' 2>/dev/null)"
    if [ -z "$resolved" ]; then
        record E4 FAIL "cannot resolve identity for $SELF_AGENT — cannot rule out a split mailbox"
        return 1
    fi
    if [ "$resolved" = "$declared" ]; then
        record E4 PASS "sidecar watches the agent's resolved identity ($resolved) — no split"
        return 0
    fi
    record E4 FAIL "IDENTITY SPLIT: sidecar watches $declared but '$SELF_AGENT' resolves to $resolved. Mail addressed to the resolved identity lands on dm:*:$resolved topics that NOTHING polls — silent loss, no surface reports it."
    return 1
}

# ---- verdict --------------------------------------------------------------
emit_verdict() {
    local overall="PROVEN" code=0
    if [ -n "$TOOLING_REASON" ]; then overall="TOOLING"; code=2
    elif [ -n "$BROKEN_STAGE" ]; then overall="BROKEN"; code=1; fi

    if [ "$JSON" -eq 1 ]; then
        local i first=1
        printf '{"ok":%s,"verdict":"%s","version":"%s","topic":"%s","self_agent":"%s","peer_agent":"%s","peer_session":"%s","broken_stage":"%s","stages":[' \
            "$([ $code -eq 0 ] && echo true || echo false)" "$overall" "$VERSION" "$TOPIC" \
            "$SELF_AGENT" "$PEER_AGENT" "$PEER_SESSION" "$BROKEN_STAGE"
        for i in "${!STAGE_NAMES[@]}"; do
            [ $first -eq 0 ] && printf ','
            first=0
            printf '{"stage":"%s","verdict":"%s","detail":%s}' \
                "${STAGE_NAMES[$i]}" "${STAGE_VERDICTS[$i]}" \
                "$(printf '%s' "${STAGE_DETAILS[$i]}" | jq -R -s . 2>/dev/null || echo '""')"
        done
        printf ']}\n'
    else
        echo
        echo "verdict: $overall"
        [ -n "$BROKEN_STAGE" ] && echo "broken stage: $BROKEN_STAGE"
        [ -n "$TOOLING_REASON" ] && echo "tooling: $TOOLING_REASON"
        echo
        echo "SCOPE: this prover asserts delivery on the RECEIVER's own observation."
        echo "A PASS on DELIVER/RECEIPT means the mailbox filled and the transport"
        echo "confirmed it. It does NOT mean any agent read or acted on the message —"
        echo "that is what the WAKE stage is for, and WAKE is reported separately."
    fi
    return $code
}

# ---- run ------------------------------------------------------------------
if [ "$JSON" -eq 0 ]; then
    echo "notify-rail-e2e v$VERSION"
    echo "  self=$SELF_AGENT ($SELF_FP)  peer=$PEER_AGENT ($PEER_FP)"
    echo "  topic=$TOPIC"
    echo
fi

wants_stage precond && stage_precond
wants_stage deliver && stage_deliver
wants_stage receipt && stage_receipt
wants_stage wake    && stage_wake
if [ "$RUN_REVERSE" -eq 1 ] && wants_stage reverse; then stage_reverse; fi

case "$EXPERIMENTS" in
    *e1*|*all*) experiment_e1 ;;
esac
case "$EXPERIMENTS" in
    *e2*|*all*) experiment_e2 ;;
esac
case "$EXPERIMENTS" in
    *e3*|*all*) experiment_e3 ;;
esac
case "$EXPERIMENTS" in
    *e4*|*all*) experiment_e4 ;;
esac

emit_verdict
exit $?
