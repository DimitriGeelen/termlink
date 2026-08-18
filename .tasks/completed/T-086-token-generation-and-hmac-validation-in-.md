---
id: T-086
name: "Token generation and HMAC validation in auth.rs"
description: >
  Add Token struct, HMAC-SHA256 sign/verify, generate_secret(), token_secret in Registration.
  Unit tests for creation, validation, expiry, tampering. From T-079 inception.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-10T23:27:00Z
last_update: '2026-08-18T18:58:41Z'
date_finished: 2026-03-11T07:40:38Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:41Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=2 (body:lightly-promoted); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:41Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-086: Token generation and HMAC validation in auth.rs

## Context

Phase 3 security: HMAC-SHA256 token generation and validation for capability-based auth. Design: `docs/reports/T-079-capability-tokens.md`

## Acceptance Criteria

### Agent
- [x] `TokenPayload` struct with scope, session_id, issued_at, expires_at, nonce
- [x] `generate_secret()` produces 32-byte random secrets
- [x] `create_token()` signs payload with HMAC-SHA256
- [x] `validate_token()` verifies signature, checks expiry, checks session ID
- [x] `token_secret` field added to Registration (optional, backward compatible)
- [x] 11 unit tests: create/validate, wrong secret, tampered payload, expired, format, session mismatch, all scopes

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink-session -- auth
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

### 2026-03-10T23:27:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-086-token-generation-and-hmac-validation-in-.md
- **Context:** Initial task creation

### 2026-03-11T07:40:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
