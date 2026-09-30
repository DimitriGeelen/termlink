---
id: T-3291
name: "Session lifecycle leak - zombie register processes and stale registration files
  accumulate"
description: >
  Inception: Session lifecycle leak - zombie register processes and stale registration
  files accumulate

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-09-30T21:17:16Z
last_update: 2026-09-30T21:18:01Z
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
  - ts: '2026-09-30T21:18:01Z'
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
---

# T-3291: Session lifecycle leak - zombie register processes and stale registration files accumulate

## Problem Statement

Measured 2026-09-30 while preparing the hub restart (T-3290). On this host:
- `/var/lib/termlink/sessions/` holds **10,885** registration files.
- About **460** `termlink register` processes are running, 422 of them on a binary file that has since been replaced.

The operator is certain these are not all genuinely active. This is debt that builds up: every hub scan walks the files, and every process holds memory and a socket. It grows until it drags the host down. The hub supervisor has a sweep (`crates/termlink-hub/src/supervisor.rs::sweep`, `liveness::cleanup_stale`) that should remove dead registrations, and it evidently does not keep up. Operator ruling: do an RCA, then a **structural** remediation, so that sessions exit properly and zombies (processes and files) cannot accumulate.

## Assumptions

- A1: Most of the ~460 register processes are not used by anyone: no live parent agent, no client traffic.
- A2: Most of the 10,885 files belong to dead pids (the sweep is not removing them), not to live processes.
- A3: The leak has a small number of identifiable producers (specific spawners or scripts), not a uniform background.

## Open Questions

- **IW-1: What are the ~460 register processes: who spawned them, how old are they, is the parent alive, is anything using them?**
  confidence: 3
  disposition: answered
  rationale: 508 live; ~483 idle detached tl-* tmux shells (95%), 208 >7d; producers vendored fw termlink dispatch (434) + claude-fw --termlink (46); register exits only on SIGINT (session.rs:467-493) — docs/reports/T-3291-iw1-processes.md
- **IW-2: Why does the hub sweep leave 10,885 files: are they dead-pid, live-pid, or unparseable, and does the sweep run and fail, or never reach them?**
  confidence: 3
  disposition: answered
  rationale: 10,144 orphan *.sock.data; sweep runs every 30s and deletes .json+.sock only (liveness.rs:44-55) so .sock.data is unfindable after; /tmp/termlink-0 pool never swept (discovery.rs:44-46) — docs/reports/T-3291-iw2-files.md
- **IW-3: What is the structural fix: exit on parent death, a TTL, a sweep repair, a canary, or several?**
  confidence: 2
  disposition: answered
  rationale: four slices S1-S4 (file reap, shell-exit ends register, zombie reap + upstream exit, session-leak canary) — docs/reports/T-3291-session-lifecycle-leak-rca.md § Recommendation

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

Read-only measurement only; nothing is killed or deleted during the RCA.
1. Processes (IW-1): classify all live `termlink register` pids by age, parent chain (live or reparented to init), cmdline `--name` family, tty/tmux, and socket activity. Time-box: 30 min.
2. Files (IW-2): classify all registration files by pid liveness, age, parseability and name family, and trace why `sweep`/`cleanup_stale` skips them (code plus hub logs). Time-box: 30 min.
3. Producers: map the name families back to the scripts or spawners that create them.
4. Design options for IW-3 with a recommendation, in `docs/reports/T-3291-session-lifecycle-leak-rca.md`.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

<!-- What's IN scope for this exploration? What's explicitly OUT? -->

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

Two independent leaks, each with an identified root cause and a bounded local fix: orphan `.sock.data` files (the sweep never deletes that third file) and zombie `register` sessions (nothing ends a spawned shell, and `register` never exits on shell exit). Four slices, S1–S4, ordered by risk. Only S3's one-time reap of today's zombies needs operator authority per run, through runme.

**Evidence:**

- docs/reports/T-3291-session-lifecycle-leak-rca.md (synthesis + slices)
- docs/reports/T-3291-iw1-processes.md — 508 procs, 483 idle, producers, exit-path file:line
- docs/reports/T-3291-iw2-files.md — 10,144 orphan .sock.data, sweep predicate liveness.rs:44-55, journal 845 sweeps

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

### 2026-09-30T21:18:01Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
