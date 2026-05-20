---
id: T-1729
name: "termlink_agent_inbox MCP — cross-topic unread digest parity with CLI (T-1553)"
description: >
  termlink_agent_inbox MCP — cross-topic unread digest parity with CLI (T-1553)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-20T19:37:37Z
last_update: 2026-05-20T19:45:54Z
date_finished: 2026-05-20T19:45:54Z
---

# T-1729: termlink_agent_inbox MCP — cross-topic unread digest parity with CLI (T-1553)

## Context

CLI `termlink agent inbox` (T-1553, work-completed) is the operator's first-command-of-session ("what needs my attention?"). It walks the local cursor store (`~/.termlink/cursors.json`), joins with hub-side `channel.list` topic counts, and reports unread-per-topic. MCP-aware agents currently have no equivalent — they have to manually call `termlink_channel_list` and have no way to read their own subscription cursors. This task ships `termlink_agent_inbox` as a thin MCP wrapper — identity load → cursor enumeration → channel.list → compute_unread_rows. Follows the established T-1715/T-1719 pattern: port pure helpers as `*_mcp` variants and reuse hub-socket / channel.list path verbatim.

## Acceptance Criteria

### Agent
- [x] `AgentInboxParams` struct defined in `crates/termlink-mcp/src/tools.rs` — parameter-less (mirrors CLI which only has `--hub`/`--json`; MCP defaults to local hub, JSON-only output). Doc-comment cites T-1553 (CLI parity) + T-1729.
- [x] Pure helper `cursor_list_for_fingerprint_mcp(fingerprint: &str) -> Result<Vec<(String, u64)>, String>` ported into tools.rs — reads `${TERMLINK_IDENTITY_DIR:-${HOME}/.termlink}/cursors.json`, parses BTreeMap<String, u64>, filters keys ending in `::<fingerprint>`, strips suffix, sort by topic asc. Mirrors `cursor_store::list_for_fingerprint` (channel.rs:592) one-to-one. Defensive: missing file → empty vec (matches CLI), invalid JSON → error string.
- [x] Pure helper `compute_unread_rows_mcp(cursors, topic_counts) -> Vec<UnreadRowMcp>` ported into tools.rs — mirrors `compute_unread_rows` (channel.rs:7422) one-to-one: drops topics missing from counts, drops count==0, drops cursor>=latest, sorts by descending `unread` then ascending `topic`.
- [x] `termlink_agent_inbox` tool method registered via `#[tool(name = "termlink_agent_inbox", ...)]`. Flow: hub_socket_path check → load identity (HOME-based, matches T-1719) → `cursor_list_for_fingerprint_mcp(my_id)` → empty cursors → return `{ok, dms: []}` early-out → `channel.list` RPC → extract `(name, count)` counts map → `compute_unread_rows_mcp` → return `{ok, my_id, unread_topics: [{topic, cursor, latest, unread}, ...]}`. NO new RPC surface — only `channel.list`.
- [x] Tool description cites T-1553 (CLI parity) + T-1719 (sibling DM-directory pattern), explains the cursor-store dependency (cursors recorded by `subscribe --resume`), and distinguishes single-topic-walk verbs (`agent unread`, `agent dms`) from this cross-topic digest.
- [x] `cargo build --release -p termlink-mcp` clean — only the pre-existing `cur_run_end` warning.
- [x] **≥4** new unit tests added under `tools::tests`: (a) `compute_unread_rows_mcp` empty cursors → []; (b) caller caught up (cursor==latest) → topic dropped; (c) caller behind → row emitted with correct unread+latest; (d) sort: high-unread topic before low-unread, alpha tie-break; (e) `cursor_list_for_fingerprint_mcp` missing file → Ok(empty); (f) params deserialize from `{}`. All pass.

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
cargo test --release -p termlink-mcp agent_inbox

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

### 2026-05-20T19:37:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1729-termlinkagentinbox-mcp--cross-topic-unre.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-12117041
- **Timestamp:** 2026-05-20T19:50:37Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#1 (Agent)** — `AgentInboxParams` struct defined in `crates/termlink-mcp/src/tools.rs` — parameter-less (mirrors CLI which only has `--hub`/`--json`; MCP defaults to local hub, JSON-only output). Doc-comment cites T
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-mcp/src/tools.rs in: `AgentInboxParams` struct defined in `crates/termlink-mcp/src/tools.rs` — parameter-less (mirrors CLI which only has `--hub`/`--json`; MCP defaults to`

### 2026-05-20T19:45:54Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
