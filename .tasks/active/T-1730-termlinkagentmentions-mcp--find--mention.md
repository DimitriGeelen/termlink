---
id: T-1730
name: "termlink_agent_mentions MCP — find @-mentions of a user on agent-chat-arc (T-1513 parity)"
description: >
  termlink_agent_mentions MCP — find @-mentions of a user on agent-chat-arc (T-1513 parity)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-20T19:52:14Z
last_update: 2026-05-20T19:52:14Z
date_finished: null
---

# T-1730: termlink_agent_mentions MCP — find @-mentions of a user on agent-chat-arc (T-1513 parity)

## Context

CLI `termlink agent mentions <USER>` (T-1513, work-completed) is a thin wrapper over `channel mentions-of agent-chat-arc <USER>` — finds every envelope whose `metadata.mentions` CSV matches the target. Distinct from `agent search` (substring on payload): this verb filters on the structured mentions array. Glob `*` matches any non-empty CSV. MCP-aware agents currently can't ask "what's @-tagged for me?" without walking the topic manually and parsing metadata themselves. This task ships `termlink_agent_mentions` — port the pure `compute_mentions_of` helper + its dependencies (`mentions_match`, `extract_mentions`, `redacted_offsets`, `decode_payload_lossy`) into the MCP crate, register the tool, return the same JSON envelope shape as CLI `--json`.

## Acceptance Criteria

### Agent
- [x] `AgentMentionsParams` struct defined in tools.rs with single field `user: String` (the mention target — agent identity FP or peer name; `"*"` matches any non-empty CSV per T-1333 rules). Doc-comment cites T-1513 (CLI parity) + T-1730.
- [x] Pure helpers ported into tools.rs:
  - `mentions_match_mcp(csv, target) -> bool` — mirror of `mentions_match` (channel.rs:2701).
  - `extract_mentions_mcp(env) -> Option<String>` — mirror of `extract_mentions` (channel.rs:2722).
  - `decode_payload_lossy_mcp(env) -> String` — mirror of `decode_payload_lossy` (channel.rs:8518): base64::STANDARD decode of `payload_b64`, lossy UTF-8, empty on missing/invalid.
  - `redacted_offsets_mcp(msgs) -> HashSet<u64>` — mirror of `redacted_offsets`+`extract_redaction` (channel.rs:3213/3238): collects targets of `msg_type=redaction` envelopes whose `metadata.redacts` parses as u64.
  - `MentionsOfRowMcp` struct + `compute_mentions_of_mcp(envelopes, user) -> Vec<MentionsOfRowMcp>` — mirror of `MentionsOfRow`+`compute_mentions_of` (channel.rs:5211/5243): same filter chain (redacted-skip, META_MSG_TYPES-skip, mentions_match, ts_unix_ms-preferred-with-ts-fallback), same descending-offset sort.
- [x] `termlink_agent_mentions` tool method registered. Flow: hub_socket_path check → walk `agent-chat-arc` via existing `walk_topic_full_mcp` → `compute_mentions_of_mcp(&envelopes, &p.user)` → return `{ok, topic: "agent-chat-arc", user, mentions: [{mention_offset, sender_id, payload, mentions_csv, ts_ms}, ...]}`. Empty user input → JSON error. Empty result → `mentions: []`, still `ok: true`. NO new RPC surface — only `channel.subscribe` via `walk_topic_full_mcp`.
- [x] Tool description cites T-1513 (CLI parity), explains the `metadata.mentions` filter is structured (distinct from `termlink_agent_search`'s substring match), documents the `*` glob, names the hardcoded topic `agent-chat-arc`.
- [x] `cargo build --release -p termlink-mcp` clean — only the pre-existing `cur_run_end` warning.
- [x] **≥5** new unit tests under `tools::tests`: (a) `mentions_match_mcp` literal hit; (b) `mentions_match_mcp` `*` target matches any non-empty CSV; (c) `mentions_match_mcp` `*` in CSV matches every specific target; (d) `extract_mentions_mcp` returns None on missing metadata; (e) `compute_mentions_of_mcp` returns descending-offset rows; (f) `compute_mentions_of_mcp` skips redacted offsets; (g) `compute_mentions_of_mcp` skips META_MSG_TYPES envelopes; (h) `AgentMentionsParams` deserializes. All pass.

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

cargo build --release -p termlink-mcp
cargo test --release -p termlink-mcp agent_mentions

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

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-05-20T19:52:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1730-termlinkagentmentions-mcp--find--mention.md
- **Context:** Initial task creation
