#!/usr/bin/env bash
# T-3135 — notify-sidecar API: the LOCAL control surface for this host's mailbox and prompt
# (arc-011 slices S1 + S12). Design: docs/design/arc-011-sidecar-api-architecture.md §6.
# Ops doc: docs/operations/notify-sidecar-api.md.
#
# WHAT IT IS
# ----------
# Five questions an operator could previously answer only by cat-ing flag files and guessing:
#
#   status                        am I alive, when did I last cycle, is the hub reachable, who am I
#   queue                         what is pending for this agent, in the order it WILL be injected
#   inject <id|next>              put this in front of my agent now (READY-gated — SQ-4)
#   agent-state                   READY | BUSY | UNKNOWN | NOT-RUNNING
#   ack <offset> --evidence KIND  tell the sender it reached the PROMPT (L3), never without evidence
#
# It invents no transport and no state. Every verb wraps a seam that already exists and is
# proven: notify-sidecar.sh's flag/heartbeat files, the journal.sqlite mirror (T-2298/T-3071),
# notify-injector.sh (T-3069), notify-ack-read.sh (T-3067), scripts/lib/pty-state.sh
# (T-2402/T-3069), notify-sidecar-supervisor.sh --loop (T-3135, SQ-8).
#
# THE BRIGHT LINE (§6) — ENFORCED, NOT REMEMBERED
# -----------------------------------------------
# This API may read and act on THIS HOST'S OWN state. The moment it moves a message BETWEEN
# hosts it has become a second bus, which the charter refuses (non-goal #1, G-060). So any
# remote-address argument (--hub / --peer / --to / --address / --remote) is refused with exit 2,
# and tests/notify-sidecar-api-fixtures.sh statically asserts this file never invokes a
# cross-host verb (channel post, agent contact, agent send, remote, artifact put, broadcast).
# Mirrors crates/termlink-hub/tests/no_federation_tripwire.rs. A future edit adding one fails
# the suite rather than shipping quietly under a name that sounds local.
#
# `inject <id>` IS DELIBERATELY NARROW. The queue has a declared priority policy (T-3071:
# band first, FIFO inside the band). This verb accepts only the id that policy says is NEXT;
# any other id exits 1 and names the actual head. Jumping the queue here would be a second
# ordering policy in a second place — the drift shape this repo has documented three times.
#
# EXIT CODES (every verb)
#   0  answered / acted        (status: listener ALIVE; agent-state: READY; inject: INJECTED+VERIFIED)
#   1  a not-OK verdict        (DEAF, BUSY/UNKNOWN deferred, NOT-RUNNING, not-next, acker refused)
#   2  usage / tooling         "I could not look" — NEVER a verdict (fail-closed)
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TERMLINK="${TERMLINK_BIN:-termlink}"
NOTIFY_DIR="${TERMLINK_NOTIFY_DIR:-$HOME/.termlink/notify}"
JOURNAL="${TERMLINK_JOURNAL_PATH:-$HOME/.termlink/journals/journal.sqlite}"
INJECTOR="${SIDECAR_API_INJECTOR:-$HERE/notify-injector.sh}"
ACKER="${SIDECAR_API_ACKER:-$HERE/notify-ack-read.sh}"
DEAF_AFTER="${SIDECAR_API_DEAF_AFTER:-90}"
META_TYPES="('receipt','reaction','redaction','edit','topic_metadata')"

AGENT_ID=""; SESSION=""; TOPIC=""; EVIDENCE=""; JSON=0; QUIET=0
VERB=""; ARG=""

usage() {
    cat <<'EOF'
Usage: notify-sidecar-api.sh <verb> [arg] --agent-id NAME [options]

Verbs (local control only — this host's own mailbox and prompt, §6 bright line):
  status                         listener alive? last cycle? hub reachable from here? who am I?
  queue                          pending messages for this agent, in injection order
  inject <id|next>               put the NEXT message in front of the agent (READY-gated, SQ-4)
  agent-state                    READY | BUSY | UNKNOWN | NOT-RUNNING  (needs --session)
  ack <offset> --evidence KIND   post L3 stage=read for this host's own mail (evidence mandatory)

Options:
  --agent-id NAME     required — the agent whose rail this is
  --session NAME      the agent's registered PTY (required by inject and agent-state)
  --topic T           dm: topic (default: the flag's last_mail_topic)
  --evidence KIND     idle-gated-inject | observed-turn | operator   (ack only)
  --notify-dir DIR    flag/heartbeat dir (default ~/.termlink/notify)
  --journal PATH      queue source (default ~/.termlink/journals/journal.sqlite)
  --json              one JSON object (queue: one object carrying a rows array)
  --quiet             verdict line only
  -h, --help          this text

Refused (exit 2): --hub, --peer, --to, --address, --remote — anything that would make
this a second bus. Cross-host delivery is `channel.post` via the hub, full stop.

Exit: 0 answered/acted · 1 not-OK verdict · 2 usage/tooling (never a verdict)
Test seams (PL-213): SIDECAR_API_TEST_FQDN, SIDECAR_API_TEST_IP, SIDECAR_API_TEST_HUB_RC,
  SIDECAR_API_TEST_PTY_STATE, SIDECAR_API_TEST_SESSION_STATE, SIDECAR_API_INJECTOR, SIDECAR_API_ACKER
EOF
}

die_usage() { echo "notify-sidecar-api: $*" >&2; echo "Try --help for usage." >&2; exit 2; }

while [ $# -gt 0 ]; do
    case "$1" in
        --agent-id)   AGENT_ID="${2:-}"; shift 2 ;;
        --session)    SESSION="${2:-}"; shift 2 ;;
        --topic)      TOPIC="${2:-}"; shift 2 ;;
        --evidence)   EVIDENCE="${2:-}"; shift 2 ;;
        --notify-dir) NOTIFY_DIR="${2:-}"; shift 2 ;;
        --journal)    JOURNAL="${2:-}"; shift 2 ;;
        --json)       JSON=1; shift ;;
        --quiet)      QUIET=1; shift ;;
        -h|--help)    usage; exit 0 ;;
        --hub|--hub=*|--peer|--peer=*|--to|--to=*|--address|--address=*|--remote|--remote=*)
            echo "notify-sidecar-api: REFUSED '$1' — bright line (design §6): this API reads and acts on THIS host's own state only." >&2
            echo "  Moving a message between hosts is channel.post via the hub (scripts/agent-send.sh, /agent-handoff), never this surface." >&2
            exit 2 ;;
        --*) die_usage "unknown option: $1" ;;
        *)
            if [ -z "$VERB" ]; then VERB="$1"
            elif [ -z "$ARG" ]; then ARG="$1"
            else die_usage "unexpected argument: $1"; fi
            shift ;;
    esac
done

[ -n "$VERB" ]     || die_usage "a verb is required (status | queue | inject | agent-state | ack)"
[ -n "$AGENT_ID" ] || die_usage "--agent-id is required"
case "$VERB" in status|queue|inject|agent-state|ack) : ;; *) die_usage "unknown verb: $VERB" ;; esac

say()  { [ "$QUIET" = "1" ] || echo "notify-sidecar-api: $*"; }
loud() { echo "notify-sidecar-api: $*"; }
kv()   { grep -E "^$2=" "$1" 2>/dev/null | head -1 | cut -d= -f2-; }
now_ms() { echo $(( $(date +%s) * 1000 )); }
json_str() { printf '%s' "$1" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))' 2>/dev/null || printf '"%s"' "$1"; }

FLAG="$NOTIFY_DIR/$AGENT_ID.flag"
HB="$NOTIFY_DIR/$AGENT_ID.heartbeat"
flag_topic() { [ -n "$TOPIC" ] && { echo "$TOPIC"; return; }; kv "$FLAG" last_mail_topic; }
sanitize()   { printf '%s' "$1" | tr -c 'A-Za-z0-9._-' '_'; }

# ---- shared: the queue query. IDENTICAL predicate + ordering to notify-injector.sh --------
# Content rows only (meta envelopes are not work), above the LOCAL delivered marker, priority
# band DESC then FIFO (ts, offset). The injector additionally maxes the marker with the hub's
# L3 watermark; this surface stays hub-independent on purpose (design §3: answerable during
# an outage) and SAYS so via watermark_source=local.
queue_wm() {    # $1 topic ; echoes the LOCAL delivered watermark (-1 when none)
    local wm=-1 off="$NOTIFY_DIR/.$AGENT_ID.$(sanitize "$1").delivered-offset"
    [ -r "$off" ] && wm="$(tr -dc '0-9' < "$off")"; : "${wm:=-1}"
    case "$wm" in ''|*[!0-9-]*) wm=-1 ;; esac
    echo "$wm"
}
queue_rows() {  # $1 topic ; echoes offset|priority|ts|sender_id|payload-head per line
    local topic="$1" sql_topic wm prio_order
    sql_topic="${topic//\'/\'\'}"
    wm="$(queue_wm "$topic")"
    if sqlite3 "$JOURNAL" "SELECT 1 FROM pragma_table_info('messages') WHERE name='priority';" 2>/dev/null | grep -q 1; then
        prio_order="COALESCE(priority,0) DESC, "
    else prio_order=""; fi
    sqlite3 -separator '|' "$JOURNAL" \
        "SELECT offset, COALESCE(priority,0), ts, sender_id, replace(substr(payload,1,160),char(10),' ') FROM messages
          WHERE topic='$sql_topic' AND offset > $wm AND msg_type NOT IN $META_TYPES
          ORDER BY ${prio_order}ts ASC, offset ASC;"
}
need_sqlite() {
    command -v sqlite3 >/dev/null 2>&1 || { loud "sqlite3 not found — cannot read the queue (tooling)"; exit 2; }
    [ -r "$JOURNAL" ] || { loud "journal not readable: $JOURNAL — cannot read the queue (tooling, not 'empty')"; exit 2; }
}

# ---- shared: session + prompt state -------------------------------------------------------
session_present() {  # 0 present, 1 absent
    if [ -n "${SIDECAR_API_TEST_SESSION_STATE:-}" ]; then
        [ "$SIDECAR_API_TEST_SESSION_STATE" = "present" ]; return
    fi
    "$TERMLINK" list 2>/dev/null | awk 'NR>2 {print $1; print $2}' | grep -qxF -- "$SESSION"
}
prompt_state() {
    if [ -n "${SIDECAR_API_TEST_PTY_STATE:-}" ]; then printf '%s' "$SIDECAR_API_TEST_PTY_STATE"; return; fi
    # shellcheck source=lib/pty-state.sh
    . "$HERE/lib/pty-state.sh"
    pushwaker_probe_pty "$SESSION"
}

# =========================================================================================
case "$VERB" in
# -----------------------------------------------------------------------------------------
status)
    hb_ms=""; hb_age=""; listener="DEAF"
    if [ -r "$HB" ]; then
        hb_ms="$(tr -dc '0-9' < "$HB")"
        [ -n "$hb_ms" ] && hb_age=$(( ( $(now_ms) - hb_ms ) / 1000 ))
    fi
    if [ -n "$hb_age" ] && [ "$hb_age" -le "$DEAF_AFTER" ]; then listener="ALIVE"; fi
    pending="$(kv "$FLAG" pending)"; : "${pending:=0}"
    last_topic="$(kv "$FLAG" last_mail_topic)"
    last_ts="$(kv "$FLAG" last_mail_ts)"
    sidecar_pid=""
    [ -r "$NOTIFY_DIR/$AGENT_ID.pid" ] && sidecar_pid="$(tr -dc '0-9' < "$NOTIFY_DIR/$AGENT_ID.pid")"
    [ -z "$sidecar_pid" ] && sidecar_pid="$(pgrep -f -- "notify-sidecar.sh --agent-id $AGENT_ID( |\$)" 2>/dev/null | head -1)"
    if [ -n "$sidecar_pid" ] && ! kill -0 "$sidecar_pid" 2>/dev/null; then sidecar_pid=""; fi
    sup_pid=""; sup_age=""
    if [ -r "$NOTIFY_DIR/.supervisor.pid" ]; then
        sup_pid="$(tr -dc '0-9' < "$NOTIFY_DIR/.supervisor.pid")"
        [ -n "$sup_pid" ] && ! kill -0 "$sup_pid" 2>/dev/null && sup_pid=""
    fi
    if [ -r "$NOTIFY_DIR/.supervisor.heartbeat" ]; then
        v="$(tr -dc '0-9' < "$NOTIFY_DIR/.supervisor.heartbeat")"; [ -n "$v" ] && sup_age=$(( ( $(now_ms) - v ) / 1000 ))
    fi
    if [ -n "${SIDECAR_API_TEST_HUB_RC:-}" ]; then hub_rc="$SIDECAR_API_TEST_HUB_RC"
    else timeout 5 "$TERMLINK" hub status --json >/dev/null 2>&1; hub_rc=$?; fi
    hub_reachable=false; [ "$hub_rc" = "0" ] && hub_reachable=true

    # AC4 — own identity, re-resolved every call and compared to the last recorded values.
    # A change is REPORTED loudly (host_identity_changed=true old->new), never silently absorbed.
    fqdn="${SIDECAR_API_TEST_FQDN:-}"; ip="${SIDECAR_API_TEST_IP:-}"
    [ -n "$fqdn" ] || fqdn="$(hostname -f 2>/dev/null || hostname 2>/dev/null || echo unknown)"
    if [ -z "$ip" ]; then
        ip="$(hostname -I 2>/dev/null | awk '{print $1}')"
        [ -n "$ip" ] || ip="$(ip -4 route get 1.1.1.1 2>/dev/null | awk '{for(i=1;i<=NF;i++) if($i=="src"){print $(i+1); exit}}')"
        [ -n "$ip" ] || ip="$(ifconfig 2>/dev/null | awk '/inet / && $2!="127.0.0.1" {print $2; exit}')"
        : "${ip:=unknown}"
    fi
    IDF="$NOTIFY_DIR/.host-identity"
    old_fqdn="$(kv "$IDF" fqdn)"; old_ip="$(kv "$IDF" ip)"
    changed=false
    if [ -r "$IDF" ] && { [ "$old_fqdn" != "$fqdn" ] || [ "$old_ip" != "$ip" ]; }; then changed=true; fi
    mkdir -p "$NOTIFY_DIR" 2>/dev/null
    { printf 'fqdn=%s\nip=%s\nts=%s\n' "$fqdn" "$ip" "$(now_ms)"; } > "$IDF.tmp" 2>/dev/null && mv -f "$IDF.tmp" "$IDF" 2>/dev/null

    if [ "$JSON" = "1" ]; then
        printf '{"ok":%s,"agent_id":%s,"listener":"%s","heartbeat_age_secs":%s,"deaf_after_secs":%s,"pending":%s,"last_mail_topic":%s,"last_mail_ts":%s,"sidecar_pid":%s,"supervisor_pid":%s,"supervisor_heartbeat_age_secs":%s,"hub_reachable":%s,"host_fqdn":%s,"host_ip":%s,"host_identity_changed":%s,"host_identity_prev":{"fqdn":%s,"ip":%s},"notify_dir":%s}\n' \
            "$([ "$listener" = ALIVE ] && echo true || echo false)" "$(json_str "$AGENT_ID")" "$listener" "${hb_age:-null}" "$DEAF_AFTER" \
            "$pending" "$(json_str "$last_topic")" "${last_ts:-null}" "${sidecar_pid:-null}" "${sup_pid:-null}" "${sup_age:-null}" \
            "$hub_reachable" "$(json_str "$fqdn")" "$(json_str "$ip")" "$changed" "$(json_str "$old_fqdn")" "$(json_str "$old_ip")" "$(json_str "$NOTIFY_DIR")"
    else
        loud "listener=$listener heartbeat_age=${hb_age:-none}s (deaf after ${DEAF_AFTER}s) sidecar_pid=${sidecar_pid:-none}"
        say  "mail: pending=$pending last_topic=${last_topic:-none} last_ts=${last_ts:-none}"
        say  "supervisor: pid=${sup_pid:-none} heartbeat_age=${sup_age:-none}s"
        say  "hub_reachable=$hub_reachable"
        say  "host: fqdn=$fqdn ip=$ip"
        [ "$changed" = true ] && loud "HOST IDENTITY CHANGED: fqdn ${old_fqdn:-?} -> $fqdn, ip ${old_ip:-?} -> $ip (recorded in $IDF)"
    fi
    [ "$listener" = "ALIVE" ] && exit 0
    [ "$JSON" = "1" ] || loud "DEAF — heartbeat missing or stale; 'no mail' cannot be trusted until the sidecar is back (bash scripts/notify-sidecar-supervisor.sh)"
    exit 1 ;;
# -----------------------------------------------------------------------------------------
queue)
    need_sqlite
    topic="$(flag_topic)"
    if [ -z "$topic" ]; then
        if [ "$JSON" = "1" ]; then printf '{"ok":true,"agent_id":%s,"topic":null,"rows":[],"count":0,"note":"no arrival recorded on the flag and no --topic given"}\n' "$(json_str "$AGENT_ID")"
        else say "no arrival recorded on $FLAG and no --topic given — nothing to list"; fi
        exit 0
    fi
    QUEUE_WM="$(queue_wm "$topic")"
    rows="$(queue_rows "$topic")" || { loud "journal query failed on $JOURNAL (tooling)"; exit 2; }
    count=0; [ -n "$rows" ] && count="$(printf '%s\n' "$rows" | wc -l | tr -d ' ')"
    if [ "$JSON" = "1" ]; then
        printf '{"ok":true,"agent_id":%s,"topic":%s,"watermark_source":"local","delivered_watermark":%s,"count":%s,"rows":[' \
            "$(json_str "$AGENT_ID")" "$(json_str "$topic")" "$QUEUE_WM" "$count"
        first=1
        while IFS='|' read -r off prio ts sender head; do
            [ -n "$off" ] || continue
            [ "$first" = 1 ] || printf ','; first=0
            printf '{"offset":%s,"priority":%s,"ts":%s,"sender_id":%s,"payload_head":%s}' "$off" "$prio" "${ts:-0}" "$(json_str "$sender")" "$(json_str "$head")"
        done <<< "$rows"
        printf ']}\n'
    else
        loud "queue for $AGENT_ID on $topic: $count pending (above local watermark $QUEUE_WM; injector also consults hub L3)"
        [ -n "$rows" ] && printf '%s\n' "$rows" | awk -F'|' '{printf "  %-8s prio=%-2s ts=%-14s from=%-20s %s\n", $1, $2, $3, $4, $5}'
    fi
    exit 0 ;;
# -----------------------------------------------------------------------------------------
agent-state)
    [ -n "$SESSION" ] || die_usage "--session is required for agent-state (the agent's registered PTY)"
    if ! session_present; then
        state="NOT-RUNNING"
    else
        state="$(prompt_state)"
        case "$state" in READY|BUSY|UNKNOWN) : ;; *) state="UNKNOWN" ;; esac
    fi
    if [ "$JSON" = "1" ]; then printf '{"ok":%s,"agent_id":%s,"session":%s,"state":"%s"}\n' "$([ "$state" = READY ] && echo true || echo false)" "$(json_str "$AGENT_ID")" "$(json_str "$SESSION")" "$state"
    else loud "agent-state=$state (session=$SESSION)"; fi
    [ "$state" = "READY" ] && exit 0
    exit 1 ;;
# -----------------------------------------------------------------------------------------
inject)
    [ -n "$ARG" ]     || die_usage "inject needs <id|next>"
    [ -n "$SESSION" ] || die_usage "--session is required for inject (the agent's registered PTY)"
    [ -x "$INJECTOR" ] || [ -r "$INJECTOR" ] || { loud "injector not found: $INJECTOR (tooling)"; exit 2; }
    need_sqlite
    topic="$(flag_topic)"
    [ -n "$topic" ] || { loud "no arrival recorded on $FLAG and no --topic — nothing to inject"; exit 1; }
    QUEUE_WM="$(queue_wm "$topic")"
    head_row="$(queue_rows "$topic" | head -1)" || { loud "journal query failed (tooling)"; exit 2; }
    head_off="${head_row%%|*}"
    [ -n "$head_off" ] || { loud "queue empty on $topic above watermark $QUEUE_WM — nothing to inject"; exit 1; }
    if [ "$ARG" != "next" ] && [ "$ARG" != "$head_off" ]; then
        loud "REFUSED: $ARG is not next. The priority policy (T-3071) says offset $head_off is next on $topic; this surface never re-orders the queue. To move a message forward, raise its priority at the source."
        [ "$JSON" = "1" ] && printf '{"ok":false,"refused":"not-next","requested":%s,"next":%s,"topic":%s}\n' "$(json_str "$ARG")" "$head_off" "$(json_str "$topic")"
        exit 1
    fi
    say "delegating to $(basename "$INJECTOR") for offset $head_off on $topic (READY-gated, SQ-4)"
    inj_args=(--agent-id "$AGENT_ID" --session "$SESSION" --notify-dir "$NOTIFY_DIR" --journal "$JOURNAL" --topic-filter "$topic")
    [ "$QUIET" = "1" ] && inj_args+=(--quiet)
    out="$(bash "$INJECTOR" "${inj_args[@]}" 2>&1)"; rc=$?
    [ "$JSON" = "1" ] || printf '%s\n' "$out"
    case "$rc" in
        0) verdict="INJECTED+VERIFIED"; ok=true; code=0 ;;
        1) verdict="nothing-to-do";     ok=false; code=1 ;;
        2) verdict="tooling";           ok=false; code=2 ;;
        3) verdict="NOT-RUNNING";       ok=false; code=1 ;;
        4) verdict="deferred-busy-or-unknown"; ok=false; code=1 ;;
        5) verdict="INJECTED-NOT-VERIFIED";    ok=false; code=1 ;;
        *) verdict="unexpected-rc-$rc"; ok=false; code=2 ;;
    esac
    [ "$JSON" = "1" ] && printf '{"ok":%s,"verdict":"%s","injector_rc":%s,"offset":%s,"topic":%s}\n' "$ok" "$verdict" "$rc" "$head_off" "$(json_str "$topic")"
    [ "$JSON" = "1" ] || loud "inject verdict=$verdict (injector rc=$rc)"
    exit "$code" ;;
# -----------------------------------------------------------------------------------------
ack)
    [ -n "$ARG" ]      || die_usage "ack needs <offset>"
    case "$ARG" in ''|*[!0-9]*) die_usage "ack <offset> must be an integer content offset — a count is not an offset" ;; esac
    [ -n "$EVIDENCE" ] || die_usage "ack requires --evidence (idle-gated-inject | observed-turn | operator). There is no evidence kind for 'I sent it and hoped'."
    topic="$(flag_topic)"
    [ -n "$topic" ]    || die_usage "ack needs a topic: none recorded on $FLAG and no --topic given"
    [ -x "$ACKER" ] || [ -r "$ACKER" ] || { loud "acker not found: $ACKER (tooling)"; exit 2; }
    ack_args=(--topic "$topic" --up-to "$ARG" --evidence "$EVIDENCE" --agent-id "$AGENT_ID")
    [ "$QUIET" = "1" ] && ack_args+=(--quiet)
    out="$(bash "$ACKER" "${ack_args[@]}" 2>&1)"; rc=$?
    [ "$JSON" = "1" ] || printf '%s\n' "$out"
    case "$rc" in 0) ok=true; code=0; verdict="posted-or-already-acked" ;; 2) ok=false; code=2; verdict="tooling" ;; *) ok=false; code=1; verdict="refused-rc-$rc" ;; esac
    [ "$JSON" = "1" ] && printf '{"ok":%s,"verdict":"%s","acker_rc":%s,"topic":%s,"up_to":%s,"evidence":%s}\n' "$ok" "$verdict" "$rc" "$(json_str "$topic")" "$ARG" "$(json_str "$EVIDENCE")"
    exit "$code" ;;
esac
