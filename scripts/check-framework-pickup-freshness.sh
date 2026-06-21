#!/usr/bin/env bash
# T-2231 — Surface unprocessed framework:pickup filings (G-063 mitigation).
#
# The `framework:pickup` hub topic receives bug-reports / feature-proposals /
# RCAs filed by peer projects (e.g. ring20). termlink has NO automatic consumer
# of that topic (G-063): a high-severity ring20 RCA sat ~27h unprocessed (T-2229)
# precisely because nothing surfaced it. This canary makes new filings visible.
#
# Model: a filing is "surfaced" once an operator/agent has acked the topic up to
# its offset. Firing = any filing on the topic with offset > the last-acked
# marker. ANY new filing surfaces (severity in these payloads is free-form YAML/
# text, so gating on a parsed field would be fragile — see T-2225 false-positive
# lesson); acking makes it quiet again. Pickups are rare, so this is low-noise.
#
# Empty output (in --quiet) = healthy. Workflow:
#   bash scripts/check-framework-pickup-freshness.sh          # see what's new
#   ...process the filings (triage / file tasks / reply)...
#   bash scripts/check-framework-pickup-freshness.sh --ack    # mark surfaced
#
# Exit codes:
#   0  — nothing new (all filings surfaced/acked)
#   1  — unprocessed filing(s) present
#   2  — network/tooling error (could not read the topic)
#
# Usage:
#   check-framework-pickup-freshness.sh           # human-readable, one-shot
#   check-framework-pickup-freshness.sh --json    # JSON for scripting
#   check-framework-pickup-freshness.sh --quiet   # only print on unprocessed (cron)
#   check-framework-pickup-freshness.sh --ack     # bump marker to current max offset
#   check-framework-pickup-freshness.sh --window-days N   # lookback (default 60)
#   check-framework-pickup-freshness.sh --no-heartbeat    # suppress heartbeat touch

set -eu

TOPIC="${FW_PICKUP_TOPIC:-framework:pickup}"
MARKER="${FW_PICKUP_CANARY_MARKER:-.context/working/.framework-pickup-canary.seen-offset}"
HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.framework-pickup-canary.heartbeat}"

FORMAT=human
QUIET=0
HEARTBEAT=1
ACK=0
WINDOW_DAYS=60

while [ $# -gt 0 ]; do
    case "$1" in
        --json)  FORMAT=json ;;
        --quiet) QUIET=1 ;;
        --ack)   ACK=1 ;;
        --no-heartbeat) HEARTBEAT=0 ;;
        --window-days) shift; WINDOW_DAYS="${1:-60}" ;;
        --window-days=*) WINDOW_DAYS="${1#*=}" ;;
        -h|--help) sed -n '2,33p' "$0"; exit 0 ;;
        *) echo "unknown arg: $1" >&2; exit 2 ;;
    esac
    shift
done

# Heartbeat first (prove the canary ran even on error/healthy cycles — T-1723).
if [ "$HEARTBEAT" = 1 ]; then
    mkdir -p "$(dirname "$HEARTBEAT_FILE")" 2>/dev/null || true
    touch -- "$HEARTBEAT_FILE" 2>/dev/null || true
fi

if ! command -v termlink >/dev/null 2>&1; then
    echo "framework-pickup canary: termlink not on PATH (cannot read $TOPIC)" >&2
    exit 2
fi

SEEN=-1
if [ -f "$MARKER" ]; then
    SEEN="$(tr -dc '0-9-' < "$MARKER" 2>/dev/null || echo -1)"
    [ -n "$SEEN" ] || SEEN=-1
fi

NOW_S="$(date +%s)"
SINCE_MS=$(( (NOW_S - WINDOW_DAYS * 86400) * 1000 ))

# Fetch NDJSON. A read failure = canary blind = tooling error (exit 2).
RAW="$(termlink channel subscribe "$TOPIC" --since "$SINCE_MS" --json 2>/dev/null)" || {
    echo "framework-pickup canary: failed to read topic '$TOPIC' (hub down?)" >&2
    exit 2
}

# Parse + render via python. Emits the rendered report on stdout, and on a
# trailing line: "MAXOFF=<n>\tNEW=<count>" for the shell to act on.
PARSED="$(printf '%s' "$RAW" | python3 -c "
import sys, json, base64
seen = int('''$SEEN''')
fmt = '''$FORMAT'''
lines = [l for l in sys.stdin if l.strip()]
entries = []
maxoff = seen
for l in lines:
    try:
        m = json.loads(l)
    except Exception:
        continue
    off = m.get('offset')
    if off is None:
        continue
    off = int(off)
    if off > maxoff:
        maxoff = off
    if off <= seen:
        continue
    md = m.get('metadata', {}) or {}
    mt = m.get('msg_type', '?')
    proj = md.get('from_project') or md.get('source_project') or '?'
    # best-effort: decode payload + sniff a severity keyword (annotation only)
    body = ''
    if m.get('payload_b64'):
        try:
            body = base64.b64decode(m['payload_b64']).decode('utf8', 'replace')
        except Exception:
            body = ''
    elif isinstance(m.get('payload'), str):
        body = m['payload']
    sev = ''
    low = body.lower()
    for kw in ('critical', 'high-sever', 'severity: high', 'severity:high', 'urgent', 'high severity'):
        if kw in low:
            sev = 'HIGH'
            break
    first = ''
    for bl in body.splitlines():
        bl = bl.strip()
        if bl:
            first = bl[:100]
            break
    entries.append({'offset': off, 'msg_type': mt, 'from_project': proj,
                    'severity_hint': sev, 'first_line': first})

entries.sort(key=lambda e: e['offset'])

if fmt == 'json':
    out = {'ok': len(entries) == 0, 'topic': '''$TOPIC''', 'seen_offset': seen,
           'max_offset': maxoff, 'unprocessed': entries}
    print(json.dumps(out))
else:
    if entries:
        print('framework-pickup canary: %d unprocessed filing(s) on $TOPIC (acked up to offset %d)' % (len(entries), seen))
        for e in entries:
            tag = (' [%s]' % e['severity_hint']) if e['severity_hint'] else ''
            print('  off=%d  %s  from=%s%s' % (e['offset'], e['msg_type'], e['from_project'], tag))
            if e['first_line']:
                print('     %s' % e['first_line'])
        print('  → process them, then: bash scripts/check-framework-pickup-freshness.sh --ack')

" 2>/dev/null )" || { echo "framework-pickup canary: parse error" >&2; exit 2; }

# Recover MAXOFF / NEW from the python stderr line (captured separately).
META="$(printf '%s' "$RAW" | python3 -c "
import sys, json
seen=int('''$SEEN''')
maxoff=seen; new=0
for l in sys.stdin:
    if not l.strip(): continue
    try: m=json.loads(l)
    except Exception: continue
    off=m.get('offset')
    if off is None: continue
    off=int(off)
    if off>maxoff: maxoff=off
    if off>seen: new+=1
print('%d %d' % (maxoff, new))
")"
MAXOFF="${META%% *}"
NEW="${META##* }"

if [ "$ACK" = 1 ]; then
    mkdir -p "$(dirname "$MARKER")" 2>/dev/null || true
    printf '%s\n' "$MAXOFF" > "$MARKER"
    [ "$QUIET" = 1 ] || echo "framework-pickup canary: acked up to offset $MAXOFF"
    exit 0
fi

if [ "$NEW" = 0 ]; then
    if [ "$QUIET" != 1 ]; then
        if [ "$FORMAT" = json ]; then printf '%s\n' "$PARSED"
        else echo "framework-pickup canary: healthy — all filings surfaced (acked up to offset $SEEN)"; fi
    fi
    exit 0
fi

# Unprocessed present — print report (always, including --quiet for cron log).
printf '%s\n' "$PARSED"
exit 1
