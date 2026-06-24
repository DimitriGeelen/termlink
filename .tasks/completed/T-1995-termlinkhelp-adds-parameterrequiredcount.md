---
id: T-1995
name: "termlink_help adds parameter_required_count — cost-aware ranking signal (cycle 12 slice 3)"
description: >
  termlink_help adds parameter_required_count — cost-aware ranking signal (cycle 12 slice 3)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-06-05T12:35:45Z
last_update: 2026-06-05T12:35:45Z
date_finished: 2026-06-05T14:05:29Z
---

# T-1995: termlink_help adds parameter_required_count — cost-aware ranking signal (cycle 12 slice 3)

## Context

Cycle 12 slice 3 (predecessor: T-1972 parameter_count, T-1984+T-1994 limit/offset). Today `parameter_count` tells an LLM "this tool has N params" — but a tool with 12 params where 2 are required and 10 are optional is much cheaper to call than one with 4 all-required. `parameter_required_count` carries the required-arity (sum of `optional==false` params) so LLM clients can rank by call-cost. Source: tool_params() already carries `{name, type, optional, doc}` per param (T-1953); this slice exposes the derived count without changing storage.

## Acceptance Criteria

### Agent
- [x] `parameter_required_count` added to name_filter match rows (== count of params where `optional==false`)
- [x] `parameter_required_count` added to default-mode rows (mirrors name_filter shape — T-1972 consistency invariant)
- [x] `parameter_required_count` added to tool_detail envelope (mirrors `parameter_count` line for at-a-glance arity)
- [x] `parameter_required_count` added to essentials rows (each starter-set tool carries the cost signal)
- [x] Field value always satisfies `0 <= parameter_required_count <= parameter_count` (derived invariant)
- [x] A zero-arity tool emits `parameter_required_count=0` (canonical zero-config case)
- [x] A tool whose params are all optional emits `parameter_required_count=0` even when `parameter_count > 0`
- [x] A tool with mixed required/optional params returns the correct count (verified against a known fixture)
- [x] Macro description string updated to document `parameter_required_count` semantics
- [x] Drift test `required_fields` table gains `("parameter_required_count", "T-1995")`
- [x] Invariant tests added: name_filter-presence, default-mode-presence, tool_detail-presence, essentials-presence, zero-arity, all-optional-case, bound-invariant (<= parameter_count)
- [x] `cargo test -p termlink-mcp --lib` passes (778 baseline → 785+, 0 failed)

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
set -o pipefail; cargo test -p termlink-mcp --lib 2>&1 | tail -5 | grep -q "test result: ok"

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

### 2026-06-05T12:35:45Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1995-termlinkhelp-adds-parameterrequiredcount.md
- **Context:** Initial task creation
