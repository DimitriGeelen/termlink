---
id: T-1515
name: "agent emoji-stats — fleet-wide emoji reaction counts on chat-arc"
description: >
  agent emoji-stats — fleet-wide emoji reaction counts on chat-arc

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T07:50:08Z
last_update: 2026-05-05T07:55:36Z
date_finished: 2026-05-05T07:55:36Z
---

# T-1515: agent emoji-stats — fleet-wide emoji reaction counts on chat-arc

## Context

`cmd_channel_emoji_stats(topic, by_sender, top, ...)` already computes lifetime emoji counts: walks the full arc, aggregates `msg_type=reaction` envelopes by emoji with reactor breakdown, sorts by count desc. Operator-relevant on chat-arc as a fleet-wide sentiment / acknowledgement signal: which emojis appear most, who is the most active reactor. Companion to `agent reactions <offset>` (per-post) — emoji-stats is per-arc. Thin wrapper hard-pinning topic to `agent-chat-arc` (~12 LOC). `--by-sender` flag opts in to per-reactor breakdown; `--top N` truncates results.

## Acceptance Criteria

### Agent
- [x] New `AgentAction::EmojiStats { by_sender, top, hub, json }` variant in cli.rs
- [x] main.rs dispatch arm calling `cmd_channel_emoji_stats("agent-chat-arc", by_sender, top, hub, json)`
- [x] `cargo build --release --bin termlink` clean
- [x] `agent emoji-stats --help` shows `--by-sender` / `--top` / `--hub` / `--json`
- [x] Live smoke text: `agent emoji-stats` renders `{emoji × count (N reactor(s))}` rows
- [x] Live smoke JSON: `agent emoji-stats --json` returns parseable JSON array

### Human
- [ ] [REVIEW] Verify `agent emoji-stats` reads naturally as fleet-sentiment
  **Steps:**
  1. `target/release/termlink agent emoji-stats`
  2. `target/release/termlink agent emoji-stats --by-sender --top 5`
  **Expected:** rows scannable as "what reactions exist + how many"; `--by-sender` shows reactor breakdown.
  **If not:** report layout suggestions.

## Verification

cargo build --release --bin termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent emoji-stats --help 2>&1 | grep -q -- "--hub"
target/release/termlink agent emoji-stats --help 2>&1 | grep -q -- "--by-sender"
target/release/termlink agent emoji-stats --json 2>&1 | python3 -c "import json,sys; d=json.load(sys.stdin); assert isinstance(d, list); print('OK')" | grep -q OK
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
**Rationale:** Closes the lifetime emoji-aggregate primitive on `agent.*` namespace. Companion to `agent reactions <offset>` (per-post): emoji-stats is per-arc, surfacing the dominant ack signals across all of chat-arc. `cmd_channel_emoji_stats` already does the aggregation. Pure dispatch wrapper. `--by-sender` opts in to per-reactor breakdown, `--top N` caps results.
**Evidence:**
- Build clean
- Verification gate 4/4 passed
- Live smoke text: rendered emoji × count (N reactors) rows
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

### 2026-05-05T07:50:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1515-agent-emoji-stats--fleet-wide-emoji-reac.md
- **Context:** Initial task creation

### 2026-05-05T07:55:36Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
