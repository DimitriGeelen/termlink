#!/usr/bin/env bash
# version-derivation-check-fixtures.sh (T-2746)
#
# Hermetic load-bearing proof for scripts/check-version-derivation.sh — no cargo, no
# build, no live binary, no repo state. Builds a scratch crate root with synthetic crate
# trees exercising each branch of the detector and asserts the verdict + exit code on
# each. This is the deterministic complement to the real-tree revert proof (removing the
# emit line from crates/termlink-session/build.rs re-fires the real check).
#
# Exit 0 = all fixtures pass; exit 1 = a fixture assertion failed.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-version-derivation.sh"
[ -f "$CHECK" ] || { echo "FAIL: check script not found at $CHECK" >&2; exit 1; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
ROOT="$SCRATCH/crates"; mkdir -p "$ROOT"
ALLOW="$SCRATCH/allowlist"; : > "$ALLOW"

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
run_rc() { bash "$CHECK" --no-heartbeat --root "$ROOT" --allowlist "$ALLOW" "$@" >/dev/null 2>&1; echo $?; }
run_out() { bash "$CHECK" --no-heartbeat --root "$ROOT" --allowlist "$ALLOW" "$@" 2>&1; }

mkcrate() { # <name>
    mkdir -p "$ROOT/$1/src"
}

echo "== fixture 1: reads the version, no build.rs (the T-2744 / T-1458 shape) =="
mkcrate reader-no-buildrs
cat > "$ROOT/reader-no-buildrs/src/lib.rs" <<'RS'
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
RS
rc="$(run_rc)"
assert_rc "fires on a version-reading crate with no build.rs" 1 "$rc"
out="$(run_out)"
assert_contains "names the crate" "$out" "reader-no-buildrs"
assert_contains "names the missing-file reason" "$out" "has no build.rs"

echo "== fixture 2: PL-219 control — a crate that reads nothing and has no build.rs =="
mkcrate silent-crate
cat > "$ROOT/silent-crate/src/lib.rs" <<'RS'
pub fn add(a: u32, b: u32) -> u32 { a + b }
RS
out="$(run_out)"
assert_absent "does NOT fire on a crate that never reads the version" "$out" "silent-crate"

echo "== fixture 3: reads and derives — the healthy shape =="
mkcrate good-crate
cat > "$ROOT/good-crate/src/lib.rs" <<'RS'
pub fn version() -> &'static str { env!("CARGO_PKG_VERSION") }
RS
cat > "$ROOT/good-crate/build.rs" <<'RS'
fn main() {
    println!("cargo:rustc-env=CARGO_PKG_VERSION=1.2.3");
}
RS
out="$(run_out)"
assert_absent "does NOT fire on a crate that reads and derives" "$out" "good-crate"

echo "== fixture 4: has a build.rs that never emits the override =="
mkcrate inert-buildrs
cat > "$ROOT/inert-buildrs/src/lib.rs" <<'RS'
pub fn version() -> &'static str { env!("CARGO_PKG_VERSION") }
RS
cat > "$ROOT/inert-buildrs/build.rs" <<'RS'
fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
}
RS
out="$(run_out)"
assert_contains "fires on a build.rs that never emits the override" "$out" "inert-buildrs"
assert_contains "distinguishes it from the missing-file reason" "$out" "never emits"

echo "== fixture 5: a commented-out read is not a read =="
mkcrate comment-only
cat > "$ROOT/comment-only/src/lib.rs" <<'RS'
// Historically this returned env!("CARGO_PKG_VERSION"), see T-2744 for why it moved.
pub fn version() -> &'static str { "static" }
RS
out="$(run_out)"
assert_absent "prose mentioning the macro does not count as a read" "$out" "comment-only"

echo "== fixture 6: allowlist suppresses firing but still reports =="
echo "reader-no-buildrs  # fixture: acknowledged on purpose" > "$ALLOW"
rc="$(run_rc)"
# inert-buildrs is still firing, so rc stays 1 — assert the allowlisted one left the fire list.
out="$(run_out)"
assert_absent "allowlisted crate is not in the firing list" "${out%%---*}" "↳ reader-no-buildrs"
json="$(run_out --json)"
assert_contains "allowlisted crate appears in acknowledged[]" "$json" "reader-no-buildrs"
assert_contains "acknowledged_count reflects it" "$json" '"acknowledged_count":1'

echo "== fixture 7: allowlisting every firing crate returns rc 0 =="
echo "inert-buildrs  # fixture: acknowledged on purpose" >> "$ALLOW"
rc="$(run_rc)"
assert_rc "clean once all firing crates are acknowledged" 0 "$rc"
json="$(run_out --json)"
assert_contains "json reports ok:true" "$json" '"ok":true'
assert_contains "json still counts both acknowledgements" "$json" '"acknowledged_count":2'

echo "== fixture 8: removing an allowlist line re-fires that crate =="
grep -v 'inert-buildrs' "$ALLOW" > "$ALLOW.tmp" && mv "$ALLOW.tmp" "$ALLOW"
rc="$(run_rc)"
assert_rc "re-fires once the acknowledgement is removed" 1 "$rc"

echo "== fixture 9: a bad scan root is a tooling error, not a clean bill =="
bash "$CHECK" --no-heartbeat --root "$SCRATCH/nope" --allowlist "$ALLOW" >/dev/null 2>&1
assert_rc "missing scan root exits 2" 2 "$?"

echo
echo "version-derivation fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ] || exit 1
exit 0
