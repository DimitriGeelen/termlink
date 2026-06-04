---
id: T-1984
name: "termlink_help name_filter accepts limit param — deterministic result sizes for LLM clients (cycle 12 slice 1)"
description: >
  Add Option<usize> limit param to HelpParams. When set with name_filter mode, cap matches[] at the first N (post-filter, deterministic order — alphabetical by name within category, preserves category iteration order). Emit total_matched (pre-cap count) and limit_applied=true so LLM clients can detect truncation and request the next page later. Without limit: behavior unchanged. The cycle-11 retirement filters + arity filters compose normally. Direct value: an LLM client running termlink_help(name_filter='agent') currently gets 100+ matches in one shot; with limit it can paginate safely.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [mcp, help-registry, client-arc]
components: []
related_tasks: []
created: 2026-06-04T08:14:50Z
last_update: 2026-06-04T08:14:50Z
date_finished: null
---

# T-1984: termlink_help name_filter accepts limit param — deterministic result sizes for LLM clients (cycle 12 slice 1)

## Context

Cycle 12 slice 1 (predecessor: T-1976..T-1983 cycle-11 filter axis). Pagination cap on `name_filter` mode — directly enables LLM-client pagination, prevents context-blowup on broad queries (e.g. `name_filter='agent'` → 100+ tools). Mirrors cycle-10/11 invariant style: opt-in `Option<usize>` param, composes with the existing filter stack, emits `total_matched` + `limit_applied` envelope fields so clients can detect truncation.

## Acceptance Criteria

### Agent
- [ ] `limit: Option<usize>` field added to `HelpParams` with rmcp schemars doc comment
- [ ] `build_help_json` signature accepts `limit: Option<usize>` (signature grows 11 → 12 args)
- [ ] `limit` is applied ONLY in the `name_filter` branch (other modes — tool_detail, list_categories, summary, essentials, default — ignore it)
- [ ] When `name_filter` matches are collected, `total_matched` is captured BEFORE truncation; matches are then truncated to first N
- [ ] When `limit` is set: result carries `total_matched` (pre-cap count, integer) AND `limit_applied` (bool, true iff truncation actually happened)
- [ ] When `limit` is unset: result does NOT carry `total_matched` or `limit_applied` fields (backward compatibility)
- [ ] `limit = 0` returns empty matches[] but still surfaces `total_matched` and `limit_applied=true` when pre-cap count > 0
- [ ] Invariant tests added: caps-at-N, total_matched-preserved, limit_applied-transitions, composes-with-max_min_parameters, composes-with-exclude_deprecated
- [ ] Drift test `required_fields` table updated for new envelope keys
- [ ] `cargo test -p termlink-mcp --lib` passes (764 baseline → 767+, 0 failed)

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

### 2026-06-04T08:14:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1984-termlinkhelp-namefilter-accepts-limit-pa.md
- **Context:** Initial task creation
