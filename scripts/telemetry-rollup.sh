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
# T-3615 adds two aggregates (counts only; no raw row, argument, path or content leaves the host):
#   usage           <runtime_dir>/invocation-audit.jsonl (+ .1), any stray sink at the old fallback
#                   paths (/tmp, $TMPDIR, /var/tmp, $XDG_RUNTIME_DIR termlink dirs), and every
#                   account's per-user sink <home>/.termlink/invocation-audit.jsonl (non-root callers
#                   write there when the hub dir is root-only, slice 3b); each file read once,
#                   deduplicated by real path. Counts per class x surface x name, polling verbs in a
#                   separate `polling` block, per caller bucket (class|harness|kind|model, unknown
#                   values beyond an allowlist as "other") and per class_rule. Rows written before
#                   attribution shipped count under class "unattributed". Each sink is reported by
#                   LABEL (host, stray-*, user-<uid>), never by path, with its coverage of the day:
#                   complete, partial (rotation cut the start of the day) or none (rotated away).
#                   If any sink's rows for the day have rotated away, usage is null with a reason
#                   in sources.invocation_audit, never near-zero counts.
#   worker_outcomes <project>/.context/dispatch/*/ and /tmp/tl-dispatch/*/ (meta.json, exit_code,
#                   timed_out, result.md, finished_at, worker_kind.txt) per
#                   class/kind/model/job_type, kind "unknown" when no worker_kind is recorded, runs
#                   started that day and not finished counted as unfinished;
#                   <runtime_dir>/dispatch-close-states.jsonl per model/task_type;
#                   <project>/.context/reviews/verdicts.jsonl + applied.jsonl;
#                   standing workers: termlink-sidecar@* units counted, restarts as a DAILY delta of
#                   systemd's lifetime NRestarts (state kept on the host), else "not measured".
#                   <project> is this checkout when it is a framework project, else the TermLink
#                   checkout found under /opt; with none, those sources say "not collected on this
#                   host" (slice 4 widens this to every project).
# A source that does not exist is reported as "absent", never as zero.
#
# Usage: telemetry-rollup.sh [--day YYYY-MM-DD] [--hub ADDR] [--topic T] [--host NAME]
#                            [--catch-up N] [--dry-run] [--json]
#   --day      UTC day to roll up (default: yesterday)
#   --catch-up after yesterday, also roll up any day of the last N (1..14) that this host
#              never posted, i.e. a GAP: only days after the first day it ever posted, so a
#              fresh install never back-fills. Each such report carries "catch_up": true
#              (its size-rotated sources may no longer hold the whole day). Why: the rollup
#              ran for yesterday only, so a host down at the cron minute lost that day for
#              good (.107 missed 2026-10-24: down 10-25 01:10..11:40 local, T-3615).
#              Posted days are recorded in .context/working/.telemetry-rollup.posted
#              (the success lines of .telemetry-rollup.log also count, for the days before).
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
# T-3615 slice 3 seams: TELEMETRY_POSTED_STATE (posted-days file), TELEMETRY_POSTED_LOG
# (the legacy success log read for posted days), TELEMETRY_CATCH_UP=1 (internal: marks a
# catch-up report).
# T-3615 slice 3b seams: TELEMETRY_INVOCATION_SINKS entries may be "label=path";
# TELEMETRY_USER_HOMES (colon list of "uid=home"; empty = no per-user sinks; unset = every
# account in the passwd database); TELEMETRY_RUNTIME_CANDIDATES (colon list, primary first,
# replaces the binary's candidate list); TELEMETRY_PROJECT_ROOT (the project whose dispatch and
# review ledgers are read; "none" = not collected); TELEMETRY_STANDING_STATE (the NRestarts
# baseline file).
set -euo pipefail

DAY="" HUB="${TELEMETRY_HUB:-}" TOPIC="telemetry:daily" HOST="${TELEMETRY_HOST:-}" DRY=0 JSON=0 CATCH_UP=""
while [ $# -gt 0 ]; do
    case "$1" in
        --day) DAY="${2:?--day needs YYYY-MM-DD}"; shift 2 ;;
        --hub) HUB="${2:?--hub needs an address}"; shift 2 ;;
        --topic) TOPIC="${2:?--topic needs a name}"; shift 2 ;;
        --host) HOST="${2:?--host needs a name}"; shift 2 ;;
        --catch-up) CATCH_UP="${2:?--catch-up needs a day count}"; shift 2 ;;
        --dry-run) DRY=1; shift ;;
        --json) JSON=1; shift ;;
        -h|--help) sed -n '2,58p' "$0"; exit 0 ;;
        *) echo "telemetry-rollup: unknown argument: $1" >&2; exit 2 ;;
    esac
done

command -v python3 >/dev/null || { echo "telemetry-rollup: python3 not found" >&2; exit 2; }
TL="${TELEMETRY_TERMLINK:-termlink}"
[ -n "$HOST" ] || HOST="$(hostname)"

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
POSTED_STATE="${TELEMETRY_POSTED_STATE:-$PROJECT_ROOT/.context/working/.telemetry-rollup.posted}"
POSTED_LOG="${TELEMETRY_POSTED_LOG:-$PROJECT_ROOT/.context/working/.telemetry-rollup.log}"

# --catch-up N: roll up yesterday as usual, then every gap day of the last N. Each day is a
# separate run of this script, so a catch-up report is built exactly like an on-time one.
if [ -n "$CATCH_UP" ]; then
    case "$CATCH_UP" in ''|*[!0-9]*) echo "telemetry-rollup: --catch-up needs 1..14" >&2; exit 2 ;; esac
    [ "$CATCH_UP" -ge 1 ] && [ "$CATCH_UP" -le 14 ] || { echo "telemetry-rollup: --catch-up needs 1..14" >&2; exit 2; }
    [ -z "$DAY" ] || { echo "telemetry-rollup: --catch-up and --day are exclusive" >&2; exit 2; }
    [ "$DRY" = 0 ] || { echo "telemetry-rollup: --catch-up does not combine with --dry-run" >&2; exit 2; }
    PASS=(--host "$HOST" --topic "$TOPIC")
    [ -n "$HUB" ] && PASS+=(--hub "$HUB")
    rc=0
    bash "${BASH_SOURCE[0]}" "${PASS[@]}" || rc=1
    GAPS="$(python3 - "$CATCH_UP" "$HOST" "$POSTED_STATE" "$POSTED_LOG" <<'PY'
import os, re, sys, datetime as dt
n, host, state, log = int(sys.argv[1]), sys.argv[2], sys.argv[3], sys.argv[4]
now = float(os.environ.get("TELEMETRY_NOW") or dt.datetime.now(dt.timezone.utc).timestamp())
yesterday = dt.datetime.fromtimestamp(now, dt.timezone.utc).date() - dt.timedelta(days=1)
posted = set()
for path, rx in ((state, re.compile(r"^(\S+) (\d{4}-\d\d-\d\d)$")),
                 (log, re.compile(r"^telemetry-rollup: (\S+) (\d{4}-\d\d-\d\d) -> "))):
    try:
        for line in open(path, errors="replace"):
            m = rx.match(line.strip())
            if m and m.group(1) == host:
                posted.add(m.group(2))
    except OSError:
        pass
first = min(posted, default=None)
for i in range(n, 1, -1):
    d = (yesterday - dt.timedelta(days=i - 1)).isoformat()
    if first is not None and d > first and d not in posted:
        print(d)
PY
)" || { echo "telemetry-rollup: could not work out the catch-up days" >&2; exit 2; }
    for d in $GAPS; do
        TELEMETRY_CATCH_UP=1 bash "${BASH_SOURCE[0]}" "${PASS[@]}" --day "$d" || rc=1
    done
    exit "$rc"
fi
# The runtime dir, resolved the way the binary resolves its sink (T-3615 slice 3b, review item 3):
# an explicit TERMLINK_RUNTIME_DIR; else the first of the binary's candidates (primary, then
# /var/lib/termlink, $XDG_RUNTIME_DIR/termlink, /tmp/termlink-<uid>) that holds a hub.sock;
# else the primary. The primary is discovery::runtime_dir: $XDG_RUNTIME_DIR/termlink, else
# $TMPDIR/termlink-<uid>, else /tmp/termlink-<uid>.
UID_="$(id -u)"
resolve_runtime_dir() {
    [ -n "${TERMLINK_RUNTIME_DIR:-}" ] && { echo "$TERMLINK_RUNTIME_DIR"; return; }
    local primary cands d
    if [ -n "${XDG_RUNTIME_DIR:-}" ]; then primary="$XDG_RUNTIME_DIR/termlink"
    elif [ -n "${TMPDIR:-}" ]; then primary="$TMPDIR/termlink-$UID_"
    else primary="/tmp/termlink-$UID_"; fi
    cands="${TELEMETRY_RUNTIME_CANDIDATES-$primary:/var/lib/termlink:${XDG_RUNTIME_DIR:+$XDG_RUNTIME_DIR/termlink}:/tmp/termlink-$UID_}"
    IFS=: read -ra _c <<< "$cands"
    for d in "${_c[@]}"; do
        [ -n "$d" ] && [ -e "$d/hub.sock" ] && { echo "$d"; return; }
    done
    for d in "${_c[@]}"; do [ -n "$d" ] && { echo "$d"; return; }; done
    echo "$primary"
}
RUNTIME_DIR="$(resolve_runtime_dir)"

# The project whose dispatch and review ledgers are read (review item 33). On the fleet hubs the
# rollup runs from /var/lib/termlink-telemetry, which is not a project, so it used to read empty
# paths there and report them "absent" as if the host had no workers.
if [ -n "${TELEMETRY_PROJECT_ROOT:-}" ]; then
    LEDGER_ROOT="$TELEMETRY_PROJECT_ROOT"
elif [ -f "$PROJECT_ROOT/.framework.yaml" ]; then
    LEDGER_ROOT="$PROJECT_ROOT"
else
    LEDGER_ROOT=none
    for d in /opt/*/; do
        d="${d%/}"
        if [ -f "$d/.framework.yaml" ] && [ -f "$d/scripts/telemetry-rollup.sh" ]; then LEDGER_ROOT="$d"; break; fi
    done
fi
[ "$LEDGER_ROOT" = none ] || [ -d "$LEDGER_ROOT" ] || LEDGER_ROOT=none
QUEUE_LOG="${TELEMETRY_QUEUE_LOG:-$HOME/.termlink/queue.log}"
RECONNECT_LOG="${TELEMETRY_RECONNECT_LOG:-$HOME/.termlink/reconnect.log}"
BREAKER_LOG="${TELEMETRY_BREAKER_LOG:-$RUNTIME_DIR/circuit-breaker.jsonl}"
OUTAGE_SAMPLES="${TELEMETRY_OUTAGE_SAMPLES:-$PROJECT_ROOT/.context/working/.hub-outage-samples.ndjson}"

# T-3615: usage sinks as label=path — the host sink first, then the stray fallback sinks callers
# without TERMLINK_RUNTIME_DIR wrote to before the one-sink rule shipped. Per-user sinks are
# added by the aggregator (TELEMETRY_USER_HOMES).
if [ -n "${TELEMETRY_INVOCATION_SINKS+x}" ]; then
    INV_SINKS="$TELEMETRY_INVOCATION_SINKS"
else
    INV_SINKS="host=$RUNTIME_DIR/invocation-audit.jsonl:stray-tmp=/tmp/termlink-$UID_/invocation-audit.jsonl"
    [ -n "${TMPDIR:-}" ] && INV_SINKS="$INV_SINKS:stray-tmpdir=$TMPDIR/termlink-$UID_/invocation-audit.jsonl"
    INV_SINKS="$INV_SINKS:stray-vartmp=/var/tmp/termlink-$UID_/invocation-audit.jsonl"
    [ -n "${XDG_RUNTIME_DIR:-}" ] && INV_SINKS="$INV_SINKS:stray-xdg=$XDG_RUNTIME_DIR/termlink/invocation-audit.jsonl"
fi
if [ "$LEDGER_ROOT" = none ]; then
    DISPATCH_DIRS="${TELEMETRY_DISPATCH_DIRS-/tmp/tl-dispatch}"
    VERDICTS="${TELEMETRY_VERDICTS:-none}"
    APPLIED="${TELEMETRY_APPLIED:-none}"
else
    DISPATCH_DIRS="${TELEMETRY_DISPATCH_DIRS-$LEDGER_ROOT/.context/dispatch:/tmp/tl-dispatch}"
    VERDICTS="${TELEMETRY_VERDICTS:-$LEDGER_ROOT/.context/reviews/verdicts.jsonl}"
    APPLIED="${TELEMETRY_APPLIED:-$LEDGER_ROOT/.context/reviews/applied.jsonl}"
fi
CLOSE_STATES="${TELEMETRY_CLOSE_STATES:-$RUNTIME_DIR/dispatch-close-states.jsonl}"
STANDING_STATE="${TELEMETRY_STANDING_STATE:-$PROJECT_ROOT/.context/working/.telemetry-rollup.nrestarts}"
[ "$LEDGER_ROOT" = none ] || STANDING_STATE="${TELEMETRY_STANDING_STATE:-$LEDGER_ROOT/.context/working/.telemetry-rollup.nrestarts}"
[ -n "${TELEMETRY_STANDING_STATE:-}" ] || [ -d "$(dirname "$STANDING_STATE")" ] || STANDING_STATE="$PROJECT_ROOT/.context/working/.telemetry-rollup.nrestarts"
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
LEDGER_ROOT_SET=1; [ "$LEDGER_ROOT" = none ] && LEDGER_ROOT_SET=0
STANDING_WRITE=0; [ "$DRY" = 0 ] && [ "${TELEMETRY_CATCH_UP:-0}" != 1 ] && STANDING_WRITE=1
export INV_SINKS DISPATCH_DIRS CLOSE_STATES VERDICTS APPLIED STANDING LEDGER_ROOT_SET STANDING_STATE STANDING_WRITE

if [ -n "${TELEMETRY_QUEUE_STATUS_JSON:-}" ]; then
    QSTATUS="$(cat "$TELEMETRY_QUEUE_STATUS_JSON")"
else
    QSTATUS="$(timeout 15 "$TL" channel queue-status --json 2>/dev/null || true)"
fi

STANDING_NEXT="$(mktemp "${TMPDIR:-/tmp}/.telemetry-nrestarts.XXXXXX")"
trap 'rm -f "$STANDING_NEXT"' EXIT
export STANDING_NEXT
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
        # Peer addresses stay on the host (review item 34); only how many distinct peers.
        "distinct_peers": len({str(d.get("peer")) for d in rc}),
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
    "cli": {"channel unread", "channel subscribe", "channel state", "pty output", "ping",
            "event wait"},
    "mcp": {"termlink_channel_unread", "termlink_channel_subscribe", "termlink_channel_state",
            "termlink_output", "termlink_ping", "termlink_wait"},
}
# `list` and `hub status` are deliberate looks when a person or an agent asks (review A20); they
# count as polling only from loops: background, standing and pre-attribution rows.
POLLING_FROM_LOOPS = {
    "cli": {"list", "hub status"},
    "mcp": {"termlink_list_sessions", "termlink_hub_status"},
}
LOOP_CLASSES = {"background", "standing", "unattributed"}

def is_polling(cls, surf, name):
    return name in POLLING.get(surf, ()) or (cls in LOOP_CLASSES and name in POLLING_FROM_LOOPS.get(surf, ()))

# Caller buckets (review A3/item 35): a value outside the allowlist is "other", an absent one
# "none", so a free-text env value can never leave the host.
HARNESSES = {"claude-code", "codex", "opencode", "antigravity", "gemini-cli", "aider", "goose",
             "cursor", "cline"}
KINDS = {"claude", "codex", "opencode", "antigravity", "gemini", "local", "ollama", "ollama-loop"}
MODEL_FAMILIES = ["opus", "sonnet", "haiku", "fable", "gpt", "o3", "o4", "glm", "gemini", "qwen",
                  "llama", "deepseek", "mistral", "gemma", "phi"]
JOB_TYPES = {"build", "review", "refactor", "test", "specification", "design", "inception",
             "decommission"}
CLASS_RULES = {"explicit", "fw-dispatch", "sidecar-unit", "systemd-agent", "entrypoint-cli",
               "entrypoint-sdk", "unmarked-service", "unmarked-no-tty", "no-rule"}

def harness_bucket(v):
    if not v:
        return "none"
    h = str(v).split("_", 1)[0].lower()
    return h if h in HARNESSES else "other"

def kind_bucket(v):
    if not v:
        return "none"
    k = str(v).strip().lower()
    return k if k in KINDS else "other"

def model_bucket(v):
    """Model family (normalises `opus` and `claude-opus-5-5` alike, review item 5)."""
    if not v:
        return "none"
    m = str(v).lower()
    return next((f for f in MODEL_FAMILIES if f in m), "other")

def job_bucket(v):
    if not v:
        return "unknown"
    j = str(v).strip().lower()
    return j if j in JOB_TYPES else "other"

def sink_specs():
    out = []
    for i, x in enumerate([x for x in os.environ.get("INV_SINKS", "").split(":") if x]):
        label, _, path = x.rpartition("=")
        out.append((label or f"sink{i}", path))
    homes = os.environ.get("TELEMETRY_USER_HOMES")
    if homes is None:
        try:
            import pwd
            entries = [(str(e.pw_uid), e.pw_dir) for e in pwd.getpwall() if e.pw_dir]
        except Exception:
            entries = []
    else:
        entries = [tuple(h.split("=", 1)) for h in homes.split(":") if "=" in h]
    for uid, home in entries:
        out.append((f"user-{uid}", os.path.join(home, ".termlink", "invocation-audit.jsonl")))
    return out

seen_real, sinks_read, unreadable_sinks = set(), [], 0
usage, polling, totals, by_caller, by_rule, bad_usage = {}, {}, {}, {}, {}, 0
for label, base in sink_specs():
    files = []
    for p in (base + ".1", base):
        try:
            if not os.path.exists(p):
                continue
            real = os.path.realpath(p)
        except OSError:
            continue
        if real in seen_real:
            continue
        seen_real.add(real)
        files.append(p)
    if not files:
        continue
    rotated = os.path.exists(base + ".1")
    n, oldest = 0, None
    try:
        for p in files:
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
                    if t is None:
                        continue
                    oldest = t if oldest is None else min(oldest, t)
                    if not (start <= t < end):
                        continue
                    n += 1
                    cls = str(d.get("class") or "unattributed")
                    surf, name = str(d.get("surface")), str(d.get("name"))
                    poll = is_polling(cls, surf, name)
                    block = polling if poll else usage
                    key = f"{surf}:{name}"
                    block.setdefault(cls, {})[key] = block.setdefault(cls, {}).get(key, 0) + 1
                    which = "polling" if poll else "calls"
                    totals.setdefault(cls, {"calls": 0, "polling": 0})[which] += 1
                    ck = "|".join((cls, harness_bucket(d.get("harness")), kind_bucket(d.get("worker_kind")),
                                   model_bucket(d.get("model"))))
                    by_caller.setdefault(ck, {"calls": 0, "polling": 0})[which] += 1
                    rule = d.get("class_rule")
                    rk = "none" if not rule else (rule if rule in CLASS_RULES else "other")
                    by_rule.setdefault(f"{cls}|{rk}", {"calls": 0, "polling": 0})[which] += 1
    except OSError:
        unreadable_sinks += 1
        continue
    # Coverage (review A5/item 32): rows can only be missing if the sink has rotated (a .1
    # exists); then the oldest kept row says how much of the day is left.
    if oldest is None:
        cov = "empty"
    elif rotated and oldest >= end:
        cov = "none"
    elif rotated and oldest > start:
        cov = "partial"
    else:
        cov = "complete"
    sinks_read.append({"label": label, "rows_in_day": n, "coverage": cov,
                       "status": "present" if n else "no-rows"})
if not sinks_read:
    sources["invocation_audit"] = "absent" if not unreadable_sinks else "unreadable"
elif any(x["coverage"] == "none" for x in sinks_read):
    gone = ",".join(x["label"] for x in sinks_read if x["coverage"] == "none")
    sources["invocation_audit"] = f"rotated: this day's rows are no longer on the host (sink {gone})"
    signals["usage"] = None
else:
    sources["invocation_audit"] = "present" if any(x["rows_in_day"] for x in sinks_read) else "no-rows"
    partial = [x["label"] for x in sinks_read if x["coverage"] == "partial"]
    signals["usage"] = {"by_class": usage, "polling": polling, "totals": totals,
                        "by_caller": by_caller, "by_rule": by_rule,
                        "coverage": "partial" if partial else "complete",
                        "sinks": sinks_read, "malformed_lines": bad_usage,
                        "unreadable_sinks": unreadable_sinks}

# ── T-3615: worker-outcome aggregate ────────────────────────────────────────
def read_text(p):
    try:
        with open(p, errors="replace") as f:
            return f.read().strip()
    except OSError:
        return None

NOT_COLLECTED = "not collected on this host (the rollup runs outside a project checkout; slice 4)"
ledger = os.environ.get("LEDGER_ROOT_SET") == "1"
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
        # No recorded kind is "unknown", never assumed claude (review A9).
        raw_kind = read_text(os.path.join(w, "worker_kind.txt")) or meta.get("worker_kind")
        kind = kind_bucket(raw_kind) if raw_kind else "unknown"
        model = model_bucket(meta.get("model_used") or meta.get("model"))
        job = job_bucket(meta.get("task_type"))
        cls = "reviewer" if meta.get("task_type") == "review" else "dispatched"
        # First root wins for a name (the project's own dispatch dir comes first).
        dmeta.setdefault(name, (cls, kind, model, job))
        fin = parse_ts(read_text(os.path.join(w, "finished_at")) or meta.get("ended"))
        started = parse_ts(meta.get("started"))
        if fin is None:
            # Started that day and never finished: a crash, a hang, or still running (item 7).
            if started is None or not (start <= started < end):
                continue
        elif not (start <= fin < end):
            continue
        o = outcomes.setdefault(f"{cls}|{kind}|{model}|{job}", {
            "class": cls, "kind": kind, "model": model, "job_type": job, "runs": 0,
            "unfinished": 0, "exit_codes": {}, "timeouts": 0, "empty_results": 0, "close_states": {}})
        if fin is None:
            o["unfinished"] += 1
            continue
        o["runs"] += 1
        ec = read_text(os.path.join(w, "exit_code"))
        ec = ec if ec not in (None, "") else str(meta.get("exit_code", "none"))
        ec = ec if ec.lstrip("-").isdigit() or ec == "none" else "other"
        o["exit_codes"][ec] = o["exit_codes"].get(ec, 0) + 1
        if os.path.exists(os.path.join(w, "timed_out")):
            o["timeouts"] += 1
        rm = os.path.join(w, "result.md")
        if not os.path.exists(rm) or os.path.getsize(rm) == 0:
            o["empty_results"] += 1
        cs = str(meta.get("close_state") or read_text(os.path.join(w, "close_state")) or "none")
        o["close_states"][cs] = o["close_states"].get(cs, 0) + 1
sources["dispatch_dirs"] = "present" if dirs_seen else "absent"
if not ledger:
    sources["project_ledgers"] = NOT_COLLECTED

close_ledger = None
cpath = os.environ.get("CLOSE_STATES", "")
if cpath and os.path.exists(cpath):
    sources["close_states"] = "present"
    close_ledger = {}
    rows, _ = day_lines(cpath)
    for _, d in rows or []:
        m, j = model_bucket(d.get("model")), job_bucket(d.get("task_type"))
        k = f"{m}|{j}"
        c = close_ledger.setdefault(k, {"model": m, "task_type": j,
                                        "runs": 0, "exit_codes": {}, "outcomes": {}})
        c["runs"] += 1
        ec, oc = str(d.get("exit_code")), str(d.get("outcome"))
        c["exit_codes"][ec] = c["exit_codes"].get(ec, 0) + 1
        c["outcomes"][oc] = c["outcomes"].get(oc, 0) + 1
else:
    sources["close_states"] = "absent"

verdicts = None
vpath, apath = os.environ.get("VERDICTS", ""), os.environ.get("APPLIED", "")
if not ledger and vpath == "none":
    sources["verdicts"] = NOT_COLLECTED
elif vpath and os.path.exists(vpath):
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
        cls, kind, model, job = dmeta.get(str(d.get("dispatch_id") or d.get("worker")),
                                          ("reviewer", "unknown", "none", "review"))
        k = f"{cls}|{kind}|{model}|{job}"
        v = verdicts.setdefault(k, {"class": cls, "kind": kind, "model": model, "job_type": job,
                                    "recorded": 0, "by_verdict": {}, "applied": 0, "withdrawn": 0,
                                    "pending": 0})
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

# Standing workers (review A17/item 36): systemd's NRestarts is a LIFETIME counter, so the report
# carries the delta since the previous on-time rollup (baseline kept on the host, never posted).
# A catch-up or a first observation cannot know the day's restarts: null with a reason.
standing_raw = os.environ.get("STANDING", "none")
try:
    units = json.loads(standing_raw) if standing_raw != "none" else None
except ValueError:
    units = None
if units:
    cur = {str(u.get("unit")): int(u["nrestarts"]) for u in units
           if str(u.get("nrestarts", "")).isdigit()}
    state_path = os.environ.get("STANDING_STATE", "")
    try:
        prev = json.load(open(state_path)).get("nrestarts", {}) if state_path else {}
    except (OSError, ValueError):
        prev = {}
    standing = {"measured": True, "units": len(units),
                "active": sum(1 for u in units if u.get("active") == "active")}
    if os.environ.get("TELEMETRY_CATCH_UP") == "1":
        standing["restarts"] = None
        standing["restarts_reason"] = "late report: restarts are only measured on the on-time day"
    elif not prev:
        standing["restarts"] = None
        standing["restarts_reason"] = "first observation: no baseline yet"
    else:
        # A counter below its baseline means the unit was re-created, and a unit with no
        # baseline is new: either way all of its count is since the last rollup.
        standing["restarts"] = sum(n - prev[u] if u in prev and n >= prev[u] else n
                                   for u, n in cur.items())
        standing["units_new"] = sum(1 for u in cur if u not in prev)
    nxt = os.environ.get("STANDING_NEXT", "")
    if nxt and os.environ.get("STANDING_WRITE") == "1":
        with open(nxt, "w") as f:
            json.dump({"day": day.isoformat(), "nrestarts": cur}, f)
else:
    standing = {"measured": False, "reason": "not measured (no systemd, or no standing-worker unit on this host)"}

signals["worker_outcomes"] = {
    "dispatch": sorted(outcomes.values(), key=lambda o: (o["class"], o["kind"], o["model"], o["job_type"])),
    "close_ledger": None if close_ledger is None else sorted(close_ledger.values(), key=lambda c: (str(c["model"]), str(c["task_type"]))),
    "verdicts": None if verdicts is None else sorted(verdicts.values(), key=lambda v: (v["class"], v["kind"], v["model"], v["job_type"])),
    "standing": standing,
}

report = {
    "schema": "r5-rollup/1", "host": host, "day": day.isoformat(),
    "generated_at": dt.datetime.fromtimestamp(now, dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "signals": signals, "sources": sources,
}
if os.environ.get("TELEMETRY_CATCH_UP") == "1":
    report["catch_up"] = True
print(json.dumps(report, sort_keys=True))
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
    # Only a post the hub accepted ("Posted to <topic> — offset=N") marks the day posted
    # (review item 11/39). A "Queued to" post sits in the offline queue and may yet be
    # dead-lettered, so the day stays a gap and the next --catch-up posts it again; the
    # fleet view keeps one report per host and day, so a late duplicate does no harm.
    if printf '%s\n' "$OUT" | grep -q "^Posted to .*offset="; then
        echo "telemetry-rollup: $HOST $DAY_OUT -> $TOPIC: $OUT"
        echo "$HOST $DAY_OUT" >> "$POSTED_STATE" 2>/dev/null || true
        # The standing-restart baseline moves only with a delivered report, so a lost report
        # does not swallow its restarts.
        if [ -s "$STANDING_NEXT" ]; then cp "$STANDING_NEXT" "$STANDING_STATE" 2>/dev/null || true; fi
    else
        echo "telemetry-rollup: $HOST $DAY_OUT -> $TOPIC: not confirmed, left for catch-up: ${OUT:-no output}"
    fi
    exit 0
fi
echo "telemetry-rollup: post failed for $HOST $DAY_OUT -> $TOPIC: $OUT" >&2
exit 1
