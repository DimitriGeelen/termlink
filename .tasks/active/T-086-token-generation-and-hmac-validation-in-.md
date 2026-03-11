---
id: T-086
name: "Token generation and HMAC validation in auth.rs"
description: >
  Add Token struct, HMAC-SHA256 sign/verify, generate_secret(), token_secret in Registration. Unit tests for creation, validation, expiry, tampering. From T-079 inception.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-10T23:27:00Z
last_update: 2026-03-10T23:27:00Z
date_finished: null
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

/Users/dimidev32/.cargo/bin/cargo test -p termlink-session -- auth 2>&1 | grep -q "0 failed"
/Users/dimidev32/.cargo/bin/cargo test --workspace 2>&1 | grep -q "0 failed"

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
