---
id: T-1859
name: "/peers slash skill — fleet listener discovery (T-1837/1839 skill-layer wrap)"
description: >
  /peers slash skill — fleet listener discovery (T-1837/1839 skill-layer wrap)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-28T22:43:57Z
last_update: 2026-05-28T22:46:27Z
date_finished: 2026-05-28T22:46:27Z
---

# T-1859: /peers slash skill — fleet listener discovery (T-1837/1839 skill-layer wrap)

## Context

The interactive doorbell+mail arc is complete at script + skill + MCP layers
across SEND (T-1834/1431), RECEIVE (/check-arc), PRESENCE (T-1832/T-1841),
READ (T-1849/1851/1852), and BROADCAST (T-1856/1857/1858). One skill-layer
gap remains: **LIST-PEERS**. To initiate a conversation today the operator
must know `scripts/agent-listeners-fleet.sh` exists and how to invoke it —
there is no `/peers` one-keystroke verb that answers "who's around?" before
firing `/agent-handoff` or `/broadcast-chat`.

This task closes that gap by shipping a Claude Code slash skill that wraps
`scripts/agent-listeners-fleet.sh` (T-1837) with operator-friendly defaults
and per-peer copy-pasteable handoff hints. Read-only, no auth, no state
mutation — pure discovery.

Related: T-1837 (single-script fleet discovery), T-1839 (MCP wrapper),
T-1841 (/be-reachable — the SELF-advertise counterpart), T-1431
(/agent-handoff — the verb /peers feeds into), PL-187 (verb-stack pattern
rung 6: ephemeral session skills).

## Acceptance Criteria

### Agent
- [x] `.claude/commands/peers.md` exists with $ARGUMENTS pass-through, default-LIVE filter, and per-peer handoff-hint format
- [x] Skill calls `bash scripts/agent-listeners-fleet.sh --json` and surfaces output via Bash (no new helper script needed)
- [x] Empty-LIVE-fleet path prints a clear hint pointing to `/be-reachable` (advertise self) or `/broadcast-chat` (fan to whenever a peer arrives) — never silent
- [x] `--all` flag passes through to include OFFLINE entries
- [x] CLAUDE.md Quick Reference table has a `Peers (LIST)` row pointing at the skill
- [x] Live invocation in this session demonstrates the skill works against the real fleet (5 hubs)

**Live demo evidence (5-hub fleet, 0 LIVE):**

Default form (LIVE-only):
```
$ bash scripts/agent-listeners-fleet.sh
Fleet agent-presence — 5 hubs scanned, 0 failed, 0 listeners (0 LIVE / 0 STALE / 0 OFFLINE)
```
→ skill's empty-fleet hint correctly triggers, pointing operator at /be-reachable + /broadcast-chat.

`--all` form (include OFFLINE):
```
$ bash scripts/agent-listeners-fleet.sh --include-offline
Fleet agent-presence — 5 hubs scanned, 0 failed, 1 listeners (0 LIVE / 0 STALE / 1 OFFLINE)
AGENT_ID                   ROLE        STATUS   AGE_S   HUB                LISTEN_TOPICS
root-claude-dimitrimintdev claude-code OFFLINE  2872    127.0.0.1:9100     dm:root-claude-dimitrimintdev:*,agent-chat-arc
```

JSON form: passes wrapper envelope through verbatim (verified shape: ok / total_listeners / live / offline / listeners[]).

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

test -f .claude/commands/peers.md
grep -q "peers.md" CLAUDE.md
grep -q "agent-listeners-fleet" .claude/commands/peers.md

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

### 2026-05-28T22:43:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1859-peers-slash-skill--fleet-listener-discov.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-393ce444
- **Timestamp:** 2026-05-28T22:46:28Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** yes
- **Findings:** 1

**Per-AC findings:**

- **AC#2 (Agent)** — Skill calls `bash scripts/agent-listeners-fleet.sh --json` and surfaces output via Bash (no new helper script needed)
  - **AC-verify-mismatch** (narrow, heuristic) — `path=scripts/agent-listeners-fleet.sh in: Skill calls `bash scripts/agent-listeners-fleet.sh --json` and surfaces output via Bash (no new helper script needed)`

- **Layer-1 escalations:** 1
  1. **external-publish** (high) — External publish or release
     - matched: `broadcast`

### 2026-05-28T22:46:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
