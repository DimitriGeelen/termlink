---
id: T-285
name: "termlink push — one-command cross-project file delivery with PTY notification"
description: >
  termlink push — one-command cross-project file delivery with PTY notification

status: work-completed
workflow_type: build
owner: human
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-25T20:02:11Z
last_update: '2026-08-18T18:59:17Z'
date_finished: 2026-03-25T20:27:38Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:01Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 2
      D3: 0
      D4: 4
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=2 (body:telemetry-or-audit-entry); D3=0 
      (no-signal); D4=4 (body:cross-machine); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:17Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-285: termlink push — one-command cross-project file delivery with PTY notification

## Context

T-283 investigation revealed cross-project notification is broken: 3-4 fragile commands, escaping issues, `send-file` doesn't auto-materialize, no delivery confirmation. This command replaces that with one atomic operation.

## Acceptance Criteria

### Agent
- [x] `termlink push --help` shows usage
- [x] `termlink push <hub-or-profile> <session> <file>` delivers file to target's inbox via `remote exec`
- [x] After delivery, injects one-line PTY notification: `[TERMLINK] Received: <filename> — cat <path>`
- [x] Reports delivery confirmation to sender (file path, size, target)
- [x] `--message` flag allows inline text push without a file
- [x] `--json` flag for structured output
- [x] Uses profile-based auth (resolves hub profiles like other remote commands)
- [x] Inbox path is `/tmp/termlink-inbox/` on target (created if missing)
- [x] All existing tests pass (`cargo test --workspace`)
- [x] 0 compiler warnings

### Human
- [x] [REVIEW] Push a file from .112 to fw-agent on .107, verify file arrives and agent sees notification
  **Steps:**
  1. `echo "test push" > /tmp/push-test.md`
  2. `termlink push mint fw-agent /tmp/push-test.md`
  3. `termlink remote exec mint fw-agent "cat /tmp/termlink-inbox/push-test.md"`
  **Expected:** File content matches, push reports success
  **If not:** Check `termlink remote list mint` for connectivity

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace

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

### 2026-03-25T20:02:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-285-termlink-push--one-command-cross-project.md
- **Context:** Initial task creation

### 2026-03-25T20:27:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
