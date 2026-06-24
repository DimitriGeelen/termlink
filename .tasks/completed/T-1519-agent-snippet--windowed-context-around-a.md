---
id: T-1519
name: "agent snippet — windowed context around a chat-arc offset"
description: >
  agent snippet — windowed context around a chat-arc offset

status: work-completed
workflow_type: build
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-05T08:09:44Z
last_update: 2026-05-20T13:23:01Z
date_finished: 2026-05-05T08:15:00Z
---

# T-1519: agent snippet — windowed context around a chat-arc offset

## Context

`cmd_channel_snippet(topic, offset, lines, header, ...)` already exists: walks the arc, locates the target offset, picks up to N content envelopes on each side (filtered to post/chat/note types — meta types like reaction/edit/redaction excluded), and renders a fenced markdown block with `>>` marking the target. Operator-relevant gap: `agent quote <offset>` shows just the parent + immediate replies; `agent thread <root>` shows a whole subtree but only along the in_reply_to dimension; `agent snippet <offset>` shows nearby chronological context regardless of threading. Useful for "what was being discussed around when this post landed". Thin wrapper hard-pinning topic to `agent-chat-arc` (~12 LOC).

## Acceptance Criteria

### Agent
- [x] New `AgentAction::Snippet { offset, lines, header, hub, json }` variant in cli.rs
- [x] main.rs dispatch arm calling `cmd_channel_snippet("agent-chat-arc", offset, lines, header, hub, json)`
- [x] `cargo build --release --bin termlink` clean
- [x] `agent snippet --help` shows `<OFFSET>` positional + `--lines` / `--header` / `--hub` / `--json`
- [x] Live smoke text: `agent snippet 318 --lines 3` renders `>>` marker on target + 3-line context window
- [x] Live smoke JSON: `agent snippet 318 --lines 3 --json` returns parseable envelope with `target_offset` + `lines`

### Human
- [ ] [REVIEW] Verify `agent snippet` reads naturally as context-window
  **Steps:**
  1. `target/release/termlink agent snippet 318 --lines 3`
  2. `target/release/termlink agent snippet 318 --lines 5 --header`
  **Expected:** fenced code block, target offset prefixed with `>>`, surrounding lines prefixed with two spaces.
  **If not:** report layout suggestions.

## Verification

cargo build --release --bin termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent snippet --help 2>&1 | grep -q -- "--lines"
target/release/termlink agent snippet --help 2>&1 | grep -qi "OFFSET"
target/release/termlink agent snippet 318 --lines 3 --json 2>&1 | python3 -c "import json,sys; d=json.load(sys.stdin); assert 'target_offset' in d and 'lines' in d; print('OK')" | grep -q OK
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
**Rationale:** Closes the chronological-context-window primitive on `agent.*` namespace. Distinct from `agent quote` (parent + immediate replies via in_reply_to) and `agent thread` (full subtree along in_reply_to): snippet shows what was being discussed *around* the target in time, regardless of threading. `cmd_channel_snippet` already does the work. Pure dispatch wrapper.
**Evidence:**
- Build clean
- Verification gate 4/4 passed
- Live smoke text: rendered fenced block with `>>` target marker
- Live smoke JSON: parseable envelope with target_offset + lines

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

### 2026-05-05T08:09:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1519-agent-snippet--windowed-context-around-a.md
- **Context:** Initial task creation

### 2026-05-05T08:15:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed


### 2026-05-20T13:23:01Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `agent snippet`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
