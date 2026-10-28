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
# T-3615 adds two aggregates (counts only; no raw row, argument or content leaves the host):
#   usage           <runtime_dir>/invocation-audit.jsonl (+ .1), plus any stray sink at the old
#                   fallback paths (/tmp, $TMPDIR, /var/tmp, $XDG_RUNTIME_DIR termlink dirs; each
#                   file read once, deduplicated by real path): counts per class x surface x name,
#                   the polling verbs in a separate `polling` block; rows written before
#                   attribution shipped count under class "unattributed".
#   worker_outcomes .context/dispatch/*/ and /tmp/tl-dispatch/*/ (meta.json, exit_code, timed_out,
#                   result.md, finished_at, worker_kind.txt) per class/kind/model;
#                   <runtime_dir>/dispatch-close-states.jsonl per model/task_type;
#                   .context/reviews/verdicts.jsonl + applied.jsonl (recorded, applied, withdrawn);
#                   standing workers: systemd NRestarts of termlink-sidecar@*, else "not measured".
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
# T-3615 seams: TELEMETRY_INVOCATION_SINKS (colon list, replaces the sink search),
# TELEMETRY_DISPATCH_DIRS (colon list of dispatch roots), TELEMETRY_CLOSE_STATES,
# TELEMETRY_VERDICTS, TELEMETRY_APPLIED, TELEMETRY_STANDING_UNITS_JSON (canned
# [{"unit","nrestarts","active"}] list; "none" = no systemd, i.e. not measured).
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

# T-3615: usage sinks — the host sink first, then the stray fallback sinks callers without
# TERMLINK_RUNTIME_DIR wrote to before the one-sink rule shipped.
UID_="$(id -u)"
if [ -n "${TELEMETRY_INVOCATION_SINKS+x}" ]; then
    INV_SINKS="$TELEMETRY_INVOCATION_SINKS"
else
    INV_SINKS="$RUNTIME_DIR/invocation-audit.jsonl"
    for d in "/tmp/termlink-$UID_" "${TMPDIR:+$TMPDIR/termlink-$UID_}" "/var/tmp/termlink-$UID_" \
             "${XDG_RUNTIME_DIR:+$XDG_RUNTIME_DIR/termlink}"; do
        [ -n "$d" ] && INV_SINKS="$INV_SINKS:$d/invocation-audit.jsonl"
    done
fi
DISPATCH_DIRS="${TELEMETRY_DISPATCH_DIRS-$PROJECT_ROOT/.context/dispatch:/tmp/tl-dispatch}"
CLOSE_STATES="${TELEMETRY_CLOSE_STATES:-$RUNTIME_DIR/dispatch-close-states.jsonl}"
VERDICTS="${TELEMETRY_VERDICTS:-$PROJECT_ROOT/.context/reviews/verdicts.jsonl}"
APPLIED="${TELEMETRY_APPLIED:-$PROJECT_ROOT/.context/reviews/applied.jsonl}"
if [ -n "${TELEMETRY_STANDING_UNITS_JSON:-}" ]; then
    STANDING="$(cat "$TELEMETRY_STANDING_UNITS_JSON" 2>/dev/null || echo none)"
elif command -v systemctl >/dev/null 2>&1 && [ -d /run/systemd/system ]; then
    STANDING="$(systemctl list-units --all --plain --no-legend 'termlink-sidecar@*' 2>/dev/null | awk '{print $1}' |
        while read -r u; do
            [ -n "$u" ] || continue
            printf '{"unit":"%s","nrestarts":"%s","active":"%s"}\n' "$u" \
                "$(systemctl show "$u" -p NRestarts --value 2>/dev/null)" \
                "$(systemctl show "$u" -p ActiveState --value 2>/dev/null)"
        done | python3 -c 'import json,sys; print(json.dumps([json.loads(l) for l in sys.stdin if l.strip()]))' 2>/dev/null || echo none)"
else
    STANDING="none"
fi
export INV_SINKS DISPATCH_DIRS CLOSE_STATES VERDICTS APPLIED STANDING

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

# ── T-3615: usage aggregate ─────────────────────────────────────────────────
# Polling verbs: read loops whose volume says how often something WAITS, not what it uses.
POLLING = {
    "cli": {"channel unread", "channel subscribe", "channel state", "pty output", "ping", "list",
            "event wait", "hub status"},
    "mcp": {"termlink_channel_unread", "termlink_channel_subscribe", "termlink_channel_state",
            "termlink_output", "termlink_ping", "termlink_list_sessions", "termlink_wait",
            "termlink_hub_status"},
}
seen_real, sinks_read = set(), []
usage, polling, totals, bad_usage = {}, {}, {}, 0
for base in [x for x in os.environ.get("INV_SINKS", "").split(":") if x]:
    for p in (base + ".1", base):
        if not os.path.exists(p):
            continue
        real = os.path.realpath(p)
        if real in seen_real:
            continue
        seen_real.add(real)
        n = 0
        with open(p, errors="replace") as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    d = json.loads(line)
                except ValueError:
                    bad_usage += 1; continue
                t = parse_ts(d.get("ts"))
                if t is None or not (start <= t < end):
                    continue
                n += 1
                cls = str(d.get("class") or "unattributed")
                surf, name = str(d.get("surface")), str(d.get("name"))
                block = polling if name in POLLING.get(surf, ()) else usage
                key = f"{surf}:{name}"
                block.setdefault(cls, {})[key] = block.setdefault(cls, {}).get(key, 0) + 1
                tc = totals.setdefault(cls, {"calls": 0, "polling": 0})
                tc["polling" if block is polling else "calls"] += 1
        sinks_read.append({"path": p, "rows_in_day": n})
if sinks_read:
    sources["invocation_audit"] = "present"
    signals["usage"] = {"by_class": usage, "polling": polling, "totals": totals,
                        "sinks": sinks_read, "malformed_lines": bad_usage}
else:
    sources["invocation_audit"] = "absent"

# ── T-3615: worker-outcome aggregate ────────────────────────────────────────
def read_text(p):
    try:
        with open(p, errors="replace") as f:
            return f.read().strip()
    except OSError:
        return None

outcomes, dmeta, dirs_seen = {}, {}, False
for root in [x for x in os.environ.get("DISPATCH_DIRS", "").split(":") if x]:
    if not os.path.isdir(root):
        continue
    dirs_seen = True
    for name in sorted(os.listdir(root)):
        w = os.path.join(root, name)
        mp = os.path.join(w, "meta.json")
        if not os.path.isfile(mp):
            continue
        try:
            meta = json.loads(read_text(mp) or "")
        except ValueError:
            continue
        kind = read_text(os.path.join(w, "worker_kind.txt")) or "claude"
        model = str(meta.get("model_used") or meta.get("model") or "unknown")
        cls = "reviewer" if meta.get("task_type") == "review" else "dispatched"
        dmeta[name] = (cls, kind, model)
        fin = parse_ts(read_text(os.path.join(w, "finished_at")) or meta.get("ended"))
        if fin is None or not (start <= fin < end):
            continue
        o = outcomes.setdefault(f"{cls}|{kind}|{model}", {
            "class": cls, "kind": kind, "model": model, "runs": 0, "exit_codes": {},
            "timeouts": 0, "empty_results": 0, "close_states": {}})
        o["runs"] += 1
        ec = read_text(os.path.join(w, "exit_code"))
        ec = ec if ec not in (None, "") else str(meta.get("exit_code", "none"))
        o["exit_codes"][ec] = o["exit_codes"].get(ec, 0) + 1
        if os.path.exists(os.path.join(w, "timed_out")):
            o["timeouts"] += 1
        rm = os.path.join(w, "result.md")
        if not os.path.exists(rm) or os.path.getsize(rm) == 0:
            o["empty_results"] += 1
        cs = str(meta.get("close_state") or read_text(os.path.join(w, "close_state")) or "none")
        o["close_states"][cs] = o["close_states"].get(cs, 0) + 1
sources["dispatch_dirs"] = "present" if dirs_seen else "absent"

close_ledger = None
cpath = os.environ.get("CLOSE_STATES", "")
if cpath and os.path.exists(cpath):
    sources["close_states"] = "present"
    close_ledger = {}
    rows, _ = day_lines(cpath)
    for _, d in rows or []:
        k = f"{d.get('model')}|{d.get('task_type')}"
        c = close_ledger.setdefault(k, {"model": d.get("model"), "task_type": d.get("task_type"),
                                        "runs": 0, "exit_codes": {}, "outcomes": {}})
        c["runs"] += 1
        ec, oc = str(d.get("exit_code")), str(d.get("outcome"))
        c["exit_codes"][ec] = c["exit_codes"].get(ec, 0) + 1
        c["outcomes"][oc] = c["outcomes"].get(oc, 0) + 1
else:
    sources["close_states"] = "absent"

verdicts = None
vpath, apath = os.environ.get("VERDICTS", ""), os.environ.get("APPLIED", "")
if vpath and os.path.exists(vpath):
    sources["verdicts"] = "present"
    applied_ids, withdrawn_ids = set(), set()
    def vid(x):
        return x.get("verdict_id") if isinstance(x, dict) else x
    if apath and os.path.exists(apath):
        sources["applied"] = "present"
        with open(apath, errors="replace") as f:
            for line in f:
                try:
                    a = json.loads(line)
                except ValueError:
                    continue
                applied_ids |= {vid(x) for x in a.get("ticked") or []}
                withdrawn_ids |= {vid(x) for x in a.get("withdrawn") or []}
    else:
        sources["applied"] = "absent"
    verdicts = {}
    rows, _ = day_lines(vpath)
    for _, d in rows or []:
        cls, kind, model = dmeta.get(str(d.get("dispatch_id") or d.get("worker")),
                                     ("reviewer", "unknown", "unknown"))
        k = f"{cls}|{kind}|{model}"
        v = verdicts.setdefault(k, {"class": cls, "kind": kind, "model": model, "recorded": 0,
                                    "by_verdict": {}, "applied": 0, "withdrawn": 0, "pending": 0})
        v["recorded"] += 1
        vv = str(d.get("verdict") or d.get("outcome"))
        v["by_verdict"][vv] = v["by_verdict"].get(vv, 0) + 1
        i = d.get("id")
        if i in withdrawn_ids:
            v["withdrawn"] += 1
        elif i in applied_ids:
            v["applied"] += 1
        else:
            v["pending"] += 1
else:
    sources["verdicts"] = "absent"

standing_raw = os.environ.get("STANDING", "none")
try:
    units = json.loads(standing_raw) if standing_raw != "none" else None
except ValueError:
    units = None
if units:
    standing = {"measured": True, "units": [
        {"unit": u.get("unit"), "active": u.get("active"),
         "nrestarts": int(u["nrestarts"]) if str(u.get("nrestarts", "")).isdigit() else None}
        for u in units]}
else:
    standing = {"measured": False, "reason": "not measured (no systemd, or no standing-worker unit on this host)"}

signals["worker_outcomes"] = {
    "dispatch": sorted(outcomes.values(), key=lambda o: (o["class"], o["kind"], o["model"])),
    "close_ledger": None if close_ledger is None else sorted(close_ledger.values(), key=lambda c: (str(c["model"]), str(c["task_type"]))),
    "verdicts": None if verdicts is None else sorted(verdicts.values(), key=lambda v: (v["class"], v["kind"], v["model"])),
    "standing": standing,
}

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
