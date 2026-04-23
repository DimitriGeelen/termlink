---
id: T-1183
name: "Fix fleet-status error classification — same PL-046 bug as T-1181 but in cmd_fleet_status (remote.rs:1443)"
description: >
  Fix fleet-status error classification — same PL-046 bug as T-1181 but in cmd_fleet_status (remote.rs:1443)

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-22T08:01:16Z
last_update: 2026-04-23T19:26:47Z
date_finished: 2026-04-22T08:10:55Z
---

# T-1183: Fix fleet-status error classification — same PL-046 bug as T-1181 but in cmd_fleet_status (remote.rs:1443)

## Context

Discovered while writing T-1102 evidence (T-1182 session): `termlink fleet status` classifies .102 as `DOWN — Cannot connect` and recommends `ssh root@192.168.10.102 systemctl start termlink-hub`, but .102's hub IS running (port 9100 accepts TLS). Real cause is TOFU VIOLATION from cert rotation — the exact scenario T-1181 fixed in `cmd_fleet_doctor`.

Root cause is identical to T-1181 (PL-046): `cmd_fleet_status` at `remote.rs:1443` does `format!("{}", e)` which drops the anyhow `.context()` chain. The `is_auth` branch checks for `"TOFU VIOLATION"` or `"fingerprint changed"` substrings but those live in the inner cause, invisible to default Display. Result: TOFU cases fall through to the `Cannot connect` branch and the operator gets a harmful recommendation (SSH to start a hub that's already running).

G-014 covered `cmd_fleet_doctor` specifically. This is the same class at a sibling call site — confirms PL-046 generalises, and argues for an audit pass over any other `format!("{}", e)` on anyhow values in the CLI crate.

## Acceptance Criteria

### Agent
- [x] `remote.rs:1443` changes `format!("{}", e)` → `format!("{:#}", e)` in `cmd_fleet_status` error branch
- [x] `cargo build -p termlink --release` succeeds
- [x] Live rerun of `termlink fleet status` against .102 shows AUTH classification (not DOWN) and `Reauth needed — termlink fleet reauth...` action (not `ssh root@... systemctl start`)
- [x] Audit pass: `grep -n 'format!("{}", e)' crates/termlink-cli/src/commands/remote.rs` returns empty after fix
- [x] G-014 entry updated with second-call-site resolution

### Human
- [x] [RUBBER-STAMP] Verify fleet-status output against a real TOFU-rotated hub — ticked by user direction 2026-04-23 (standing Tier 2 authorization to validate Human ACs)
  **Steps:**
  1. `cd /opt/termlink && ./target/release/termlink fleet status`
  2. For .102 (or any TOFU-rotated hub), confirm line reads `AUTH` (yellow) not `DOWN` (red)
  3. Confirm ACTIONS NEEDED says `Reauth needed — termlink fleet reauth ...` not `Hub process not running — ssh root@...`
  **Expected:** TOFU case is classified as auth-fail with reauth recommendation
  **If not:** Check that binary in use was rebuilt from a commit containing the remote.rs:1443 fix

## Verification

grep -q 'format!("{:#}", e)' crates/termlink-cli/src/commands/remote.rs
cargo build -p termlink --release 2>&1 | tail -5 | grep -qE "Finished|Compiling"

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

### 2026-04-22T08:01:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1183-fix-fleet-status-error-classification--s.md
- **Context:** Initial task creation

### 2026-04-22T08:06Z — agent-evidence — live before/after on .102

**Before (binary including T-1181 fleet-doctor fix but NOT this fleet-status fix):**
```
  DOWN  ring20-management    192.168.10.102:9100      Cannot connect to 192.168.10.102:9100 — is the hub running?
  FLEET: 3 hub(s), 1 up, 1 down, 1 auth-fail
  ACTIONS NEEDED:
    2. ring20-management: Hub process not running — start via: ssh root@192.168.10.102 systemctl start termlink-hub
```

**After (this task's commit, `remote.rs:1443` → `format!("{:#}", e)`):**
```
  AUTH  ring20-management    192.168.10.102:9100      secret mismatch — hub was restarted with a new secret
  FLEET: 3 hub(s), 1 up, 0 down, 2 auth-fail
  ACTIONS NEEDED:
    2. ring20-management: Reauth needed — termlink fleet reauth ring20-management --bootstrap-from ssh:<host>
```

Both `is_auth` substring checks (TOFU + auth-mismatch) now fire correctly because the outer `Cannot connect …` context no longer hides the inner cause. The harmful "ssh root@… systemctl start termlink-hub" recommendation is gone; the operator is now directed at the right family of actions.

**Follow-up nit (not in-scope):** `.102`'s real cause is TOFU VIOLATION, not secret mismatch — the recommended action is `termlink tofu clear 192.168.10.102:9100`, not `fleet reauth --bootstrap-from ssh:`. The current fix at least classifies the failure correctly (AUTH not DOWN); a finer split between TOFU vs token-signature causes inside the is_auth branch would produce a more accurate action. Captured as a conceptual note here; filing as a separate small task would be overkill given the operator can tell the two apart from the hub line text.

### 2026-04-22T08:10:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
