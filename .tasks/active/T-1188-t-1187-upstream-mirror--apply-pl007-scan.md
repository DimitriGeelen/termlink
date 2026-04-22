---
id: T-1188
name: "T-1187 upstream mirror — apply pl007-scanner patch in framework repo"
description: >
  T-1187 built pl007-scanner.sh in the termlink-vendored copy at .agentic-framework/agents/context/pl007-scanner.sh. Since .agentic-framework is gitignored (vendored copy), the patch does not persist — next fw upgrade will overwrite. Mirror the patch into /opt/999-Agentic-Engineering-Framework/agents/context/pl007-scanner.sh and commit there. Use termlink dispatch to cross the project boundary per T-559 policy.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: [T-1187, T-1176]
created: 2026-04-22T11:18:03Z
last_update: 2026-04-22T11:26:31Z
date_finished: 2026-04-22T11:26:31Z
---

# T-1188: T-1187 upstream mirror — apply pl007-scanner patch in framework repo

## Context

T-1187 built `pl007-scanner.sh` at `.agentic-framework/agents/context/pl007-scanner.sh`
(the vendored framework copy). `.agentic-framework/` is gitignored in the termlink
repo, so the patch does not persist — next `fw upgrade` will restore whatever the
upstream framework ships, which currently is **nothing** (scanner was never upstream).

The same T-976 false-complete that motivated T-1187 also affects **T-977**
("fw hook-enable command"): G-015 audit found `bin/hook-enable.sh` does not exist
anywhere either. A future T-1189 may bundle the hook-enable fix with this mirror,
but this task tracks only the scanner.

Upstream path: `/opt/999-Agentic-Engineering-Framework/agents/context/pl007-scanner.sh`
Scanner sha256 (termlink-vendored, 2026-04-22): `40b2986fc96f21575a02a26ca759be0ddd379fb9be879f57409e97bef541a84f`

Cross-project write is **T-559 boundary-blocked** from /opt/termlink; any attempt
to `cd /opt/999-*` from this project is refused by the check-project-boundary hook.
The sanctioned path is a TermLink dispatch or a direct human action in the framework
session.

## Acceptance Criteria

### Agent
- [x] Pickup task self-contained — scanner source, sha256, verification plan all embedded
- [x] `.agentic-framework/agents/context/pl007-scanner.sh` exists in the termlink vendored
  copy and passes its 3 verification commands (T-1187 `## Verification` block)
- [x] G-015 registered in `.context/project/concerns.yaml` documenting the
  false-completion class this pickup remediates

### Human
- [ ] [RUBBER-STAMP] Apply the scanner to the framework repo and commit
  **Steps:**
  1. In a session rooted at `/opt/999-Agentic-Engineering-Framework` (NOT from /opt/termlink):
     ```
     cp /opt/termlink/.agentic-framework/agents/context/pl007-scanner.sh \
        /opt/999-Agentic-Engineering-Framework/agents/context/pl007-scanner.sh
     chmod +x /opt/999-Agentic-Engineering-Framework/agents/context/pl007-scanner.sh
     ```
  2. Verify sha256 match: `sha256sum /opt/termlink/.agentic-framework/agents/context/pl007-scanner.sh /opt/999-Agentic-Engineering-Framework/agents/context/pl007-scanner.sh` → both hashes identical
  3. Smoke-test upstream:
     ```
     python3 -c "import json; print(json.dumps({'tool_name':'Bash','tool_input':{'command':'echo hi'},'tool_response':{'stdout':'please run: fw ' + 'inception decide T-1 go'}}))" \
       | /opt/999-Agentic-Engineering-Framework/agents/context/pl007-scanner.sh \
       | grep -q 'PL-007 REMINDER'
     ```
  4. Commit in the framework repo:
     ```
     cd /opt/999-Agentic-Engineering-Framework
     git add agents/context/pl007-scanner.sh
     git commit -m "T-1176-class: Add pl007-scanner PostToolUse hook (mirrors termlink T-1187)"
     ```
  5. Push (follow framework's own push policy — OneDev-first if that's the framework policy too)
  **Expected:** Scanner lives in framework repo under version control; next `fw upgrade` for
  any consumer preserves the scanner instead of leaving it as a vendored-only phantom.
  **If not:** File the divergence as a new task in the framework repo.

## Verification

test -x /opt/termlink/.agentic-framework/agents/context/pl007-scanner.sh
grep -q 'PL-007 REMINDER' /opt/termlink/.agentic-framework/agents/context/pl007-scanner.sh
test -f /opt/termlink/.tasks/active/T-1188-t-1187-upstream-mirror--apply-pl007-scan.md

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

### 2026-04-22T11:18:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1188-t-1187-upstream-mirror--apply-pl007-scan.md
- **Context:** Initial task creation

### 2026-04-22T11:26:30Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-22T11:26:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
