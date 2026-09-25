#!/usr/bin/env bash
# artifact-cli-fixtures.sh (T-3134, arc-011 S2)
#
# Hermetic load-bearing proof for `termlink artifact put`/`termlink artifact get` —
# no live hub required. Exercises: --help surfaces both new verbs; `get` refuses a
# missing --expected-sha256 (clap-required, IW-3); `get` fails fast on a
# sha256/--expected-sha256 mismatch BEFORE any hub contact (so it works with no hub
# running); `put` on a nonexistent file fails cleanly; --json mode emits a parseable
# {"ok":false,...} envelope on the fail-fast path.
#
# Exit 0 = all fixtures pass; exit 1 = a fixture assertion failed; exit 2 = tooling
# (binary not found — build it first).
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${TERMLINK_BIN:-}"
if [ -z "$BIN" ]; then
    if [ -x "$REPO_ROOT/target/debug/termlink" ]; then
        BIN="$REPO_ROOT/target/debug/termlink"
    elif [ -x "$REPO_ROOT/target/release/termlink" ]; then
        BIN="$REPO_ROOT/target/release/termlink"
    fi
fi
if [ -z "$BIN" ] || [ ! -x "$BIN" ]; then
    echo "FAIL: no termlink binary found — run \`cargo build -p termlink\` first (or set TERMLINK_BIN)" >&2
    exit 2
fi

# Isolate from any real local hub / identity state on this host — the fixtures
# below must never depend on (or disturb) a live hub.
SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
export TERMLINK_RUNTIME_DIR="$SCRATCH/runtime"
export TERMLINK_IDENTITY_DIR="$SCRATCH/identity"
mkdir -p "$TERMLINK_RUNTIME_DIR" "$TERMLINK_IDENTITY_DIR"

pass=0; fail=0
assert_rc() { # <desc> <expected-rc> <actual-rc>
    if [ "$2" -eq "$3" ]; then pass=$((pass+1)); echo "  ok: $1 (rc=$3)";
    else fail=$((fail+1)); echo "  FAIL: $1 — expected rc=$2 got rc=$3" >&2; fi
}
assert_contains() { # <desc> <haystack> <needle>
    if printf '%s' "$2" | grep -q -- "$3"; then pass=$((pass+1)); echo "  ok: $1";
    else fail=$((fail+1)); echo "  FAIL: $1 — output did not contain '$3'" >&2; fi
}

echo "== fixture 1: 'artifact put --help' documents --to =="
out="$("$BIN" artifact put --help 2>&1)"
assert_contains "put --help mentions --to" "$out" "\-\-to"

echo "== fixture 2: 'artifact get --help' documents --expected-sha256 =="
out="$("$BIN" artifact get --help 2>&1)"
assert_contains "get --help mentions --expected-sha256" "$out" "expected-sha256"

echo "== fixture 3: 'artifact get' without --expected-sha256 is a clap parse error =="
out="$("$BIN" artifact get abc123 -o "$SCRATCH/out.bin" 2>&1)"
rc=$?
assert_rc "missing --expected-sha256 rejected" 2 "$rc"
assert_contains "clap names the missing arg" "$out" "expected-sha256"

echo "== fixture 4: 'artifact get' with mismatched --expected-sha256 fails fast, no hub needed =="
out="$("$BIN" artifact get "$(printf 'a%.0s' {1..64})" --expected-sha256 "$(printf 'b%.0s' {1..64})" -o "$SCRATCH/out.bin" 2>&1)"
rc=$?
assert_rc "mismatch is non-zero exit" 1 "$rc"
assert_contains "mismatch names the fail-fast reason" "$out" "SHA-256 argument mismatch"
[ -e "$SCRATCH/out.bin" ] && { fail=$((fail+1)); echo "  FAIL: mismatch path must not write an output file" >&2; } || { pass=$((pass+1)); echo "  ok: no output file written on mismatch"; }

echo "== fixture 5: matching sha256 args pass the fail-fast check and reach the (absent) hub =="
same="$(printf 'c%.0s' {1..64})"
out="$("$BIN" artifact get "$same" --expected-sha256 "$same" -o "$SCRATCH/out2.bin" 2>&1)"
rc=$?
assert_rc "no-hub error, not a mismatch error" 1 "$rc"
assert_contains "fails on hub reachability, not argument mismatch" "$out" "local-hub-only\|No local hub socket\|connect hub"

echo "== fixture 6: 'artifact put' on a nonexistent file fails cleanly (no panic, no hub needed) =="
out="$("$BIN" artifact put "$SCRATCH/does-not-exist.bin" --to some-peer 2>&1)"
rc=$?
assert_rc "nonexistent file is non-zero exit" 1 "$rc"
assert_contains "names the read failure" "$out" "Failed to read file"

echo "== fixture 7: --json mode on the mismatch fail-fast path emits a parseable {ok:false} envelope =="
out="$("$BIN" artifact get "$(printf 'd%.0s' {1..64})" --expected-sha256 "$(printf 'e%.0s' {1..64})" -o "$SCRATCH/out3.bin" --json 2>&1)"
assert_contains "json envelope reports ok:false" "$out" "\"ok\":false"
assert_contains "json envelope names sha256_requested" "$out" "sha256_requested"
if command -v python3 >/dev/null 2>&1; then
    if printf '%s\n' "$out" | tail -1 | python3 -c 'import json,sys; json.loads(sys.stdin.read())' 2>/dev/null; then
        pass=$((pass+1)); echo "  ok: last line is valid JSON"
    else
        fail=$((fail+1)); echo "  FAIL: last line of --json output is not valid JSON: $out" >&2
    fi
fi

echo
echo "artifact-cli fixtures: $pass passed, $fail failed"
if [ "$fail" -eq 0 ]; then
    echo "ALL PASS"
    exit 0
fi
exit 1
