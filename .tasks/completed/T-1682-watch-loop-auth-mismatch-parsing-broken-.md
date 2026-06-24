---
id: T-1682
name: "watch loop auth-mismatch parsing broken — T-1681 follow-up bug fix"
description: >
  watch loop auth-mismatch parsing broken — T-1681 follow-up bug fix

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/remote.rs]
related_tasks: []
created: 2026-05-18T06:04:38Z
last_update: 2026-05-18T06:09:00Z
date_finished: 2026-05-18T06:09:00Z
---

# T-1682: watch loop auth-mismatch parsing broken — T-1681 follow-up bug fix

## Context

T-1681 added an OR-gate in `cmd_fleet_doctor_watch` so `--auto-heal` fires on
EITHER cert-drift (`new_pin == "drift"`) OR secret-only rotation
(`new_state.0 == "auth-mismatch"`). But the JSON the watch loop parses is
emitted by `cmd_fleet_doctor`'s single-shot pass, which only ever writes
`"status": "ok" | "error" | "timeout"` on each hub object — `"auth-mismatch"`
is never written. The class is detected internally via `auth_mismatch_class(&msg)`
and used to feed the failure-streak tracking (T-1053), but does NOT leak into
the JSON. Net result: the `auth_mismatch_now` gate in T-1681 is dead code.

Fix: in the watch loop parser (~line 2924), derive an effective conn class —
if `status == "error"` AND `auth_mismatch_class(error_msg)` returns
`Some("auth-mismatch")`, surface `"auth-mismatch"` as the conn state instead
of `"error"`. Keep the JSON output unchanged (consumers may depend on the
existing literal); only the watch loop's in-memory classification gets
sharper.

## Acceptance Criteria

### Agent
- [x] Watch parser classifies `status="error"` + auth-mismatch error message as `conn="auth-mismatch"` in its in-memory state — via new `derive_watch_conn` helper
- [x] `cargo check -p termlink` passes (clean, only pre-existing unrelated warning in termlink-mcp)
- [x] `cargo test -p termlink fleet_learning_classifies_auth_errors` passes (no regression in existing classifier)
- [x] Unit test added: 5 cases covering auth-mismatch, tofu-violation, ok/timeout pass-through, generic-error fallback, missing fields — all pass
- [x] JSON output of `fleet doctor --json` UNCHANGED — only watch parser's in-memory state remaps; emitter at lines 3587/3612/3502 untouched

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

cargo check -p termlink 2>&1 | tail -5
cargo test -p termlink --bin termlink derive_watch_conn 2>&1 | tail -10
cargo test -p termlink --bin termlink fleet_learning_classifies_auth_errors 2>&1 | tail -10

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

## RCA

**Symptom:** T-1681 shipped a code path in `cmd_fleet_doctor_watch` that
gates auto-heal on `new_state.0 == "auth-mismatch"`. The gate never fires
because the watch parser reads `hub["status"]` which is only ever
"ok"/"error"/"timeout" — never "auth-mismatch". Effect: PL-162's continuous-
monitor closure claimed in T-1681 is dead code.

**Root cause:** Two layers using different conn-class vocabularies and
nobody bridged them. The single-shot pass emits a coarse status enum
("ok"/"error"/"timeout") in JSON, while the failure-tracking layer
uses a finer class ("auth-mismatch"/"tofu-violation") derived via
`auth_mismatch_class(&error_msg)`. The finer class lives only in
internal `maybe_track_fleet_failure` / `maybe_record_auth_mismatch_learning`
calls — it's not exposed in the JSON. T-1681 assumed the finer vocabulary
was already in the JSON without checking.

**Why structurally allowed:** T-1681's smoke test was a startup-clean check
(`--watch ... --auto-heal --include-pin-check` starts cleanly) — it never
exercised a live auth-mismatch transition. No unit test pinned the
JSON-derived conn-state vocabulary. Two adjacent layers diverging in
naming is the kind of silent contract drift that only shows up under
actual operational stress.

**Prevention:** Unit test added that pins the watch parser's classification
behavior given a synthetic JSON doc. Future refactors of the conn-state
vocabulary will trip the test if the watch parser's mapping isn't kept
in sync. Also a learning to capture: don't add a gate referencing a
literal without verifying the literal is actually produced somewhere
the parser sees it.

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

### 2026-05-18T06:04:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1682-watch-loop-auth-mismatch-parsing-broken-.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-48e796f5
- **Timestamp:** 2026-05-18T06:09:24Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-18T06:09:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
