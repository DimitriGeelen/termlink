---
id: T-2311
name: "arc-004 active reconnect-to-WS with backoff — should --push retry the socket after a drop instead of staying on poll?"
description: >
  Inception: arc-004 active reconnect-to-WS with backoff — should --push retry the socket after a drop instead of staying on poll?

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-07-02T18:48:51Z
last_update: 2026-07-02T18:52:31Z
date_finished: null
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
---

# T-2311: arc-004 active reconnect-to-WS with backoff — should --push retry the socket after a drop instead of staying on poll?

## Problem Statement

`channel subscribe <topic> --push` (arc-004 S3b / T-2309) gives long-lived live
agents a sub-second DM path (~90 ms measured, T-2310). But it tries the WebSocket
**once** — on any drop it degrades to the poll loop and **stays on poll until the
process restarts**. A single transient blip (hub restart, NAT rebind, brief
partition) therefore permanently downgrades a long-lived agent from ~90 ms push to
the 1 s poll floor (~10×), silently, with no recovery short of a restart the agent
may never do. **For whom:** persistent live agents (the exact sessions most exposed
to blips). **Why now:** the arc build + demo evidence just landed; this is the one
robustness gap standing between "works" and "stays fast in the field".

## Assumptions

- **A1** — A backoff cap (~5 attempts over ~30 s) before settling to steady poll is
  a sane default; the reconnect must never tight-spin against a genuinely dead hub.
- **A2** — A poll catch-up pass from the durable cursor on each reconnect covers the
  gap without double-rendering events straddling the drop.
- **A3** — No hub-side change is needed; this is purely a CLI consumer-loop change.

## Open Questions

<!-- filed per T-2194/G-067 readiness gate; disposed before work-completed -->

- **IW-1: Should `--push` actively reconnect the WebSocket after a drop, or is permanent degrade-to-poll the correct v1?**
  confidence: 2
  disposition: deferred
  rationale: Agent advises GO (reconnect); final go/no-go is the human's via `fw inception decide` (sovereignty-gated).

- **IW-2: Which reconnect shape preserves the no-miss guarantee at least cost — concurrent poll+WS (A), alternating reconnect-with-poll-catch-up (B), or stay-on-poll-with-WS-probe (C)?**
  confidence: 2
  disposition: deferred
  rationale: Analysis (docs/reports/T-2311-*.md) favours B as bounded + no-miss-preserving; confirm during build (RB2 scripted-drop test).

- **IW-3: Does a poll catch-up + WS resume double-render events straddling the reconnect gap?**
  confidence: 1
  disposition: deferred
  rationale: Needs the RB2 scripted-drop wire test; cursor-based position tracking is the intended dedup mechanism but unproven for the resume path.

<!-- T-2190 (T-2186 Slice 4): every IW-N question must be disposed before
     --status work-completed. Disposition gate (agents/task-create/update-task.sh
     check_disposition_gate) refuses on under-disposed inceptions.

     Per-question shape:

       - **IW-1: <question text>**
         confidence: 0-3      (your confidence in your current answer; 0=guess, 3=verified)
         disposition: answered | deferred | dissolved
         rationale: <one-line evidence — file:line, decision id, dialogue ref>

     Never bare yes/no — the gate refuses bare checkboxes. See 050-Inceptions.md
     §Disposition Gate. Bypass: --skip-disposition-gate "rationale" (direct) or
     FW_SKIP_DISPOSITION_GATE=1 (env-var, T-1890 producer/consumer parity).
-->

## Exploration Plan

Exploration is a code-read + design analysis (no prototype needed — the pieces
already exist and compose):
1. **Read** the current `--push` → poll-loop composition (done: `channel.rs`
   ~8488 branch + poll loop below; `run_ws_push` at ~397 returns
   `WsPushOutcome::{Ended,Unsupported}`/Err).
2. **Enumerate** reconnect shapes (A/B/C) and score against the no-miss invariant
   + change size (done: `docs/reports/T-2311-arc-004-active-reconnect-inception.md`).
3. **Recommend** an approach + suggested build slices + open assumptions (done: GO / Option B).
4. **Human go/no-go** via `fw task review T-2311` (pending — sovereignty-gated).

## Technical Constraints

- **No-miss invariant (hard):** the poll loop reading the durable cursor is the
  authoritative floor and must remain so — WS is a faster transport, never a source
  of truth. Any reconnect design must preserve this (aggregator is live-only, no gap replay).
- **Aggregator is live-only:** a re-subscribed WS starts from "now" — gap events
  posted while the socket was down are NOT replayed over WS; they must be drained
  from the durable cursor.
- **No hub-side change:** consumer-loop-only; no protocol/hub change (A3).
- **Safety:** reconnect must be backoff-capped (no tight-spin against a dead hub);
  `Unsupported` (Unix hub) must still degrade immediately (no WS-over-Unix here).

## Scope Fence

- **IN:** whether/how `channel subscribe --push` should reconnect the WS after a
  drop; approach selection; suggested build slices; open assumptions.
- **OUT:** WS-over-Unix push (separate follow-on / separate inception); any
  hub-side or protocol change; the build itself (post-GO, separate build tasks
  RB1/RB2); active reconnect for any consumer other than the CLI `--push` path.

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [ ] Problem statement validated
<!-- @auto-tick-on-decide -->
- [ ] Assumptions tested
<!-- @auto-tick-on-decide -->
- [ ] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO if:**
- Reconnect can be added as a **bounded** consumer-loop change reusing `run_ws_push`
  + the existing poll loop (no hub/protocol change). — met (Option B)
- The design **preserves the no-miss guarantee** (poll-from-cursor stays
  authoritative; catch-up drains the gap). — met by construction (Option B)
- Reconnect is **safe** (backoff + cap, no tight-spin; Unix still degrades). — met

**NO-GO if:**
- Robustness could only be achieved by making WS a source of truth (breaking the
  no-miss invariant), or by an unbounded concurrent-stream rewrite.
- The permanent-degrade v1 is judged acceptable because agents restart often enough
  that a stuck-on-poll window is not worth the added loop complexity. — the human's call.

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.

## Recommendation

**Recommendation:** GO

**Rationale:**

The --push feature exists to give long-lived live agents a sub-second DM path; a long-lived agent is precisely the one most likely to hit a transient network blip. Today (S3b/T-2309) a single WS drop degrades to the poll loop and STAYS there until process restart — the agent is permanently kneecapped to the 1s poll floor by one blip. Correctness is already safe (the poll loop reads from the durable cursor and never misses), so this is a latency-robustness gap, not a data gap. A bounded reconnect loop (run_ws_push -> on drop do one poll catch-up pass from cursor to drain gap events -> exponential backoff -> retry WS; cap retries then settle to poll) restores the fast path after a blip while preserving the existing no-miss guarantee, and is feasible as one build slice. GO to explore/scope; final decision is the human's.

**Evidence:**

- `crates/termlink-cli/src/commands/channel.rs` ~8488 — the `--push` branch calls
  `run_ws_push` **once**, prints a degrade notice on `Ended`/`Unsupported`/`Err`,
  then falls through to the poll loop below and never returns to WS (the gap).
- `run_ws_push` (~397) returns `WsPushOutcome::{Ended,Unsupported}` / `Err` — a
  clean seam to wrap in an outer reconnect loop.
- The poll loop reads from the durable `cursor` — the no-miss floor is already
  in place; reconnect must preserve it (drain the gap from cursor on reconnect).
- T-2310 demo: measured push ~90 ms vs 1 s poll floor — quantifies the ~10×
  regression a stuck-on-poll agent suffers after a blip.
- Full analysis + option scoring: `docs/reports/T-2311-arc-004-active-reconnect-inception.md`.

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

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-07-02T18:49:58Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
