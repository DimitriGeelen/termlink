---
id: T-1861
name: "agent-chat-arc-recent.sh --exclude-heartbeats + /pulse heartbeat-aware render (T-1860 follow-on)"
description: >
  agent-chat-arc-recent.sh --exclude-heartbeats + /pulse heartbeat-aware render (T-1860 follow-on)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-29T09:46:43Z
last_update: 2026-05-29T09:46:43Z
date_finished: null
---

# T-1861: agent-chat-arc-recent.sh --exclude-heartbeats + /pulse heartbeat-aware render (T-1860 follow-on)

## Context

`/pulse` first-run revealed a structural blind spot: a fleet that looks
HOT by post volume (`unique_speakers=3`) can be entirely systemd
heartbeat bots (`*-vendored` agent_ids posting T-1438 emitter content).
An operator running `/pulse` to answer "are there real conversations
happening?" gets a false positive — bot bookkeeping inflates the
unique_speakers count.

The directive's "no active conversations" framing is precisely about
making this distinction visible. Currently it's invisible without
manually grepping payload text.

Fix shape:
1. `agent-chat-arc-recent.sh` gets a `--exclude-heartbeats` flag.
   Heuristic: sender ends with `-vendored` (T-1832/T-1840 structural
   naming convention used by all vendored-arc emitters). Hardcoded
   suffix; future emitters get added by extension if convention
   changes.
2. `/pulse` calls the script with `--exclude-heartbeats` by default and
   surfaces BOTH counts in its header: `unique_speakers=K real + N bots`.
   So the cold-conversation state is structurally readable: K=0
   instantly tells the operator "rail has no real activity even though
   posts > 0".
3. `/recent-chat` doc gets a mention of the flag for operators who want
   the same filter in stand-alone reads.

This is a minimal, scoped change. Pure read-side filter, no behavior
change for unflagged calls.

Related: T-1860 (/pulse), T-1851 (/recent-chat), T-1832 (heartbeat
emitter convention), PL-190 (volume vs actor-diversity).

## Acceptance Criteria

### Agent
- [x] `agent-chat-arc-recent.sh` accepts `--exclude-heartbeats`; jq filter excludes posts where `metadata.agent_id // metadata._from // sender_id` ends with `-vendored`
- [x] JSON envelope's `.summary` gains a `heartbeat_posts` and `heartbeat_speakers` count when the flag is on (so callers can render both numbers)
- [x] Text mode header reflects the flag: `(window: ..., unique_speakers: K, heartbeats excluded: N)`
- [x] `/pulse` skill spec updated to call with `--exclude-heartbeats` by default + render BOTH counts
- [x] `/recent-chat` skill doc gets `--exclude-heartbeats` in its flag table
- [x] Live test: with the flag, a populated fleet shows zero posts when only vendored-arc heartbeats are present; without the flag, count is unchanged
- [x] Help text updated with the new flag and its heuristic

**Live demo (24h window, populated fleet):**

```
═══ rail pulse (T-1861 heartbeat-aware) ═══

PEERS (LIVE / total): 0 / 0
  (no LIVE peers)

RECENT (last 5 in 24h window, unique speakers=1 + 3 heartbeat bots hidden / 96 heartbeat posts):
  2026-05-29T09:41:40Z  root-claude-dimitrimintdev  T-1860 shipped: /pulse ...
```

Before T-1861: operator saw "unique_speakers=4" and assumed lively
conversation. After T-1861: structurally clear that the rail has 96
bookkeeping posts vs 1 real conversation post — actionable signal that
matches the directive's "no active conversations" framing.

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

bash scripts/agent-chat-arc-recent.sh --help 2>&1 | grep -q '\-\-exclude-heartbeats'
bash scripts/agent-chat-arc-recent.sh --exclude-heartbeats --json --limit 1 --since 24 2>/dev/null | jq -e '.summary.heartbeat_posts != null'
grep -q 'exclude-heartbeats' .claude/commands/pulse.md
grep -q 'exclude-heartbeats' .claude/commands/recent-chat.md

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

### 2026-05-29T09:46:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1861-agent-chat-arc-recentsh---exclude-heartb.md
- **Context:** Initial task creation
