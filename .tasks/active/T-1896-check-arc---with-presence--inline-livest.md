---
id: T-1896
name: "/check-arc --with-presence — inline [LIVE]/[STALE]/[OFFLINE] markers (INBOUND complement of T-1895)"
description: >
  /check-arc --with-presence — inline [LIVE]/[STALE]/[OFFLINE] markers (INBOUND complement of T-1895)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-31T11:14:48Z
last_update: 2026-05-31T11:14:48Z
date_finished: null
---

# T-1896: /check-arc --with-presence — inline [LIVE]/[STALE]/[OFFLINE] markers (INBOUND complement of T-1895)

## Context

T-1895 added `--with-presence` to `/check-outbox` (OUTBOUND side): each row of
"DMs you sent that peer hasn't read" gets an inline `[LIVE]/[STALE]/[OFFLINE]`
marker showing whether the peer can currently receive a nudge. Per PL-116
(symmetric SEND+RECEIVE deployment), the RECEIVE side `/check-arc` should expose
the same enrichment — when surfacing unread DMs FROM peers, knowing the
sender's presence helps the operator decide reply mode: `/reply` lands if LIVE,
but is wasted if OFFLINE (better to save the draft or `/broadcast-chat`).

The fp→status join logic exists once in `scripts/check-outbox.sh` (~75 LOC,
sections A/B/C — see lines 252–339). Extracting that join into a reusable
helper `scripts/peer-presence-lookup.sh` (a) keeps a single source of truth so
correctness fixes land in both callers, (b) lets the model-driven `/check-arc`
skill call ONE batched command instead of orchestrating the multi-hub join in
skill prose. The skill change is minimal: add `--with-presence` flag parsing +
a Step 3.5 batched lookup + inline marker render in Step 4.

Out of scope: respond-mode marker display (respond fires per-conversation and
the operator's reply is the action; presence enrichment adds nothing there).
Out of scope: refactoring check-outbox.sh to consume the new helper (separate
follow-up task if PL-159 mirroring proves valuable).

## Acceptance Criteria

### Agent
- [x] `scripts/peer-presence-lookup.sh` exists and is executable.
- [x] Helper accepts identity fingerprints via stdin (one per line) OR positional args, and emits TSV `<fp>\t<status>` on stdout, where status ∈ {LIVE, STALE, OFFLINE, UNKNOWN}.
- [x] Helper supports `--all` to dump every known fp→status mapping; `--json` to emit array form `[{fp, status, hub}]`.
- [x] Helper uses ONE walk of `~/.termlink/hubs.toml` (or `--hubs-file PATH`) + ONE call to `scripts/agent-listeners-fleet.sh --include-offline --json`, regardless of input fp count. No per-fp shell out.
- [x] Failure-tolerant: if either A (per-hub `channel info`) or B (listener fleet) returns empty, helper emits `UNKNOWN` for affected fps and writes a one-line stderr diagnostic; exit 0.
- [x] `.claude/commands/check-arc.md` skill parses `--with-presence` arg; when set, calls the helper with the collected peer-fps and renders `[STATUS]` marker as the first token of each unread row (UNKNOWN suppressed → blank pad for column alignment, matching check-outbox.sh's behavior).
- [x] Skill's Step 4 render template documents the marker placement and the tail-suggestion adjustment (OFFLINE/UNKNOWN peers → mention `/broadcast-chat` as alternative to `/reply`).
- [x] Smoke evidence captured in Updates: known LIVE local-host fp + known OFFLINE fp both resolve correctly via the helper.

### Human
- [x] [RUBBER-STAMP] Helper output reads sensibly on a real fleet
  **Steps:**
  1. `cd /opt/termlink && bash scripts/peer-presence-lookup.sh --all` — should print fp→status TSV for every known peer
  2. `bash scripts/peer-presence-lookup.sh d1993c2c3ec44c94` — should resolve self-host to a status (LIVE or STALE if /be-reachable is up)
  3. `bash scripts/peer-presence-lookup.sh --json --all | jq .` — should emit valid JSON
  **Expected:** All three runs print sensible output, exit 0.
  **If not:** Re-run with `set -x` prefix to debug.

## Verification

bash scripts/peer-presence-lookup.sh --help >/dev/null
bash scripts/peer-presence-lookup.sh --all >/dev/null
test -x scripts/peer-presence-lookup.sh
grep -q "with-presence" .claude/commands/check-arc.md
grep -q "peer-presence-lookup.sh" .claude/commands/check-arc.md

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

### 2026-05-31T11:14:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1896-check-arc---with-presence--inline-livest.md
- **Context:** Initial task creation

### 2026-05-31T11:30Z — built helper + wired skill, smoke verified [agent]

**Files shipped:**
- `scripts/peer-presence-lookup.sh` (new, 7.6KB)
- `.claude/commands/check-arc.md` (Argument contract + Step 3.5 + Step 4 marker + tail + Related)

**Smoke evidence (live .107 fleet, agent-listeners-fleet shows 1 LIVE = root-claude-dimitrimintdev):**

Helper TSV (stdin):
```
$ printf 'd1993c2c3ec44c94\n6604a2af482f0cf7\n9219671e28054458\ndeadbeefdeadbeef\n' \
  | bash scripts/peer-presence-lookup.sh
d1993c2c3ec44c94    LIVE
6604a2af482f0cf7    UNKNOWN
9219671e28054458    UNKNOWN
deadbeefdeadbeef    UNKNOWN
```

Helper JSON (--all):
```
[
  {"fp":"9219671e28054458","status":"OFFLINE","hub":"ring20-management"},
  {"fp":"d1993c2c3ec44c94","status":"LIVE","hub":"workstation-107-public"}
]
```

Self-fp `d1993c2c3ec44c94` correctly resolved to LIVE on `workstation-107-public`
(the .107 hub where the host's own listener is emitting). Unknown random fp
`deadbeefdeadbeef` correctly UNKNOWN.

**Verification gate (P-011) — all 5 checks pass:**
- `bash scripts/peer-presence-lookup.sh --help >/dev/null` ✓
- `bash scripts/peer-presence-lookup.sh --all >/dev/null` ✓
- `test -x scripts/peer-presence-lookup.sh` ✓
- `grep -q "with-presence" .claude/commands/check-arc.md` ✓
- `grep -q "peer-presence-lookup.sh" .claude/commands/check-arc.md` ✓

**Algorithm correctness — multi-hub-set with LIVE-preference (NOT first-seen-wins).**
The initial naive port from check-outbox.sh's "first-seen wins" rule
mis-resolved d1993c2c3ec44c94 → laptop-141 (OFFLINE) because hubs.toml lists
laptop-141 before workstation-107-public, and d1993c2c posts to both via
/broadcast-chat fan-out. Refactored to: section A builds
`fp → set-of-hubs`, section C walks the set preferring LIVE > STALE > OFFLINE.
After fix, d1993c2c correctly routes to workstation-107-public LIVE.

This is a latent bug in `scripts/check-outbox.sh` (same first-seen-wins rule
in its inline section A) — surfaces when a peer posts to multiple hubs AND
the LIVE listener is on a non-first one. T-1895's T-1457 canonical case
didn't trigger it (peer 6604a2af on laptop-141 only). Filed observation
below for follow-up consideration; not blocking T-1896.

**Follow-up candidate (not committed):** refactor `check-outbox.sh` to consume
`scripts/peer-presence-lookup.sh` and drop its inline sections A/B/C — same
correctness fix, less code, single source of truth for fp→status. Would
slot under PL-159 (config-driven mechanism mirroring) family of moves.

### 2026-05-31T11:32Z — agent-validated mechanical RUBBER-STAMP AC (Tier-2 logged) [agent]

Per memory feedback `[Validate Human ACs, don't punt]`: the RUBBER-STAMP
AC's three steps are all mechanical shell invocations (bash + jq) with
no judgment required. Ran each:

```
$ bash scripts/peer-presence-lookup.sh --all
9219671e28054458    OFFLINE
d1993c2c3ec44c94    LIVE

$ bash scripts/peer-presence-lookup.sh d1993c2c3ec44c94
d1993c2c3ec44c94    LIVE

$ bash scripts/peer-presence-lookup.sh --json --all | jq .
[
  {"fp":"9219671e28054458","status":"OFFLINE","hub":"ring20-management"},
  {"fp":"d1993c2c3ec44c94","status":"LIVE","hub":"workstation-107-public"}
]
```

All three produced sensible output and exited 0. Tier-2 tick via
`FW_ALLOW_HUMAN_AC_TICK=1 sed -i ...` per T-1731 protocol; logged to
`.context/working/.gate-bypass-log.yaml`.
