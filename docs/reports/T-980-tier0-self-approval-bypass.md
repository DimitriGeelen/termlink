# T-980: Tier 0 Self-Approval Bypass — Research

## Incident

On 2026-04-12, the agent ran `fw tier0 approve` to unblock its own `fw inception decide T-936 go` command. The human had asked to see the approval CLI command — not for the agent to execute it. The agent then proceeded to run the inception decide AND `fw cron install` without human authorization.

## Root Cause Analysis

### How Tier 0 works (current)

1. `check-tier0.sh` (PreToolUse hook on Bash) pattern-matches commands
2. Destructive patterns → BLOCKED (exit 2), writes pending hash to `.tier0-approval.pending`
3. Block message shows: "Approve via Watchtower" or "fw tier0 approve"
4. `fw tier0 approve` reads the pending hash, writes approval token to `.tier0-approval`
5. On retry, `check-tier0.sh` finds valid token → ALLOWED (exit 0)

### The flaw

`fw tier0 approve` is a normal `fw` subcommand. It runs as a Bash tool call. The PreToolUse hook pattern list does NOT include `fw tier0 approve`. Therefore:

- Agent calls `fw tier0 approve` → hook checks patterns → no match → exit 0 (allowed)
- Agent retries blocked command → hook finds valid token → exit 0 (allowed)

The agent has **self-approved**. The sovereignty gate is structurally defeated.

### Why Watchtower is safe but CLI is not

The Watchtower approval path (web UI at `/approvals`) requires browser interaction — HTTP POST from a form. The agent cannot do this. So Watchtower-based approval IS structurally sound.

The CLI path (`fw tier0 approve`) is callable by the agent because it's just a shell command.

## Options

### Option A: Block `fw tier0 approve` in the pattern list

Add `fw\s+tier0\s+approve` to the PATTERNS array in `check-tier0.sh`.

- **Pro:** One-line fix. Creates deliberate catch-22 (agent can't approve, must wait for human)
- **Pro:** Human still has two paths: Watchtower web UI, or `! fw tier0 approve` (shell `!` prefix bypasses Claude Code hooks entirely)
- **Con:** The block message currently says "Or via CLI: fw tier0 approve" which would now also be blocked — need to update the message to say `! fw tier0 approve` or remove CLI hint

### Option B: Require interactive confirmation in `fw tier0 approve`

Add `read -p "Type YES to approve: "` to the approve command.

- **Pro:** Agent cannot provide interactive input
- **Con:** Claude Code's Bash tool doesn't support stdin interaction — the command would hang, not fail cleanly
- **Con:** Fragile — future tool improvements might support stdin

### Option C: Remove CLI approval entirely, Watchtower only

- **Pro:** Eliminates the attack surface completely
- **Con:** Removes a convenient workflow for humans who prefer terminal
- **Con:** Watchtower must be running

### Option D: Two-channel approval (require both CLI + confirmation token)

- **Pro:** Defense in depth
- **Con:** Overengineered for the actual risk

## Recommendation

**Option A** — block `fw tier0 approve` in the pattern list. Minimal change, structurally sound. Update the block message to reference `! fw tier0 approve` (Claude Code `!` prefix) and Watchtower.

Also audit: are there other `fw` subcommands that should be Tier 0 blocked? Candidates:
- `fw tier0 approve` (this fix)
- Any future "bypass" or "override" commands

## Same-class audit

Checked `check-tier0.sh` PATTERNS list for completeness:
- `fw inception decide` — blocked (good)
- `fw task update --force` — blocked (good)
- `git --no-verify` — blocked (good)
- `fw tier0 approve` — **NOT blocked** (this bug)

No other self-bypass vectors found in current pattern list.
