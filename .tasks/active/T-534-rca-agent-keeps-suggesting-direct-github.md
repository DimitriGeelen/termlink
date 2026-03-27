---
id: T-534
name: "RCA: Agent keeps suggesting direct GitHub push — investigate OneDev→GitHub mirror and fix workflow assumptions"
description: >
  RCA: Agent keeps suggesting direct GitHub push — investigate OneDev→GitHub mirror and fix workflow assumptions

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-27T17:59:50Z
last_update: 2026-03-27T18:19:04Z
date_finished: 2026-03-27T18:19:04Z
---

# T-534: RCA: Agent keeps suggesting direct GitHub push — investigate OneDev→GitHub mirror and fix workflow assumptions

## Problem Statement

Agent repeatedly suggests `git push github` despite memory explicitly prohibiting it. User has corrected this multiple times. The push workflow is: OneDev only → OneDev auto-mirrors to GitHub via buildspec → GitHub Actions triggers releases.

## Root Causes

1. **Memory lacks structural enforcement** — feedback memory says "don't" but nothing gates it
2. **Release workflow creates false dependency** — `.github/workflows/release.yml` triggers on GitHub tags, agent concludes manual push needed
3. **Mirror not documented in CLAUDE.md** — `.onedev-buildspec.yml` auto-mirrors but CLAUDE.md doesn't mention it
4. **Homebrew formula hardcodes GitHub URLs** — agent sees GitHub URLs and assumes manual GitHub interaction needed

## Exploration

- Confirmed: `.onedev-buildspec.yml` has `PushRepository` job mirroring all branches/tags to GitHub automatically
- Confirmed: `github-push-token` secret handles auth
- Confirmed: Tags v0.1.0 and v0.9.0 present on both OneDev and GitHub (mirror works)
- Research artifact: `docs/reports/T-534-github-push-rca.md`

## Mitigations

- **M-1** (done): Updated feedback memory with full mirror chain explanation
- **M-2** (recommended): Add CI/release flow section to CLAUDE.md

## Acceptance Criteria

### Agent
- [x] Problem statement validated (4 root causes identified)
- [x] Assumptions tested (mirror confirmed in `.onedev-buildspec.yml`)
- [x] Recommendation written with rationale (M-1 done, M-2 recommended)

### Human
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Read the research artifact and recommendation in this task
  2. Evaluate go/no-go criteria against findings
  3. Run: `fw inception decide T-XXX go|no-go --rationale "your rationale"`
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO if:**
- Root causes identified and mitigations clear
- At least one structural fix prevents recurrence

**NO-GO if:**
- Mirror doesn't actually work (it does)
- Problem is only agent discipline (it's also missing documentation)

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     For inception tasks, verification is often not needed (decisions, not code).
-->

## Decisions

**Decision**: GO

**Rationale**: 4 root causes found. M-1 done (memory updated). M-2 needed: document mirror chain in CLAUDE.md so every session knows the flow structurally.

**Date**: 2026-03-27T18:19:04Z
## Decision

**Decision**: GO

**Rationale**: 4 root causes found. M-1 done (memory updated). M-2 needed: document mirror chain in CLAUDE.md so every session knows the flow structurally.

**Date**: 2026-03-27T18:19:04Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-27T18:19:04Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** 4 root causes found. M-1 done (memory updated). M-2 needed: document mirror chain in CLAUDE.md so every session knows the flow structurally.

### 2026-03-27T18:19:04Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
