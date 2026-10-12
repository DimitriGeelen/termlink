#!/usr/bin/env bash
# T-3425 fixtures: tmux socket-path guard + root-mode canary. Hermetic: never runs
# tmux, never stats or touches the real `/`.
set -u
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
G="$HERE/scripts/hooks/check-tmux-socket-path.sh"
C="$HERE/scripts/check-root-mode-freshness.sh"
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok() { PASS=$((PASS+1)); echo "  ok   $1"; }
bad() { FAIL=$((FAIL+1)); echo "  FAIL $1"; }

guard() {  # guard <expected-rc> <name> <command>
    local js; js=$(python3 -c 'import json,sys;print(json.dumps({"tool_name":"Bash","tool_input":{"command":sys.argv[1]}}))' "$3")
    TMUX_GUARD_INPUT="$js" bash "$G" >/dev/null 2>&1; local rc=$?
    [ "$rc" = "$1" ] && ok "$2 (rc $rc)" || bad "$2 (rc $rc, want $1)"
}
mkdir -p "$TMP/dir"; : > "$TMP/file"
python3 -c 'import socket,sys;s=socket.socket(socket.AF_UNIX);s.bind(sys.argv[1])' "$TMP/real.sock"

echo "tmux socket-path guard"
guard 2 "G1 the incident: tmux -S /proc/<pid>/cwd"          "tmux -S /proc/2908/cwd 2>/dev/null"
guard 2 "G2 -S on an existing directory"                    "tmux -S $TMP/dir ls"
guard 2 "G3 -S on an existing regular file"                 "tmux -S $TMP/file ls"
guard 2 "G4 attached form -S<path>"                         "tmux -S$TMP/dir ls"
guard 2 "G5 inside a chain, after cd"                       "cd /tmp && ps -p 1; tmux -S /proc/self/root show-environment -g"
guard 2 "G6 full path to the binary"                        "/usr/bin/tmux -S $TMP/dir ls"
guard 2 "G7 -L with a path"                                 "tmux -L /proc/1/cwd ls"
guard 0 "G8 -S on a real socket"                            "tmux -S $TMP/real.sock ls"
guard 0 "G9 -S on a path that does not exist yet"           "tmux -S $TMP/new.sock new-session -d"
guard 0 "G10 -S with an unresolved variable"                'tmux -S $S/t.sock new-session -d'
guard 0 "G11 -L with a plain name"                          "tmux -L fixture ls"
guard 0 "G12 no tmux at all"                                "ls /proc/1/cwd"
guard 0 "G13 tmux mentioned only as an argument"            "grep -n 'tmux -S' scripts/*.sh"
guard 0 "G14 /proc after the tmux command ends"             "tmux ls; cat /proc/1/environ"
guard 0 "G15 unparseable quoting fails open"                "tmux -S '/proc/1/cwd"
guard 0 "G16 non-JSON input fails open"                     ""
TMUX_GUARD_INPUT='not json' bash "$G" >/dev/null 2>&1 && ok "G17 garbage input fails open" || bad "G17 garbage input fails open"

echo "root-mode canary"
canary() {  # canary <expected-rc> <name> <path>
    local out; out=$(ROOT_MODE_PATH="$3" bash "$C" --quiet --no-heartbeat 2>&1); local rc=$?
    [ "$rc" = "$1" ] && ok "$2 (rc $rc)" || bad "$2 (rc $rc, want $1): $out"
}
if [ "$(id -u)" = 0 ]; then
    mkdir -p "$TMP/r755" "$TMP/r644"; chmod 755 "$TMP/r755"; chmod 644 "$TMP/r644"
    canary 0 "C1 755 root directory is healthy" "$TMP/r755"
    canary 1 "C2 the incident: 644 fires"       "$TMP/r644"
    out=$(ROOT_MODE_PATH="$TMP/r755" bash "$C" --quiet --no-heartbeat); [ -z "$out" ] && ok "C3 --quiet healthy prints nothing" || bad "C3 --quiet printed: $out"
    canary 1 "C4 a file is not a root directory" "$TMP/file"
    chown 65534 "$TMP/r755" 2>/dev/null && canary 1 "C5 755 directory not owned by root fires" "$TMP/r755"
else
    echo "  skip C1-C5 (need root to set ownership)"
fi
canary 2 "C6 unstat-able path is tooling, never healthy" "$TMP/missing"
HBF="$TMP/hb"; ROOT_MODE_PATH="$TMP/missing" ROOT_MODE_HEARTBEAT_FILE="$HBF" bash "$C" --quiet >/dev/null 2>&1
[ -f "$HBF" ] && ok "C7 heartbeat written on exit" || bad "C7 heartbeat written on exit"

echo "Pass: $PASS  Fail: $FAIL"
[ "$FAIL" = 0 ]
