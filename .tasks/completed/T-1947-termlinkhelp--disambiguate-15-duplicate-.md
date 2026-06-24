---
id: T-1947
name: "termlink_help — disambiguate 15 duplicate description pairs (agent_* vs channel_* parallel families)"
description: >
  Differentiate help-registry descriptions where agent_X and channel_X share identical text

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-06-03T20:19:00Z
last_update: 2026-06-03T20:24:03Z
date_finished: 2026-06-03T20:26:48Z
---

# T-1947: termlink_help — disambiguate 15 duplicate description pairs (agent_* vs channel_* parallel families)

## Context

T-1945 closed help coverage at 100% (252/252). T-1946 locked the bi-directional
invariant. Operating audit of the registry surfaces a remaining LLM-discoverability
issue: 15 description pairs are byte-identical between the `agent_*` and `channel_*`
parallel families. Example:

- `termlink_agent_starred` → "Your starred posts"
- `termlink_channel_starred` → "Your starred posts"

When an LLM calls `termlink_help name_filter="starred posts"`, both surface
identically and the LLM cannot pick the right one. The real semantic axis is:

- `agent_*` operates on the well-known `agent-chat-arc` topic (chat layer)
- `channel_*` operates on any user-specified topic (raw channel layer)

This slice differentiates the 15 pairs by appending a one-phrase axis
indicator: `(chat-arc)` for agent_* and `(any topic)` for channel_*. Brief
enough to not bloat the help output; specific enough that an LLM can
choose correctly. The full per-tool macro `description = "…"` (read by
agents that drill in) already differentiates — only the registry one-liners
need the fix.

## Acceptance Criteria

### Agent
- [x] All 15 `agent_*` entries in the duplicate set get `" (chat-arc)"` suffix in `help_categories()`
  - Evidence: 15 agent_* entries patched — `ack`, `react`, `topic_stats`, `relations`, `ancestors`, `pinned`, `replies_of`, `edits_of`, `forwards_of`, `typers`, `pin`, `pin_history`, `reactions`, `star`, `starred`. Commit `24736fff`
- [x] All 15 `channel_*` entries in the duplicate set get `" (any topic)"` suffix in `help_categories()`
  - Evidence: 15 channel_* entries patched — `ack`, `react`, `topic_stats`, `relations`, `ancestors`, `pinned`, `replies_of`, `edits_of`, `forwards_of`, `typing_list`, `pin`, `pin_history`, `reactions_on`, `star`, `starred`. Commit `24736fff`
- [x] No two entries in `help_categories()` share an identical description after the fix (verified by `awk` dedup count = 0)
  - Evidence: `awk '/^fn help_categories/,/^}$/' crates/termlink-mcp/src/tools.rs | grep -oE '\("termlink_[a-z_]+", "[^"]+"\)' | sed -E 's/.*", "(.*)"\)/\1/' | sort | uniq -d | wc -l` → `0`
- [x] `cargo test -p termlink-mcp --lib` still passes 679 (no regression in phantom/coverage guards)
  - Evidence: `test result: ok. 679 passed; 0 failed; 0 ignored; 0 measured` — invariants from T-1941 + T-1946 both still hold

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
cargo test -p termlink-mcp --lib help_registry -- --nocapture
! awk '/^fn help_categories/,/^}$/' crates/termlink-mcp/src/tools.rs | grep -oE '\("termlink_[a-z_]+", "[^"]+"\)' | sed -E 's/.*", "(.*)"\)/\1/' | sort | uniq -d | grep -q .

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

### 2026-06-03T20:19:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1947-termlinkhelp--disambiguate-15-duplicate-.md
- **Context:** Initial task creation
