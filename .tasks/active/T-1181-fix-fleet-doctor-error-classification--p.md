---
id: T-1181
name: "Fix fleet-doctor error classification — preserve anyhow context chain so TOFU/auth causes aren't misclassified as 'Cannot connect'"
description: >
  Fix fleet-doctor error classification — preserve anyhow context chain so TOFU/auth causes aren't misclassified as 'Cannot connect'

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-22T05:26:15Z
last_update: 2026-04-22T05:41:36Z
date_finished: 2026-04-22T05:28:30Z
---

# T-1181: Fix fleet-doctor error classification — preserve anyhow context chain so TOFU/auth causes aren't misclassified as 'Cannot connect'

## Context

Discovered during T-1179 evidence work: `termlink fleet doctor` reported ring20-management (.102) as `Cannot connect (hub not running or IP drift)` while a follow-up `termlink remote ping ring20-management` revealed the actual cause was `TOFU VIOLATION: Hub 192.168.10.102:9100 fingerprint changed`. The hub was reachable; the fingerprint had rotated (T-1028 cert-rotation-on-restart). Fleet-doctor's `classify_fleet_error` already has a `TOFU VIOLATION || fingerprint changed` branch, but it never fires because at `remote.rs:1646` the incoming error is formatted via `format!("{}", e)` — which prints only the top-level `anyhow::Error` message. The TOFU string lives in the inner chain added by `connect_addr`'s underlying error; the top-level message is the `.context("Cannot connect to {} — is the hub running?")` wrapper added at `remote.rs:93`.

One-line structural fix: use `format!("{:#}", e)` (anyhow's alternate formatter) to walk the full chain, so `classify_fleet_error` sees `"Cannot connect to ... — is the hub running?: TOFU VIOLATION: ..."` and matches the correct branch.

Registered as G-014 in concerns.yaml (this session).

## Acceptance Criteria

### Agent
- [x] `remote.rs:1646` changes `format!("{}", e)` → `format!("{:#}", e)` in `cmd_fleet_doctor` error branch
- [x] `cargo build -p termlink` succeeds
- [x] Add unit test for `classify_fleet_error` with a wrapped-cause sample message to pin behaviour
- [x] Concerns register carries G-014 marking the detection path
- [x] PL-045 (or next ID) recorded: anyhow::Error → Display drops chain by default; use `{:#}` or walk `e.chain()` when string-matching inner causes

### Human
- [ ] [RUBBER-STAMP] Verify fleet-doctor output against a real TOFU-rotated hub
  **Steps:**
  1. `cd /opt/termlink && cargo build -p termlink --release`
  2. `./target/release/termlink fleet doctor`
  3. For any hub whose cert has rotated, confirm the FAIL line's `hint:` says "Hub certificate changed. If expected..." rather than "Unexpected error — check hub logs"
  **Expected:** TOFU cases get the actionable `termlink tofu clear <addr>` hint instead of the generic fallback
  **If not:** Check that the build included the `remote.rs:1646` change and that the hub error actually carries TOFU VIOLATION in its chain

## Verification

grep -q 'format!("{:#}", e)' crates/termlink-cli/src/commands/remote.rs
cargo build -p termlink 2>&1 | tail -5 | grep -qE "Finished|Compiling"

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

### 2026-04-22T05:26:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1181-fix-fleet-doctor-error-classification--p.md
- **Context:** Initial task creation

### 2026-04-22T05:28:30Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-22T07:41Z — agent-evidence (rubber-stamp support, does NOT check Human AC per T-193)

Release build + live fleet-doctor run completed against a real TOFU-rotated hub (.102):

**Build:**
```
$ cargo build -p termlink --release
    Finished `release` profile [optimized] target(s) in 2m 21s
$ ./target/release/termlink --version
termlink 0.9.274
```

**Before fix (installed binary 0.9.206, same hubs.toml, same TOFU pin state):**
```
--- ring20-management (192.168.10.102:9100) ---
  [FAIL] Cannot connect to 192.168.10.102:9100 — is the hub running?
  hint: Unexpected error — check hub logs on the remote host for details
```

**After fix (rebuilt 0.9.274 from this task's HEAD):**
```
--- ring20-management (192.168.10.102:9100) ---
  [FAIL] Cannot connect to 192.168.10.102:9100 — is the hub running?: unexpected error: TOFU VIOLATION: Hub 192.168.10.102:9100 fingerprint changed!
  ...
  hint: Hub certificate changed. If expected (hub restart), clear with: termlink tofu clear 192.168.10.102:9100
```

The hint changed from the generic fallback to the actionable TOFU branch — exactly what the task targeted. The `{:#}` alternate formatter now surfaces the inner chain so `classify_fleet_error` matches the correct branch. The RUBBER-STAMP AC is satisfied in substance; only the checkbox remains for the human.
