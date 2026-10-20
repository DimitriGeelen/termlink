#!/usr/bin/env bash
# telemetry-rollup.sh — T-3466 (R5b, T-2250 GO): roll up one UTC day of this host's
# local telemetry into ONE aggregate and post it over TermLink to a bounded collection
# topic, so every host's view is collected and not only .107's.
#
# Raw data never leaves the host (design T-2250 §4 Tier 0); only the aggregate does.
# The post goes through `termlink channel post`, i.e. the durable offline queue: a hub
# that is down queues the aggregate, a rejected one is dead-lettered (T-2243) — never
# silently lost.
#
# Inputs (each read for the chosen day only; T-3465 bounds them by size, not age):
#   ~/.termlink/queue.log                      queue flaps   {kind: pending|drained, ...}
#   ~/.termlink/reconnect.log                  reconnects    {signal: reconnect, downtime_ms, ...}
#   <runtime_dir>/circuit-breaker.jsonl        breaker trips {signal: breaker_trip, state, open_ms}
#   .context/working/.hub-outage-samples.ndjson  T-3383 probes (only on hosts that run it)
#   `termlink channel queue-status --json`     dead_letters  (operator ruling B, 2026-10-20)
# A source that does not exist is reported as "absent", never as zero.
#
# Usage: telemetry-rollup.sh [--day YYYY-MM-DD] [--hub ADDR] [--topic T] [--host NAME]
#                            [--dry-run] [--json]
#   --day      UTC day to roll up (default: yesterday)
#   --hub      collector hub (default: $TELEMETRY_HUB, else the local hub)
#   --topic    collection topic (default telemetry:daily, retention days:400)
#   --host     host label (default: $TELEMETRY_HOST, else `hostname`)
#   --dry-run  print the aggregate, post nothing
# Exit: 0 posted (or dry-run) · 1 the post failed outright · 2 tooling / bad usage.
#
# Test seams (PL-213): TELEMETRY_QUEUE_LOG, TELEMETRY_RECONNECT_LOG, TELEMETRY_BREAKER_LOG,
# TELEMETRY_OUTAGE_SAMPLES, TELEMETRY_QUEUE_STATUS_JSON (canned queue-status output),
# TELEMETRY_NOW (epoch seconds), TELEMETRY_TERMLINK (binary).
set -euo pipefail

DAY="" HUB="${TELEMETRY_HUB:-}" TOPIC="telemetry:daily" HOST="${TELEMETRY_HOST:-}" DRY=0 JSON=0
while [ $# -gt 0 ]; do
    case "$1" in
        --day) DAY="${2:?--day needs YYYY-MM-DD}"; shift 2 ;;
        --hub) HUB="${2:?--hub needs an address}"; shift 2 ;;
        --topic) TOPIC="${2:?--topic needs a name}"; shift 2 ;;
        --host) HOST="${2:?--host needs a name}"; shift 2 ;;
        --dry-run) DRY=1; shift ;;
        --json) JSON=1; shift ;;
        -h|--help) sed -n '2,32p' "$0"; exit 0 ;;
        *) echo "telemetry-rollup: unknown argument: $1" >&2; exit 2 ;;
    esac
done

command -v python3 >/dev/null || { echo "telemetry-rollup: python3 not found" >&2; exit 2; }
TL="${TELEMETRY_TERMLINK:-termlink}"
[ -n "$HOST" ] || HOST="$(hostname)"

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUNTIME_DIR="${TERMLINK_RUNTIME_DIR:-}"
if [ -z "$RUNTIME_DIR" ]; then
    if [ -d /var/lib/termlink ]; then RUNTIME_DIR=/var/lib/termlink; else RUNTIME_DIR="/tmp/termlink-$(id -u)"; fi
fi
QUEUE_LOG="${TELEMETRY_QUEUE_LOG:-$HOME/.termlink/queue.log}"
RECONNECT_LOG="${TELEMETRY_RECONNECT_LOG:-$HOME/.termlink/reconnect.log}"
BREAKER_LOG="${TELEMETRY_BREAKER_LOG:-$RUNTIME_DIR/circuit-breaker.jsonl}"
OUTAGE_SAMPLES="${TELEMETRY_OUTAGE_SAMPLES:-$PROJECT_ROOT/.context/working/.hub-outage-samples.ndjson}"

if [ -n "${TELEMETRY_QUEUE_STATUS_JSON:-}" ]; then
    QSTATUS="$(cat "$TELEMETRY_QUEUE_STATUS_JSON")"
else
    QSTATUS="$(timeout 15 "$TL" channel queue-status --json 2>/dev/null || true)"
fi

AGG="$(QSTATUS="$QSTATUS" python3 - "$DAY" "$HOST" "$QUEUE_LOG" "$RECONNECT_LOG" "$BREAKER_LOG" "$OUTAGE_SAMPLES" <<'PY'
import json, os, sys, datetime as dt

day_arg, host, qlog, rlog, blog, samples = sys.argv[1:7]
now = float(os.environ.get("TELEMETRY_NOW") or dt.datetime.now(dt.timezone.utc).timestamp())
if day_arg:
    try:
        day = dt.date.fromisoformat(day_arg)
    except ValueError:
        print(f"telemetry-rollup: bad --day {day_arg!r}", file=sys.stderr); sys.exit(2)
else:
    day = dt.datetime.fromtimestamp(now, dt.timezone.utc).date() - dt.timedelta(days=1)
start = dt.datetime(day.year, day.month, day.day, tzinfo=dt.timezone.utc).timestamp()
end = start + 86400

def parse_ts(v):
    if isinstance(v, (int, float)):
        return v / 1000 if v > 1e11 else float(v)
    if isinstance(v, str):
        try:
            return dt.datetime.fromisoformat(v.replace("Z", "+00:00")).timestamp()
        except ValueError:
            return None
    return None

def day_lines(path):
    """Lines of `path` and its `.1` backup whose ts falls in the day. None = source absent."""
    found, out, bad = False, [], 0
    for p in (path + ".1", path):
        if not os.path.exists(p):
            continue
        found = True
        with open(p, errors="replace") as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    d = json.loads(line)
                except ValueError:
                    bad += 1; continue
                t = parse_ts(d.get("ts"))
                if t is not None and start <= t < end:
                    out.append((t, d))
    return (sorted(out, key=lambda x: x[0]), bad) if found else (None, 0)

signals, sources = {}, {}

# Queue flaps: count pending/drained, the deepest backlog, and the longest pending spell.
rows, bad = day_lines(qlog)
sources["queue_log"] = "absent" if rows is None else "present"
if rows is not None:
    pend = [d for _, d in rows if d.get("kind") == "pending"]
    drn = [d for _, d in rows if d.get("kind") == "drained"]
    longest, since = 0.0, None
    for t, d in rows:
        if d.get("kind") == "pending" and since is None:
            since = t
        elif d.get("kind") == "drained" and since is not None:
            longest, since = max(longest, t - since), None
    signals["queue"] = {
        "pending_events": len(pend), "drained_events": len(drn),
        "max_pending": max([int(d.get("new_pending") or 0) for d in pend], default=0),
        "max_backlog_s": round(longest), "still_pending_at_day_end": since is not None,
        "malformed_lines": bad,
    }

# Reconnects: count and longest downtime.
rows, bad = day_lines(rlog)
sources["reconnect_log"] = "absent" if rows is None else "present"
if rows is not None:
    rc = [d for _, d in rows if d.get("signal") == "reconnect"]
    signals["reconnect"] = {
        "count": len(rc),
        "max_downtime_s": round(max([int(d.get("downtime_ms") or 0) for d in rc], default=0) / 1000),
        "peers": sorted({str(d.get("peer")) for d in rc}),
        "malformed_lines": bad,
    }

# Breaker trips: count per state and the longest open spell reported.
rows, bad = day_lines(blog)
sources["breaker_log"] = "absent" if rows is None else "present"
if rows is not None:
    bt = [d for _, d in rows if d.get("signal") == "breaker_trip"]
    by = {}
    for d in bt:
        by[str(d.get("state"))] = by.get(str(d.get("state")), 0) + 1
    signals["breaker"] = {
        "trips": len(bt), "by_state": by,
        "max_open_s": round(max([int(d.get("open_ms") or 0) for d in bt], default=0) / 1000),
        "malformed_lines": bad,
    }

# Hub outages seen by this host's T-3383 probes: outages and longest downtime per hub.
if os.path.exists(samples):
    sources["hub_outage_samples"] = "present"
    per = {}
    with open(samples, errors="replace") as f:
        for line in f:
            try:
                d = json.loads(line)
            except ValueError:
                continue
            t = parse_ts(d.get("ts"))
            if t is None or not (start <= t < end):
                continue
            per.setdefault(str(d.get("hub")), []).append((t, bool(d.get("reachable"))))
    hubs = {}
    for hub, s in per.items():
        s.sort()
        outages, longest, down_since = 0, 0.0, None
        for t, ok in s:
            if not ok and down_since is None:
                down_since, outages = t, outages + 1
            elif ok and down_since is not None:
                longest, down_since = max(longest, t - down_since), None
        if down_since is not None:
            longest = max(longest, s[-1][0] - down_since)
        hubs[hub] = {"samples": len(s), "outages": outages, "max_downtime_s": round(longest)}
    signals["hub_outages"] = hubs
else:
    sources["hub_outage_samples"] = "absent"

# Dead letters: a point-in-time count (the table has no per-day history).
try:
    q = json.loads(os.environ.get("QSTATUS") or "")
    signals["dead_letters"] = {"count_at_rollup": int(q.get("dead_letters") or 0)}
    sources["queue_status"] = "present"
except ValueError:
    sources["queue_status"] = "unreadable"

print(json.dumps({
    "schema": "r5-rollup/1", "host": host, "day": day.isoformat(),
    "generated_at": dt.datetime.fromtimestamp(now, dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "signals": signals, "sources": sources,
}, sort_keys=True))
PY
)" || exit $?

if [ "$DRY" = 1 ]; then
    if [ "$JSON" = 1 ]; then echo "$AGG"; else echo "$AGG" | python3 -m json.tool; fi
    exit 0
fi

HUB_ARGS=()
[ -n "$HUB" ] && HUB_ARGS=(--hub "$HUB")
# Idempotent: an existing topic is fine. Bounded on purpose (T-2250 §4: never forever-raw,
# never latest-per-cv-key, which would destroy the series).
timeout 20 "$TL" channel create "$TOPIC" --retention days:400 \
    --reason "T-3466 daily telemetry aggregates, one per host per day" "${HUB_ARGS[@]}" >/dev/null 2>&1 || true

DAY_OUT="$(python3 -c 'import json,sys; print(json.loads(sys.argv[1])["day"])' "$AGG")"
if OUT="$(timeout 30 "$TL" channel post "$TOPIC" --msg-type telemetry-rollup --payload "$AGG" \
        --metadata "host=$HOST" --metadata "day=$DAY_OUT" "${HUB_ARGS[@]}" 2>&1)"; then
    echo "telemetry-rollup: $HOST $DAY_OUT -> $TOPIC: ${OUT:-posted}"
    exit 0
fi
echo "telemetry-rollup: post failed for $HOST $DAY_OUT -> $TOPIC: $OUT" >&2
exit 1
