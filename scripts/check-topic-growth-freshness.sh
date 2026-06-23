#!/usr/bin/env bash
# T-2252 — Topic-growth canary (arc-002 R2 sweep-cron guard; PL-168 / G-019 class).
#
# R2 (T-2245) bounds high-rate topics like `agent-presence` via
# `channel set-retention latest-per-cv-key` + a periodic `channel sweep`. But the
# bus runs NO background sweep thread (T-1155: enforcement is explicit, never
# implicit), so `sweep` depends on an operator CRON. If that cron is never
# installed or silently stops, the topic regrows — a T-1991 recurrence with
# nothing to surface it. This is the same "framework relies on out-of-band hygiene
# that may never run" class T-2251 fixed for the audit log; this canary closes it
# for high-rate topics.
#
# Model: read `termlink channel list --json`. A topic FIRES when:
#   - its name matches a high-rate watch pattern (default: agent-presence,
#     agent-listeners-*, agent-conv-*, dm:*; override TERMLINK_GROWTH_WATCH_PATTERNS), AND
#   - its name is NOT in the operator-durable exclusion set (channel:learnings,
#     policy-decisions, framework:pickup, broadcast:global — intentionally Forever,
#     mirrors the T-2057 audit §5 / runbook §1 exclusions), AND
#   - its record `count` exceeds --threshold (default 5000).
# The retention.kind of a firing topic selects the remediation hint:
#   - forever  → retention was never set: run `set-retention latest-per-cv-key` + `sweep`.
#   - bounded  → policy set but the topic is large anyway → the sweep cron isn't firing.
#
# Empty output (in --quiet) = healthy — same convention as the mirror /
# frozen-husk / framework-pickup / substrate-preflight canaries. /canaries
# auto-discovers this canary via the .heartbeat companion + the cron log.
#
# Exit codes:
#   0  — healthy (no watched topic over threshold)
#   1  — one or more watched topics over threshold (regrowth)
#   2  — tooling error (hub unreachable / parse failure)
#
# Usage:
#   check-topic-growth-freshness.sh                 # human-readable, one-shot
#   check-topic-growth-freshness.sh --json          # JSON envelope for scripting
#   check-topic-growth-freshness.sh --quiet         # print only on firing (cron)
#   check-topic-growth-freshness.sh --threshold N   # count ceiling (default 5000)
#   check-topic-growth-freshness.sh --hub ADDR      # target a specific hub
#   check-topic-growth-freshness.sh --no-heartbeat  # suppress heartbeat touch
#
# Test hook (PL-213): set TERMLINK_GROWTH_TEST_JSON=<file> to feed a canned
# `channel list` JSON instead of calling the live hub — makes firing logic
# verifiable hub-independently.

set -eu

HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.topic-growth-canary.heartbeat}"
THRESHOLD=5000
HUB=""
FORMAT=human
QUIET=0
HEARTBEAT=1

# High-rate patterns that SHOULD be bounded. csv; `*` suffix = name prefix match,
# otherwise exact match. Override via TERMLINK_GROWTH_WATCH_PATTERNS.
WATCH_PATTERNS="${TERMLINK_GROWTH_WATCH_PATTERNS:-agent-presence,agent-listeners-*,agent-conv-*,dm:*}"
# Operator-durable topics — intentionally Forever; never fire (audit §5 / runbook §1).
EXCLUDE_TOPICS="${TERMLINK_GROWTH_EXCLUDE_TOPICS:-channel:learnings,policy-decisions,framework:pickup,broadcast:global}"

TERMLINK_BIN="${TERMLINK_BIN:-termlink}"

while [ $# -gt 0 ]; do
    case "$1" in
        --json)  FORMAT=json ;;
        --quiet) QUIET=1 ;;
        --no-heartbeat) HEARTBEAT=0 ;;
        --threshold) shift; THRESHOLD="${1:-5000}" ;;
        --threshold=*) THRESHOLD="${1#*=}" ;;
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

# Acquire the channel-list JSON: canned (test hook) or live hub.
if [ -n "${TERMLINK_GROWTH_TEST_JSON:-}" ]; then
    LIST_JSON="$(cat -- "$TERMLINK_GROWTH_TEST_JSON" 2>/dev/null)" || {
        echo "topic-growth canary: cannot read test JSON $TERMLINK_GROWTH_TEST_JSON" >&2; exit 2; }
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
                echo "topic-growth canary: hub unreachable (could not read channel list) — tooling error" >&2
            fi
        fi
        exit 2
    fi
fi

# Stage the JSON in a temp file and pass its PATH to python (argv). A large
# channel list (hundreds of topics) exceeds the per-env-var / per-arg string
# limit (MAX_ARG_STRLEN, ~128KB), so neither an env var nor an inline arg is
# safe — a file path always is. The program itself rides the heredoc on stdin.
LIST_TMP="$(mktemp "${TMPDIR:-/tmp}/termlink-topic-growth.XXXXXX")" || {
    echo "topic-growth canary: cannot create temp file" >&2; exit 2; }
trap 'rm -f -- "$LIST_TMP"' EXIT
printf '%s' "$LIST_JSON" > "$LIST_TMP"

REPORT="$(python3 - "$LIST_TMP" "$THRESHOLD" "$FORMAT" "$WATCH_PATTERNS" "$EXCLUDE_TOPICS" <<'PY' 2>/dev/null || true
import sys, json

list_path = sys.argv[1]
threshold = int(sys.argv[2]); fmt = sys.argv[3]
watch = [p for p in sys.argv[4].split(",") if p]
exclude = set(p for p in sys.argv[5].split(",") if p)

try:
    with open(list_path) as fh:
        data = json.load(fh)
except Exception:
    print("PARSE_ERROR=1"); sys.exit(0)

topics = data.get("topics", []) if isinstance(data, dict) else []

def is_watched(name):
    for p in watch:
        if p.endswith("*"):
            if name.startswith(p[:-1]):
                return True
        elif name == p:
            return True
    return False

firing = []
for t in topics:
    name = t.get("name", "")
    if not name or name in exclude or not is_watched(name):
        continue
    count = t.get("count", 0)
    try:
        count = int(count)
    except Exception:
        continue
    if count > threshold:
        ret = (t.get("retention") or {}).get("kind", "unknown")
        firing.append({"name": name, "count": count, "retention": ret})

firing.sort(key=lambda f: -f["count"])

if fmt == "json":
    print(json.dumps({
        "ok": len(firing) == 0,
        "threshold": threshold,
        "watched_patterns": watch,
        "excluded": sorted(exclude),
        "firing": firing,
    }))
else:
    if firing:
        print("topic-growth canary: %d watched topic(s) over threshold (%d records)" % (len(firing), threshold))
        for f in firing:
            if f["retention"] == "forever":
                hint = "retention=forever — run: termlink channel set-retention %s --retention latest-per-cv-key && termlink channel sweep %s" % (f["name"], f["name"])
            else:
                hint = "retention=%s but large — the periodic `channel sweep %s` cron is NOT firing (T-1155: no in-process sweep)" % (f["retention"], f["name"])
            print("  %s  count=%d  [%s]" % (f["name"], f["count"], f["retention"]))
            print("    → %s" % hint)
        print("  (excluded durable topics are never fired on: %s)" % ", ".join(sorted(exclude)))

print("FIRE=%d" % len(firing))
PY
)"

if printf '%s\n' "$REPORT" | grep -q '^PARSE_ERROR=1'; then
    echo "topic-growth canary: could not parse channel list JSON" >&2
    exit 2
fi

FIRE="$(printf '%s\n' "$REPORT" | sed -n 's/^FIRE=//p' | tail -1)"
BODY="$(printf '%s\n' "$REPORT" | grep -v -e '^FIRE=' || true)"

if [ -z "${FIRE:-}" ]; then
    echo "topic-growth canary: internal error (no FIRE sentinel)" >&2
    exit 2
fi

if [ "${FIRE:-0}" = 0 ]; then
    if [ "$QUIET" != 1 ]; then
        if [ "$FORMAT" = json ]; then
            printf '%s\n' "$BODY"
        else
            echo "topic-growth canary: healthy — no watched topic over $THRESHOLD records"
        fi
    fi
    exit 0
fi

# Firing — always print (including --quiet, so the cron log captures it).
printf '%s\n' "$BODY"
exit 1
