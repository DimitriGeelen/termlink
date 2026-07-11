#!/usr/bin/env bash
# T-2403 — unit test for tl-claude.sh build_claude_cmd PROJECT_ROOT sanitization
# (+ composition with the T-2400 IS_SANDBOX auto-accept prefix). Sources
# tl-claude in lib mode (TL_CLAUDE_LIB=1) so preflight/dispatch don't run.
set -u

SELF_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=/dev/null
TL_CLAUDE_LIB=1 . "${SELF_DIR}/tl-claude.sh"

fail=0
check() { if [ "$2" = "$3" ]; then echo "ok: $1"; else echo "FAIL: $1"$'\n'"  expected: $2"$'\n'"  got:      $3"; fail=1; fi; }

TL_CLAUDE_CMD=claude

# reachable + default → sanitize AND auto-accept, composed as one env prefix.
REACHABLE=1; TL_NO_AUTO_ACCEPT=0; TL_KEEP_PROJECT_ROOT=0
CLAUDE_ARGS=(--resume)
check "reachable: env -u PROJECT_ROOT + IS_SANDBOX + skip flag" \
    "env -u PROJECT_ROOT IS_SANDBOX=1 claude --resume --dangerously-skip-permissions" \
    "$(build_claude_cmd)"

# non-reachable → sanitize only, no IS_SANDBOX, no skip flag.
REACHABLE=0; TL_NO_AUTO_ACCEPT=0; TL_KEEP_PROJECT_ROOT=0
CLAUDE_ARGS=(--resume)
check "non-reachable: env -u PROJECT_ROOT only" \
    "env -u PROJECT_ROOT claude --resume" \
    "$(build_claude_cmd)"

# opt-out keeps PROJECT_ROOT, still auto-accepts when reachable.
REACHABLE=1; TL_NO_AUTO_ACCEPT=0; TL_KEEP_PROJECT_ROOT=1
CLAUDE_ARGS=(--resume)
check "TL_KEEP_PROJECT_ROOT=1 reachable: IS_SANDBOX only (no env -u)" \
    "IS_SANDBOX=1 claude --resume --dangerously-skip-permissions" \
    "$(build_claude_cmd)"

# opt-out + non-reachable → back to the bare pre-T-2403 command.
REACHABLE=0; TL_NO_AUTO_ACCEPT=0; TL_KEEP_PROJECT_ROOT=1
CLAUDE_ARGS=(--resume)
check "TL_KEEP_PROJECT_ROOT=1 non-reachable: bare claude" \
    "claude --resume" \
    "$(build_claude_cmd)"

# reachable but caller already passed the skip flag → sanitize, no double flag.
REACHABLE=1; TL_NO_AUTO_ACCEPT=0; TL_KEEP_PROJECT_ROOT=0
CLAUDE_ARGS=(--resume --dangerously-skip-permissions)
check "idempotent: caller's skip flag not duplicated, still sanitized" \
    "env -u PROJECT_ROOT claude --resume --dangerously-skip-permissions" \
    "$(build_claude_cmd)"

if [ "$fail" -eq 0 ]; then echo "RESULT: PASS"; else echo "RESULT: FAIL"; exit 1; fi
