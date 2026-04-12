# T-972: Deep RCA — Agent Command Amnesia

## The Incident Timeline (this session, 2026-04-12)

1. **09:30** — User says "use termlink inject!" (3rd+ time across sessions)
2. **09:47** — PL-007 memory updated, CLAUDE.md rule added
3. **09:50** — T-969: `fw inception decide` auto-invokes `fw task review` (structural fix)
4. **09:51** — T-970: review.sh port detection fixed (3000 to 3002)
5. **09:54** — T-971: browser open fixed (root to desktop user sudo)
6. **09:55** — Agent outputs bare `fw inception decide` command to user — VIOLATION
7. **09:56** — User: "hello !!!! what did we just worked on ??!!!!"
8. **09:57** — Agent uses `fw task review` (correct)
9. **09:57** — Tier 0 blocks, outputs "Or via CLI: ./bin/fw tier0 approve" — FRAMEWORK VIOLATION
10. **09:58** — Agent relays the Tier 0 command to user — AGENT RELAYS FRAMEWORK VIOLATION
11. **09:59** — User: "why did now already again fail ?!!!!"

## Three Reinforcing Root Causes

### RC-1: Framework scripts output bare commands (the source)

Every gate script follows the same anti-pattern:
```bash
echo "Run this command:"
echo "  fw inception decide T-XXX go --rationale '...'"
exit 1
```

Affected scripts:
- `check-tier0.sh` — Tier 0 approval suggestion
- `inception.sh` — Review requirement hint
- `update-task.sh` — AC check hint
- `verify-acs.sh` — Human AC review hint
- `review.sh` — Decision command at bottom

These are framework-generated messages. The AGENT sees them in tool output and relays them.

### RC-2: No governance over agent text output (the gap)

Claude Code hooks:
- PreToolUse — can block tool calls
- PostToolUse — can warn after tool calls
- PreTextOutput — DOES NOT EXIST
- PostMessage — DOES NOT EXIST

The agent's prose output is completely ungoverned. Rules in CLAUDE.md compete with 200K+ tokens.

### RC-3: No shared URL helper (the multiplier)

Each script independently constructs Watchtower URLs:
- `review.sh:51` — reads PORT config, probes ports (fixed in T-970)
- `check-tier0.sh` — hardcoded 3000
- `verify-acs.sh:54` — reads PORT config
- `init.sh:788` — hardcoded 3000

Fixing one doesn't fix the others. No `_watchtower_url()` helper exists.

## The Fix Architecture

### Layer 1: Shared helper (eliminates RC-3)
Create `lib/watchtower.sh`:
- `_watchtower_url()` — port detection + host detection
- `_watchtower_open()` — URL + browser open (desktop user aware)

### Layer 2: Gate scripts invoke UX flows (eliminates RC-1)
Replace bare command suggestions with Watchtower URL + browser open.

### Layer 3: Agent output scanning (mitigates RC-2)
PostToolUse hook scans tool output for "fw inception decide" patterns and injects reminder.

## Recommendation

**Recommendation:** GO

**Rationale:** Structural, not behavioral. Five fixes in one session failed because they addressed symptoms. Three root causes reinforce each other — all must be fixed together.

**Evidence:**
- Agent violated PL-007 within 3 minutes of building it
- Tier 0 block message itself contains a bare command
- Port 3000 hardcoded in 4+ scripts despite fix in review.sh
- Same feedback given 3+ times across sessions
