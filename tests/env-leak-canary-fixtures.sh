#!/usr/bin/env bash
# T-3551 — fixtures for scripts/check-env-leak-freshness.sh (SEV-1 T-3549 detector).
# Hermetic: a fake /proc, canned `termlink list --json` and canned tmux global env through the
# ENV_LEAK_* seams. No live hub, no real processes, no tmux server.
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CANARY="$ROOT/scripts/check-env-leak-freshness.sh"
SCAN="$ROOT/scripts/lib/env-leak-scan.py"
W="$(mktemp -d)"
trap 'rm -rf "$W"' EXIT
pass=0; fail=0
ok()  { echo "ok   $1"; pass=$((pass+1)); }
bad() { echo "FAIL $1"; fail=$((fail+1)); }

# Two projects, a worktree of A, and a directory in no project.
A="$W/a"; B="$W/b"; WT="$W/a-wt"; NONE="$W/none"
mkdir -p "$A/.git/worktrees/wt" "$A/sub" "$B" "$WT" "$NONE"
touch "$A/.framework.yaml" "$B/.framework.yaml" "$WT/.framework.yaml"
echo "gitdir: $A/.git/worktrees/wt" > "$WT/.git"

# session <pid> <cwd> [VAR=VALUE ...]: one registered session whose root process has that env.
LIST="$W/list.json"; PROC="$W/proc"; TMUXENV="$W/tmux.env"
reset() { rm -rf "$PROC"; mkdir -p "$PROC"; echo "[]" > "$LIST"; : > "$TMUXENV"; }
session() {
    local pid="$1" cwd="$2"; shift 2
    mkdir -p "$PROC/$pid"
    { printf 'HOME=/root\0'; for kv in "$@"; do printf '%s\0' "$kv"; done; } > "$PROC/$pid/environ"
    echo "$cwd" > "$PROC/$pid/cwd"
    python3 - "$LIST" "$pid" "$cwd" <<'PY'
import json, sys
p, pid, cwd = sys.argv[1], int(sys.argv[2]), sys.argv[3]
d = json.load(open(p)); d.append({"id": f"tl-{pid}", "display_name": f"s{pid}", "pid": pid, "metadata": {"cwd": cwd}})
json.dump(d, open(p, "w"))
PY
}
run() { ENV_LEAK_LIST_JSON="$LIST" ENV_LEAK_PROC_DIR="$PROC" ENV_LEAK_TMUX_ENV_FILE="$TMUXENV" \
        bash "$CANARY" --no-heartbeat "$@" > "$W/out" 2>&1; echo $?; }

reset; session 11 "$A" "PROJECT_ROOT=$A" "TASKS_DIR=$A/.tasks"
[ "$(run)" = 0 ] && grep -q healthy "$W/out" && ok "1 own-project variables are healthy" || bad "1 own project: $(cat "$W/out")"

reset; session 12 "$A/sub" "TASKS_DIR=$B/.tasks" "CONTEXT_DIR=$B/.context"
[ "$(run)" = 1 ] && grep -q "TASKS_DIR=$B/.tasks" "$W/out" && grep -q "CONTEXT_DIR=" "$W/out" \
    && ok "2 another project's TASKS_DIR/CONTEXT_DIR fire (the 2026-10-22 shape)" || bad "2 foreign: $(cat "$W/out")"

reset; session 13 "$A" "TASKS_DIR=$NONE/.tasks"
[ "$(run)" = 0 ] && ok "3 a value outside every project (test temp dir) is not judged" || bad "3 temp value: $(cat "$W/out")"

reset; session 14 "$NONE" "PROJECT_ROOT=$B"
[ "$(run)" = 0 ] && ok "4 a session outside every project is not judged" || bad "4 no own project: $(cat "$W/out")"

reset; session 15 "$WT" "PROJECT_ROOT=$A"
[ "$(run)" = 0 ] && ok "5 a worktree counts as its main checkout" || bad "5 worktree: $(cat "$W/out")"

reset; session 16 "$A" "FRAMEWORK_ROOT=$B/.agentic-framework"
[ "$(run)" = 0 ] && ok "6 FRAMEWORK_ROOT in a session is not judged (shared tooling)" || bad "6 framework_root: $(cat "$W/out")"

reset; printf 'PROJECT_ROOT=%s\n' "$B" > "$TMUXENV"
[ "$(run)" = 1 ] && grep -q "GLOBAL env: PROJECT_ROOT" "$W/out" && ok "7 a project variable in the server's global env fires" || bad "7 server global: $(cat "$W/out")"

reset; printf -- '-PROJECT_ROOT\nTERM=xterm\n' > "$TMUXENV"
[ "$(run)" = 0 ] && ok "8 a removed (-VAR) server global and host vars do not fire" || bad "8 removed global: $(cat "$W/out")"

reset; printf 'FRAMEWORK_ROOT=%s\n' "$B" > "$TMUXENV"
[ "$(run)" = 1 ] && ok "9 FRAMEWORK_ROOT in the server's global env fires" || bad "9 server framework_root: $(cat "$W/out")"

reset; session 17 "$A" "TASKS_DIR=$B/.tasks"
rc="$(run --json)"
[ "$rc" = 1 ] && python3 -c "import json,sys;d=json.load(open(sys.argv[1]));assert d['ok'] is False and len(d['foreign'])==1 and d['foreign'][0]['value_project']==sys.argv[2]" "$W/out" "$B" \
    && ok "10 --json carries ok:false and the foreign variable's project" || bad "10 json: $(cat "$W/out")"

reset; session 18 "$A" "TASKS_DIR=$B/.tasks"; rm -f "$PROC/18/environ"
[ "$(run)" = 0 ] && grep -q "unreadable=1" "$W/out" && ok "11 an unreadable process is counted, not guessed" || bad "11 unreadable: $(cat "$W/out")"

reset; echo "not json" > "$LIST"
[ "$(run)" = 2 ] && ok "12 an unreadable session list is a tooling error, never healthy" || bad "12 bad list: $(cat "$W/out")"

reset; HB="$W/hb"
ENV_LEAK_LIST_JSON="$LIST" ENV_LEAK_PROC_DIR="$PROC" ENV_LEAK_TMUX_ENV_FILE="$TMUXENV" ENV_LEAK_HEARTBEAT_FILE="$HB" \
    bash "$CANARY" --quiet > "$W/out" 2>&1
[ -s "$HB" ] && [ ! -s "$W/out" ] && ok "13 heartbeat written on exit; --quiet prints nothing when healthy" || bad "13 heartbeat/quiet"

# Mutant: disable the project comparison. Case 2 must then go silent, proving it bites.
MUT="$W/mut"; mkdir -p "$MUT/scripts/lib"
cp "$CANARY" "$MUT/scripts/"; sed 's/if vp is not None and vp != own:/if False:/' "$SCAN" > "$MUT/scripts/lib/env-leak-scan.py"
grep -q "if False:" "$MUT/scripts/lib/env-leak-scan.py" || bad "M1 mutant did not apply"
reset; session 19 "$A" "TASKS_DIR=$B/.tasks"
rc="$(ENV_LEAK_LIST_JSON="$LIST" ENV_LEAK_PROC_DIR="$PROC" ENV_LEAK_TMUX_ENV_FILE="$TMUXENV" bash "$MUT/scripts/check-env-leak-freshness.sh" --no-heartbeat >/dev/null 2>&1; echo $?)"
[ "$rc" = 0 ] && ok "M1 mutant without the comparison misses the leak (so case 2 is load-bearing)" || bad "M1 mutant still fired rc=$rc"

echo "---"
echo "env-leak canary fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
