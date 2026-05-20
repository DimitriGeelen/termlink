---
id: T-1724
name: "Capture PL-175: runtime-signature-alone RCA is unsafe (T-1695 learning)"
description: >
  Capture PL-175: runtime-signature-alone RCA is unsafe (T-1695 learning)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-20T10:18:04Z
last_update: 2026-05-20T18:46:16Z
date_finished: 2026-05-20T18:46:16Z
---

# T-1724: Capture PL-175: runtime-signature-alone RCA is unsafe (T-1695 learning)

## Context

T-1695 cost ~2 hours of diagnostic loop on the wrong root cause. The OneDev → GitHub mirror failure was diagnosed as "PAT missing Workflows scope" → "cache issue" → "v2 PAT" → "permission edit" → "regenerate" → all on a problem that did not exist at the OneDev auth layer. The **actual** root cause was an executor mismatch (`penelope-shell` executor on a build node that couldn't satisfy it) — visible only in the OneDev UI build log line `10:32:17 Pending resource allocation...`. The REST API had no log endpoint, so without UI access there was no diagnostic. Runtime signature alone (job stuck, no error) is identical between "executor unavailable" and "auth-401" failure modes.

This task captures PL-175 as a structural learning so the next agent doesn't repeat the loop.

## Acceptance Criteria

### Agent
- [x] PL-175 entry added to `.context/project/learnings.yaml` with: date_observed=2026-05-20, source_task=T-1695, source_pattern=P-001 (diagnostic discipline)
- [x] Body covers: (a) symptom — job hung with no error output, (b) wrong RCA path taken, (c) actual RCA — executor mismatch visible only in UI build log, (d) rule — when REST API has no log endpoint AND runtime signature alone is the only signal, INSIST on UI logs before forming hypotheses
- [x] Linked from T-1695 final Updates entry (so future agents reading the task arrive at the learning)

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

**Symptom:** T-1695 burned ~2 hours diagnosing OneDev → GitHub mirror failure as PAT-scope / cache / token-version issues, when the actual cause was OneDev executor config (`penelope-shell` on a build node that couldn't satisfy it).

**Root cause:** The agent formed RCA hypotheses from runtime signature alone (`git push` returns success, no GitHub commits appear). That signature is identical for both "OneDev auth-401 to GitHub" and "OneDev build job stuck Pending resource allocation". Without UI log access (the REST API has no build-log endpoint on this OneDev version), no observable signal distinguishes the two — so hypotheses were unfalsifiable until the operator pasted UI log content.

**Why structurally allowed:** No protocol existed for "form RCA hypothesis only after reading authoritative log." The diagnostic ladder didn't include "first, demand log access" as a step. CLAUDE.md's Hypothesis-Driven Debugging rule (3-hypothesis cap before escalation) addresses *quantity* of hypotheses but not the *evidence quality* required to form them.

**Prevention:** PL-175 codifies the rule. Apply at the start of any RCA flow where the only observable is "job stuck, no error" — stop, identify the missing log source, demand operator-paste (or UI access via Playwright/screenshot) before forming hypotheses. This is distinct from the fix itself (T-1695 closed with executor pin); PL-175 is the learning that prevents the next 2-hour theory-chase.

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

### 2026-05-20T18:44:43Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.4)

- **Scan ID:** R-8e3496aa
- **Timestamp:** 2026-05-20T18:46:17Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#1 (Agent)** — PL-175 entry added to `.context/project/learnings.yaml` with: date_observed=2026-05-20, source_task=T-1695, source_pattern=P-001 (diagnostic discipline)
  - **AC-verify-mismatch** (narrow, heuristic) — `path=context/project/learnings.yaml in: PL-175 entry added to `.context/project/learnings.yaml` with: date_observed=2026-05-20, source_task=T-1695, source_pattern=P-001 (diagnostic disciplin`

### 2026-05-20T18:46:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** PL-175 captured to learnings.yaml; linked from T-1695 Updates; RCA section filled
