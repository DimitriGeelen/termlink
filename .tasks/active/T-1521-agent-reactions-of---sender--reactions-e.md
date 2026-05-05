---
id: T-1521
name: "agent reactions-of [--sender] — reactions emitted by an identity on chat-arc"
description: >
  agent reactions-of [--sender] — reactions emitted by an identity on chat-arc

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T08:21:07Z
last_update: 2026-05-05T08:28:11Z
date_finished: 2026-05-05T08:28:11Z
---

# T-1521: agent reactions-of [--sender] — reactions emitted by an identity on chat-arc

## Context

`cmd_channel_reactions_of(topic, sender, ...)` already exists: walks the arc, lists every reaction envelope authored BY the given sender (default: local identity). Distinct from `agent reactions <offset>` (T-1514, reactions ON a parent). This is "show me everywhere agent X has reacted on chat-arc". Operator workflow: triage a peer's engagement signal — what they've ack'd, applauded, flagged. Thin wrapper hard-pinning topic to `agent-chat-arc` (~10 LOC). Part of the by-sender trio (T-1521 reactions-of, T-1522 forwards-of, T-1523 replies-of).

## Acceptance Criteria

### Agent
- [x] New `AgentAction::ReactionsOf { sender, hub, json }` variant in cli.rs
- [x] main.rs dispatch arm calling `cmd_channel_reactions_of("agent-chat-arc", sender, hub, json)`
- [x] `cargo build --release --bin termlink` clean
- [x] `agent reactions-of --help` shows `--sender` / `--hub` / `--json`
- [x] Live smoke text: `agent reactions-of` renders self-reaction rows or empty-message
- [x] Live smoke JSON: `agent reactions-of --json` returns parseable JSON array

### Human
- [ ] [REVIEW] Verify `agent reactions-of` reads naturally as engagement signal
  **Steps:**
  1. `target/release/termlink agent reactions-of`
  2. `target/release/termlink agent reactions-of --sender 9219671e6604a2af`
  **Expected:** rows show `<emoji> → offset <parent> (<preview>)`. Empty-message when no reactions.
  **If not:** report layout suggestions.

## Verification

cargo build --release --bin termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent reactions-of --help 2>&1 | grep -q -- "--sender"
target/release/termlink agent reactions-of --json 2>&1 | python3 -c "import json,sys; d=json.load(sys.stdin); assert isinstance(d, list); print('OK')" | grep -q OK
# Shell commands that MUST pass before work-completed. One per line.

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
**Rationale:** Closes the by-sender reactions primitive on `agent.*` namespace. Companion to `agent reactions <offset>` (T-1514, parent-side): reactions-of is sender-side. Triple-shipped with T-1522 forwards-of + T-1523 replies-of. `cmd_channel_reactions_of` already does the work. Pure dispatch wrapper.
**Evidence:**
- Build clean
- Verification gate 3/3 passed
- Live smoke text: rendered reaction rows or empty-message
- Live smoke JSON: parseable array

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

### 2026-05-05T08:21:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1521-agent-reactions-of---sender--reactions-e.md
- **Context:** Initial task creation

### 2026-05-05T08:28:11Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
