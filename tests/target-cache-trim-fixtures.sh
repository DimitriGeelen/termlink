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

# ---- T-3820 (ruling A): cargo build dirs under .context/working/ ------------------------------------------------
W="$T/repo/.context/working"
cargo_dir() {  # <path> <age: old|new> [marker: tag|debug|none]
    mkdir -p "$1"
    case "${3:-tag}" in tag) echo "Signature: 8a477f597d28d172789f06886806bc55" > "$1/CACHEDIR.TAG" ;; debug) mkdir -p "$1/debug" ;; none) : ;; esac
    echo 4 > "$1/.fake-gib"; mkdir -p "$1/x"; echo data > "$1/x/f"
    [ "$2" = old ] && find "$1" -exec touch -h -d '10 days ago' {} +
    return 0
}
setup 12; cargo_dir "$W/t3700/target" old; cargo_dir "$W/t3701/target-w1" old debug; out=$(run --quiet); rc=$?
check "7 old scratch cargo dirs removed even with target/debug below the threshold; OK line; rc 0" \
    '[ $rc = 0 ] && [ ! -e "$W/t3700/target" ] && [ ! -e "$W/t3701/target-w1" ] && grep -q "OK      removed 2 scratch build dir(s) under .context/working, about 8 GiB" <<< "$out" && [ -d "$T/repo/target/debug" ]'
setup 12; cargo_dir "$W/t3702/target" new; out=$(run --quiet); rc=$?
check "8 a scratch cargo dir used in the last 3 days is kept" '[ $rc = 0 ] && [ -e "$W/t3702/target/CACHEDIR.TAG" ]'
setup 12; cargo_dir "$W/notes/target" old none; out=$(run --quiet); rc=$?
check "9 an old dir named target without cargo markers is kept" '[ $rc = 0 ] && [ -e "$W/notes/target/x/f" ]'
setup 12; cargo_dir "$W/t3703/target" old; sleep 39.3820 & SP=$!; out=$(BUSY='^sleep 39\.3820' run); rc=$?; kill "$SP" 2>/dev/null; wait "$SP" 2>/dev/null; SP=""
check "10 a running build: scratch dirs untouched, skip line" '[ $rc = 0 ] && [ -e "$W/t3703/target/CACHEDIR.TAG" ] && grep -q "skip    scratch build dirs under .context/working: a build is running" <<< "$out"'
setup 12; cargo_dir "$W/t3704/target" old; out=$(run --dry-run); rc=$?
check "11 dry-run lists the scratch dir and removes nothing" '[ $rc = 0 ] && [ -e "$W/t3704/target/CACHEDIR.TAG" ] && grep -q "\[DRY\]   would remove scratch build dir .context/working/t3704/target (4 GiB" <<< "$out"'
setup 12; rm -rf "$T/outside"; cargo_dir "$T/outside/target" old; mkdir -p "$W"; ln -s "$T/outside" "$W/link"; ln -s "$T/outside/target" "$W/target-ln"; out=$(run --quiet); rc=$?
check "12 never through a symlink: a cargo dir outside .context/working is untouched" '[ $rc = 0 ] && [ -e "$T/outside/target/CACHEDIR.TAG" ] && [ -L "$W/target-ln" ]'
rm -rf "$T/outside"

echo "---"
echo "target-cache-trim fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = 0 ]
