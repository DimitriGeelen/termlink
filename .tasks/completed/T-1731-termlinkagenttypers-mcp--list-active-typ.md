---
id: T-1731
name: "termlink_agent_typers MCP — list active typers on agent-chat-arc (T-1551 parity)"
description: >
  termlink_agent_typers MCP — list active typers on agent-chat-arc (T-1551 parity)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-20T20:05:24Z
last_update: 2026-05-20T20:12:19Z
date_finished: 2026-05-20T20:12:19Z
---

# T-1731: termlink_agent_typers MCP — list active typers on agent-chat-arc (T-1551 parity)

## Context

CLI `termlink agent typers` (T-1551, work-completed) is a thin wrapper over `channel typing-list agent-chat-arc` — walks the topic, runs `compute_active_typers` (latest typing envelope per sender, filtered by `expires_at_ms > now`), and returns one row per active typer. Write companion is `termlink_agent_typing` (T-1550, MCP-shipped). MCP-aware agents currently can EMIT typing indicators but can't READ them — they have to walk the topic themselves. This task ships `termlink_agent_typers` — port the pure `compute_active_typers` helper + `TyperRowMcp` struct, register the tool, return the same JSON envelope shape.

## Acceptance Criteria

### Agent
- [x] `AgentTypersParams` struct defined in tools.rs — parameter-less (mirrors CLI which only has `--hub`/`--json`; MCP always uses local hub, always JSON). Doc-comment cites T-1551 (CLI parity) + T-1731.
- [x] Pure helper + struct ported into tools.rs:
  - `TyperRowMcp` — `{sender_id, expires_at_ms, ts}` (mirror of `TyperRow` channel.rs:3331).
  - `compute_active_typers_mcp(envelopes, now_ms) -> Vec<TyperRowMcp>` — mirror of `compute_active_typers` (channel.rs:3359) one-to-one: filter `msg_type=typing`, latest-per-sender by insertion order (envelopes arrive in offset order), parse `metadata.expires_at_ms` (string-encoded i64), drop where `expires_at_ms <= now_ms`, sort by `ts` descending then `sender_id` ascending for determinism.
- [x] `termlink_agent_typers` tool method registered. Flow: hub_socket_path check → walk `agent-chat-arc` via existing `walk_topic_full_mcp` → compute `now_ms` from `SystemTime` → `compute_active_typers_mcp(&envelopes, now_ms)` → return `{ok, topic: "agent-chat-arc", now_ms, typers: [{sender_id, expires_at_ms, ts}, ...]}`. Empty result → `typers: []`, still `ok: true`. NO new RPC surface — only `channel.subscribe` via `walk_topic_full_mcp`.
- [x] Tool description cites T-1551 (CLI parity) + T-1550 sibling (write companion `termlink_agent_typing`), explains the 5s default TTL filter semantics, names hardcoded topic `agent-chat-arc`. Distinguishes from `termlink_agent_presence_now` (last-post recency).
- [x] `cargo build --release -p termlink-mcp` clean — only the pre-existing `cur_run_end` warning.
- [x] **≥4** new unit tests under `tools::tests`: (a) empty input → []; (b) single active typer survives; (c) expired typer dropped; (d) latest-per-sender: later envelope from same sender overwrites earlier; (e) sort: ts descending, sender_id alpha tie-break; (f) `AgentTypersParams` deserialize from `{}`. All pass.

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
cargo test --release -p termlink-mcp agent_typers

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

### 2026-05-20T20:05:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1731-termlinkagenttypers-mcp--list-active-typ.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-7331659a
- **Timestamp:** 2026-05-20T20:16:42Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-20T20:12:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
