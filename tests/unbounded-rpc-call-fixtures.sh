#!/usr/bin/env bash
# unbounded-rpc-call-fixtures.sh (T-2669)
#
# Hermetic load-bearing proof for scripts/check-unbounded-rpc-call.sh — no live binary,
# no repo state. Builds a scratch scan-root with synthetic .rs files exercising each
# branch of the detector and asserts the check's verdict + exit code on each.
#
# This is the deterministic complement to the real-tree revert proof: reverting any
# T-2669-migrated site (e.g. tools.rs termlink_ping) back to the raw `rpc_call` form
# re-fires the real check on that fn by name, and restoring returns the tree to clean.
#
# The most load-bearing fixtures here are B1/B2 and D1: B1/B2 pin the property the whole
# migration rests on — that the BOUNDED variants are excluded by the regex's construction
# rather than by an exception list, so a migrated site can never be mistaken for an
# unmigrated one. D1 pins the empty-acknowledged-set JSON, which was a real bug in this
# check's first draft: under `pipefail` a `grep .` on empty input fails the pipeline, so
# a trailing `|| echo '[]'` fired AFTER jq had already printed its own `[]`, emitting
# `[]\n[]` and producing invalid JSON on exactly the path a firing tree takes.
#
# Exit 0 = all fixtures pass; exit 1 = a fixture assertion failed.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-unbounded-rpc-call.sh"
[ -f "$CHECK" ] || { echo "FAIL: check script not found at $CHECK" >&2; exit 1; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
SRC="$SCRATCH/src"; mkdir -p "$SRC"
ALLOW="$SCRATCH/allowlist"; : > "$ALLOW"

pass=0; fail=0
assert_rc() { # <desc> <expected-rc> <actual-rc>
    if [ "$2" -eq "$3" ]; then pass=$((pass+1)); echo "  ok: $1 (rc=$3)";
    else fail=$((fail+1)); echo "  FAIL: $1 — expected rc=$2 got rc=$3" >&2; fi
}
assert_contains() { # <desc> <needle> <haystack>
    if printf '%s' "$3" | grep -qF -- "$2"; then pass=$((pass+1)); echo "  ok: $1";
    else fail=$((fail+1)); echo "  FAIL: $1 — expected to find '$2'" >&2; fi
}
assert_not_contains() { # <desc> <needle> <haystack>
    if printf '%s' "$3" | grep -qF -- "$2"; then fail=$((fail+1)); echo "  FAIL: $1 — did not expect '$2'" >&2;
    else pass=$((pass+1)); echo "  ok: $1"; fi
}
run()      { bash "$CHECK" --no-heartbeat --root "$SRC" --allowlist "$ALLOW" >/dev/null 2>&1; echo $?; }
run_out()  { bash "$CHECK" --no-heartbeat --root "$SRC" --allowlist "$ALLOW" 2>&1; }
run_json() { bash "$CHECK" --no-heartbeat --root "$SRC" --allowlist "$ALLOW" --json 2>/dev/null; }
reset_src() { rm -f "$SRC"/*.rs; : > "$ALLOW"; }

echo "== A: the core detection =="
reset_src
cat > "$SRC/a.rs" <<'RS'
async fn termlink_ping(&self) -> String {
    match client::rpc_call(reg.socket_path(), "termlink.ping", serde_json::json!({})).await {
        Ok(resp) => ok(resp),
        Err(e) => err(e),
    }
}
RS
assert_rc "A1 unbounded rpc_call with no timeout in scope fires" 1 "$(run)"
assert_contains "A2 firing output names the enclosing fn" "termlink_ping" "$(run_out)"

echo "== B: the bounded variants are excluded BY CONSTRUCTION =="
# This is the property the entire T-2669 migration rests on. If the regex could match a
# bounded call, every migrated site would keep firing and the check would be unusable.
reset_src
cat > "$SRC/b.rs" <<'RS'
async fn migrated_socket(&self) -> String {
    match client::rpc_call_with_timeout(reg.socket_path(), "termlink.ping", p, client::DEFAULT_RPC_TIMEOUT).await {
        Ok(resp) => ok(resp),
        Err(e) => err(e),
    }
}
async fn migrated_addr(&self) -> String {
    match client::rpc_call_addr_with_timeout(&addr, "termlink.ping", p, d).await {
        Ok(resp) => ok(resp),
        Err(e) => err(e),
    }
}
RS
assert_rc "B1 rpc_call_with_timeout / rpc_call_addr_with_timeout do not fire" 0 "$(run)"
assert_contains "B2 bounded-only tree reports zero call sites scanned" '0 call site(s) scanned' "$(run_out)"

echo "== C: an explicit tokio::time::timeout wrap is an equally valid bound =="
reset_src
cat > "$SRC/c.rs" <<'RS'
async fn wrapped(&self) -> String {
    let fut = client::rpc_call(sock, "query.status", p);
    match tokio::time::timeout(std::time::Duration::from_secs(5), fut).await {
        Ok(Ok(resp)) => ok(resp),
        _ => err(),
    }
}
RS
assert_rc "C1 explicit timeout wrap clears the site" 0 "$(run)"

echo "== D: the allowlist, and the empty-acknowledged-set JSON =="
reset_src
cat > "$SRC/d.rs" <<'RS'
async fn termlink_event_subscribe(&self) -> String {
    match client::rpc_call(reg.socket_path(), "event.subscribe", params).await {
        Ok(resp) => ok(resp),
        Err(e) => err(e),
    }
}
RS
json_firing="$(run_json)"
assert_rc "D1 unacknowledged long-poll still fires before it is ledgered" 1 "$(run)"
if command -v python3 >/dev/null 2>&1; then
    if printf '%s' "$json_firing" | python3 -c 'import json,sys; json.load(sys.stdin)' 2>/dev/null; then
        pass=$((pass+1)); echo "  ok: D2 --json is valid JSON when the acknowledged set is EMPTY"
    else
        fail=$((fail+1)); echo "  FAIL: D2 --json is malformed when the acknowledged set is empty" >&2
    fi
fi
echo "$SRC/d.rs::termlink_event_subscribe::unbounded-rpc-call  # intentional long-poll" > "$ALLOW"
assert_rc "D3 ledgering the site clears it" 0 "$(run)"
assert_contains "D4 a ledgered site is still COUNTED, not vanished" '1 acknowledged' "$(run_out)"

echo "== E: the clean path must not over-claim (T-2680) =="
# A ledger holding un-migrated hazards must never be reported as if every entry were an
# intentional long-poll — that is the exact over-claim T-2680 was raised for.
out_clean="$(run_out)"
assert_contains "E1 clean output states an ack is not a claim of boundedness" 'NOT a claim' "$out_clean"
assert_not_contains "E2 clean output does not label all acks 'intentional long-poll'" 'acknowledged as intentional long-poll' "$out_clean"
assert_contains "E3 clean output carries the scope disclaimer" 'not a proof' "$out_clean"

echo "== F: comments are not code =="
reset_src
cat > "$SRC/f.rs" <<'RS'
async fn only_a_comment(&self) -> String {
    // historical: this used to call client::rpc_call(sock, "x", p) unbounded
    ok()
}
RS
assert_rc "F1 a commented-out rpc_call does not fire" 0 "$(run)"

echo "== G: tooling errors are exit 2, never a clean bill =="
assert_rc "G1 a missing scan root is a tooling error" 2 "$(bash "$CHECK" --no-heartbeat --root "$SCRATCH/nope" >/dev/null 2>&1; echo $?)"
assert_rc "G2 an unknown flag is a tooling error" 2 "$(bash "$CHECK" --no-heartbeat --bogus >/dev/null 2>&1; echo $?)"

echo "== H: PL-219 control — the real tree scans clean =="
cd "$REPO_ROOT" || exit 1
assert_rc "H1 the real repo tree is clean" 0 "$(bash "$CHECK" --no-heartbeat >/dev/null 2>&1; echo $?)"

echo
echo "unbounded-rpc-call fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ] || exit 1
exit 0
