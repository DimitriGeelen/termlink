---
id: T-1655
name: "Tag 12 genuinely-standalone fabric cards to clear recurring 8x-WARN"
description: >
  Tag 12 genuinely-standalone fabric cards to clear recurring 8x-WARN

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-17T13:43:59Z
last_update: 2026-05-17T13:51:55Z
date_finished: 2026-05-17T13:51:55Z
---

# T-1655: Tag 12 genuinely-standalone fabric cards to clear recurring 8x-WARN

## Context

Audit WARN "Fabric: 12/122 cards have no edges" has fired 8 times in 14 days (trend analysis flagged it as practice candidate). The mitigation "Run: fw fabric enrich" is a no-op for these 12 — `fw fabric enrich --dry-run` finds 0 edges to add. The cards are genuinely standalone artifacts (install.sh, watchdog.sh, test scripts, transcripts, build pseudo-components) with no source-tree dependencies that fabric's import-detector can find.

The audit's edge-counter (`audit.sh:658`) already supports `standalone: true` opt-out — tagged cards are excluded from the count. This is the structural fix: data-side annotation, not framework code change.

## Acceptance Criteria

### Agent
- [x] All 12 zero-edge cards inspected; each classified as genuinely-standalone (all 12 had no source-tree imports — they are installer/operational/test/build-script artifacts)
- [x] Tagged cards have `standalone: true` field added (boolean, top-level YAML)
- [x] `fw audit` re-run; "Fabric: N/122 cards have no edges" WARN no longer fires — now `[PASS] Fabric edges: 110/110 cards enriched (0 without edges)`
- [x] Cards tagged standalone are documented in their `purpose` field (10 had TODO placeholders; all updated with concrete one-line descriptions)
- [x] Commit with T-1655 prefix (7987f1f0)

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
.agentic-framework/bin/fw audit 2>&1 | grep -E "Fabric: [0-9]+/[0-9]+ cards have no edges" && exit 1 || exit 0
python3 -c "import yaml,glob; [yaml.safe_load(open(f)) for f in glob.glob('.fabric/components/*.yaml')]"

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

### 2026-05-17T13:43:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1655-tag-12-genuinely-standalone-fabric-cards.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-13f7860b
- **Timestamp:** 2026-05-17T13:54:04Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-17T13:51:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
