---
id: T-1857
name: "/broadcast-chat slash skill — wraps chat-arc-broadcast.sh (T-1856 follow-on)"
description: >
  /broadcast-chat slash skill — wraps chat-arc-broadcast.sh (T-1856 follow-on)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-28T21:55:41Z
last_update: 2026-05-28T21:55:41Z
date_finished: null
---

# T-1857: /broadcast-chat slash skill — wraps chat-arc-broadcast.sh (T-1856 follow-on)

## Context

T-1856 shipped `scripts/chat-arc-broadcast.sh` for cross-hub broadcast.
The discovery triangle has script + skill + MCP parity for /recent-chat,
/be-reachable, /check-arc, /agent-handoff. Broadcast needs the same
ergonomic skill-layer entry so a claude session can fan a status note
to the fleet without remembering the script path or its flag set.
Mirrors T-1851's pattern for /recent-chat.

## Acceptance Criteria

### Agent
- [x] `.claude/commands/broadcast-chat.md` created with frontmatter + steps in the same shape as `.claude/commands/recent-chat.md` (T-1851)
- [x] Skill pre-flights `bash scripts/chat-arc-broadcast.sh --help`, parses `$ARGUMENTS` so first positional becomes the broadcast text, surfaces the wrapper's stdout verbatim, declares itself NOT read-only (writes to chat-arc)
- [x] CLAUDE.md Quick Reference table gains a row for `/broadcast-chat` next to the existing PRESENCE/RECEIVE/SEND/CONTEXT rows
- [x] Skill explicitly references the R2-class write-vs-read distinction (this skill DOES mutate; pairs with read-only /recent-chat)
- [x] Skill ships safety rules — never auto-broadcast without explicit operator text, never paraphrase, never silently default sender

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
test -f .claude/commands/broadcast-chat.md
grep -q "chat-arc-broadcast.sh" .claude/commands/broadcast-chat.md
grep -q "broadcast-chat" CLAUDE.md

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

### 2026-05-28T21:55:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1857-broadcast-chat-slash-skill--wraps-chat-a.md
- **Context:** Initial task creation
