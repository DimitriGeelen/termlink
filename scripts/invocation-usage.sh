#!/usr/bin/env bash
# guard-layer: source
#
# T-2996 (value-review C-45): report per-tool invocation counts.
#
# Reads <runtime_dir>/invocation-audit.jsonl, written at the MCP tool-dispatch
# choke point by crates/termlink-hub/src/invocation_audit.rs.
#
# WHY THIS IS NOT `fw metrics api-usage`. That reader tallies rpc-audit.jsonl by
# RPC METHOD. Every usage question is about TOOLS, and the two do not
# correspond: termlink_agent_top_reacted / top_replied / top_repliers all
# dispatch the same channel.subscribe, and they PAGE, so one tool call emits
# many dispatches. On this host channel.subscribe is the largest method at
# 403,555 dispatches and cannot tell you whether ANY of the 28 off-charter
# analytics tools was ever called. That is why every value-review run capped its
# usage verdicts at UNMEASURED.
#
# SCOPE — read a zero narrowly (T-2680). This reader covers the MCP TOOL
# surface only. CLI verbs and the session-daemon kv.* / session.* blind spot
# (value-review C-30/C-31) are NOT instrumented and are NOT represented here.
# A tool absent from this report was not observed ON THE MCP SURFACE SINCE
# INSTRUMENTATION BEGAN; that is not the same as "unused", and must not be used
# on its own as a deletion warrant.
set -uo pipefail

RUNTIME_DIR="${TERMLINK_RUNTIME_DIR:-}"
SINCE_DAYS=""
JSON=0
SINK=""

usage() {
    cat <<EOF
invocation-usage.sh — per-tool MCP invocation counts (T-2996 / C-45)

  --since-days N   only count records newer than N days
  --sink PATH      read an explicit sink file (default: <runtime_dir>/invocation-audit.jsonl)
  --json           machine-readable envelope
  -h, --help       this text

Exit: 0 report produced · 2 tooling error (sink unreadable / no python3).
Never exits 1: this is a REPORT, not a gate — it has no failing condition.
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --since-days) SINCE_DAYS="${2:-}"; shift 2 ;;
        --sink)       SINK="${2:-}"; shift 2 ;;
        --json)       JSON=1; shift ;;
        -h|--help)    usage; exit 0 ;;
        *) echo "unknown argument: $1" >&2; usage >&2; exit 2 ;;
    esac
done

command -v python3 >/dev/null 2>&1 || { echo "invocation-usage: python3 not found" >&2; exit 2; }

if [ -z "$SINK" ]; then
    # Resolve the runtime dir by the binary's own order (T-2729: never hardcode
    # /tmp/termlink-0 — a guard that inspects a directory the product is not
    # using reports PASS on the one failure mode it exists to catch).
    if [ -n "$RUNTIME_DIR" ]; then :
    elif [ -n "${XDG_RUNTIME_DIR:-}" ]; then RUNTIME_DIR="$XDG_RUNTIME_DIR/termlink"
    elif [ -n "${TMPDIR:-}" ]; then RUNTIME_DIR="$TMPDIR/termlink-$(id -u)"
    else RUNTIME_DIR="/tmp/termlink-$(id -u)"
    fi
    SINK="$RUNTIME_DIR/invocation-audit.jsonl"
fi

SINK="$SINK" SINCE_DAYS="$SINCE_DAYS" JSON="$JSON" python3 - <<'PY'
import json, os, sys, time
sink = os.environ["SINK"]
since = os.environ.get("SINCE_DAYS") or ""
as_json = os.environ.get("JSON") == "1"

SCOPE = ("MCP tool surface only. CLI verbs and session-daemon kv.*/session.* "
         "(C-30/C-31) are NOT instrumented. Absence here means 'not observed on "
         "the MCP surface since instrumentation began', NOT 'unused'.")

if not os.path.exists(sink):
    # A sink that has never been created is not an error and is not a zero
    # census either — say which it is, rather than printing an empty table that
    # reads as "nothing is used".
    msg = ("no invocation sink yet at %s — the instrument has not recorded a "
           "call (no MCP tool invoked since it shipped, or telemetry disabled "
           "via TERMLINK_INVOCATION_AUDIT=0)" % sink)
    if as_json:
        print(json.dumps({"ok": True, "sink": sink, "exists": False,
                          "tools": [], "total": 0, "scope": SCOPE, "note": msg}))
    else:
        print("invocation-usage: " + msg)
        print("SCOPE: " + SCOPE)
    sys.exit(0)

cutoff_ms = None
if since.strip():
    try:
        cutoff_ms = int((time.time() - float(since) * 86400) * 1000)
    except ValueError:
        print("invocation-usage: --since-days must be numeric", file=sys.stderr)
        sys.exit(2)

counts, total, malformed, skipped_old = {}, 0, 0, 0
try:
    with open(sink, "r", errors="replace") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                rec = json.loads(line)
            except Exception:
                malformed += 1          # counted, never silently dropped
                continue
            if cutoff_ms is not None and int(rec.get("ts", 0)) < cutoff_ms:
                skipped_old += 1
                continue
            name = rec.get("name")
            if not isinstance(name, str):
                malformed += 1
                continue
            key = (rec.get("surface", "?"), name)
            counts[key] = counts.get(key, 0) + 1
            total += 1
except OSError as e:
    print("invocation-usage: cannot read %s: %s" % (sink, e), file=sys.stderr)
    sys.exit(2)

rows = sorted(((s, n, c) for (s, n), c in counts.items()),
              key=lambda r: (-r[2], r[1]))

if as_json:
    print(json.dumps({
        "ok": True, "sink": sink, "exists": True,
        "tools": [{"surface": s, "name": n, "count": c} for s, n, c in rows],
        "total": total, "distinct": len(rows),
        "malformed_lines": malformed, "skipped_outside_window": skipped_old,
        "since_days": since or None, "scope": SCOPE,
    }))
else:
    win = (" (last %s day(s))" % since) if since else ""
    print("Per-tool MCP invocations%s — %d call(s), %d distinct tool(s)"
          % (win, total, len(rows)))
    print("sink: %s" % sink)
    if malformed:
        print("malformed lines skipped: %d" % malformed)
    print("")
    for s, n, c in rows:
        print("  %8d  %-4s  %s" % (c, s, n))
    if not rows:
        print("  (no invocations in window)")
    print("")
    print("SCOPE: " + SCOPE)
PY
