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
        --hub) shift; HUB="${1:-}" ;;
        --hub=*) HUB="${1#*=}" ;;
        -h|--help) sed -n '2,60p' "$0"; exit 0 ;;
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
trap 'rm -f -- "$LIST_TMP"' EXIT
printf '%s' "$LIST_JSON" > "$LIST_TMP"

REPORT="$(python3 - "$LIST_TMP" "$THRESHOLD" "$FORMAT" "$EXCLUDE_TOPICS" <<'PY' 2>/dev/null || true
import sys, json

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
        firing.append({"name": name, "count": count, "retention": ret})

firing.sort(key=lambda f: -f["count"])

if fmt == "json":
    print(json.dumps({
        "ok": len(firing) == 0,
        "threshold": threshold,
        "excluded": sorted(exclude),
        "firing": firing,
    }))
else:
    if firing:
        print("forever-archival canary: %d Forever topic(s) over archival ceiling (%d records) — non-goal #2 drift (system-of-record)" % (len(firing), threshold))
        for f in firing:
            print("  %s  count=%d  [forever]" % (f["name"], f["count"]))
            print("    → a Forever topic this large is archival storage, which TermLink is NOT (charter non-goal #2).")
            print("      Bound it: termlink channel set-retention %s --retention messages --retention-value N && termlink channel sweep %s" % (f["name"], f["name"]))
            print("      Or, if genuinely operator-durable, add %s to TERMLINK_FOREVER_EXCLUDE_TOPICS." % f["name"])
        print("  (allowlisted operator-durable Forever topics never fire: %s)" % ", ".join(sorted(exclude)))

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
            echo "forever-archival canary: healthy — no Forever topic over $THRESHOLD records"
        fi
    fi
    exit 0
fi

# Firing — always print (including --quiet, so the cron log captures it).
printf '%s\n' "$BODY"
exit 1
