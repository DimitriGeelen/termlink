#!/usr/bin/env bash
# arc-011 PH-1 kill-checking (architecture note 9.1 and 9.3; X2, R-69).
#
#   bash scripts/arc011-mutants.sh [--only <name>] [--report <file.md>]
#
# Each patch in crates/termlink-sidecar/tests/mutants/*.patch breaks one fail-closed
# rule (FC-1..FC-17) and names, in its header, the test that must catch it:
#
#   # fc: FC-4
#   # mutation: <what the patch breaks>
#   # test: --test <binary> <exact test name>
#
# The runner copies the workspace sources (no .git, no target) into a scratch
# directory, checks the unpatched suite is green there, then for each patch:
# applies it, builds, runs the named test, and reverts it. A test that still passes
# under its mutant means the rule is not actually checked. The working tree is
# never modified.
#
# Exit: 0 every mutant turned its test red; 1 at least one survived; 2 tooling
# (no termlink binary for the throwaway hub, baseline red, a patch that does not
# apply or does not build).
#
# Env: ARC011_MUTANTS_TARGET (cargo target dir, default target/arc011-mutants, kept
# between runs so rebuilds are incremental), TERMLINK_BIN (hub binary).
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MUT_DIR="crates/termlink-sidecar/tests/mutants"
ONLY=""
REPORT=""
while [ $# -gt 0 ]; do
  case "$1" in
    --only) ONLY="${2:-}"; shift 2 ;;
    --report) REPORT="${2:-}"; shift 2 ;;
    -h|--help) sed -n '2,24p' "$0"; exit 0 ;;
    *) echo "usage: $0 [--only <name>] [--report <file.md>]" >&2; exit 2 ;;
  esac
done

cd "$ROOT" || exit 2
command -v cargo >/dev/null 2>&1 || { echo "arc011-mutants: cargo not found" >&2; exit 2; }
command -v patch >/dev/null 2>&1 || { echo "arc011-mutants: patch not found" >&2; exit 2; }

if [ -z "${TERMLINK_BIN:-}" ]; then
  for b in target/debug/termlink target/release/termlink; do
    if [ -x "$b" ] && { [ -z "${TERMLINK_BIN:-}" ] || [ "$b" -nt "$TERMLINK_BIN" ]; }; then
      TERMLINK_BIN="$ROOT/$b"
    fi
  done
fi
[ -n "${TERMLINK_BIN:-}" ] && [ -x "$TERMLINK_BIN" ] || { echo "arc011-mutants: no termlink binary for the throwaway hub (build it or set TERMLINK_BIN)" >&2; exit 2; }
export TERMLINK_BIN
export CARGO_TARGET_DIR="${ARC011_MUTANTS_TARGET:-$ROOT/target/arc011-mutants}"

shopt -s nullglob
patches=("$MUT_DIR"/*.patch)
[ "${#patches[@]}" -gt 0 ] || { echo "arc011-mutants: no patches in $MUT_DIR" >&2; exit 2; }

SCRATCH="$(mktemp -d /tmp/arc011-mutants.XXXXXX)" || exit 2
trap 'rm -rf "$SCRATCH"' EXIT
mkdir -p "$SCRATCH/tests"
cp -a Cargo.toml Cargo.lock "$SCRATCH/" || exit 2
[ -d .cargo ] && cp -a .cargo "$SCRATCH/"
tar -C "$ROOT" --exclude='*/target' -cf - crates | tar -C "$SCRATCH" -xf - || exit 2
cp -a tests/fixtures "$SCRATCH/tests/" || exit 2

echo "arc011-mutants: scratch copy $SCRATCH, target $CARGO_TARGET_DIR, hub $TERMLINK_BIN"
echo "arc011-mutants: baseline (unpatched suite must be green)..."
if ! (cd "$SCRATCH" && timeout 2400 cargo test -q -p termlink-sidecar >"$SCRATCH/baseline.log" 2>&1); then
  echo "arc011-mutants: BASELINE RED: the unpatched suite fails; no mutant result would mean anything" >&2
  tail -30 "$SCRATCH/baseline.log" >&2
  exit 2
fi

rows=()
killed=0; survived=0; errors=0
for p in "${patches[@]}"; do
  name="$(basename "$p" .patch)"
  [ -n "$ONLY" ] && [ "$name" != "$ONLY" ] && continue
  fc="$(sed -n 's/^# fc: //p' "$p" | head -1)"
  what="$(sed -n 's/^# mutation: //p' "$p" | head -1)"
  test_args="$(sed -n 's/^# test: //p' "$p" | head -1)"
  if [ -z "$test_args" ]; then
    result="ERROR (no test named)"; errors=$((errors+1))
  elif ! (cd "$SCRATCH" && patch -p1 -s <"$ROOT/$p"); then
    result="ERROR (patch does not apply)"; errors=$((errors+1))
  else
    # shellcheck disable=SC2086
    if ! (cd "$SCRATCH" && timeout 1800 cargo test -q -p termlink-sidecar --no-run $test_args >"$SCRATCH/$name.build.log" 2>&1); then
      result="ERROR (does not build)"; errors=$((errors+1))
    elif (cd "$SCRATCH" && timeout 1200 cargo test -q -p termlink-sidecar $test_args -- --exact >"$SCRATCH/$name.log" 2>&1); then
      result="SURVIVED"; survived=$((survived+1))
    else
      result="killed"; killed=$((killed+1))
    fi
    (cd "$SCRATCH" && patch -p1 -R -s <"$ROOT/$p") || { echo "arc011-mutants: cannot revert $name" >&2; exit 2; }
  fi
  printf '%-6s %-40s %-9s %s\n' "$fc" "$name" "$result" "$test_args"
  rows+=("| $fc | \`$name\` | $what | \`${test_args##* }\` | $result |")
done

total=$((killed+survived+errors))
if [ "$errors" -gt 0 ]; then
  verdict="TOOLING: $errors of $total mutants could not be run"
  rc=2
elif [ "$survived" -gt 0 ]; then
  verdict="FAIL: $survived of $total mutants survived (their rule is not checked)"
  rc=1
else
  verdict="PASS: all $total mutants turned their test red"
  rc=0
fi
echo "arc011-mutants: $verdict"

if [ -n "$REPORT" ]; then
  {
    echo "| FC | Mutant | What it breaks | Test that must go red | Result |"
    echo "|---|---|---|---|---|"
    printf '%s\n' "${rows[@]}"
    echo
    echo "$verdict. Run $(date -u +%Y-%m-%dT%H:%MZ) on $(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)."
  } >"$REPORT"
fi
exit "$rc"
