# T-099: PostMessage / SessionEnd Hook Request for Claude Code
## Research Artifact | Created: 2026-03-18

---

## Problem (Original)

Claude Code's hook system lacked response-boundary events, making it impossible to enforce
framework governance on pure conversations — sessions with zero tool calls were invisible
to all enforcement. Two hooks were proposed:
- **PostMessage** — fires after each assistant response
- **SessionEnd** — fires on normal session exit

## Research Findings

### Claude Code Hook System (as of March 2026)

Claude Code now supports **24 hook event types**, significantly more than when T-094
originally identified the gap. The relevant ones:

| Hook | When It Fires | Our Use Case |
|------|---------------|--------------|
| **Stop** | When Claude finishes responding | Response boundary detection |
| **SessionEnd** | When a session terminates | Mandatory handover enforcement |
| **SubagentStop** | When a subagent finishes | Sub-agent result enforcement |
| **UserPromptSubmit** | Before Claude processes a prompt | Prompt enrichment/validation |
| **PreCompact** | Before context compaction | Auto-handover (already using) |
| **PostCompact** | After context compaction | Context recovery (already using) |
| **SessionStart** | Session begins/resumes | Context injection (already using) |

### Gap Analysis: What We Needed vs. What Exists

**PostMessage (response boundary):**
- **`Stop` hook** fires when Claude finishes responding, with `last_assistant_message` in input
- Supports `decision: "block"` to control flow
- **This IS our PostMessage.** It fires after every response, not just tool-based ones
- Issue #3145 requested exactly this, was resolved by pointing to Stop hook

**SessionEnd:**
- **Already exists** since v1.0.85
- Matchers: `clear`, `logout`, `prompt_input_exit`, `bypass_permissions_disabled`, `other`
- Known bugs: doesn't fire on /exit (#17885), doesn't fire on API 500 (#20197)
- **This IS our SessionEnd.** We can use it for mandatory handover enforcement

### What's Already Available for Our Framework

| Original Need | Claude Code Hook | Status |
|---------------|-----------------|--------|
| Enforce governance after each response | `Stop` | Available, not yet wired |
| Mandatory handover on session exit | `SessionEnd` | Available, not yet wired |
| Context injection on session start | `SessionStart` | Already using |
| Auto-handover before compaction | `PreCompact` | Already using |
| Tool call enforcement | `PreToolUse` / `PostToolUse` | Already using |
| Budget monitoring | `PreToolUse` (budget-gate.sh) | Already using |
| Sub-agent result management | `SubagentStop` | Available, not yet wired |

### What's Still Missing (True Gaps)

1. **Per-turn hook (between tool calls within a response):** Stop fires at response END,
   not between individual tool calls in a multi-tool response. For N-exchange guard within
   a single turn, we'd still need PostToolUse (which we already have).

2. **Reliable SessionEnd:** Known bugs mean it doesn't always fire. Not reliable enough
   for mandatory handover without fallback.

3. **Response content in Stop hook:** `last_assistant_message` is provided, but it's unclear
   if this includes the full response or just the final text segment.

### Submission Channel

- **GitHub Issues:** https://github.com/anthropics/claude-code/issues (primary channel)
- **Format:** Feature request with `[FEATURE]` prefix, description, use case, proposed API
- **No CONTRIBUTING.md** for code PRs — Claude Code is not open source for code contributions
- **Discord:** https://anthropic.com/discord for community discussion

## Go/No-Go Assessment

### Original T-099 Scope: Submit PR for PostMessage + SessionEnd

**NO-GO on the original scope** — both hooks already exist:
- `Stop` = our PostMessage (fires after each response)
- `SessionEnd` = already implemented (with known bugs)

### Revised Scope Options

**Option A: Wire existing hooks into framework (BUILD, not PR)**
- Add `Stop` hook to enforce N-exchange guard / conversation logging
- Add `SessionEnd` hook for mandatory handover enforcement
- This is internal framework work, not an Anthropic feature request

**Option B: File bug reports for SessionEnd reliability**
- SessionEnd doesn't fire on /exit (#17885 already filed)
- SessionEnd doesn't fire on API errors (#20197 already filed)
- These bugs are already reported — no new issues needed

**Option C: Request SubagentStop enhancements**
- `SubagentStop` provides `agent_transcript_path` and `last_assistant_message`
- Could be used for fw bus result enforcement
- Worth exploring but not a gap — it's an opportunity

## Recommendation

**Close T-099 as NO-GO** — the original problem (missing hooks) is solved by Claude Code's
evolution. The hooks exist. What remains is internal framework work to wire them up.

**Create new build task(s)** if desired:
- Wire `Stop` hook for conversation-level governance
- Wire `SessionEnd` hook for mandatory handover
- Wire `SubagentStop` for sub-agent result enforcement

## Sources

- [Claude Code Hooks Reference](https://code.claude.com/docs/en/hooks)
- [Issue #3145: postAllResponses hook](https://github.com/anthropics/claude-code/issues/3145) — resolved via Stop hook
- [Issue #4318: SessionStart and SessionEnd](https://github.com/anthropics/claude-code/issues/4318) — SessionStart implemented, SessionEnd later added
- [Issue #17885: SessionEnd doesn't fire on /exit](https://github.com/anthropics/claude-code/issues/17885)
- [Issue #20197: SessionEnd doesn't fire on API 500](https://github.com/anthropics/claude-code/issues/20197)
- [Issue #34340: Expose context window usage to hooks](https://github.com/anthropics/claude-code/issues/34340)
