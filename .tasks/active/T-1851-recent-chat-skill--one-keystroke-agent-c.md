---
id: T-1851
name: "/recent-chat skill — one-keystroke agent-chat-arc recent posts (T-1849 UX wrapper)"
description: >
  Claude Code slash-command wrapping scripts/agent-chat-arc-recent.sh. Makes the T-1849 'what's been said?' verb one-keystroke for any claude session, closing the context-before-reply gap operationally. Follows the /be-reachable / /check-arc pattern.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [doorbell-mail, skill, t-1849-followon, arc:t-1830]
components: []
related_tasks: []
created: 2026-05-28T19:50:46Z
last_update: 2026-05-28T19:50:46Z
date_finished: null
---

# T-1851: /recent-chat skill — one-keystroke agent-chat-arc recent posts (T-1849 UX wrapper)

## Context

T-1849 shipped `scripts/agent-chat-arc-recent.sh` (the "what's been said?" verb). For operators landing in a fresh claude session, that's still two shell calls + remembering the path. A slash-command shortcut makes the discovery flow one keystroke and mirrors the convention established by `/be-reachable` (T-1841), `/agent-handoff` (T-1815), `/check-arc` (T-1810). Closes the operational UX gap on the discovery triangle.

## Acceptance Criteria

### Agent
- [x] `.claude/commands/recent-chat.md` (NEW) — wraps `scripts/agent-chat-arc-recent.sh`. Default form `/recent-chat` returns last 20 posts in 24h window. Args passthrough: `--since N`, `--limit N`, `--hub addr`, `--filter-sender ID`, `--all-msg-types`, `--json`.
- [x] Skill respects the `/be-reachable` convention: step-by-step, pre-flight wrapper exists, clear error printing, no AskUserQuestion.
- [x] CLAUDE.md Quick Reference table gains a row pointing to the new skill (kept in sync with other recently-added skills like `/be-reachable`).
- [x] Smoke: `bash scripts/agent-chat-arc-recent.sh --since 24 --limit 5 --json` returns ≥1 post on the live fleet.

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

test -f .claude/commands/recent-chat.md
grep -q '/recent-chat' .claude/commands/recent-chat.md
grep -q '/recent-chat' CLAUDE.md
bash scripts/agent-chat-arc-recent.sh --since 24 --limit 5 --json | jq -e '.posts | type == "array"' >/dev/null

# Old toolchain hints below — kept for reference.
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

### 2026-05-28T19:50:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1851-recent-chat-skill--one-keystroke-agent-c.md
- **Context:** Initial task creation
