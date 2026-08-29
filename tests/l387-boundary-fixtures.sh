#!/usr/bin/env bash
# L-387 boundary fixtures (T-2852).
#
# Pins the MEASUREMENT that the repo's verification guidance now rests on:
# `echo "$out" | grep -q PAT` under `set -o pipefail` is safe below the pipe
# capacity and returns 141 above it, while the herestring and file-redirect
# forms are correct at any size.
#
# This exists so the claim in CLAUDE.md and .tasks/templates/default.md can be
# FALSIFIED by anyone on any host, rather than being taken on the authority of
# whoever measured it last. If a future kernel, shell or pipe size changes the
# answer, this suite says so instead of the docs quietly going stale.
#
# NOT a guard-layer member: it asserts a property of the SHELL, not of this
# tree, so it has no finding to report and nothing here to fix.

set -uo pipefail

pass=0; fail=0
ok()   { printf '  ok   %s\n' "$1"; pass=$((pass+1)); }
bad()  { printf '  FAIL %s\n' "$1"; fail=$((fail+1)); }
assert_rc() { # name expected actual
    if [ "$2" = "$3" ]; then ok "$1 (rc=$3)"; else bad "$1 — expected rc=$2, got rc=$3"; fi
}

# Build a capture of N bytes whose match is on the FIRST line, so `grep -q`
# exits immediately and the upstream is still writing. Match POSITION is part
# of the mechanism: with the pattern at the END grep consumes everything, never
# exits early, and the pipeline returns 0 at any size.
capture() { printf 'MATCHME\n'; head -c "$1" /dev/zero | tr '\0' 'x'; }

echo "=== A. the capture-then-echo form is size-dependent ==="
#
# 65536 itself -- the exact pipe capacity -- is deliberately NOT asserted. It sits
# in a race between echo's write completing and grep's exit, and measured both 0
# and 141 on the same host. Pinning a flaky value would make this suite the thing
# people learn to ignore. The band being nondeterministic is itself the argument
# against the form: it fails rarely and unpredictably rather than always.

for n in 4096 16384 32768; do
    out=$(capture "$n")
    ( set -o pipefail; echo "$out" | grep -q "MATCHME" ); rc=$?
    assert_rc "below pipe capacity (${n}B) the echo form succeeds" 0 "$rc"
done

for n in 131072 1048576; do
    out=$(capture "$n")
    ( set -o pipefail; echo "$out" | grep -q "MATCHME" ); rc=$?
    assert_rc "above pipe capacity (${n}B) the echo form returns SIGPIPE 141" 141 "$rc"
done

echo
echo "=== B. match POSITION is why this survived so long ==="

out=$(head -c 1048576 /dev/zero | tr '\0' 'x'; printf '\nMATCHME\n')
( set -o pipefail; echo "$out" | grep -q "MATCHME" ); rc=$?
assert_rc "a LATE match is safe at any size (grep never exits early)" 0 "$rc"

echo
echo "=== C. the prescribed alternatives are correct at any size ==="

out=$(capture 1048576)
( set -o pipefail; grep -q "MATCHME" <<< "$out" ); rc=$?
assert_rc "herestring: no pipe, so no SIGPIPE" 0 "$rc"

tmp="$(mktemp)"; trap 'rm -f "$tmp"' EXIT
capture 1048576 > "$tmp" 2>&1
( set -o pipefail; grep -q "MATCHME" "$tmp" ); rc=$?
assert_rc "file redirect: no pipe either" 0 "$rc"

echo
echo "=== D. the redirect form ALSO preserves the producer's exit code ==="

# The capture form discards it: a failing command yields an empty capture that
# grep merely fails to match, which is a different verdict from "it failed".
( set -o pipefail; false > "$tmp" 2>&1 && grep -q "MATCHME" "$tmp" ); rc=$?
assert_rc "redirect + && surfaces a failing producer" 1 "$rc"

echo
echo "l387-boundary-fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
