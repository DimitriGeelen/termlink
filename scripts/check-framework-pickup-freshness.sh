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

# T-2816: who WE are, for the own-filing filter below.
#
# Deliberately a CONSTANT, not `basename "$PROJECT_ROOT"`. A path-derived slug is wrong
# the moment this runs inside a git worktree (it yields the worktree name), which is
# exactly the defect filed upstream as T-2815 — repeating it here would silently disable
# the filter in precisely the sessions that post the most filings.
#
# Failure direction is safe: if the project is ever renamed and this constant is not
# updated, the filter stops matching and our own posts merely become visible again —
# noisy, never silent.
SELF_PROJECT="${FW_PICKUP_SELF_PROJECT:-010-termlink}"

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

# Test seam (PL-213): feed canned `channel subscribe --json` NDJSON so the fixtures can
# exercise the filter without a live hub. Only consulted when the file is readable.
TEST_NDJSON="${FW_PICKUP_TEST_NDJSON:-}"

if [ -z "$TEST_NDJSON" ] && ! command -v termlink >/dev/null 2>&1; then
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
if [ -n "$TEST_NDJSON" ]; then
    RAW="$(cat -- "$TEST_NDJSON" 2>/dev/null)" || {
        echo "framework-pickup canary: cannot read test NDJSON: $TEST_NDJSON" >&2
        exit 2
    }
else
    RAW="$(termlink channel subscribe "$TOPIC" --since "$SINCE_MS" --json 2>/dev/null)" || {
        echo "framework-pickup canary: failed to read topic '$TOPIC' (hub down?)" >&2
        exit 2
    }
fi

# Parse + render via python. Emits the rendered report on stdout, and on a
# trailing line: "MAXOFF=<n>\tNEW=<count>" for the shell to act on.
PARSED="$(printf '%s' "$RAW" | python3 -c "
import sys, json, base64
seen = int('''$SEEN''')
fmt = '''$FORMAT'''
self_project = '''$SELF_PROJECT'''
lines = [l for l in sys.stdin if l.strip()]
entries = []
own = []
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
    # T-2816 attribution chain. agent_id is accepted as a fallback because some of our
    # own filings set that key instead of from_project; both carry the project name.
    proj = md.get('from_project') or md.get('source_project') or md.get('agent_id') or '?'
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
    rec = {'offset': off, 'msg_type': mt, 'from_project': proj,
           'severity_hint': sev, 'first_line': first}
    # T-2816: our OWN outbound filings are not inbound work. They are counted and
    # reported, but never fire — otherwise filing a bug report upstream makes this
    # canary fire at us, and the --ack needed to clear the echo also acks any genuine
    # inbound filing that landed in between (the G-063 failure, via its own mitigation).
    #
    # Unknown attribution ('?') deliberately still fires: we cannot prove it is ours,
    # and a false fire is cheap while a false silence is the whole reason G-063 exists.
    if proj == self_project:
        own.append(rec)
    else:
        entries.append(rec)

entries.sort(key=lambda e: e['offset'])
own.sort(key=lambda e: e['offset'])

if fmt == 'json':
    out = {'ok': len(entries) == 0, 'topic': '''$TOPIC''', 'seen_offset': seen,
           'max_offset': maxoff, 'unprocessed': entries,
           'own_count': len(own), 'self_project': self_project, 'own': own}
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
    # T-2816: ALWAYS report suppression, including on the healthy path. A quiet canary
    # must never be ambiguous between 'nothing inbound' and 'the filter ate something'.
    #
    # NOTE: this whole python program is embedded in a DOUBLE-QUOTED shell string, so a
    # literal double quote anywhere in it -- comments included -- closes that string and
    # silently truncates the program from that point on. It fails quietly: the truncated
    # prefix still runs, so the counts look right while the tail of the report vanishes.
    # Use single quotes in this block, always.
    if own:
        print('  (%d own filing(s) from %s not counted — outbound, not inbound work)'
              % (len(own), self_project))

" 2>/dev/null )" || { echo "framework-pickup canary: parse error" >&2; exit 2; }

# Recover MAXOFF / NEW from a second parse.
#
# T-2816: this pass is the actual FIRING GATE (NEW==0 -> exit 0), so it must apply the
# same own-filing filter as the report above. It previously counted every new offset
# regardless of author, which is why filtering the report alone would have changed what
# the canary SAYS without changing what it DOES.
#
# MAXOFF still tracks the true maximum across ALL filings, own ones included — --ack must
# advance past our own posts too, or they would be re-counted forever.
META="$(printf '%s' "$RAW" | python3 -c "
import sys, json
seen=int('''$SEEN''')
self_project='''$SELF_PROJECT'''
maxoff=seen; new=0
for l in sys.stdin:
    if not l.strip(): continue
    try: m=json.loads(l)
    except Exception: continue
    off=m.get('offset')
    if off is None: continue
    off=int(off)
    if off>maxoff: maxoff=off
    if off<=seen: continue
    md=m.get('metadata') or {}
    proj=md.get('from_project') or md.get('source_project') or md.get('agent_id') or '?'
    if proj==self_project: continue
    new+=1
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
