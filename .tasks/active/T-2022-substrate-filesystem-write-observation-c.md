---
id: T-2022
name: "Substrate: filesystem-write observation (CSMA/CD-style collision detection)"
description: >
  §6 primitive 4 (Keystone, SOFT per §9). Nothing watches agent file I/O at runtime.
  This is what would let the system PHYSICALLY detect collisions and therefore makes
  a future optimistic mode safe per §4. novel_mechanism: yes, biggest single build,
  expected to split into sub-pieces. Absence FORCES conservative launch policy; presence
  is precondition (not trigger) for optimistic.

status: captured
workflow_type: inception
owner: human
horizon: later
tags: [arc:arc-parallel-substrate, novel-mechanism]
components: []
related_tasks: [T-2018]
created: 2026-06-07T11:36:29Z
last_update: 2026-06-08T11:20:49Z
date_finished:
revisit_at: 2026-09-08            # T-1451: DEFER pending Foundation primitives + AEF serialization-cost evidence
revisit_evidence_needed: "Either (a) AEF-layer incident attributable to lacking write-observation; (b) successful git-hook path-declaration spike (T-2022a); or (c) ring20 deployment-shape change that opens up CAP_BPF or CAP_SYS_ADMIN."
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
bvp_scores_proposed:
  - ts: '2026-06-07T11:41:30Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 2
      D3: 2
      D4: 2
    rationale: D1=2 (no-signal); D2=2 (no-signal); D3=2 (no-signal); D4=2 
      (no-signal)
    rubric_sha: missing
---

# T-2022: Substrate: filesystem-write observation (CSMA/CD-style collision detection)

## Problem Statement

§6 primitive of [arc-parallel-substrate](../../docs/architecture/parallel-execution-substrate.md). **Group:** Keystone. **§9 boundary:** **SOFT per §9** — co-discovered with AEF layer, NOT pre-contracted. The only ADR primitive that is genuinely two-sided unknown..

**Role per ADR §6:** An exhaustive code search found no inotify / fanotify / fs-watch / equivalent at runtime. The hub cannot see what files an agent touches. This absence is what FORCES the conservative collision policy in §4. Its presence is the *precondition* (not the trigger) for an optimistic mode.

**Why captured now:** Capture-while-fresh per [[PL-203]]. Likely splits into sub-tasks at design phase — this inception is primarily an unblocker for an Inception sub-arc.

**Status disclosure ([[PL-203]]):** Filed `status: captured, horizon: later`. BVP scores will be estimator-proposed (not confirmed) until promotion. No commitment to RPC shape, build order, or cost-estimate is locked at filing. Per-primitive design phase opens at operator promotion via `fw bvp confirm` + `--horizon now`.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

- **IW-1: Mechanism — inotify, fanotify, ptrace, LD_PRELOAD wrapper, eBPF, FUSE?**
  confidence: 4
  disposition: resolved
  rationale: NONE at OS level. Mechanism survey (docs/reports/T-2022-fs-write-observation-inception.md §2): inotify=Linux-only (blocks macOS); fanotify+eBPF=CAP requirements not available in container deployment; ptrace=10× syscall slowdown unacceptable; LD_PRELOAD=Rust binaries variable + statically-linked bypass; FUSE=breaks POSIX semantics. The intersection of (portable across Linux+macOS) ∧ (works in capability-dropped containers) ∧ (catches Rust agent writes) ∧ (acceptable perf) is EMPTY. Re-scope to git-hook-enforced path declaration. See artifact §3.

- **IW-2: Per-ring20-host viability — does the chosen mechanism work on every host? Container restrictions?**
  confidence: 4
  disposition: resolved
  rationale: OS-LEVEL MECHANISMS FAIL on portability OR container caps for ring20. Git-hook approach is portable by construction (every host already has git). See artifact §2-§3.

- **IW-3: Blind spots — what file ops are NOT observable?**
  confidence: 3
  disposition: resolved
  rationale: For OS-level: every mechanism has a blind spot that breaks the §4 soundness argument. For git-hook: blind spot = unstaged scratch writes that never become commits — but those don't matter because they never persist to shared state. The orchestrator only cares about writes that AGREE WITH OTHER AGENTS' WORK at merge time. See artifact §3, §5.IW-3.

- **IW-4: Cost — per-syscall overhead, kernel buffer pressure, scaling with concurrent agents?**
  confidence: 4
  disposition: resolved
  rationale: OS-level: prohibitive for most mechanisms (ptrace 10×, FUSE severe). Git-hook: one network post per commit (typically O(seconds) apart per agent), negligible cost. Scales linearly with commit rate, not write rate. See artifact §5.IW-4.

- **IW-5: Granularity — directory-level, file-level, byte-range — and what does AEF layer need?**
  confidence: 4
  disposition: resolved
  rationale: FILE-LEVEL is sufficient. AEF needs "is anyone else touching path X" — directory is coarse, byte-range is overkill. Git already operates at file granularity, matching the natural abstraction. See artifact §5.IW-5.

## Exploration Plan

**Expected to split.** First step at promotion time: 1-week spike comparing inotify vs fanotify vs LD_PRELOAD on ring20 hosts under simulated agent load. Likely produces a sub-arc rather than a single build task.

## Technical Constraints

**Dependencies (upstream):** None (independent mechanism question)

**Dependencies (downstream):** **THIS IS THE GATE ON THE CONSERVATIVE→OPTIMISTIC FLIP** (§4). Without write-observation, optimistic mode is unsafe. Shipping this does NOT flip policy — that's a separate operator-gated decision against criteria defined in advance (§4 two-step that must not be collapsed).

**ADR §9 boundary:** **SOFT per §9** — co-discovered with AEF layer, NOT pre-contracted. The only ADR primitive that is genuinely two-sided unknown.

## Scope Fence

**IN scope (this inception):** Validate that §6's description still holds in light of what's been learned from earlier primitives. Refine open questions into a design proposal. Recommend GO / NO-GO / DEFER with rationale. Surface any newly-discovered sub-decomposition.

**OUT of scope (this inception):** Build/code work — that's a follow-on task created on GO. Other primitives' shapes — they have their own tasks. AEF orchestration layer integration — that's the §9 collaboration seam, owned at the boundary.

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [x] Problem statement validated
<!-- @auto-tick-on-decide -->
- [x] Assumptions tested
<!-- @auto-tick-on-decide -->
- [x] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO if:**
- Mechanism chosen with measured blind-spot list and per-host viability confirmed; sub-task decomposition produced.

**NO-GO if:**
- All candidate mechanisms have blind spots that invalidate them for the conservative→optimistic gate (in which case the substrate ships nothing and policy stays conservative forever — that's a valid outcome per §4).

**DEFER if:**
- Predecessor primitives have shifted shape in ways that change the open questions; capture the shift, update the ADR, re-file.

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

**Recommendation:** DEFER as captured (OS-level FS observation is not viable for ring20). Re-scope to a three-spike sub-arc exploring git-hook-enforced path declaration. revisit_at=2026-09-08.

**Rationale (one-paragraph):** OS-level FS observation as a substrate primitive is not viable in the ring20 deployment. The mechanism survey (artifact §2) finds an empty intersection of (portable across Linux+macOS) ∧ (works in capability-dropped containers) ∧ (catches Rust agent writes) ∧ (acceptable perf cost). HOWEVER, the *concern* the primitive addresses — detecting parallel writes to the same file for the conservative→optimistic flip — is real and worth resolving at a different abstraction layer. The key insight: §4's "honor-system is unsafe" worry assumes *voluntary* announcement; if announcement is structurally enforced via a pre-commit git hook (the agent can't bypass without disabling the hook, which is itself observable), it moves from "honor-system" to "hook-enforced declaration" — sound for the ring20 cooperating-agent trust model. File granularity matches what AEF actually needs (git already operates there). Three small spikes test this re-scoping. If they succeed, the substrate gets the capability §6 #4 asks for at a tractable abstraction; if they fail, T-2022c falls back to kernel-mechanism on the Linux subset only.

**Full analysis:** see [docs/reports/T-2022-fs-write-observation-inception.md](../../docs/reports/T-2022-fs-write-observation-inception.md).

**Re-scoped sub-arc (file on DEFER, run after Foundation primitives land):**

**Spike T-2022a — git-hook path declaration (~80 LOC, ≤1 session):**
- Pre-commit hook on AEF worktrees posts `{agent_id, branch, paths_modified, paths_added, paths_deleted}` to a coordination topic.
- Hub maintains sliding-window view: which agents have declared which paths in last N min.
- Orchestrator queries before dispatching; conservative-applies on overlap.

**Spike T-2022b — bypass detection (~50 LOC, ≤1 session):**
- Test: agent disables hook + commits. Does orchestrator notice?
- Mechanism: count commits per agent vs declared path-sets; divergence → alert.
- Addresses §4's "agent that forgets to announce" failure mode.

**Spike T-2022c — kernel-mechanism fallback (1-2 sessions, CONDITIONAL):**
- Only if 2022a OR 2022b fail.
- inotify on Linux-only subset, dropping macOS coverage.

**GO criteria evaluation (from §Go/No-Go Criteria):**
- ❌ "Mechanism chosen with measured blind-spot list" — OS-level: blind spots are catastrophic. Git-hook: blind spots are tolerable but unverified — that's exactly what the spikes test.
- ⏸ "Per-host viability confirmed" — OS-level: no. Git-hook: by construction yes, but unproven at scale.
- ⏸ "Sub-task decomposition produced" — yes, 3 spikes above. Not auto-GO until spikes run.

**Why DEFER vs full NO-GO:** Conservative policy is correct today but expensive — serializes work that could parallelize. ROI on cracking this is real, just not via the captured mechanism. NO-GO would close the question; DEFER + spike-arc leaves it productively open.

**Documentation follow-up:** add §3's "OS-level vs git-hook trade-off" reasoning to `docs/architecture/parallel-execution-substrate.md` as a §4 addendum.

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

**Decision**: DEFER

**Rationale**: Recommendation: DEFER as captured (OS-level FS observation is not viable for ring20). Re-scope to a three-spike sub-arc exploring git-hook-enforced path declaration. revisit_at=2026-09-08.

Rationale (one-paragraph): OS-level FS observation as a substrate primitive is not viable in the ring20 deployment. The mechanism survey (artifact §2) finds an empty intersection of (portable across Linux+macOS) ∧ (works in capability-dropped containers) ∧ (catches Rust agent writes) ∧ (acceptable perf cost). HOWEVER, the concern the primitive addresses — detecting parallel writes to the same file for the conservative→optimistic flip — is real and worth resolving at a different abstraction layer. The key insight: §4's "honor-system is unsafe" worry assumes voluntary announcement; if announcement is structurally enforced via a pre-commit git hook (the agent can't bypass without disabling the hook, which is itself observable), it moves from "honor-system" to "hook-enforced declaration" — sound for the ring20 cooperating-agent trust model. File granularity matches what AEF actually needs (git already operates there). Three small spikes test this re-scoping. If they succeed, the substrate gets the capability §6 #4 asks for at a tractable abstraction; if they fail, T-2022c falls back to kernel-mechanism on the Linux subset only.

Full analysis: see [docs/reports/T-2022-fs-write-observation-inception.md](../../docs/reports/T-2022-fs-write-observation-inception.md).

Re-scoped sub-arc (file on DEFER, run after Foundation primitives land):

Spike T-2022a — git-hook path declaration (~80 LOC, ≤1 session):
- Pre-commit hook on AEF worktrees posts `{agent_id, branch, paths_modified, paths_added, paths_deleted}` to a coordination topic.
- Hub maintains sliding-window view: which agents have declared which paths in last N min.
- Orchestrator queries before dispatching; conservative-applies on overlap.

Spike T-2022b — bypass detection (~50 LOC, ≤1 session):
- Test: agent disables hook + commits. Does orchestrator notice?
- Mechanism: count commits per agent vs declared path-sets; divergence → alert.
- Addresses §4's "agent that forgets to announce" failure mode.

Spike T-2022c — kernel-mechanism fallback (1-2 sessions, CONDITIONAL):
- Only if 2022a OR 2022b fail.
- inotify on Linux-only subset, dropping macOS coverage.

GO criteria evaluation (from §Go/No-Go Criteria):
- ❌ "Mechanism chosen with measured blind-spot list" — OS-level: blind spots are catastrophic. Git-hook: blind spots are tolerable but unverified — that's exactly what the spikes test.
- ⏸ "Per-host viability confirmed" — OS-level: no. Git-hook: by construction yes, but unproven at scale.
- ⏸ "Sub-task decomposition produced" — yes, 3 spikes above. Not auto-GO until spikes run.

Why DEFER vs full NO-GO: Conservative policy is correct today but expensive — serializes work that could parallelize. ROI on cracking this is real, just not via the captured mechanism. NO-GO would close the question; DEFER + spike-arc leaves it productively open.

Documentation follow-up: add §3's "OS-level vs git-hook trade-off" reasoning to `docs/architecture/parallel-execution-substrate.md` as a §4 addendum.

**Date**: 2026-06-08T11:20:49Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-06-08T07:45:43Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-06-08T11:20:49Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** DEFER
- **Rationale:** Recommendation: DEFER as captured (OS-level FS observation is not viable for ring20). Re-scope to a three-spike sub-arc exploring git-hook-enforced path declaration. revisit_at=2026-09-08.

Rationale (one-paragraph): OS-level FS observation as a substrate primitive is not viable in the ring20 deployment. The mechanism survey (artifact §2) finds an empty intersection of (portable across Linux+macOS) ∧ (works in capability-dropped containers) ∧ (catches Rust agent writes) ∧ (acceptable perf cost). HOWEVER, the concern the primitive addresses — detecting parallel writes to the same file for the conservative→optimistic flip — is real and worth resolving at a different abstraction layer. The key insight: §4's "honor-system is unsafe" worry assumes voluntary announcement; if announcement is structurally enforced via a pre-commit git hook (the agent can't bypass without disabling the hook, which is itself observable), it moves from "honor-system" to "hook-enforced declaration" — sound for the ring20 cooperating-agent trust model. File granularity matches what AEF actually needs (git already operates there). Three small spikes test this re-scoping. If they succeed, the substrate gets the capability §6 #4 asks for at a tractable abstraction; if they fail, T-2022c falls back to kernel-mechanism on the Linux subset only.

Full analysis: see [docs/reports/T-2022-fs-write-observation-inception.md](../../docs/reports/T-2022-fs-write-observation-inception.md).

Re-scoped sub-arc (file on DEFER, run after Foundation primitives land):

Spike T-2022a — git-hook path declaration (~80 LOC, ≤1 session):
- Pre-commit hook on AEF worktrees posts `{agent_id, branch, paths_modified, paths_added, paths_deleted}` to a coordination topic.
- Hub maintains sliding-window view: which agents have declared which paths in last N min.
- Orchestrator queries before dispatching; conservative-applies on overlap.

Spike T-2022b — bypass detection (~50 LOC, ≤1 session):
- Test: agent disables hook + commits. Does orchestrator notice?
- Mechanism: count commits per agent vs declared path-sets; divergence → alert.
- Addresses §4's "agent that forgets to announce" failure mode.

Spike T-2022c — kernel-mechanism fallback (1-2 sessions, CONDITIONAL):
- Only if 2022a OR 2022b fail.
- inotify on Linux-only subset, dropping macOS coverage.

GO criteria evaluation (from §Go/No-Go Criteria):
- ❌ "Mechanism chosen with measured blind-spot list" — OS-level: blind spots are catastrophic. Git-hook: blind spots are tolerable but unverified — that's exactly what the spikes test.
- ⏸ "Per-host viability confirmed" — OS-level: no. Git-hook: by construction yes, but unproven at scale.
- ⏸ "Sub-task decomposition produced" — yes, 3 spikes above. Not auto-GO until spikes run.

Why DEFER vs full NO-GO: Conservative policy is correct today but expensive — serializes work that could parallelize. ROI on cracking this is real, just not via the captured mechanism. NO-GO would close the question; DEFER + spike-arc leaves it productively open.

Documentation follow-up: add §3's "OS-level vs git-hook trade-off" reasoning to `docs/architecture/parallel-execution-substrate.md` as a §4 addendum.

### 2026-06-08T11:20:49Z — status-update [task-update-agent]
- **Change:** horizon: now → later
- **Change:** status: started-work → captured (auto-sync)
- **Reason:** Inception decision: DEFER — parking task
