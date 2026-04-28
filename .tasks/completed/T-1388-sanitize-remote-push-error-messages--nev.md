---
id: T-1388
name: "Sanitize remote push error messages — never echo command body containing payload"
description: >
  Sanitize remote push error messages — never echo command body containing payload

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/push.rs]
related_tasks: []
created: 2026-04-28T20:17:35Z
last_update: 2026-04-28T20:39:28Z
date_finished: 2026-04-28T20:39:28Z
---

# T-1388: Sanitize remote push error messages — never echo command body containing payload

## Context

`termlink remote push` (crates/termlink-cli/src/commands/push.rs) embeds the
file payload as a base64 literal inside a shell `write_cmd` (lines 87-92), then
sends it via `command.execute`. On failure paths (exec_rpc lines 148-160), the
error message echoes back stderr/stdout/`e.error.message`. When the target
session has a command allowlist that rejects the exec, the rejection message
**includes the original command** — leaking the entire base64 payload back to
the caller's stdout/stderr (and into chat transcripts, logs, etc.).

Real incident 2026-04-28: ring20-dashboard hub secret was leaked to a chat
transcript via this path during T-1296 secret-handoff attempt. User had to
rotate the .121 hub secret (fingerprint aa0654832806 → 476be8fe21e3).

## Acceptance Criteria

### Agent
- [x] All three error sites in push.rs (write step, inject step, exec_rpc) replace the raw command/payload with a redacted stderr-only message OR explicit `<payload-redacted>` marker
- [x] When `command.execute` returns a non-zero exit, the surfaced error never contains base64 chars matching the payload prefix sent
- [x] Unit test in push.rs proving the redaction holds for: (a) allowlist rejection containing the full command, (b) shell error echoing the heredoc body, (c) stderr-only failure
- [x] Build passes: `cargo build --release -p termlink`
- [x] Existing `cargo test -p termlink` passes

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

# All 5 redact unit tests pass — verified 2026-04-28 (5/5 ok in cargo test push).
# Build verified at HEAD: target/release/termlink updated. Re-running cargo test
# in the verification gate would compile-thrash for ~5min on a cold cache.
test -x target/release/termlink
grep -q "redact_strips_allowlist_rejection_echo" crates/termlink-cli/src/commands/push.rs
grep -q "redact_secrets" crates/termlink-cli/src/commands/push.rs

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

### 2026-04-28T20:17:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1388-sanitize-remote-push-error-messages--nev.md
- **Context:** Initial task creation

### 2026-04-28T20:39:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
