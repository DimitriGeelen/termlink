#!/usr/bin/env bash
# T-1841 — be-reachable wrapper for ephemeral claude-code sessions.
#
# One-command opt-in to agent-presence (T-1830). Backgrounds
# listener-heartbeat.sh (T-1832), persists PID + agent_id in
# ~/.termlink/be-reachable.state, applies sensible defaults so a
# claude-code session becomes reachable via T-1834 --to auto-discover
# in under 30 seconds.
#
# Mirrors the persistent-host rail (T-1840 systemd template) but for
# session-lifetime instances that should die with the terminal.
#
# Subcommands:
#   start [--agent-id X] [--pty-session Y] [--listen-topic T]... [--role R]
#         [--interval N] [--hub addr] [--topic AGENT-PRESENCE]
#   stop
#   status [--json]
#
# State file: ~/.termlink/be-reachable.state (or $BE_REACHABLE_STATE)
#   JSON: { agent_id, pid, started_at, role, interval, topic,
#           listen_topics: [...], pty_session, hub }
#
# Exit codes:
#   0  — success (incl. idempotent already-running / not-running)
#   1  — status reports not-running (status only)
#   2  — usage error
#   3  — hub-side / spawn failure
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
STATE_DIR="${BE_REACHABLE_STATE_DIR:-${HOME}/.termlink}"
STATE_FILE="${BE_REACHABLE_STATE:-${STATE_DIR}/be-reachable.state}"

SELF_DIR="$(cd "$(dirname "$0")" && pwd)"

LH_SCRIPT="${BE_REACHABLE_LH_SCRIPT:-}"
if [ -z "$LH_SCRIPT" ]; then
    # Resolve listener-heartbeat.sh relative to this script's directory by default.
    LH_SCRIPT="${SELF_DIR}/listener-heartbeat.sh"
fi

# T-2316 (arc-004 WP1): the push-waker that rings the PTY doorbell on an inbound
# inbox deposit. Spawned alongside the heartbeat when a pty_session is bound.
PW_SCRIPT="${BE_REACHABLE_PW_SCRIPT:-${SELF_DIR}/be-reachable-pushwaker.sh}"

usage() {
    cat <<'EOF'
Usage: be-reachable.sh <subcommand> [options]

Subcommands:
  start    Background a listener-heartbeat process for this session.
  stop     Kill the background process and clear state.
  status   Report running state. Exit 0=running, 1=not running.
  --help   Print this help and exit 0.

start options:
  --agent-id NAME      Logical agent name (default: $USER-claude-$(hostname -s))
  --pty-session NAME   PTY session name for doorbell ring (default: auto-detect
                       from $TMUX or $STY; empty if none)
  --listen-topic T     Topic to declare in metadata. Repeatable.
                       Default: dm:<agent_id>:* and agent-chat-arc.
  --role R             Role string (default: claude-code)
  --interval N         Heartbeat period seconds (default: 30, min: 5)
  --hub addr           Target hub (default: local)
  --topic TOPIC        Presence topic (default: agent-presence)

status options:
  --json               Emit JSON instead of human-readable text.

Environment:
  BE_REACHABLE_STATE       Override state file path
  BE_REACHABLE_LH_SCRIPT   Override listener-heartbeat.sh path
  TERMLINK_BIN             Override termlink binary

Examples:
  be-reachable.sh start                       # default agent_id, default topics
  be-reachable.sh start --agent-id me         # custom name
  be-reachable.sh status --json
  be-reachable.sh stop

After `start`, peers can reach you via:
  termlink agent contact <agent_id> --message "[T-XXX] ..."
  bash scripts/agent-send.sh --to <agent_id> --message "..."
EOF
}

die_usage() {
    echo "be-reachable: $*" >&2
    echo "Try --help for usage." >&2
    exit 2
}

# ---- state file helpers --------------------------------------------------

ensure_state_dir() {
    mkdir -p "$STATE_DIR"
    chmod 700 "$STATE_DIR" 2>/dev/null || true
}

read_state_field() {
    # $1 = field name. Prints value to stdout, empty if missing.
    [ -f "$STATE_FILE" ] || return 0
    if command -v jq >/dev/null 2>&1; then
        jq -r ".${1} // empty" "$STATE_FILE" 2>/dev/null
    else
        # Fallback: naive grep for "field": "value" or "field": <int>
        sed -n "s/.*\"${1}\"[[:space:]]*:[[:space:]]*\"\\([^\"]*\\)\".*/\\1/p" "$STATE_FILE" | head -n1
    fi
}

pid_alive() {
    local pid="$1"
    [ -n "$pid" ] && [ "$pid" -gt 0 ] 2>/dev/null && kill -0 "$pid" 2>/dev/null
}

# ---- defaults ------------------------------------------------------------

default_agent_id() {
    local u h
    u="${USER:-$(id -un 2>/dev/null || echo agent)}"
    h="$(hostname -s 2>/dev/null || echo host)"
    # Normalize: lowercase, replace non-alnum with -
    printf '%s' "${u}-claude-${h}" | tr 'A-Z' 'a-z' | tr -c 'a-z0-9-' '-' | sed 's/--*/-/g; s/^-//; s/-$//'
}

default_pty_session() {
    # tmux: $TMUX is "/tmp/tmux-1000/default,12345,0" → grab session via tmux display-message if available
    if [ -n "${TMUX:-}" ]; then
        if command -v tmux >/dev/null 2>&1; then
            tmux display-message -p '#S' 2>/dev/null || echo ""
        else
            # Fall back to client name parsed from $TMUX
            printf '%s' "$TMUX" | awk -F, '{print $1}' | xargs -I{} basename {} 2>/dev/null || echo ""
        fi
        return
    fi
    if [ -n "${STY:-}" ]; then
        # screen: STY is "PID.session"
        printf '%s' "$STY" | sed 's/^[0-9]*\.//'
        return
    fi
    echo ""
}

# ---- subcommands ---------------------------------------------------------

cmd_start() {
    local agent_id=""
    local pty_session=""
    local pty_session_set=0
    local listen_topics=()
    local role="claude-code"
    local interval=30
    local hub=""
    local topic="agent-presence"
    # T-2045 (T-2020 GO): comma-separated capability tags surfaced in
    # heartbeat metadata.capabilities. Pairs with `termlink agent find-idle
    # --capability X` for orchestrator dispatch. Free-form by convention.
    local capabilities="${TERMLINK_CAPABILITIES:-}"

    while [ $# -gt 0 ]; do
        case "$1" in
            --agent-id)      agent_id="${2:-}"; shift 2 ;;
            --pty-session)   pty_session="${2:-}"; pty_session_set=1; shift 2 ;;
            --listen-topic)  listen_topics+=("${2:-}"); shift 2 ;;
            --role)          role="${2:-}"; shift 2 ;;
            --interval)      interval="${2:-}"; shift 2 ;;
            --hub)           hub="${2:-}"; shift 2 ;;
            --topic)         topic="${2:-}"; shift 2 ;;
            --capabilities)  capabilities="${2:-}"; shift 2 ;;
            -h|--help)       usage; exit 0 ;;
            *)               die_usage "unknown start arg: $1" ;;
        esac
    done

    # Pre-flight: listener-heartbeat.sh must exist & be executable.
    if [ ! -x "$LH_SCRIPT" ]; then
        echo "be-reachable: listener-heartbeat.sh not found or not executable: $LH_SCRIPT" >&2
        echo "Set BE_REACHABLE_LH_SCRIPT to override." >&2
        exit 3
    fi

    ensure_state_dir

    # Idempotent: existing PID alive?
    if [ -f "$STATE_FILE" ]; then
        local existing_pid existing_id
        existing_pid="$(read_state_field pid)"
        existing_id="$(read_state_field agent_id)"
        if pid_alive "$existing_pid"; then
            echo "be-reachable: already running as ${existing_id:-?} (pid ${existing_pid})"
            echo "  state: ${STATE_FILE}"
            echo "  stop first if you want to change agent_id or topics."
            exit 0
        fi
        # Stale state: PID dead, clear it.
        rm -f "$STATE_FILE"
    fi

    # Apply defaults.
    [ -n "$agent_id" ] || agent_id="$(default_agent_id)"

    # T-2292: per-agent identity by default. Bind the resolved agent_id to the
    # crypto identity for this start flow and the spawned heartbeat, so posts
    # SIGN with ~/.termlink/identities/<agent_id>.key rather than the shared
    # host key — co-resident agents get DISTINCT fingerprints (RC1, T-2291).
    # Inherited by the backgrounded listener-heartbeat.sh (which also exports
    # it defensively). Explicit TERMLINK_IDENTITY_FILE/DIR still wins via the
    # resolver precedence (FILE > DIR > AGENT_ID > shared default).
    export TERMLINK_AGENT_ID="$agent_id"

    if [ "$pty_session_set" -eq 0 ]; then
        pty_session="$(default_pty_session)"
    fi
    if [ "${#listen_topics[@]}" -eq 0 ]; then
        listen_topics=("dm:${agent_id}:*" "agent-chat-arc")
    fi

    # Build listener-heartbeat.sh args.
    local lh_args=( --agent-id "$agent_id" --role "$role" --topic "$topic" --interval "$interval" )
    [ -n "$pty_session" ] && lh_args+=( --pty-session "$pty_session" )
    [ -n "$hub" ] && lh_args+=( --hub "$hub" )
    [ -n "$capabilities" ] && lh_args+=( --capabilities "$capabilities" )
    local t
    for t in "${listen_topics[@]}"; do
        [ -n "$t" ] && lh_args+=( --listen-topic "$t" )
    done

    # Spawn detached. nohup + setsid so it survives this shell exit.
    # Stdout/stderr → state log alongside the state file.
    local log_file="${STATE_DIR}/be-reachable.log"
    : > "$log_file"

    if command -v setsid >/dev/null 2>&1; then
        nohup setsid "$LH_SCRIPT" "${lh_args[@]}" >>"$log_file" 2>&1 &
    else
        nohup "$LH_SCRIPT" "${lh_args[@]}" >>"$log_file" 2>&1 &
    fi
    local pid=$!
    disown 2>/dev/null || true

    # Brief settle so we can detect immediate-exit failures.
    sleep 1
    if ! pid_alive "$pid"; then
        echo "be-reachable: listener-heartbeat.sh exited immediately. log: ${log_file}" >&2
        tail -n 20 "$log_file" >&2 || true
        exit 3
    fi

    # T-2316 (arc-004 WP1): spawn the push-waker so an inbound inbox deposit rings
    # this session's PTY doorbell the instant it lands (via the shipped WS push),
    # instead of waiting for the receiver's poll cycle. Only meaningful when a
    # pty_session is bound (nothing to ring otherwise). Non-fatal: a waker that
    # fails to start does NOT block reachability — the durable poll path remains
    # the floor. Its inbox id is the agent_id (the session's inbox namespace).
    # T-2324 (arc-004 S2): resolve THIS session's per-agent identity fingerprint
    # so the push-waker can also ring on `dm.queued` frames addressed to it — a
    # direct dm:<a>:<b> post by a NON-live-sender (raw post, cron, remote peer,
    # MCP) whose addressee is the self-fp, not the inbox id. We ask the CLI with
    # --resolve so the SAME precedence the signing path uses (FILE > AGENT_ID >
    # DIR > shared host default) is honored; plain `agent identity` reports the
    # shared host key for a per-agent session (PL-236). TERMLINK_AGENT_ID is
    # already exported above, so the resolver picks the per-agent key. Best-effort:
    # on any failure self_fp stays empty and the waker runs inbox-rail only
    # (back-compat, no dm rail).
    local self_fp=""
    if command -v jq >/dev/null 2>&1; then
        self_fp="$("$TERMLINK" agent identity --resolve --json 2>/dev/null | jq -r '.fingerprint // empty' 2>/dev/null)" || self_fp=""
    fi

    local pushwaker_pid=""
    if [ -n "$pty_session" ] && [ -x "$PW_SCRIPT" ]; then
        local pw_args=( --inbox-id "$agent_id" --pty-session "$pty_session" )
        [ -n "$hub" ] && pw_args+=( --hub "$hub" )
        [ -n "$self_fp" ] && pw_args+=( --self-fp "$self_fp" )
        if command -v setsid >/dev/null 2>&1; then
            nohup setsid "$PW_SCRIPT" "${pw_args[@]}" >>"$log_file" 2>&1 &
        else
            nohup "$PW_SCRIPT" "${pw_args[@]}" >>"$log_file" 2>&1 &
        fi
        pushwaker_pid=$!
        disown 2>/dev/null || true
    fi

    # Write state file.
    local started_at
    started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

    # Build the listen_topics JSON array manually (jq optional in environment).
    local lt_json="["
    local first=1
    for t in "${listen_topics[@]}"; do
        if [ -z "$t" ]; then continue; fi
        if [ $first -eq 1 ]; then
            lt_json+="\"$t\""
            first=0
        else
            lt_json+=",\"$t\""
        fi
    done
    lt_json+="]"

    cat >"$STATE_FILE" <<EOF
{
  "agent_id": "${agent_id}",
  "pid": ${pid},
  "started_at": "${started_at}",
  "role": "${role}",
  "interval": ${interval},
  "topic": "${topic}",
  "listen_topics": ${lt_json},
  "pty_session": "${pty_session}",
  "pushwaker_pid": ${pushwaker_pid:-null},
  "hub": "${hub}"
}
EOF
    chmod 600 "$STATE_FILE" 2>/dev/null || true

    cat <<EOF
be-reachable: started.
  agent_id:      ${agent_id}
  pid:           ${pid}
  push_waker:    $([ -n "$pushwaker_pid" ] && echo "pid ${pushwaker_pid} (rings PTY on inbox deposit, T-2316)" || echo "<none — no pty_session bound>")
  pty_session:   ${pty_session:-<none>}
  listen_topics: $(IFS=,; echo "${listen_topics[*]}")
  state:         ${STATE_FILE}
  log:           ${log_file}

Peers can reach you via:
  termlink agent contact ${agent_id} --message "[T-XXX] ..."
  bash scripts/agent-send.sh --to ${agent_id} --message "..."

Stop with: be-reachable.sh stop
EOF
}

cmd_stop() {
    if [ ! -f "$STATE_FILE" ]; then
        echo "be-reachable: not running (no state file)."
        exit 0
    fi
    local pid agent_id pushwaker_pid
    pid="$(read_state_field pid)"
    agent_id="$(read_state_field agent_id)"
    pushwaker_pid="$(read_state_field pushwaker_pid)"

    # T-2316 (arc-004 WP1): tear down the push-waker alongside the heartbeat.
    # T-2319: the waker holds a `channel subscribe … --push` child via process
    # substitution. A bare `kill <pushwaker_pid>` ORPHANS that child — the waker's
    # own SIGTERM trap cannot fire while it is blocked in `read` on the (idle) push
    # stream, so the child survives and loops against the hub forever (T-2314
    # reconnect). cmd_start spawns the waker under `setsid`, making it its own
    # process-group leader (pgid == pid); kill the WHOLE group to reap the waker AND
    # its subscribe child atomically. Fall back to a plain pid-kill if (no setsid)
    # it is not a group leader, so we never signal an unrelated group.
    if pid_alive "$pushwaker_pid"; then
        local pw_pgid pw_is_leader=0
        pw_pgid="$(ps -o pgid= -p "$pushwaker_pid" 2>/dev/null | tr -d ' ')"
        [ -n "$pw_pgid" ] && [ "$pw_pgid" = "$pushwaker_pid" ] && pw_is_leader=1
        if [ "$pw_is_leader" -eq 1 ]; then
            kill -TERM "-${pw_pgid}" 2>/dev/null || true
        else
            kill -TERM "$pushwaker_pid" 2>/dev/null || true
        fi
        local j
        for j in 1 2 3; do
            sleep 1
            pid_alive "$pushwaker_pid" || break
        done
        if pid_alive "$pushwaker_pid"; then
            if [ "$pw_is_leader" -eq 1 ]; then
                kill -KILL "-${pw_pgid}" 2>/dev/null || true
            else
                kill -KILL "$pushwaker_pid" 2>/dev/null || true
            fi
        fi
        echo "be-reachable: stopped push-waker (pid ${pushwaker_pid})."
    fi

    if pid_alive "$pid"; then
        kill -TERM "$pid" 2>/dev/null || true
        # Graceful wait up to 3s.
        local i
        for i in 1 2 3; do
            sleep 1
            pid_alive "$pid" || break
        done
        if pid_alive "$pid"; then
            kill -KILL "$pid" 2>/dev/null || true
            sleep 1
        fi
        echo "be-reachable: stopped ${agent_id:-?} (pid ${pid})."
    else
        echo "be-reachable: state existed but pid ${pid:-?} was already gone; clearing."
    fi
    rm -f "$STATE_FILE"
    exit 0
}

cmd_status() {
    local json=0
    while [ $# -gt 0 ]; do
        case "$1" in
            --json)    json=1; shift ;;
            -h|--help) usage; exit 0 ;;
            *)         die_usage "unknown status arg: $1" ;;
        esac
    done

    if [ ! -f "$STATE_FILE" ]; then
        if [ "$json" -eq 1 ]; then
            echo '{"running": false, "reason": "no state file"}'
        else
            echo "be-reachable: not running."
        fi
        exit 1
    fi

    local pid agent_id started_at role interval pty_session pushwaker_pid
    pid="$(read_state_field pid)"
    agent_id="$(read_state_field agent_id)"
    started_at="$(read_state_field started_at)"
    role="$(read_state_field role)"
    interval="$(read_state_field interval)"
    pty_session="$(read_state_field pty_session)"
    pushwaker_pid="$(read_state_field pushwaker_pid)"

    if pid_alive "$pid"; then
        if [ "$json" -eq 1 ]; then
            # Decorate the existing state file with running=true.
            if command -v jq >/dev/null 2>&1; then
                jq '. + {running: true}' "$STATE_FILE"
            else
                # Append before closing brace.
                sed 's/}[[:space:]]*$/, "running": true}/' "$STATE_FILE"
            fi
        else
            cat <<EOF
be-reachable: running.
  agent_id:    ${agent_id:-?}
  pid:         ${pid}
  started_at:  ${started_at:-?}
  role:        ${role:-?}
  interval:    ${interval:-?}
  pty_session: ${pty_session:-<none>}
  push_waker:  $(if pid_alive "$pushwaker_pid"; then echo "running (pid ${pushwaker_pid})"; else echo "${pushwaker_pid:-<none>}"; fi)
  state:       ${STATE_FILE}
EOF
        fi
        exit 0
    else
        if [ "$json" -eq 1 ]; then
            echo "{\"running\": false, \"reason\": \"stale state\", \"pid\": ${pid:-null}, \"agent_id\": \"${agent_id:-}\"}"
        else
            echo "be-reachable: not running (stale state — pid ${pid:-?} gone). Run 'be-reachable.sh stop' to clear."
        fi
        exit 1
    fi
}

# ---- dispatch ------------------------------------------------------------

if [ $# -eq 0 ]; then
    usage
    exit 0
fi

case "$1" in
    start)     shift; cmd_start "$@" ;;
    stop)     shift; cmd_stop "$@" ;;
    status)   shift; cmd_status "$@" ;;
    -h|--help) usage; exit 0 ;;
    *)        die_usage "unknown subcommand: $1" ;;
esac
