---
id: T-1512
name: "agent unread — count new posts on chat-arc since last receipt"
description: >
  agent unread — count new posts on chat-arc since last receipt

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T07:32:14Z
last_update: 2026-05-20T13:22:57Z
date_finished: 2026-05-05T07:37:38Z
---

# T-1512: agent unread — count new posts on chat-arc since last receipt

## Context

`cmd_channel_unread` already exists: queries the hub for the caller's last channel.receipts up_to value, then walks from up_to+1 to current end, counting content envelopes. Operator-frequent question on chat-arc: "how many new posts since I last looked?" Without `agent unread`, the operator must remember the topic and run `channel unread agent-chat-arc`. Thin wrapper hard-pinning topic to `agent-chat-arc`. Pairs with read-tracker workflow (channel.ack writes a receipt; agent.unread surfaces the gap).

## Acceptance Criteria

### Agent
- [x] New `AgentAction::Unread { sender, hub, json }` variant in cli.rs
- [x] main.rs dispatch arm calling `cmd_channel_unread("agent-chat-arc", sender, hub, json)`
- [x] `cargo build --release --bin termlink` clean
- [x] `agent unread --help` shows `--sender` / `--hub` / `--json`
- [x] Live smoke text: `agent unread` renders "up to date" or "N unread" with first/last offsets
- [x] Live smoke JSON: `agent unread --json` returns parseable envelope with `unread_count`

### Human
- [ ] [REVIEW] Verify `agent unread` reads naturally as gap-since-last-read
  **Steps:**
  1. `target/release/termlink agent unread`
  2. `target/release/termlink agent unread --json`
  **Expected:** count + first/last unread offsets; JSON has unread_count/first_unread/last_offset/up_to.
  **If not:** report layout suggestions.

## Verification

cargo build --release --bin termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent unread --help 2>&1 | grep -q -- "--hub"
target/release/termlink agent unread --json 2>&1 | python3 -c "import json,sys; d=json.load(sys.stdin); assert 'unread_count' in d; print('OK')" | grep -q OK
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
**Rationale:** Closes the gap-since-last-read primitive on `agent.*` namespace. `cmd_channel_unread` already does the work (server-side receipt lookup + arc walk + count). Pure dispatch wrapper (~10 LOC). Pairs with the channel.ack receipts the existing read verbs already emit. Operator workflow: `agent unread` → "you have 14 new posts" → `agent timeline -n 14` → catch up.
**Evidence:**
- Build clean
- Verification gate 3/3 passed
- Live smoke text: rendered count and offsets
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

### 2026-05-05T07:32:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1512-agent-unread--count-new-posts-on-chat-ar.md
- **Context:** Initial task creation

### 2026-05-05T07:37:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed


### 2026-05-20T13:22:57Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `agent unread`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
