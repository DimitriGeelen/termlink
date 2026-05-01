---
id: T-1437
name: "T-1638 extract strip_ansi_codes to shared ansi module"
description: >
  T-1638 extract strip_ansi_codes to shared ansi module

status: work-completed
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: [crates/termlink-session/src/governance_subscriber.rs, crates/termlink-session/src/handler.rs, crates/termlink-session/src/lib.rs]
related_tasks: []
created: 2026-05-01T10:54:31Z
last_update: 2026-05-01T10:58:42Z
date_finished: 2026-05-01T10:58:42Z
---

# T-1437: T-1638 extract strip_ansi_codes to shared ansi module

## Context

Cross-repo dispatch from AEF (T-1638). `strip_ansi_codes` is duplicated byte-identically in
`crates/termlink-session/src/handler.rs` (line 374) and
`crates/termlink-session/src/governance_subscriber.rs` (line 121). Risk: the two copies drift,
so display-side ANSI stripping and governance-pattern matching diverge silently. Pure refactor:
extract to a shared `ansi` module inside the same crate.

## Acceptance Criteria

### Agent
- [x] `crates/termlink-session/src/ansi.rs` exists with `pub(crate) fn strip_ansi_codes(s: &str) -> String`.
- [x] `pub(crate) mod ansi;` is declared in `crates/termlink-session/src/lib.rs`.
- [x] The private `fn strip_ansi_codes` is removed from `handler.rs` and its call site uses `crate::ansi::strip_ansi_codes`.
- [x] The private `fn strip_ansi_codes` is removed from `governance_subscriber.rs` and its call site uses `crate::ansi::strip_ansi_codes`.
- [x] No new dependencies, no other crates touched.
- [x] `cargo check -p termlink-session` exits 0.
- [x] `cargo test -p termlink-session --lib` exits 0 with the same passing test count as the pre-refactor baseline (316).
- [x] Worker artefact `docs/reports/T-1638-strip-ansi-shared-module.md` records line refs of removed duplicates, byte-identity confirmation, test count delta, and exit codes.

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
test -f crates/termlink-session/src/ansi.rs
grep -q "pub(crate) mod ansi;" crates/termlink-session/src/lib.rs
! grep -q "^fn strip_ansi_codes" crates/termlink-session/src/handler.rs
! grep -q "^fn strip_ansi_codes" crates/termlink-session/src/governance_subscriber.rs
test -f docs/reports/T-1638-strip-ansi-shared-module.md
cargo check -p termlink-session

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

### 2026-05-01T10:54:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1437-t-1638-extract-stripansicodes-to-shared-.md
- **Context:** Initial task creation

### 2026-05-01T10:58:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
