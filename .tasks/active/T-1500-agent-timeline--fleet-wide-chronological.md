---
id: T-1500
name: "agent timeline — fleet-wide chronological log (tail -f for the fleet)"
description: >
  Add 'agent timeline' verb — fleet-wide chronological log of posts across all peers in a window. Mirrors recent/on-thread shape but no peer or thread filter required (both optional). Compose with --thread, --project, --msg-type, --window-secs, --n, --watch. Operator gets a 'tail -f for the fleet' primitive: see all activity chronologically without first picking a peer or thread. Pure wrapper around extract_recent_posts (peer=None). New render_timeline_body helper that prefixes each post with peer_short for multi-peer disambiguation.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T20:29:04Z
last_update: 2026-05-04T21:45:19Z
date_finished: 2026-05-04T21:44:28Z
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

## Recommendation

**Recommendation:** GO
**Rationale:** Closes the fleet-wide chronological log primitive — "tail -f for the fleet". Companion to `agent recent` (one peer) and `agent on-thread` (one thread). Pure dispatch wrapper composing existing `extract_recent_posts` reducer with no peer filter.
**Evidence:**
- Build clean
- Live smoke: timeline returns chronologically-ordered posts across all peers, with peer-short prefix in render
- Composes cleanly with --thread, --project, --msg-type, --watch, --json

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

### 2026-05-04T21:44:28Z — status-update [manual]
- **Change:** status: started-work → work-completed (G-054 workaround: fw task update flock-deadlocked)
- **Owner:** agent → human (partial-complete; Human REVIEW AC pending)

### 2026-06-13T13:42:27Z — G-008 fresh evidence [resmoke-agent]
- **Action:** Re-ran Human-AC Steps to capture fresh output (>2wk since build smoke)
- **Command(s):** `agent timeline --window-secs 86400 --n 20 ; agent timeline --window-secs 3600 --watch ; agent timeline --msg-type note --window-secs 86400 --n 10`
- **Result:** exit=0; 124(watch); 0; ok
- **Output:**
  ```
  ### CMD1: timeline --window-secs 86400 --n 20
  # agent timeline | window=86400s | n=20
  [14h ago] [d1993c2c] @3176 msg_type=chat thread=T-1438 project=010-termlink
      T-1438 vendored-arc heartbeat from dimitrimintdev (x86_64, Linux) at 2026-06-13T01:17:01+02:00. Binary: /usr/local/bin/termlink (termlink 0.9.1542).
  
  [13h ago] [d1993c2c] @3178 msg_type=chat thread=T-1438 project=010-termlink
      T-1438 vendored-arc heartbeat from dimitrimintdev (x86_64, Linux) at 2026-06-13T02:17:01+02:00. Binary: /usr/local/bin/termlink (termlink 0.9.1542).
  
  [12h ago] [d1993c2c] @3179 msg_type=chat thread=T-1438 project=010-termlink
      T-1438 vendored-arc heartbeat from dimitrimintdev (x86_64, Linux) at 2026-06-13T03:17:01+02:00. Binary: /usr/local/bin/termlink (termlink 0.9.1542).
  
  [11h ago] [d1993c2c] @3180 msg_type=chat thread=T-1438 project=010-termlink
      T-1438 vendored-arc heartbeat from dimitrimintdev (x86_64, Linux) at 2026-06-13T04:17:01+02:00. Binary: /usr/local/bin/termlink (termlink 0.9.1542).
  
  [10h ago] [d1993c2c] @3181 msg_type=chat thread=T-1438 project=010-termlink
  ```
- **Note:** Human [REVIEW] AC remains UNCHECKED — sovereignty; evidence provided for batch-confirm.
