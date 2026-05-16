---
id: T-1650
name: "remote profile list — surface bootstrap_from per profile + heal-readiness summary (T-1648 ergonomic family)"
description: >
  remote profile list — surface bootstrap_from per profile + heal-readiness summary (T-1648 ergonomic family)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [auth, fleet, ergonomic, discoverability]
components: []
related_tasks: [T-1648, T-1649, T-1291, T-1054, T-1055]
created: 2026-05-16T22:03:58Z
last_update: 2026-05-16T22:03:58Z
date_finished: null
---

# T-1650: remote profile list — surface bootstrap_from per profile + heal-readiness summary (T-1648 ergonomic family)

## Context

T-1648/T-1649 made the heal-time hint surface declarative `bootstrap_from`. But the configuration is invisible at *idle* time — operators reviewing their fleet via `termlink remote profile list` don't see which profiles have a declared trust anchor. So heal-readiness is only discoverable at incident time. Add a `HEAL` column (showing `auto` when declared, `-` otherwise) and a footer summary that counts profiles missing `bootstrap_from` and recommends declaration. Proactive ergonomic, mirrors the same heal-readiness signal at idle.

## Acceptance Criteria

### Agent
- [x] `remote profile list` table gains a `HEAL` column showing `auto` when `bootstrap_from` is declared, `-` otherwise (live smoke: 5/5 show `-`)
- [x] JSON output includes `bootstrap_from` field per profile (verified: `"bootstrap_from": null` present)
- [x] Footer summary line appears when N≥1 profiles lack `bootstrap_from`, naming the count and recommending declaration (live: "5 profile(s) lack `bootstrap_from` — declare with...")
- [x] When all profiles have `bootstrap_from` declared, footer summary is suppressed (test: heal_readiness_footer_silent_when_all_declared)
- [x] When no profiles configured, current empty-state output unchanged (early return before list logic)
- [x] Unit test: footer summary fires when ≥1 profile lacks declaration (test: heal_readiness_footer_fires_when_any_profile_undeclared)
- [x] Unit test: footer summary suppressed when all profiles declared (+ extra empty-fleet test)
- [x] `cargo build --release -p termlink` succeeds (5m58s)
- [x] `cargo test --release -p termlink --bin termlink heal_readiness` passes (3/3 ok)

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

cargo test --release -p termlink --bin termlink heal_readiness 2>&1 | tee /tmp/t1650-test.log | tail -5
grep -E "test result: ok|0 failed" /tmp/t1650-test.log

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

### 2026-05-16T22:03:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1650-remote-profile-list--surface-bootstrapfr.md
- **Context:** Initial task creation
