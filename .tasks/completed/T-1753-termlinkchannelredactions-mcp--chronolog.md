---
id: T-1753
name: "termlink_channel_redactions MCP — chronological redaction audit log"
description: >
  termlink_channel_redactions MCP — chronological redaction audit log

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-21T08:30:52Z
last_update: 2026-05-21T08:33:44Z
date_finished: 2026-05-21T08:33:44Z
---

# T-1753: termlink_channel_redactions MCP — chronological redaction audit log

## Context

Mirror `cmd_channel_redactions` + `compute_redactions` + `RedactionRow`
(channel.rs:5460/5491/5529) to MCP. Returns chronological redaction events
for an arbitrary topic. Each row carries event offset, target offset,
redactor sender, optional reason, ts, and a target_payload preview when
the target is in the snapshot. Use case: moderation audit — what was
retracted, by whom, when, and why.

## Acceptance Criteria

### Agent
- [x] `RedactionRowMcp { event_offset, target_offset, redactor_sender, reason: Option<String>, ts_ms, target_payload: Option<String> }` + `to_json_mcp`.
- [x] Pure helper `compute_redactions_mcp(envelopes) -> Vec<RedactionRowMcp>` — walks msg_type=redaction envelopes with parseable metadata.redacts, captures optional metadata.reason, sender_id (default "?"), sorts event_offset asc. target_payload via decode_payload_lossy_mcp when target offset is in envelope set.
- [x] `ChannelRedactionsParams { topic }`.
- [x] `#[tool] termlink_channel_redactions` returns `{ok, topic, rows, count}`.
- [x] Tests cover: empty input, single redaction with reason, single redaction without reason, multiple redactions sorted event_offset asc, malformed (missing redacts / non-numeric) silently skipped, non-redaction msg_types ignored, target_payload populated when target present and None otherwise.
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

### 2026-05-21T08:30:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1753-termlinkchannelredactions-mcp--chronolog.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-e42e8307
- **Timestamp:** 2026-05-21T08:34:00Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T08:33:44Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
