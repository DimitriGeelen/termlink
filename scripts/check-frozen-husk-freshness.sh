#!/usr/bin/env bash
# T-2239 — Frozen-husk regression canary (G-019 prevention for T-2230/T-2235).
#
# The T-2230/T-2235 arc fixed the *symptom* of "frozen husk" sessions: a process
# that calls `termlink register` (or `register --self`) and never advances its
# `heartbeat_at`. Before that arc, an ALIVE session could register once and then
# sit forever with a stale heartbeat, and NOTHING in the framework surfaced it.
# This canary closes that detection gap so a regression (or a field host still
# running a pre-fix binary) is caught instead of silently rotting.
#
# Model: walk the local runtime_dir's session registrations
# ($TERMLINK_RUNTIME_DIR/sessions/*.json). A "frozen husk" is a session whose
#   - pid is ALIVE (kill -0 succeeds), AND
#   - heartbeat_at is older than --threshold-secs (default 600; the heartbeat
#     interval is 30s, so 600s is ~20 missed beats — well past any GC/scheduling
#     slack and immune to flapping).
# A live process that has stopped (or never started) heartbeating is exactly the
# bug class T-2230/T-2235 fixed; post-fix it must never appear.
#
# Dead-pid registrations (process gone, JSON left behind) are a DIFFERENT class
# (orphan cruft → deregister/cleanup, not the heartbeat bug). They are counted
# and reported as informational context but do NOT fire the canary — gating on
# them would be noisy (the sessions dir accumulates dead registrations over time).
#
# Empty output (in --quiet) = healthy — same convention as the mirror /
# substrate-preflight / framework-pickup canaries. /canaries auto-discovers this
# canary via the .heartbeat companion + the cron log.
#
# Exit codes:
#   0  — healthy (no live frozen husks)
#   1  — one or more live frozen husks detected
#   2  — tooling error (no runtime dir / parse failure)
#
# Usage:
#   check-frozen-husk-freshness.sh                  # human-readable, one-shot
#   check-frozen-husk-freshness.sh --json           # JSON envelope for scripting
#   check-frozen-husk-freshness.sh --quiet          # print only on firing (cron)
#   check-frozen-husk-freshness.sh --threshold-secs N   # staleness gate (default 600)
#   check-frozen-husk-freshness.sh --runtime-dir PATH   # override sessions root
#   check-frozen-husk-freshness.sh --regressions-only   # fire ONLY on post-fix regressions
#   check-frozen-husk-freshness.sh --no-heartbeat   # suppress heartbeat touch
#
# Husk classes (T-2240): each husk is classified by its registered
# termlink_version against the heartbeat-fix threshold (>= 0.11.1359):
#   - REGRESSION: binary HAS the fix yet heartbeat froze anyway — the alarming
#     case this canary exists to catch.
#   - pre-fix:    binary predates the fix (e.g. v0.9.0) or version unknown — a
#     frozen heartbeat is EXPECTED; remediation is a binary upgrade (known
#     upgrade-backlog, not an incident).
# Default fires (exit 1) on ANY husk (back-compat with T-2239). --regressions-only
# fires only on REGRESSION husks and treats pre-fix husks as informational
# (exit 0) — use it for cron so the daily log accumulates only on genuine
# regressions, keeping "empty log = healthy" meaningful while a fleet still has
# old binaries in the field.

set -eu

RUNTIME_DIR="${TERMLINK_RUNTIME_DIR:-/var/lib/termlink}"
HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.frozen-husk-canary.heartbeat}"
THRESHOLD_SECS=600
# Version at/after which a frozen heartbeat is a genuine regression (T-2230/T-2235
# landed at 0.11.1359). Tuple-compared in python.
FIX_VERSION="${FROZEN_HUSK_FIX_VERSION:-0.11.1359}"

FORMAT=human
QUIET=0
HEARTBEAT=1
REGRESSIONS_ONLY=0

while [ $# -gt 0 ]; do
    case "$1" in
        --json)  FORMAT=json ;;
        --quiet) QUIET=1 ;;
        --no-heartbeat) HEARTBEAT=0 ;;
        --regressions-only) REGRESSIONS_ONLY=1 ;;
        --threshold-secs) shift; THRESHOLD_SECS="${1:-600}" ;;
        --threshold-secs=*) THRESHOLD_SECS="${1#*=}" ;;
        --runtime-dir) shift; RUNTIME_DIR="${1:-/var/lib/termlink}" ;;
        --runtime-dir=*) RUNTIME_DIR="${1#*=}" ;;
        -h|--help) sed -n '2,57p' "$0"; exit 0 ;;
        *) echo "unknown arg: $1" >&2; exit 2 ;;
    esac
    shift
done

# Heartbeat first (prove the canary ran even on healthy/error cycles — T-1723).
if [ "$HEARTBEAT" = 1 ]; then
    mkdir -p "$(dirname "$HEARTBEAT_FILE")" 2>/dev/null || true
    touch -- "$HEARTBEAT_FILE" 2>/dev/null || true
fi

SESS_DIR="$RUNTIME_DIR/sessions"
if [ ! -d "$SESS_DIR" ]; then
    # No sessions dir = no registrations on this host. Healthy (nothing to rot).
    if [ "$QUIET" != 1 ]; then
        if [ "$FORMAT" = json ]; then
            printf '{"ok": true, "runtime_dir": "%s", "reason": "no sessions dir", "husks": [], "dead_orphans": 0, "regression_count": 0, "prefix_count": 0}\n' "$RUNTIME_DIR"
        else
            echo "frozen-husk canary: healthy — no sessions dir at $SESS_DIR (nothing registered)"
        fi
    fi
    exit 0
fi

# Parse all registration JSONs in python: classify each as frozen-husk / dead /
# fresh. pid liveness is checked here via os.kill(pid, 0). Emits the rendered
# report on stdout and a trailing "FIRE=<n>" sentinel line for the shell.
REPORT="$(python3 - "$SESS_DIR" "$THRESHOLD_SECS" "$FORMAT" "$RUNTIME_DIR" "$FIX_VERSION" "$REGRESSIONS_ONLY" <<'PY'
import sys, os, json, glob, time

sess_dir, threshold, fmt, runtime_dir = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
fix_version, regressions_only = sys.argv[5], (sys.argv[6] == "1")
now = int(time.time())

def version_tuple(v):
    # "0.11.1367" -> (0, 11, 1367). Non-numeric / missing -> None (treated pre-fix).
    if not isinstance(v, str):
        return None
    parts = v.strip().split(".")
    out = []
    for p in parts:
        try:
            out.append(int(p))
        except Exception:
            return None
    return tuple(out) if out else None

FIX_T = version_tuple(fix_version) or (0, 11, 1359)

def classify(ver):
    # "regression" = binary has the fix but heartbeat froze anyway (alarming).
    # "pre-fix"    = binary predates the fix or version unknown (expected; upgrade).
    vt = version_tuple(ver)
    if vt is None:
        return "pre-fix"
    return "regression" if vt >= FIX_T else "pre-fix"

def parse_ts(v):
    # heartbeat_at / created_at serialize as "<secs>Z"
    if not isinstance(v, str):
        return None
    s = v.strip().rstrip('Z')
    try:
        return int(s)
    except Exception:
        return None

def pid_alive(pid):
    try:
        os.kill(int(pid), 0)
        return True
    except ProcessLookupError:
        return False
    except PermissionError:
        return True   # exists, owned by someone else
    except Exception:
        return False

def pid_is_termlink(pid):
    # Guard against pid-recycle false positives: a dead session's pid may have
    # been reused by an unrelated process. Confirm the live pid is actually a
    # termlink process before calling it a husk. On Linux, read /proc cmdline.
    # If /proc is unavailable (non-Linux) we cannot tell — return None so the
    # caller falls back to liveness-only (best-effort, documented).
    try:
        with open("/proc/%d/cmdline" % int(pid), "rb") as f:
            cmd = f.read().replace(b"\x00", b" ").decode("utf8", "replace").lower()
        return "termlink" in cmd
    except FileNotFoundError:
        return False   # /proc exists but pid gone (raced) → not a husk
    except Exception:
        return None    # /proc unreadable → undecidable, fall back to liveness

husks = []
dead = 0
total = 0
for path in sorted(glob.glob(os.path.join(sess_dir, "*.json"))):
    try:
        m = json.load(open(path))
    except Exception:
        continue
    total += 1
    pid = m.get("pid")
    hb = parse_ts(m.get("heartbeat_at"))
    created = parse_ts(m.get("created_at"))
    if pid is None or hb is None:
        continue
    age = now - hb
    alive = pid_alive(pid)
    if not alive:
        if age > threshold:
            dead += 1
        continue
    # alive — but is it really termlink, or a recycled pid?
    is_tl = pid_is_termlink(pid)
    if is_tl is False:
        # live pid, but NOT a termlink process → recycled pid, the real session
        # is gone. Treat as a dead orphan, not a frozen husk.
        if age > threshold:
            dead += 1
        continue
    # is_tl True (confirmed termlink) or None (undecidable → best-effort liveness)
    if age > threshold:
        ver = (m.get("metadata", {}) or {}).get("termlink_version")
        husks.append({
            "id": m.get("id"),
            "display_name": m.get("display_name"),
            "pid": pid,
            "heartbeat_age_secs": age,
            "never_advanced": (created is not None and created == hb),
            "termlink_version": ver,
            "class": classify(ver),
        })

husks.sort(key=lambda h: -h["heartbeat_age_secs"])
regressions = [h for h in husks if h["class"] == "regression"]
prefix = [h for h in husks if h["class"] == "pre-fix"]
# Firing count depends on mode: --regressions-only fires only on regressions.
fire_count = len(regressions) if regressions_only else len(husks)

if fmt == "json":
    print(json.dumps({
        "ok": fire_count == 0,
        "runtime_dir": runtime_dir,
        "threshold_secs": threshold,
        "fix_version": fix_version,
        "regressions_only": regressions_only,
        "total_registrations": total,
        "dead_orphans": dead,
        "regression_count": len(regressions),
        "prefix_count": len(prefix),
        "husks": husks,
    }))
else:
    if husks:
        print("frozen-husk canary: %d live frozen husk(s) — %d REGRESSION, %d pre-fix (heartbeat stale > %ds)" % (
            len(husks), len(regressions), len(prefix), threshold))
        for h in husks:
            na = " [heartbeat NEVER advanced]" if h["never_advanced"] else ""
            ver = h["termlink_version"] or "?"
            label = "[REGRESSION]" if h["class"] == "regression" else ("[pre-fix v%s — upgrade]" % ver)
            print("  %s %s  (%s)  pid=%s  age=%ds  v%s%s" % (
                label, h["id"], h["display_name"], h["pid"], h["heartbeat_age_secs"], ver, na))
        if regressions:
            print("  → REGRESSION: binary >= %s HAS the heartbeat fix but the heartbeat froze" % fix_version)
            print("    anyway — investigate as a genuine T-2230/T-2235 regression (file a bug task).")
        if prefix:
            print("  → pre-fix: binary predates %s; a frozen heartbeat is expected. Upgrade the" % fix_version)
            print("    host binary (>= %s) and re-register, or terminate+deregister (termlink deregister <id>)." % fix_version)
        if dead:
            print("  (also %d dead-pid orphan registration(s) — cleanup, non-firing)" % dead)
    # healthy text printed by the shell below (needs QUIET awareness)
print("FIRE=%d" % fire_count)
print("HUSKS=%d" % len(husks))
PY
)" || { echo "frozen-husk canary: parse error" >&2; exit 2; }

FIRE="$(printf '%s\n' "$REPORT" | sed -n 's/^FIRE=//p' | tail -1)"
HUSKS="$(printf '%s\n' "$REPORT" | sed -n 's/^HUSKS=//p' | tail -1)"
BODY="$(printf '%s\n' "$REPORT" | grep -v -e '^FIRE=' -e '^HUSKS=' || true)"

if [ "${FIRE:-0}" = 0 ]; then
    # Not firing. Two sub-cases when not quiet:
    #   - genuinely no husks → healthy message
    #   - husks present but none fire (--regressions-only, only pre-fix husks) →
    #     show them as informational upgrade-backlog (still exit 0, cron log stays empty)
    if [ "$QUIET" != 1 ]; then
        if [ "$FORMAT" = json ]; then
            printf '%s\n' "$BODY"
        elif [ "${HUSKS:-0}" != 0 ]; then
            printf '%s\n' "$BODY"
            echo "frozen-husk canary: no regressions (exit 0) — above are pre-fix upgrade-backlog only."
        else
            echo "frozen-husk canary: healthy — no live frozen husks under $SESS_DIR"
        fi
    fi
    exit 0
fi

# Firing — always print (including --quiet, so the cron log captures it).
printf '%s\n' "$BODY"
exit 1
