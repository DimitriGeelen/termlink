---
id: T-1843
name: "fleet-adoption-snapshot.sh — measure REAL doorbell+mail traffic (distinct from T-1831 health canary)"
description: >
  fleet-adoption-snapshot.sh — measure REAL doorbell+mail traffic (distinct from T-1831 health canary)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [scripts/fleet-adoption-snapshot.sh, scripts/test-fleet-adoption-snapshot.sh]
related_tasks: []
created: 2026-05-28T17:29:47Z
last_update: 2026-05-28T17:58:59Z
date_finished: 2026-05-28T17:58:59Z
---

# T-1843: fleet-adoption-snapshot.sh — measure REAL doorbell+mail traffic (distinct from T-1831 health canary)

## Context

T-1831 fleet-doorbell-mail-canary measures **health** (loopback plumbing). It
returns PASS forever on a hub where nobody is actually using the rail. That's
exactly the silent-failure mode the standing directive ("no active doorbell+mail
conversations arc") is pointing at. This task ships a **distinct gauge** that
measures REAL traffic — per-hub and fleet-wide rollup of:

- agent-presence heartbeats (live listeners)
- dm:* topic activity (peer-to-peer DMs)
- agent-chat-arc posts (fleet broadcast traffic)

Different mental model: T-1831 = "is the plumbing intact?", T-1843 = "is anyone
actually talking?". Both required; neither substitutes for the other.

Composes on top of T-1837 (`agent-listeners-fleet.sh`) and `termlink channel
list` / `channel subscribe`. Read-only; no auth-mutating writes. G-060 graceful
degradation (T-1842 pattern): missing topics on a fresh hub → 0, not error.

## Acceptance Criteria

### Agent
- [x] `scripts/fleet-adoption-snapshot.sh` exists, executable, `--help` exits 0 with usage
- [x] Walks every profile in `~/.termlink/hubs.toml` (same source as T-1837 / canary)
- [x] For each hub measures: `live_listeners` (via agent-presence heartbeats), `dm_topic_count` (lifetime count of `dm:*:*` topics with ≥1 post), `chat_arc_posts` (windowed count on `agent-chat-arc`)
- [x] `--since <hours>` window (default 24, clamp 1..=720)
- [x] `--json` flag emits one envelope `{ok, window_hours, summary:{hubs, reachable_hubs, live_listeners, chat_arc_posts, dm_topics_active, adoption_state}, profiles:[...]}` parseable by jq
- [x] Human (default) format prints a one-screen summary + per-hub rows
- [x] `adoption_state` field: `HOT` (≥1 chat_arc post in window AND ≥1 live listener), `WARM` (≥1 live listener but no chat_arc activity), `COLD` (zero live listeners fleet-wide)
- [x] G-060 graceful: missing `agent-presence` / `agent-chat-arc` topic on a hub returns zeros for that hub, not an error (handles -32013, "unknown topic", and "Topic not found")
- [x] Unreachable hub (network error) returns `verdict:"unreachable"` for that hub but does not fail the overall sweep (exit 0 still)
- [x] Exit codes: 0 = sweep completed (any adoption_state), 2 = usage error, 3 = setup-fail (hubs.toml missing / jq missing)
- [x] `scripts/test-fleet-adoption-snapshot.sh` exists and passes — 9/9 covering help, unknown arg, --since validation, missing hubs-file, parse round-trip, adoption_state values, human format labels
- [x] Live run against local fleet: shows state=HOT, hubs=5, live_listeners=2, chat_arc_posts=174 in last 24h
- [x] **Bonus:** wrapped all termlink calls with `timeout 8` — without this `channel info --hub <unreachable>` hangs forever (50+ zombie processes observed in the wild over a week)

## Recommendation

**Recommendation:** GO

**Rationale:** Delivers a distinct adoption gauge complementing the T-1831
health canary. The health canary returns PASS on a fleet where nobody is
using the rail — exactly the silent-failure mode the standing directive
("no active doorbell+mail conversations arc") was pointing at. The
snapshot answers a different question: "is anyone ACTUALLY talking right
now?". HOT/WARM/COLD adoption_state gives the operator a single-glance
answer.

Discovered + closed a latent bug along the way (T-1844: seek-to-tail in
agent-listeners.sh) and captured the bypass-timeout cure for the
`channel info` hang pathology (will file as a separate concern).

Live verification on the fleet (post-fix): state=HOT, 5/5 hubs reachable,
2 live listeners (root-claude-dimitrimintdev — my own be-reachable session
visible across both 107 and 127.0.0.1 addresses), 174 chat_arc_posts in
the 24h window, 258 dm_topics_active fleet-wide.

**Evidence:**
- `scripts/fleet-adoption-snapshot.sh` — 220 LOC, executable, --help ok
- `scripts/test-fleet-adoption-snapshot.sh` — 9/9 pass
- `bash scripts/fleet-adoption-snapshot.sh --json` returns valid envelope
  with `{ok:true, window_hours:24, summary.adoption_state:"HOT"}`
- Companion fix shipped as commit `812517a3` (T-1844)
- Linked learning: PL-188 (channel subscribe cursor semantics)

**Human action:** Tick the [RUBBER-STAMP] AC once you've run the snapshot
yourself and confirmed the output is useful as a "is anyone using this?"
gauge. Iteration welcome if a signal feels missing or misleading.

### Human
- [ ] [RUBBER-STAMP] Snapshot is operator-useful as a "is anyone using this?" check
  **Steps:**
  1. Run `bash scripts/fleet-adoption-snapshot.sh`
  2. Inspect the one-screen summary
  **Expected:** Output answers "is anyone actually using the doorbell+mail arc right now?" at a glance, with a colored or labelled adoption_state field (HOT/WARM/COLD).
  **If not:** Note which signal you wished was there but wasn't (or which one is misleading), and the agent will iterate.

## Verification

test -x scripts/fleet-adoption-snapshot.sh
test -x scripts/test-fleet-adoption-snapshot.sh
bash scripts/fleet-adoption-snapshot.sh --help >/dev/null
bash scripts/test-fleet-adoption-snapshot.sh
bash scripts/fleet-adoption-snapshot.sh --json | jq -e '.ok == true and (.summary | has("adoption_state"))' >/dev/null

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

### 2026-05-28T17:29:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1843-fleet-adoption-snapshotsh--measure-real-.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-5c90d7e2
- **Timestamp:** 2026-05-28T17:59:26Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** yes
- **Findings:** 3

**Per-AC findings:**

- **AC#2 (Agent)** — Walks every profile in `~/.termlink/hubs.toml` (same source as T-1837 / canary)
  - **AC-verify-mismatch** (narrow, heuristic) — `path=termlink/hubs.toml in: Walks every profile in `~/.termlink/hubs.toml` (same source as T-1837 / canary)`

**Verification-level findings:**

  1. **empty-output-success** (partial, heuristic) @ Verification:line 3
     - evidence: `bash scripts/fleet-adoption-snapshot.sh --help >/dev/null`
  2. **empty-output-success** (partial, heuristic) @ Verification:line 5
     - evidence: `bash scripts/fleet-adoption-snapshot.sh --json | jq -e '.ok == true and (.summary | has("adoption_state"))' >/dev/null`

- **Layer-1 escalations:** 1
  1. **cross-project-blast** (medium) — Cross-project or cross-repo change
     - matched: `fleet-wide`

### 2026-05-28T17:58:59Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
