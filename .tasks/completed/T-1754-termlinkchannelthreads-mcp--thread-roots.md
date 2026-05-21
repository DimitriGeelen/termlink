---
id: T-1754
name: "termlink_channel_threads MCP — thread roots index with reply counts"
description: >
  termlink_channel_threads MCP — thread roots index with reply counts

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-21T08:35:06Z
last_update: 2026-05-21T08:40:25Z
date_finished: 2026-05-21T08:40:25Z
---

# T-1754: termlink_channel_threads MCP — thread roots index with reply counts

## Context

Mirror `cmd_channel_threads` + `compute_threads_index` + `ThreadIndexRow`
(channel.rs:7010/7047/7138) to MCP. Indexes every thread root on an
arbitrary topic with reply_count, distinct participants, last_ts, and
root payload preview. Sorted by recency (last_ts desc). Filters: redacted
roots dropped, redacted replies don't count toward reply_count/participants,
threads with zero non-redacted replies dropped, non-numeric `in_reply_to`
ignored. Optional `top` parameter for top-N most-recent. Use case: agent
navigation, fleet activity dashboards, thread digest.

## Acceptance Criteria

### Agent
- [x] Reuses existing `ThreadIndexRowMcp`, `compute_threads_index_mcp`, `parent_offset_of_mcp` from T-1732 (agent_threads port) — these helpers were already MCP-ported for the chat-arc tool; channel_threads is genuinely additive (topic-parametrized + `top` truncation).
- [x] `ChannelThreadsParams { topic, top: Option<u64> }` — when `top` set, returns first N rows post-sort (matches CLI `--top` semantics).
- [x] `#[tool] termlink_channel_threads` returns `{ok, topic, top, rows, count}`. Tool method delegates to existing helper, applies optional top truncation post-sort.
- [x] Tests: params deserialization (minimal + with-top). Helper-level semantics (BFS, redaction filtering, transitive counting, participants distinct, last_ts sorting) are covered by the existing T-1732 test suite at `agent_threads_*` tests.
- [x] `cargo build -p termlink-mcp` clean.
- [x] `cargo test -p termlink-mcp` passes (454 → 456, +2 net new tests).

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
cargo build -p termlink-mcp
cargo test -p termlink-mcp --quiet

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

### 2026-05-21T08:35:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1754-termlinkchannelthreads-mcp--thread-roots.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-2b68358b
- **Timestamp:** 2026-05-21T08:40:41Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T08:40:25Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
