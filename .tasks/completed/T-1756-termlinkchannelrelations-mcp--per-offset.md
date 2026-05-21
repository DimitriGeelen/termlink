---
id: T-1756
name: "termlink_channel_relations MCP — per-offset relations on arbitrary topic"
description: >
  termlink_channel_relations MCP — per-offset relations on arbitrary topic

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-21T10:01:36Z
last_update: 2026-05-21T10:03:49Z
date_finished: 2026-05-21T10:03:49Z
---

# T-1756: termlink_channel_relations MCP — per-offset relations on arbitrary topic

## Context

Port `cmd_channel_relations` (channel.rs:6181) to MCP. Topic-flexible variant of `termlink_agent_relations` (T-1737, hardcoded to agent-chat-arc). Reuses existing `compute_relations_mcp` + `RelationItemMcp` + `RelationsReportMcp` helpers. Uses `walk_topic_full_mcp` (matches CLI + agent sister) because old target offsets need full-history reachability. T-1166 epic continuation.

## Acceptance Criteria

### Agent
- [x] `ChannelRelationsParams { topic: String, target: u64 }` params struct added
- [x] `termlink_channel_relations` `#[tool]` method added — walks the topic, refuses with `json_err` when target absent (matches CLI bail + agent_relations behavior), returns `{ok, topic, target_offset, target_sender, target_payload, replies, reactions, edits, redactions}`
- [x] 2 unit tests added (params shape) — helper coverage is shared with `compute_relations_mcp`/T-1737
- [x] `cargo build -p termlink-mcp` clean; 469 lib tests pass (467 → 469, +2)

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
cargo build -p termlink-mcp 2>&1 | grep -E "error\[|^error:" && exit 1 || true
cargo test -p termlink-mcp --lib channel_relations 2>&1 | tail -5 | grep -q "test result: ok"

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

### 2026-05-21T10:01:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1756-termlinkchannelrelations-mcp--per-offset.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-30fac0f6
- **Timestamp:** 2026-05-21T10:03:50Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T10:03:49Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
