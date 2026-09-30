#!/usr/bin/env bash
# T-3296 — fixtures for scripts/check-session-leak-freshness.sh and the shared
# detector scripts/lib/session-zombies.py. Hermetic: canned ps + tmux tables and
# scratch sessions dirs; no live process, tmux server or real runtime dir is read.
#
# Weighted to the classes that must NEVER be called zombies — the same detector
# drives the S3b reap, so a false zombie here is a terminated live session there.
set -u

CHK="${CHK:-scripts/check-session-leak-freshness.sh}"
PASS=0; FAIL=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

W="$(mktemp -d)"; trap 'rm -rf "$W"' EXIT
mkdir -p "$W/s1" "$W/s2"
export SESSION_LEAK_DIRS="$W/s1 $W/s2" SESSION_LEAK_HEARTBEAT_FILE="$W/hb/beat"
TL=/root/.cargo/bin/termlink
DAY=90000   # > 24h
# ps: pid ppid etimes args. tmux server 100; systemd-ish parent 1.
cat > "$W/ps" <<EOF
  100     1 $DAY /usr/bin/tmux new-session -d
  201   100 $DAY $TL register --name zomb-a --shell
  202   201 $DAY /usr/bin/bash
  211   100 $DAY $TL register --name zomb-b --shell
  212   211 $DAY /usr/bin/bash
  221   100 $DAY $TL register --name busy-agent --shell
  222   221 $DAY /usr/bin/bash
  223   222 $DAY claude --resume
  231   100 $DAY $TL register --name watched --shell
  232   231 $DAY /usr/bin/bash
  241   100   600 $TL register --name fresh --shell
  242   241   600 /usr/bin/bash
  251     1 $DAY $TL register --name framework-agent-systemd --shell
  252   251 $DAY /usr/bin/bash
  261   100 $DAY $TL register --name claude-master-900 --shell
  262   261 $DAY /usr/bin/bash
  900     1 $DAY /usr/bin/claude-fw --termlink
  271   100 $DAY $TL register --name claude-master-999 --shell
  272   271 $DAY /usr/bin/bash
  281     1 $DAY $TL register --self --name endpoint
EOF
# tmux panes: pane_pid session attached. 251 and 281 are not panes.
cat > "$W/tmux" <<'EOF'
201 tl-zomb-a 0
211 tl-zomb-b 0
221 tl-busy-agent 0
231 tl-watched 1
241 tl-fresh 0
261 tl-claude-master-900 0
271 tl-claude-master-999 0
EOF
export SESSION_LEAK_PS_FILE="$W/ps" SESSION_LEAK_TMUX_FILE="$W/tmux"

DET="${DET:-scripts/lib/session-zombies.py}"
det() { python3 "$DET" --ps "$W/ps" --tmux "$W/tmux" --dir "$W/s1" --dir "$W/s2"; }
J="$(det)"
names="$(printf '%s' "$J" | python3 -c 'import json,sys; print(" ".join(sorted(z["name"] for z in json.load(sys.stdin)["zombie"])))')"

# 1. Exactly the three real zombies (claude-master-999: its launcher pid is dead).
if [ "$names" = "claude-master-999 zomb-a zomb-b" ]; then pass "D1 zombies are exactly the idle, detached, old tmux panes: $names"
else fail "D1 zombie set wrong: '$names'"; fi
field() { printf '%s' "$J" | python3 -c "import json,sys; d=json.load(sys.stdin); v=d['$1']; print(len(v) if isinstance(v,list) else v)"; }
[ "$(field busy)" = "1" ]           && pass "D2 a shell with a child (agent running) is busy, never a zombie" || fail "D2 busy=$(field busy)"
[ "$(field attached)" = "1" ]       && pass "D3 an attached tmux session is never a zombie" || fail "D3 attached=$(field attached)"
[ "$(field young)" = "1" ]          && pass "D4 a session younger than 24h is too young to judge" || fail "D4 young=$(field young)"
[ "$(field unmanaged)" = "1" ]      && pass "D5 a non-tmux register (systemd agent) is unmanaged, never a zombie" || fail "D5 unmanaged=$(field unmanaged)"
[ "$(field launcher_alive)" = "1" ] && pass "D6 claude-master-<pid> with a live claude launcher is never a zombie" || fail "D6 launcher_alive=$(field launcher_alive)"
[ "$(field non_shell)" = "1" ]      && pass "D7 a non-shell register (--self endpoint) is counted, never classified" || fail "D7 non_shell=$(field non_shell)"

# Canary verdicts.
out="$(bash "$CHK" --threshold 5)"; rc=$?
[ "$rc" = "0" ] && echo "$out" | grep -q "healthy — 3 zombie(s) (threshold 5)" && pass "C1 3 zombies under threshold 5 => healthy rc 0" || fail "C1 rc=$rc: $out"
out="$(bash "$CHK" --threshold 2)"; rc=$?
[ "$rc" = "1" ] && echo "$out" | grep -q "zombie sessions: 3 (threshold 2)" && echo "$out" | grep -q "zomb-a" && pass "C2 3 zombies over threshold 2 => FIRING rc 1, names listed" || fail "C2 rc=$rc: $out"
out="$(bash "$CHK" --threshold 5 --quiet)"; rc=$?
[ "$rc" = "0" ] && [ -z "$out" ] && pass "C3 --quiet healthy prints nothing (empty log = healthy)" || fail "C3 rc=$rc out='$out'"
out="$(bash "$CHK" --threshold 2 --json)"; rc=$?
printf '%s' "$out" | python3 -c 'import json,sys; d=json.load(sys.stdin); assert d["zombie_count"]==3 and d["ok"] is False' 2>/dev/null && [ "$rc" = "1" ] \
    && pass "C4 --json carries zombie_count and ok:false when firing" || fail "C4 rc=$rc: $out"

# Orphans: old without .json fires; with .json or young does not.
: > "$W/s1/tl-live.sock.data"; : > "$W/s1/tl-live.json"
: > "$W/s2/tl-new.sock.data"
out="$(bash "$CHK" --threshold 5)"; rc=$?
[ "$rc" = "0" ] && pass "O1 data socket with a registration, or younger than 1h, is not an orphan" || fail "O1 rc=$rc: $out"
: > "$W/s2/tl-gone.sock.data"; touch -d '2 hours ago' "$W/s2/tl-gone.sock.data"
out="$(bash "$CHK" --threshold 5)"; rc=$?
[ "$rc" = "1" ] && echo "$out" | grep -q "orphan data sockets: 1 in $W/s2" && pass "O2 an orphan older than 1h fires, naming its dir" || fail "O2 rc=$rc: $out"
rm -f "$W/s2/tl-gone.sock.data"

# Fail-closed and heartbeat.
out="$(SESSION_LEAK_PS_FILE="$W/nope" bash "$CHK" 2>&1)"; rc=$?
[ "$rc" = "2" ] && pass "F1 unreadable process table => exit 2, never a clean result" || fail "F1 rc=$rc: $out"
rm -rf "$W/hb"; bash "$CHK" --threshold 5 >/dev/null
[ -s "$W/hb/beat" ] && pass "H1 a run writes the heartbeat on exit" || fail "H1 no heartbeat"
rm -rf "$W/hb"; bash "$CHK" --threshold 5 --no-heartbeat >/dev/null
[ ! -e "$W/hb/beat" ] && pass "H2 --no-heartbeat writes none" || fail "H2 heartbeat written"

echo
echo "session-leak canary fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
