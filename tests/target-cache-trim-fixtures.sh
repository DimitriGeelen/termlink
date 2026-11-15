#!/usr/bin/env bash
# Hermetic fixtures for scripts/target-cache-trim.sh (T-3799). Scratch dirs only; a fake
# cargo records its argv; sizes come from a .fake-gib file per dir; "busy" is a sleep with
# a unique argument. Never touches the real target/.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SUT="$ROOT/scripts/target-cache-trim.sh"
T="$(mktemp -d "${TMPDIR:-/var/tmp}/t3799-fix.XXXXXX")"; trap 'kill "$SP" 2>/dev/null; rm -rf "$T"' EXIT
PASS=0 FAIL=0 SP=""
ok() { PASS=$((PASS + 1)); echo "ok   $*"; }
bad() { FAIL=$((FAIL + 1)); echo "FAIL $*"; }
check() { if eval "$2"; then ok "$1"; else bad "$1"; fi; }

cat > "$T/cargo" <<'EOF'
#!/usr/bin/env bash
echo "$*" >> "$FAKE_CARGO_LOG"
[ "${FAKE_CARGO_NOOP:-0}" = 1 ] && exit 0
case "$*" in
  "clean --profile dev") rm -rf "$PWD/target/debug" ;;
  "clean --target-dir "*) rm -rf "${*#clean --target-dir }" ;;
esac
EOF
chmod +x "$T/cargo"
printf '#!/usr/bin/env bash\ncat "$1/.fake-gib" 2>/dev/null || echo 0\n' > "$T/size"; chmod +x "$T/size"

setup() {  # <debug GiB> [mutants GiB]
    rm -rf "$T/repo"; mkdir -p "$T/repo/target/debug" "$T/repo/target/release"
    echo "$1" > "$T/repo/target/debug/.fake-gib"; echo 7 > "$T/repo/target/release/.fake-gib"
    [ -n "${2:-}" ] && { mkdir -p "$T/repo/target/arc011-mutants"; echo "$2" > "$T/repo/target/arc011-mutants/.fake-gib"; }
    echo 9 > "$T/repo/target/.fake-gib"; : > "$T/cargo.log"
}
run() { TRIM_ROOT="$T/repo" TRIM_CARGO="${CARGO_BIN:-$T/cargo}" TRIM_SIZE_CMD="$T/size" FAKE_CARGO_LOG="$T/cargo.log" \
        TRIM_BUSY_PATTERN="${BUSY:-^no-such-process-t3799\$}" bash "$SUT" "$@" 2>&1; }

setup 12; out=$(run --quiet); rc=$?
check "1 below the threshold: no output, no clean, debug kept" '[ $rc = 0 ] && [ -z "$out" ] && [ ! -s "$T/cargo.log" ] && [ -d "$T/repo/target/debug" ]'

setup 45 3; out=$(run --dry-run); rc=$?
check "2 dry-run: says what it would do, cargo not called" '[ $rc = 0 ] && grep -q "\[DRY\]   would run cargo clean --profile dev (target/debug 45 GiB)" <<< "$out" && [ ! -s "$T/cargo.log" ]'

setup 45 3; sleep 39.3799 & SP=$!; out=$(BUSY='^sleep 39\.3799' run); rc=$?; kill "$SP" 2>/dev/null; wait "$SP" 2>/dev/null; SP=""
check "3 a build running: skip line, nothing cleaned, debug kept" '[ $rc = 0 ] && grep -q "skip    target/debug 45 GiB, a build is running" <<< "$out" && [ ! -s "$T/cargo.log" ] && [ -d "$T/repo/target/debug" ]'

setup 45 3; out=$(run --quiet); rc=$?
check "4 trim: cargo clean --profile dev, mutants cleaned, release kept, OK line" '[ $rc = 0 ] && grep -qx "clean --profile dev" "$T/cargo.log" && grep -q "clean --target-dir $T/repo/target/arc011-mutants" "$T/cargo.log" && [ ! -d "$T/repo/target/debug" ] && [ -d "$T/repo/target/release" ] && grep -q "OK      trimmed target/debug 45 GiB -> 0 GiB" <<< "$out"'

setup 45; out=$(FAKE_CARGO_NOOP=1 run); rc=$?
check "5 clean ran but freed nothing: FAILED, exit 1" '[ $rc = 1 ] && grep -q "FAILED  cargo clean --profile dev ran, but target/debug is still 45 GiB" <<< "$out"'

setup 45; out=$(CARGO_BIN="$T/no-cargo" run); rc=$?
check "6 cargo missing: tooling, exit 2, nothing removed" '[ $rc = 2 ] && [ -d "$T/repo/target/debug" ]'

echo "---"
echo "target-cache-trim fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = 0 ]
