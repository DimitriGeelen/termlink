#!/usr/bin/env bash
# T-3069 — THE INJECTOR. The missing middle of the rail (arc-011 slices S7 + S10).
#
# WHAT IT IS
# ----------
# Everything before this was plumbing that delivered to nobody: a message arrives,
# is journaled, a flag rises, a receipt goes back — and no agent ever sees it. This
# is the step that puts the message in front of the agent and then CHECKS that it
# landed.
#
#   arrival record -> agent reachable? -> prompt free? -> queue -> INJECT
#       -> VERIFY it was consumed -> L3 stage=read (evidence=idle-gated-inject)
#
# It is the first truthful caller of L3. That rung was built yesterday and had no
# honest caller: the wake consumer posted evidence=wake-consumer, which asserted
# "this reached a prompt" when all that happened was "a script saw a flag". That
# evidence kind has been removed; this script earns the claim instead.
#
# THE FIVE OUTCOMES, AND WHY THEY ARE FIVE
# ----------------------------------------
# Collapsing any two of these produces a specific known failure:
#
#   0  INJECTED + VERIFIED   the only case that posts L3
#   1  nothing to do         queue empty / no new arrival. Not an error.
#   2  tooling               could not look. NEVER a delivery verdict.
#   3  AGENT NOT RUNNING     the session is gone. Distinct from busy ON PURPOSE:
#                            the operator asked for exactly this ("I want to
#                            inject, but I check if my agent is running, I see
#                            it's not running"). Folded into BUSY it would retry
#                            forever against a session that no longer exists.
#   4  prompt BUSY / UNKNOWN deferred, will retry. Not a failure.
#   5  INJECTED, NOT VERIFIED the loud one. The text went in and we could NOT
#                            confirm it was consumed, so it may be sitting
#                            unsubmitted in a composer. Posts NO L3. Silence here
#                            would be the T-2396 failure dressed as success.
#
# WHY VERIFICATION IS NOT OPTIONAL
# --------------------------------
# `termlink inject` RETURNS BEFORE SUBMISSION. On a busy or manual-accept session
# the text lands in the composer unsubmitted and is discarded on the next
# `claude --continue` — durably written, never read (T-2396, proven live). So
# "I injected it" is not evidence anybody read it. After injecting, this script
# re-reads the PTY and requires the composer to have cleared. If it cannot
# confirm, it exits 5 and posts nothing.
#
# FAIL-SAFE BIAS, INHERITED DELIBERATELY
# --------------------------------------
# The classifier in scripts/lib/pty-state.sh returns READY only on a POSITIVE idle
# marker; ambiguity is UNKNOWN. This script preserves that: UNKNOWN defers, never
# injects. A wrong READY is the blind inject this rung exists to kill.
set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TERMLINK="${TERMLINK:-termlink}"

# ONE copy of the classifier, shared with be-reachable-pushwaker.sh.
# shellcheck source=lib/pty-state.sh
. "$HERE/lib/pty-state.sh"

AGENT_ID=""
SESSION=""
NOTIFY_DIR="${NOTIFY_DIR:-$HOME/.termlink/notify}"
JOURNAL="${TERMLINK_JOURNAL_PATH:-$HOME/.termlink/journals/journal.sqlite}"
AS_IDENTITY=""
TOPIC_FILTER=""
DRY=0
QUIET=0
VERIFY_TIMEOUT="${INJECTOR_VERIFY_TIMEOUT:-20}"

usage() {
    cat <<'USAGE'
Usage: notify-injector.sh --agent-id ID --session TLID [options]

Takes one queued message, injects it into the agent's prompt when the prompt is
free, verifies it was consumed, and posts L3 stage=read.

Required:
  --agent-id ID       agent whose flag + mailbox to serve
  --session TLID      the agent's termlink session (its PTY)

Options:
  --as-identity ID    TERMLINK_AGENT_ID to post L3 as. Declared, never derived
  --topic-filter T    only consider messages on this topic
  --notify-dir DIR    flag directory (default ~/.termlink/notify)
  --journal PATH      queue source (default ~/.termlink/journals/journal.sqlite)
  --verify-timeout N  seconds to confirm the injection landed (default 20)
  --dry-run           decide and print; inject nothing, post nothing
  --quiet             suppress progress output
  --help              this text

Exit: 0 injected+verified · 1 nothing to do · 2 tooling · 3 agent not running
      · 4 prompt busy/unknown (deferred) · 5 injected but NOT verified
USAGE
}

while [ $# -gt 0 ]; do
    case "$1" in
        --agent-id)       AGENT_ID="${2:-}"; shift 2 ;;
        --session)        SESSION="${2:-}"; shift 2 ;;
        --as-identity)    AS_IDENTITY="${2:-}"; shift 2 ;;
        --topic-filter)   TOPIC_FILTER="${2:-}"; shift 2 ;;
        --notify-dir)     NOTIFY_DIR="${2:-}"; shift 2 ;;
        --journal)        JOURNAL="${2:-}"; shift 2 ;;
        --verify-timeout) VERIFY_TIMEOUT="${2:-}"; shift 2 ;;
        --dry-run)        DRY=1; shift ;;
        --quiet)          QUIET=1; shift ;;
        --help|-h)        usage; exit 0 ;;
        *) echo "notify-injector: unknown argument: $1" >&2; usage >&2; exit 2 ;;
    esac
done

[ -n "$AGENT_ID" ] || { echo "notify-injector: --agent-id is required" >&2; exit 2; }
[ -n "$SESSION" ]  || { echo "notify-injector: --session is required (the agent's PTY)" >&2; exit 2; }
command -v sqlite3 >/dev/null 2>&1 || { echo "notify-injector: sqlite3 not found" >&2; exit 2; }

log()  { [ "$QUIET" -eq 1 ] || echo "notify-injector: $*"; }
loud() { echo "notify-injector: $*"; }

[ -n "$AS_IDENTITY" ] && export TERMLINK_AGENT_ID="$AS_IDENTITY"

FLAG="$NOTIFY_DIR/$AGENT_ID.flag"
SEEN="$NOTIFY_DIR/.$AGENT_ID.injected-seen"

kv() { grep -E "^$2=" "$1" 2>/dev/null | head -1 | cut -d= -f2-; }

# ---- 1. is there anything to do? ------------------------------------------
# Keyed on the ARRIVAL RECORD (last_mail_ts), not on `pending`. pending is cleared
# by the L2 acker in the same cycle it detects mail, so it is a value the acker
# races to zero; last_mail_ts survives the ack (T-3068).
[ -r "$FLAG" ] || { log "no flag at $FLAG — nothing to do"; exit 1; }
mail_ts="$(kv "$FLAG" last_mail_ts | tr -dc '0-9')"; : "${mail_ts:=0}"
mail_topic="$(kv "$FLAG" last_mail_topic)"
seen_ts=0
[ -r "$SEEN" ] && seen_ts="$(tr -dc '0-9' < "$SEEN")"; : "${seen_ts:=0}"

if [ "$mail_ts" -le "$seen_ts" ] 2>/dev/null || [ -z "$mail_topic" ]; then
    log "no new arrival (last_mail_ts=$mail_ts, already handled through $seen_ts)"
    exit 1
fi
if [ -n "$TOPIC_FILTER" ] && [ "$mail_topic" != "$TOPIC_FILTER" ]; then
    log "arrival on $mail_topic does not match --topic-filter"
    exit 1
fi

# ---- 2. is the agent even running? ----------------------------------------
# Deliberately BEFORE the prompt check and with its own exit code. An absent
# session and a busy prompt want opposite responses: retry-later vs
# stop-and-tell-someone.
if [ -n "${INJECTOR_TEST_SESSION_STATE:-}" ]; then
    sess_state="$INJECTOR_TEST_SESSION_STATE"
else
    if "$TERMLINK" list 2>/dev/null | grep -qE "^${SESSION}[[:space:]]"; then
        sess_state="present"
    else
        sess_state="absent"
    fi
fi
if [ "$sess_state" != "present" ]; then
    loud "AGENT NOT RUNNING: session '$SESSION' is not registered. Mail is waiting on $mail_topic and nothing can consume it until the agent is back."
    exit 3
fi

# ---- 3. is the prompt free? -----------------------------------------------
if [ -n "${INJECTOR_TEST_PTY_STATE:-}" ]; then
    state="$INJECTOR_TEST_PTY_STATE"
else
    state="$(pushwaker_probe_pty "$SESSION")"
fi
case "$state" in
    READY) : ;;
    BUSY)    log "prompt BUSY — deferring (the agent is mid-turn)"; exit 4 ;;
    *)       log "prompt UNKNOWN — deferring. Ambiguity never resolves to READY: a wrong READY is a blind inject."; exit 4 ;;
esac

# ---- 4. read the queue ----------------------------------------------------
# FLAT FIFO by (ts, offset), per operator: priority is a later slice (T-3071).
# Content only — meta envelopes (receipts/reactions/...) are not work.
sql_topic="${mail_topic//\'/\'\'}"
row="$(sqlite3 -separator '|' "$JOURNAL" \
    "SELECT offset, sender_id, substr(payload,1,4000) FROM messages
      WHERE topic='$sql_topic'
        AND msg_type NOT IN ('receipt','reaction','redaction','edit','topic_metadata')
      ORDER BY ts ASC, offset ASC
      LIMIT 1;" 2>/dev/null)"

if [ -z "$row" ]; then
    # The flag says mail arrived but the journal has no content row for it. That is
    # a real inconsistency, not a quiet no-op: the journal mirror may have failed
    # while the ack succeeded.
    loud "QUEUE EMPTY for $mail_topic though the flag reports an arrival at $mail_ts — journal mirror may have failed. Not injecting."
    exit 2
fi

offset="${row%%|*}"; rest="${row#*|}"
sender="${rest%%|*}"; body="${rest#*|}"
log "queued: topic=$mail_topic offset=$offset from=${sender:0:8}"

# ---- 5. inject ------------------------------------------------------------
# The injected text names its provenance so the agent (and a human reading the
# transcript) can tell an injected peer message from something the operator typed.
inject_text="[peer-message ${sender:0:8} @${mail_topic} #${offset}] ${body}"

if [ "$DRY" -eq 1 ]; then
    echo "notify-injector: [DRY-RUN] prompt READY; would inject into $SESSION:"
    echo "  $inject_text"
    echo "notify-injector: [DRY-RUN] would then verify, then post L3 evidence=idle-gated-inject up_to=$offset"
    exit 0
fi

if [ -n "${INJECTOR_TEST_INJECT_RC:-}" ]; then
    inject_rc="$INJECTOR_TEST_INJECT_RC"
else
    "$TERMLINK" inject "$SESSION" "$inject_text" --enter >/dev/null 2>&1
    inject_rc=$?
fi
if [ "$inject_rc" -ne 0 ]; then
    loud "inject FAILED (rc=$inject_rc) — posting no L3"
    exit 5
fi

# ---- 6. VERIFY it was actually consumed -----------------------------------
# This is the step that separates this script from theatre. `inject` returned,
# which means the keystrokes were delivered to the PTY — NOT that the REPL
# accepted and submitted them. A session that was READY a moment ago should now
# be BUSY (it took the turn) or back to READY with the composer clear.
#
# The signal we require is a TRANSITION: the prompt must stop looking idle-with-
# our-text-pending. We re-probe until it reports BUSY (turn started — strongest
# evidence) or the timeout expires.
verified=0
if [ -n "${INJECTOR_TEST_VERIFY_STATE:-}" ]; then
    [ "$INJECTOR_TEST_VERIFY_STATE" = "BUSY" ] && verified=1
else
    deadline=$(( $(date +%s) + VERIFY_TIMEOUT ))
    while [ "$(date +%s)" -lt "$deadline" ]; do
        post="$(pushwaker_probe_pty "$SESSION")"
        if [ "$post" = "BUSY" ]; then verified=1; break; fi
        sleep 2
    done
fi

if [ "$verified" -ne 1 ]; then
    loud "INJECTED BUT NOT VERIFIED on $SESSION (offset=$offset). The text was delivered to the PTY but the prompt never started a turn within ${VERIFY_TIMEOUT}s — it may be sitting UNSUBMITTED in the composer and will be discarded on the next --continue (T-2396). Posting NO L3: the sender must not be told this was read."
    exit 5
fi

# ---- 7. L3, earned --------------------------------------------------------
# NOTIFY_DIR is passed THROUGH. notify-ack-read.sh keeps its idempotency guard
# under $NOTIFY_DIR, and without this it defaulted to ~/.termlink/notify while the
# injector was working somewhere else — so under --notify-dir the guard landed in
# the wrong directory, and a stale guard there made the ack a silent no-op that
# still returned 0. Found by a fixture asserting "exactly one L3 posted" and
# getting zero while the injector cheerfully logged "L3 posted".
if NOTIFY_DIR="$NOTIFY_DIR" bash "$HERE/notify-ack-read.sh" --topic "$mail_topic" --up-to "$offset" \
        --evidence idle-gated-inject --agent-id "$AGENT_ID" --quiet; then
    printf '%s\n' "$mail_ts" > "$SEEN.tmp" 2>/dev/null && mv -f "$SEEN.tmp" "$SEEN" 2>/dev/null || true
    log "INJECTED + VERIFIED: offset=$offset on $mail_topic; L3 posted (evidence=idle-gated-inject)"
    exit 0
fi

loud "injected and verified, but the L3 receipt was REJECTED by the hub. The agent has the message; the sender has not been told."
exit 5
