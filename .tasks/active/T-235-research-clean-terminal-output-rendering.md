---
id: T-235
name: "Research clean terminal output rendering for mirror mode"
description: >
  Research clean terminal output rendering for mirror mode

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-23T09:09:57Z
last_update: 2026-04-22T10:03:15Z
date_finished: null
---

# T-235: Research clean terminal output rendering for mirror mode

## Problem Statement

`termlink mirror` (T-234) pass-through writes source PTY bytes to viewer stdout unmodified (`cmd_mirror` / `mirror_loop` at `crates/termlink-cli/src/commands/pty.rs:595-684`). For cursor-addressable TUIs (vim/htop/less) and under geometry drift (viewer cols ≠ source cols), this produces visual artefacts. Multi-session mirror (T-236) needs a grid abstraction that raw bytes cannot provide. Research target: a rendering strategy that is clean for TUIs, small in binary cost, and reusable for T-236. Full research artifact: `docs/reports/T-235-terminal-output-rendering.md`.

## Assumptions

- A-1: Mirror's primary use case is TUI apps, not scrollback logs (partially tested — episodic scan found 17 mirror-related entries, mostly T-232/234/236 design notes, no strong evidence of heavy real-world TUI mirroring yet)
- A-2: vte + minimal grid (~200-400 LoC) covers 90% of real-world PTYs (untested — needs spike)
- A-3: vte `Perform` trait API is stable (evidence: no breaking changes 0.10→0.13)
- A-4: T-236 multi-session grid is a credible follow-on (captured at horizon:next, not abandoned)

## Exploration Plan

1. Crate landscape survey (done) — vte / alacritty_terminal / termwiz / ansi-parser compared on fidelity / complexity / T-236-fit.
2. Current-state read (done) — byte-faithful pass-through + existing hand-rolled `strip_ansi_codes` filter traced.
3. Recommendation drafted (done) — approach C, vte + minimal grid.
4. Untested spikes (deferred to post-GO build task): `impl vte::Perform for Grid`, golden-output tests against vim/htop/less, binary-size delta measurement.

## Technical Constraints

- No TTY-on-viewer assumption (mirror can run in a pipe).
- Variable geometry between source and viewer.
- Unicode width (CJK, emoji, combining marks).
- Small release binary target (musl + Homebrew distribution).
- Safe Rust only.
- Streaming incremental render, not buffered redraw.

## Scope Fence

**IN:** rendering-strategy recommendation; fit for T-236; trade-off analysis.
**OUT:** implementation; `--strip-ansi` filter changes (orthogonal); alt-screen buffering across disconnects; graphics protocols.

## Acceptance Criteria

### Agent
- [x] Problem statement validated (code read + artefact surface identified)
- [x] Recommendation written with rationale (GO on approach C, vte + in-process grid)
- [ ] Assumptions fully tested — A-1 partial, A-3 confirmed; A-2 and A-4 deferred to post-GO spike

### Human
- [ ] [REVIEW] Approve GO/NO-GO on approach C (vte + minimal grid) vs alternatives in the artifact
  **Steps:**
  1. Read the Recommendation and Rejected sections in `docs/reports/T-235-terminal-output-rendering.md`
  2. If GO: run `.agentic-framework/bin/fw inception decide T-235 go --rationale "..."`, then create a build task from the "Follow-on Build Task Scope" section of the artifact
  3. If NO-GO or needs a different approach: annotate the Dialogue Log in the artifact and run `.agentic-framework/bin/fw inception decide T-235 no-go --rationale "..."`
  **Expected:** Decision recorded, follow-on build task created (if GO), or artifact updated with why-not (if NO-GO)
  **If not:** Spike A-2 (vte grid 200-LoC coverage against vim/htop/less) is the fastest way to move confidence — timebox 2-4h

## Go/No-Go Criteria

**GO if:**
- vte + minimal grid ≤400 LoC covers bash/vim/htop/less in a spike
- Binary size delta <100 KB
- T-236 remains a credible follow-on

**NO-GO if:**
- Grid blows the LoC budget (>1000) → switch to alacritty_terminal's grid (pay binary cost)
- vte callback surface has hidden gotchas (DCS/OSC passthrough) that can't be ignored safely
- T-236 is abandoned AND A-1 holds — then ship A+B-only and close T-235

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     For inception tasks, verification is often not needed (decisions, not code).
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

## Recommendation

**GO on approach C — `vte` tokeniser + in-process minimal grid.**

- Smallest dep footprint that still gives cursor-addressable fidelity (vte ≈120 KB, zero-tree).
- Natural grid abstraction for T-236 multi-session composition.
- Unknown CSI sequences become observable events (unhandled callbacks countable) → supports antifragility.
- A grid model makes the viewer's output a deterministic function of the source's frame stream → supports reliability.

Rejected: D (alacritty_terminal — right for a renderer, over-scoped + transitive bloat for a CLI mirror), E (termwiz — render-layer overlap with our stdout-owning `mirror_loop`), B-only (already implemented as `--strip-ansi`; lossy), A status-quo (broken for TUIs/geometry drift). Full rationale in `docs/reports/T-235-terminal-output-rendering.md`.

## Decision

<!-- Filled by human via: .agentic-framework/bin/fw inception decide T-235 go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-26 — research-artifact [agent]
- **Artifact:** `docs/reports/T-235-terminal-output-rendering.md`

### 2026-03-23T09:10:09Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-22T04:52:50Z — status-update [task-update-agent]
- **Change:** horizon: later → next
- **Change:** status: started-work → captured (auto-sync)

### 2026-04-22T10:03:15Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
