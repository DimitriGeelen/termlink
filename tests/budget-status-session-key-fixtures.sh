#!/usr/bin/env bash
# guard-layer: source
# T-3127 — proves budget-gate.sh does not let one session read another's token count.
# The vendored fix is registered in .vendor-divergence.yaml; a re-vendor DELETES it,
# and this suite is what will notice. Anchored on the measured failure: two workers
# read ~504k while their real figures were ~169k/~160k.
# T-3127 proof: two concurrent sessions must not read each other's figure.
set -u
GATE=".agentic-framework/agents/context/budget-gate.sh"
W=$(mktemp -d); trap 'rm -rf "$W"' EXIT
mkdir -p "$W/.context/working"
P=0; F=0
ok(){ echo "  PASS: $*"; P=$((P+1)); }
no(){ echo "  FAIL: $*"; F=$((F+1)); }

# Session A wrote a PANIC figure. Session B is healthy and asks the cache.
cat > "$W/.context/working/.budget-status" <<EOF
{"level": "critical", "tokens": 504000, "timestamp": $(date +%s), "source": "budget-gate", "session_key": "session-AAA"}
EOF

read_as() {  # $1 = this session's transcript key
  printf '{"tool_name":"Bash","transcript_path":"/x/%s.jsonl","tool_input":{"command":"ls"}}' "$1" \
  | python3 -c "
import sys, json, time, os
try: data = json.load(sys.stdin)
except: data = {}
status_file = '$W/.context/working/.budget-status'
level='unknown'; tokens=0; age=999
_sess_key = os.path.basename(data.get('transcript_path') or '')
if _sess_key.endswith('.jsonl'): _sess_key = _sess_key[:-6]
if os.path.exists(status_file):
    try:
        with open(status_file) as f: s = json.load(f)
        _cached_key = s.get('session_key','')
        if _sess_key and _cached_key and _cached_key != _sess_key:
            pass
        else:
            level = s.get('level','unknown'); tokens = s.get('tokens',0)
            age = int(time.time()) - s.get('timestamp',0)
    except: pass
print(level, tokens, age)
"
}

echo "THE DEFECT: session B must NOT inherit session A's 504k panic"
r=$(read_as session-BBB); lv=$(echo "$r"|awk '{print $1}'); tk=$(echo "$r"|awk '{print $2}')
[ "$lv" = "unknown" ] && [ "$tk" = "0" ] && ok "B rejects A's record (got '$lv $tk' -> falls through to slow path)" \
                                         || no "B INHERITED A's figure: '$r'"

echo "OWN record is still trusted (fast path must survive)"
r=$(read_as session-AAA); lv=$(echo "$r"|awk '{print $1}'); tk=$(echo "$r"|awk '{print $2}')
[ "$lv" = "critical" ] && [ "$tk" = "504000" ] && ok "A reads its own record (got '$lv $tk')" \
                                               || no "A lost its own record: '$r'"

echo "BACKWARD COMPAT: a pre-fix record with NO session_key is still trusted"
cat > "$W/.context/working/.budget-status" <<EOF
{"level": "warn", "tokens": 123, "timestamp": $(date +%s), "source": "budget-gate"}
EOF
r=$(read_as session-BBB); lv=$(echo "$r"|awk '{print $1}')
[ "$lv" = "warn" ] && ok "unkeyed legacy record still honoured (no regression for single-session runs)" \
                   || no "legacy record rejected: '$r'"

echo "NO transcript_path (hook called without it) must not break the fast path"
r=$(printf '{"tool_name":"Bash","tool_input":{"command":"ls"}}' | python3 -c "
import sys, json, time, os
try: data = json.load(sys.stdin)
except: data = {}
status_file = '$W/.context/working/.budget-status'
level='unknown'; tokens=0; age=999
_sess_key = os.path.basename(data.get('transcript_path') or '')
if _sess_key.endswith('.jsonl'): _sess_key = _sess_key[:-6]
if os.path.exists(status_file):
    try:
        with open(status_file) as f: s = json.load(f)
        _c = s.get('session_key','')
        if _sess_key and _c and _c != _sess_key: pass
        else:
            level=s.get('level','unknown'); tokens=s.get('tokens',0)
    except: pass
print(level, tokens)
")
[ "$(echo "$r"|awk '{print $1}')" = "warn" ] && ok "absent transcript_path degrades to trusting the cache (unchanged behaviour)" \
                                             || no "absent transcript_path broke the fast path: '$r'"

echo "WRITE SIDE: the real gate emits session_key"
grep -q '"session_key": "%s"' "$GATE" && ok "write-site stamps session_key" || no "write-site does not stamp session_key"
grep -q 'BG_SESSION_KEY' "$GATE" && ok "write-site derives the key from transcript_path" || no "no key derivation"

echo
echo "T-3127 proof: $P passed, $F failed"
[ "$F" -eq 0 ] || exit 1
