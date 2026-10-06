#!/usr/bin/env bash
# T-2294 (arc-003 reliable-comms, V3a) — deterministic notify sidecar.
#
# The no-LLM half of the §5 deterministic-sidecar wake (AEF ADR §5;
# docs/architecture/parallel-execution-substrate.md). A turn-based agent cannot
# afford a preemptive mid-turn PTY injection (T-1800 doorbell, T-2285 miss-gap:
# keystrokes injected mid-turn are dropped → recipient never wakes). Instead this
# sidecar does a remote-read of the agent's mail and materializes it into a
# **local flag file** plus a **fresh heartbeat timestamp**; the agent then polls
# that local flag cooperatively at its own yield points (see notify-check.sh).
#
# Determinism comes from the timestamp, NOT the transport: the *absence* of a
# fresh heartbeat delta is itself the signal ("listener is deaf"). The flag is a
# file, never a keystroke — so a missed beat is self-detected by the consumer
# rather than silently lost.
#
# Why a FILE and not `termlink kv`: kv is session-scoped + in-memory + hub-
# mediated (crates/termlink-session/src/handler.rs — per-session HashMap, lost on
# session exit, requires the hub). The self-check must remain trustworthy
# *precisely when the hub is down* (that is exactly when an agent most needs to
# learn its listener went deaf), so the flag lives on the local filesystem,
# mirroring the offline-queue path discipline (~/.termlink/...).
#
# Homes (all shipped): the local flag dir (this script), agent-presence
# (liveness), and the mail being detected: the dm:<self>:<peer> topics plus the
# inbox:<circuit>/<self-project> mailbox (T-3203).
#
# Lifecycle mirrors listener-heartbeat.sh: loop posting every --interval seconds
# until SIGINT/SIGTERM; --once for a single probe-and-write cycle.
#
# Exit codes:
#   0  — normal exit (clean signal received OR --once success)
#   2  — usage error (missing required flag, unknown arg)
#   3  — runtime error (cannot resolve self identity in real-probe mode)
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# T-3203: which project's `inbox:` mailbox is OURS. A declared constant with an env
# override, deliberately NOT `basename "$PROJECT_ROOT"` — a path-derived slug is wrong
# inside a git worktree (T-2815), which is why FW_PICKUP_SELF_PROJECT is a constant too
# (T-2816). journal-mirror.sh reads the SAME variable: the ingest half and the detect
# half of the rail must never disagree about who we are, or one will mirror mail the
# other refuses to notice.
SELF_PROJECT="${FW_SIDECAR_SELF_PROJECT:-010-termlink}"

die_usage() {
    echo "notify-sidecar: $*" >&2
    echo "Try --help for usage." >&2
    exit 2
}

usage() {
    cat <<'EOF'
Usage: notify-sidecar.sh --agent-id NAME [OPTIONS]

Deterministic notify sidecar (T-2294, arc-003 reliable-comms V3a). Polls this
agent's mail on the hub and materializes it into a LOCAL flag file + a fresh
heartbeat timestamp, so a turn-based agent can wake cooperatively at its yield
points (notify-check.sh) without preemptive PTY injection.

Required:
  --agent-id NAME      Logical agent name (e.g. "claude-code-A"). Also pins the
                       per-agent crypto identity (TERMLINK_AGENT_ID, T-2292) so
                       mail detection scopes to THIS agent's dm: topics. The
                       inbox: half is scoped by PROJECT, not by agent — see
                       FW_SIDECAR_SELF_PROJECT below.

Optional:
  --self-fp FP         Identity fingerprint used to find dm:<self>:* topics.
                       Does NOT select inbox: topics — those are addressed
                       inbox:<circuit>/<project>, so they are matched by project
                       name via FW_SIDECAR_SELF_PROJECT (default 010-termlink),
                       the same constant journal-mirror.sh reads.
                       Default: resolved via `termlink whoami --json`, then the
                       be-reachable.state file. Pass explicitly if neither
                       resolves (e.g. headless sidecar with no live session).
  --notify-dir DIR     Flag/heartbeat directory (default: ~/.termlink/notify).
  --interval N         Probe period in seconds (default: 15; min: 5).
  --hub addr           Target hub (default: local).
  --include-broadcast  Also count unread agent-chat-arc broadcasts as mail.
  --auto-confirm       arc-003 V6-S3: on each cycle, for every dm:<self>:* topic
                       with unread mail, (a) mirror the topic into the S1 journal
                       (journal-mirror.sh) and (b) auto-post a mechanism-A receipt
                       (--msg-type receipt --metadata stage=delivered --metadata
                       up_to=<latest_offset>). This makes the direct path
                       store-and-forward: the recipient confirms L2-delivered with
                       NO LLM turn, and the journal+receipt survive a restart. A
                       durable per-topic offset guard (under --notify-dir) prevents
                       ack spam — a receipt is posted only when the offset advances.
                       Default OFF: without this flag the sidecar behaves exactly as
                       V3a (no post, no journal write). The doorbell-optional-on-
                       direct sender change is S4, not this slice.
  --as-identity NAME   T-3346: the receipt-signing key is stored under a different
                       name than --agent-id (e.g. the mailbox claude-termlink-alt
                       belongs to the per-agent key identities/claude-termlink.key).
                       The resolver also tries <identities>/NAME.key, and still
                       signs only if that key's fingerprint equals self-fp. Mirrors
                       notify-wake-consumer.sh --as-identity.
  --no-project-inbox   T-3370: watch only this agent's dm:<self>:* topics, never the
                       project mailbox inbox:<circuit>/<project>. Used once the
                       project inbox has its own receiver (the AEF sidecar, OD-9):
                       two receivers on one inbox are forbidden, because each one
                       acks and wakes independently and mail lands in the wrong
                       session (the 2026-10-06 misrouting).
  --once               Probe once, write flag+heartbeat, exit 0.
  --json               Emit one JSON status line per cycle.
  -h, --help           Print this help and exit 0.

Files written each cycle (NOTIFY_DIR/<agent_id>.*):
  <agent_id>.heartbeat   epoch-ms of this cycle (proof-of-life; ALWAYS written)
  <agent_id>.flag        key=value lines: pending=<N> latest_topic=<T> ts=<ms>

Test hook (hub-independent, mirrors TERMLINK_GROWTH_TEST_JSON convention):
  TERMLINK_NOTIFY_TEST_UNREAD=<N>          force the unread count to N
  TERMLINK_NOTIFY_TEST_LATEST_TOPIC=<T>    force the latest-topic label
  TERMLINK_NOTIFY_TEST_TOPICS=<t1 t2 ...>  scope the dm-topic enumeration to an
                                           explicit list (isolates --auto-confirm
                                           tests from production dm: topics)

Exit codes: 0 normal/--once  2 usage error  3 runtime (no self identity)

Consumer: notify-check.sh reads these files at the agent's yield points and
returns exit 10 (mail) / 3 (deaf) / 0 (clear). See
docs/operations/deterministic-notify-sidecar.md.
EOF
}

agent_id=""
self_fp=""
notify_dir="${TERMLINK_NOTIFY_DIR:-$HOME/.termlink/notify}"
interval=15
hub=""
include_broadcast=0
auto_confirm=0
as_identity=""
no_project_inbox=0
once=0
json=0

while [ $# -gt 0 ]; do
    case "$1" in
        --agent-id)         agent_id="${2:-}"; shift 2 ;;
        --self-fp)          self_fp="${2:-}"; shift 2 ;;
        --notify-dir)       notify_dir="${2:-}"; shift 2 ;;
        --interval)         interval="${2:-}"; shift 2 ;;
        --hub)              hub="${2:-}"; shift 2 ;;
        --include-broadcast) include_broadcast=1; shift ;;
        --auto-confirm)     auto_confirm=1; shift ;;
        --as-identity)      as_identity="${2:-}"; shift 2 ;;
        --no-project-inbox) no_project_inbox=1; shift ;;
        --once)             once=1; shift ;;
        --json)             json=1; shift ;;
        -h|--help)          usage; exit 0 ;;
        *)                  die_usage "unknown arg: $1" ;;
    esac
done

[ -n "$agent_id" ] || die_usage "missing required --agent-id"
[ -n "$notify_dir" ] || die_usage "--notify-dir must not be empty"

case "$interval" in
    ''|*[!0-9]*) die_usage "--interval must be a positive integer (got: $interval)" ;;
esac
[ "$interval" -ge 5 ] || die_usage "--interval must be >= 5 seconds (got: $interval)"

# T-2292: pin per-agent identity so any hub call below signs as THIS agent.
export TERMLINK_AGENT_ID="$agent_id"

hub_args=()
[ -n "$hub" ] && hub_args+=(--hub "$hub")

now_ms() { date +%s%3N 2>/dev/null || echo "$(( $(date +%s) * 1000 ))"; }

# Resolve self fingerprint (only needed for the REAL probe path; the test hook
# short-circuits before this is consulted).
resolve_self_fp() {
    [ -n "$self_fp" ] && { echo "$self_fp"; return 0; }
    local fp
    fp="$("$TERMLINK" whoami --json 2>/dev/null | jq -r '.session.identity_fingerprint // empty' 2>/dev/null)"
    [ -n "$fp" ] && { echo "$fp"; return 0; }
    # Fallback: be-reachable.state (PL-195 sender-resolution chain).
    local state="${TERMLINK_BE_REACHABLE_STATE:-$HOME/.termlink/be-reachable.state}"
    if [ -f "$state" ]; then
        fp="$(grep -E '^(self_fp|fingerprint)=' "$state" 2>/dev/null | head -1 | cut -d= -f2-)"
        [ -n "$fp" ] && { echo "$fp"; return 0; }
    fi
    return 1
}

# T-3065 — resolve WHICH local identity the auto-confirm receipt must be signed with.
#
# The receipt has to be signed by the key whose fingerprint EQUALS the dm-party fp we
# are acking for, because that is the identity a sender looks for: `--await-ack` derives
# the recipient from the dm topic name (derive_dm_recipient) and polls channel.receipts
# for THAT sender_id. Relabelling with `channel post --sender-id` cannot work and must
# not be attempted — hub channel.rs:777 (T-1427) refuses a claimed sender_id that does
# not match the fingerprint derived from the signing pubkey. The signing KEY is what has
# to change, and the hub refusing the shortcut is correct behaviour, not an obstacle.
#
# Why this is not simply "stop exporting TERMLINK_AGENT_ID": that export is T-2292
# per-agent identity and is right for every other call. Measured on this host, the three
# declared agents disagree about which key carries their declared self-fp:
#
#   claude-termlink          declares d1993c2c3ec44c94  resolves 6738c073bbcc587a  MISMATCH
#   framework-agent-systemd  declares 3bba15e681b3a078  resolves 3bba15e681b3a078  ok
#   claude-termlink-alt      declares 6738c073bbcc587a  resolves 84c04584abd5cf88  MISMATCH
#
# So no single blanket rule is correct: one agent needs the per-agent key, one needs the
# host default, and one matches nothing at all. We probe instead of assuming, and the
# agent that matches nothing is REFUSED rather than silently mis-signed — see below.
#
# Echoes the env assignment to prefix the receipt post with, or "NONE".
# Resolution order follows the documented signing precedence
# (TERMLINK_IDENTITY_FILE > TERMLINK_AGENT_ID > TERMLINK_IDENTITY_DIR > host default).
_resolve_receipt_identity() {
    local want="$1" got

    # (a) the per-agent identity already exported — the common case when correct.
    got="$("$TERMLINK" agent identity --resolve --json 2>/dev/null | jq -r '.fingerprint // empty' 2>/dev/null)"
    [ "$got" = "$want" ] && { echo "KEEP"; return 0; }

    # (b) the shared host default (TERMLINK_AGENT_ID unset).
    got="$(env -u TERMLINK_AGENT_ID -u TERMLINK_IDENTITY_FILE \
            "$TERMLINK" agent identity --resolve --json 2>/dev/null | jq -r '.fingerprint // empty' 2>/dev/null)"
    [ "$got" = "$want" ] && { echo "HOST_DEFAULT"; return 0; }

    # (c) an explicit per-agent key file, if one exists under the identities dir.
    local keyfile="${TERMLINK_IDENTITY_DIR:-$HOME/.termlink/identities}/$agent_id.key"
    if [ -r "$keyfile" ]; then
        got="$(TERMLINK_IDENTITY_FILE="$keyfile" \
                "$TERMLINK" agent identity --resolve --json 2>/dev/null | jq -r '.fingerprint // empty' 2>/dev/null)"
        [ "$got" = "$want" ] && { echo "FILE:$keyfile"; return 0; }
    fi

    # (d) T-3346: the key stored under another agent's name (--as-identity). Same
    # fingerprint check: a name match alone never earns a signature.
    if [ -n "${as_identity:-}" ]; then
        keyfile="${TERMLINK_IDENTITY_DIR:-$HOME/.termlink/identities}/$as_identity.key"
        if [ -r "$keyfile" ]; then
            got="$(TERMLINK_IDENTITY_FILE="$keyfile" \
                    "$TERMLINK" agent identity --resolve --json 2>/dev/null | jq -r '.fingerprint // empty' 2>/dev/null)"
            [ "$got" = "$want" ] && { echo "FILE:$keyfile"; return 0; }
        fi
    fi

    echo "NONE"
    return 1
}

# arc-003 V6-S3: recipient-side auto-confirm for one dm: topic. Mirrors the topic
# into the S1 journal and posts a mechanism-A `stage=delivered` receipt acking the
# topic's latest offset — no LLM turn. Idempotent via a durable per-topic offset
# guard under notify_dir: a receipt is posted only when the latest offset advances,
# so a steady unread state does not spam receipts (and the guard survives restart).
# Best-effort: any failure is non-fatal (the heartbeat/flag cycle still completes).
_auto_confirm_topic() {
    local topic="$1" fp="$2" foreign_min="${3:--}" first_unread="${4:-0}" latest_off guard prev
    # Watermark to ack = the latest CONTENT offset on the topic. `channel unread
    # --json`'s latest_offset is unreliable (null even when unread>0), so read the
    # topic tail directly and take the max offset over content envelopes only —
    # excluding meta types (receipt/reaction/redaction/edit/topic_metadata) exactly
    # as `channel unread` does. Excluding receipts is load-bearing for the offset
    # guard: our OWN posted receipt must not bump the watermark, else every re-run
    # re-acks (the offset-guard failure mode).
    latest_off="$("$TERMLINK" channel subscribe "$topic" "${hub_args[@]}" \
                    --cursor 0 --limit 1000 --json 2>/dev/null \
        | jq -s '[ .[]
                   | select(.msg_type != "receipt" and .msg_type != "reaction"
                            and .msg_type != "redaction" and .msg_type != "edit"
                            and .msg_type != "topic_metadata")
                   | .offset ] | (max // empty)' 2>/dev/null)"
    case "$latest_off" in ''|*[!0-9]*) return 0 ;; esac   # no content to ack

    # T-3325: the receipt is a WATERMARK on an identity co-resident agents share, so
    # acking past a message addressed to one of them marks it read for them too. Ack
    # only up to the message before the first one addressed elsewhere.
    case "$foreign_min" in
        ''|-|*[!0-9]*) ;;
        *) [ $((foreign_min - 1)) -lt "$latest_off" ] && latest_off=$((foreign_min - 1))
           # Capped below the first unread message: nothing of ours to ack.
           case "$first_unread" in ''|*[!0-9]*) first_unread=0 ;; esac
           [ "$latest_off" -ge "$first_unread" ] || return 0 ;;
    esac

    # Resolve the receipt-signing identity BEFORE the guard is computed (T-3065).
    # Order matters: the guard name embeds the identity, so resolving afterwards would
    # let the OLD guard suppress the corrective re-ack and the fix would silently do
    # nothing on exactly the topics that need it most.
    #
    # Cached per process: it costs a subprocess and the answer cannot change while we run.
    if [ -z "${_receipt_identity:-}" ]; then
        _receipt_identity="$(_resolve_receipt_identity "$fp")"
        if [ "$_receipt_identity" = "NONE" ]; then
            # LOUD REFUSAL (T-3065 AC3). Posting anyway is the pre-fix behaviour and is
            # worse than not posting: the receipt satisfies nobody, yet the offset guard
            # records the topic as acked, so the ack is never retried. A sender waiting
            # on it dead-letters while every local surface looks healthy.
            # Logged once per (agent, fp), not once per cycle (T-3346): this function
            # runs in a pipeline subshell, so the cache above does not survive a cycle,
            # and 32,205 identical lines buried the one that mattered.
            _refusal_marker="$notify_dir/.$agent_id.refusal"
            if [ "$(cat "$_refusal_marker" 2>/dev/null)" != "$fp" ]; then
                echo "notify-sidecar: REFUSING to auto-confirm — no local identity resolves to self-fp '$fp'" >&2
                echo "notify-sidecar:   a receipt signed by any other key satisfies nobody (T-1427 + derive_dm_recipient)." >&2
                echo "notify-sidecar:   fix the declared --self-fp for '$agent_id', install the matching key, or pass --as-identity <key-name>." >&2
                echo "notify-sidecar:   (logged once; recorded in $_refusal_marker — delete it to log again)" >&2
                printf '%s\n' "$fp" > "$_refusal_marker" 2>/dev/null || true
            fi
        else
            rm -f "$notify_dir/.$agent_id.refusal" 2>/dev/null || true
        fi
    fi
    [ "$_receipt_identity" = "NONE" ] && return 0

    # Per-topic guard file, keyed by signing identity (T-3065 AC4). Receipts already
    # written under the WRONG key left guards at offsets that would suppress the
    # corrective re-ack forever. Embedding the identity re-arms exactly the topics whose
    # signing identity changed and leaves correctly-acked ones untouched — no manual
    # guard deletion, and no re-ack storm on topics that were already right.
    guard="$notify_dir/.$agent_id.$(printf '%s' "$topic" | tr -c 'A-Za-z0-9' '_')"
    guard="$guard.$(printf '%s' "$_receipt_identity" | tr -c 'A-Za-z0-9' '_').acked"
    prev=-1
    [ -f "$guard" ] && prev="$(cat "$guard" 2>/dev/null)"
    case "$prev" in ''|*[!0-9-]*) prev=-1 ;; esac
    [ "$latest_off" -le "$prev" ] && return 0             # already acked this offset (or newer)

    # (a) journal the topic (S1 read-side mirror) — best-effort, inherits
    #     TERMLINK_JOURNAL_PATH so a test can isolate the store.
    bash "$HERE/journal-mirror.sh" --topic "$topic" "${hub_args[@]}" >/dev/null 2>&1 || true

    # (b) post the mechanism-A stage=delivered receipt (the L2-delivered producer),
    #     signed as the DM PARTY so --await-ack can actually find it (T-3065).
    _post_receipt() {
        case "$_receipt_identity" in
            KEEP)         "$TERMLINK" channel post "$topic" "${hub_args[@]}" --msg-type receipt \
                              --metadata stage=delivered --metadata up_to="$latest_off" --json >/dev/null 2>&1 ;;
            HOST_DEFAULT) env -u TERMLINK_AGENT_ID -u TERMLINK_IDENTITY_FILE \
                              "$TERMLINK" channel post "$topic" "${hub_args[@]}" --msg-type receipt \
                              --metadata stage=delivered --metadata up_to="$latest_off" --json >/dev/null 2>&1 ;;
            FILE:*)       TERMLINK_IDENTITY_FILE="${_receipt_identity#FILE:}" \
                              "$TERMLINK" channel post "$topic" "${hub_args[@]}" --msg-type receipt \
                              --metadata stage=delivered --metadata up_to="$latest_off" --json >/dev/null 2>&1 ;;
            *)            return 1 ;;
        esac
    }
    if _post_receipt; then
        printf '%s\n' "$latest_off" > "$guard.tmp" 2>/dev/null \
            && mv -f "$guard.tmp" "$guard" 2>/dev/null || true
    fi
}

# T-3325 — five-level circuit addressing (operator ruling Q1 = C). Each unread
# message is classified by scripts/lib/circuit.py against OUR circuit; only mail
# for us (or unaddressed, or falling back to us) counts and wakes, and auto-confirm
# never acks past mail addressed to a co-resident agent that shares our identity —
# the ack watermark is shared, so acking it would mark THEIR mail read for them.
#
# Our circuit, resolved once per process. Overridable for tests and odd hosts:
#   FW_SIDECAR_SELF_HOST (default `hostname -f`), FW_SIDECAR_SELF_HUB (default the
#   local hub's TLS fingerprint). An empty level is simply not compared.
CIRCUIT_PY="$HERE/lib/circuit.py"
_self_host="" _self_hub="" _self_circuit_resolved=0
_resolve_self_circuit() {
    [ "$_self_circuit_resolved" -eq 1 ] && return 0
    _self_circuit_resolved=1
    _self_host="${FW_SIDECAR_SELF_HOST-$(hostname -f 2>/dev/null || hostname 2>/dev/null)}"
    if [ -n "${FW_SIDECAR_SELF_HUB+x}" ]; then
        _self_hub="$FW_SIDECAR_SELF_HUB"
    else
        _self_hub="$("$TERMLINK" hub fingerprint 2>/dev/null | head -1 | sed -n 's/^sha256:\([0-9a-f]\{16\}\).*/\1/p')"
    fi
}

# Agents LIVE on this host = a sidecar heartbeat (epoch-ms) in our notify_dir no older
# than FW_SIDECAR_LIVE_WINDOW_MS (default 120000 = 8 missed 15s cycles). Used only for
# the fallback ladder: mail naming a co-resident agent that is NOT live falls back to
# the project level and wakes us, instead of waiting for an agent that is not there.
_live_agents() {
    local now window f a hb out=""
    now="$(now_ms)"; window="${FW_SIDECAR_LIVE_WINDOW_MS:-120000}"
    for f in "$notify_dir"/*.heartbeat; do
        [ -f "$f" ] || continue
        a="$(basename "$f" .heartbeat)"
        [ "$a" = "$agent_id" ] && continue
        hb="$(tr -dc '0-9' < "$f" 2>/dev/null)"
        [ -n "$hb" ] || continue
        [ $((now - hb)) -le "$window" ] && out="$out${out:+,}$a"
    done
    # Plus agents LIVE on the presence rail (agent-listeners.sh): an agent can be
    # reachable without running a sidecar here (055 at the time of writing), and
    # mail addressed to it must not fall back to us. Seam FW_SIDECAR_PRESENCE_AGENTS
    # (comma list, may be empty) replaces the lookup for hermetic tests.
    local pres
    if [ -n "${FW_SIDECAR_PRESENCE_AGENTS+x}" ]; then
        pres="$FW_SIDECAR_PRESENCE_AGENTS"
    else
        pres="$(timeout 15 bash "$HERE/agent-listeners.sh" --json 2>/dev/null \
            | jq -r '(.listeners // .)[]? | select(.status=="LIVE") | .agent_id // empty' 2>/dev/null \
            | grep -vxF "$agent_id" | sort -u | paste -sd, -)"
    fi
    [ -n "$pres" ] && out="$out${out:+,}$pres"
    printf '%s' "$out"
}

# Classify one topic's unread envelopes. Echoes "<wake> <foreign_min|-> <newest_wake|->"
# or returns non-zero when classification is unavailable (no first_unread, fetch or
# parse failure); the caller then falls back to the raw unread count, so a broken
# classifier degrades to the old behaviour (spurious wake), never to a missed one.
_classify_topic() {
    local topic="$1" first="$2" n="$3" limit res
    case "$first" in ''|*[!0-9]*) return 1 ;; esac
    [ -r "$CIRCUIT_PY" ] || return 1
    _resolve_self_circuit
    limit=$(( n * 3 + 50 )); [ "$limit" -gt 1000 ] && limit=1000
    res="$("$TERMLINK" channel subscribe "$topic" "${hub_args[@]}" \
                --cursor "$first" --limit "$limit" --json 2>/dev/null \
        | python3 "$CIRCUIT_PY" classify --first-unread "$first" \
            --self-host "$_self_host" --self-hub "$_self_hub" \
            --self-project "$SELF_PROJECT" --self-agent "$agent_id" \
            --live "$(_live_agents)" 2>/dev/null)" || return 1
    # Every unread message must have been classified. A short fetch (hub hiccup, limit)
    # would otherwise report "0 to wake" for mail it never saw — a silent miss.
    local seen
    seen="$(printf '%s' "$res" | jq -r '(.mine + .unaddressed + .fallback + .foreign)' 2>/dev/null)"
    case "$seen" in ''|*[!0-9]*) return 1 ;; esac
    [ "$seen" -ge "$n" ] || return 1
    printf '%s' "$res" | jq -r '"\(.wake) \(.foreign_min_offset // "-") \(.newest_wake_offset // "-")"' 2>/dev/null \
        | grep -E '^[0-9]+ ' || return 1
}

# Probe unread mail. Echoes "<count>\t<latest_topic>\t<sig>". Honors the test hook for
# hub-independent unit testing; otherwise sums unread across dm:<self>:* topics.
probe_mail() {
    if [ -n "${TERMLINK_NOTIFY_TEST_UNREAD:-}" ]; then
        printf '%s\t%s\t?\n' "${TERMLINK_NOTIFY_TEST_UNREAD}" "${TERMLINK_NOTIFY_TEST_LATEST_TOPIC:-}"
        return 0
    fi

    local fp
    fp="$(resolve_self_fp)" || return 3

    # sig: per-topic newest offset of mail that wakes us. write_cycle advances the
    # arrival record (last_mail_ts) only when it changes, so mail for us that sits
    # behind a co-resident's unacked message does not re-wake us every cycle. A "?"
    # entry (classifier unavailable) keeps the old advance-every-cycle behaviour.
    local total=0 latest="" sig=""
    # dm:<sorted_a>:<sorted_b> — self appears in either slot.
    local topics
    if [ -n "${TERMLINK_NOTIFY_TEST_TOPICS:-}" ]; then
        # Test hook: use an explicit topic list (whitespace/newline separated)
        # instead of live enumeration — scopes a test to its own topic so the real
        # enumeration (which would match every dm: topic this fp participates in)
        # does not touch production conversations. Mirrors TERMLINK_NOTIFY_TEST_UNREAD.
        topics="$(printf '%s\n' $TERMLINK_NOTIFY_TEST_TOPICS)"
    else
        # `channel list --json` returns {"topics":[{"name":...}]} (object), but tolerate
        # a bare array shape too: (.topics // .) handles both. A naive `.[]?.name`
        # errors on the object form and silently yields zero topics (the V3a probe bug
        # found in the AC1 live proof).
        local _dm_topics _inbox_topics
        _dm_topics="$("$TERMLINK" channel list "${hub_args[@]}" --prefix "dm:" --json 2>/dev/null \
            | jq -r --arg fp "$fp" '(.topics // .)[]?.name // empty | select(contains($fp))' 2>/dev/null)"

        # T-3203: ALSO probe `inbox:` topics addressed to THIS project.
        #
        # This function writes last_mail_ts (see write_cycle), and notify-injector keys
        # on that flag advancing — not on the journal. So a topic missing from THIS
        # enumeration cannot wake the session no matter how well it is mirrored.
        # T-3201 fixed the mirror's identical dm:-only blindness and the journal went
        # 0 -> 50 rows; the rail still did not deliver, because the arrival flag is
        # written here. That fix was necessary and not sufficient — this is the rest.
        #
        # The two selectors are keyed DIFFERENTLY and that is not an inconsistency to
        # tidy up later: `dm:<fp_a>:<fp_b>` is addressed by identity FINGERPRINT, while
        # `inbox:<circuit>/<project>` is addressed by PROJECT NAME. Reusing the fp
        # predicate here would match nothing and would look exactly like a working fix.
        #
        # Scoped to our OWN mailbox, never a blanket `inbox:` prefix: 1 of the 23 inbox
        # topics on this hub is ours, and 95 of 232 records belong to AEF's ephemeral
        # e2e identities. Probing those would auto-confirm and journal another project's
        # mail and then inject it into our prompt — not noise, someone else's post.
        # T-3370: --no-project-inbox hands the project mailbox to its own receiver.
        if [ "$no_project_inbox" -eq 1 ]; then
            _inbox_topics=""
        else
        _inbox_topics="$("$TERMLINK" channel list "${hub_args[@]}" --prefix "inbox:" --json 2>/dev/null \
            | jq -r --arg self "$SELF_PROJECT" \
                '(.topics // .)[]?.name // empty
                 | select(endswith("/" + $self) or contains("/" + $self + "/"))' 2>/dev/null)"
        fi

        topics="$(printf '%s\n%s\n' "$_dm_topics" "$_inbox_topics" | sed '/^$/d')"
    fi

    local t n u first cls wake fmin newest
    while IFS= read -r t; do
        [ -n "$t" ] || continue
        u="$("$TERMLINK" channel unread "$t" "${hub_args[@]}" --sender "$fp" --json 2>/dev/null)"
        n="$(printf '%s' "$u" | jq -r '.unread_count // 0' 2>/dev/null)"
        case "$n" in ''|*[!0-9]*) n=0 ;; esac
        if [ "$n" -gt 0 ]; then
            first="$(printf '%s' "$u" | jq -r '.first_unread // empty' 2>/dev/null)"
            if cls="$(_classify_topic "$t" "$first" "$n")"; then
                read -r wake fmin newest <<<"$cls"
            else
                wake="$n" fmin="-" newest="?"
            fi
            if [ "$wake" -gt 0 ]; then
                total=$((total + wake))
                latest="$t"
                sig="$sig$t@$newest;"
            fi
            # V6-S3: recipient auto-confirm (journal + stage=delivered receipt), capped
            # below the first message addressed to someone else (T-3325).
            [ "$auto_confirm" -eq 1 ] && _auto_confirm_topic "$t" "$fp" "$fmin" "$first"
        fi
    done <<EOF
$topics
EOF

    if [ "$include_broadcast" -eq 1 ]; then
        local b
        b="$("$TERMLINK" channel unread agent-chat-arc "${hub_args[@]}" --sender "$fp" --json 2>/dev/null \
            | jq -r '.unread_count // 0' 2>/dev/null)"
        case "$b" in ''|*[!0-9]*) b=0 ;; esac
        if [ "$b" -gt 0 ]; then
            total=$((total + b))
            [ -z "$latest" ] && latest="agent-chat-arc"
            sig="${sig}agent-chat-arc@?;"
        fi
    fi

    printf '%s\t%s\t%s\n' "$total" "$latest" "$sig"
}

write_cycle() {
    local hb pending latest probe rc
    hb="$(now_ms)"

    probe="$(probe_mail)"; rc=$?
    if [ "$rc" -eq 3 ]; then
        echo "notify-sidecar: cannot resolve self identity (pass --self-fp)" >&2
        return 3
    fi
    local rest sig
    pending="${probe%%$'\t'*}"
    rest="${probe#*$'\t'}"
    latest="${rest%%$'\t'*}"
    if [ "$rest" != "${rest#*$'\t'}" ]; then sig="${rest#*$'\t'}"; else sig="?"; fi
    case "$pending" in ''|*[!0-9]*) pending=0 ;; esac

    mkdir -p "$notify_dir" 2>/dev/null || { echo "notify-sidecar: cannot create $notify_dir" >&2; return 3; }

    # Heartbeat ALWAYS written (proof-of-life), even when there is no mail —
    # that is what lets the consumer distinguish "alive, no mail" from "deaf".
    # Write-then-rename for atomic reads by notify-check.sh.
    printf '%s\n' "$hb" > "$notify_dir/.$agent_id.heartbeat.tmp" \
        && mv -f "$notify_dir/.$agent_id.heartbeat.tmp" "$notify_dir/$agent_id.heartbeat"

    # T-3068 — MONOTONIC ARRIVAL RECORD, alongside the unread snapshot.
    #
    # pending/latest_topic describe what is unread AT THIS INSTANT, and that is
    # all they ever described. When --auto-confirm acks, pending goes to 0 and
    # latest_topic goes EMPTY, so the flag stops carrying any memory that mail
    # arrived at all. A consumer polling the flag is then not losing a race — the
    # information has been destroyed. Measured: two controlled sends, sidecar
    # acked, consumer saw nothing across 40s.
    #
    # last_mail_ts/last_mail_topic PERSIST across the ack. They answer the
    # question the flag previously could not: "did anything arrive since I last
    # looked?" — as opposed to "is something unread right now". A consumer
    # triggers on last_mail_ts advancing past its own durable marker, which is a
    # value the acker cannot race to zero.
    #
    # Carried forward from the previous flag rather than kept in memory, so the
    # record survives a sidecar restart exactly as the ack guards do.
    local prev_mail_ts="" prev_mail_topic="" prev_mail_sig=""
    if [ -r "$notify_dir/$agent_id.flag" ]; then
        prev_mail_ts="$(grep -E '^last_mail_ts=' "$notify_dir/$agent_id.flag" 2>/dev/null | head -1 | cut -d= -f2-)"
        prev_mail_topic="$(grep -E '^last_mail_topic=' "$notify_dir/$agent_id.flag" 2>/dev/null | head -1 | cut -d= -f2-)"
        prev_mail_sig="$(grep -E '^last_mail_sig=' "$notify_dir/$agent_id.flag" 2>/dev/null | head -1 | cut -d= -f2-)"
    fi
    local mail_ts="$prev_mail_ts" mail_topic="$prev_mail_topic" mail_sig="$prev_mail_sig"
    if [ "$pending" -gt 0 ] 2>/dev/null; then
        # T-3325: advance only on NEW mail for us (sig changed). A "?" in sig means
        # the classifier was unavailable for some topic: keep the pre-T-3325
        # advance-every-cycle behaviour rather than risk sitting on unseen mail.
        case "$sig" in
            *'?'*) mail_ts="$hb"; mail_topic="$latest" ;;
            *)     if [ "$sig" != "$prev_mail_sig" ]; then mail_ts="$hb"; mail_topic="$latest"; fi ;;
        esac
        mail_sig="$sig"
    fi

    {
        printf 'pending=%s\n' "$pending"
        printf 'latest_topic=%s\n' "$latest"
        printf 'ts=%s\n' "$hb"
        printf 'last_mail_ts=%s\n' "$mail_ts"
        printf 'last_mail_topic=%s\n' "$mail_topic"
        printf 'last_mail_sig=%s\n' "$mail_sig"
    } > "$notify_dir/.$agent_id.flag.tmp" \
        && mv -f "$notify_dir/.$agent_id.flag.tmp" "$notify_dir/$agent_id.flag"

    if [ "$json" -eq 1 ]; then
        printf '{"agent_id":"%s","pending":%s,"latest_topic":"%s","heartbeat_ms":%s}\n' \
            "$agent_id" "$pending" "$latest" "$hb"
    fi
    return 0
}

# Loop mode — graceful SIGINT/SIGTERM.
keep_running=1
on_signal() { keep_running=0; }
trap on_signal INT TERM

if [ "$once" -eq 1 ]; then
    write_cycle
    exit $?
fi

while [ "$keep_running" -eq 1 ]; do
    write_cycle || exit 3
    n="$interval"
    while [ "$n" -gt 0 ] && [ "$keep_running" -eq 1 ]; do
        sleep 1
        n=$((n - 1))
    done
done

exit 0
