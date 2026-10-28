#!/usr/bin/env bash
# telemetry-fleet-view.sh — T-3466 (R5b): the fleet view over the daily aggregates that
# scripts/telemetry-rollup.sh posts to `telemetry:daily`. Per host and per day: queue flaps,
# reconnects, breaker trips, dead letters and the worst hub outage seen — plus which hosts did
# NOT report on which day. A missing host is the signal, not a gap: a host that stops
# reporting is exactly the one whose view we have lost.
# T-3615: a second table per host/day shows usage (calls and polling per worker class) and
# worker outcomes (dispatch runs, non-zero exits, timeouts, empty results, verdicts recorded /
# applied / withdrawn, standing-worker restarts or "n/m" = not measured). --json carries the
# full `usage` and `worker_outcomes` blocks of each report.
#
# T-3615 slice 3b (external review): reads EVERY page of the topic (it read one page of 5,000
# and silently lost newer reports past it); an empty fleet is not ok; when a host re-ran a day,
# the most complete report wins (usage covering the whole day over a partial or rotated one),
# then the latest; usage that rotated away shows "rotated", never zero.
#
# Usage: telemetry-fleet-view.sh [--days N] [--expect h1,h2,...|reported] [--hub ADDR] [--topic T] [--json]
#   --days    window of UTC days ending yesterday (default 7, clamped 1..90)
#   --expect  hosts that must report. Default: the fleet inventory, i.e. this host plus every
#             profile in ~/.termlink/hubs.toml named in $TELEMETRY_FLEET_PROFILES (default: the
#             hubs runme.sh action 23 installs the rollup on; a fixture keeps the two lists
#             equal), plus any host that reported. "reported" = only the hosts that reported.
#   --hub     hub holding the topic (default: $TELEMETRY_HUB, else the local hub)
# Exit: 0 every expected host reported every day · 1 at least one missing report, or no
#       report at all · 2 tooling (hub unreachable, unreadable topic).
#
# Test seams (PL-213): TELEMETRY_VIEW_NDJSON (canned `channel subscribe --json` output; skips
# paging), TELEMETRY_NOW (epoch seconds), TELEMETRY_TERMLINK (binary), TELEMETRY_VIEW_PAGE
# (page size, default 5000), TELEMETRY_HUBS_TOML, TELEMETRY_SELF_HOST (default `hostname`).
set -euo pipefail

DAYS=7 EXPECT="${TELEMETRY_EXPECT:-}" HUB="${TELEMETRY_HUB:-}" TOPIC="telemetry:daily" JSON=0
while [ $# -gt 0 ]; do
    case "$1" in
        --days) DAYS="${2:?--days needs N}"; shift 2 ;;
        --expect) EXPECT="${2:?--expect needs a list}"; shift 2 ;;
        --hub) HUB="${2:?--hub needs an address}"; shift 2 ;;
        --topic) TOPIC="${2:?--topic needs a name}"; shift 2 ;;
        --json) JSON=1; shift ;;
        -h|--help) sed -n '2,25p' "$0"; exit 0 ;;
        *) echo "telemetry-fleet-view: unknown argument: $1" >&2; exit 2 ;;
    esac
done
command -v python3 >/dev/null || { echo "telemetry-fleet-view: python3 not found" >&2; exit 2; }
TL="${TELEMETRY_TERMLINK:-termlink}"

# The fleet inventory (review item 28): who SHOULD report, not only who did.
if [ -z "$EXPECT" ]; then
    FLEET_PROFILES="${TELEMETRY_FLEET_PROFILES:-ring20-management ring20-dashboard greenfield}"
    HUBS_TOML="${TELEMETRY_HUBS_TOML:-$HOME/.termlink/hubs.toml}"
    EXPECT="${TELEMETRY_SELF_HOST:-$(hostname)}"
    for p in $FLEET_PROFILES; do
        grep -qE "^\[hubs\.$p\]" "$HUBS_TOML" 2>/dev/null && EXPECT="$EXPECT,$p"
    done
elif [ "$EXPECT" = reported ]; then
    EXPECT=""
fi

if [ -n "${TELEMETRY_VIEW_NDJSON:-}" ]; then
    RAW="$(cat "$TELEMETRY_VIEW_NDJSON")"
else
    HUB_ARGS=()
    [ -n "$HUB" ] && HUB_ARGS=(--hub "$HUB")
    PAGE_LIMIT="${TELEMETRY_VIEW_PAGE:-5000}"
    ERR="$(mktemp "${TMPDIR:-/tmp}/.telemetry-view.XXXXXX")"
    trap 'rm -f "$ERR"' EXIT
    RAW="" cursor=0 pages=0
    while :; do
        if ! PAGE="$(timeout 30 "$TL" channel subscribe "$TOPIC" --json --cursor "$cursor" --limit "$PAGE_LIMIT" "${HUB_ARGS[@]}" 2>"$ERR")"; then
            if [ "$pages" = 0 ] && grep -qi "unknown topic\|not found\|no such topic" "$ERR"; then
                break
            fi
            echo "telemetry-fleet-view: cannot read $TOPIC at cursor $cursor: $(head -c 300 "$ERR")" >&2
            exit 2
        fi
        RAW="$RAW"$'\n'"$PAGE"
        pages=$((pages + 1))
        # The trailer says whether the page was cut by the limit and where the next one starts.
        next="$(printf '%s\n' "$PAGE" | python3 -c '
import json, sys
nxt, last, n = None, None, 0
for line in sys.stdin:
    try:
        d = json.loads(line)
    except ValueError:
        continue
    if "next_cursor" in d and "offset" not in d:
        if d.get("note") == "page_limit":
            nxt = d["next_cursor"]
    elif "offset" in d:
        n += 1; last = d["offset"]
if nxt is None and n >= int(sys.argv[1]) and last is not None:
    nxt = last + 1
print("" if nxt is None else nxt)' "$PAGE_LIMIT")"
        [ -n "$next" ] || break
        [ "$next" -gt "$cursor" ] || { echo "telemetry-fleet-view: paging did not advance at cursor $cursor" >&2; exit 2; }
        [ "$pages" -lt 1000 ] || { echo "telemetry-fleet-view: more than 1000 pages, refusing to guess" >&2; exit 2; }
        cursor="$next"
    done
fi

RAW="$RAW" python3 - "$DAYS" "$EXPECT" "$JSON" <<'PY'
import base64, json, os, sys, datetime as dt

days = max(1, min(90, int(sys.argv[1])))
expect = [h for h in sys.argv[2].split(",") if h]
as_json = sys.argv[3] == "1"
now = float(os.environ.get("TELEMETRY_NOW") or dt.datetime.now(dt.timezone.utc).timestamp())
last = dt.datetime.fromtimestamp(now, dt.timezone.utc).date() - dt.timedelta(days=1)
window = [(last - dt.timedelta(days=i)).isoformat() for i in range(days - 1, -1, -1)]

def completeness(b):
    u = (b.get("signals") or {}).get("usage")
    if not isinstance(u, dict):
        return 0
    return 2 if u.get("coverage", "complete") == "complete" else 1

def rank(b):
    return (completeness(b), b.get("generated_at", ""))

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
    # A re-run for the same host and day: the most complete report wins, then the latest
    # (review item 37: a later catch-up whose rows had rotated must not replace a whole day).
    if key not in reports or rank(body) >= rank(reports[key]):
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

def restarts_cell(st):
    """A daily delta since slice 3b; old reports carried the lifetime NRestarts per unit, which
    is not a day's restarts, so it is not shown as one."""
    if not st.get("measured"):
        return "n/m"
    if isinstance(st.get("units"), list):
        return "n/m(lifetime)"
    r = st.get("restarts")
    return "n/m" if r is None else str(r)

def usage_row(b):
    """T-3615: compact usage + worker-outcome cells; None where the report predates them."""
    s = b.get("signals", {})
    u, w = s.get("usage"), s.get("worker_outcomes")
    out = {"usage": u, "worker_outcomes": w}
    src = str((b.get("sources") or {}).get("invocation_audit", ""))
    if u is None and src.startswith("rotated"):
        out["usage_cell"] = "rotated"
    elif u is not None:
        t = u.get("totals") or {}
        out["usage_cell"] = " ".join(f"{c}={v.get('calls', 0)}+{v.get('polling', 0)}p" for c, v in sorted(t.items())) or "0"
        if u.get("coverage") == "partial":
            out["usage_cell"] += " (partial day)"
    else:
        out["usage_cell"] = None
    if w is not None:
        disp = w.get("dispatch") or []
        runs = sum(o.get("runs", 0) for o in disp)
        unfinished = sum(o.get("unfinished", 0) for o in disp)
        nonzero = sum(n for o in disp for ec, n in (o.get("exit_codes") or {}).items() if ec != "0")
        vd = w.get("verdicts")
        st = w.get("standing") or {}
        out["workers_cell"] = (
            f"runs={runs} unfinished={unfinished} exit!=0={nonzero} timeouts={sum(o.get('timeouts', 0) for o in disp)} "
            f"empty={sum(o.get('empty_results', 0) for o in disp)} "
            + ("verdicts=-" if vd is None else
               f"verdicts={sum(v.get('recorded', 0) for v in vd)}/a{sum(v.get('applied', 0) for v in vd)}"
               f"/w{sum(v.get('withdrawn', 0) for v in vd)}")
            + " restarts=" + restarts_cell(st))
    else:
        out["workers_cell"] = None
    return out

rows = [{"host": h, "day": d, "catch_up": bool(reports[(h, d)].get("catch_up")),
         **summary(reports[(h, d)]), **usage_row(reports[(h, d)])}
        for h in hosts for d in window if (h, d) in reports]
result = {"ok": bool(rows) and not missing, "window": window, "hosts": hosts,
          "rows": rows, "missing": [{"host": h, "day": d} for h, d in missing],
          "unreadable_envelopes": unreadable}

if as_json:
    print(json.dumps(result, indent=2))
else:
    if not rows:
        print(f"telemetry: no report in the window ({window[0]} .. {window[-1]}); that is not ok")
    def cell(v):
        return "-" if v is None else str(v)
    if rows:
        print(f"{'host':<22} {'day':<10} {'flaps':>5} {'backlog_s':>9} {'reconn':>6} {'trips':>5} {'dead':>4}  worst hub outage")
        for r in rows:
            print(f"{r['host']:<22} {r['day']:<10} {cell(r['queue_flaps']):>5} {cell(r['max_backlog_s']):>9} "
                  f"{cell(r['reconnects']):>6} {cell(r['breaker_trips']):>5} {cell(r['dead_letters']):>4}  {cell(r['worst_hub_outage'])}")
        print("('-' = that source does not exist on the host, not zero)")
        if any(r["usage_cell"] is not None or r["workers_cell"] is not None for r in rows):
            print()
            print(f"{'host':<22} {'day':<10}  usage (class=calls+polling)  |  workers")
            for r in rows:
                print(f"{r['host']:<22} {r['day']:<10}  {cell(r['usage_cell'])}  |  {cell(r['workers_cell'])}")
            print("('-' = the report predates T-3615 usage/outcome aggregates; rotated = that day's usage rows were gone"
                  " before it was rolled up; n/m = not measured)")
    for r in rows:
        if r["catch_up"]:
            print(f"LATE     {r['host']} {r['day']} was rolled up late (--catch-up); rotated sources may be partial")
    for h, d in missing:
        print(f"MISSING  {h} did not report for {d}")
    if unreadable:
        print(f"note: {unreadable} envelope(s) could not be decoded")
sys.exit(0 if result["ok"] else 1)
PY
