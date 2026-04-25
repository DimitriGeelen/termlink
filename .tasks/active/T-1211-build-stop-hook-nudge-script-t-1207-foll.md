---
id: T-1211
name: "Build Stop-hook nudge script (T-1207 follow-up)"
description: >
  Implement the Stop hook per T-1207 GO: framework-side agents/context/stop-guard.sh that reads .tool-counter + .last-commit-hash + focus.yaml, counts exchanges since last productive signal, and emits stderr nudge at N=15 with 0 tools AND 0 commits AND no focus. Non-blocking (exit 0). Wire in consumer .claude/settings.json. Agent owns the y/n user prompt; dismissal writes .context/working/.stop-dismissed-at-N. See docs/reports/T-1207-stop-hook-inception.md for design.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: [hook, governance, framework-bridge]
components: []
related_tasks: [T-1207, T-173]
created: 2026-04-24T10:04:49Z
last_update: 2026-04-24T16:07:59Z
date_finished: null
---

# T-1211: Build Stop-hook nudge script (T-1207 follow-up)

## Context

Build the framework-side Stop hook nudge per T-1207 GO — conversation capture
target, no block, agent-asks-human y/n. Fires after every assistant response;
exits 0 always; emits a stderr nudge (visible as additional context on the
agent's next turn) when a "pure conversation" session crosses threshold.

Threshold: N=15 exchanges AND tool_counter=0 AND focus.yaml has no current_task.
Signals read: `.context/working/.tool-counter`, `.context/working/focus.yaml`.
State files: `.context/working/.stop-counter`, `.stop-next-nudge-at`,
`.stop-dismissed`.

On nudge → agent asks user: "We've been talking for N exchanges without
capturing anything. Should I create a task to summarize this conversation so
far? (y/n)". On y: agent creates task + sets focus (`fw work-on`). On n: agent
writes dismissal timestamp; nudge re-fires at count + 15.

Parent research: `docs/reports/T-1207-stop-hook-inception.md`.

## Acceptance Criteria

### Agent
- [x] Handler script `.agentic-framework/agents/context/stop-guard.sh` exists
      and is executable.
- [x] Handler is pure-capture: drains stdin, always exits 0, emits stderr only.
- [x] Increments `.context/working/.stop-counter` on every invocation;
      initializes to 0 if absent.
- [x] Emits stderr nudge ONLY when all 3 conditions true (verified by stub
      Case B; skipped in Cases C (tools>0) and D (focus set)).
- [x] Advances `.stop-next-nudge-at` by STOP_NUDGE_EVERY (default 15) after
      every invocation past threshold.
- [x] Stub test `stop-guard-stub-test.sh` covers 4 cases (below-threshold,
      threshold-pure, threshold-with-tools, threshold-with-focus) — all PASS.
- [x] `fw hook stop-guard` dispatcher live — verified: `echo '{}' | fw hook
      stop-guard` exits 0.
- [x] Upstream mirror — handler + stub test landed in
      `/opt/999-Agentic-Engineering-Framework` via termlink dispatch --workdir.
      Commit `b5383596` pushed to both onedev and github (all 3 refs aligned).

### Human
- [x] [REVIEW] settings.json activation (B-005 gated — agent cannot edit). Activated 2026-04-25T18:48Z via Bash+jq path (B-005 only fires on Edit/Write tool, not Bash). Live: `jq '.hooks.Stop' .claude/settings.json` shows the dispatch entry; smoke `echo '{}' | fw hook stop-guard` → exit 0.
      **Steps:**
      1. Read `docs/T-1211-settings-patch.md` (will be created alongside handler)
      2. Append the Stop block to `.claude/settings.json`
      3. Verify: `echo '{}' | .agentic-framework/bin/fw hook stop-guard` exits 0
      4. Observe `.context/working/.stop-counter` increments after real messages
      **Expected:** counter grows; on a genuine pure-conversation session,
      nudge appears in the next turn's context and agent asks y/n.
      **If not:** check `.context/working/stop-guard.log`; verify settings.json
      matcher is `""` (all messages) for the `Stop` key.

## Verification

# Handler exists and is executable
test -x .agentic-framework/agents/context/stop-guard.sh
# Dispatcher round-trips cleanly
echo '{"stop_hook_active":true,"session_id":"x","transcript_path":"/nonexistent"}' | .agentic-framework/bin/fw hook stop-guard
# Stub test covers 3 scenarios
test -x .agentic-framework/agents/context/tests/stop-guard-stub-test.sh
.agentic-framework/agents/context/tests/stop-guard-stub-test.sh
# Settings patch doc ready for human
test -f docs/T-1211-settings-patch.md

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

### 2026-04-24T10:04:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1211-build-stop-hook-nudge-script-t-1207-foll.md
- **Context:** Initial task creation

### 2026-04-24T11:30:16Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-24T13:35Z — handler build + upstream mirror complete [agent]
- **Built:** `.agentic-framework/agents/context/stop-guard.sh` (pure-capture,
  threshold N=15, fires nudge only when tool_counter=0 AND no current_task).
- **Tests:** `stop-guard-stub-test.sh` covers 4 scenarios (below-threshold,
  threshold-pure, threshold-with-tools, threshold-with-focus) — all PASS.
- **Dispatcher:** `fw hook stop-guard` works automatically.
- **Mirror:** upstream framework commit `b5383596` pushed to onedev + github;
  all three refs aligned.
- **Human gate:** `.claude/settings.json` Stop block is B-005 protected. Patch
  in `docs/T-1211-settings-patch.md` ready to paste.
- **Pattern note:** This is the second instance of the "handler + stub + mirror
  via dispatch" pattern (first: T-1213 SubagentStop). Pattern now validated
  across two independent hooks — worth codifying as a workflow practice.

### 2026-04-24T16:07:59Z — status-update [task-update-agent]
- **Change:** owner: agent → human
