---
id: T-1499
name: "agent recent / on-thread --msg-type — signal vs noise filter"
description: >
  Add --msg-type allowlist filter to agent recent + agent on-thread (composable with existing thread/project filters). Operator wants signal-vs-noise: see only note (real content) and skip status/star (heartbeats). Helper extract_recent_posts gains filter_msg_types: Option<&[&str]> param. Single comma-sep value at the CLI surface (e.g., --msg-type note,edit,status). Pure helper change with new unit tests.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T20:08:44Z
last_update: 2026-05-04T20:24:00Z
date_finished: 2026-05-04T20:24:00Z
---

# T-1499: agent recent / on-thread --msg-type — signal vs noise filter

## Context

The fleet chat-arc carries multiple msg_types: `note` (real content),
`status` (heartbeat-style updates), `star` (signal flags), plus
already-excluded meta types (`reaction`, `edit`, `redaction`,
`topic_metadata`, `receipt`). For an operator wanting to read what
actually happened, heartbeats are noise. Today there's no way to
narrow: `agent recent <peer>` and `agent on-thread <T-XXX>` show
everything non-meta. This task adds an allowlist `--msg-type
<list>` flag (comma-separated) that, when set, restricts output to
posts whose msg_type matches. Composes with the existing
filter-thread / filter-project / filter-peer-fp filters (AND-composed).
Pure helper change in `extract_recent_posts` (T-1492 / T-1493) plus
flag plumbing in cli/main/agent.

## Acceptance Criteria

### Agent
- [x] `extract_recent_posts` gains `filter_msg_types: Option<&[&str]>` param — when Some, posts whose msg_type is NOT in the list are skipped
- [x] AND-composes with existing peer / thread / project filters (any/all may be set independently)
- [x] Meta exclusion (reaction/edit/redaction/topic_metadata/receipt) still applies first — `--msg-type edit` does NOT bypass meta exclusion (intentional: edits are not direct-readable content)
- [x] `--msg-type <type[,type...]>` flag on Recent variant (clap parses comma-sep into Vec<String>)
- [x] `--msg-type <type[,type...]>` flag on OnThread variant (same shape)
- [x] main.rs propagates new value through both AgentAction::Recent and AgentAction::OnThread dispatches
- [x] cmd_agent_recent passes filter to all 3 callsites (one-shot json, one-shot text, watch loop)
- [x] cmd_agent_on_thread passes filter to all 3 callsites (one-shot json, one-shot text, watch loop)
- [x] Text-mode header includes `msg_type=<csv>` suffix when set (for both verbs, both one-shot and watch)
- [x] JSON envelope includes `filter_msg_types: [...]` when set (omitted when unset, matches existing field-omission convention)
- [x] New unit tests in channel.rs: (1) filter_msg_types=Some(["note"]) filters out status; (2) filter_msg_types=Some(["note","status"]) keeps both; (3) filter_msg_types=None keeps all non-meta (regression); (4) filter_msg_types AND-composes with filter_peer_fp
- [x] All existing extract_recent_posts unit tests pass (signature change is additive — None preserves prior behavior)
- [x] `cargo build --release -p termlink` clean
- [x] Live smoke: `agent recent --target-fp d1993c2c3ec44c94 --window-secs 86400 --msg-type note --n 5` returns only msg_type=note posts; same with `--msg-type note,status` returns both types

### Human
- [ ] [REVIEW] Verify --msg-type filtering output is operator-readable
  **Steps:**
  1. `target/release/termlink agent recent --target-fp <fp> --window-secs 86400 --msg-type note` (run from /opt/termlink); compare to no-filter run
  2. Same for `agent on-thread <T-XXX> --msg-type note,status`
  **Expected:** Only listed msg_types shown; header includes `msg_type=<csv>` suffix; JSON includes `filter_msg_types: [...]` when --json
  **If not:** suggest header layout / additional msg-type variants worth surfacing

## Verification

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
cargo test --release -p termlink --lib commands::channel::tests::recent_posts 2>&1 | tail -3 | grep -qE "test result: ok"
target/release/termlink agent recent --help 2>&1 | grep -q -- "--msg-type"
target/release/termlink agent on-thread --help 2>&1 | grep -q -- "--msg-type"
out=$(target/release/termlink agent recent --target-fp d1993c2c3ec44c94 --window-secs 86400 --msg-type note --n 5 2>&1); echo "$out" | grep -qE "msg_type=note|no posts found"

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

**Rationale:** Closes signal-vs-noise filtering for the chronological-reading verbs (recent + on-thread). Operator can `--msg-type note` to skip status/star heartbeats and see only real content; `--msg-type note,edit,status` to compose. AND-composes with the 3 existing filters (peer, thread, project), so ops like "all `note` posts on T-XXX from <peer>" reduce to one verb. Pure helper change in `extract_recent_posts` covered by 5 new unit tests; signature change is additive (None preserves prior behavior, all 11 prior tests still pass).

**Evidence:**
- 16 unit tests pass (11 prior + 5 new T-1499)
- Live: `agent recent --target-fp d1993c2c3ec44c94 --msg-type note` → 1 post (filtered out 2 heartbeat posts)
- Live: `agent recent --msg-type note,status` → 5 posts (both types kept)
- Live: `agent recent --json --msg-type note` → envelope includes `filter_msg_types: ["note"]`
- Header: `# agent recent ... | window=86400s | n=5 msg_type=note` (suffix when set)
- Verification: 5/5 commands pass

## Decisions

### 2026-05-04 — Allowlist (--msg-type) vs denylist (--exclude-msg-type)
- **Chose:** Allowlist semantics; meta types (reaction/edit/redaction/topic_metadata/receipt) always excluded regardless of allowlist.
- **Why:** Allowlist matches operator intent — they want to focus on a few types, not enumerate all noise types to exclude. Meta exclusion is structural (these types are not direct-readable content); allowing them via --msg-type would surface stale data and confuse the reading view.
- **Rejected:** Denylist semantics (--exclude-msg-type) — would require operator to know the full type vocabulary. Could add later if real demand surfaces.

## Updates

### 2026-05-04T20:08:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1499-agent-recent--on-thread---msg-type--sign.md
- **Context:** Initial task creation

### 2026-05-04T20:24:00Z — status-update [manual]
- **Change:** status: started-work → work-completed (G-054 workaround: fw task update flock-deadlocked)
- **Owner:** agent → human (partial-complete; Human REVIEW AC pending)
