---
id: T-079
name: "Capability token system — HMAC-signed scoped tokens for multi-agent"
description: >
  Token generation, HMAC-SHA256 signing, token validation in dispatch. Enables fine-grained per-agent permissions. Depends on T-077 and T-078.

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-10T20:44:22Z
last_update: 2026-03-10T23:24:04Z
date_finished: null
---

# T-079: Capability token system — HMAC-signed scoped tokens for multi-agent

## Context

Phase 3 of TermLink's security model. Currently: UID-based auth (Phase 1, T-077) + 4-tier permission scoping (Phase 2, T-078/T-084). Same-UID connections get Execute scope (full access). For multi-agent scenarios, different agents running as the same user need different permission levels. Capability tokens provide fine-grained, per-agent authorization without requiring different UIDs.

Related gaps: G-001 (command injection), G-002 (no auth beyond UID check). Research artifact: `docs/reports/T-079-capability-tokens.md`

## Acceptance Criteria

### Agent
- [x] Research artifact created with problem analysis and design options
- [x] At least 2 design alternatives explored with trade-offs
- [x] GO/NO-GO recommendation with rationale

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     Examples:
       python3 -c "import yaml; yaml.safe_load(open('path/to/file.yaml'))"
       curl -sf http://localhost:3000/page
       grep -q "expected_string" output_file.txt
-->

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

### 2026-03-10T20:44:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-079-capability-token-system--hmac-signed-sco.md
- **Context:** Initial task creation

### 2026-03-10T23:24:03Z — status-update [task-update-agent]
- **Change:** horizon: later → now

### 2026-03-10T23:24:04Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
