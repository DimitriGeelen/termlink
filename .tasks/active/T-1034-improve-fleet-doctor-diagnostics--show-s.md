---
id: T-1034
name: "Improve fleet-doctor diagnostics — show secret file path and suggest fix for auth failures"
description: >
  Improve fleet-doctor diagnostics — show secret file path and suggest fix for auth failures

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/remote.rs]
related_tasks: []
created: 2026-04-13T18:31:29Z
last_update: 2026-04-15T13:47:09Z
date_finished: 2026-04-13T18:36:51Z
---

# T-1034: Improve fleet-doctor diagnostics — show secret file path and suggest fix for auth failures

## Context

Fleet doctor shows cryptic errors like "Authentication failed: -32010 Token validation failed: invalid signature" without actionable diagnostic info. After T-1027 deployment attempts, both .109 and .121 fail with auth errors but no hint about which secret file is being used or how to fix it.

## Acceptance Criteria

### Agent
- [x] Fleet doctor shows secret_file path for each hub in non-JSON output
- [x] Auth failure errors include a diagnostic hint ("secret may be stale — fetch current secret from remote hub")
- [x] TOFU violation errors include hint to clear known_hubs entry
- [x] Connection timeout errors include network diagnostic hint
- [x] JSON output includes secret_file and diagnostic fields
- [x] Builds with zero clippy warnings
- [x] Existing fleet-doctor integration test still passes (no fleet test exists — network-dependent)

### Human
- [ ] [REVIEW] Run `termlink fleet doctor` and verify diagnostic hints appear for failing hubs
  **Steps:** `cd /opt/termlink && cargo run -- fleet doctor`
  **Expected:** Failing hubs show what secret file was used and suggest fixes
  **If not:** Check cmd_fleet_doctor output formatting


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, fleet-doctor-diagnostics):** `termlink fleet doctor` against ring20-dashboard prints: `[FAIL] Authentication failed: -32010 Token validation failed: invalid signature` followed by `secret: /root/.termlink/secrets/ring20-dashboard.hex` and the hint `Secret mismatch — hub was likely restarted with a new secret.` Secret path surfaced inline, hint present. RUBBER-STAMPable (or REVIEW-approvable).

## Verification

cargo build -p termlink 2>&1 | grep -q "Finished"
cargo clippy -p termlink -- -D warnings 2>&1 | grep -v "^warning:" | grep -q "Finished"

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

### 2026-04-13T18:31:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1034-improve-fleet-doctor-diagnostics--show-s.md
- **Context:** Initial task creation

### 2026-04-13T18:36:51Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-16T19:00:39Z — programmatic-evidence [T-1087]
- **Evidence:** termlink fleet doctor shows secret file path and diagnostic hints for failed hubs
- **Verified by:** automated command execution


### 2026-04-16T23:24:50Z — e2e-evidence [T-1097]
- **Evidence:** fleet doctor shows secret_source path + diagnostic hints for auth failures ('Secret mismatch — hub was likely restarted with a new secret') on .121
- **Verified by:** termlink fleet doctor (3 hubs, 1 pass, 2 fail with diagnostics)
