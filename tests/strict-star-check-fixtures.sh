#!/usr/bin/env bash
# tests/strict-star-check-fixtures.sh (T-2703)
#
# Hermetic fixtures for scripts/check-strict-star.sh — the guard for
# "spokes never connect to one another" (substrate doc § 10, T-2702 F1).
#
# The load-bearing cases are 2 and 3: a NEW unacknowledged dial site must FIRE,
# and removing an acknowledgement must re-fire that site. Everything else here
# would also pass against a check that always says "clean"; those two are what
# prove it can detect anything at all. A guard only ever shown to say "clean"
# has not been shown to work.
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHECK="${STRICT_STAR_CHECK:-$HERE/../scripts/check-strict-star.sh}"
pass=0; fail=0

ok() { pass=$((pass+1)); }
bad() { fail=$((fail+1)); echo "  FAIL: $1"; }
assert_rc() { if [ "$1" = "$2" ]; then ok; else bad "$3 (expected rc=$1, got rc=$2)"; fi; }
assert_contains() { case "$1" in *"$2"*) ok ;; *) bad "$3 (missing: $2)" ;; esac; }
assert_not_contains() { case "$1" in *"$2"*) bad "$3 (unexpectedly present: $2)" ;; *) ok ;; esac; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# --- Case 1: an acknowledged dial site scans clean ---------------------------
mkdir -p "$TMP/c1/src"
cat > "$TMP/c1/src/hub_client.rs" <<'RS'
pub async fn connect_hub(addr: &str) {
    let _s = TcpStream::connect(addr).await.unwrap();
}
RS
printf '%s  # the hub address; the star centre\n' "$TMP/c1/src/hub_client.rs::connect_hub" > "$TMP/c1.allow"
out="$(bash "$CHECK" --root "$TMP/c1/src" --allowlist "$TMP/c1.allow" 2>&1)"; rc=$?
assert_rc 0 "$rc" "an acknowledged dial site scans clean"
assert_contains "$out" "clean" "clean run says so"
assert_contains "$out" "scope:" "clean run still discloses its scope (T-2680)"

# --- Case 2 (LOAD-BEARING): a NEW unacknowledged dial site FIRES -------------
# This is the property T-2571's behavioural test structurally cannot have.
mkdir -p "$TMP/c2/src"
cat > "$TMP/c2/src/mesh.rs" <<'RS'
pub async fn dial_peer_spoke(peer: &str) {
    let _s = TcpStream::connect(peer).await.unwrap();
}
RS
: > "$TMP/c2.allow"
out="$(bash "$CHECK" --root "$TMP/c2/src" --allowlist "$TMP/c2.allow" 2>&1)"; rc=$?
assert_rc 1 "$rc" "a new spoke-to-spoke dial fires"
assert_contains "$out" "dial_peer_spoke" "the firing site is named"
assert_contains "$out" "unacknowledged" "the verdict names the class"

# --- Case 3 (LOAD-BEARING): removing an acknowledgement re-fires -------------
out="$(bash "$CHECK" --root "$TMP/c1/src" --allowlist "$TMP/c2.allow" 2>&1)"; rc=$?
assert_rc 1 "$rc" "the same site fires once its acknowledgement is removed"
assert_contains "$out" "connect_hub" "the de-acknowledged site is named"

# --- Case 4: test-context dials are cleared automatically --------------------
mkdir -p "$TMP/c4/src"
cat > "$TMP/c4/src/server.rs" <<'RS'
pub fn serve() {}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn round_trip() {
        let _s = UnixStream::connect(&socket_path).await.unwrap();
    }
}
RS
: > "$TMP/c4.allow"
out="$(bash "$CHECK" --root "$TMP/c4/src" --allowlist "$TMP/c4.allow" 2>&1)"; rc=$?
assert_rc 0 "$rc" "a test connecting to a socket it bound itself is not a mesh"
assert_contains "$out" "1 test-context" "test-context sites are counted, not hidden"

# --- Case 5: a dial BEFORE the cfg(test) region still fires ------------------
# Guards against "put it in the test file and it disappears".
mkdir -p "$TMP/c5/src"
cat > "$TMP/c5/src/mixed.rs" <<'RS'
pub async fn production_dial(peer: &str) {
    let _s = TcpStream::connect(peer).await.unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn t() { let _ = UnixStream::connect("/x"); }
}
RS
: > "$TMP/c5.allow"
out="$(bash "$CHECK" --root "$TMP/c5/src" --allowlist "$TMP/c5.allow" 2>&1)"; rc=$?
assert_rc 1 "$rc" "a production dial above the test module still fires"
assert_contains "$out" "production_dial" "the production site is named"
assert_not_contains "$out" "fn t" "the test-module dial is not reported"

# --- Case 6: commented-out dials are not sites -------------------------------
mkdir -p "$TMP/c6/src"
cat > "$TMP/c6/src/doc.rs" <<'RS'
/// Historically this used TcpStream::connect(peer) directly.
// let _s = TcpStream::connect(peer).await;
pub fn nothing() {}
RS
: > "$TMP/c6.allow"
out="$(bash "$CHECK" --root "$TMP/c6/src" --allowlist "$TMP/c6.allow" 2>&1)"; rc=$?
assert_rc 2 "$rc" "a root with only commented dials has zero sites -> fail-closed, never clean"
assert_contains "$out" "Refusing to report clean" "zero-site scan refuses rather than reporting clean"

# --- Case 7: a missing scan root is a tooling error, not a clean bill --------
out="$(bash "$CHECK" --root "$TMP/nope" --allowlist "$TMP/c6.allow" 2>&1)"; rc=$?
assert_rc 2 "$rc" "a nonexistent root is a tooling error"
assert_contains "$out" "fail-closed" "the refusal names why"

# --- Case 8: JSON envelope carries the counts and the scope ------------------
out="$(bash "$CHECK" --root "$TMP/c2/src" --allowlist "$TMP/c2.allow" --json 2>&1)"; rc=$?
assert_rc 1 "$rc" "json mode preserves the exit contract"
assert_contains "$out" '"ok":false' "json reports not-ok when firing"
assert_contains "$out" '"scope":' "json carries the scope disclaimer"
assert_contains "$out" '"dial_peer_spoke"' "json names the firing fn"
if command -v jq >/dev/null 2>&1; then
    if printf '%s' "$out" | jq -e . >/dev/null 2>&1; then ok; else bad "json output is not valid JSON"; fi
else
    ok
fi

# --- Case 9: PL-219 real-tree control ----------------------------------------
# The fixture suite must not pass while the live invocation is broken.
out="$(cd "$HERE/.." && bash "$CHECK" 2>&1)"; rc=$?
assert_rc 0 "$rc" "the real tree scans clean"
assert_contains "$out" "acknowledged" "the real-tree run reports its acknowledged count"

echo "strict-star check fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ] || exit 1
exit 0
