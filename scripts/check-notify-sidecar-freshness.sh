#!/usr/bin/env bash
# T-3051 — notify-sidecar liveness canary (arc-003 deterministic wake rail).
#
# T-3049 measured that rail dark for 82 days and NOTHING noticed. The eighteen
# existing canaries watch hubs, binaries, wakers, claims, queues, topics and
# receipts — every one of them asks whether the FLEET is well. None asks whether
# THIS host can still hear. An agent with dead ears looks perfectly healthy to all
# of them, which is precisely how the rail decayed in silence.
#
# T-3050 added a supervisor that starts and self-heals the sidecar. That is a FIXER,
# not detection, and the difference is load-bearing: the supervisor can fail
# silently — a sidecar that crash-loops is restarted and dies every cycle, a start
# that fails every five minutes looks identical to one that never runs, and if the
# supervisor's own crontab was never installed it does nothing at all. This canary
# reads HEARTBEATS ONLY and never consults the supervisor, so it still fires in each
# of those cases. A canary satisfied by the existence of its fixer detects nothing.
#
# SCOPED TO DECLARED AGENTS, deliberately. ~/.termlink/notify/ still holds July test
# residue (s3g, s3probe, s3smoke, s3t*) whose heartbeats are ~82 days stale and will
# never beat again. A canary that adopted every heartbeat it found would fire forever
# on dead test agents — permanently red, therefore unread (T-2818), which is the same
# disease by a different route.
#
# Two firing classes, reported separately because the remediation differs:
#   DEAD    — declared, heartbeat present, older than --threshold-secs.
#             The listener was running and stopped.
#   MISSING — declared, no heartbeat file at all.
#             It has never run on this host (fresh clone, new agent, or the
#             supervisor has never successfully started it).
#
# Exit: 0 healthy · 1 a declared agent is DEAD or MISSING · 2 tooling error
set -uo pipefail

PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
CONF="${NOTIFY_CANARY_CONF:-$PROJECT_ROOT/.context/cron/notify-sidecar-agents.conf}"
NOTIFY_DIR="${TERMLINK_NOTIFY_DIR:-$HOME/.termlink/notify}"
THRESHOLD="${NOTIFY_CANARY_THRESHOLD_SECS:-300}"
FORMAT=human
QUIET=0

usage() {
    sed -n '3,30p' "$0" | sed 's/^# \{0,1\}//'
    cat <<'EOF'

Usage: check-notify-sidecar-freshness.sh [OPTIONS]
  --threshold-secs N  Heartbeat age at which a declared agent is DEAD (default 300)
  --conf PATH         Declared-agents file
  --notify-dir PATH   Heartbeat directory (default ~/.termlink/notify)
  --json              Emit a JSON envelope
  --quiet             Print only when firing (cron-friendly)
  -h, --help          This help

Default threshold is 300s: the supervisor runs every 5 minutes, so anything it can
fix is fixed before this fires. What survives 300s is what the supervisor could NOT
fix — which is exactly what an operator needs to see.

Test seams (PL-213): NOTIFY_CANARY_CONF, TERMLINK_NOTIFY_DIR,
NOTIFY_CANARY_THRESHOLD_SECS.

Exit: 0 healthy · 1 firing · 2 tooling error
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --threshold-secs) shift; [ $# -ge 1 ] || { echo "notify-canary: --threshold-secs requires a value" >&2; exit 2; }; THRESHOLD="$1" ;;
        --conf)           shift; [ $# -ge 1 ] || { echo "notify-canary: --conf requires a value" >&2; exit 2; }; CONF="$1" ;;
        --notify-dir)     shift; [ $# -ge 1 ] || { echo "notify-canary: --notify-dir requires a value" >&2; exit 2; }; NOTIFY_DIR="$1" ;;
        --json)           FORMAT=json ;;
        --quiet)          QUIET=1 ;;
        -h|--help)        usage; exit 0 ;;
        *) echo "notify-canary: unknown arg: $1" >&2; exit 2 ;;
    esac
    shift
done

case "$THRESHOLD" in ''|*[!0-9]*) echo "notify-canary: --threshold-secs must be an integer" >&2; exit 2 ;; esac
[ -r "$CONF" ] || { echo "notify-canary: declared-agents conf not readable: $CONF" >&2; exit 2; }

export NOTIFY_CANARY_CONF="$CONF" TERMLINK_NOTIFY_DIR="$NOTIFY_DIR"
export NOTIFY_CANARY_THRESHOLD_SECS="$THRESHOLD" NOTIFY_CANARY_FORMAT="$FORMAT"
export NOTIFY_CANARY_QUIET="$QUIET"

python3 - <<'PYEOF'
import json, os, sys, time

CONF      = os.environ["NOTIFY_CANARY_CONF"]
NDIR      = os.environ["TERMLINK_NOTIFY_DIR"]
THRESHOLD = int(os.environ["NOTIFY_CANARY_THRESHOLD_SECS"])
FORMAT    = os.environ.get("NOTIFY_CANARY_FORMAT", "human")
QUIET     = os.environ.get("NOTIFY_CANARY_QUIET") == "1"

NOW = time.time()

declared = []
try:
    with open(CONF, "r", errors="replace") as fh:
        for line in fh:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            declared.append(line.split()[0])
except OSError as e:
    print("notify-canary: cannot read conf: %s" % e, file=sys.stderr)
    sys.exit(2)

# Fail closed. A conf declaring nobody is not a healthy host — it is a checker with
# an empty question, and reporting that as PASS asserts coverage it does not have
# (T-3105 / T-2680). The same applies to an absent notify dir: "I could not look"
# and "I looked and all is well" must never share an exit code.
if not declared:
    print("notify-canary: conf declares zero agents — nothing is being watched: %s" % CONF,
          file=sys.stderr)
    sys.exit(2)
if not os.path.isdir(NDIR):
    print("notify-canary: notify dir does not exist: %s" % NDIR, file=sys.stderr)
    sys.exit(2)

agents = []
for name in declared:
    hb = os.path.join(NDIR, "%s.heartbeat" % name)
    if not os.path.exists(hb):
        agents.append({"agent": name, "state": "MISSING", "age_secs": None})
        continue
    try:
        raw = "".join(c for c in open(hb, errors="replace").read() if c.isdigit())
        age = int(NOW - int(raw) / 1000.0)
    except (OSError, ValueError):
        agents.append({"agent": name, "state": "MISSING", "age_secs": None})
        continue
    agents.append({"agent": name,
                   "state": "DEAD" if age > THRESHOLD else "alive",
                   "age_secs": age})

dead    = [a for a in agents if a["state"] == "DEAD"]
missing = [a for a in agents if a["state"] == "MISSING"]
fire = bool(dead or missing)

if FORMAT == "json":
    print(json.dumps({"ok": not fire, "declared": len(agents),
                      "dead_count": len(dead), "missing_count": len(missing),
                      "threshold_secs": THRESHOLD, "notify_dir": NDIR,
                      "agents": agents}))
    sys.exit(1 if fire else 0)

if QUIET and not fire:
    sys.exit(0)

if dead:
    print("notify-canary: %d declared agent(s) DEAD — the listener stopped beating:" % len(dead))
    for a in dead:
        print("  DEAD: %s (heartbeat %ds old > %ds)" % (a["agent"], a["age_secs"], THRESHOLD))
    print("  This host cannot hear its peers. Messages are NOT being lost — they queue on")
    print("  the hub — but nothing here knows they arrived, and no receipt is being written.")
    print("  The supervisor runs every 5 min, so a DEAD agent is one it could not fix:")
    print("  read .context/working/notify-sidecar-<agent>.log before restarting by hand.")

if missing:
    if dead:
        print("")
    print("notify-canary: %d declared agent(s) MISSING — no heartbeat has ever been written:" % len(missing))
    for a in missing:
        print("  MISSING: %s" % a["agent"])
    print("  Either the supervisor has never successfully started it, or its crontab was")
    print("  never installed. Check: bash scripts/check-cron-install-drift.sh")

if not fire:
    print("notify-canary: healthy (%d declared agent(s), all beating within %ds)"
          % (len(agents), THRESHOLD))

sys.exit(1 if fire else 0)
PYEOF
