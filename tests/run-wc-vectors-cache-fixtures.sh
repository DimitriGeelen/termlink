#!/usr/bin/env bash
# guard-layer: source
#
# T-3545 — run-wc-vectors.sh offline fallback. Hermetic: the fetch goes to a URL that cannot
# resolve, the cache is a temp dir, and WC_VECTORS_SELECT_ONLY stops before cargo runs.
set -uo pipefail
S="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/scripts/run-wc-vectors.sh"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok() { echo "  ok   $1"; PASS=$((PASS+1)); }
bad() { echo "  FAIL $1"; [ -n "${2:-}" ] && echo "       $2"; FAIL=$((FAIL+1)); }
run() { WC_VECTORS_URL="file:///nonexistent/vectors.yaml" WC_VECTORS_CACHE_DIR="$TMP/cache" \
        WC_VECTORS_SELECT_ONLY=1 bash "$S" --ref abc1234 2>&1; }

out="$(run)"; rc=$?
[ "$rc" = "2" ] && grep -q 'no cached copy for abc1234' <<<"$out" && ok "no network, no cache → exit 2, nothing judged" \
    || bad "cache miss" "rc=$rc $out"

mkdir -p "$TMP/cache" && printf 'vectors: []\n' > "$TMP/cache/abc1234.yaml"
out="$(run)"; rc=$?
[ "$rc" = "0" ] && grep -q 'cached copy' <<<"$out" && ok "no network, cached REF → uses the copy and says so" \
    || bad "cache hit" "rc=$rc $out"

out="$(WC_VECTORS_URL="file:///nonexistent/vectors.yaml" WC_VECTORS_CACHE_DIR="$TMP/cache" \
       WC_VECTORS_SELECT_ONLY=1 bash "$S" --ref other99 2>&1)"; rc=$?
[ "$rc" = "2" ] && ok "a copy for another REF is never used" || bad "ref isolation" "rc=$rc $out"

printf 'vectors: [fresh]\n' > "$TMP/src.yaml"
out="$(WC_VECTORS_URL="file://$TMP/src.yaml" WC_VECTORS_CACHE_DIR="$TMP/cache2" \
       WC_VECTORS_SELECT_ONLY=1 bash "$S" --ref new1 2>&1)"; rc=$?
[ "$rc" = "0" ] && cmp -s "$TMP/src.yaml" "$TMP/cache2/new1.yaml" && ok "a successful fetch refreshes the cache" \
    || bad "cache write" "rc=$rc $out"

printf '\nrun-wc-vectors cache fixtures: %d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" = 0 ]
