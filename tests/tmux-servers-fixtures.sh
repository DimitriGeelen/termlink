#!/usr/bin/env bash
# T-3552 — fixtures for scripts/lib/tmux_servers.py and its use in session-zombies.py.
# Starts PRIVATE tmux servers only, under a temp TMUX_TMPDIR, and kills them only through
# their own -L name (055 T-537: a test that fell back to the default socket killed the fleet).
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LIB="$ROOT/scripts/lib"
pass=0; fail=0
ok()  { echo "ok   $1"; pass=$((pass+1)); }
bad() { echo "FAIL $1"; fail=$((fail+1)); }

if ! command -v tmux >/dev/null 2>&1; then echo "skip: tmux not installed"; exit 0; fi
W="$(mktemp -d)"
export TMUX_TMPDIR="$W/tmux"; mkdir -p "$TMUX_TMPDIR"
unset TMUX
SRV="tl-fx-$(printf '%08x' $$ | cut -c1-8)"
cleanup() { tmux -L "$SRV" kill-server 2>/dev/null; rm -rf "$W"; }
trap cleanup EXIT

tmux -L "$SRV" -f /dev/null new-session -d -s sess 'sleep 600' || { echo "could not start a private server"; exit 2; }
# A socket that is not a TermLink name, and one whose server is gone: neither may be used as a live server.
SDIR="$TMUX_TMPDIR/tmux-$(id -u)"
touch "$SDIR/not-ours" "$SDIR/tl-dead-0000beef"

names="$(cd "$LIB" && python3 -c 'import tmux_servers; print(" ".join(tmux_servers.termlink_servers()))')"
case " $names " in *" $SRV "*) ok "1 the private project server is enumerated" ;; *) bad "1 enumerate: $names" ;; esac
case " $names " in *" not-ours "*) bad "2 a non-TermLink socket name was enumerated" ;; *) ok "2 a non-TermLink socket name is never used" ;; esac

dead="$(cd "$LIB" && python3 -c 'import tmux_servers; print([s for s, t in tmux_servers.run_on_each(["list-sessions"]) if t is None])')"
case "$dead" in *tl-dead-0000beef*) ok "3 a server that does not answer is reported, not read as empty" ;; *) bad "3 dead server: $dead" ;; esac

# session-zombies: the pane on the project server is seen and named <server>:<session>.
PANE_PID="$(tmux -L "$SRV" list-panes -a -F '#{pane_pid}' | head -1)"
printf '%s 1 999999 /usr/bin/termlink register --shell --name fx-zombie\n' "$PANE_PID" > "$W/ps.txt"
out="$(python3 "$LIB/session-zombies.py" --ps "$W/ps.txt" --min-age 60)"
if python3 -c 'import json,sys; d=json.loads(sys.argv[1]); z=d["zombie"]; assert len(z)==1 and z[0]["tmux_session"]==sys.argv[2]+":sess", d' "$out" "$SRV"; then
    ok "4 a register on a project server is classified (zombie), not invisible"
else bad "4 session-zombies: $out"; fi

# Control: the same process table with only the default server consulted (the pre-T-3552
# shape) leaves the register outside every pane: unmanaged, never a zombie.
out="$(python3 "$LIB/session-zombies.py" --ps "$W/ps.txt" --tmux /dev/null --min-age 60)"
if python3 -c 'import json,sys; d=json.loads(sys.argv[1]); assert not d["zombie"] and len(d["unmanaged"])==1, d' "$out"; then
    ok "5 control: without project servers the same register is invisible as a zombie"
else bad "5 control: $out"; fi

# env-leak scan: a project variable in a project server's GLOBAL env is reported with its server.
tmux -L "$SRV" set-environment -g TASKS_DIR /opt/other/tk
echo "[]" > "$W/list.json"; mkdir -p "$W/proc"
out="$(python3 "$LIB/env-leak-scan.py" --list-json "$W/list.json" --proc "$W/proc")"
if python3 -c 'import json,sys; d=json.loads(sys.argv[1]); assert {"name":"TASKS_DIR","value":"/opt/other/tk","server":sys.argv[2]} in d["server_global"], d' "$out" "$SRV"; then
    ok "6 env-leak scan reads every project server's global env"
else bad "6 env-leak scan: $out"; fi

echo "---"
echo "tmux-servers fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
