---
id: T-1939
name: "MCP help registry — surface channel_* family (primitive bus operations)"
description: >
  Add the channel_* primitive bus family to termlink_help so LLM consumers can discover the lower-level operations beneath agent_*. Currently ~50 channel_* tools are registered but invisible via help. Scope: channel primitives (create/list/post/subscribe/info/describe/snapshot) + edits/redactions/pins/stars/reactions/quotes/replies/forwards. Out of scope: aggregate stats (emoji_stats, quote_stats, edit_stats, queue_status), poll lifecycle (covered as agent_poll already), members/typing (admin-tier).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-03T00:39:31Z
last_update: 2026-06-03T00:39:31Z
date_finished: null
---

# T-1939: MCP help registry — surface channel_* family (primitive bus operations)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] `channel` category exists in `termlink_help` with core primitives: `channel_create`, `channel_list`, `channel_post`, `channel_reply`, `channel_subscribe`, `channel_info`, `channel_describe`, `channel_snapshot`, `channel_state`, `channel_unread`, `channel_ack`
- [ ] `channel_threading` category exists with `channel_thread`, `channel_threads`, `channel_ancestors`, `channel_replies_of`, `channel_quote`, `channel_quote_stats`, `channel_relations`
- [ ] `channel_moderation` category exists with `channel_edit`, `channel_edits_of`, `channel_redact`, `channel_redactions`, `channel_pin`, `channel_pin_history`, `channel_pinned`, `channel_forward`, `channel_forwards_of`
- [ ] `channel_engagement` category exists with `channel_react`, `channel_reactions_of`, `channel_reactions_on`, `channel_star`, `channel_starred`, `channel_mentions`, `channel_mentions_of`, `channel_search`, `channel_snippet`, `channel_digest`
- [ ] Unknown-category error message lists all 4 new categories
- [ ] Tool description mentions the new channel categories
- [ ] `cargo build --release -p termlink-mcp` is warning-free

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
! cargo build --release -p termlink-mcp 2>&1 | grep -q "warning:"
grep -q '"channel", vec!' crates/termlink-mcp/src/tools.rs
grep -q '"channel_threading", vec!' crates/termlink-mcp/src/tools.rs
grep -q '"channel_moderation", vec!' crates/termlink-mcp/src/tools.rs
grep -q '"channel_engagement", vec!' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_channel_post"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_channel_subscribe"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_channel_redact"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_channel_search"' crates/termlink-mcp/src/tools.rs

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

### 2026-06-03T00:39:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1939-mcp-help-registry--surface-channel-famil.md
- **Context:** Initial task creation
