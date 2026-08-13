---
id: T-2678
name: "Charter guard-coverage review — non-goal guards, canary false-assurance, unnamed surfaces"
description: >
  Inception: Charter guard-coverage review — non-goal guards, canary false-assurance, unnamed surfaces

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-08-13T23:06:57Z
last_update: 2026-08-13T23:11:00Z
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

# T-2678: Charter guard-coverage review — non-goal guards, canary false-assurance, unnamed surfaces

## Problem Statement

The two prior purpose reviews (T-2419, T-2468) both measured **product against charter**.
Neither asked whether the charter is itself **load-bearing** — i.e. what mechanically
happens when the product drifts from it. PL-271 (registered by T-2483, the last review's
own closing lesson) states that a recurring human "please review purpose" mandate IS the
symptom of a missing structural check. This is the fifth such review run by hand, which is
evidence the loop was not closed.

For: the human operator (who should not have to hand-run this review), and every future
agent that reads `/canaries` green and concludes the charter is being upheld.

Why now: the human asked for an ultra-critical purpose/goals review. Rather than re-run
the same product-vs-charter pass a third time, this explores the guard layer beneath it.

Research artifact: `docs/reports/T-2678-charter-guard-coverage-review.md` (C-001).

## Assumptions

- A-1: The charter decomposes cleanly into two enforceable halves — four verbs (what
  TermLink must do) and five non-goals (what it must refuse). *Verified by reading
  `docs/CHARTER.md`.*
- A-2: An existing guard reporting "healthy" is evidence it measured what it claims.
  *This assumption is the one under test, and it is FALSE — see IW-2.*

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

- **IW-1: Do the charter's four verbs and five non-goals each have a structural guard, and does any artifact assert that matrix?**
  confidence: 3
  disposition: answered
  rationale: Verbs 4/4 (provers T-2482/T-2485/T-2151 + canaries T-2387/T-2558/T-2556/T-2557); non-goals 2/5 (NG2=T-2562, NG3=T-2483-but-weak; NG1 unguarded with T-2569 filed 2026-08-09 still `horizon: later`; NG4 human-owned T-2570; NG5 doc-only). No artifact asserts either matrix — both reconstructed by hand for this review.

- **IW-2: Is the one non-goal guard that exists (T-2483 charter-drift canary) load-bearing for the class it reports on?**
  confidence: 3
  disposition: answered
  rationale: No. Proven with its own PL-213 test hook — `termlink_agent_top_reacted` fires (exit 1), `termlink_agent_top_repliers` (identical class) passes clean (exit 0). It emits `{checked:214, live_off_charter:0}` while structurally blind to 28 LIVE tools in `agent_rankings`/`agent_stats`/`channel_engagement`/`agent_thread_health` — the exact family T-2548 is incepting to subtract. Full-surface assurance for a six-family name regex.

- **IW-3: Are there shipped surfaces the charter does not account for, and does any of them violate a non-goal?**
  confidence: 3
  disposition: answered
  rationale: `kv.*` (5 live tools) is unnamed by the charter but does NOT violate non-goal 2 — it is in-memory, per-session, dies with the session. It does however carry an unbounded-growth defect: `SessionContext.kv` is a plain uncapped `HashMap<String,Value>` (`handler.rs:29`, insert at `handler.rs:966`) reachable at Interact scope (`auth.rs:191`) inside the PTY-owning daemon. Third instance of the class fixed by T-2675/T-2676 last session.

- **IW-4: Is the cron/canary layer actually installed, or shipped-dark (G-069) — i.e. can the guard findings above even be trusted to run?**
  confidence: 3
  disposition: answered
  rationale: Installed. `check-cron-install-drift.sh` → 22 installed-and-matching, 0 MISSING, 2 content-drift warnings (`fleet-doorbell-mail-canary`, `substrate-preflight-canary`). Guard layer runs; the problem is coverage and honesty, not liveness.

- **IW-5: Which gaps are in agent authority to close, and which must stay with the human?**
  confidence: 3
  disposition: answered
  rationale: In-authority — G1 canary honesty (fix reporting, not the surface), G2 NG1 federation tripwire (T-2569 is `owner: agent`), G3 bound `kv`, G4 make the matrix load-bearing, G5 crontab reconcile. Human-only — G6/NG4 orchestration guard (T-2570 `owner: human`) and the off-charter DELETION decision (T-2548 `owner: human`). Nothing built here pre-empts T-2548.

## Exploration Plan

1. **Charter decomposition** (30 min, done) — split `docs/CHARTER.md` into verbs + non-goals.
2. **Guard inventory** (60 min, done) — grep scripts/canaries/tests per charter clause;
   cross-check against `.context/cron/*.crontab` and CLAUDE.md's canary catalogue.
3. **Load-bearing stress test** (30 min, done) — feed each existing guard the violation it
   claims to catch via its PL-213 test hook; confirm fire/no-fire. This is the step prior
   reviews skipped and it produced IW-2.
4. **Unnamed-surface sweep** (30 min, done) — enumerate live MCP categories via
   `termlink help --json`, check each against the four verbs and five non-goals.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

**IN scope:**
- Whether each charter clause (4 verbs, 5 non-goals) has a structural guard.
- Whether existing guards are load-bearing for the class they report on.
- Shipped surfaces the charter does not name, and whether any violates a non-goal.
- Making the guard MATRIX itself load-bearing (the G-019 second half).

**OUT of scope (explicitly):**
- Re-litigating the canonical purpose sentence — it is human-blessed (T-2470) and in sync
  (T-2484 passing). Not touched.
- **Deleting or deprecating any off-charter tool.** That is T-2548's decision and it is
  `owner: human`. G1 is scoped to making the canary honest about what it measured, never
  to removing a surface. An acknowledgement allowlist is the correct instrument precisely
  because it makes the pending human decision *visible* rather than resolving it.
- Non-goal 4 (orchestration) guard — T-2570 is `owner: human`.
- The multi-tenant / per-agent authorization product decision (T-2422, G-064).

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

**Recommendation:** GO

**Rationale:**

Three evidenced gaps in how the charter is made load-bearing. (1) The four charter VERBS have complete prover+canary coverage (4/4 each, T-2482/T-2485/T-2151 provers; T-2556/T-2557 closed the canary axis), but the five NON-GOALS are guarded 2/5: NG2 by T-2562, NG3 partially by T-2483; NG1 (no federation) has zero test — T-2569 was filed 2026-08-09 and has sat at horizon:later ever since; NG4 is human-DEFER (T-2570). Nothing tracks that matrix, so the gap is invisible — the G-019 shape. (2) The T-2483 charter-drift canary is false-assurance by construction: it emits {checked:214, live_off_charter:0} — a full-surface clean bill — while its 6-family name regex structurally cannot see 28 LIVE tools in categories literally named agent_rankings / agent_stats / channel_engagement / agent_thread_health. Proven with its own test hook: termlink_agent_top_reacted FIRES, termlink_agent_top_repliers (a functionally identical leaderboard) passes clean. That is the exact family T-2548 is incepting to subtract, reported as clean. (3) kv.set inserts into an uncapped HashMap<String,Value> in the session daemon that owns real PTYs, reachable at Interact scope — the same unbounded-peer-driven-growth class fixed twice last session (T-2675 PresenceTracker, T-2676 circuit-breaker map). GO on the in-authority subset: bound kv, make the drift canary honest via category-awareness plus an acknowledgement allowlist citing T-2548, build the NG1 federation tripwire, and make the non-goal guard matrix load-bearing. The off-charter DELETION decision stays human (T-2548); none of this pre-empts it.

**Evidence:**

<!-- Add evidence bullets as exploration progresses (file paths,
     commit hashes, test results). The filing-time recommendation
     can be revised before fw inception decide. -->

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

### 2026-08-13T23:08:22Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
