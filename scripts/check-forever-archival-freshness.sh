#!/usr/bin/env bash
# T-2562 (T-2468 purpose-review, non-goal #2) — Forever-topic archival-growth canary.
#
# The charter's non-goal #2 is "NOT a durable database / system of record": topics
# are retention-bounded append logs sized for coordination, and "durability" means
# "survives a hub blip and replays", NOT "stored forever". That boundary had a
# structural blind spot. Two existing guards do NOT cover the general case:
#   - The create-time warn (channel.rs T-2058 / T-2648) is warn-only ("not a
#     refusal") and only fires when the topic NAME matches a high-rate /
#     single-value-state pattern.
#   - The topic-growth canary (T-2252) fires ONLY on watched high-rate patterns
#     (agent-presence, agent-listeners-*, agent-conv-*, dm:*) and EXPLICITLY
#     excludes operator-durable topics — so it deliberately skips every other
#     `Retention::Forever` topic.
# Net: an arbitrarily-named topic set to `Retention::Forever` and grown unboundedly
# as archival storage matches NO watch pattern → invisible to both guards. That is
# exactly the "system of record" non-goal being violated with nothing firing.
#
# This canary closes the gap. It reads `termlink channel list --json` (the same
# plumbing T-2252 uses) and FIRES (exit 1) on ANY topic whose
# `retention.kind == "forever"` and whose record `count` exceeds an archival
# ceiling (default 50000, --threshold N), MINUS the operator-durable allowlist
# (channel:learnings, policy-decisions, framework:pickup, broadcast:global —
# intentionally Forever, mirrors the T-2252 / T-2057 audit §5 exclusions). The
# ceiling is deliberately high (a small Forever topic is fine; a Forever topic
# with tens of thousands of records is being used as a database).
#
# Distinct axis from T-2252: T-2252 asks "is a high-rate topic un-swept?" (a
# hygiene-cron failure); THIS asks "is a Forever topic being used as archival
# storage?" (a non-goal violation). Same read, orthogonal firing gate.
#
# Empty output (in --quiet) = healthy — the same convention as the topic-growth /
# dead-letter / mirror / frozen-husk canaries. /canaries auto-discovers this canary
# via the .heartbeat companion + the cron log.
#
# Exit codes:
#   0  — healthy (no non-allowlisted Forever topic over the archival ceiling)
#   1  — one or more Forever topics over the ceiling (non-goal #2 drift)
#   2  — tooling error (hub unreachable / parse failure)
#
# Usage:
#   check-forever-archival-freshness.sh                 # human-readable, one-shot
#   check-forever-archival-freshness.sh --json          # JSON envelope for scripting
#   check-forever-archival-freshness.sh --quiet         # print only on firing (cron)
#   check-forever-archival-freshness.sh --threshold N   # archival ceiling (default 50000)
#   check-forever-archival-freshness.sh --hub ADDR      # target a specific hub
#   check-forever-archival-freshness.sh --no-heartbeat  # suppress heartbeat touch
#
# Test hook (PL-213): set TERMLINK_FOREVER_TEST_JSON=<file> to feed a canned
# `channel list` JSON instead of calling the live hub — makes firing logic
# verifiable hub-independently.

set -eu

HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.forever-archival-canary.heartbeat}"
THRESHOLD=50000
# T-3220 (C-15): secondary rate trigger for Forever topics under the ceiling.
RATE_PER_DAY=50
RATE_MIN_DAYS=7
HUB=""
FORMAT=human
QUIET=0
HEARTBEAT=1

# Operator-durable topics — intentionally Forever; never fire (mirrors T-2252 /
# audit §5 / retention-reset runbook §1 exclusions). Override via env for tests.
EXCLUDE_TOPICS="${TERMLINK_FOREVER_EXCLUDE_TOPICS:-channel:learnings,policy-decisions,framework:pickup,broadcast:global}"

TERMLINK_BIN="${TERMLINK_BIN:-termlink}"

while [ $# -gt 0 ]; do
    case "$1" in
        --json)  FORMAT=json ;;
        --quiet) QUIET=1 ;;
        --no-heartbeat) HEARTBEAT=0 ;;
        --threshold) shift; THRESHOLD="${1:-50000}" ;;
        --threshold=*) THRESHOLD="${1#*=}" ;;
        --rate-per-day) shift; RATE_PER_DAY="${1:-50}" ;;
        --rate-min-days) shift; RATE_MIN_DAYS="${1:-7}" ;;
        --hub) shift; HUB="${1:-}" ;;
        --hub=*) HUB="${1#*=}" ;;
        -h|--help) sed -n '2,60p' "$0"; exit 0 ;;
        *) echo "unknown arg: $1" >&2; exit 2 ;;
    esac
    shift
done

# Heartbeat first (prove the canary ran even on healthy/error cycles — T-1723).
# T-2691: heartbeat is written on COMPLETION, not on start, so its freshness proves
# the run FINISHED rather than merely that cron began it. This script already owns an
# EXIT trap, so the helper is SELF-GUARDING and is chained onto that trap below —
# registering a second `trap ... EXIT` here would silently replace the cleanup.
_canary_hb() {
    [ "$HEARTBEAT" = 1 ] || return 0
    mkdir -p "$(dirname "$HEARTBEAT_FILE")" 2>/dev/null || true
    touch -- "$HEARTBEAT_FILE" 2>/dev/null || true
}
# Arm immediately so an early tooling-error exit (before the combined trap below
# is installed) still records that the run completed. The later
# `trap '<cleanup>; _canary_hb' EXIT` supersedes this one and calls the helper too,
# so the heartbeat is covered continuously rather than only after that point.
trap _canary_hb EXIT

# Acquire the channel-list JSON: canned (test hook) or live hub.
if [ -n "${TERMLINK_FOREVER_TEST_JSON:-}" ]; then
    LIST_JSON="$(cat -- "$TERMLINK_FOREVER_TEST_JSON" 2>/dev/null)" || {
        echo "forever-archival canary: cannot read test JSON $TERMLINK_FOREVER_TEST_JSON" >&2; exit 2; }
else
    if [ -n "$HUB" ]; then
        LIST_JSON="$("$TERMLINK_BIN" channel list --json --hub "$HUB" 2>/dev/null)" || LIST_JSON=""
    else
        LIST_JSON="$("$TERMLINK_BIN" channel list --json 2>/dev/null)" || LIST_JSON=""
    fi
    if [ -z "$LIST_JSON" ]; then
        # Hub unreachable / no output → tooling error (NOT a healthy 0, NOT a fire).
        if [ "$QUIET" != 1 ]; then
            if [ "$FORMAT" = json ]; then
                printf '{"ok": false, "reason": "hub unreachable or empty channel list", "threshold": %s, "firing": []}\n' "$THRESHOLD"
            else
                echo "forever-archival canary: hub unreachable (could not read channel list) — tooling error" >&2
            fi
        fi
        exit 2
    fi
fi

# Stage the JSON in a temp file and pass its PATH to python (argv). A large
# channel list can exceed MAX_ARG_STRLEN (~128KB) as an inline arg or env var, so a
# file path is always safe. The program itself rides the heredoc on stdin.
LIST_TMP="$(mktemp "${TMPDIR:-/tmp}/termlink-forever-archival.XXXXXX")" || {
    echo "forever-archival canary: cannot create temp file" >&2; exit 2; }
trap 'rm -f -- "$LIST_TMP"; _canary_hb' EXIT
printf '%s' "$LIST_JSON" > "$LIST_TMP"

REPORT="$(RATE_PER_DAY="$RATE_PER_DAY" RATE_MIN_DAYS="$RATE_MIN_DAYS" HUB="$HUB" TERMLINK_BIN="$TERMLINK_BIN" \
    python3 - "$LIST_TMP" "$THRESHOLD" "$FORMAT" "$EXCLUDE_TOPICS" <<'PY' 2>/dev/null || true
import sys, json, os, subprocess, time

list_path = sys.argv[1]
threshold = int(sys.argv[2]); fmt = sys.argv[3]
exclude = set(p for p in sys.argv[4].split(",") if p)

try:
    with open(list_path) as fh:
        data = json.load(fh)
except Exception:
    print("PARSE_ERROR=1"); sys.exit(0)

topics = data.get("topics", []) if isinstance(data, dict) else []

firing = []
candidates = []
rate_unchecked = []
for t in topics:
    name = t.get("name", "")
    if not name or name in exclude:
        continue
    ret = (t.get("retention") or {}).get("kind", "unknown")
    if ret != "forever":
        continue
    count = t.get("count", 0)
    try:
        count = int(count)
    except Exception:
        continue
    if count > threshold:
        firing.append({"name": name, "count": count, "retention": ret, "trigger": "ceiling"})
    else:
        candidates.append((name, count))

# T-3220 (C-15): a Forever topic UNDER the ceiling but growing every day is the
# class both T-2252 (watched names only) and the ceiling above miss. Forever
# topics are never swept, so the offset-0 envelope's ts is a true time base and
# records/day needs no state file. A topic whose first ts cannot be read is
# reported as rate-unchecked, never counted as healthy.
rate_per_day = float(os.environ.get("RATE_PER_DAY", "50"))
rate_min_days = float(os.environ.get("RATE_MIN_DAYS", "7"))
now_ms = int(os.environ.get("TERMLINK_FOREVER_TEST_NOW_MS") or time.time() * 1000)
seam = os.environ.get("TERMLINK_FOREVER_TEST_FIRST_TS_JSON")
seam_map = None
if seam:
    try:
        seam_map = json.load(open(seam))
    except Exception:
        seam_map = {}

def first_ts_ms(topic):
    if seam_map is not None:
        v = seam_map.get(topic)
        return int(v) if v is not None else None
    cmd = [os.environ.get("TERMLINK_BIN", "termlink"), "channel", "subscribe", topic,
           "--cursor", "0", "--limit", "1", "--json"]
    if os.environ.get("HUB"):
        cmd += ["--hub", os.environ["HUB"]]
    try:
        out = subprocess.run(cmd, capture_output=True, text=True, timeout=15).stdout
    except Exception:
        return None
    for line in out.splitlines():
        try:
            ts = json.loads(line).get("ts")
        except Exception:
            continue
        if isinstance(ts, (int, float)) and ts > 0:
            return int(ts)
    return None

for name, count in candidates:
    if count == 0:
        continue
    ts0 = first_ts_ms(name)
    if ts0 is None:
        rate_unchecked.append(name)
        continue
    age_days = (now_ms - ts0) / 86400000.0
    if age_days < rate_min_days:
        continue
    per_day = count / age_days
    if per_day > rate_per_day:
        firing.append({"name": name, "count": count, "retention": "forever", "trigger": "rate",
                       "per_day": round(per_day, 1), "age_days": round(age_days, 1)})

firing.sort(key=lambda f: -f["count"])

if fmt == "json":
    print(json.dumps({
        "ok": len(firing) == 0,
        "threshold": threshold,
        "excluded": sorted(exclude),
        "rate_per_day": rate_per_day,
        "rate_min_days": rate_min_days,
        "rate_unchecked": sorted(rate_unchecked),
        "firing": firing,
    }))
else:
    if firing:
        print("forever-archival canary: %d Forever topic(s) over the archival ceiling (%d records) or growing > %g/day — non-goal #2 drift (system-of-record)" % (len(firing), threshold, rate_per_day))
        for f in firing:
            if f.get("trigger") == "rate":
                print("  %s  count=%d  [forever]  RATE %.1f/day over %.1f days" % (f["name"], f["count"], f["per_day"], f["age_days"]))
                print("    → under the ceiling but growing without bound; at this rate it becomes archival storage (charter non-goal #2).")
            else:
                print("  %s  count=%d  [forever]" % (f["name"], f["count"]))
                print("    → a Forever topic this large is archival storage, which TermLink is NOT (charter non-goal #2).")
            print("      Bound it: termlink channel set-retention %s --retention messages --retention-value N && termlink channel sweep %s" % (f["name"], f["name"]))
            print("      Or, if genuinely operator-durable, add %s to TERMLINK_FOREVER_EXCLUDE_TOPICS." % f["name"])
        print("  (allowlisted operator-durable Forever topics never fire: %s)" % ", ".join(sorted(exclude)))
    if rate_unchecked:
        print("  (rate not checked — first envelope unreadable: %s)" % ", ".join(sorted(rate_unchecked)))

print("FIRE=%d" % len(firing))
PY
)"

if printf '%s\n' "$REPORT" | grep -q '^PARSE_ERROR=1'; then
    echo "forever-archival canary: could not parse channel list JSON" >&2
    exit 2
fi

FIRE="$(printf '%s\n' "$REPORT" | sed -n 's/^FIRE=//p' | tail -1)"
BODY="$(printf '%s\n' "$REPORT" | grep -v -e '^FIRE=' || true)"

if [ -z "${FIRE:-}" ]; then
    echo "forever-archival canary: internal error (no FIRE sentinel)" >&2
    exit 2
fi

if [ "${FIRE:-0}" = 0 ]; then
    if [ "$QUIET" != 1 ]; then
        if [ "$FORMAT" = json ]; then
            printf '%s\n' "$BODY"
        else
            echo "forever-archival canary: healthy — no Forever topic over $THRESHOLD records or growing > ${RATE_PER_DAY}/day"
            printf '%s\n' "$BODY" | grep 'rate not checked' || true
        fi
    fi
    exit 0
fi

# Firing — always print (including --quiet, so the cron log captures it).
printf '%s\n' "$BODY"
exit 1
