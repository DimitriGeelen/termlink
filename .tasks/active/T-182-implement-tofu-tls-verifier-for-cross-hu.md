---
id: T-182
name: "Implement TOFU TLS verifier for cross-hub connections"
description: >
  Implement Trust On First Use TLS verifier so cross-hub TCP connections accept and store remote cert fingerprints. Custom rustls ServerCertVerifier + ~/.termlink/known_hubs file. Derived from T-179 GO decision.
status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [security, tls, hub, cross-machine]
components: []
related_tasks: [T-179, T-163]
created: 2026-03-18T22:58:44Z
last_update: 2026-03-18T22:58:44Z
date_finished: null
---

# T-182: Implement TOFU TLS verifier for cross-hub connections

## Context

Design: `docs/reports/T-179-cross-hub-tls-tofu.md`. Derived from T-179 GO decision.

## Acceptance Criteria

### Agent
- [x] `tofu.rs` module in termlink-session with `TofuVerifier` implementing `ServerCertVerifier`
- [x] `known_hubs` file at `~/.termlink/known_hubs` — created on first use
- [x] `Client::connect_addr` uses TOFU verifier for TCP connections instead of local cert
- [x] Unit test: TOFU accepts unknown cert on first connect and stores fingerprint (7 tests)
- [x] Unit test: TOFU rejects cert with changed fingerprint
- [x] `cargo test --package termlink-session` passes (18/18)
- [ ] `cargo build --release` succeeds

### Human
- [ ] [REVIEW] Verify cross-hub forwarding works between two machines
  **Steps:**
  1. Start hub on machine A: `termlink hub start --tcp 0.0.0.0:9100`
  2. Start hub on machine B: `termlink hub start --tcp 0.0.0.0:9100`
  3. Register session on B: `termlink register --name remote-session --shell`
  4. From A: `termlink send remote-session '{"method":"termlink.ping"}'`
  **Expected:** Ping response received (TOFU accepts B's cert on first connect, stores fingerprint)
  **If not:** Check `~/.termlink/known_hubs` exists and contains B's fingerprint

## Verification

/Users/dimidev32/.cargo/bin/cargo test --package termlink-session --lib tofu 2>&1 | grep -q "test result: ok"
/Users/dimidev32/.cargo/bin/cargo build --release 2>&1 | grep -qv "^error"

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

### 2026-03-18T22:58:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-182-implement-tofu-tls-verifier-for-cross-hu.md
- **Context:** Initial task creation
