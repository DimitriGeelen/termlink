---
id: T-1724
name: "Capture PL-175: runtime-signature-alone RCA is unsafe (T-1695 learning)"
description: >
  Capture PL-175: runtime-signature-alone RCA is unsafe (T-1695 learning)

status: captured
workflow_type: build
owner: agent
horizon: next
tags: []
components: []
related_tasks: []
created: 2026-05-20T10:18:04Z
last_update: 2026-05-20T10:18:42Z
date_finished: null
---

# T-1724: Capture PL-175: runtime-signature-alone RCA is unsafe (T-1695 learning)

## Context

T-1695 cost ~2 hours of diagnostic loop on the wrong root cause. The OneDev → GitHub mirror failure was diagnosed as "PAT missing Workflows scope" → "cache issue" → "v2 PAT" → "permission edit" → "regenerate" → all on a problem that did not exist at the OneDev auth layer. The **actual** root cause was an executor mismatch (`penelope-shell` executor on a build node that couldn't satisfy it) — visible only in the OneDev UI build log line `10:32:17 Pending resource allocation...`. The REST API had no log endpoint, so without UI access there was no diagnostic. Runtime signature alone (job stuck, no error) is identical between "executor unavailable" and "auth-401" failure modes.

This task captures PL-175 as a structural learning so the next agent doesn't repeat the loop.

## Acceptance Criteria

### Agent
- [ ] PL-175 entry added to `.context/project/learnings.yaml` with: date_observed=2026-05-20, source_task=T-1695, source_pattern=P-001 (diagnostic discipline)
- [ ] Body covers: (a) symptom — job hung with no error output, (b) wrong RCA path taken, (c) actual RCA — executor mismatch visible only in UI build log, (d) rule — when REST API has no log endpoint AND runtime signature alone is the only signal, INSIST on UI logs before forming hypotheses
- [ ] Linked from T-1695 final Updates entry (so future agents reading the task arrive at the learning)

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

### 2026-05-20T10:18:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1724-capture-pl-175-runtime-signature-alone-r.md
- **Context:** Initial task creation

### 2026-05-20T10:18:42Z — status-update [task-update-agent]
- **Change:** status: started-work → captured
- **Change:** horizon: now → next
