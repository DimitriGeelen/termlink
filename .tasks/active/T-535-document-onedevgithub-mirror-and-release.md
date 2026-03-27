---
id: T-535
name: "Document OneDev→GitHub mirror and release chain in CLAUDE.md"
description: >
  Document OneDev→GitHub mirror and release chain in CLAUDE.md

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-27T18:19:21Z
last_update: 2026-03-27T18:19:21Z
date_finished: null
---

# T-535: Document OneDev→GitHub mirror and release chain in CLAUDE.md

## Context

T-534 RCA found agent repeatedly suggests `git push github` because CLAUDE.md doesn't document the OneDev→GitHub auto-mirror chain. Fix: add CI/Release section to CLAUDE.md.

## Acceptance Criteria

### Agent
- [x] CLAUDE.md has a CI/Release section documenting the push→mirror→release chain
- [x] Section states OneDev is the only push target
- [x] Section explains auto-mirror via `.onedev-buildspec.yml`
- [x] Section explains GitHub Actions release workflow triggers automatically

## Verification

grep -q 'onedev-buildspec' CLAUDE.md
grep -q 'auto-mirror' CLAUDE.md
grep -q 'NEVER push to github' CLAUDE.md

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

### 2026-03-27T18:19:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-535-document-onedevgithub-mirror-and-release.md
- **Context:** Initial task creation
