# T-176: Hook Audit — Framework Usage of Claude Code Hooks

## Current State

Framework configures **4 of ~24** available Claude Code hook types:

| Hook Type | Matchers | Scripts |
|-----------|----------|---------|
| **PreCompact** | 1 | `fw hook pre-compact` (auto-handover) |
| **SessionStart** | 2 | `fw hook post-compact-resume` (context injection) |
| **PreToolUse** | 4 | `check-active-task`, `check-tier0`, `budget-gate`, `block-plan-mode` |
| **PostToolUse** | 4 | `checkpoint`, `error-watchdog`, `check-dispatch`, `check-fabric-new-file` |

Total: 11 individual hook matchers across 4 event types.

## Unused Hooks — Value Assessment

### HIGH Value (wire now)

**Stop** — fires after each Claude response
- Closes pure-conversation governance gap (T-094)
- Enables per-response logging for conversations without tool calls
- Currently: no enforcement between tool calls in dialogue-heavy exchanges

**SessionEnd** — fires on session termination
- Mandatory handover enforcement (T-174)
- Known reliability bugs: Claude Code issues #17885 (no fire on `/exit`), #20197 (no fire on API 500)
- Needs fallback mechanism (timer-based or wrapper script)

### MEDIUM Value (wire later)

**SubagentStop** — fires after subagent completes
- Sub-agent result validation (T-175)
- Chain-of-thought capture for agent mesh orchestration
- Would improve dispatch protocol compliance

**UserPromptSubmit** — fires before Claude processes user prompt
- Prompt enrichment (inject context, warnings)
- Could enforce "check task before starting work" at prompt level
- Lower priority since PreToolUse already gates edits

### LOW Value

**PostCompact** — fires after context compaction
- Complementary to PreCompact (which already handles handover)
- Useful only for audit/recovery, which PreCompact + SessionStart already cover

**Other ~15 undocumented hooks** — insufficient documentation to assess. May include: InstructionsLoaded, ConfigChange, PreMessage, PostMessage, PreSubagentStart, etc. Need Claude Code API docs for full inventory.

## Gaps in Current Configuration

1. **No response-level governance** — between tool calls, Claude can produce arbitrary responses with no hook firing
2. **No session exit enforcement** — sessions can end without handover (SessionEnd not wired)
3. **No sub-agent result validation** — dispatched agents return results unchecked
4. **Tool coverage is comprehensive** — PreToolUse/PostToolUse cover the main enforcement needs

## Recommendation

**Priority order for hook wiring:**

1. **Stop hook** (T-173) — immediate, closes the biggest governance gap
2. **SessionEnd hook** (T-174) — needs reliability workaround, high value
3. **SubagentStop** (T-175) — medium priority, improves agent mesh
4. Skip others until Claude Code documents the full hook API

## Evidence

- Settings checked: `/opt/termlink/.claude/settings.json`
- Hook reference: T-099 report (`docs/reports/T-099-postmessage-sessionend-hook-request.md`)
- Framework hook scripts: `.agentic-framework/bin/fw hook *`
- Claude Code hook issues: #17885 (SessionEnd /exit), #20197 (SessionEnd API 500)
