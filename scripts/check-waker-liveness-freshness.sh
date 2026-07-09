#!/usr/bin/env bash
# T-2387 — Waker-liveness canary (G-069 shipped≠live guard for the comms rail).
#
# arc-004 push-wake shipped and was live-proven (T-2388), but "0 wakers
# fleet-wide" sat dark for weeks with nothing firing (T-2380 E4/F3) — the
# operator only found out by asking "why is there still no response?". This
# canary is the standing guard: it fires when a host LOOKS reachable but
# cannot actually be push-woken.
#
# Signals (the load-bearing one is T-1834/T-2385: `metadata.pty_session` on the
# agent-presence heartbeat == the waker-running signal):
#   (a) LIVE-but-unwakeable — a LIVE agent-presence listener WITHOUT
#       pty_session: peers can DM it durably but nothing rings its PTY
#       (T-2380 breakpoint #2). Read via scripts/agent-listeners.sh --json.
#   (b) dead-waker — a local ~/.termlink/be-reachable*.state whose recorded
#       pushwaker_pid is not alive (or the pid was recycled by a non-waker
#       process — /proc cmdline guard, T-2239 pattern): the waker was armed
#       and died silently. Clean `be-reachable stop` removes the state file,
#       so a lingering file with a dead waker means silent death, not shutdown.
#   (c) rail-dark (opt-in, --expect-armed) — ZERO LIVE listeners carry
#       pty_session at all: the literal G-069 "0 wakers" state, which (a)
#       alone cannot see (no LIVE listeners -> nothing to flag). Off by
#       default so hosts that legitimately run no agents stay quiet; the
#       .107 cron passes it because agents ARE expected armed there.
#
# Empty output (in --quiet) = healthy — same convention as the other eight
# canaries. /canaries auto-discovers via the cron log + .heartbeat companion.
# An unreachable hub is a tooling/informational condition, NOT a firing one
# (PL-219 — fleet doctor/status already surface down hubs); class (b) still
# runs in that case.
#
# Exit codes:
#   0 — healthy (no firing class)
#   1 — one or more firing classes detected
#   2 — tooling error (parse failure / missing helper)
#
# Usage:
#   check-waker-liveness-freshness.sh                 # human-readable, one-shot
#   check-waker-liveness-freshness.sh --json          # JSON envelope for scripting
#   check-waker-liveness-freshness.sh --quiet         # print only on firing (cron)
#   check-waker-liveness-freshness.sh --expect-armed  # also fire on rail-dark (c)
#   check-waker-liveness-freshness.sh --hub ADDR      # probe a specific hub
#   check-waker-liveness-freshness.sh --no-heartbeat  # suppress heartbeat touch
#
# Test hooks (PL-213 — hub-independent verification):
#   TERMLINK_WAKER_TEST_JSON=<file>   canned agent-listeners.sh --json envelope
#   TERMLINK_WAKER_STATE_DIR=<dir>    fixture dir for be-reachable*.state scan
#   TERMLINK_WAKER_SKIP_PROC=1        skip /proc cmdline guard (fixture pids)

set -eu

SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.waker-liveness-canary.heartbeat}"
STATE_DIR="${TERMLINK_WAKER_STATE_DIR:-${HOME}/.termlink}"
TEST_JSON="${TERMLINK_WAKER_TEST_JSON:-}"

FORMAT=human
QUIET=0
HEARTBEAT=1
EXPECT_ARMED=0
HUB=""

while [ $# -gt 0 ]; do
    case "$1" in
        --json)  FORMAT=json ;;
        --quiet) QUIET=1 ;;
        --no-heartbeat) HEARTBEAT=0 ;;
        --expect-armed) EXPECT_ARMED=1 ;;
        --hub) shift; HUB="${1:-}" ;;
        --hub=*) HUB="${1#*=}" ;;
        -h|--help) sed -n '2,52p' "$0"; exit 0 ;;
        *) echo "unknown arg: $1" >&2; exit 2 ;;
    esac
    shift
done

# Heartbeat first (prove the canary ran even on healthy/error cycles — T-1723).
if [ "$HEARTBEAT" = 1 ]; then
    mkdir -p "$(dirname "$HEARTBEAT_FILE")" 2>/dev/null || true
    touch -- "$HEARTBEAT_FILE" 2>/dev/null || true
fi

# ── Presence read (classes a + c) ────────────────────────────────────────────
# Failure to fetch presence is informational (PL-219), not firing: we record
# presence_error and continue with the local state-file scan (class b).
LISTENERS_JSON=""
PRESENCE_ERROR=""
if [ -n "$TEST_JSON" ]; then
    if [ -r "$TEST_JSON" ]; then
        LISTENERS_JSON="$(cat "$TEST_JSON")"
    else
        echo "waker-liveness canary: test json not readable: $TEST_JSON" >&2
        exit 2
    fi
else
    LISTENERS_SCRIPT="${SELF_DIR}/agent-listeners.sh"
    if [ ! -f "$LISTENERS_SCRIPT" ]; then
        echo "waker-liveness canary: missing helper $LISTENERS_SCRIPT" >&2
        exit 2
    fi
    hub_args=()
    [ -n "$HUB" ] && hub_args=(--hub "$HUB")
    if ! LISTENERS_JSON="$(bash "$LISTENERS_SCRIPT" --json "${hub_args[@]}" 2>/dev/null)"; then
        PRESENCE_ERROR="presence fetch failed (hub unreachable?) — classes (a)/(c) skipped this run"
        LISTENERS_JSON=""
    fi
fi

# ── Classify in python (presence + state files) ──────────────────────────────
REPORT="$(LISTENERS_JSON="$LISTENERS_JSON" python3 - "$STATE_DIR" "$FORMAT" \
    "$EXPECT_ARMED" "$PRESENCE_ERROR" "${TERMLINK_WAKER_SKIP_PROC:-0}" <<'PY'
import sys, os, json, glob

state_dir, fmt = sys.argv[1], sys.argv[2]
expect_armed = (sys.argv[3] == "1")
presence_error = sys.argv[4]
skip_proc = (sys.argv[5] == "1")

# -- presence (classes a + c) -------------------------------------------------
listeners_raw = os.environ.get("LISTENERS_JSON", "").strip()
unwakeable = []          # class (a): LIVE, no pty_session
live_total = 0
live_armed = 0
presence_read = False
if listeners_raw:
    try:
        env = json.loads(listeners_raw)
        presence_read = True
    except Exception:
        presence_error = presence_error or "presence JSON parse failed"
        env = {}
    for l in (env.get("listeners") or []):
        if l.get("status") != "LIVE":
            continue
        live_total += 1
        if l.get("pty_session"):
            live_armed += 1
        else:
            unwakeable.append({
                "agent_id": l.get("agent_id"),
                "identity_fingerprint": (l.get("identity_fingerprint") or "")[:16],
                "age_secs": l.get("age_secs"),
                "host": l.get("host"),
            })

rail_dark = bool(presence_read and expect_armed and live_armed == 0)  # class (c)

# -- local state files (class b) ---------------------------------------------
def pid_alive(pid):
    try:
        os.kill(int(pid), 0)
        return True
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    except Exception:
        return False

def pid_is_waker(pid):
    # Guard pid-recycle: a dead waker's pid may be reused by an unrelated
    # process. Returns True/False, or None when /proc is unreadable
    # (non-Linux) — caller falls back to liveness-only.
    try:
        with open("/proc/%d/cmdline" % int(pid), "rb") as f:
            cmd = f.read().replace(b"\x00", b" ").decode("utf8", "replace").lower()
        return "pushwaker" in cmd
    except FileNotFoundError:
        return False   # pid raced away → gone
    except Exception:
        return None

dead_wakers = []         # class (b)
armed_states = 0
dormant_states = 0       # state present but pushwaker_pid null (PL-237 dormant;
                         # class (a) covers it via presence, so informational here)
for path in sorted(glob.glob(os.path.join(state_dir, "be-reachable*.state"))):
    try:
        st = json.load(open(path))
    except Exception:
        continue
    pw = st.get("pushwaker_pid")
    if pw in (None, "", "null"):
        dormant_states += 1
        continue
    armed_states += 1
    alive = pid_alive(pw)
    if alive and not skip_proc:
        is_waker = pid_is_waker(pw)
        if is_waker is False:
            alive = False    # recycled pid — the real waker is gone
    if not alive:
        dead_wakers.append({
            "state_file": path,
            "agent_id": st.get("agent_id"),
            "pushwaker_pid": pw,
            "pty_session": st.get("pty_session"),
        })

fire = len(unwakeable) + len(dead_wakers) + (1 if rail_dark else 0)

if fmt == "json":
    print(json.dumps({
        "ok": fire == 0,
        "expect_armed": expect_armed,
        "presence_read": presence_read,
        "presence_error": presence_error or None,
        "live_total": live_total,
        "live_armed": live_armed,
        "rail_dark": rail_dark,
        "unwakeable": unwakeable,
        "dead_wakers": dead_wakers,
        "armed_states": armed_states,
        "dormant_states": dormant_states,
        "state_dir": state_dir,
    }))
else:
    if fire:
        print("waker-liveness canary: FIRING — %d unwakeable LIVE agent(s), %d dead waker(s)%s"
              % (len(unwakeable), len(dead_wakers), ", RAIL DARK" if rail_dark else ""))
        for u in unwakeable:
            print("  [LIVE-no-waker] %s  fp=%s  age=%ss  host=%s — presence-advertised but"
                  % (u["agent_id"], u["identity_fingerprint"], u["age_secs"], u["host"] or "?"))
            print("      nothing can ring it (no pty_session on heartbeat). Relaunch via:")
            print("      bash scripts/tl-claude.sh start --reachable --agent-id %s -- --resume" % u["agent_id"])
        for d in dead_wakers:
            print("  [dead-waker] agent=%s  pid=%s  state=%s — waker died silently after"
                  % (d["agent_id"], d["pushwaker_pid"], d["state_file"]))
            print("      arming. Re-arm: bash scripts/be-reachable.sh start --agent-id %s --pty-session %s"
                  % (d["agent_id"], d["pty_session"] or "<pty>"))
        if rail_dark:
            print("  [rail-dark] ZERO LIVE listeners carry pty_session on this hub — the G-069")
            print("      '0 wakers' state. Every DM sent here waits on the ~15s poll floor at")
            print("      best, forever at worst. Arm agents via the T-2388 launcher (see above).")
        if presence_error:
            print("  (note: %s)" % presence_error)
    print("FIRE=%d" % fire)
PY
)" || { echo "waker-liveness canary: classify error" >&2; exit 2; }

FIRE="$(printf '%s\n' "$REPORT" | sed -n 's/^FIRE=//p' | tail -1)"
BODY="$(printf '%s\n' "$REPORT" | grep -v '^FIRE=' || true)"

if [ "${FIRE:-0}" = 0 ]; then
    if [ "$QUIET" != 1 ]; then
        if [ "$FORMAT" = json ]; then
            printf '%s\n' "$BODY"
        else
            note=""
            [ -n "$PRESENCE_ERROR" ] && note=" (note: ${PRESENCE_ERROR})"
            echo "waker-liveness canary: healthy — no unwakeable LIVE agents, no dead wakers${note}"
        fi
    fi
    exit 0
fi

# Firing — always print (including --quiet, so the cron log captures it).
printf '%s\n' "$BODY"
exit 1
