#!/usr/bin/env bash
# test-render-formula.sh — hermetic tests for homebrew/render-formula.sh (T-3754).
#
# No network, no real release, no tap: fixtures live in a temp dir. Run by the
# release workflow's homebrew-tap job before it renders anything, and by hand:
#   bash homebrew/test-render-formula.sh
# Exit 0 all pass, 1 any fail, 2 tooling (cannot set up).
set -uo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
render="$here/render-formula.sh"
formula="$here/Formula/termlink.rb"
tmp="$(mktemp -d "${TMPDIR:-/tmp}/t3754-render.XXXXXX")" || exit 2
trap 'rm -rf "$tmp"' EXIT

pass=0; fail=0
ok()  { pass=$((pass + 1)); echo "  ok   $1"; }
bad() { fail=$((fail + 1)); echo "  FAIL $1"; }

artifacts="termlink-darwin-aarch64 termlink-darwin-x86_64 termlink-linux-aarch64 termlink-linux-x86_64 termlink-linux-x86_64-static"

# One distinct hash per artifact, so a hash landing under the wrong url is caught.
mkhash() { printf '%064d' "$1" | tr '0' "$2"; }
i=0
: > "$tmp/sums.txt"
for a in $artifacts; do
    i=$((i + 1))
    printf '%s  %s\n' "$(mkhash 0 "$(printf '%x' $((i + 9)))")" "$a" >> "$tmp/sums.txt"
done

# check_pairs <rendered> <sums>: every url's artifact is followed by its own hash.
check_pairs() {
    awk 'FNR == NR { s[$2] = $1; next }
         /url "/ { u = $0; sub(/^[ \t]*url "/, "", u); sub(/".*$/, "", u); n = split(u, p, "/"); a = p[n]; next }
         /sha256 "/ { h = $0; sub(/^[ \t]*sha256 "/, "", h); sub(/".*$/, "", h)
                      if (s[a] == "" || h != s[a]) bad = 1; seen++ }
         END { exit (bad || seen != 4) }' "$2" "$1"
}

# expect_refusal <name> <expected rc> <stderr pattern> -- <args...>
expect_refusal() {
    local name="$1" want="$2" pat="$3"; shift 4
    local rc=0
    bash "$render" "$@" > "$tmp/out" 2> "$tmp/err" || rc=$?
    if [ "$rc" -eq "$want" ] && [ ! -s "$tmp/out" ] && grep -q -- "$pat" "$tmp/err"; then
        ok "$name (rc $rc, no formula printed)"
    else
        bad "$name (rc $rc want $want; stdout $(wc -c < "$tmp/out") bytes; stderr: $(head -1 "$tmp/err"))"
    fi
}

echo "render-formula.sh"

# 1. Positive: version and each hash under its own url.
if bash "$render" 9.8.7 "$tmp/sums.txt" > "$tmp/r1" 2> "$tmp/e1" \
   && grep -q '^  version "9.8.7"$' "$tmp/r1" && check_pairs "$tmp/r1" "$tmp/sums.txt"; then
    ok "renders version and four per-artifact hashes"
else
    bad "positive render ($(head -1 "$tmp/e1"))"
fi

# 2. Tag form vX.Y.Z is accepted and stripped.
if bash "$render" v9.8.7 "$tmp/sums.txt" > "$tmp/r2" 2>/dev/null && grep -q '^  version "9.8.7"$' "$tmp/r2"; then
    ok "accepts the tag form v9.8.7"
else
    bad "tag form v9.8.7"
fi

# 3. sha256sum binary mode ("*name") parses the same.
sed 's/  /  */' "$tmp/sums.txt" > "$tmp/sums-bin.txt"
if bash "$render" 9.8.7 "$tmp/sums-bin.txt" > "$tmp/r3" 2>/dev/null && cmp -s "$tmp/r1" "$tmp/r3"; then
    ok "binary-mode checksum names (*name) give the same formula"
else
    bad "binary-mode checksum names"
fi

# 4. Round trip: the in-repo formula with its own values renders byte-identical.
own_version="$(sed -n 's/^  version "\(.*\)"$/\1/p' "$formula")"
awk '/url "/ { u = $0; sub(/^[ \t]*url "/, "", u); sub(/".*$/, "", u); n = split(u, p, "/"); a = p[n] }
     /sha256 "/ { h = $0; sub(/^[ \t]*sha256 "/, "", h); sub(/".*$/, "", h); print h "  " a }' "$formula" > "$tmp/own.txt"
if [ -n "$own_version" ] && bash "$render" "$own_version" "$tmp/own.txt" > "$tmp/r4" 2>/dev/null && cmp -s "$formula" "$tmp/r4"; then
    ok "in-repo formula reproduces itself byte-identical ($own_version)"
else
    bad "round trip of the in-repo formula (version '$own_version'; diff: $(diff "$formula" "$tmp/r4" 2>&1 | head -2 | tr '\n' ' '))"
fi

# Negative controls: each must refuse with no formula on stdout.
grep -v 'termlink-darwin-x86_64$' "$tmp/sums.txt" > "$tmp/missing.txt"
expect_refusal "missing artifact refused" 1 "termlink-darwin-x86_64 is not in the checksums" -- 9.8.7 "$tmp/missing.txt"

awk 'NR == 1 { $1 = substr($1, 2) } { print $1 "  " $2 }' "$tmp/sums.txt" > "$tmp/short.txt"
expect_refusal "63-char hash refused" 1 "not 64 lowercase hex" -- 9.8.7 "$tmp/short.txt"

awk 'NR == 2 { $1 = toupper($1) } { print $1 "  " $2 }' "$tmp/sums.txt" > "$tmp/upper.txt"
expect_refusal "uppercase hash refused" 1 "not 64 lowercase hex" -- 9.8.7 "$tmp/upper.txt"

for v in 0.12 latest 0.12.11-rc1 ""; do
    expect_refusal "version '$v' refused" 1 "is not X.Y.Z" -- "$v" "$tmp/sums.txt"
done

: > "$tmp/empty.txt"
expect_refusal "empty checksums refused" 1 "is empty" -- 9.8.7 "$tmp/empty.txt"
expect_refusal "unreadable checksums is tooling (2)" 2 "cannot read" -- 9.8.7 "$tmp/does-not-exist.txt"
expect_refusal "usage error is tooling (2)" 2 "usage" -- 9.8.7

printf 'class X\n  version "1.0.0"\n  sha256 "%064d"\nend\n' 0 > "$tmp/orphan.rb"
expect_refusal "sha256 with no url above refused" 1 "has no url line above" -- 9.8.7 "$tmp/sums.txt" "$tmp/orphan.rb"

printf 'class X\n  version "1.0.0"\nend\n' > "$tmp/nourl.rb"
expect_refusal "template without url/sha256 pairs refused" 1 "expected equal and > 0" -- 9.8.7 "$tmp/sums.txt" "$tmp/nourl.rb"

# Mutant: a renderer that puts one hash under every url must fail check_pairs,
# proving the positive assertion (case 1) can fail.
sed 's/h = sums\[artifact\]/h = sums["termlink-darwin-aarch64"]/' "$render" > "$tmp/mutant.sh"
if grep -q 'sums\["termlink-darwin-aarch64"\]' "$tmp/mutant.sh" \
   && bash "$tmp/mutant.sh" 9.8.7 "$tmp/sums.txt" "$formula" > "$tmp/rm" 2>/dev/null \
   && ! check_pairs "$tmp/rm" "$tmp/sums.txt"; then
    ok "mutant (same hash under every url) is caught by the pair check"
else
    bad "mutant survived or could not be built"
fi

echo "render-formula tests: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
