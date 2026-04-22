---
id: T-1189
name: "Build fw hook-enable command — T-977 claimed complete but bin/hook-enable.sh missing (G-015 Hit #2)"
description: >
  T-977 closed 2026-04-12 with 4 Agent AC [x] claiming bin/hook-enable.sh exists and fw hook-enable route was added. G-015 audit confirmed both are missing. Build honest deliverable in vendored .agentic-framework/ copy, create follow-up upstream-mirror task (T-1188 pattern).

status: started-work
workflow_type: build
owner: human
horizon: now
tags: [framework, hook, g-015-repair]
components: []
related_tasks: [T-977, T-1187, T-1188]
created: 2026-04-22T18:33:20Z
last_update: 2026-04-22T18:33:20Z
date_finished: null
---

# T-1189: Build fw hook-enable command — T-977 claimed complete but bin/hook-enable.sh missing (G-015 Hit #2)

## Context

T-977 claimed `bin/hook-enable.sh` and `fw hook-enable` route exist. G-015 blast-radius
audit (2026-04-22) confirmed both absent — `find / -name 'hook-enable*'` returns empty.
Second G-015 hit (first was T-976/T-1187/pl007-scanner).

Existing `bin/fw` has a generic `hook <name>` RUNNER (dispatches to
`$AGENTS_DIR/context/<name>.sh`) but no REGISTRAR. Human currently must hand-edit
`.claude/settings.json` to wire a new hook (see T-1187 Human RUBBER-STAMP).

This task builds the registrar: `fw hook-enable <name> --matcher <pat> --event <evt>`
idempotently adds the matching entry to the settings.json hooks array.

Same vendored-copy constraint as T-1187: `.agentic-framework/` is gitignored in
termlink, so the patch does not persist without an upstream mirror task
(T-1190 will follow, matching the T-1188 pattern).

**B-005 implication:** settings.json is framework-protected from agent edits. This
command is meant to be run BY THE HUMAN in their own shell (interactive invocation).
Agent invocation will be correctly blocked by the same check-active-task hook that
blocked this session's mechanical completion of T-1187's RUBBER-STAMP.

## Acceptance Criteria

### Agent
- [x] `.agentic-framework/bin/hook-enable.sh` exists and is executable
- [x] Accepts `--name <str> --matcher <pat> --event <PostToolUse|PreToolUse|SessionStart|PreCompact|Stop|SubagentStop>` (explicit allowlist prevents typo-creating bogus hook slots)
- [x] Edits `.claude/settings.json` via a Python one-shot (preserves JSON formatting, no partial writes, atomic rename)
- [x] Idempotent: re-running with identical args leaves the file byte-identical (exit 0, prints "already registered")
- [x] Adds `.agentic-framework/bin/fw hook <name>` as the command (not a raw path)
- [x] `bin/fw` dispatches `fw hook-enable <args…>` to this script
- [x] `bin/fw help` shows the new subcommand
- [x] Verification block embeds a dry-run (copy settings.json to /tmp, run, diff is a valid JSON diff, restore)

### Human
- [ ] [RUBBER-STAMP] Register pl007-scanner via the new command (unblocks T-1187 Human AC too)
  **Steps:**
  1. From `/opt/termlink`: `cd /opt/termlink && .agentic-framework/bin/fw hook-enable --name pl007-scanner --matcher Bash --event PostToolUse`
  2. Verify: `grep pl007-scanner .claude/settings.json` shows one entry
  3. Run twice: second invocation prints "already registered" (exit 0), file unchanged
  **Expected:** `.claude/settings.json` contains exactly one pl007-scanner entry under `PostToolUse` → Bash matcher.
  **If not:** Check `.agentic-framework/bin/hook-enable.sh` is executable and `.agentic-framework/bin/fw hook-enable` dispatches. Run with `-x` for trace.

- [ ] [RUBBER-STAMP] Mirror the new artifacts to framework repo (see T-1190 follow-up)

## Verification

test -x /opt/termlink/.agentic-framework/bin/hook-enable.sh
grep -q "hook-enable" /opt/termlink/.agentic-framework/bin/fw
bash -c 'cp /opt/termlink/.claude/settings.json /tmp/t1189-before.json && /opt/termlink/.agentic-framework/bin/hook-enable.sh --name t1189-dryrun --matcher Bash --event PostToolUse --file /tmp/t1189-before.json --dry-run | python3 -c "import sys,json; json.loads(sys.stdin.read())" && rm -f /tmp/t1189-before.json'

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-04-22T18:33:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1189-build-fw-hook-enable-command--t-977-clai.md
- **Context:** Initial task creation
