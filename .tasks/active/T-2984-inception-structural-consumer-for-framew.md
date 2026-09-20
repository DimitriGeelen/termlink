---
id: T-2984
name: "Inception: structural consumer for framework:pickup (G-063)"
description: >
  S-6/C-22 consumer half: termlink has no automatic consumer of framework:pickup;
  the daily canary only detects backlog. Explore a structural consumer (auto-triage/routing)
  and produce a go/no-go. Evidence: docs/reports/VALUE-REVIEW-repo-2026-09-19-consolidated.md
  C-22.

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [value-review, arc:arc-009]
components: []
related_tasks: []
created: 2026-09-19T22:08:13Z
last_update: 2026-09-20T20:15:14Z
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
  - ts: '2026-09-20T08:45:19Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 4
      effort: 6
    rationale: blast_radius=3 (target_blast_radius:inception-T-2189); tier=4 
      (workflow:inception); effort=6 (lines=115,acs=4)
    rubric_sha: e4a00f38e801
---

# T-2984: Inception: structural consumer for framework:pickup (G-063)

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

`framework:pickup` is the hub topic peer projects post bug reports, RCAs and
proposals to. Termlink is a prolific **poster** to it and has never been a
**reader**: `lib/pickup.sh` contains zero references to the topic, and the only
code that touches it (`lib/pickup-channel-bridge.sh`) is outbound-only. That is
the G-063 write-only-sink class — the one that let a high-severity ring20 RCA sit
~27h unprocessed, and a stranded envelope (P-043) sit 73 days while the bug it
reported was independently re-discovered and re-fixed.

What the exploration changed: the project already HAS an automatic pickup consumer
running every 60 seconds — it is simply wired to the file inbox, not the topic. So
the question is not "build a consumer" but "should the existing one be pointed at a
second source", and the measurements say not yet, and not for the stated reason.

Full measurements: `docs/reports/T-2984-pickup-consumer-design.md`.

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

- **IW-1: Does an automatic INBOUND consumer of `framework:pickup` exist in this project today, or is every consumption path manual?**
  confidence: 3
  disposition: answered
  rationale: A consumer exists but is wired to a DIFFERENT rail — `fw pickup process` drains the FILE inbox every 60s (cron+flock); `lib/pickup.sh` has zero refs to `framework:pickup`, and the only topic code (`lib/pickup-channel-bridge.sh`, 106 lines) is outbound-only. The hub topic has no reader (artifact §Measurements)

- **IW-2: What is the real inbound backlog — how many peer filings are unprocessed, and how old is the oldest?**
  confidence: 3
  disposition: answered
  rationale: ONE. Acked to offset 125; 6 filings since, of which 5 are ours (4 confirmed against our own commit messages) and only off=131 (proxmox-ring20-management) is genuine inbound. The canary's firing set is 67% self-echo

- **IW-3: What decision would a structural consumer have to make per filing, and is that decision agent-delegable or sovereign?**
  confidence: 2
  disposition: answered
  rationale: ignore / file / route / defer. File-and-route are judgements about whether a peer's report matters here — the same judgement the 150-deep review queue records and the human cannot keep up with. Delegable only once a drain exists; until then sovereign (artifact §IW-3)

- **IW-4: If a consumer auto-files tasks, where do they land — and what stops it from enlarging a review queue that already holds 58 items older than 30 days?**
  confidence: 3
  disposition: answered
  rationale: Nothing stops it, and the queue is deeper than the filing assumed — 150 awaiting review, 143 active owner:human. The existing auto-filer ALREADY duplicates: 9 groups / 11 extra files / 70 Pickup: tasks (16%), at exactly the 60s drain interval

## Exploration Plan

<!-- How will we validate assumptions? Spikes, prototypes, research? Time-box each. -->

Four read-only spikes against the working tree. No envelope was consumed, acked,
promoted or filed; the canary was run in its plain (non-`--ack`) form.

1. **Consumer search** — cron registry, `scripts/`, and the vendored pickup lib for
   anything that READS the topic → IW-1. Found the rail split and the outbound-only
   bridge.
2. **Backlog measurement** — run the freshness canary, then attribute each firing
   offset against our own commit messages → IW-2. This is what exposed the self-echo.
3. **Destination measurement** — `fw review-queue` depth and the `owner: human`
   active count → IW-4.
4. **Duplication audit** — group `Pickup:` task names corpus-wide and read the
   created-timestamps of each group → IW-4. The 59/60-second spacing named the cause.

Spike 2's attribution step was not in the original plan. It was added after the
canary's output showed `from=root` on offsets our own commit log claims — treating
the canary's firing set as evidence rather than as an answer is what turned a
"3 unprocessed filings" reading into "1".

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

<!-- What's IN scope for this exploration? What's explicitly OUT? -->

**IN:** establishing whether a consumer exists and for which rail; measuring the
genuine inbound backlog separately from our own echo; measuring the queue any
auto-filer would feed; auditing whether the existing auto-filer is sound enough to
extend; producing a dependency-ordered recommendation.

**OUT — deliberately:**
- Building the consumer, or any topic→inbox bridge. This is an inception.
- Acking the topic. `--ack` would bump the marker past the one genuine peer filing
  (off=131) and reintroduce the exact G-063 miss the canary exists to prevent.
- Fixing the 60s duplicate re-file. Confirmed live defect, but one bug = one task.
- Fixing the attribution leak (T-3025's `metadata.from_project`). Recommended as
  step 1; not executed here.
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

**Recommendation:** **NO-GO on the filing's scope; GO on a materially smaller one.**

**Rationale:** The gap is real — `framework:pickup` has no reader, and the G-063
cost is on record (P-043 stranded 73 days, a bug re-discovered and re-fixed while
its report sat unread). But three measurements say a consumer is the wrong next
move:

1. **A consumer already exists**, wired to the file inbox and draining every 60s.
   The hub topic is simply not one of its sources.
2. **The genuine inbound backlog is ONE filing**, not eleven. Of 6 filings past the
   ack point, 5 are ours (4 confirmed against our own commit messages); only
   `off=131` is a peer's. The canary's firing set is **67% self-echo**, because
   `metadata.from_project` is not sender-settable (T-3025) so our own filings arrive
   unattributed and fall through the unknown-attribution arm.
3. **The existing auto-filer already duplicates** — 9 groups, 11 extra task files
   across 70 `Pickup:` tasks (16%), at exactly the 60-second drain interval — into a
   queue holding **150 tasks awaiting human review** and 143 active `owner: human`.

So this is a signal-to-noise problem, not a throughput problem, and the proposed
remedy would point a second automated filer — a filer with a live duplication
defect — at the one queue that is already saturated.

**Dependency-ordered scope:** (1) fix the attribution leak so the canary fires only
on genuine inbound; the backlog then reads 1 and needs no automation to clear.
(2) fix the 60s duplicate re-file, as its own task. (3) only then consider a
topic→inbox bridge, since it would reuse that same filer. (4) the drain itself is
sovereign — no consumer design changes 150.

**Note on the prior text:** this section arrived pre-filled with "GO" and the figure
"11 filings currently unprocessed", written before any exploration ran. The measured
figure is 1 genuine peer filing; the canary reports 3 and two of those are ours. The
gap it names is real; the number it cites, and the conclusion drawn from it, are not.

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

### 2026-09-19T22:35:28Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-009

### 2026-09-20T20:15:14Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
