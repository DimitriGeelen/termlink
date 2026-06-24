---
id: T-2001
name: "Capture cycle-12 as reusable pattern — learning + doc for paged-ranked-filtered-projected tool registry API"
description: >
  Capture cycle-12 as reusable pattern — learning + doc for paged-ranked-filtered-projected tool registry API

status: work-completed
workflow_type: refactor
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-06-05T17:03:51Z
last_update: 2026-06-05T17:03:51Z
date_finished: 2026-06-05T17:08:12Z
---

# T-2001: Capture cycle-12 as reusable pattern — learning + doc for paged-ranked-filtered-projected tool registry API

## Context

T-1984..T-2000 shipped 8 axes on `termlink_help` (limit, offset, parameter_required_count, sort_by, default-mode-bulk-flat-routing, fields projection, categories array, exclude_categories array) in one session. The slice-by-slice mechanics are now consistent enough to extract a reusable pattern for future tool-registry expansions.

Capture: (1) a learning that names the pattern + its mechanics, (2) a doc at `docs/operations/mcp-help-registry.md` that explains the paged-ranked-filtered-projected API to operators + LLM-client authors.

## Acceptance Criteria

### Agent
- [x] New learning PL-202 appended to `.context/project/learnings.yaml` capturing the cycle-12 slice pattern: opt-in `Option<T>` param → HelpParams field → signature grow → bulk caller patch (deterministic Python depth-tracking script) → applied in name_filter + bulk-flat branches → emit envelope `*_applied` / `*_unknown` for input validation → drift-table token + macro-doc reference → 7-11 invariant tests per slice
- [x] Learning explicitly names the seven shape-extension axes shipped (limit, offset, sort_by, fields, categories, exclude_categories, parameter_required_count) AND the route-extension axis (bulk-flat-listing routing for no-needle paging)
- [x] Learning includes the canonical cold-start LLM call composing all axes
- [x] Learning notes the backcompat invariant: every new axis MUST keep envelope shape unchanged when the param is unset (verified by existing tests staying green at each slice)
- [x] New doc `docs/operations/mcp-help-registry.md` written for operators + LLM-client authors covering: the seven axes, the canonical cold-start call, the bulk-flat-listing trigger, the envelope validation fields (`*_applied` / `*_unknown` pattern), and the backcompat invariant
- [x] Doc cross-references the eight T-IDs (T-1984/1994/1995/1996/1997/1998/1999/2000) so future readers can find the slice commits
- [x] No source code changes — this is pure capture
- [x] `python3 -c "import yaml; yaml.safe_load(open('.context/project/learnings.yaml'))"` passes (catches escape errors that broke PL-200 update in T-1989)

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

python3 -c "import yaml; yaml.safe_load(open('.context/project/learnings.yaml'))"
test -f docs/operations/mcp-help-registry.md
grep -q "PL-202" .context/project/learnings.yaml

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

### 2026-06-05T17:03:51Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2001-capture-cycle-12-as-reusable-pattern--le.md
- **Context:** Initial task creation
