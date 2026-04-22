---
id: T-1190
name: "T-1189 upstream mirror — apply fw hook-enable patch in framework repo"
description: >
  T-1189 built hook-enable.sh + bin/fw route in termlink-vendored copy at .agentic-framework/ (gitignored). Mirror the patch into /opt/999-Agentic-Engineering-Framework/ and commit there. Use cross-project human step per T-559 boundary policy (agent sessions rooted in /opt/termlink are blocked from cd'ing into the framework repo).

status: started-work
workflow_type: build
owner: human
horizon: now
tags: [framework, upstream-mirror, g-015-repair]
components: []
related_tasks: [T-977, T-1187, T-1188, T-1189]
created: 2026-04-22T18:35:47Z
last_update: 2026-04-22T18:35:47Z
date_finished: null
---

# T-1190: T-1189 upstream mirror — apply fw hook-enable patch in framework repo

## Context

T-1189 built two artifacts in the termlink-vendored copy (`.agentic-framework/` is
gitignored in this repo, so the patch does not persist through `fw upgrade` without
an upstream mirror):

1. `.agentic-framework/bin/hook-enable.sh` (new, 3968 bytes)
2. `.agentic-framework/bin/fw` (modified: added `hook-enable)` case + help line)

Agent sessions rooted in `/opt/termlink` are **T-559 boundary-blocked** from
operating on `/opt/999-Agentic-Engineering-Framework/`, so this mirror is Human
RUBBER-STAMP (same pattern as T-1188).

**Artifacts to mirror (sha256, termlink-vendored, 2026-04-22):**
- `bin/hook-enable.sh` → `91ba6bd5213d42de40904935d77cab6baa0c76255b045b74934ce656ccad1ebd`
- `bin/fw` → **diff only** (2 hunks: `hook-enable)` case + help line). Do NOT bulk-copy
  bin/fw over the framework's own copy — their `bin/fw` may have legitimate changes
  the termlink vendored copy lacks. Apply a 2-hunk patch instead.

## Acceptance Criteria

### Agent
- [x] Pickup self-contained — sha256, diff locations, apply plan all embedded here
- [x] T-1189 produced a working script locally (verification block passed 3/3)

### Human
- [ ] [RUBBER-STAMP] Copy hook-enable.sh and patch bin/fw in framework repo
  **Steps:**
  1. In a session rooted at `/opt/999-Agentic-Engineering-Framework` (NOT from /opt/termlink):
     ```
     cp /opt/termlink/.agentic-framework/bin/hook-enable.sh \
        /opt/999-Agentic-Engineering-Framework/bin/hook-enable.sh
     chmod +x /opt/999-Agentic-Engineering-Framework/bin/hook-enable.sh
     ```
  2. Apply the two bin/fw hunks manually (show them with: `diff /opt/999-Agentic-Engineering-Framework/bin/fw /opt/termlink/.agentic-framework/bin/fw | grep -A20 "hook-enable"`)
     - First hunk: new `hook-enable)` case after the existing `hook)` case dispatch
     - Second hunk: add help line `"  hook-enable <args>   Register a hook in .claude/settings.json (idempotent)"` after the existing `hook <name>` help line
  3. Verify sha256 of hook-enable.sh matches `91ba6bd5213d42de40904935d77cab6baa0c76255b045b74934ce656ccad1ebd`
  4. Smoke-test:
     ```
     cd /opt/999-Agentic-Engineering-Framework
     bin/fw hook-enable --help
     ```
  5. Commit in framework repo:
     ```
     git add bin/hook-enable.sh bin/fw
     git commit -m "T-977-repair: Add hook-enable registrar (mirrors termlink T-1189)"
     ```
  6. Push per framework's own push policy
  **Expected:** `fw hook-enable` works in framework repo; next `fw upgrade` for any consumer preserves the command.
  **If not:** File divergence as new task in framework repo.

## Verification

test -x /opt/termlink/.agentic-framework/bin/hook-enable.sh
grep -q "hook-enable" /opt/termlink/.agentic-framework/bin/fw
test -f /opt/termlink/.tasks/active/T-1190-t-1189-upstream-mirror--apply-fw-hook-en.md

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

### 2026-04-22T18:35:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1190-t-1189-upstream-mirror--apply-fw-hook-en.md
- **Context:** Initial task creation
