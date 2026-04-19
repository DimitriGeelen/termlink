---
id: T-1140
name: "Add github-mirror freshness check to detect G-007 silent drift"
description: >
  Add github-mirror freshness check to detect G-007 silent drift

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [framework, ci, mirror-drift, G-007]
components: [scripts/check-mirror-freshness.sh]
related_tasks: []
created: 2026-04-19T15:59:11Z
last_update: 2026-04-19T16:02:08Z
date_finished: 2026-04-19T16:02:08Z
---

# T-1140: Add github-mirror freshness check to detect G-007 silent drift

## Context

G-007 mitigation candidate: "add a doctor check — if HEAD on origin != HEAD on github
for >15min, WARN." OneDev's PushRepository buildspec asynchronously mirrors to GitHub;
when it stalls, recent commits on OneDev never reach GitHub and Actions silently stop
firing. No structural detector exists today — operators only notice when a release or
install-check workflow fails to run. This task adds a project-local, standalone
checker that makes the drift observable on demand.

## Acceptance Criteria

### Agent
- [x] `scripts/check-mirror-freshness.sh` exists, executable, POSIX-portable (`sh -n` passes, `shellcheck` clean)
- [x] Script compares `git ls-remote origin HEAD` against `git ls-remote <github-url> HEAD` and classifies: `synced`, `drift` (GitHub ancestor of origin), or `diverged`
- [x] Supports `--json`, `--quiet` (cron-friendly), `--help`
- [x] Exit code 0 on synced, 1 on drift/diverged, 2 on tooling/network failure
- [x] Manual invocation against current repo reports `synced` (HEADs match post-push)
- [x] G-007 concern `mitigation_candidate` updated to reference this script

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

test -x /opt/termlink/scripts/check-mirror-freshness.sh
sh -n /opt/termlink/scripts/check-mirror-freshness.sh
shellcheck /opt/termlink/scripts/check-mirror-freshness.sh
/opt/termlink/scripts/check-mirror-freshness.sh --json | grep -q '"status":"\(synced\|drift\|diverged\)"'
grep -q "scripts/check-mirror-freshness.sh" /opt/termlink/.context/project/concerns.yaml

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

### 2026-04-19T15:59:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1140-add-github-mirror-freshness-check-to-det.md
- **Context:** Initial task creation

### 2026-04-19T16:02:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
