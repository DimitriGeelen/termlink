---
id: T-1511
name: "agent digest — period summary on chat-arc"
description: >
  agent digest — period summary on chat-arc

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T07:25:56Z
last_update: 2026-05-20T13:22:56Z
date_finished: 2026-05-05T07:31:35Z
---

# T-1511: agent digest — period summary on chat-arc

## Context

`cmd_channel_digest` already exists and computes period summaries for any topic: posts count, distinct senders, top senders by volume, top reactions, pin/forward counts, recent_chats sample. Operator-frequent question on chat-arc: "what happened in the last hour?" / "what happened today?" Without `agent digest`, the operator must remember the topic name and run `channel digest agent-chat-arc --since-mins 60`. Thin wrapper hard-pinning topic to `agent-chat-arc`. Completes the operator-fluency loop alongside timeline/recent/search.

## Acceptance Criteria

### Agent
- [x] New `AgentAction::Digest { since_mins, since, hub, json }` variant in cli.rs
- [x] main.rs dispatch arm calling `cmd_channel_digest("agent-chat-arc", since_mins, since, hub, json)`
- [x] `cargo build --release --bin termlink` clean
- [x] `agent digest --help` shows `--since-mins` / `--since` / `--hub` / `--json`
- [x] Live smoke text: `agent digest --since-mins 60` renders posts/senders/recent_chats summary
- [x] Live smoke JSON: `agent digest --since-mins 60 --json` returns parseable envelope

### Human
- [ ] [REVIEW] Verify `agent digest` reads naturally as period summary
  **Steps:**
  1. `target/release/termlink agent digest --since-mins 60`
  2. `target/release/termlink agent digest --since-mins 1440` (last 24h)
  **Expected:** posts count, distinct senders, top senders, recent_chats sample. Operator-scannable as "what happened recently".
  **If not:** report layout suggestions.

## Verification

cargo build --release --bin termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent digest --help 2>&1 | grep -q -- "--since-mins"
target/release/termlink agent digest --help 2>&1 | grep -q -- "--hub"
target/release/termlink agent digest --since-mins 60 --json 2>&1 | python3 -c "import json,sys; d=json.load(sys.stdin); assert 'posts' in d; print('OK')" | grep -q OK
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
**Rationale:** Closes the period-summary primitive on `agent.*` namespace. `cmd_channel_digest` already computes the summary; this verb just hard-pins topic to `agent-chat-arc`. ~10 LOC dispatch wrapper. Pairs naturally with `agent timeline` (raw stream) and `agent stats` (lifetime counts) — digest fills the "compressed view of a recent slice" gap.
**Evidence:**
- Build clean
- Verification gate 4/4 passed
- Live smoke text: digest rendered with posts/senders/recent_chats
- Live smoke JSON: parseable

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

### 2026-05-05T07:25:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1511-agent-digest--period-summary-on-chat-arc.md
- **Context:** Initial task creation

### 2026-05-05T07:31:35Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed


### 2026-05-20T13:22:56Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `agent digest`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
