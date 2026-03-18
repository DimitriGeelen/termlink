---
id: T-164
name: "Enforce token auth on TCP hub connections"
description: >
  Require auth.token on all TCP connections before granting any RPC scope. Default TCP to zero scope. Make token_secret mandatory when TCP hub enabled.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [security, tcp]
components: []
related_tasks: []
created: 2026-03-18T10:08:25Z
last_update: 2026-03-18T11:19:22Z
date_finished: null
---

# T-164: Enforce token auth on TCP hub connections

## Context

TCP hub connections bypass all auth (T-163 research finding). Session server already has
token auth pattern (server.rs:24-108). Replicate for hub. See docs/reports/T-163-*.md.

## Acceptance Criteria

### Agent
- [x] Hub generates `token_secret` on startup, writes to `hub.secret` in runtime dir (600 perms)
- [x] TCP connections default to zero scope — only `hub.auth` method allowed
- [x] Unix connections keep current behavior (same-UID → full access)
- [x] `hub.auth` RPC validates token and upgrades connection scope
- [x] Hub-specific method scope mapping (discover=Observe, broadcast/register_remote=Interact, forward=per-method)
- [x] All existing hub tests pass
- [x] New tests: TCP connection rejected without auth, TCP connection works after auth
- [x] `hub start --tcp` prints token or path to hub.secret for client use

### Human
- [ ] [REVIEW] Verify TCP auth works end-to-end
  **Steps:**
  1. `termlink hub start --tcp 0.0.0.0:9100`
  2. From another terminal, try raw TCP: `echo '{"jsonrpc":"2.0","method":"session.discover","id":"1","params":{}}' | nc localhost 9100`
  3. Verify request is rejected (auth required)
  4. Use `termlink token create --hub` to get a token, then authenticate
  **Expected:** Unauthenticated TCP gets "Permission denied", authenticated TCP works
  **If not:** Note which step fails

## Verification

/Users/dimidev32/.cargo/bin/cargo test --package termlink-hub 2>&1 | grep -q "test result: ok"
grep -q "hub.auth" crates/termlink-hub/src/server.rs
grep -q "hub.secret" crates/termlink-hub/src/server.rs

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

### 2026-03-18T10:08:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-164-enforce-token-auth-on-tcp-hub-connection.md
- **Context:** Initial task creation

### 2026-03-18T11:19:22Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
