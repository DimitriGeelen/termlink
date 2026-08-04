#!/usr/bin/env bash
# alloc-sink-check-fixtures.sh (T-2527) — load-bearing tests for
# scripts/check-alloc-sink-clamps.sh. No live binary; pure fixtures.
#
# Proves the detector (a) FIRES on an unclamped caller-param allocation sink,
# (b) does NOT fire once the same site is wrapped in `.clamp(...)`, (c) respects
# a binding-level clamp, (d) respects the allowlist, (e) skips comments &
# out-of-scope compound exprs. Run: bash tests/alloc-sink-check-fixtures.sh
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit 2

CHECK=scripts/check-alloc-sink-clamps.sh
[ -f "$CHECK" ] || { echo "FAIL: $CHECK not found"; exit 1; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
FIX="$TMP/fixture.rs"
EMPTY_ALLOW="$TMP/empty-allowlist"
: > "$EMPTY_ALLOW"

pass=0 fail=0
ok()   { echo "  ok: $1"; pass=$((pass+1)); }
bad()  { echo "  FAIL: $1"; fail=$((fail+1)); }

run() { bash "$CHECK" --root "$TMP" --allowlist "${1:-$EMPTY_ALLOW}" --no-heartbeat --json 2>/dev/null; }

# --- (a) unclamped bare-ident caller param -> MUST fire ----------------------
cat > "$FIX" <<'RS'
fn handler(p: Params) -> Vec<u8> {
    let count = p.count;
    let mut out = Vec::with_capacity(count);
    out
}
RS
out="$(run)"
if printf '%s' "$out" | grep -q '"ok":false' && printf '%s' "$out" | grep -q 'with_capacity(count)'; then
    ok "(a) unclamped Vec::with_capacity(count) fires"
else bad "(a) expected fire on unclamped count; got: $out"; fi

# --- (b) inline .clamp -> MUST NOT fire --------------------------------------
cat > "$FIX" <<'RS'
fn handler(p: Params) -> Vec<u8> {
    let count = p.count;
    let mut out = Vec::with_capacity(count.clamp(1, 256));
    out
}
RS
out="$(run)"
if printf '%s' "$out" | grep -q '"ok":true'; then
    ok "(b) inline .clamp(1,256) does not fire"
else bad "(b) expected clean on inline-clamped site; got: $out"; fi

# --- (c) binding-level clamp -> MUST NOT fire --------------------------------
cat > "$FIX" <<'RS'
fn handler(p: Params) -> Vec<u8> {
    let count = p.count.clamp(1, 256);
    let mut out = Vec::with_capacity(count);
    out
}
RS
out="$(run)"
if printf '%s' "$out" | grep -q '"ok":true'; then
    ok "(c) binding-level let count = ....clamp(...) clears the site"
else bad "(c) expected clean on binding-clamped site; got: $out"; fi

# --- (d) allowlist ack -> MUST NOT fire --------------------------------------
cat > "$FIX" <<'RS'
fn handler(p: Params) -> Vec<u8> {
    let count = p.count;
    let mut out = Vec::with_capacity(count);
    out
}
RS
ALLOW="$TMP/allow"
echo "$TMP/fixture.rs::with_capacity(count)" > "$ALLOW"
out="$(run "$ALLOW")"
if printf '%s' "$out" | grep -q '"ok":true'; then
    ok "(d) allowlisted signature suppresses the site"
else bad "(d) expected clean on allowlisted site; got: $out"; fi

# --- (e) comment + compound expr -> MUST NOT fire ----------------------------
cat > "$FIX" <<'RS'
fn handler(p: Params) -> Vec<u8> {
    // pre-allocated (`Vec::with_capacity(count)`) documented in a comment
    let m = p.m;
    let mut out = Vec::with_capacity(m + 1);
    let lit = Vec::<u8>::with_capacity(128);
    let mat: Vec<u8> = Vec::with_capacity(out.len());
    out
}
RS
out="$(run)"
if printf '%s' "$out" | grep -q '"ok":true'; then
    ok "(e) comment mention + (m+1) compound + literal + .len() all skipped"
else bad "(e) expected clean; got: $out"; fi

# --- (f) Semaphore::new bare ident -> MUST fire ------------------------------
cat > "$FIX" <<'RS'
fn handler(p: Params) {
    let max_parallel = p.max_parallel.unwrap_or(10);
    let sem = tokio::sync::Semaphore::new(max_parallel);
}
RS
out="$(run)"
if printf '%s' "$out" | grep -q '"ok":false' && printf '%s' "$out" | grep -q 'Semaphore::new(max_parallel)'; then
    ok "(f) unclamped Semaphore::new(max_parallel) fires (the T-2523 shape)"
else bad "(f) expected fire on Semaphore::new(max_parallel); got: $out"; fi

echo ""
echo "alloc-sink fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ] || exit 1
