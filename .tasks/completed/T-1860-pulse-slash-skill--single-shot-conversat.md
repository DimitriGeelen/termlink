---
id: T-1860
name: "/pulse slash skill — single-shot conversation arc digest (peers + recent in one render)"
description: >
  /pulse slash skill — single-shot conversation arc digest (peers + recent in one render)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-29T09:36:08Z
last_update: 2026-05-29T09:36:08Z
date_finished: null
---

# T-1860: /pulse slash skill — single-shot conversation arc digest (peers + recent in one render)

## Context

The conversation-arc skill set is complete at six discrete corners (T-1859
close-out). But an operator landing fresh still needs to run three skills
sequentially to answer "what's happening on the arc right now?":

  /peers       → who's around
  /recent-chat → what's been said
  /check-arc   → my inbox

Three round-trips + mental synthesis before engaging. That's friction the
directive's "no active conversations" framing makes visible: even with the
verbs available, the cold-start cost dampens engagement.

This task ships `/pulse` — a pure-composition single-shot digest verb that
runs /peers + /recent-chat in parallel and renders one unified view. The
"is the rail alive and what's the pulse?" verb. No new logic, no new
script — just a Bash orchestration skill that calls the existing wrappers
with --json, parses, and renders a compact 2-section summary.

Related: T-1859 (/peers), T-1851 (/recent-chat), T-1841 (/be-reachable),
PL-187 (verb-stack rung 6).

## Acceptance Criteria

### Agent
- [x] `.claude/commands/pulse.md` exists; runs the two wrappers via Bash with --json
- [x] Output has two sections (PEERS / RECENT) clearly separated
- [x] Empty fleet (0 LIVE + 0 recent) prints a clear "rail is cold — try /be-reachable + /broadcast-chat" hint
- [x] Either wrapper failing degrades gracefully (other section still renders)
- [x] CLAUDE.md Quick Reference row added below `/recent-chat`
- [x] Live demonstrated against the real fleet

**Live demo evidence (parallel composition, 5-hub fleet):**

```
═══ rail pulse ═══

PEERS (LIVE / total): 0 / 0
  (no LIVE peers)

RECENT (last 5 in 24h window, unique speakers=3):
  2026-05-29T09:17:01Z  dimitrixpro-vendored     T-1438 vendored-arc heartbeat from dimitrixpro ...
  2026-05-29T09:17:01Z  dimitrimintdev-vendored  T-1438 vendored-arc heartbeat from dimitrimintdev ...
  2026-05-29T09:17:01Z  ring20-manager-vendored  T-1438 vendored-arc heartbeat from ring20-manager ...
  ...
```

Notable: output cleanly distinguishes interactive PRESENCE (0 LIVE — no
claude-code sessions) from systemd heartbeat NOISE (3 vendored speakers
from T-1840 emitters). Exactly the diagnostic the cold-start verb is for.

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

test -f .claude/commands/pulse.md
grep -q "pulse.md" CLAUDE.md
grep -q "agent-listeners-fleet" .claude/commands/pulse.md
grep -q "agent-chat-arc-recent" .claude/commands/pulse.md

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
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

### 2026-05-29T09:36:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1860-pulse-slash-skill--single-shot-conversat.md
- **Context:** Initial task creation
