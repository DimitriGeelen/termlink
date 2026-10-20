#!/usr/bin/env bash
# telemetry-fleet-view.sh — T-3466 (R5b): the fleet view over the daily aggregates that
# scripts/telemetry-rollup.sh posts to `telemetry:daily`. Per host and per day: queue flaps,
# reconnects, breaker trips, dead letters and the worst hub outage seen — plus which hosts did
# NOT report on which day. A missing host is the signal, not a gap: a host that stops
# reporting is exactly the one whose view we have lost.
#
# Usage: telemetry-fleet-view.sh [--days N] [--expect h1,h2,...] [--hub ADDR] [--topic T] [--json]
#   --days    window of UTC days ending yesterday (default 7, clamped 1..90)
#   --expect  hosts that must report; default = every host that reported in the window
#   --hub     hub holding the topic (default: $TELEMETRY_HUB, else the local hub)
# Exit: 0 every expected host reported every day · 1 at least one missing report ·
#       2 tooling (hub unreachable, unreadable topic). A topic with no reports at all is 1
#       when --expect is given, else 0 with a note ("nothing has reported yet").
#
# Test seams (PL-213): TELEMETRY_VIEW_NDJSON (canned `channel subscribe --json` output),
# TELEMETRY_NOW (epoch seconds), TELEMETRY_TERMLINK (binary).
set -euo pipefail

DAYS=7 EXPECT="" HUB="${TELEMETRY_HUB:-}" TOPIC="telemetry:daily" JSON=0
while [ $# -gt 0 ]; do
    case "$1" in
        --days) DAYS="${2:?--days needs N}"; shift 2 ;;
        --expect) EXPECT="${2:?--expect needs a list}"; shift 2 ;;
        --hub) HUB="${2:?--hub needs an address}"; shift 2 ;;
        --topic) TOPIC="${2:?--topic needs a name}"; shift 2 ;;
        --json) JSON=1; shift ;;
        -h|--help) sed -n '2,18p' "$0"; exit 0 ;;
        *) echo "telemetry-fleet-view: unknown argument: $1" >&2; exit 2 ;;
    esac
done
command -v python3 >/dev/null || { echo "telemetry-fleet-view: python3 not found" >&2; exit 2; }
TL="${TELEMETRY_TERMLINK:-termlink}"

if [ -n "${TELEMETRY_VIEW_NDJSON:-}" ]; then
    RAW="$(cat "$TELEMETRY_VIEW_NDJSON")"
else
    HUB_ARGS=()
    [ -n "$HUB" ] && HUB_ARGS=(--hub "$HUB")
    if ! RAW="$(timeout 30 "$TL" channel subscribe "$TOPIC" --json --cursor 0 --limit 5000 "${HUB_ARGS[@]}" 2>/tmp/.telemetry-view.err)"; then
        if grep -qi "unknown topic\|not found\|no such topic" /tmp/.telemetry-view.err; then
            RAW=""
        else
            echo "telemetry-fleet-view: cannot read $TOPIC: $(head -c 300 /tmp/.telemetry-view.err)" >&2
            exit 2
        fi
    fi
fi

RAW="$RAW" python3 - "$DAYS" "$EXPECT" "$JSON" <<'PY'
import base64, json, os, sys, datetime as dt

days = max(1, min(90, int(sys.argv[1])))
expect = [h for h in sys.argv[2].split(",") if h]
as_json = sys.argv[3] == "1"
now = float(os.environ.get("TELEMETRY_NOW") or dt.datetime.now(dt.timezone.utc).timestamp())
last = dt.datetime.fromtimestamp(now, dt.timezone.utc).date() - dt.timedelta(days=1)
window = [(last - dt.timedelta(days=i)).isoformat() for i in range(days - 1, -1, -1)]

reports, unreadable = {}, 0
for line in os.environ.get("RAW", "").splitlines():
    line = line.strip()
    if not line.startswith("{"):
        continue
    try:
        env = json.loads(line)
        if "payload_b64" in env:
            body = json.loads(base64.b64decode(env["payload_b64"]))
        else:
            p = env.get("payload")
            body = json.loads(p) if isinstance(p, str) else p
        if not isinstance(body, dict) or body.get("schema", "").split("/")[0] != "r5-rollup":
            continue
    except (ValueError, TypeError):
        unreadable += 1
        continue
    key = (body.get("host"), body.get("day"))
    # A re-run for the same host and day replaces the earlier one (latest generated_at wins).
    if key not in reports or body.get("generated_at", "") >= reports[key].get("generated_at", ""):
        reports[key] = body

hosts = sorted(set(expect) | {h for (h, d) in reports if d in window})
missing = [(h, d) for h in hosts for d in window if (h, d) not in reports]

def summary(b):
    s = b.get("signals", {})
    q, r, br = s.get("queue"), s.get("reconnect"), s.get("breaker")
    ho = s.get("hub_outages") or {}
    worst = max(ho.items(), key=lambda kv: kv[1].get("max_downtime_s", 0), default=None)
    return {
        "queue_flaps": None if q is None else q.get("pending_events", 0),
        "max_backlog_s": None if q is None else q.get("max_backlog_s", 0),
        "reconnects": None if r is None else r.get("count", 0),
        "breaker_trips": None if br is None else br.get("trips", 0),
        "dead_letters": (s.get("dead_letters") or {}).get("count_at_rollup"),
        "worst_hub_outage": None if worst is None else f"{worst[0]} {worst[1].get('max_downtime_s', 0)}s",
    }

rows = [{"host": h, "day": d, **summary(reports[(h, d)])} for h in hosts for d in window if (h, d) in reports]
result = {"ok": not missing and not (expect and not reports), "window": window, "hosts": hosts,
          "rows": rows, "missing": [{"host": h, "day": d} for h, d in missing],
          "unreadable_envelopes": unreadable}

if as_json:
    print(json.dumps(result, indent=2))
else:
    if not hosts:
        print(f"telemetry: nothing has reported yet ({window[0]} .. {window[-1]})")
    def cell(v):
        return "-" if v is None else str(v)
    if rows:
        print(f"{'host':<22} {'day':<10} {'flaps':>5} {'backlog_s':>9} {'reconn':>6} {'trips':>5} {'dead':>4}  worst hub outage")
        for r in rows:
            print(f"{r['host']:<22} {r['day']:<10} {cell(r['queue_flaps']):>5} {cell(r['max_backlog_s']):>9} "
                  f"{cell(r['reconnects']):>6} {cell(r['breaker_trips']):>5} {cell(r['dead_letters']):>4}  {cell(r['worst_hub_outage'])}")
        print("('-' = that source does not exist on the host, not zero)")
    for h, d in missing:
        print(f"MISSING  {h} did not report for {d}")
    if unreadable:
        print(f"note: {unreadable} envelope(s) could not be decoded")
sys.exit(0 if result["ok"] else 1)
PY
