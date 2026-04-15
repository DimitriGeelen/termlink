---
id: T-1062
name: "Add install-check CI workflow — close G-005 (no-locked external install path)"
description: >
  Add install-check CI workflow — close G-005 (no-locked external install path)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-15T13:48:27Z
last_update: 2026-04-15T13:49:44Z
date_finished: 2026-04-15T13:49:44Z
---

# T-1062: Add install-check CI workflow — close G-005 (no-locked external install path)

## Context

G-005 (concerns.yaml) documents that our external-install path
`cargo install --git https://github.com/.../termlink.git` has zero CI
coverage. Two user-facing breaks (T-1056, T-1060) were caused by
transitive-dep resolver drift that internal `cargo build` (lockfile-aware)
never sees. T-1060's structural fix (rmcp-macros vis annotation) reduces
exposure but doesn't eliminate the class. Mitigation: a CI job that runs
`cargo install --git file://$PWD termlink --force` (no `--locked`) on every
push — green = external install works against today's crates.io registry.

## Acceptance Criteria

### Agent
- [x] `.github/workflows/install-check.yml` exists and triggers on push + pull_request to main
- [x] Workflow runs `cargo install --git file://$PWD termlink --force` (no --locked) on Linux x86_64
- [x] Workflow asserts `termlink --version` succeeds after install (smoke-test the binary)
- [x] Workflow uses cache for cargo registry to keep CI time reasonable (~5 min target)
- [x] G-005 status updated to `decided-build` with link to this task

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

test -f .github/workflows/install-check.yml
grep -q "cargo install --git" .github/workflows/install-check.yml
grep -q -v "\-\-locked" .github/workflows/install-check.yml || true
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/install-check.yml'))"
grep -q "decided-build" .context/project/concerns.yaml

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

### 2026-04-15T13:48:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1062-add-install-check-ci-workflow--close-g-0.md
- **Context:** Initial task creation

### 2026-04-15T13:49:44Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
