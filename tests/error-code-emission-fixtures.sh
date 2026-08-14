#!/usr/bin/env bash
# error-code-emission-fixtures.sh (T-2699)
#
# Hermetic proof for scripts/check-error-code-emission.sh — no live binary, no host
# state. Builds a scratch definition file plus a scratch crate root and drives the
# check through its --def-file / --root / --allowlist seams.
#
# The load-bearing fixtures are 4 and 5. Fixture 4 pins the case the check exists for:
# a code whose only "use" is a BUILDER living beside the definition, with no callers —
# that is PROTOCOL_VERSION_TOO_OLD, which every conventional coverage signal reports as
# green because the builder has a passing unit test. Fixture 5 pins the false NEGATIVE
# the check itself shipped with on its first run: prose mentioning "(-32011)" in a doc
# comment was counted as an emission, silently clearing a genuinely dead code.
#
# Exit 0 = all fixtures pass; exit 1 = a fixture assertion failed.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-error-code-emission.sh"
[ -f "$CHECK" ] || { echo "FAIL: check not found at $CHECK" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "FAIL: jq required" >&2; exit 1; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
SRC="$SCRATCH/src"; DEF="$SCRATCH/src/control.rs"; ALLOW="$SCRATCH/allowlist"
mkdir -p "$SRC"; : > "$ALLOW"

pass=0; fail=0
ok()  { pass=$((pass+1)); echo "  ok: $1"; }
bad() { fail=$((fail+1)); echo "  FAIL: $1" >&2; }
assert_rc() { [ "$2" -eq "$3" ] && ok "$1 (rc=$3)" || bad "$1 — expected rc=$2 got rc=$3"; }
assert_eq() { [ "$2" = "$3" ] && ok "$1 ($3)" || bad "$1 — expected '$2' got '$3'"; }

run()      { bash "$CHECK" --def-file "$DEF" --root "$SRC" --allowlist "$ALLOW" >/dev/null 2>&1; echo $?; }
run_json() { bash "$CHECK" --def-file "$DEF" --root "$SRC" --allowlist "$ALLOW" --json 2>/dev/null; }
reset()    { rm -f "$SRC"/*.rs 2>/dev/null; : > "$ALLOW"; }

write_def() { # <body>
    printf 'pub mod error_code {\n%s}\n' "$1" > "$DEF"
}

# --- fixture 1: an emitted code is clean ----------------------------------------
reset
write_def '    pub const ALPHA_FAILED: i64 = -32001;
'
cat > "$SRC/handler.rs" <<'RS'
fn reject() -> Response {
    ErrorResponse::new(id, error_code::ALPHA_FAILED, "nope").into()
}
RS
assert_rc "an emitted code is clean" 0 "$(run)"
assert_eq "one code scanned" "1" "$(run_json | jq -r '.checked')"

# --- fixture 2: a never-emitted code fires --------------------------------------
reset
write_def '    pub const BETA_UNUSED: i64 = -32002;
'
cat > "$SRC/handler.rs" <<'RS'
fn unrelated() -> u32 { 7 }
RS
assert_rc "a never-emitted code fires" 1 "$(run)"
assert_eq "firing names the code" "BETA_UNUSED" "$(run_json | jq -r '.firing[0].code')"
assert_eq "firing carries the wire value" "-32002" "$(run_json | jq -r '.firing[0].value')"

# --- fixture 3: allowlisting suppresses, removal re-fires (load-bearing) --------
echo "BETA_UNUSED  # reserved for protocol v2" > "$ALLOW"
assert_rc "allowlisted dead code is suppressed" 0 "$(run)"
assert_eq "still counted as scanned" "1" "$(run_json | jq -r '.checked')"
: > "$ALLOW"
assert_rc "clearing the allowlist re-fires" 1 "$(run)"

# --- fixture 4: THE CASE THIS EXISTS FOR — builder beside the definition --------
# A code constructed by a helper that lives in the DEFINING file, with no callers
# outside it. Every conventional coverage signal reads green (the builder has a test),
# yet the code can never be returned. This is PROTOCOL_VERSION_TOO_OLD.
reset
write_def '    pub const GAMMA_VERSION: i64 = -32011;
}

pub fn check_gamma(declared: u8, required: u8) -> Option<ErrorResponse> {
    if declared >= required { return None; }
    Some(ErrorResponse::new(id, error_code::GAMMA_VERSION, "too old"))
}

#[cfg(test)]
mod t {
    #[test]
    fn builder_works() {
        assert_eq!(check_gamma(1, 2).unwrap().error.code, error_code::GAMMA_VERSION);
    }
'
cat > "$SRC/handler.rs" <<'RS'
fn handles_requests() -> u32 { 1 }
RS
assert_rc "a builder with no callers still fires" 1 "$(run)"
assert_eq "the unwired code is named" "GAMMA_VERSION" "$(run_json | jq -r '.firing[0].code')"

# --- fixture 5: THE CHECK'S OWN FALSE NEGATIVE — prose is not emission ----------
# The first run of this check cleared PROTOCOL_VERSION_TOO_OLD because tools.rs
# mentions "(-32011)" in a doc comment. A comment ABOUT a code is not a use of it.
reset
write_def '    pub const DELTA_DEAD: i64 = -32011;
'
cat > "$SRC/handler.rs" <<'RS'
/// Distinguishes auth-fail, protocol-version-skew (-32011), and method-not-found.
fn documented_only() -> u32 { 3 }
RS
assert_rc "a doc-comment mention does NOT count as emission" 1 "$(run)"
assert_eq "the code is still reported dead" "DELTA_DEAD" "$(run_json | jq -r '.firing[0].code')"

# --- fixture 6: a bare numeric literal DOES count as emission -------------------
# The inverse guard: emitting by hand-written number is bad style but is still an
# emission, and reporting it as dead would be a false positive.
reset
write_def '    pub const EPS_LITERAL: i64 = -32012;
'
cat > "$SRC/handler.rs" <<'RS'
fn reject() -> Response {
    ErrorResponse::new(id, -32012, "emitted by literal").into()
}
RS
assert_rc "a bare numeric emission counts as emitted" 0 "$(run)"

# --- fixture 7: the defining file alone does not count as a use -----------------
reset
write_def '    pub const ZETA_SELF: i64 = -32013;
'
# No other files at all in the scan root except an unrelated one.
cat > "$SRC/other.rs" <<'RS'
fn nothing() -> u32 { 0 }
RS
assert_rc "a constant does not count as its own use" 1 "$(run)"

# --- fixture 8: multiple dead codes are all reported ----------------------------
reset
write_def '    pub const A1: i64 = -32031;
    pub const A2: i64 = -32032;
    pub const A3: i64 = -32033;
'
cat > "$SRC/handler.rs" <<'RS'
fn only_uses_one() -> Response {
    ErrorResponse::new(id, error_code::A2, "x").into()
}
RS
assert_rc "mixed tree fires" 1 "$(run)"
assert_eq "both dead codes reported, the live one not" "2" "$(run_json | jq -r '.firing | length')"
assert_eq "three codes scanned" "3" "$(run_json | jq -r '.checked')"

# --- fixture 9: absent definition file is a tooling error ----------------------
rc="$(bash "$CHECK" --def-file "$SCRATCH/nope.rs" --root "$SRC" --allowlist "$ALLOW" >/dev/null 2>&1; echo $?)"
assert_rc "absent definition file exits 2 (fail-closed)" 2 "$rc"

echo
echo "error-code-emission-fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ] || exit 1
exit 0
