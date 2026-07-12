#!/usr/bin/env bash
# tests/tl-claude-identity-binding.sh (T-2411) — hermetic test for per-agent
# identity binding into the reachable claude session. No live hub, no live PTY:
# sources tl-claude.sh in lib mode (TL_CLAUDE_LIB=1) and asserts build_claude_cmd
# threads TERMLINK_AGENT_ID; asserts agent-respond.sh prefers the env-respecting
# self-fp resolver when TERMLINK_AGENT_ID is set. Closes T-2411 AC-3.

set -u
SELF_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SELF_DIR/.."
fails=0
pass() { echo "  PASS: $1"; }
fail() { echo "  FAIL: $1"; fails=$((fails+1)); }

# --- Part 1: build_claude_cmd threads TERMLINK_AGENT_ID -------------------------
# Source in lib mode so no preflight/dispatch runs. Set the vars build_claude_cmd
# reads, then invoke it and inspect the composed command string.
build_cmd_with() {  # build_cmd_with <reachable> <agent_id> [claude_args...]
    (
        TL_CLAUDE_LIB=1
        # shellcheck disable=SC1091
        . "$ROOT/scripts/tl-claude.sh" >/dev/null 2>&1
        REACHABLE="$1"; AGENT_ID="$2"; shift 2
        CLAUDE_ARGS=("$@")
        build_claude_cmd
    )
}

out="$(build_cmd_with 1 ring20-concierge --resume)"
echo "$out" | grep -q 'TERMLINK_AGENT_ID=ring20-concierge' \
    && pass "reachable + agent-id -> TERMLINK_AGENT_ID in cmd" \
    || fail "reachable + agent-id: TERMLINK_AGENT_ID MISSING (got: $out)"

# still auto-accepts (T-2400) alongside the identity binding
echo "$out" | grep -q 'IS_SANDBOX=1' \
    && pass "reachable still composes IS_SANDBOX=1 auto-accept" \
    || fail "reachable: IS_SANDBOX=1 lost (got: $out)"

out="$(build_cmd_with 0 ring20-concierge --resume)"
echo "$out" | grep -q 'TERMLINK_AGENT_ID' \
    && fail "NOT reachable: TERMLINK_AGENT_ID should be omitted (got: $out)" \
    || pass "not reachable -> no TERMLINK_AGENT_ID (correct)"

out="$(build_cmd_with 1 '' --resume)"
echo "$out" | grep -q 'TERMLINK_AGENT_ID' \
    && fail "reachable but empty agent-id: TERMLINK_AGENT_ID should be omitted (got: $out)" \
    || pass "reachable + empty agent-id -> no TERMLINK_AGENT_ID (correct)"

# --- Part 2: agent-respond.sh prefers env-respecting resolver -------------------
# Verify the source wires the TERMLINK_AGENT_ID branch to `agent identity
# --resolve` BEFORE the PL-195 presence scrape (static-structure assertion —
# running it needs a live hub).
RESP="$ROOT/scripts/agent-respond.sh"
if grep -q 'TERMLINK_AGENT_ID:-' "$RESP" && grep -q 'agent identity --resolve' "$RESP"; then
    pass "agent-respond gates env-respecting resolver on TERMLINK_AGENT_ID"
else
    fail "agent-respond missing TERMLINK_AGENT_ID -> agent identity --resolve wiring"
fi
# the env-respecting resolve must come BEFORE the .senders[0] fallback
# grep the CODE lines (with --json), not the explanatory comments that name them
resolve_ln="$(grep -n 'agent identity --resolve --json' "$RESP" | head -1 | cut -d: -f1)"
scrape_ln="$(grep -n 'channel info agent-presence --json' "$RESP" | head -1 | cut -d: -f1)"
if [ -n "$resolve_ln" ] && [ -n "$scrape_ln" ] && [ "$resolve_ln" -lt "$scrape_ln" ]; then
    pass "env-respecting resolve precedes PL-195 presence scrape ($resolve_ln < $scrape_ln)"
else
    fail "resolve/scrape ordering wrong (resolve=$resolve_ln scrape=$scrape_ln)"
fi
# fallback preserved for non-agent-id sessions (no regression)
grep -q "channel info agent-chat-arc" "$RESP" \
    && pass "PL-195 chat-arc fallback preserved (no regression)" \
    || fail "PL-195 chat-arc fallback lost"

# --- Part 3: both edited scripts parse cleanly ---------------------------------
bash -n "$ROOT/scripts/tl-claude.sh" 2>/dev/null \
    && pass "bash -n scripts/tl-claude.sh clean" \
    || fail "bash -n scripts/tl-claude.sh FAILED"
bash -n "$ROOT/scripts/agent-respond.sh" 2>/dev/null \
    && pass "bash -n scripts/agent-respond.sh clean" \
    || fail "bash -n scripts/agent-respond.sh FAILED"

echo ""
if [ "$fails" -eq 0 ]; then echo "tl-claude-identity-binding: ALL PASS"; exit 0
else echo "tl-claude-identity-binding: $fails FAIL"; exit 1; fi
