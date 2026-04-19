---
id: T-1141
name: "Populate homebrew formula SHA256s for v0.9.1 (placeholder → real)"
description: >
  Populate homebrew formula SHA256s for v0.9.1 (placeholder → real)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [homebrew, release, v0.9.1]
components: [homebrew/Formula/termlink.rb]
related_tasks: []
created: 2026-04-19T16:06:57Z
last_update: 2026-04-19T16:08:22Z
date_finished: 2026-04-19T16:08:22Z
---

# T-1141: Populate homebrew formula SHA256s for v0.9.1 (placeholder → real)

## Context

Homebrew formula at `homebrew/Formula/termlink.rb` still carries the initial
`PLACEHOLDER_SHA256_*` values — `scripts/update-homebrew-sha.sh` was never run
after v0.9.1 shipped, so `brew install termlink` would fail sha256 verification
for any consumer using the tap. T-1135 only *modified* the linux x86_64 url to
point at the musl static variant; it didn't populate real hashes.

## Acceptance Criteria

### Agent
- [x] `scripts/update-homebrew-sha.sh v0.9.1` runs cleanly and updates all four sha256 lines + the `version` line
- [x] No `PLACEHOLDER_SHA256_*` strings remain in the formula
- [x] All four formula sha256 values match v0.9.1's `checksums.txt` contents byte-for-byte
- [x] Formula `version` matches `0.9.1` (no leading `v`)

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

! grep -q "PLACEHOLDER_SHA256" /opt/termlink/homebrew/Formula/termlink.rb
grep -q 'version "0.9.1"' /opt/termlink/homebrew/Formula/termlink.rb
bash -c 'curl -sfL https://github.com/DimitriGeelen/termlink/releases/download/v0.9.1/checksums.txt | grep "termlink-darwin-aarch64$" | awk "{print \$1}" | xargs -I{} grep -q "{}" /opt/termlink/homebrew/Formula/termlink.rb'
bash -c 'curl -sfL https://github.com/DimitriGeelen/termlink/releases/download/v0.9.1/checksums.txt | grep "termlink-linux-x86_64-static$" | awk "{print \$1}" | xargs -I{} grep -q "{}" /opt/termlink/homebrew/Formula/termlink.rb'

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

### 2026-04-19T16:06:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1141-populate-homebrew-formula-sha256s-for-v0.md
- **Context:** Initial task creation

### 2026-04-19T16:08:22Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
