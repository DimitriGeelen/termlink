---
id: T-1767
name: "termlink_channel_reactions_on MCP — per-target reaction rollup with correct repeat-tap semantics (T-1166 wedge)"
description: >
  Port channel reactions-on CLI verb to MCP. Value-add over agent_reaction_summary: topic-flexible + respects redactions + correct count semantics (CLI counts every reaction; agent_reaction_summary counts only distinct senders).

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [termlink, mcp, t-1166]
components: []
related_tasks: []
created: 2026-05-21T12:44:43Z
last_update: 2026-05-21T12:47:45Z
date_finished: 2026-05-21T12:47:45Z
---

# T-1767: termlink_channel_reactions_on MCP — per-target reaction rollup with correct repeat-tap semantics (T-1166 wedge)

## Context

T-1166 MCP-parity wedge. CLI verb `cmd_channel_reactions_on` (channel.rs:5643) uses pure helper `compute_reactions_on` (channel.rs:5594) returning `ReactionsOnRow { emoji, count, senders }` for a target offset. The CLI explicitly counts repeat taps (alice 👍 twice = 2) while keeping `senders` deduplicated. Existing MCP `termlink_agent_reaction_summary` (tools.rs:17872) shares the same JSON shape but has TWO subtle bugs:
1. No redaction handling — silently includes retracted reactions
2. `count` is set to `senders.len()` (distinct senders), not actual reaction count — diverges from CLI semantics

Value-adds:
1. Topic-flexible (any DM/topic)
2. Correct count semantics (CLI parity — repeat tapping counts)
3. Respects redactions

## Acceptance Criteria

### Agent
- [x] `ReactionsOnRowMcp { emoji, count, senders: Vec<String> }` struct + `to_json_mcp`
- [x] `compute_reactions_on_mcp(envelopes, target_offset)` pure helper mirrors CLI: count=total reactions (repeat-tap), senders=deduplicated set sorted asc, skip redacted, drop empty-emoji and missing-parent
- [x] Sort: count desc, emoji asc
- [x] `ChannelReactionsOnParams { topic, target }` struct
- [x] `termlink_channel_reactions_on` tool method
- [x] Returns `{ok, topic, target, total_count, rows: [...]}` with computed total_count
- [x] 8 helper unit tests: empty, single, repeat-tap, redaction skip, multi-emoji sort, other-target ignored, empty-emoji drop, row JSON shape
- [x] Plus 1 params test (deserialize) — all pass
- [x] `cargo build -p termlink-mcp` clean

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
cd /opt/termlink && cargo test -p termlink-mcp reactions_on 2>&1 | tail -5 | grep -q "test result: ok" && echo TESTS_OK

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

### 2026-05-21T12:44:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1767-termlinkchannelreactionson-mcp--per-targ.md
- **Context:** Initial task creation

### 2026-05-21T12:45:18Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.4)

- **Scan ID:** R-05fbb1ca
- **Timestamp:** 2026-05-21T12:47:46Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T12:47:45Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
