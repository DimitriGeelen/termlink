---
id: T-1524
name: "agent info — read agent-chat-arc topic metadata + counts"
description: >
  agent info — read agent-chat-arc topic metadata + counts

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T08:28:49Z
last_update: 2026-05-20T13:23:04Z
date_finished: 2026-05-05T08:34:34Z
---

# T-1524: agent info — read agent-chat-arc topic metadata + counts

## Context

`cmd_channel_info(topic, since, ...)` already exists: pulls retention + count from channel.list, walks the arc once, surfaces latest topic_metadata description, distinct senders summary, and per-sender receipts. Operator workflow on chat-arc: "what is this topic, how many envelopes, what's the description, who has acked up to where?" — currently requires `channel info agent-chat-arc`. Thin wrapper hard-pinning topic to `agent-chat-arc` (~10 LOC). Task originally framed as "agent describe" — pivoted to "agent info" because `cmd_channel_describe` is the WRITE verb (posts topic_metadata) while the READ verb is `cmd_channel_info`. See Decisions.

## Acceptance Criteria

### Agent
- [x] New `AgentAction::Info { since, hub, json }` variant in cli.rs
- [x] main.rs dispatch arm calling `cmd_channel_info("agent-chat-arc", since, hub, json)`
- [x] `cargo build --release --bin termlink` clean
- [x] `agent info --help` shows `--since` / `--hub` / `--json`
- [x] Live smoke text: `agent info` renders topic metadata (count/description/senders/receipts)
- [x] Live smoke JSON: `agent info --json` returns parseable envelope

### Human
- [ ] [REVIEW] Verify `agent info` reads naturally as topic-summary
  **Steps:**
  1. `target/release/termlink agent info`
  2. `target/release/termlink agent info --json | head -40`
  **Expected:** count, retention, description (if set), distinct senders, receipts.
  **If not:** report layout suggestions.

## Verification

cargo build --release --bin termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent info --help 2>&1 | grep -q -- "--hub"
target/release/termlink agent info --json 2>&1 | python3 -c "import json,sys; d=json.load(sys.stdin); assert 'topic' in d or 'count' in d; print('OK')" | grep -q OK
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
**Rationale:** Closes the topic-self-doc primitive on `agent.*` namespace. Operator-fluency parity: `agent info` answers "what is chat-arc + how big + who's here + what's the description". `cmd_channel_info` already does the work. Pure dispatch wrapper.
**Evidence:**
- Build clean
- Verification gate 3/3 passed
- Live smoke text: rendered topic metadata
- Live smoke JSON: parseable envelope

## Decisions

### 2026-05-05 — pivoted from agent describe to agent info
- **Chose:** info (READ verb)
- **Why:** `cmd_channel_describe` is the WRITE verb (posts topic_metadata envelope). The READ verb that surfaces the description plus count/retention/senders/receipts is `cmd_channel_info`. "agent info" is the right semantic for operator self-doc.
- **Rejected:** Wrap `cmd_channel_describe` as `agent describe` — would conflate write-with-read; the existing `agent post` already covers writing, and a topic_metadata-specific writer can wait until needed.

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-05-05T08:28:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1524-agent-describe--agent-chat-arc-topic-sel.md
- **Context:** Initial task creation

### 2026-05-05T08:34:34Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed


### 2026-05-20T13:23:04Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `agent info`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
