#!/usr/bin/env bash
# mcp-parity-census-fixtures.sh (T-2747)
#
# Hermetic load-bearing proof for scripts/check-mcp-parity-census.sh — no cargo, no MCP
# server, no hub, no repo state. Builds scratch tools.rs / parity.rs stand-ins exercising
# each branch and asserts the verdict + exit code. Complements the real-tree proofs
# (deleting an allowlist line, and renaming a covered tool's reference in parity.rs, both
# re-fire the real check).
#
# Exit 0 = all fixtures pass; exit 1 = a fixture assertion failed.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-mcp-parity-census.sh"
[ -f "$CHECK" ] || { echo "FAIL: check script not found at $CHECK" >&2; exit 1; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
TOOLS="$SCRATCH/tools.rs"
PARITY="$SCRATCH/parity.rs"
ALLOW="$SCRATCH/allowlist"; : > "$ALLOW"

pass=0; fail=0
assert_rc() { if [ "$2" -eq "$3" ]; then pass=$((pass+1)); echo "  ok: $1 (rc=$3)";
    else fail=$((fail+1)); echo "  FAIL: $1 — expected rc=$2 got rc=$3" >&2; fi }
assert_contains() { if printf '%s' "$2" | grep -q -- "$3"; then pass=$((pass+1)); echo "  ok: $1";
    else fail=$((fail+1)); echo "  FAIL: $1 — output lacked '$3'" >&2; fi }
assert_absent() { if printf '%s' "$2" | grep -q -- "$3"; then fail=$((fail+1)); echo "  FAIL: $1 — output unexpectedly had '$3'" >&2;
    else pass=$((pass+1)); echo "  ok: $1"; fi }
run_rc() { bash "$CHECK" --no-heartbeat --tools-file "$TOOLS" --parity-file "$PARITY" --allowlist "$ALLOW" "$@" >/dev/null 2>&1; echo $?; }
run_out() { bash "$CHECK" --no-heartbeat --tools-file "$TOOLS" --parity-file "$PARITY" --allowlist "$ALLOW" "$@" 2>&1; }

cat > "$TOOLS" <<'RS'
    #[tool(
        name = "termlink_alpha",
        description = "first"
    )]
    async fn termlink_alpha() {}

    #[tool(
        name = "termlink_beta",
        description = "second"
    )]
    async fn termlink_beta() {}

    #[tool(
        name = "termlink_gamma",
        description = "third"
    )]
    async fn termlink_gamma() {}
RS

cat > "$PARITY" <<'RS'
async fn parity_alpha() {
    let mcp = call_mcp(&client, "termlink_alpha", json!({})).await;
}
RS

echo "== fixture 1: uncovered, unacknowledged tools fire =="
rc="$(run_rc)"
assert_rc "fires when tools are neither asserted nor acknowledged" 1 "$rc"
out="$(run_out)"
assert_contains "names beta" "$out" "termlink_beta"
assert_contains "names gamma" "$out" "termlink_gamma"
assert_absent "does NOT name the covered tool" "$out" "↳ termlink_alpha"
assert_contains "states the census, not a bare verdict" "$out" "of 3 MCP tool(s) are UNEXAMINED"

echo "== fixture 2: the census counts are correct in JSON =="
json="$(run_out --json)"
assert_contains "total is 3" "$json" '"total":3'
assert_contains "covered is 1" "$json" '"covered":1'
assert_contains "unexamined is 2" "$json" '"unexamined":2'
assert_contains "coverage_pct is reported" "$json" '"coverage_pct":"33.3"'

echo "== fixture 3: acknowledging suppresses firing but is still counted =="
printf 'termlink_beta  # fixture reason\ntermlink_gamma  # fixture reason\n' > "$ALLOW"
rc="$(run_rc)"
assert_rc "clean once every uncovered tool is acknowledged" 0 "$rc"
json="$(run_out --json)"
assert_contains "acknowledged counted, not dropped" "$json" '"acknowledged":2'
assert_contains "total still 3 — acknowledgement does not shrink the surface" "$json" '"total":3'
assert_contains "covered is unchanged by acknowledgement" "$json" '"covered":1'

echo "== fixture 4: the clean path does NOT report a bare green =="
out="$(run_out)"
assert_contains "clean output names the asserted count" "$out" "1 asserted by parity.rs"
assert_contains "clean output names the acknowledged count" "$out" "2 acknowledged"
assert_contains "clean output carries the scope disclaimer" "$out" "NOT that the two implementations agree"

echo "== fixture 5: removing an acknowledgement re-fires that tool (the ratchet) =="
printf 'termlink_beta  # fixture reason\n' > "$ALLOW"
rc="$(run_rc)"
assert_rc "re-fires when an acknowledgement is removed" 1 "$rc"
out="$(run_out)"
assert_contains "names exactly the un-acknowledged tool" "$out" "↳ termlink_gamma"

echo "== fixture 6: a NEW tool is in neither set and fires immediately =="
printf 'termlink_beta  # fixture reason\ntermlink_gamma  # fixture reason\n' > "$ALLOW"
cat >> "$TOOLS" <<'RS'

    #[tool(
        name = "termlink_delta",
        description = "added later"
    )]
    async fn termlink_delta() {}
RS
rc="$(run_rc)"
assert_rc "a newly added tool fires without any other change" 1 "$rc"
out="$(run_out)"
assert_contains "names the new tool" "$out" "↳ termlink_delta"
assert_contains "total grew to 4" "$(run_out --json)" '"total":4'

echo "== fixture 7: comment lines are not declarations and not assertions =="
cat > "$TOOLS" <<'RS'
    #[tool(
        name = "termlink_alpha",
        description = "first"
    )]
    async fn termlink_alpha() {}
    // A doc comment discussing `name = "termlink_phantom"` must not create a tool.
RS
cat > "$PARITY" <<'RS'
async fn parity_alpha() {
    let mcp = call_mcp(&client, "termlink_alpha", json!({})).await;
    // Prose mentioning "termlink_beta" is not an assertion about it.
}
RS
: > "$ALLOW"
json="$(run_out --json)"
assert_contains "commented tool name does not inflate the total" "$json" '"total":1'
assert_contains "only the real assertion counts as covered" "$json" '"covered":1'
rc="$(run_rc)"
assert_rc "clean when the only real tool is genuinely covered" 0 "$rc"

echo "== fixture 8: an empty tool surface is a tooling error, not a clean bill =="
: > "$TOOLS"
rc="$(run_rc)"
assert_rc "refuses to report a clean census over zero tools" 2 "$rc"

echo "== fixture 9: a missing source file is a tooling error =="
bash "$CHECK" --no-heartbeat --tools-file "$SCRATCH/nope.rs" --parity-file "$PARITY" --allowlist "$ALLOW" >/dev/null 2>&1
assert_rc "missing tools file exits 2" 2 "$?"

echo
echo "mcp-parity-census fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ] || exit 1
exit 0
