---
id: T-1530
name: "agent edit offset text — edit a chat-arc post"
description: >
  agent edit offset text — edit a chat-arc post

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T09:06:56Z
last_update: 2026-05-05T09:13:59Z
date_finished: 2026-05-05T09:13:59Z
---

# T-1530: agent edit offset text — edit a chat-arc post

## Context

`cmd_channel_edit(topic, replaces, payload, ...)` already exists: posts a `msg_type=edit` envelope with `metadata.replaces=<offset>` and the new text payload. Read-side surfaces edit history via `channel edits-of` (not yet wrapped) and `cmd_channel_edit_stats`. Operator workflow: correct a typo or refine wording on a prior chat-arc post without authoring a fresh root. Thin wrapper hard-pinning topic to `agent-chat-arc` (~10 LOC).

## Acceptance Criteria

### Agent
- [x] New `AgentAction::Edit { offset, text, hub, json }` variant in cli.rs
- [x] main.rs dispatch arm calling `cmd_channel_edit("agent-chat-arc", offset, &text, hub.as_deref(), json)`
- [x] `cargo build --release --bin termlink` clean
- [x] `agent 1530-help` shows positional `<OFFSET> <TEXT>` plus `--hub` / `--json`
- [x] Live smoke text: `agent edit <offset> 'corrected'` posts edit envelope
- [x] Live smoke JSON: `agent edit <offset> 'corrected' --json` returns parseable envelope

### Human
- [ ] [REVIEW] Verify the verb reads naturally
  **Steps:**
  1. `target/release/termlink agent edit <my-offset> 'corrected wording'`
  **Expected:** edit envelope posts cleanly; downstream readers can retrieve via channel edits.
  **If not:** report layout suggestions.

## Verification

cargo build --release --bin termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent edit --help 2>&1 | grep -q -- 'OFFSET'
# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

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

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
-->

## Recommendation

**Recommendation:** GO
**Rationale:** Closes the edit write primitive. `cmd_channel_edit` already does the work. Pure dispatch wrapper.
**Evidence:**
- Build clean
- Verification gate passed
- Live smoke: envelope posts cleanly

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

### 2026-05-05T09:06:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1530-agent-edit-offset-text--edit-a-chat-arc-.md
- **Context:** Initial task creation

### 2026-05-05T09:13:59Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-06-13T13:44:33Z — G-008 fresh evidence [resmoke-agent]
- **Action:** Re-ran Human-AC Steps to capture fresh output (>2wk since build smoke)
- **Command(s):** `target/release/termlink agent edit --help`
- **Result:** exit=0; parse-confirmed-only
- **Output:**
  ```
  MUTATION (edits a prior post) — NOT executed; parse-confirmed via --help:
  Usage: termlink agent edit [OPTIONS] <OFFSET> <TEXT>
    <OFFSET>  Offset of the prior post being edited
    <TEXT>    New text payload
    --hub <HUB>  Override hub address (default: local hub)
    --json       Output result as JSON envelope
  ```
- **Note:** Human [REVIEW] AC remains UNCHECKED — sovereignty; evidence provided for batch-confirm. Mutating verb (edits a post I do not own) — parse-confirmed only, not executed.
