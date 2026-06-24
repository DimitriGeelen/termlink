---
id: T-1998
name: "termlink_help — fields projection on matches rows (cycle 12 slice 6)"
description: >
  termlink_help — fields projection on matches rows (cycle 12 slice 6)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-06-05T15:11:09Z
last_update: 2026-06-05T15:11:09Z
date_finished: 2026-06-05T15:35:23Z
---

# T-1998: termlink_help — fields projection on matches rows (cycle 12 slice 6)

## Context

Cycle-12 slice 6. T-1984/T-1994/T-1995/T-1996/T-1997 gave `termlink_help` a real paged-ranked-filtered API. The remaining largest per-row payload is now the `description` string (averaging several hundred bytes per tool). For LLM clients that want to rank or paginate but don't need the descriptive prose on every page, a `fields` projection lets them say "I just want name + parameter_required_count" and shrink the response by ~80%.

Strict projection — what the user requests is what they get, no implicit name retention. Unknown field names are silently dropped from the request but surfaced via the envelope `fields_unknown` so the caller sees the silently-ignored input. Empty `fields:[]` is treated as no projection (degenerate, backcompat-friendly).

Allowed fields (the row-shape surface area shipped by T-1960..T-1995): `name`, `category`, `category_tool_count`, `description`, `deprecated`, `parameter_count`, `parameter_required_count`, `replacement_hint`.

Applies in `name_filter` mode and (via T-1997) bulk-flat-listing mode. Other modes (default categories-keyed, tool_detail, list_categories, summary, essentials) are unaffected — they emit purpose-built shapes that already meet client needs.

## Acceptance Criteria

### Agent
- [x] HelpParams gains `fields: Option<Vec<String>>` field with doc comment referencing T-1998
- [x] `build_help_json` signature grows 14 → 15 args (new `fields: Option<&[String]>` last)
- [x] All ~100 callers patched to pass `None` for the new arg
- [x] Production caller (`call_termlink_help`) wires `let fields = p.fields.as_deref();`
- [x] Eight allowed field names enumerated as a const set
- [x] When `fields` is `Some(non-empty)`, every row in `matches[]` is filtered to retain ONLY requested keys present in the allowed set (strict; no implicit `name`)
- [x] Unknown field names dropped from projection AND surfaced via envelope `fields_unknown: [...]`
- [x] Envelope emits `fields_applied: [...]` echoing the recognized fields when projection effected
- [x] Empty `fields:[]` treated as no projection (envelope omits both fields_applied and fields_unknown)
- [x] Drift table gains `("fields_applied", "T-1998")` and `("fields_unknown", "T-1998")`
- [x] 11 invariant tests added covering: single field, multi field, no implicit name retention, unknown field surfaces in fields_unknown, mixed valid+unknown, unset omits both envelope fields (backcompat), empty array omits both, composes with limit, composes with sort_by (projection runs LAST), composes with bulk-flat-listing (no needle), only requested keys appear in row
- [x] `cargo test -p termlink-mcp --lib`: baseline 804 → 815 passed, 0 failed

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

cd /opt/termlink && cargo test -p termlink-mcp --lib 2>&1 | tail -3 | grep -q "test result: ok"

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

### 2026-06-05T15:11:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1998-termlinkhelp--fields-projection-on-match.md
- **Context:** Initial task creation
