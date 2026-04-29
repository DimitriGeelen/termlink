---
id: T-1409
name: "Capture peer_addr (TCP) in rpc-audit — mirror T-1407 for TCP side"
description: >
  Capture peer_addr (TCP) in rpc-audit — mirror T-1407 for TCP side

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-hub/src/rpc_audit.rs, crates/termlink-hub/src/server.rs]
related_tasks: []
created: 2026-04-29T21:40:56Z
last_update: 2026-04-29T21:52:21Z
date_finished: 2026-04-29T21:52:21Z
---

# T-1409: Capture peer_addr (TCP) in rpc-audit — mirror T-1407 for TCP side

## Context

T-1407 added `peer_pid` to rpc-audit but only Unix-socket connections carry SO_PEERCRED — TCP+TLS callers still appear as `(none)/(unknown)`. The current bake-window probe identified `192.168.10.143` as the source of the mystery `inbox.status` poller via journal correlation, but the audit log itself doesn't preserve that fact. Mirror the T-1407 wiring: thread `peer_addr: Option<String>` from the TCP accept path through `handle_connection` → `record()` / `warn_if_legacy()` → `build_audit_line()`, omit when None. Surface in `fw metrics api-usage` next to the peer_pid breakdown.

## Acceptance Criteria

### Agent
- [x] `crates/termlink-hub/src/rpc_audit.rs::record()` and `warn_if_legacy()` accept a 4th param `peer_addr: Option<&str>`; `build_audit_line()` includes `"peer_addr":"X"` when Some, omits when None
- [x] `crates/termlink-hub/src/server.rs::handle_connection` accepts and forwards `peer_addr: Option<String>`; both TCP+TLS and TCP-no-TLS accept paths pass `Some(remote_addr.to_string())`; Unix accept path passes `None`
- [x] New unit tests in `rpc_audit.rs` covering: `peer_addr` only, `peer_addr + from`, `peer_addr + peer_pid`, all-three combined; `cargo test -p termlink-hub --lib rpc_audit` PASS (21/21)
- [x] `.agentic-framework/agents/metrics/api-usage.sh` parses `peer_addr` and prints `Legacy callers by addr (last Nd):` next to peer_pid breakdown; mirrored upstream to /opt/999-Agentic-Engineering-Framework

## Verification

cargo test -p termlink-hub --lib rpc_audit 2>&1 | grep -q "21 passed"
out=$(.agentic-framework/bin/fw metrics api-usage --last-Nd 1 --json 2>&1 || true); echo "$out" | python3 -c "import sys,json; d=json.loads(sys.stdin.read()); assert 'legacy_callers_by_addr' in d, 'JSON missing legacy_callers_by_addr field'"
grep -q '"peer_addr":"192.168.10.143' /var/lib/termlink/rpc-audit.jsonl

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

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.

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

### 2026-04-29T21:40:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1409-capture-peeraddr-tcp-in-rpc-audit--mirro.md
- **Context:** Initial task creation

### 2026-04-29T21:52:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
