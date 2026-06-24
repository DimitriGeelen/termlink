---
id: T-1751
name: "termlink_channel_edits_of MCP — chronological edit history for a target offset"
description: >
  termlink_channel_edits_of MCP — chronological edit history for a target offset

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-21T08:21:46Z
last_update: 2026-05-21T08:25:26Z
date_finished: 2026-05-21T08:25:26Z
---

# T-1751: termlink_channel_edits_of MCP — chronological edit history for a target offset

## Context

Mirror `cmd_channel_edits_of` + `compute_edits_of` + `EditRow` + `EditsOfReport`
(channel.rs:6902/6964) to MCP. Given a content offset, returns the original
post plus the chronological list of edits that replace it. Distinct from
`compute_state*` (which collapses to latest-wins) — `edits_of` is the *full
history*. Use case: audit how a message evolved, regression diagnosis,
conversation forensics. Returns None when target is missing or redacted
out (matches CLI behavior).

## Acceptance Criteria

### Agent
- [x] `EditRowMcp { offset, sender_id, ts_ms, payload }` + `to_json_mcp`.
- [x] `EditsOfReportMcp { original: EditRowMcp, edits: Vec<EditRowMcp> }` + `to_json_mcp` (shape: `{original, edits}`).
- [x] Pure helper `compute_edits_of_mcp(envelopes, target: u64) -> Option<EditsOfReportMcp>`. Returns `None` if target missing or in `redacted_offsets_mcp` set. Edits sorted ts_ms asc then offset asc tiebreak. Filters: non-numeric metadata.replaces ignored, redacted edit offsets dropped, edits targeting other offsets ignored.
- [x] `ChannelEditsOfParams { topic, target: u64 }`.
- [x] `#[tool] termlink_channel_edits_of` returns `{ok, topic, target, found: bool, report?: {original, edits}}` (or `{ok, topic, target, found: false}` when target missing/redacted).
- [x] Tests cover: empty input, target missing, target present with no edits, target present with multiple edits sorted by ts then offset, redacted target → None, edit pointing at OTHER target excluded, edit referencing missing target ignored, malformed metadata.replaces ignored, edit that is itself redacted dropped.
- [x] `cargo build -p termlink-mcp` clean.
- [x] `cargo test -p termlink-mcp` passes.

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

### 2026-05-21T08:21:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1751-termlinkchanneleditsof-mcp--chronologica.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-82d38678
- **Timestamp:** 2026-05-21T08:25:42Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T08:25:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
