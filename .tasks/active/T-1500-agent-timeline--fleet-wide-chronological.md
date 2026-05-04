---
id: T-1500
name: "agent timeline — fleet-wide chronological log (tail -f for the fleet)"
description: >
  Add 'agent timeline' verb — fleet-wide chronological log of posts across all peers in a window. Mirrors recent/on-thread shape but no peer or thread filter required (both optional). Compose with --thread, --project, --msg-type, --window-secs, --n, --watch. Operator gets a 'tail -f for the fleet' primitive: see all activity chronologically without first picking a peer or thread. Pure wrapper around extract_recent_posts (peer=None). New render_timeline_body helper that prefixes each post with peer_short for multi-peer disambiguation.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T20:29:04Z
last_update: 2026-05-04T20:29:04Z
date_finished: null
---

# T-1500: agent timeline — fleet-wide chronological log (tail -f for the fleet)

## Context

T-1492 (agent recent <peer>) shows posts from one peer; T-1493 (agent on-thread <T-XXX>) shows posts on one thread across all peers. Missing primitive: see ALL fleet activity chronologically without first picking a peer or thread — a "tail -f for the fleet". This task adds `agent timeline` as a pure wrapper around the same `extract_recent_posts` helper (peer=None, thread/project/msg-type all optional). New `render_timeline_body` helper prefixes each post with peer-short (8-char fp prefix) for multi-peer disambiguation. Composes with --thread, --project, --msg-type, --window-secs, --n, --watch, --json. No new helper changes — reuses the T-1492/T-1493/T-1499 plumbing.

## Acceptance Criteria

### Agent
- [x] New AgentAction::Timeline variant (window_secs default 3600, n default 50, hub, json, watch, watch_interval, filter_thread, filter_project, filter_msg_types)
- [x] main.rs propagates new value through AgentAction::Timeline dispatch
- [x] cmd_agent_timeline in agent.rs: one-shot json + one-shot text + watch loop branches (mirrors cmd_agent_recent shape)
- [x] watch+json mutually exclusive (clap conflict OR runtime error matching prior verbs)
- [x] Pure wrapper: calls extract_recent_posts(msgs, n, window, now, None /*peer*/, filter_thread, filter_project, filter_msg_types) — no helper changes
- [x] New render_timeline_body helper prefixes each post with peer_short (first 8 chars of peer fp)
- [x] Text-mode header: `# agent timeline | window=Ns | n=N` plus optional `thread=... project=... msg_type=<csv>` suffixes when set
- [x] JSON envelope includes filter_thread / filter_project / filter_msg_types when set (omitted when unset, matches existing field-omission convention)
- [x] `cargo build --release -p termlink` clean
- [x] `cargo test --release -p termlink --lib commands::channel::tests::recent_posts` still passes (no regressions to shared helper)
- [x] Live smoke: `agent timeline --window-secs 86400 --n 10` returns at least one post with peer-short prefix
- [x] Live smoke: `agent timeline --json --window-secs 86400 --n 5 --msg-type note` JSON envelope includes filter_msg_types: ["note"]

### Human
- [ ] [REVIEW] Verify timeline output is operator-readable as fleet "tail -f"
  **Steps:**
  1. `target/release/termlink agent timeline --window-secs 86400 --n 20` (run from /opt/termlink)
  2. `target/release/termlink agent timeline --window-secs 3600 --watch` (Ctrl+C to exit)
  3. `target/release/termlink agent timeline --msg-type note --window-secs 86400 --n 10`
  **Expected:** Posts ordered chronologically, peer-short prefix lets you see who posted each line, --watch refreshes without flicker, --msg-type filter works.
  **If not:** suggest layout improvements / additional filter fields worth surfacing.

## Verification

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
cargo test --release --bin termlink commands::channel::tests::recent_posts 2>&1 | tail -3 | grep -qE "test result: ok"
target/release/termlink agent timeline --help 2>&1 | grep -q -- "--window-secs"
target/release/termlink agent timeline --help 2>&1 | grep -q -- "--watch"
target/release/termlink agent timeline --help 2>&1 | grep -q -- "--msg-type"
out=$(target/release/termlink agent timeline --window-secs 86400 --n 5 --json 2>&1); echo "$out" | grep -qE '"posts":\['

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

### 2026-05-04T20:29:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1500-agent-timeline--fleet-wide-chronological.md
- **Context:** Initial task creation
