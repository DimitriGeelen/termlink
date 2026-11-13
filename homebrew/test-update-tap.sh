#!/usr/bin/env bash
# test-update-tap.sh — hermetic tests for homebrew/update-tap.sh (T-3754).
#
# The "tap" is a local bare git repo in a temp dir; nothing touches GitHub.
# Run by the release workflow's homebrew-tap job before the real push, and by hand:
#   bash homebrew/test-update-tap.sh
# Exit 0 all pass, 1 any fail, 2 tooling.
set -uo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
upd="$here/update-tap.sh"
tmp="$(mktemp -d "${TMPDIR:-/tmp}/t3754-tap.XXXXXX")" || exit 2
trap 'rm -rf "$tmp"' EXIT
command -v git > /dev/null || exit 2

pass=0; fail=0
ok()  { pass=$((pass + 1)); echo "  ok   $1"; }
bad() { fail=$((fail + 1)); echo "  FAIL $1"; }

i=0
: > "$tmp/sums.txt"
for a in termlink-darwin-aarch64 termlink-darwin-x86_64 termlink-linux-aarch64 termlink-linux-x86_64 termlink-linux-x86_64-static; do
    i=$((i + 1))
    printf '%064d  %s\n' 0 "$a" | sed "s/^0\{64\}/$(printf '%064d' 0 | tr 0 "$(printf '%x' $((i + 9)))")/" >> "$tmp/sums.txt"
done

newbare() { git init --quiet --bare -b main "$tmp/$1"; }
commits() { git -C "$tmp/$1" rev-list --count main 2>/dev/null || echo 0; }
tapver()  { git -C "$tmp/$1" show main:Formula/termlink.rb 2>/dev/null | sed -n 's/^  version "\(.*\)"$/\1/p'; }

# run <name> <bare> [ENV=...]: runs update-tap against a local bare repo, fresh workdir.
run() {
    local name="$1" bare="$2"; shift 2
    rm -rf "$tmp/work-$name"
    env -u TAP_TOKEN TAG=v9.8.7 CHECKSUMS="$tmp/sums.txt" TAP_PUSH_URL="$tmp/$bare" TAP_READ_URL="$tmp/$bare" \
        WORKDIR="$tmp/work-$name" "$@" bash "$upd" > "$tmp/$name.out" 2>&1
}

echo "update-tap.sh"

# 1. Empty tap: formula is created, read back, and the proof line printed.
newbare tap1
if run first tap1 && [ "$(tapver tap1)" = "9.8.7" ] && [ "$(commits tap1)" = 1 ] \
   && grep -q '^tap formula version == v9.8.7$' "$tmp/first.out"; then
    ok "empty tap: formula pushed, read back, proof line printed"
else
    bad "empty tap ($(tail -2 "$tmp/first.out" | tr '\n' ' '))"
fi

# 2. Same tag again: idempotent, no new commit, still proven.
if run again tap1 && [ "$(commits tap1)" = 1 ] && grep -q 'nothing to push' "$tmp/again.out" \
   && grep -q '^tap formula version == v9.8.7$' "$tmp/again.out"; then
    ok "re-run of the same tag pushes nothing and still proves the tap"
else
    bad "idempotent re-run ($(tail -2 "$tmp/again.out" | tr '\n' ' '))"
fi

# 3. Newer tag: updates.
if run newer tap1 TAG=v9.9.0 && [ "$(tapver tap1)" = "9.9.0" ] && [ "$(commits tap1)" = 2 ]; then
    ok "newer tag updates the tap (9.8.7 -> 9.9.0)"
else
    bad "newer tag ($(tail -2 "$tmp/newer.out" | tr '\n' ' '))"
fi

# 4. Older tag re-run (e.g. a re-run of an old release job): never downgrades.
if run older tap1 TAG=v9.8.7 && [ "$(tapver tap1)" = "9.9.0" ] && [ "$(commits tap1)" = 2 ] \
   && grep -q 'not downgrading' "$tmp/older.out"; then
    ok "older tag does not downgrade a newer tap"
else
    bad "downgrade guard ($(tail -2 "$tmp/older.out" | tr '\n' ' '))"
fi

# 5. Pre-release tag: skipped, nothing pushed.
if run pre tap1 TAG=v9.10.0-rc1 && [ "$(commits tap1)" = 2 ] && grep -q 'only updated for plain releases' "$tmp/pre.out"; then
    ok "pre-release tag skipped with a notice"
else
    bad "pre-release skip"
fi

# Negative controls: each exits non-zero with an ::error:: naming the cause.
newbare tap2
rc=0; env -u TAP_TOKEN -u TAP_PUSH_URL TAG=v9.8.7 CHECKSUMS="$tmp/sums.txt" WORKDIR="$tmp/work-nosecret" \
    bash "$upd" > "$tmp/nosecret.out" 2>&1 || rc=$?
if [ "$rc" = 1 ] && grep -q '::error::.*HOMEBREW_TAP_TOKEN is not set' "$tmp/nosecret.out"; then
    ok "no secret: rc 1, error names HOMEBREW_TAP_TOKEN"
else
    bad "no secret (rc $rc)"
fi

rc=0; run norepo does-not-exist || rc=$?
if [ "$rc" = 1 ] && grep -q '::error::.*cannot clone the tap' "$tmp/norepo.out"; then
    ok "missing tap repo: rc 1, error says cannot clone"
else
    bad "missing tap repo (rc $rc)"
fi

rc=0; run noread tap2 TAP_READ_URL="$tmp/not-public" || rc=$?
if [ "$rc" = 1 ] && grep -q '::error::.*needs the tap repo to be PUBLIC' "$tmp/noread.out"; then
    ok "tap not readable without credentials: rc 1, error says it must be public"
else
    bad "unreadable tap (rc $rc)"
fi

# Read-back mismatch: the push lands, but the URL brew reads serves an older formula.
# This is the control that proves the version check can fail.
newbare stale
run seed stale TAG=v9.0.0 > /dev/null 2>&1
newbare tap3
rc=0; run mismatch tap3 TAP_READ_URL="$tmp/stale" || rc=$?
if [ "$rc" = 1 ] && grep -q "::error::.*tap formula version is '9.0.0', expected 9.8.7" "$tmp/mismatch.out" \
   && ! grep -q '^tap formula version ==' "$tmp/mismatch.out"; then
    ok "read-back serving an older version: rc 1, no proof line"
else
    bad "read-back mismatch (rc $rc; $(tail -1 "$tmp/mismatch.out"))"
fi

grep -v 'linux-aarch64$' "$tmp/sums.txt" > "$tmp/short.txt"
newbare tap4
rc=0; run badsums tap4 CHECKSUMS="$tmp/short.txt" || rc=$?
if [ "$rc" = 2 ] && [ "$(commits tap4)" = 0 ]; then
    ok "incomplete checksums: rc 2, nothing pushed"
else
    bad "incomplete checksums (rc $rc, commits $(commits tap4))"
fi

echo "update-tap tests: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
