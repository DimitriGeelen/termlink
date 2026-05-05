---
id: T-1590
name: "termlink_agent_envelope + termlink_agent_who_is — single-offset hydrate + fingerprint→display-name resolver MCP read tools"
description: >
  termlink_agent_envelope + termlink_agent_who_is — single-offset hydrate + fingerprint→display-name resolver MCP read tools

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T19:51:32Z
last_update: 2026-05-05T19:51:32Z
date_finished: null
---

# T-1590: termlink_agent_envelope + termlink_agent_who_is — single-offset hydrate + fingerprint→display-name resolver MCP read tools

## Context

T-1589 brought MCP read surface to 124 tools. Wave 46 adds two **identity/orientation primitives**:

- `termlink_agent_envelope` — single offset deep-fetch. Walks topic, finds envelope at given offset, returns the FULL hydrated record: `{offset, sender_id, msg_type, payload_decoded, payload_b64, metadata, ts_unix_ms}`. Replaces `agent_quote` (single-line preview) with full structured payload + metadata. Useful for forensics ("what exactly was at offset X with all fields?") and as a building block for higher-level UIs.
- `termlink_agent_who_is` — fingerprint resolver. Given a `sender_id` (fingerprint hex), walks topic and returns `{sender_id, display_name, first_seen_ts, last_seen_ts, post_count}` — display_name from latest envelope where `metadata.display_name` is set, plus engagement summary. Useful for "who is this fingerprint?" in audit logs and for surfacing peers in operator-facing UIs.

Both pure walk + filter.

## Acceptance Criteria

### Agent
- [x] New `AgentEnvelopeParams` struct (offset u64)
- [x] New `AgentWhoIsParams` struct (sender_id String)
- [x] New `termlink_agent_envelope` walks topic + finds envelope at exact offset + base64-decodes payload
- [x] New `termlink_agent_who_is` walks topic + filters by sender_id + extracts latest display_name + first/last seen + post count
- [x] envelope returns null/error when offset not found
- [x] who_is returns post_count=0 + null timestamps when sender_id never seen
- [x] who_is extracts display_name from metadata.display_name when present
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=126 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_envelope` + `termlink_agent_who_is` are operator-fluent over MCP
  **Steps:**
  1. Pick any offset from `termlink_agent_recent`
  2. Call `termlink_agent_envelope` with that offset
  3. Verify the full payload + metadata is returned with decoded body
  4. Pick a sender_id and call `termlink_agent_who_is` with it
  5. Verify display_name + first/last seen + post count
  **Expected:** envelope hydrates one record fully; who_is resolves fingerprint to readable identity.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_envelope"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_who_is"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two identity/orientation primitives. envelope is the first single-offset deep-fetch — replaces multiple existing single-line previewers with full hydrated record (raw payload, decoded body, all metadata). who_is is the first fingerprint resolver — gives display_name + engagement summary for any sender_id, removing the need to manually correlate fingerprints across tools. Both pure walk + filter, ~80 LOC each. Brings session total to 11 waves, +22 read tools, mcp_tools 104→126.
**Evidence:**
- Build clean (4m 13s)
- `termlink version --json` reports mcp_tools=126 (was 124 after T-1589) — +2
- Verification gate 4/4 passed
- envelope: O(n) walk + offset match + base64-decode + full hydrate; who_is: O(n) walk + sender filter + min/max ts + latest display_name extraction

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

### 2026-05-05T19:51:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1590-termlinkagentenvelope--termlinkagentwhoi.md
- **Context:** Initial task creation
