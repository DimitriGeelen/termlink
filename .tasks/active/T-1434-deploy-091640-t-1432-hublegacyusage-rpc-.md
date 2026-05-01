---
id: T-1434
name: "Deploy 0.9.1640 (T-1432 hub.legacy_usage RPC) to laptop-141"
description: >
  Deploy 0.9.1640 (T-1432 hub.legacy_usage RPC) to laptop-141

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-01T08:54:10Z
last_update: 2026-05-01T08:54:10Z
date_finished: null
---

# T-1434: Deploy 0.9.1640 (T-1432 hub.legacy_usage RPC) to laptop-141

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] musl 0.9.1640 built locally with T-1432 baked in (hub.legacy_usage RPC + summarize_lines parser + 3 unit tests passing)
- [x] fleet-deploy-binary.sh streamed 453 chunks to .141 with --probe + --swap-restart; probe confirmed binary executes on the WSL host before swap
- [x] Hub on .141 came back UP within 90s (observed 10s in deploy log); fleet doctor confirms PASS post-swap
- [x] Live fleet doctor --legacy-usage from .107 successfully invokes hub.legacy_usage on .141 and receives a real summary (audit_present=true, total_legacy=0) — proves the RPC plumbing works cross-host
- [x] Cut-readiness verdict now shows .141 as CLEAN (7d) instead of UNSUPPORTED — same fleet doctor invocation tracks the upgrade in real time
- [x] No SSH used; all transport via termlink remote exec (PL-015)

## Verification

target/release/termlink fleet doctor --legacy-usage 2>&1 | grep -A 1 "T-1166 cut-readiness" | grep -q "Verdict:"
target/release/termlink fleet doctor --legacy-usage 2>&1 | grep -q "CLEAN.*laptop-141"
target/release/termlink remote exec laptop-141 tl-hmfi6wpa "/mnt/c/ntb-acd-plugin/termlink/target/release/termlink --version" 2>&1 | grep -q "0.9.1640"

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

### 2026-05-01T08:54:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1434-deploy-091640-t-1432-hublegacyusage-rpc-.md
- **Context:** Initial task creation
