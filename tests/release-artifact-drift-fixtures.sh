#!/usr/bin/env bash
# release-artifact-drift-fixtures.sh (T-2751)
#
# Hermetic load-bearing proof for scripts/check-release-artifact-drift.sh — no network,
# no live release, no GitHub, no repo state beyond the check script itself. Builds
# synthetic install.sh / release.yml / formula files exercising each branch of the
# comparison and asserts the verdict + exit code on each.
#
# Includes a PL-219 control: an assertion that the check does NOT fire on the real,
# agreeing tree. Without it the suite could pass with a detector wired to fire always.
#
# Exit 0 = all fixtures pass; exit 1 = a fixture assertion failed.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-release-artifact-drift.sh"
[ -f "$CHECK" ] || { echo "FAIL: check script not found at $CHECK" >&2; exit 1; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT

pass=0; fail=0
assert_rc() { # <desc> <expected-rc> <actual-rc>
    if [ "$2" -eq "$3" ]; then pass=$((pass+1)); echo "  ok: $1 (rc=$3)";
    else fail=$((fail+1)); echo "  FAIL: $1 — expected rc=$2 got rc=$3" >&2; fi
}
assert_contains() { # <desc> <haystack> <needle>
    if printf '%s' "$2" | grep -q -- "$3"; then pass=$((pass+1)); echo "  ok: $1";
    else fail=$((fail+1)); echo "  FAIL: $1 — output did not contain '$3'" >&2; fi
}
assert_absent() { # <desc> <haystack> <needle>
    if printf '%s' "$2" | grep -q -- "$3"; then fail=$((fail+1)); echo "  FAIL: $1 — output unexpectedly contained '$3'" >&2;
    else pass=$((pass+1)); echo "  ok: $1"; fi
}

I="$SCRATCH/install.sh"
R="$SCRATCH/release.yml"
F="$SCRATCH/formula.rb"

run_rc()  { bash "$CHECK" --no-heartbeat --install-sh "$I" --release-yml "$R" --formula "$F" "$@" >/dev/null 2>&1; echo $?; }
run_out() { bash "$CHECK" --no-heartbeat --install-sh "$I" --release-yml "$R" --formula "$F" "$@" 2>&1; }

# Helpers that write a synthetic file from a list of artifact names.
write_install() { : > "$I"; echo '#!/bin/sh' >> "$I"; for a in "$@"; do echo "    artifact=\"$a\" ;;" >> "$I"; done; }
write_release() { : > "$R"; echo 'jobs:' >> "$R"; echo '  files: |' >> "$R"; for a in "$@"; do echo "            dist/$a" >> "$R"; done; echo '            dist/checksums.txt' >> "$R"; }
write_formula() { : > "$F"; for a in "$@"; do echo "      url \"https://github.com/x/y/releases/download/v1/$a\"" >> "$F"; done; }

THREE_A="termlink-linux-x86_64"
THREE_B="termlink-darwin-aarch64"
THREE_C="termlink-linux-aarch64"

echo "== fixture 1: all three sets agree =="
write_install "$THREE_A" "$THREE_B" "$THREE_C"
write_release "$THREE_A" "$THREE_B" "$THREE_C"
write_formula "$THREE_A" "$THREE_B" "$THREE_C"
assert_rc "agreeing sets are clean" 0 "$(run_rc)"
out="$(run_out)"
assert_contains "clean output states the census" "$out" "3 published asset"
assert_contains "clean output states the scope limit" "$out" "does NOT verify"

echo "== fixture 2: install.sh offers a name release.yml does not publish =="
write_install "$THREE_A" "$THREE_B" "termlink-linux-riscv64"
write_release "$THREE_A" "$THREE_B"
write_formula "$THREE_A"
assert_rc "offered-but-unpublished fires" 1 "$(run_rc)"
out="$(run_out)"
assert_contains "names the offending artifact" "$out" "termlink-linux-riscv64"
assert_contains "explains the user-visible consequence" "$out" "404"

echo "== fixture 3: release.yml publishes a name install.sh never selects =="
write_install "$THREE_A"
write_release "$THREE_A" "termlink-darwin-x86_64"
write_formula "$THREE_A"
assert_rc "published-but-unreachable fires" 1 "$(run_rc)"
out="$(run_out)"
assert_contains "names the unreachable artifact" "$out" "termlink-darwin-x86_64"
assert_contains "explains it is built but undeliverable" "$out" "cannot deliver"

echo "== fixture 4: the two directions are distinct, not one merged message =="
write_install "$THREE_A" "termlink-only-in-install"
write_release "$THREE_A" "termlink-only-in-release"
write_formula "$THREE_A"
assert_rc "both directions at once fire" 1 "$(run_rc)"
out="$(run_out)"
assert_contains "reports the install-side name" "$out" "termlink-only-in-install"
assert_contains "reports the release-side name" "$out" "termlink-only-in-release"
assert_contains "reports two mismatches" "$out" "2 artifact-name mismatch"

echo "== fixture 5: formula omitting a published artifact does NOT fire (deliberate subset, T-1135) =="
write_install "$THREE_A" "$THREE_B"
write_release "$THREE_A" "$THREE_B"
write_formula "$THREE_A"
assert_rc "formula subset is clean" 0 "$(run_rc)"
out="$(run_out)"
assert_contains "counts the formula references" "$out" "1 formula reference"

echo "== fixture 6: formula pointing at an unpublished artifact DOES fire =="
write_install "$THREE_A"
write_release "$THREE_A"
write_formula "$THREE_A" "termlink-ghost-target"
assert_rc "formula superset fires" 1 "$(run_rc)"
out="$(run_out)"
assert_contains "names the ghost artifact" "$out" "termlink-ghost-target"
assert_contains "explains brew consequence" "$out" "brew install"

echo "== fixture 7: commented-out lines are not artifacts =="
write_install "$THREE_A"
write_release "$THREE_A"
write_formula "$THREE_A"
echo '#     artifact="termlink-commented-out"' >> "$I"
echo '#           dist/termlink-also-commented' >> "$R"
assert_rc "comments do not create phantom artifacts" 0 "$(run_rc)"
out="$(run_out)"
assert_absent "commented install name absent" "$out" "termlink-commented-out"

echo "== fixture 8: the formula's Dir[] runtime glob is not parsed as an artifact =="
write_install "$THREE_A"
write_release "$THREE_A"
write_formula "$THREE_A"
echo '    binary = Dir["termlink-*"].first || "termlink"' >> "$F"
assert_rc "Dir[] glob is not an artifact reference" 0 "$(run_rc)"

echo "== fixture 9: an empty extraction is a tooling error, never a clean census =="
: > "$I"; echo '#!/bin/sh' >> "$I"; echo 'echo hello' >> "$I"
write_release "$THREE_A"
write_formula "$THREE_A"
assert_rc "zero offered names is exit 2" 2 "$(run_rc)"
out="$(run_out)"
assert_contains "refuses to report clean" "$out" "refusing to report clean"

write_install "$THREE_A"
: > "$R"; echo 'jobs: {}' >> "$R"
assert_rc "zero published names is exit 2" 2 "$(run_rc)"

echo "== fixture 10: a missing input file is a tooling error =="
write_install "$THREE_A"
write_release "$THREE_A"
rc=$(bash "$CHECK" --no-heartbeat --install-sh "$SCRATCH/does-not-exist.sh" --release-yml "$R" >/dev/null 2>&1; echo $?)
assert_rc "missing install.sh is exit 2" 2 "$rc"

echo "== fixture 11: an absent formula is tolerated (optional input) =="
write_install "$THREE_A"
write_release "$THREE_A"
rc=$(bash "$CHECK" --no-heartbeat --install-sh "$I" --release-yml "$R" --formula "$SCRATCH/no-formula.rb" >/dev/null 2>&1; echo $?)
assert_rc "absent formula does not fire or error" 0 "$rc"

echo "== fixture 12: JSON envelope shape =="
write_install "$THREE_A" "termlink-orphan"
write_release "$THREE_A"
write_formula "$THREE_A"
out="$(run_out --json)"
assert_contains "json reports not-ok" "$out" '"ok":false'
assert_contains "json carries the firing name" "$out" '"name":"termlink-orphan"'
assert_contains "json carries the scope disclaimer" "$out" '"scope":'
if printf '%s' "$out" | python3 -c "import json,sys; d=json.load(sys.stdin); assert d['published_count']==1; assert d['offered_count']==2" 2>/dev/null; then
    pass=$((pass+1)); echo "  ok: json parses with correct counts"
else
    fail=$((fail+1)); echo "  FAIL: json did not parse or counts were wrong" >&2
fi

echo "== fixture 13: --quiet prints nothing when clean =="
write_install "$THREE_A"
write_release "$THREE_A"
write_formula "$THREE_A"
out="$(run_out --quiet)"
if [ -z "$out" ]; then pass=$((pass+1)); echo "  ok: quiet clean run is silent";
else fail=$((fail+1)); echo "  FAIL: quiet clean run printed: $out" >&2; fi

echo "== fixture 14: unknown argument is a tooling error =="
rc=$(run_rc --nonsense)
assert_rc "unknown arg is exit 2" 2 "$rc"

echo "== fixture 15: PL-219 control — the REAL tree must scan clean =="
# Without this, every assertion above could pass with a detector that fires
# unconditionally. This is the assertion that can fail if the check is wired wrong.
rc=$(cd "$REPO_ROOT" && bash "$CHECK" --no-heartbeat >/dev/null 2>&1; echo $?)
assert_rc "real repo tree is clean" 0 "$rc"
real_out="$(cd "$REPO_ROOT" && bash "$CHECK" --no-heartbeat 2>&1)"
assert_contains "real tree reports 5 published assets" "$real_out" "5 published asset"

echo
echo "release-artifact-drift fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ] || exit 1
exit 0
