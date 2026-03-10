# C-003 Audit Check: Path Resolution Bug in Shared-Tooling Mode

**Task:** T-076 (Address recurring audit warnings)
**Date:** 2026-03-10
**Author:** Claude Code agent (TermLink project)
**For:** Framework agent (`/opt/999-Agentic-Engineering-Framework`)

---

## Summary

The C-003 audit check (`audit.sh:1346-1354`) reports a false `WARN` in shared-tooling mode. The C-003 inception research checkpoint logic **already exists** in the framework's `checkpoint.sh`, but the audit looks in the wrong location.

## Root Cause Analysis

### What the audit checks (audit.sh:1348)

```bash
if grep -q "inception-research-counter\|INCEPTION_RESEARCH_INTERVAL\|C-003" \
   "$PROJECT_ROOT/agents/context/checkpoint.sh" 2>/dev/null; then
```

It checks `$PROJECT_ROOT/agents/context/checkpoint.sh` — a **project-local** path.

### What actually exists

| Location | Has C-003 logic? | Notes |
|----------|-----------------|-------|
| `/usr/local/opt/agentic-fw/libexec/agents/context/checkpoint.sh` | **YES** (lines 238-267) | Framework install path (shared tooling) |
| `$PROJECT_ROOT/agents/context/checkpoint.sh` | **NO** (file doesn't exist) | Project has no local copy |

### How the hook is wired (project's .claude/settings.json)

```json
{
  "command": "PROJECT_ROOT=/Users/dimidev32/001-projects/010-termlink /usr/local/opt/agentic-fw/libexec/agents/context/checkpoint.sh post-tool"
}
```

The project references the **framework install path**, not a project-local script. This is correct for shared-tooling mode (via `brew install` or symlink to `/usr/local/opt/`).

### The bug

The audit hardcodes `$PROJECT_ROOT/agents/context/checkpoint.sh` but should also check the framework's libexec path. In shared-tooling mode, project repos don't contain agent scripts — they reference the centrally installed framework.

## Where C-003 logic lives (framework checkpoint.sh)

```bash
# --- Research Capture Checkpoint (C-003, T-194) ---
# Every 20 tool calls, check if focused inception task has uncommitted research
INCEPTION_RESEARCH_INTERVAL=20
if [ $((count % INCEPTION_RESEARCH_INTERVAL)) -eq 0 ]; then
    # ... reads focus.yaml, checks workflow_type: inception
    # ... looks for docs/reports/${focus_task}-* artifact
    # ... warns if missing or stale (>30 min)
fi
```

This logic is functional. It fires every 20 tool calls during inception tasks. The audit just can't find it because it looks in the wrong directory.

## Proposed Fix

In `agents/audit/audit.sh`, change the C-003 check to resolve the checkpoint.sh path the same way hooks resolve it — check both the project-local path AND the framework libexec path:

```bash
# C-003 OE: Check checkpoint hook is wired and firing
CHECKPOINT_LOG="$CONTEXT_DIR/working/.inception-checkpoint-log"

# Resolve checkpoint.sh — may be project-local or in framework libexec
CHECKPOINT_SH="$PROJECT_ROOT/agents/context/checkpoint.sh"
if [ ! -f "$CHECKPOINT_SH" ]; then
    # Shared-tooling mode: resolve from fw's own libexec
    FW_LIBEXEC="$(cd "$(dirname "$(readlink -f "$0")")/../.." && pwd)"
    CHECKPOINT_SH="$FW_LIBEXEC/agents/context/checkpoint.sh"
fi

if grep -q "inception-research-counter\|INCEPTION_RESEARCH_INTERVAL\|C-003" \
   "$CHECKPOINT_SH" 2>/dev/null; then
    pass "C-003: Research checkpoint logic present in checkpoint.sh"
else
    warn "C-003: Research checkpoint logic missing from checkpoint.sh" \
         "checkpoint.sh doesn't contain C-003 inception research check" \
         "Add C-003 research checkpoint to checkpoint.sh post-tool handler"
fi
```

Alternatively, extract the path resolution into a helper function since this pattern (project-local vs. framework-libexec) likely affects other audit checks too.

## Impact Assessment

- **Severity:** Low (false warning, not a false pass)
- **Frequency:** Every audit run in shared-tooling mode
- **Blast radius:** Only the C-003 check; other audit checks don't reference `$PROJECT_ROOT/agents/`
- **Risk if unfixed:** Noise — the warning trains users to ignore audit output, which undermines D4 (audit trend detection)

## Investigation Prompt for Framework Agent

```
Investigate and fix the C-003 audit path resolution bug in agents/audit/audit.sh.

Context:
- Line 1348 checks $PROJECT_ROOT/agents/context/checkpoint.sh for C-003 logic
- In shared-tooling mode (brew install), projects don't have local agent scripts
- The C-003 logic exists in the framework's own checkpoint.sh (libexec path)
- The audit should resolve the checkpoint.sh path the same way hooks do

Steps:
1. Read agents/audit/audit.sh around line 1346-1354 (C-003 check)
2. Search for other checks that reference $PROJECT_ROOT/agents/ — they may have the same bug
3. Implement path resolution that checks both project-local and framework-libexec
4. Test with a project that uses shared-tooling mode (no local agents/ directory)
5. Consider extracting a resolve_agent_script() helper if multiple checks need it

Acceptance criteria:
- C-003 check passes when checkpoint.sh is in framework libexec (not project-local)
- No regression for projects that DO have project-local agent scripts
- Any other $PROJECT_ROOT/agents/ references in audit.sh are also fixed
```

## Related

- **T-194**: Original task that added C-003 research checkpoint to checkpoint.sh
- **CTL-020** (cron audit): Separate issue — no cron job configured for periodic audits
- **D5** (lifecycle anomalies): Separate issue — historical early tasks with fast cycle times
