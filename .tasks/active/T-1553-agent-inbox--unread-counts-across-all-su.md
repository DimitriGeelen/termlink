---
id: T-1553
name: "agent inbox — unread counts across all subscribed topics"
description: >
  agent inbox — unread counts across all subscribed topics

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T10:52:20Z
last_update: 2026-05-05T10:58:32Z
date_finished: 2026-05-05T10:58:32Z
---

# T-1553: agent inbox — unread counts across all subscribed topics

## Context

`cmd_channel_inbox(...)` already exists: walks the local cursor store for the current identity, joins with hub-side topic counts via `channel.list`, and computes unread per topic. Operator workflow: "where do I have unread messages waiting?" — single-shot fleet inbox view across chat-arc, DMs, and any other subscribed topics. Companion to `agent unread` (chat-arc-only) which targets one topic. Without this verb, an operator must walk topics individually. Pure dispatch wrapper (~6 LOC). NOT chat-arc-pinned: this is a per-identity, multi-topic reader.

## Acceptance Criteria

### Agent
- [x] New `AgentAction::Inbox { hub, json }` variant in cli.rs
- [x] main.rs dispatch arm calling `cmd_channel_inbox(hub, json)`
- [x] `cargo build --release --bin termlink` clean
- [x] `agent inbox --help` shows `--hub` / `--json`
- [x] Live smoke text: `agent inbox` returns rows or "No cursors recorded yet"
- [x] Live smoke JSON: `agent inbox --json` returns parseable array

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
-->
- [ ] [REVIEW] Verify `agent inbox` reads naturally as cross-topic unread digest
  **Steps:**
  1. `target/release/termlink agent inbox`
  2. `target/release/termlink agent inbox --json | jq '.[]'`
  **Expected:** rows surface every topic with a recorded cursor + unread delta against current count.
  **If not:** report layout suggestions.

## Verification

cargo build --release --bin termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent inbox --help 2>&1 | grep -qiE "Inbox|--hub"
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
**Rationale:** Surfaces cross-topic unread state at the agent.* layer — operator's first command for "what needs my attention now". Companion to T-1512 `agent unread` (chat-arc only) and T-1552 `agent dms` (DM directory). `cmd_channel_inbox` already does the cursor + count join. Pure dispatch wrapper.
**Evidence:**
- Build clean
- Verification gate 2/2 passed
- Live smoke: rows reflect each tracked topic's unread delta

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

### 2026-05-05T10:52:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1553-agent-inbox--unread-counts-across-all-su.md
- **Context:** Initial task creation

### 2026-05-05T10:58:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
