---
id: T-1764
name: "termlink_channel_digest MCP — period summary with reactions/pins/forwards on arbitrary topic (T-1166 wedge)"
description: >
  Port channel digest CLI verb to MCP. Major value-add over agent_digest: includes top_reactions, pins_added/removed, forwards_in, content-only posts count, recent_chats with payload — agent_digest has none of these.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [termlink, mcp, t-1166]
components: []
related_tasks: []
created: 2026-05-21T12:34:06Z
last_update: 2026-05-21T12:37:33Z
date_finished: 2026-05-21T12:37:33Z
---

# T-1764: termlink_channel_digest MCP — period summary with reactions/pins/forwards on arbitrary topic (T-1166 wedge)

## Context

T-1166 MCP-parity wedge. CLI verb `cmd_channel_digest` (channel.rs:4282) uses pure helper `compute_digest` (channel.rs:4169) returning `DigestSummary { since_ms, posts, distinct_senders, top_senders, top_reactions, pins_added, pins_removed, forwards_in, recent_chats }`. The MCP side has `termlink_agent_digest` (tools.rs:15176) but with a stripped shape — `{since_ts, total_in_window, by_msg_type, top_senders, latest_5_offsets}` — missing top_reactions, pins, forwards_in, content-only posts, and rendered recent chats.

Major value-adds:
1. Topic-flexible (any DM/topic)
2. Content-only `posts` count (vs agent's `total_in_window` which includes meta)
3. `top_reactions` — emoji aggregate over the window
4. `pins_added` / `pins_removed` — curation event counts
5. `forwards_in` — cross-topic ingestion count
6. `recent_chats` with payload + sender + ts — actionable summary, not just offsets

## Acceptance Criteria

### Agent
- [x] `DigestSummaryMcp { since_ms, posts, distinct_senders, top_senders, top_reactions, pins_added, pins_removed, forwards_in, recent_chats }` struct
- [x] `DigestChatMcp { offset, sender_id, ts, payload }` struct + `to_json_mcp`
- [x] `compute_digest_mcp(envelopes, since_ms)` pure helper mirrors CLI `compute_digest` semantics
- [x] Top-3 caps on top_senders and top_reactions; recent_chats takes last 3 by offset asc
- [x] `ChannelDigestParams { topic, since_mins?, since_ms? }` — since_ms takes precedence, default last-60-min
- [x] `termlink_channel_digest` tool method uses `walk_topic_full_mcp` + `compute_digest_mcp`
- [x] Returns full digest shape with all 9 metrics + recent_chats
- [x] 8 helper unit tests: empty, window filter, content/meta split, top-senders, pins, top-reactions, forwards_in, recent_chats
- [x] Plus 2 params tests — all pass
- [x] `cargo build -p termlink-mcp` clean (only pre-existing cur_run_end warning)

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

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
cd /opt/termlink && cargo build -p termlink-mcp 2>&1 | tail -3 | grep -qE "Finished|warning" && echo OK
cd /opt/termlink && cargo test -p termlink-mcp digest 2>&1 | tail -5 | grep -q "test result: ok" && echo TESTS_OK

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

### 2026-05-21T12:34:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1764-termlinkchanneldigest-mcp--period-summar.md
- **Context:** Initial task creation

### 2026-05-21T12:34:44Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.4)

- **Scan ID:** R-69c69326
- **Timestamp:** 2026-05-21T12:37:33Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T12:37:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
