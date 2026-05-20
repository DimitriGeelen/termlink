---
id: T-1662
name: "Document rotation-protocol detection verbs in CLAUDE.md"
description: >
  Document rotation-protocol detection verbs in CLAUDE.md

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-17T18:22:43Z
last_update: 2026-05-17T18:24:48Z
date_finished: 2026-05-17T18:24:48Z
---

# T-1662: Document rotation-protocol detection verbs in CLAUDE.md

## Context

The G-011 rotation-protocol diagnostic stack (T-1656/57/58/59/60/61) shipped end-of-segment 2026-05-17 but is undiscoverable from the operator's canonical reference — CLAUDE.md's "Hub Auth Rotation Protocol" section documents only the heal path verbs (`fleet reauth`, `--bootstrap-from`). Operators investigating rotation symptoms read that section and find no detection verbs. This task adds a "Detection — primitive verbs" subsection before the heal paths so the workflow reads: detect → identify → heal → re-pin.

PL-162 records the cert-vs-secret rotation coverage nuance; the docs section must reflect this so operators don't mistake `fleet verify` for a complete diagnostic (it is not — `fleet doctor` remains needed for secret-only rotation).

## Acceptance Criteria

### Agent
- [x] CLAUDE.md "Hub Auth Rotation Protocol" section gains a "Detection — primitive verbs" subsection listing each of the six verbs (`hub export-secret`, `hub fingerprint`, `hub probe`, `tofu verify`, `fleet verify`, `termlink_fleet_verify` MCP) with one-line purpose + exit-code semantics for the script-friendly verbs. **Verified 2026-05-17:** CLAUDE.md:118 table with all six rows ships; exit codes `0/1/2/3` and `--exit-on-drift-only` documented inline.
- [x] Subsection placed BEFORE the existing "Heal path — Tier-1" so operators read detect→identify→heal in document order. **Verified 2026-05-17:** Detection at line 118, Heal Tier-1 at line 144.
- [x] Cert-vs-secret rotation coverage note included (PL-162 nuance): `fleet verify` detects CERT rotation; `fleet doctor` remains required for SECRET-only rotation; PL-021 "both rotate" is covered by either. **Verified 2026-05-17:** "Coverage scope (PL-162)" paragraph at CLAUDE.md:135 with the secret-only carve-out and the operator disagreement-resolution rule.
- [x] Each verb's one-line purpose matches its actual `--help` surface (verified via `./target/release/termlink <verb> --help` cross-check). **Verified 2026-05-17:** All five CLI verbs cross-checked against `target/release/termlink --help` output; phrasings derive from each verb's clap long-about.

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
grep -q "Detection — primitive verbs" CLAUDE.md
grep -q "hub probe" CLAUDE.md
grep -q "tofu verify" CLAUDE.md
grep -q "fleet verify" CLAUDE.md
grep -q "termlink_fleet_verify" CLAUDE.md
grep -q "PL-162" CLAUDE.md

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

### 2026-05-17T18:22:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1662-document-rotation-protocol-detection-ver.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-8a3406b0
- **Timestamp:** 2026-05-17T18:24:49Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-17T18:24:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
