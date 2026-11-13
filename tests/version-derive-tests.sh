#!/usr/bin/env bash
# T-3774: run the unit tests of crates/version-derive.rs (build.rs code is not reached by
# `cargo test`), then a negative control: the pre-T-3774 mapping (`major.minor.N`) put back
# into a scratch copy must FAIL them. Exit 0 = both held, 1 = a test failed or the control
# survived, 2 = tooling (no rustc).
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/crates/version-derive.rs"
command -v rustc >/dev/null 2>&1 || { echo "version-derive-tests: rustc not found" >&2; exit 2; }
[ -r "$SRC" ] || { echo "version-derive-tests: $SRC missing" >&2; exit 2; }
WORK="$(mktemp -d "${TMPDIR:-/var/tmp}/t3774-vd.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT
fail=0

if ! rustc --edition 2024 --test "$SRC" -o "$WORK/vd" > "$WORK/build.log" 2>&1; then
    cat "$WORK/build.log" >&2; echo "version-derive-tests: compile failed" >&2; exit 2
fi
if "$WORK/vd" > "$WORK/run.log" 2>&1 && grep -q 'test result: ok' "$WORK/run.log"; then
    echo "PASS  unit tests: $(grep 'test result' "$WORK/run.log")"
else
    cat "$WORK/run.log"; echo "FAIL  unit tests"; fail=1
fi

# Negative control: the old scheme.
OLD='Some(format!("{}.{n}", core.rsplitn(2, '"'"'.'"'"').last().unwrap()))'
NEW='Some(format!("{base}+{n}.g{sha}"))'
grep -qF "$NEW" "$SRC" || { echo "version-derive-tests: mutation anchor not found" >&2; exit 2; }
python3 -I -c 'import sys; s=open(sys.argv[1]).read(); open(sys.argv[2],"w").write(s.replace(sys.argv[3], sys.argv[4]))' \
    "$SRC" "$WORK/mutant.rs" "$NEW" "$OLD"
rustc --edition 2024 --test "$WORK/mutant.rs" -o "$WORK/mut" > "$WORK/mbuild.log" 2>&1 \
    || { cat "$WORK/mbuild.log" >&2; echo "version-derive-tests: mutant compile failed" >&2; exit 2; }
if "$WORK/mut" > "$WORK/mrun.log" 2>&1; then
    echo "FAIL  control: the old major.minor.N mapping passed the tests (they do not bite)"; fail=1
elif grep -q 'dev_build_never_collides_with_an_older_release' "$WORK/mrun.log"; then
    echo "PASS  control: old mapping fails ($(grep -c '\.\.\. FAILED' "$WORK/mrun.log") tests, incl. the 0.12.4 collision)"
else
    cat "$WORK/mrun.log"; echo "FAIL  control failed for an unexpected reason"; fail=1
fi
exit "$fail"
