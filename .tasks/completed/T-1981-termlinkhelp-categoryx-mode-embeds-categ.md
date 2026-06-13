---
id: T-1981
name: "termlink_help: category=X mode embeds category_meta envelope block (description+counts at first round-trip)"
description: >
  When called with category=X (no other modes), the response currently returns {X: [rows...], total_tools}. Add a top-level category_meta object: {name, description, tool_count, deprecated_count, live_tool_count}. LLMs drilling into a category see its purpose + size + retirement status at the same round-trip as the row enumeration, without a separate list_categories call. Derives from category_descriptions() + the existing tool walk.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-04T07:28:27Z
last_update: 2026-06-04T07:31:12Z
date_finished: 2026-06-04T07:33:11Z
---

# T-1981: termlink_help: category=X mode embeds category_meta envelope block (description+counts at first round-trip)

## Context

Cycle 11 slice 6. Closes the round-trip count for category-drill flow: currently LLMs picking a category from list_categories then re-querying with `category=X` only see per-tool rows, not the category-level metadata they were just inspecting. This slice embeds a `category_meta` block in the envelope so list_categories → category drill carries forward the namespace metadata at the second round-trip.

## Acceptance Criteria

### Agent
- [x] `category=X` envelope gains top-level `category_meta` object: `{name, description, tool_count, deprecated_count, live_tool_count}`
- [x] `category=X` with no category description (unlikely; guarded by existing invariant) emits `description: ""` not absent — stable shape
- [x] Test: `category_mode_envelope_carries_category_meta` — `category=channel` returns `category_meta` block with correct counts
- [x] Test: `category_meta_counts_match_list_categories` — same category's counts equal what `list_categories` reports for that name
- [x] Test: `category_meta_present_for_every_category` — walks every category, asserts `category_meta` block is present and counts match arithmetic
- [x] Drift test gains required field `("category_meta", "T-1981")`
- [x] `cargo test --lib --package termlink-mcp` passes; new test count == 755 + 3 = 758

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
cargo test --lib --package termlink-mcp --quiet 2>&1 | tail -3 | grep -q "test result: ok"

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

### 2026-06-04T07:28:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1981-termlinkhelp-categoryx-mode-embeds-categ.md
- **Context:** Initial task creation

### 2026-06-04T07:29:11Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
