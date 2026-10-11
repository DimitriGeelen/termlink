#!/usr/bin/env bash
# T-3383 — turn hub outage samples into outage intervals and test H-1.
# An outage = a run of consecutive unreachable samples for one ADDRESS (hubs sharing an
# address are one hub). start = first unreachable sample ts; end = first reachable sample ts
# after it (so duration is an upper-bounded estimate, +/- one sampling interval); an outage
# with no later reachable sample is OPEN (end = last sample ts, flagged open). Gaps between
# samples larger than --max-gap (default 300s) are treated as collector downtime: the run is
# closed at the last unreachable sample and flagged "gap", never counted as one huge outage.
# Output: per-hub intervals, count, p50/p95/max seconds, and h1_covered (p95 <= --limit,
# default 3600 s = PN-1 1 h). --json for scripting. Exit 0 always on readable input; 2 tooling.
# Usage: summarise-hub-outages.sh [--json] [--samples F] [--limit S] [--max-gap S] [--since EPOCH]
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SAMPLES="${HUBOUT_SAMPLES:-$ROOT/.context/working/.hub-outage-samples.ndjson}"
json=0 limit=3600 maxgap=300 since=0
while [ $# -gt 0 ]; do
    case "$1" in
        --json) json=1; shift ;;
        --samples) SAMPLES="$2"; shift 2 ;;
        --limit) limit="$2"; shift 2 ;;
        --max-gap) maxgap="$2"; shift 2 ;;
        --since) since="$2"; shift 2 ;;
        -h|--help) sed -n '2,13p' "$0"; exit 0 ;;
        *) echo "summarise-hub-outages: unknown arg $1" >&2; exit 2 ;;
    esac
done
command -v python3 >/dev/null || { echo "summarise-hub-outages: python3 missing" >&2; exit 2; }
[ -r "$SAMPLES" ] || { echo "summarise-hub-outages: cannot read $SAMPLES" >&2; exit 2; }
python3 -I - "$SAMPLES" "$json" "$limit" "$maxgap" "$since" <<'PY'
import json, sys
path, as_json, limit, maxgap, since = sys.argv[1], sys.argv[2] == "1", int(sys.argv[3]), int(sys.argv[4]), int(sys.argv[5])
by = {}   # addr -> {"names": set, "rows": [(ts, reachable)]}
bad = 0
for ln in open(path):
    ln = ln.strip()
    if not ln: continue
    try:
        d = json.loads(ln); ts = int(d["ts"]); r = bool(d["reachable"]); a = d["addr"]
    except Exception:
        bad += 1; continue
    if ts < since: continue
    e = by.setdefault(a, {"names": set(), "rows": {}})
    e["names"].add(d.get("hub", a)); e["rows"][ts] = r and e["rows"].get(ts, True)
def pct(v, p):
    if not v: return None
    v = sorted(v); k = max(0, min(len(v) - 1, int(-(-p * len(v) // 100)) - 1)); return v[k]
hubs = {}
for addr, e in sorted(by.items()):
    rows = sorted(e["rows"].items()); iv = []; start = last_bad = None
    prev = None
    for ts, ok in rows:
        if prev is not None and ts - prev > maxgap and start is not None:
            iv.append({"start": start, "end": last_bad, "duration_s": last_bad - start, "open": False, "gap": True}); start = None
        if not ok:
            if start is None: start = ts
            last_bad = ts
        elif start is not None:
            iv.append({"start": start, "end": ts, "duration_s": ts - start, "open": False, "gap": False}); start = None
        prev = ts
    if start is not None:
        iv.append({"start": start, "end": last_bad, "duration_s": last_bad - start, "open": True, "gap": False})
    d = [i["duration_s"] for i in iv]
    p95 = pct(d, 95)
    hubs[addr] = {"names": sorted(e["names"]), "samples": len(rows), "outages": len(iv), "intervals": iv,
                  "p50_s": pct(d, 50), "p95_s": p95, "max_s": max(d) if d else None,
                  "h1_covered": (p95 is None) or p95 <= limit}
allv = [i["duration_s"] for h in hubs.values() for i in h["intervals"]]
res = {"samples_file": path, "malformed_lines": bad, "limit_s": limit, "hubs": hubs,
       "all": {"outages": len(allv), "p50_s": pct(allv, 50), "p95_s": pct(allv, 95), "max_s": max(allv) if allv else None,
               "h1_covered": (not allv) or pct(allv, 95) <= limit},
       "scope": "intervals are sample-resolution estimates (+/- sampling interval); fewer than ~20 outages is not a p95"}
if as_json:
    print(json.dumps(res, indent=1)); sys.exit(0)
for a, h in hubs.items():
    print(f"{'/'.join(h['names'])} ({a}): {h['samples']} samples, {h['outages']} outages, p95={h['p95_s']} max={h['max_s']}")
    for i in h["intervals"]:
        print(f"  {i['start']} -> {i['end']}  {i['duration_s']}s{' OPEN' if i['open'] else ''}{' GAP' if i['gap'] else ''}")
print(f"ALL: {res['all']['outages']} outages p50={res['all']['p50_s']} p95={res['all']['p95_s']} max={res['all']['max_s']} "
      f"H-1 (p95 <= {limit}s): {'covered' if res['all']['h1_covered'] else 'NOT covered'}")
print("note:", res["scope"])
PY
