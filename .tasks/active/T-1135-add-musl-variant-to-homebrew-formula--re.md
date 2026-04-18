---
id: T-1135
name: "Add musl variant to Homebrew formula — replace or supplement linux-gnu for LXC compatibility (from T-1070 GO)"
description: >
  From T-1070 inception GO. scripts/update-homebrew-sha.sh currently hashes 4 targets (darwin x86_64/aarch64 + linux-gnu x86_64/aarch64). Add musl variants (T-1019 artifacts) or swap linux-gnu → linux-musl for better LXC compatibility. Homebrew users on Linux LXC containers (no glibc) currently fall back to cargo build because the gnu binary fails to run. Touch: scripts/update-homebrew-sha.sh + homebrew-tap formula template. Verify: brew install termlink inside a fresh LXC succeeds.

status: captured
workflow_type: build
owner: agent
horizon: later
tags: [homebrew, install, T-1070, distribution]
components: []
related_tasks: []
created: 2026-04-18T23:02:30Z
last_update: 2026-04-18T23:02:30Z
date_finished: null
---

# T-1135: Add musl variant to Homebrew formula — replace or supplement linux-gnu for LXC compatibility (from T-1070 GO)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] [First criterion]
- [ ] [Second criterion]

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

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.

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

### 2026-04-18T23:02:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1135-add-musl-variant-to-homebrew-formula--re.md
- **Context:** Initial task creation
