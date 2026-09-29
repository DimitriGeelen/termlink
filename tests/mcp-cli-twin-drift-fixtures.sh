#!/usr/bin/env bash
# T-2999: fixtures for scripts/check-mcp-cli-twin-drift.sh.
#
# Hermetic: builds a scratch git repo with an mcp/ and a cli/ tree (pointed at via
# TWIN_MCP_DIR / TWIN_CLI_DIR) and a handful of commits, one per shape:
#   c1  create foo/foo_mcp and bar/bar_mcp together     → not flagged (creation)
#   c2  change only foo_mcp's body                      → FLAGGED (foo, mcp side)
#   c3  change bar and bar_mcp together                 → not flagged
#   c4  add baz to cli only (no mcp twin exists)        → not flagged (not a twin)
#   c5  add qux to cli; c6 add qux_mcp (parity catch-up) → not flagged (creation)
#   c7  change only qux (cli) after the pair exists     → FLAGGED (qux, cli side)
set -uo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-mcp-cli-twin-drift.sh"
W="$(mktemp -d)"; trap 'rm -rf "$W"' EXIT
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); echo "  ok   $1"; }
bad() { FAIL=$((FAIL+1)); echo "  FAIL $1 — $2"; }

cd "$W" && git init -q && git config user.email t@t && git config user.name t
mkdir -p mcp cli
fn() { printf 'pub fn %s() -> u32 {\n    %s\n}\n\n' "$1" "$2"; }
commit() { git add -A && git commit -qm "$1" && git rev-parse HEAD; }

{ fn foo_mcp 1; fn bar_mcp 1; } > mcp/tools.rs; { fn foo 1; fn bar 1; } > cli/lib.rs
C1=$(commit c1)
{ fn foo_mcp 2; fn bar_mcp 1; } > mcp/tools.rs
C2=$(commit c2-foo-mcp-only)
{ fn foo_mcp 2; fn bar_mcp 3; } > mcp/tools.rs; { fn foo 1; fn bar 3; } > cli/lib.rs
C3=$(commit c3-bar-both)
{ fn foo 1; fn bar 3; fn baz 1; } > cli/lib.rs
C4=$(commit c4-baz-cli-only)
{ fn foo 1; fn bar 3; fn baz 1; fn qux 1; } > cli/lib.rs
C5=$(commit c5-qux-cli-created)
{ fn foo_mcp 2; fn bar_mcp 3; fn qux_mcp 1; } > mcp/tools.rs
C6=$(commit c6-qux-mcp-parity)
{ fn foo 1; fn bar 3; fn baz 1; fn qux 7; } > cli/lib.rs
C7=$(commit c7-qux-cli-only)

run() { TWIN_MCP_DIR=mcp TWIN_CLI_DIR=cli bash "$CHECK" "$@"; }

echo "mcp-cli-twin-drift fixtures (T-2999):"
out="$(run --range "$C1..$C7" --json)"; rc=$?
[ "$rc" = 1 ] && ok "a range with one-sided changes exits 1" || bad "a range with one-sided changes exits 1" "rc=$rc"
got="$(jq -r '[.one_sided[] | "\(.twin):\(.changed)"] | sort | join(",")' <<< "$out")"
[ "$got" = "foo:mcp,qux:cli" ] && ok "flags exactly foo (mcp-only) and qux (cli-only after the pair existed)" \
    || bad "flags exactly foo (mcp-only) and qux (cli-only after the pair existed)" "got [$got]"
[ "$(jq -r '.twin_pairs' <<< "$out")" = 3 ] && ok "twin pairs counted from the tree (foo, bar, qux; baz has no twin)" \
    || bad "twin pairs counted from the tree" "got $(jq -r '.twin_pairs' <<< "$out")"

run --range "$C2..$C6" >/dev/null 2>&1; rc=$?   # c3..c6: both-sides change, non-twin, creation
[ "$rc" = 0 ] && ok "both-sides change, non-twin addition and parity creation are not flagged" \
    || bad "both-sides change, non-twin addition and parity creation are not flagged" "rc=$rc"

out="$(run --range "$C2..$C6" 2>&1)"
grep -q "REVIEW list" <<< "$out" && ok "clean output still states its scope (review list, not a verdict)" \
    || bad "clean output still states its scope" "$out"

run --range "nope..alsonope" >/dev/null 2>&1; rc=$?
[ "$rc" = 2 ] && ok "a bad range is a tooling error (2), never a clean 0" || bad "a bad range is a tooling error" "rc=$rc"

( cd "$(mktemp -d)" && TWIN_MCP_DIR=mcp TWIN_CLI_DIR=cli bash "$CHECK" >/dev/null 2>&1 ); rc=$?
[ "$rc" = 2 ] && ok "outside a git repo exits 2" || bad "outside a git repo exits 2" "rc=$rc"

echo ""
echo "mcp-cli-twin-drift fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = 0 ]
