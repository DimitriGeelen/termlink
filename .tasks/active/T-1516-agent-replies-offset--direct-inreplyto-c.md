---
id: T-1516
name: "agent topic-stats — lifetime structural breakdown of chat-arc"
description: >
  agent topic-stats — lifetime structural breakdown of chat-arc

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T07:56:04Z
last_update: 2026-05-05T08:02:02Z
date_finished: 2026-05-05T08:02:02Z
---

# T-1516: agent topic-stats — lifetime structural breakdown of chat-arc

## Context

`cmd_channel_topic_stats` already computes the full structural breakdown for any topic: total envelopes, distinct senders, by_msg_type histogram, top senders, top emojis, thread roots count, active pins, forwards_in, edits, redactions, time span. Distinct from `agent stats` (windowed counts) and from `agent digest` (period-summary): topic-stats is **lifetime** + **structural** — answers "what's the shape of this arc?" Pivoted T-1516 from `agent replies <offset>` (no matching helper — `cmd_channel_replies_of` is by-sender, not by-parent) to `agent topic-stats` after spotting the helper-shape gap. Thin wrapper hard-pinning topic to `agent-chat-arc` (~10 LOC).

## Acceptance Criteria

### Agent
- [x] New `AgentAction::TopicStats { hub, json }` variant in cli.rs
- [x] main.rs dispatch arm calling `cmd_channel_topic_stats("agent-chat-arc", hub, json)`
- [x] `cargo build --release --bin termlink` clean
- [x] `agent topic-stats --help` shows `--hub` / `--json`
- [x] Live smoke text: `agent topic-stats` renders total/sender/msg_type breakdown
- [x] Live smoke JSON: `agent topic-stats --json` returns parseable envelope with `total` + `by_msg_type`

### Human
- [ ] [REVIEW] Verify `agent topic-stats` reads naturally as lifetime shape
  **Steps:**
  1. `target/release/termlink agent topic-stats`
  **Expected:** total / distinct_senders / thread_roots / active_pins / by msg_type histogram / top senders.
  **If not:** report layout suggestions.

## Verification

cargo build --release --bin termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent topic-stats --help 2>&1 | grep -q -- "--hub"
target/release/termlink agent topic-stats --json 2>&1 | python3 -c "import json,sys; d=json.load(sys.stdin); assert 'total' in d and 'by_msg_type' in d; print('OK')" | grep -q OK
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
**Rationale:** Closes the lifetime structural-breakdown primitive on `agent.*` namespace. Distinct shape from `agent stats` (windowed) and `agent digest` (period-summary). `cmd_channel_topic_stats` already does the work. Pure dispatch wrapper. Operator workflow: `agent topic-stats` → see "shape of chat-arc" (msg_type histogram, thread roots, edits, redactions, time span) at a glance.
**Evidence:**
- Build clean
- Verification gate 3/3 passed
- Live smoke text: rendered structural breakdown
- Live smoke JSON: parseable envelope with `total` + `by_msg_type`

## Decisions

### 2026-05-05 — pivoted from agent replies to agent topic-stats
- **Chose:** topic-stats (lifetime structural breakdown)
- **Why:** `cmd_channel_replies_of` is by-sender, not by-parent-offset; no `replies_on` helper exists. The originally-framed verb `agent replies <offset>` has no thin-wrap path without building a new helper.
- **Rejected:** Keep "agent replies-of [--sender]" — narrower utility than topic-stats; reframe deferred to a later task if signal emerges.

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-05-05T07:56:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1516-agent-replies-offset--direct-inreplyto-c.md
- **Context:** Initial task creation

### 2026-05-05T08:02:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
