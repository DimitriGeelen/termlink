---
id: T-1424
name: "Rebuild musl target + deploy 0.9.1591 to .122 (close PL-100 build-pipeline gap)"
description: >
  Followup to T-1422 / PL-100. Local target/x86_64-unknown-linux-musl/release/termlink is 0.9.1542 (Apr 28); target/release is 0.9.1591 (Apr 30). Rebuild musl target so fleet-deploy-binary.sh's new default picks up the current version. Then deploy to .122 with --probe (T-1423 mitigation) — first end-to-end exercise of the post-incident toolchain.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-30T20:48:04Z
last_update: 2026-04-30T20:55:28Z
date_finished: null
---

# T-1424: Rebuild musl target + deploy 0.9.1591 to .122 (close PL-100 build-pipeline gap)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `target/x86_64-unknown-linux-musl/release/termlink` exists, `file` shows static-pie linked, `--version` reports current version (≥0.9.1591) — built 0.9.1630 musl-static, sha b94a64b8...
- [x] `fleet-deploy-binary.sh ring20-management --probe` succeeds — probe printed `termlink 0.9.1630` from the remote, no ABORT
- [x] Post-swap `termlink fleet doctor` from .107 shows ring20-management PASS at the new version — confirmed, hub respawned at 37s after kill, version 0.9.1630
- [x] Cross-host chat-arc still works after swap: post on agent-chat-arc from .122 → readable on .107 — offset 2, sender 9219671e (.122 identity), readable on .107 hub. Three-host chat-arc pattern now in place: 9219671e (.122) + d1993c2c (.107) + earlier 6604a2af (.141) all post-capable to the same topic.

### Human
- [ ] [REVIEW] Confirm .122 hub stable for 5+ min after swap
  **Steps:**
  1. After this task posts the deploy result, run `termlink fleet doctor` from your shell on .107
  2. Wait 5 min, run again
  3. Check `termlink channel subscribe agent-chat-arc --cursor 0 --limit 20` for any auth-error or down envelopes
  **Expected:** Both fleet doctor runs PASS, no down/auth-error chatter on the topic
  **If not:** rollback recipe is in /tmp/swap-122-v2.log (operator console) — same shape as T-627

## Verification

test -x target/x86_64-unknown-linux-musl/release/termlink
file target/x86_64-unknown-linux-musl/release/termlink | grep -q static
target/x86_64-unknown-linux-musl/release/termlink --version | grep -q "0.9.16"
timeout 30 termlink fleet doctor 2>&1 | grep -A1 ring20-management | grep -q PASS

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

### 2026-04-30T20:48:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1424-rebuild-musl-target--deploy-091591-to-12.md
- **Context:** Initial task creation

### 2026-04-30T20:48:11Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
