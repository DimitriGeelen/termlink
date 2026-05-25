---
id: T-1815
name: "self-audit guard: framework bus bridges must not invoke retired termlink primitives (G-019 prevention for T-1814)"
description: >
  self-audit guard: framework bus bridges must not invoke retired termlink primitives (G-019 prevention for T-1814)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-25T22:19:03Z
last_update: 2026-05-25T22:21:52Z
date_finished: 2026-05-25T22:21:52Z
---

# T-1815: self-audit guard: framework bus bridges must not invoke retired termlink primitives (G-019 prevention for T-1814)

## Context

G-019 prevention follow-up to T-1814. T-1814 fixed the framework's bus bridges
(pickup-channel-bridge.sh, publish-learning-to-bus.sh) falling back to the
legacy `event.broadcast` when channel.post fails — that fallback was the lone
live emitter resetting T-1166's 7-day cut clean-window gate, and the framework
was blind to it for *weeks* (only telemetry archaeology surfaced it). Mitigation
≠ prevention (CLAUDE.md G-019): add a structural check so the next bridge that
regresses to a retired primitive is caught at self-audit time. Home:
`agents/audit/self-audit.sh` (the framework's own integrity check — standalone,
WARN-level, non-blocking; NOT the 153KB consumer-pre-push audit.sh, per L-030
isolation). Must land upstream (channel-1) to protect all consumers.

## Acceptance Criteria

### Agent
- [x] `agents/audit/self-audit.sh` gains a "LAYER 6: BUS BRIDGE INTEGRITY" check
  that scans `lib/pickup-channel-bridge.sh` + `lib/publish-learning-to-bus.sh`
  (comment lines stripped) for actual invocations of retired termlink verbs
  (`event broadcast`, `inbox push|list|status|clear`, `file send|receive`) and
  emits WARN per offender, PASS when clean. WARN-level only. **Verified:**
  self-audit output shows `[PASS] Bus bridges use channel.* only`.
- [x] Check PASSES against the current (T-1814-fixed) bridges and WARNS against
  a deliberately-reverted copy. **Verified:** PASS on live run; temp-copy
  regression test detected a reverted `termlink event broadcast` invocation
  while NOT false-positiving on a comment mentioning "event.broadcast".
  `bash -n` clean.
- [x] G-019 gap registered as **G-061** in `.context/project/concerns.yaml`
  (weeks-long blindness to bridge-fallback-to-retired-primitive; T-1815 guard
  named as the shipped prevention; status `watching`). YAML validated (32
  concerns parse, G-061 present).
- [x] Edit lands upstream on `/opt/999-AEF` `origin/master` via channel-1
  (commit `7aa10645`). **Verified on remote:** ancestor of origin/master,
  LAYER 6 marker present, `bash -n` clean on the remote blob.

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
bash -n .agentic-framework/agents/audit/self-audit.sh
grep -q 'LAYER 6: BUS BRIDGE INTEGRITY' .agentic-framework/agents/audit/self-audit.sh
# isolate grep from self-audit's overall exit code (it may exit 1/2 on unrelated WARN/FAIL)
bash -c '.agentic-framework/agents/audit/self-audit.sh --quiet 2>&1 | grep -q "Bus bridges use channel"'

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

### 2026-05-25T22:19:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1815-self-audit-guard-framework-bus-bridges-m.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-2a370c38
- **Timestamp:** 2026-05-25T22:21:53Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** yes
- **Findings:** 1

**Per-AC findings:**

- **AC#3 (Agent)** — G-019 gap registered as **G-061** in `.context/project/concerns.yaml`
  - **AC-verify-mismatch** (narrow, heuristic) — `path=context/project/concerns.yaml in: G-019 gap registered as **G-061** in `.context/project/concerns.yaml``

- **Layer-1 escalations:** 1
  1. **external-publish** (high) — External publish or release
     - matched: `broadcast`

### 2026-05-25T22:21:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
