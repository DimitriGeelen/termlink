#!/usr/bin/env bash
# guard-layer: source
# T-3499 — fixtures for scripts/lib/hub-swap-remote.sh (used by hub-binary-swap.sh).
# The generated probe and relaunch scripts run locally against a FAKE /proc and
# stand-ins for pgrep/pkill/setsid/sleep/ss on PATH. Nothing touches a real hub,
# a real /proc entry or the real runtime dir.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
PASS=0; FAIL=0
ok() { PASS=$((PASS + 1)); echo "ok   $*"; }
bad() { FAIL=$((FAIL + 1)); echo "FAIL $*"; }
check() { if eval "$2"; then ok "$1"; else bad "$1"; fi; }
# shellcheck disable=SC1091
. "$ROOT/scripts/lib/hub-swap-remote.sh"

WEIRD="a b \"q\" \$HOME
line2"
mkdir -p "$T/proc/201" "$T/proc/202" "$T/bin"
printf '%s\0' PATH=/usr/bin FOO=bar TERMLINK_SWEEP_INTERVAL_SECS=3600 "TERMLINK_WEIRD=$WEIRD" \
    TERMLINK_RUNTIME_DIR=/var/lib/termlink > "$T/proc/201/environ"
printf '%s\0' /usr/local/bin/termlink hub start --tcp 0.0.0.0:9100 --label "my hub" > "$T/proc/201/cmdline"
printf '%s\0' termlink hub start > "$T/proc/202/cmdline"      # no environ file
cat > "$T/bin/pgrep" <<'EOS'
#!/usr/bin/env bash
echo "${FAKE_HUB_PID:-}"
EOS
printf '#!/usr/bin/env bash\nexit 0\n' > "$T/bin/pkill"; cp "$T/bin/pkill" "$T/bin/sleep"; cp "$T/bin/pkill" "$T/bin/ss"
cat > "$T/bin/setsid" <<'EOS'
#!/usr/bin/env bash
env -0 > "$REC.env"; printf '%s\0' "$@" > "$REC.argv"
EOS
chmod +x "$T"/bin/*
export HUB_PROC_ROOT="$T/proc"

write_swap_probe "$T/probe.sh"
check "1 the probe is valid bash and embeds the T-3497 reader (no copy)" 'bash -n "$T/probe.sh" && [ "$(grep -c "^hub_termlink_env() {" "$T/probe.sh")" = 1 ] && ! grep -q "TERMLINK_\[A-Za-z" "$ROOT/scripts/lib/hub-swap-remote.sh"'
out=$(FAKE_HUB_PID=201 PATH="$T/bin:$PATH" bash "$T/probe.sh" 2>"$T/err"); rc=$?
check "2 the probe on a readable hub returns 0 with env + argv" '[ "$rc" = 0 ] && grep -q "^HUB_ENV_B64=" <<< "$out" && grep -q "^HUB_ARGV_B64=" <<< "$out"'
out2=$(FAKE_HUB_PID=202 PATH="$T/bin:$PATH" bash "$T/probe.sh" 2>"$T/err"); rc=$?
check "3 an unreadable environ makes the probe fail (rc != 0) with no data" '[ "$rc" != 0 ] && ! grep -q "HUB_ENV_B64" <<< "$out2" && grep -q "cannot read" "$T/err"'

ENVB=$(grep '^HUB_ENV_B64=' <<< "$out" | cut -d= -f2-); ARGVB=$(grep '^HUB_ARGV_B64=' <<< "$out" | cut -d= -f2-)
# A rollback-mode run exercises the real relaunch path end to end on fakes.
run_relaunch() { # <script> <rec>
    printf '#!/bin/sh\necho 0.0.0\n' > "$T/termlink"; chmod +x "$T/termlink"; : > "$T/termlink.bak"
    ( cd "$T" && REC="$2" TERMLINK_CALLER_LEAK=exec-session PATH="$T/bin:$PATH" bash "$1" ) > "$T/run.out" 2>&1
    local rc=$?
    # T-3563: the relaunch starts the hub with `setsid ... &` and every wait in it is faked to return
    # at once, so the fake setsid may not have written its record yet. Wait (bounded) for .argv,
    # which it writes last; an absent record then fails the checks instead of racing them.
    for _ in $(seq 1 50); do [ -s "$2.argv" ] && break; sleep 0.1; done
    return $rc
}
has_env() { local -a e=(); local x; mapfile -d '' e < "$1.env" 2>/dev/null; for x in "${e[@]}"; do [ "$x" = "$2" ] && return 0; done; return 1; }
write_swap_relaunch "$T/rb.sh" rollback "$T/staged" 201 "$ENVB" "$ARGVB" "$T/termlink" "$T/hub.log"
run_relaunch "$T/rb.sh" "$T/r1"; rc=$?
check "4 the relaunch script runs clean on fakes" '[ "$rc" = 0 ] && [ -s "$T/r1.env" ]'
check "5 TERMLINK_SWEEP_INTERVAL_SECS=3600 reaches the relaunched hub" 'has_env "$T/r1" TERMLINK_SWEEP_INTERVAL_SECS=3600'
check "6 a value with spaces/quotes/newline arrives byte-for-byte" 'has_env "$T/r1" "TERMLINK_WEIRD=$WEIRD"'
check "7 non-TERMLINK variables and the exec session TERMLINK_* do not leak" '! has_env "$T/r1" FOO=bar && ! tr "\0" "\n" < "$T/r1.env" | grep -q "^TERMLINK_CALLER_LEAK="'
check "8 an argument with a space arrives as ONE argv word" '[ "$(tr "\0" "|" < "$T/r1.argv")" = "nohup|$T/termlink|hub|start|--tcp|0.0.0.0:9100|--label|my hub|" ]'
# fail-closed: a corrupt/empty env payload must not relaunch
write_swap_relaunch "$T/rb2.sh" rollback "$T/staged" 201 "" "$ARGVB" "$T/termlink" "$T/hub.log"
rm -f "$T/r2.env"; run_relaunch "$T/rb2.sh" "$T/r2"; rc=$?
check "9 with an empty captured environment it still relaunches only with what was captured (no TERMLINK_SWEEP)" '[ "$rc" = 0 ] && ! has_env "$T/r2" TERMLINK_SWEEP_INTERVAL_SECS=3600'
check "10 a bad mode is refused" '! write_swap_relaunch "$T/x.sh" bogus s 1 e a 2>/dev/null'

# --- Mutant: relaunch ignoring the captured environment -------------------------
sed 's/env "\${HUB_ENV_ARGS\[@\]}" setsid/setsid/' "$T/rb.sh" > "$T/m1.sh"
check "M1 precondition: the env carry was removed" '! cmp -s "$T/rb.sh" "$T/m1.sh"'
run_relaunch "$T/m1.sh" "$T/m1"
check "M1 mutant (no env carry) loses TERMLINK_SWEEP_INTERVAL_SECS -> case 5 would be red" '[ -s "$T/m1.env" ] && ! has_env "$T/m1" TERMLINK_SWEEP_INTERVAL_SECS=3600'
# --- Mutant: unquoted argv -------------------------------------------------------
sed 's/"\${RUN_ARGS\[@\]}"/${RUN_ARGS[@]}/' "$T/rb.sh" > "$T/m2.sh"
run_relaunch "$T/m2.sh" "$T/m2"
check "M2 mutant (unquoted args) splits 'my hub' -> case 8 would be red" '[ -s "$T/m2.argv" ] && [ "$(tr "\0" "|" < "$T/m2.argv")" != "nohup|$T/termlink|hub|start|--tcp|0.0.0.0:9100|--label|my hub|" ]'

echo "---"
echo "hub-binary-swap fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = 0 ]
