---
id: T-1951
name: "Disambiguate 7 more agent_* chat-arc-only tools with topic-generic help text (T-1947 follow-up)"
description: >
  Apply (chat-arc) suffix pattern to 7 tools missed in the original sweep

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-03T20:40:11Z
last_update: 2026-06-03T20:42:22Z
date_finished: 2026-06-03T20:43:35Z
---

# T-1951: Disambiguate 7 more agent_* chat-arc-only tools with topic-generic help text (T-1947 follow-up)

## Context

T-1947 applied the `(chat-arc)` / `(any topic)` suffix pattern to 15 agent_*/channel_*
pairs that shared identical descriptions. T-1950 fixed 3 LLM-mispick bugs surfaced
by a `fleet` keyword cross-check.

Continuing the audit, a second cross-check found 7 more `termlink_agent_*` tools
whose help description uses generic "topic" phrasing while the underlying tool is
chat-arc-only (per the macro `description = …`). These don't have a `channel_*`
sibling so they didn't appear in T-1947, but the ambiguity is the same: an LLM
reading the help line can't tell whether the tool takes a topic param.

Tools to fix:
- `termlink_agent_typing` — help generic "chat topic"
- `termlink_agent_ack_status` — help generic "a topic"
- `termlink_agent_reaction_summary` — help generic "thread/topic"
- `termlink_agent_top_reacted` — help generic "on a topic"
- `termlink_agent_top_replied` — help generic "on a topic"
- `termlink_agent_first_post_by` — help generic "on a topic"
- `termlink_agent_topic_metadata_history` — help generic "on a topic"

Pattern: append `(chat-arc)` to each, matching the T-1947 convention.

## Acceptance Criteria

### Agent
- [x] All 7 listed tools get `(chat-arc)` suffix appended to their help description
  - Evidence: commit `f9a98cf4` — agent_typing/ack_status/reaction_summary/top_reacted/top_replied/first_post_by/topic_metadata_history all now explicitly cite "chat-arc"
- [x] No new duplicate descriptions introduced (`awk` dedup count remains 0)
  - Evidence: `awk '/^fn help_categories/,/^}$/' ... | sort | uniq -d | wc -l` → `0`
- [x] `cargo test -p termlink-mcp --lib` still passes 682
  - Evidence: `test result: ok. 682 passed; 0 failed; 0 ignored; 0 measured`

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
cargo test -p termlink-mcp --lib help_ -- --nocapture
! awk '/^fn help_categories/,/^}$/' crates/termlink-mcp/src/tools.rs | grep -oE '\("termlink_[a-z_]+", "[^"]+"\)' | sed -E 's/.*", "(.*)"\)/\1/' | sort | uniq -d | grep -q .
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

### 2026-06-03T20:40:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1951-disambiguate-7-more-agent-chat-arc-only-.md
- **Context:** Initial task creation
