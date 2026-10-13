#!/usr/bin/env bash
# T-3438 (arc-011 P7) — hermetic fixtures for tl-claude.sh --sidecar: the
# launcher starts the agent's sidecar through the supervise script and hands the
# session TERMLINK_SIDECAR_SOCKET only when the sidecar is really up. Sources
# tl-claude.sh in lib mode with a stub supervise script; no hub, no PTY, no host
# state (scratch dirs only).
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
T=$(mktemp -d)
trap 'rm -rf "$T"' EXIT
PASS=0; FAIL=0
ok() { PASS=$((PASS + 1)); echo "ok   $*"; }
bad() { FAIL=$((FAIL + 1)); echo "FAIL $*"; }
check() { if eval "$2"; then ok "$1"; else bad "$1"; fi; }

mkdir -p "$T/env" "$T/rt/sidecar"
# A stub supervise script: records its arguments; exit code from $T/start-rc.
cat > "$T/supervise" <<EOF
#!/usr/bin/env bash
echo "\$*" >> "$T/calls"
exit "\$(cat "$T/start-rc" 2>/dev/null || echo 0)"
EOF
chmod +x "$T/supervise"
mksock() { python3 -c 'import socket,sys; s=socket.socket(socket.AF_UNIX); s.bind(sys.argv[1])' "$1"; }

# run <sidecar 0|1> <agent>: prints the composed claude command, then "STDERR:" lines.
run() {
    (
        TL_CLAUDE_LIB=1
        # shellcheck disable=SC1091
        . "$ROOT/scripts/tl-claude.sh" >/dev/null 2>&1
        SIDECAR="$1"; AGENT_ID="$2"; REACHABLE=0; CLAUDE_ARGS=(--resume)
        export SIDECAR_ENV_DIR="$T/env" TL_CLAUDE_SIDECAR_SUPERVISE="$T/supervise"
        prepare_sidecar 2> "$T/err" > "$T/out" || true
        build_claude_cmd
    )
}
envfile() { printf 'PROJECT_DIR=/x\nAGENT=%s\nTERMLINK_RUNTIME_DIR=%s\n' "$2" "$T/rt" > "$T/env/$1.env"; }

# 1 not asked: nothing happens.
out=$(run 0 claude-termlink)
check "1a without --sidecar no socket is set and nothing is started" '! grep -q TERMLINK_SIDECAR_SOCKET <<< "$out" && [ ! -e "$T/calls" ]'

# 2 the happy path.
envfile 010-termlink--claude-termlink claude-termlink
mksock "$T/rt/sidecar/claude-termlink.send.sock"
out=$(run 1 claude-termlink)
check "2a the sidecar is started by its settings name" 'grep -qx "start 010-termlink--claude-termlink" "$T/calls"'
check "2b the session gets TERMLINK_SIDECAR_SOCKET pointing at the send socket" 'grep -q "TERMLINK_SIDECAR_SOCKET=$T/rt/sidecar/claude-termlink.send.sock" <<< "$out"'
check "2c and still runs claude with its arguments" 'grep -q "claude --resume" <<< "$out"'

# 3 a failed start: no socket, loud.
echo 1 > "$T/start-rc"; : > "$T/calls"
out=$(run 1 claude-termlink)
check "3a a failed sidecar start leaves the socket unset" '! grep -q TERMLINK_SIDECAR_SOCKET <<< "$out"'
check "3b and says the session runs WITHOUT mail" 'grep -q "WITHOUT mail" "$T/err"'
echo 0 > "$T/start-rc"

# 4 started but no socket: unset, loud.
rm -f "$T/rt/sidecar/claude-termlink.send.sock"
out=$(run 1 claude-termlink)
check "4a a start that leaves no socket does not hand one over" '! grep -q TERMLINK_SIDECAR_SOCKET <<< "$out" && grep -q "is not a socket" "$T/err"'

# 5 no settings file / two settings files.
out=$(run 1 claude-termlink-alt)
check "5a no settings file for the agent: no socket, names the W1 action" '! grep -q TERMLINK_SIDECAR_SOCKET <<< "$out" && grep -q "W1 runme action" "$T/err"'
mksock "$T/rt/sidecar/claude-termlink.send.sock"
envfile other-project--claude-termlink claude-termlink; : > "$T/calls"
out=$(run 1 claude-termlink)
check "5b two projects claiming the agent: refuses to guess, starts nothing" '! grep -q TERMLINK_SIDECAR_SOCKET <<< "$out" && grep -q "more than one settings file" "$T/err" && [ ! -s "$T/calls" ]'

# 6 the project settings carry the four guarded arc-011 hooks (R-48), so a
#   framework upgrade that dropped them fails here instead of silently.
for ev in PostToolUse:post-tool-use SessionStart:session-start Stop:stop SessionEnd:session-end; do
    check "6 settings.json ${ev%%:*} runs the guarded sidecar hook ${ev#*:}" \
        "python3 -c 'import json,sys; d=json.load(open(\"$ROOT/.claude/settings.json\")); c=\"[ -z \\\"\${TERMLINK_SIDECAR_SOCKET:-}\\\" ] || exec termlink sidecar hook ${ev#*:}\"; sys.exit(0 if any(h.get(\"command\")==c for g in d[\"hooks\"].get(\"${ev%%:*}\",[]) for h in g.get(\"hooks\",[])) else 1)'"
done

echo "---"
echo "tl-claude sidecar fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = 0 ]
