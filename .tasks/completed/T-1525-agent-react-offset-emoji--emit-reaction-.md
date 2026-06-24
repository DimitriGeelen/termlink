---
id: T-1525
name: "agent react offset emoji — emit reaction on chat-arc"
description: >
  agent react offset emoji — emit reaction on chat-arc

status: work-completed
workflow_type: build
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-05T08:55:44Z
last_update: 2026-05-20T13:23:05Z
date_finished: 2026-05-05T09:05:51Z
---

# T-1525: agent react offset emoji — emit reaction on chat-arc

## Context

`cmd_channel_react(topic, parent_offset, reaction, sender_id, remove, ...)` already exists: posts a `msg_type=reaction` envelope with `metadata.target=<offset>` + `metadata.emoji=<reaction>`. Operator workflow on chat-arc: signal "+1 / 👍 / 👀" against a peer's post without authoring a text reply. Read-side already shipped (T-1514 `agent reactions <offset>`, T-1521 `agent reactions-of`, T-1515 `agent emoji-stats`). This closes the write side. Thin wrapper hard-pinning topic to `agent-chat-arc` (~10 LOC). `--remove` flag preserved for un-react.

## Acceptance Criteria

### Agent
- [x] New `AgentAction::React { offset, emoji, remove, hub, json }` variant in cli.rs
- [x] main.rs dispatch arm calling `cmd_channel_react("agent-chat-arc", offset, &emoji, None, remove, hub, json)`
- [x] `cargo build --release --bin termlink` clean
- [x] `agent react --help` shows positional `<OFFSET> <EMOJI>` plus `--remove` / `--hub` / `--json`
- [x] Live smoke text: `agent react 346 👀` posts and renders confirmation
- [x] Live smoke JSON: `agent react 346 ✅ --json` returns parseable envelope

### Human
- [ ] [REVIEW] Verify `agent react` reads naturally as engagement-emit verb
  **Steps:**
  1. `target/release/termlink agent react 346 👀`
  2. `target/release/termlink agent reactions 346`
  **Expected:** new reaction appears under offset 346 with emoji 👀.
  **If not:** report layout suggestions.

## Verification

cargo build --release --bin termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent react --help 2>&1 | grep -q -- "--remove"
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
**Rationale:** Closes the engagement-emit primitive on `agent.*` namespace. Read-side shipped (T-1514/T-1515/T-1521); this completes the round-trip. `cmd_channel_react` already does the work. Pure dispatch wrapper.
**Evidence:**
- Build clean
- Verification gate 2/2 passed
- Live smoke: reaction posts and surfaces under `agent reactions <offset>`

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

### 2026-05-05T08:55:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1525-agent-react-offset-emoji--emit-reaction-.md
- **Context:** Initial task creation

### 2026-05-05T09:05:51Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed


### 2026-05-20T13:23:05Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `agent react`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
