# T-2155 — RCA: Misread Budget at Post-Compaction /resume

**Task:** [T-2155](../../.tasks/active/T-2155-rca-misread-budget-at-post-compaction-re.md)
**Workflow:** inception (RCA → fix proposal to framework-agent)
**Status:** GO recommended
**Owner:** human (decision authority)

## Summary

After /resume from a compacted session, the agent narrated budget level
from a historical Task tool ephemeral output (`/tmp/claude-0/.../tasks/<id>.output`)
re-injected into the SessionStart:compact persisted-output block — not
from the canonical cache at `.context/working/.budget-status`. Result:
session-long misread (~140K of real headroom un-used; agent constrained
behaviour as if 27K from critical when actually at OK).

Structural fix lives at the framework layer (`/resume` skill +
SessionStart:compact hook). Proposing GO to file the pickup to
framework-agent.

## Reproduction

This session reproduced the slip end-to-end. The trail:

1. `/resume` invoked post-compaction.
2. SessionStart:compact hook injected a persisted-output preview block
   containing a prior session's Read of a Task tool ephemeral output
   file. That file's JSON payload included
   `{"level":"urgent","tokens":273016,"timestamp":...,"source":...}`.
3. Agent's `/resume` skill (current workflow at `.claude/commands/resume.md`)
   gathered: handover, git status, tasks, tool counter, web server.
   It did NOT include `cat .context/working/.budget-status`.
4. Agent parsed the historical JSON as current state, narrated "27K
   headroom up to 300K", and wrapped at "~298K".
5. Verification after the fact: `cat .context/working/.budget-status`
   showed `level=ok, tokens=159350` at session start. The 273K figure
   was 4+ hours stale.

## Root Cause

Two compounding gaps:

1. **`/resume` skill omits the budget cache.** The skill workflow (Step
   1 of the gather phase) enumerates 5 reads but does NOT include the
   canonical budget-status file. Without that step, the agent has no
   structural prompt to ground budget claims against ground truth.

2. **SessionStart:compact context-recovery flow re-injects historical
   tool results verbatim.** A historical Task tool output containing
   budget-shaped JSON is structurally indistinguishable from a current
   read of the cache (same key names: `level`, `tokens`, `timestamp`,
   `source`). System-reminders are not visually marked as historical.

## Why Structurally Allowed

- `CLAUDE.md` "After context compaction (mid-session recovery)" section
  names `fw resume status` + `fw resume sync` but does NOT name
  `.context/working/.budget-status` as a required check.
- The budget-gate hook caches level/tokens in the file but never
  re-asserts it into agent context. Enforcement is runtime (blocks
  tools) but visibility is pull-only.
- No framework signal forces "ground budget claims against cache".

## Prevention — Three Options

### Option A: Extend `/resume` skill (smallest change)

Add Step 1.6 to the gather phase:
```
6. cat .context/working/.budget-status 2>/dev/null
```
Add to the Summary template:
```
- Budget: {level} ({tokens} tokens) from cache
```
One-line addition. Ships via `userSettings:resume` update. Behaviour-only
fix — depends on agent reading + obeying the skill.

### Option B: SessionStart:compact hook surfaces current budget (best leverage)

Hook re-reads `.context/working/.budget-status` post-compaction and
prepends `Current budget: level={X} tokens={Y}` to the persisted-output
block, BEFORE any historical tool results. This makes ground truth the
first thing the agent sees, not buried in stale snapshots. Hook-side
enforcement, independent of agent behaviour.

### Option C: CLAUDE.md doc-only (weakest)

Update "After context compaction" recovery steps to name the cache file
explicitly. Relies on agent reading + obeying — same failure mode that
produced this slip.

## Recommendation

**Option B as primary** + **Option A as defence-in-depth**. B closes the
gap structurally (hook-side, independent of agent behaviour). A backs
it up at the skill layer so the cache read happens regardless of which
recovery path fires.

## Pickup Path

Fix lives at the framework layer:
- `/resume` ships from `userSettings:` (the framework-managed userSettings
  skill bundle).
- SessionStart:compact hook ships from `framework:` (vendored hook in
  `.agentic-framework/hooks/` or upstream `/opt/999-AEF`).

Project-side fix would not propagate to other consumer projects.
Pickup-to-framework-agent via `framework:pickup` topic is the correct
escalation path (T-1814-class). Sibling task
[T-2156](../../.tasks/active/) is the captured envelope awaiting GO
here to authorize the post.

## Decision Gate

**GO if:**
- Root cause identified with bounded fix path ✓
- Fix is scoped, testable, and reversible ✓ (one-line skill addition + one hook prepend)

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope ✗ (it doesn't)
- Fix cost exceeds benefit given current evidence ✗ (slip is reproducible on any future post-compaction /resume that encounters budget-shaped historical JSON; ~140K of misread headroom per occurrence)

GO recommended.

## Dialogue Log

This RCA was authored autonomously after the agent (this session)
reproduced the slip end-to-end during a prior compacted-and-resumed
window. No human-dialogue segments — the artifact reflects the agent's
analysis of its own failure mode, with structural fix proposals ranked
by leverage.

## References

- `.context/working/.budget-status` — canonical budget cache (file
  written by the PreToolUse budget-gate hook)
- `.claude/commands/resume.md` — `/resume` skill workflow (Step 1
  gather phase enumerates the reads that omit the cache)
- `CLAUDE.md` § "After context compaction (mid-session recovery)" —
  doc-side gap
- T-1814 — class precedent for "framework allowed undetected drift,
  pickup-to-framework-agent" escalation
- T-2156 — sibling pickup envelope, captured horizon=next
