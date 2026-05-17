---
id: T-1681
name: "Extend --auto-heal to fire on conn=auth-mismatch transitions — close PL-162 secret-only rotation gap"
description: >
  Extend --auto-heal to fire on conn=auth-mismatch transitions — close PL-162 secret-only rotation gap

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-17T22:39:06Z
last_update: 2026-05-17T22:39:06Z
date_finished: null
---

# T-1681: Extend --auto-heal to fire on conn=auth-mismatch transitions — close PL-162 secret-only rotation gap

## Context

T-1680 shipped `fleet doctor --watch --auto-heal` but gated only on `new_pin == "drift"` — i.e. cert rotation. PL-162 explicitly documents the gap: secret-only rotation (HMAC secret regenerated while TLS cert unchanged — e.g. partial persist-if-present landing where `hub.cert.pem` survived but `hub.secret` did not) is invisible to TLS probes. However, the watch loop's existing `hub.status` JSON DOES expose `"auth-mismatch"` when the hub rejects the auth token (remote.rs:4645), so the transition `conn: ok → auth-mismatch` is observable in the same diff cycle.

Extend the auto-heal gate to fire on EITHER cert-drift OR a new `conn → auth-mismatch` transition. Same heal action (`fleet reauth $hub --bootstrap-from auto` — the bootstrap fetch produces the new secret, which is what's needed for either rotation type). Same R2 — declared `bootstrap_from` required.

Result: `--auto-heal` now covers the full rotation-detection matrix from PL-021 / PL-162 (cert-only, secret-only, both).

## Acceptance Criteria

### Agent
- [x] Auto-heal fires on `new_pin == "drift"` (existing T-1680 behavior preserved)
- [x] Auto-heal also fires on `new_conn == "auth-mismatch"` AND `old_conn != "auth-mismatch"` (new path)
- [x] Both gates share the R2 declared-bootstrap_from check (skip with same hint when absent)
- [x] Both gates fire `fire_auto_heal` once per change cycle (deduplicated — combined OR-gate with single fire site)
- [x] `cargo check --workspace` passes
- [x] CLAUDE.md detection-verbs row for `--auto-heal` updated to note BOTH coverage paths; new "Coverage of --auto-heal (T-1681)" paragraph added under PL-162 section
- [x] Live smoke: --watch --auto-heal --include-pin-check starts cleanly without panic, baseline cycle begins as expected

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

### 2026-05-17T22:39:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1681-extend---auto-heal-to-fire-on-connauth-m.md
- **Context:** Initial task creation
