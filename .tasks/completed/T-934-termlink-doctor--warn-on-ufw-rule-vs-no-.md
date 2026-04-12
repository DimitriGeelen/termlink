---
id: T-934
name: "termlink doctor — warn on UFW-rule-vs-no-listener mismatch"
description: >
  Cheap doctor check that catches the exact state T-930 started from: UFW has an ALLOW rule for hub port 9100/tcp, but nothing is listening on 9100. Implementation: read ufw status output (sudo-free if possible), grep for 9100/tcp rule, then ss -tln to check listener. If rule present but no listener, emit a warn-level doctor check. Belt-and-braces for the systemd unit approach — catches manual kills, crashes-before-restart, and botched unit edits. From T-930 decomposition.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/infrastructure.rs]
related_tasks: [T-930]
created: 2026-04-11T22:29:30Z
last_update: 2026-04-11T23:02:05Z
date_finished: 2026-04-11T23:02:05Z
---

# T-934: termlink doctor — warn on UFW-rule-vs-no-listener mismatch

## Context

T-930 started from exactly the state this check is meant to detect: UFW
had `9100/tcp ALLOW 192.168.10.0/24 # TermLink TCP Hub (LAN only)` in
place, but the hub was bound to its unix socket only. Cross-host callers
got connection-refused with no actionable signal. A cheap doctor check
that parses `ufw status` for rules whose comment mentions "termlink",
extracts the port, and confirms `ss -tln` shows a listener, closes the
feedback gap.

## Acceptance Criteria

### Agent
- [x] `termlink doctor` parses `ufw status` output for rules containing "termlink" (case-insensitive) and extracts the associated TCP port(s).
- [x] For each such port, the check runs `ss -tln` and verifies a listener is present on that port.
- [x] Emits a `pass` check (`ufw_listener`) when all identified ports have listeners.
- [x] Emits a `warn` check with actionable text when a rule exists but no listener is bound. (Code inspected — symmetric to pass path.)
- [x] Check is best-effort: `ufw` unavailable or permission-denied silently skips the check.
- [x] `cargo build --workspace` clean.
- [x] `cargo test -p termlink --bins` 165/165 pass.
- [x] Live test on .107: `TERMLINK_RUNTIME_DIR=/var/lib/termlink termlink doctor` shows `✓ ufw_listener: ufw allows 9100/tcp — listener present`.

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

### 2026-04-11T22:29:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-934-termlink-doctor--warn-on-ufw-rule-vs-no-.md
- **Context:** Initial task creation

### 2026-04-11T22:57:28Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-11T23:02:05Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
