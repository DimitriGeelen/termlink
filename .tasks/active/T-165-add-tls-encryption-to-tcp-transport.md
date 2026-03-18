---
id: T-165
name: "Add TLS encryption to TCP transport"
description: >
  Wrap TCP hub connections with TLS via rustls. Self-signed certs for LAN use. Prevents token sniffing and MITM.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [security, tcp, tls]
components: []
related_tasks: []
created: 2026-03-18T10:08:32Z
last_update: 2026-03-18T16:10:35Z
date_finished: null
---

# T-165: Add TLS encryption to TCP transport

## Context

TCP hub connections currently transmit auth tokens and all RPC traffic in cleartext. TLS wrapping prevents token sniffing and MITM on LAN. Uses self-signed certs auto-generated on hub startup.

## Acceptance Criteria

### Agent
- [x] `rcgen` + `tokio-rustls` + `rustls-pemfile` dependencies added to workspace and hub/session crates
- [x] TLS module in hub crate: generates self-signed cert+key on startup, writes PEM to runtime dir
- [x] Hub server wraps TCP accept with TLS acceptor before passing to `handle_connection`
- [x] Client `connect_addr` wraps TCP streams with TLS connector (trusting hub's self-signed cert)
- [x] Hub writes cert PEM path alongside hub.secret so clients can discover it
- [x] All existing hub tests pass (TCP tests use TLS transparently)
- [x] CLI `hub start --tcp` uses TLS by default

### Human
- [ ] [REVIEW] Start hub with `--tcp 0.0.0.0:9100`, verify TLS handshake works
  **Steps:**
  1. Run `termlink hub start --tcp 0.0.0.0:9100`
  2. Check that `hub.cert.pem` and `hub.key.pem` exist in runtime dir
  3. Try connecting with `openssl s_client -connect 127.0.0.1:9100` — should complete TLS handshake
  **Expected:** TLS handshake completes, cert subject visible
  **If not:** Check hub logs for TLS errors

## Verification

bash -c 'out=$(/Users/dimidev32/.cargo/bin/cargo test --package termlink-hub 2>&1); echo "$out" | grep -q "0 failed"'
grep -q "TlsAcceptor" crates/termlink-hub/src/tls.rs
grep -q "tokio-rustls" crates/termlink-hub/Cargo.toml

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

### 2026-03-18T10:08:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-165-add-tls-encryption-to-tcp-transport.md
- **Context:** Initial task creation

### 2026-03-18T16:10:35Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
