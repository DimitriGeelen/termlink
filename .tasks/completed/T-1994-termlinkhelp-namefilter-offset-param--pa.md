---
id: T-1994
name: "termlink_help name_filter offset param — pagination cursor (cycle 12 slice 2)"
description: >
  termlink_help name_filter offset param — pagination cursor (cycle 12 slice 2)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-05T12:10:34Z
last_update: 2026-06-05T12:10:34Z
date_finished: 2026-06-05T12:40:42Z
---

# T-1994: termlink_help name_filter offset param — pagination cursor (cycle 12 slice 2)

## Context

Cycle 12 slice 2 (predecessor: T-1984 limit param). Pairs `offset` with the
existing `limit` cap to complete `name_filter` into a real paginated API.
Without `offset`, `limit=10` can only ever show the first 10 hits of a
100+ match query — no way to advance. With `offset`, LLM clients walk the
match window: `offset=0,limit=10` → page 1; `offset=10,limit=10` → page 2;
…until `next_offset` is absent (signaling exhaustion). Stable order is
guaranteed by the registry iteration that T-1984 documented.

## Acceptance Criteria

### Agent
- [x] `offset: Option<usize>` field added to `HelpParams` with rmcp schemars doc comment, mirrors `limit` styling
- [x] `build_help_json` signature accepts `offset: Option<usize>` (signature grows 12 → 13 args)
- [x] `offset` is applied ONLY in the `name_filter` branch (other modes ignore it, like `limit`)
- [x] Order of operations: gather all matches → record `total_matched` (pre-window) → skip first `offset` → apply `limit` truncation
- [x] When `offset` is set: result carries `offset` (echo, integer) so client sees what window it got
- [x] When `offset + matches.len() < total_matched`: result carries `next_offset` (integer == offset + matches.len()) so clients can advance without arithmetic
- [x] When at end of window (no more matches beyond current window): `next_offset` is omitted (treat absence as "done")
- [x] `offset >= total_matched` returns empty matches[] (paginated past the end), still surfaces `total_matched` so client sees overshoot
- [x] When `offset` is unset: result does NOT carry `offset` or `next_offset` fields (backward compatibility — same as T-1984's limit-unset shape)
- [x] Composes with `limit`: `offset=0,limit=10` and `offset=10,limit=10` together cover the first 20 matches without overlap or gap
- [x] Composes with `exclude_deprecated` / `min_parameters` / `max_parameters`: filters run BEFORE offset/limit, so the page window slices the filtered set
- [x] Invariant tests added: offset-skips-n, offset-plus-limit-pages-cover, offset-past-end-empty, next_offset-presence-at-window-end, next_offset-absent-at-end, offset-composes-with-arity-filter, offset-unset-omits-fields-backcompat
- [x] Drift test `required_fields` table updated for `offset` (param) + `next_offset` (envelope key)
- [x] Macro description string updated to document `offset` + `next_offset` semantics
- [x] `cargo test -p termlink-mcp --lib` passes (771 baseline → 778+, 0 failed)

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

### 2026-06-05T12:10:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1994-termlinkhelp-namefilter-offset-param--pa.md
- **Context:** Initial task creation
