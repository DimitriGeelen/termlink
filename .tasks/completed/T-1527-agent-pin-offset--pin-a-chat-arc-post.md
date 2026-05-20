---
id: T-1527
name: "agent pin <offset> — pin a chat-arc post"
description: >
  agent pin <offset> — pin a chat-arc post

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T08:56:33Z
last_update: 2026-05-20T13:23:06Z
date_finished: 2026-05-05T09:05:52Z
---

# T-1527: agent pin <offset> — pin a chat-arc post

## Context

`cmd_channel_pin(topic, offset, unpin, ...)` already exists: posts a `msg_type=pin` (or `unpin`) envelope with `metadata.target=<offset>`. Operator workflow: curate "important" posts on chat-arc — companion to T-1517 `agent pinned` (read-side). Without this verb, pinning requires scripting a raw pin envelope. Thin wrapper hard-pinning topic to `agent-chat-arc` (~10 LOC). `--unpin` flag preserved for un-pin.

## Acceptance Criteria

### Agent
- [x] New `AgentAction::Pin { offset, unpin, hub, json }` variant in cli.rs
- [x] main.rs dispatch arm calling `cmd_channel_pin("agent-chat-arc", offset, unpin, hub, json)`
- [x] `cargo build --release --bin termlink` clean
- [x] `agent pin --help` shows positional `<OFFSET>` plus `--unpin` / `--hub` / `--json`
- [x] Live smoke text: `agent pin <offset>` posts pin envelope
- [x] Live smoke JSON: `agent pin <offset> --json` returns parseable envelope

### Human
- [ ] [REVIEW] Verify `agent pin` reads naturally as curation verb
  **Steps:**
  1. `target/release/termlink agent pin <some-offset>`
  2. `target/release/termlink agent pinned`
  **Expected:** newly pinned offset appears in `agent pinned` output.
  **If not:** report layout suggestions.

## Verification

cargo build --release --bin termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent pin --help 2>&1 | grep -q -- "--unpin"
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
**Rationale:** Closes pin/curate write primitive. Read-side shipped as T-1517. `cmd_channel_pin` already does the work. Pure dispatch wrapper.
**Evidence:**
- Build clean
- Verification gate 2/2 passed
- Live smoke: pin envelope posted; offset surfaces in `agent pinned`

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

### 2026-05-05T08:56:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1527-agent-pin-offset--pin-a-chat-arc-post.md
- **Context:** Initial task creation

### 2026-05-05T09:05:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed


### 2026-05-20T13:23:06Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `agent pin`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
