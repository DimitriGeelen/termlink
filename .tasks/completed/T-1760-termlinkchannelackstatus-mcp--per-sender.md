---
id: T-1760
name: "termlink_channel_ack_status MCP — per-sender lag with non-ackers surfaced"
description: >
  termlink_channel_ack_status MCP — per-sender lag with non-ackers surfaced

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-21T10:16:18Z
last_update: 2026-05-21T10:18:51Z
date_finished: 2026-05-21T10:18:51Z
---

# T-1760: termlink_channel_ack_status MCP — per-sender lag with non-ackers surfaced

## Context

Port `cmd_channel_ack_status` (channel.rs:7266) + `compute_ack_status` (channel.rs:7216) to MCP. Value-add over existing `termlink_agent_ack_status` (T-1539, hardcoded chat-arc): includes non-ackers (lag=latest+1) AND lag field per sender, plus `pending_only` filter. Critical for coordination — answers "who's behind and by how much". Topic-flexible. T-1166 epic continuation.

## Acceptance Criteria

### Agent
- [x] `AckStatusRowMcp { sender_id, up_to: Option<u64>, latest, lag, receipt_ts }` struct + `compute_ack_status_mcp(envelopes, receipts, latest_offset)` helper added (members = union of senders + receipt-only; non-ackers get up_to=None and lag=latest+1; sort lag desc, sender asc tiebreak)
- [x] `ChannelAckStatusParams { topic: String, pending_only: Option<bool> }` params struct added
- [x] `termlink_channel_ack_status` `#[tool]` method added — walks topic, derives receipts inline (sender → max up_to), computes rows, filters by pending_only when true, returns `{ok, topic, latest_offset, rows, count}`
- [x] 9 unit tests covering: empty topic, single caught-up, non-acker (full unread), mixed lag sort, sender asc tiebreak on equal lag, receipt-only sender, ack-ahead clamping, params with/without pending_only
- [x] `cargo build -p termlink-mcp` clean; 497 lib tests pass (488 → 497)

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
cargo test -p termlink-mcp --lib ack_status_mcp 2>&1 | tail -5 | grep -q "test result: ok"

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

### 2026-05-21T10:16:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1760-termlinkchannelackstatus-mcp--per-sender.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-f9cb2139
- **Timestamp:** 2026-05-21T10:19:09Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T10:18:51Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
