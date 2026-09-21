---
id: T-2989
name: "Inception: CLAUDE.md split design (clobber-safe, <20k tokens preload)"
description: >
  S-15/C-04: CLAUDE.md preload cost is measured as excessive; design a clobber-safe
  split (T-2015 positional boundary respected) targeting <20k tokens preload. Human
  ratifies the approach (run4 Q5). Evidence: docs/reports/VALUE-REVIEW-repo-2026-09-19-consolidated.md
  C-04.

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [value-review, arc:arc-009]
components: []
related_tasks: []
created: 2026-09-19T22:12:55Z
last_update: 2026-09-20T19:52:08Z
date_finished:
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
bvp_scores_proposed:
  - ts: '2026-09-20T08:45:10Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 2
      D3: 2
      D4: 2
      F-RECALL: 2
      F-ORCH: 2
    rationale: D1=2 (no-signal); D2=2 (no-signal); D3=2 (no-signal); D4=2 
      (no-signal); F-RECALL=2 (no-signal); F-ORCH=2 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-20T08:45:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 4
      effort: 6
    rationale: blast_radius=3 (target_blast_radius:inception-T-2189); tier=4 
      (workflow:inception); effort=6 (lines=115,acs=4)
    rubric_sha: e4a00f38e801
---

# T-2989: Inception: CLAUDE.md split design (clobber-safe, <20k tokens preload)

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

CLAUDE.md is preloaded into every session in this project. Measured at 276,749
bytes (~69k–77k tokens) it consumes roughly **a quarter of a 300k context window
before any code is read**, against a stated target of <20k. For whom: every agent
session, on every turn, permanently. Why now: the file grew **+790 lines in one
month** and the growth is accelerating, because the practice the file itself
documents (T-2015: put project rules ABOVE `## Core Principle` so `fw upgrade`
cannot clobber them) directs new content straight into the preloaded half.

Full measurements and design: `docs/reports/T-2989-claude-md-split-design.md`.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

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

- **IW-1: What is CLAUDE.md's actual preload cost, and is it excessive against a stated budget?**
  confidence: 3
  disposition: answered
  rationale: Measured 276,749 bytes / 3,283 lines = ~69k-77k tokens vs <20k target — 3.5-3.8x over; ~25% of a 300k window consumed before any code is read (docs/reports/T-2989-claude-md-split-design.md §Measurements)

- **IW-2: Does a mechanism exist for CLAUDE.md to load content ON DEMAND (imports, @-references, skills), or is every byte necessarily preloaded into every session?**
  confidence: 2
  disposition: deferred
  rationale: Skills ARE on-demand (34 in .claude/commands/, bodies not preloaded); `@`-import laziness is UNVERIFIED and load-bearing — 0 imports in use today, so no local evidence. Must be confirmed before build work; see artifact §F4

- **IW-3: Where does the T-2015 clobber boundary (`## Core Principle` → EOF) fall today, and how much of the file is in the half `fw upgrade` destroys?**
  confidence: 3
  disposition: answered
  rationale: Boundary at line 2461; 2,460 lines survive, 823 destroyed. CLAUDE.md's own record (L1149: "2493 lines, boundary 1650, 844 destroyed") is STALE — +790 lines since, ~all above the boundary

- **IW-4: If content is moved out of the preload, what stops it from simply never being read again?**
  confidence: 3
  disposition: answered
  rationale: An existing TRIGGERED path to the reader — skills reached by invocation, canary docs reached when /canaries names the failing canary whose script header carries the remediation. Content with no such path must be deleted or kept, never moved (artifact §IW-4)

## Exploration Plan

<!-- How will we validate assumptions? Spikes, prototypes, research? Time-box each. -->

All four spikes were read-only measurements against the working tree; no CLAUDE.md
bytes were moved. Executed in this order:

1. **Size** — `wc`, byte count, token estimate at two ratios → IW-1.
2. **Boundary** — locate `## Core Principle`, split line counts above/below, and
   compare against the file's own recorded measurement → IW-3.
3. **Distribution** — parse headings, attribute bytes per section, rank → showed
   two blocks carry 72% of the file, which is what made the design actionable.
4. **Mechanisms** — count `@`-imports in use (0) and skills present (34) → IW-2,
   and surfaced the one load-bearing fact that could NOT be settled locally.

Deviation worth recording: the artifact skeleton was written after spike 1 rather
than before, because the Open-Questions gate correctly refused the write until the
IW questions were filed. The gate enforced the right order — declare the questions,
then research — which is stricter than C-001 alone.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

<!-- What's IN scope for this exploration? What's explicitly OUT? -->

**IN:** measuring the real preload cost; locating the clobber boundary; finding
where the bytes actually are; identifying which content has an existing on-demand
path to its reader; producing a dependency-ordered reduction plan.

**OUT — deliberately:**
- Executing the reduction. This is an inception; no CLAUDE.md bytes were moved.
- Relocating content across the `## Core Principle` boundary *as a goal*. The
  clobber-safe half IS the preloaded half, so that trade is the tension itself
  (artifact §F1), and it interacts with the pending re-vendor decision
  (T-2950/T-2949) — a governance question, not a token-budget one.
- Deciding go/no-go. `owner: human`.

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

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- Root cause identified with bounded fix path
- Fix is scoped, testable, and reversible

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope
- Fix cost exceeds benefit given current evidence

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

**Recommendation:** GO — with a scope materially different from the one this
section carried before the exploration ran.

**Rationale:** The premise is confirmed and was understated: ~69–77k tokens
against a <20k target, ~25% of the window before any code is read. But the
*implied remedy* — reorganise around the clobber boundary — would make the preload
worse, because the clobber-safe half IS the preloaded half. Following T-2015's own
advice is what put +790 lines into the preload in one month (artifact §F1).

Two blocks are **72% of the file** and both have an existing on-demand home:
- `### TermLink Substrate + Skills Reference`, ~18k tokens, duplicates a catalogue
  the 34 skills in `.claude/commands/` already surface lazily (§F2).
- The canary/guard block, ~37k tokens, duplicates remediation text already held in
  each check script's header and `docs/operations/` (§F3).

**Dependency-ordered scope:** (1) confirm whether a `@path` import in CLAUDE.md
defers loading or is inlined — UNVERIFIED and load-bearing; if inlined, an
import-based split saves zero and only genuinely on-demand surfaces help (§F4).
(2) reduce the skills reference to a pointer. (3) move the canary block to
`docs/operations/`, leaving an index. (4) re-measure; the residual gap is the Hub
Auth Rotation Protocol (~6k).

**Do not start (2)–(3) before (1) is answered** — assuming it wrong produces a
refactor that moves 55k tokens between files and reduces preload by nothing.

**Note on the prior text:** this section arrived pre-filled with "GO" and a
rationale citing "844 lines in the destroyed half" — a figure copied from a stale
2026-08-20 measurement in CLAUDE.md line 1149. The measured figure is 823, the
file is 3,283 lines not 2,493, and the destroyed half is not where the cost is.
The conclusion survives; none of its stated reasons did.

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

### 2026-09-19T22:35:29Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-009

### 2026-09-20T19:52:08Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
