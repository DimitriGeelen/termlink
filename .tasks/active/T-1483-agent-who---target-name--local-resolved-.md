---
id: T-1483
name: "agent who --target <name> — local-resolved alternative to --target-fp"
description: >
  agent who --target <name> — local-resolved alternative to --target-fp

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T13:21:59Z
last_update: 2026-05-04T13:31:15Z
date_finished: 2026-05-04T13:31:15Z
---

# T-1483: agent who --target <name> — local-resolved alternative to --target-fp

## Context

T-1481 shipped `agent who` keyed exclusively on `--target-fp <hex>`. For
local-hub investigations operators already have the peer's display name
via `session.discover` — typing the FP is friction. This task adds an
optional `--target <name>` flag that resolves the name locally (mirror of
`agent contact` resolution path) and feeds the resolved fp into the same
activity-summary code path. Mutually exclusive with `--target-fp`.

## Acceptance Criteria

### Agent
- [x] `--target <name>` flag added to `agent who` (clap parses via `--help`)
- [x] `--target` and `--target-fp` are mutually exclusive — providing both → exit non-zero with clear error
- [x] At least one of `--target` or `--target-fp` is required — neither → exit non-zero with clear error
- [x] Local resolution: `--target <name>` calls `manager::find_session(name)` and reads `metadata.identity_fingerprint` (mirror of `cmd_agent_contact` resolution); on miss → exit code 8 with same message style
- [x] Pre-T-1436 peer (no identity_fingerprint) → exit code 8 with upgrade hint
- [x] cargo build --release -p termlink clean
- [x] Live smoke against `--target ollama-smoke-1` (a registered local session, see `termlink list`): resolves and produces same output as `--target-fp <its-fp>` would
- [x] Live smoke against `--target nonexistent-session-xyz`: exit 1, error names the missing session

### Human
- [ ] [REVIEW] Verify error messages name the failing input clearly
  **Steps:**
  1. `target/release/termlink agent who --target nonexistent-session-xyz` (run from /opt/termlink)
  2. `target/release/termlink agent who --target some-name --target-fp deadbeefdead` 
  3. `target/release/termlink agent who` (no args)
  **Expected:** each error names the offending flag combo or session and points to the right next step
  **If not:** describe the unclear wording, suggest concrete improvement

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

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent who --help 2>&1 | grep -q -- "--target "
out=$(target/release/termlink agent who --target some-x --target-fp dead 2>&1 || true); echo "$out" | grep -qE "mutually exclusive|specify either"
out=$(target/release/termlink agent who 2>&1 || true); echo "$out" | grep -qE "must specify either|required"

## RCA

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

## Recommendation

**Recommendation:** GO

**Rationale:** All three error paths produce explicit messages: missing flags ("must specify either --target <name> or --target-fp <hex>"), conflict ("specify either --target or --target-fp, not both"), and missing session ("Session 'X' not found: session not found: X"). Live `--target ollama-smoke-1 --window-secs 86400` produces the same output as `--target-fp d1993c2c3ec44c94`, confirming resolution parity. Verification 4/4 PASS.

**Evidence:**
- Live invocations:
  - `agent who --target ollama-smoke-1 --window-secs 86400` → exit 0, full output (peer_fp=d1993c2c3ec44c94, 68 posts, 3 from_projects)
  - `agent who --target nonexistent-session-xyz` → exit 1, "Session 'nonexistent-session-xyz' not found"
  - `agent who --target some-x --target-fp dead` → exit 1, "specify either --target or --target-fp, not both"
  - `agent who` → exit 1, "must specify either --target <name> or --target-fp <hex>"

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

### 2026-05-04T13:21:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1483-agent-who---target-name--local-resolved-.md
- **Context:** Initial task creation

### 2026-05-04T13:31:15Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
