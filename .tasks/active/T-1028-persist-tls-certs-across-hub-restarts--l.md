---
id: T-1028
name: "Persist TLS certs across hub restarts — load-or-generate pattern (T-945 GO)"
description: >
  Apply T-933 persist-if-present pattern to TLS certs. Load existing cert+key from disk if present, else generate. Remove cert deletion from cleanup(). Fixes TOFU breakage on every hub restart.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T13:26:45Z
last_update: 2026-04-13T13:28:47Z
date_finished: null
---

# T-1028: Persist TLS certs across hub restarts — load-or-generate pattern (T-945 GO)

## Context

T-945 inception GO. `tls::cleanup()` is called on shutdown (server.rs:196), deleting cert files despite T-985 adding load-or-generate in tls.rs. This causes every hub restart to regenerate certs, breaking all client TOFU trust. Observed: .121 deployment failed due to TOFU violation after hub restart.

## Acceptance Criteria

### Agent
- [x] `tls::cleanup()` call removed from normal shutdown path in server.rs
- [x] `tls::cleanup()` retained for explicit cleanup use (function not deleted)
- [x] Builds and passes clippy with no warnings
- [x] All existing TLS tests pass (9/9)

### Human
- [ ] [REVIEW] Test hub restart preserves TLS cert fingerprint
  **Steps:**
  1. `cd /opt/termlink && cargo run -- hub start --tcp 0.0.0.0:9100 &`
  2. Note the cert fingerprint: `openssl x509 -in /tmp/termlink-0/hub.cert.pem -fingerprint -noout`
  3. `cd /opt/termlink && cargo run -- hub restart`
  4. Check fingerprint again — should be identical
  5. `termlink ping` — should succeed without TOFU violation
  **Expected:** Same cert fingerprint before and after restart, no TOFU violation
  **If not:** Check if tls::cleanup() is still being called on shutdown

## Verification

cargo build -p termlink 2>&1 | grep -q "Finished"
cargo clippy -p termlink-hub -- -D warnings 2>&1 | grep -v "^warning:" | grep -q "Finished"
cargo test -p termlink-hub tls 2>&1 | grep -q "test result: ok"

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

### 2026-04-13T13:26:45Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1028-persist-tls-certs-across-hub-restarts--l.md
- **Context:** Initial task creation
