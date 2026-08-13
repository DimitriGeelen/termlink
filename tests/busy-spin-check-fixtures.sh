#!/usr/bin/env bash
# busy-spin-check-fixtures.sh (T-2672)
#
# Hermetic load-bearing proof for scripts/check-busy-spin.sh — no live binary, no repo
# state. Builds a scratch scan-root with synthetic .rs files exercising each branch of the
# detector and asserts the check's verdict + exit code on each. This is the deterministic
# complement to the real-tree revert proof (reverting the T-2673 sleep-on-error at
# execution.rs cmd_request / file.rs cmd_file_receive / remote.rs cmd_remote_events
# re-fires the real check on that loop).
#
# Exit 0 = all fixtures pass; exit 1 = a fixture assertion failed.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-busy-spin.sh"
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
run() { bash "$CHECK" --no-heartbeat --root "$SRC" --allowlist "$ALLOW" "$@" >/dev/null 2>&1; echo $?; }

# --- fixture 1: long-poll loop, error arm re-iterates with NO sleep (should FIRE) --------
cat > "$SRC/spin.rs" <<'RS'
async fn cmd_spin() {
    loop {
        match client::rpc_call(sock, "event.subscribe", params).await {
            Ok(resp) => handle(resp),
            Err(e) => {
                tracing::warn!("Subscribe error: {}", e);
            }
        }
    }
}
RS
assert_rc "no-sleep long-poll loop fires" 1 "$(run)"

# --- fixture 2: same loop WITH the 500ms sleep-on-error (the T-2673 remediation; CLEAN) --
cat > "$SRC/spin.rs" <<'RS'
async fn cmd_spin() {
    loop {
        match client::rpc_call(sock, "event.subscribe", params).await {
            Ok(resp) => handle(resp),
            Err(e) => {
                tracing::warn!("Subscribe error: {}", e);
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }
    }
}
RS
assert_rc "sleep-on-error long-poll loop is clean" 0 "$(run)"

# --- fixture 3: error-arm RETURNS (exits loop) — grep still flags (no sleep); allowlist ---
# documents the design honestly: exit-on-error safety is NOT auto-detected from the body,
# it is acknowledged via the allowlist (the real tree's cmd_wait / termlink_request etc).
cat > "$SRC/exits.rs" <<'RS'
async fn cmd_exits() -> String {
    loop {
        match client::rpc_call(sock, "event.subscribe", params).await {
            Ok(resp) => handle(resp),
            Err(e) => return json_err(format!("connection lost: {e}")),
        }
    }
}
RS
rm -f "$SRC/spin.rs"
assert_rc "error-arm-return loop is a candidate (fires pre-allowlist)" 1 "$(run)"
echo "$SRC/exits.rs::cmd_exits::busy-spin" > "$ALLOW"
assert_rc "allowlisted exit-on-error loop is suppressed" 0 "$(run)"
: > "$ALLOW"
assert_rc "same site fires once allowlist cleared" 1 "$(run)"
rm -f "$SRC/exits.rs"

# --- fixture 4: a loop that does NOT dispatch a long-poll RPC is NOT a candidate (CLEAN) --
# (documents the narrow anchor: bounded recv / analytics / parse loops are excluded because
#  they never re-dispatch event.subscribe/collect/poll — the class cannot apply.)
cat > "$SRC/bounded.rs" <<'RS'
async fn cmd_bounded() {
    loop {
        match rx.recv().await {
            Some(msg) => process(msg),
            None => continue,
        }
    }
}
RS
assert_rc "non-long-poll loop is not a candidate" 0 "$(run)"
rm -f "$SRC/bounded.rs"

# --- fixture 5: event.collect variant also detected (should FIRE) ------------------------
cat > "$SRC/collect.rs" <<'RS'
async fn cmd_collect_spin() {
    loop {
        let resp = match client::rpc_call(sock, "event.collect", params).await {
            Ok(r) => r,
            Err(_) => continue,
        };
        handle(resp);
    }
}
RS
assert_rc "event.collect no-sleep loop fires" 1 "$(run)"
rm -f "$SRC/collect.rs"

# --- fixture 6: event.poll variant with sleep is clean ----------------------------------
cat > "$SRC/poll.rs" <<'RS'
async fn cmd_poll_ok() {
    loop {
        match client::rpc_call(sock, "event.poll", params).await {
            Ok(resp) => handle(resp),
            Err(_) => {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                continue;
            }
        }
    }
}
RS
assert_rc "event.poll with sleep is clean" 0 "$(run)"
rm -f "$SRC/poll.rs"

echo
echo "busy-spin-check-fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
