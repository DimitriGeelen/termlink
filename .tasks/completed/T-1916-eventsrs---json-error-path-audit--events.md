---
id: T-1916
name: "inbox commands --json error-path — cmd_inbox_status/clear/list bail without honoring json_output (T-1915 sibling)"
description: >
  Audit found events.rs already handles --json correctly at all 4 sites. Real bug is in infrastructure.rs cmd_inbox_status / cmd_inbox_clear / cmd_inbox_list — they take json_output: bool but bail with anyhow::bail!() on hub-down without checking the flag. T-1166 will retire these eventually, but until then `termlink inbox status --json | jq` is broken.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-01T17:22:36Z
last_update: 2026-06-01T17:22:36Z
date_finished: null
---

# T-1916: events.rs --json error-path audit — events/emit_to/subscribe (T-1915 sibling)

## Context

T-1915 introduced a helper for channel.rs's 45 sites. Auditing siblings revealed:

- **events.rs (4 sites)**: ALREADY correct — all 4 use `if !hub_socket.exists() { if json { json_error_exit(...); } anyhow::bail!(...); }`. Hand-rolled inline but functionally complete. Optional future DRY into helper, not a bug.
- **infrastructure.rs (3 sites)**: REAL BUG — `cmd_inbox_status`, `cmd_inbox_clear`, `cmd_inbox_list` accept `json_output: bool` but bail with `anyhow::bail!()` on hub-down without checking the flag. `termlink inbox status --json | jq` produces empty stdout. T-1166 will retire these eventually but they're still in the binary.

Approach: inline fix (same pattern as events.rs already uses). Not worth a helper because T-1166 retires these.

## Acceptance Criteria

### Agent
- [x] `cmd_inbox_status` (infrastructure.rs:1018): hub-down branch checks `json_output` and calls `super::json_error_exit({"ok":false,"error":...})` before bail.
- [x] `cmd_inbox_clear` (infrastructure.rs:1046): same fix; also added json_error_exit on the missing-target/--all branch (same shape).
- [x] `cmd_inbox_list` (infrastructure.rs:1080): same fix.
- [x] `cargo build -p termlink --release` succeeds (6m 21s).
- [x] Live smoke: all 3 commands now emit `{"error":"Hub is not running...","ok":false}` to stdout with exit 1 — pipes into `jq`. Tested against fresh tempdir runtime_dir with no hub.

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

cargo build -p termlink --release 2>&1 | tail -2 | grep -q "Finished"

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

## RCA

**Symptom:** `termlink inbox status --json | jq` (and `inbox clear --all --json`, `inbox list <target> --json`) produced silent empty stdout when the hub was down. Pipelines saw nothing parseable.

**Root cause:** Same shape as T-1915: `anyhow::bail!()` propagates the error before any branch on `json_output`. anyhow writes to stderr, binary exits 1 with no stdout.

**Why structurally allowed:** These commands carry a deprecation warning (T-1166 will retire them) and likely receive less testing scrutiny than first-class verbs. The events.rs siblings (`cmd_event_broadcast`, `cmd_event_emit_to`, etc.) all happen to have the correct inline pattern, masking the impression that the codebase as a whole had this right. T-1915's parity-harness coverage was scoped to channel.rs verbs, so the audit naturally moved next to events.rs/infrastructure.rs.

**Prevention:** Inline fix matches the established pattern in events.rs (same hand-rolled `if json { json_error_exit(...); } anyhow::bail!()` shape). No helper because T-1166 retires these commands. Live smoke documented in this task's commit message + ACs proves the fix; absence of regression test is intentional (sunset code).

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

### 2026-06-01T17:22:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1916-eventsrs---json-error-path-audit--events.md
- **Context:** Initial task creation
