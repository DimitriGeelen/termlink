#!/usr/bin/env bash
# Fixtures for scripts/pool-run.sh (T-3859). Fake pinned files and a fake submit script; never contacts a hub.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SUT="${POOL_RUN_SUT:-$ROOT/scripts/pool-run.sh}"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
PASS=0; FAIL=0
ok() { PASS=$((PASS + 1)); echo "ok   $*"; }
bad() { FAIL=$((FAIL + 1)); echo "FAIL $*"; }
check() { if eval "$2"; then ok "$1"; else bad "$1"; fi; }

# A throwaway repo with one commit.
R="$T/repo"; mkdir -p "$R"; git -C "$R" init -q; echo x > "$R/f"; git -C "$R" add f
git -C "$R" -c user.email=t@t -c user.name=t commit -qm init

# Fake ring20 files. The fake submit records its argv and POOL_CLIENT, then exits with $FAKE_RC.
L="$T/lib"; mkdir -p "$L/handoff"
cat > "$L/handoff/010-pool-submit.py" <<'EOF'
import os, sys
client = os.environ.get("POOL_CLIENT", "")
if os.environ.get("FAKE_LIVE_CLIENT"):          # replace the LIVE client mid-run, as happened on 2026-11-24
    open(os.environ["FAKE_LIVE_CLIENT"], "a").write("# changed mid-run\n")
used = open(client).read() if client and os.path.exists(client) else "MISSING"
open(os.environ["FAKE_MARK"], "w").write(" ".join(sys.argv[1:]) + "\nCLIENT=" + client + "\nUSED<<" + used + ">>\n")
sys.exit(int(os.environ.get("FAKE_RC", "0")))
EOF
echo "# client" > "$L/pool-client.py"; echo "# poold" > "$L/poold.py"
pin() { for f in "$L/handoff/010-pool-submit.py" "$L/pool-client.py" "$L/poold.py"; do echo "$(sha256sum "$f" | cut -c1-64)  $f"; done > "$T/pins"; }
pin
run() { rm -f "$T/mark"; FAKE_MARK="$T/mark" POOL_RUN_PINS="$T/pins" POOL_RUN_REPO="$R" POOL_RUN_OUT_ROOT="$T/out" \
        POOL_RUN_HEARTBEAT="$T/hb" bash "$SUT" "$@" 2>&1; }

out=$(FAKE_RC=0 run test); rc=$?
check "1 pins match: the submit script runs, rc 0 passes through, ADVISORY written" '[ $rc = 0 ] && [ -f "$T/mark" ] && ls "$T"/out/*-test-*/ADVISORY.txt >/dev/null 2>&1 && grep -q "ADVISORY result rc=0" <<< "$out"'
check "2 the five excludes are passed, and POOL_CLIENT is a private copy, not the live path" 'grep -q -- "--exclude .tasks" "$T/mark" && grep -q -- "--exclude .agentic-framework" "$T/mark" && grep -q -- "--exclude .claude/settings.local.json" "$T/mark" && grep -q "^CLIENT=.*/client/pool-client.py$" "$T/mark" && ! grep -q "CLIENT=$L/" "$T/mark"'
check "3 a completed run touches the heartbeat" '[ -f "$T/hb" ]'
out=$(FAKE_RC=1 run clippy); rc=$?
check "4 a failed job (rc 1) passes through" '[ $rc = 1 ]'
echo "# poold changed" >> "$L/poold.py"
out=$(FAKE_RC=0 run test); rc=$?
check "5 a changed pinned file (poold.py) is PIN MISMATCH, rc 2, and nothing runs" '[ $rc = 2 ] && grep -q "PIN MISMATCH: $L/poold.py" <<< "$out" && [ ! -f "$T/mark" ]'
pin; rm "$L/pool-client.py"
out=$(run test); rc=$?
check "6 a missing pinned file is rc 2 and nothing runs" '[ $rc = 2 ] && grep -q "missing or unreadable" <<< "$out" && [ ! -f "$T/mark" ]'
echo "# client" > "$L/pool-client.py"; pin; grep -v poold "$T/pins" > "$T/pins2"; mv "$T/pins2" "$T/pins"
out=$(run test); rc=$?
check "7 a pin file without all three files is rc 2" '[ $rc = 2 ] && grep -q "must pin" <<< "$out" && [ ! -f "$T/mark" ]'
pin
out=$(FAKE_RC=7 run test); rc=$?
check "8 any other submit exit is reported as could-not-run (rc 2), never passed through" '[ $rc = 2 ] && grep -q "exited 7" <<< "$out"'
out=$(run deploy); rc=$?
check "9 an unknown kind is rc 2" '[ $rc = 2 ]'
out=$(run test deadbeefdeadbeef); rc=$?
check "10 an unknown commit is rc 2" '[ $rc = 2 ] && grep -q "unknown commit" <<< "$out"'

pin
out=$(FAKE_RC=0 FAKE_LIVE_CLIENT="$L/pool-client.py" run test); rc=$?
check "11 a client replaced mid-run is not the one used: the run keeps the reviewed private copy" '[ $rc = 0 ] && grep -q "USED<<# client" "$T/mark" && ! grep -q "changed mid-run" "$T/mark" && grep -q "changed mid-run" "$L/pool-client.py"'

echo "---"
echo "pool-run fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = 0 ]
