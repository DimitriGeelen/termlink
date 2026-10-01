---
id: T-3200
name: "Consumer for the project inbox rail - how a durable mailbox reaches a session
  prompt"
description: >
  Inception: Consumer for the project inbox rail - how a durable mailbox reaches a
  session prompt

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-09-28T15:44:52Z
last_update: 2026-09-28T16:51:54Z
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
  - ts: '2026-09-28T15:45:49Z'
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

# T-3200: Consumer for the project inbox rail - how a durable mailbox reaches a session prompt

## Problem Statement

**DISSOLVED 2026-09-28 — the premise was false.** See Findings.

This task was opened to explore "how should a durable mailbox reach a session
prompt". That rail already exists, is built to the operator's own ten-step spec
(arc-011), and was proven live on 2026-09-22. Nine of ten slices are built.

## Findings — the real cause, and it is a bug not a design question

`scripts/journal-mirror.sh:115` enumerates topics with `--prefix "dm:"`. The file
contains **zero** references to `inbox:`. The mirror is the sole ingest point for
the journal the injector reads, so anything not on a `dm:*` topic is invisible to
the entire rail.

AEF migrated their consults to `inbox:cacc73ea32b121dd/010-termlink` on 2026-09-22
(their T-3433). Since that moment the built, proven, cron-scheduled rail has been
structurally blind to every message they sent — 49 of them.

**And the migration notice was itself sent to the new address.** Offset 7 is AEF
stating "Our consults now post to inbox:.../010-termlink" — delivered correctly to
the one place nothing was watching. A change-of-address card posted to the new
address.

One cause explains all three symptoms observed today: 49 unread consults, their
T-3434 ladder climbing to rung 4, and their H4/H5 never going green.

**Remediation is a build task, not an exploration:** extend the mirror's
enumeration to cover `inbox:*` alongside `dm:*`. Care is needed in WHICH inbox
topics — 22 exist and 95 of 232 records belong to AEF's ephemeral test identities,
so a blanket prefix swap would ingest fixture noise.

## Process note — why this task existed at all

I opened an inception over ground the operator had already designed and ruled on,
and presented IW-1 as an open sovereign question when SQ-4 had settled it and S9
had built to it. The arc file states this plainly; I read it only after the
operator asked "did we not design this in the sidecar?". Same failure as the
T-3130 miss recorded earlier in this session: starting work without reading the
record that already contains the answer. The cost here was one turn and no damage,
because the operator caught it.

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

- **IW-1: Should an inbound peer message interrupt a working session?**
  confidence: 3
  disposition: dissolved
  rationale: Already ruled by the operator as SQ-4 and BUILT. arc-011 S7 (T-3069)
    checks the prompt is free before injecting; S9 (T-3072) inverted its own slice
    title — "urgent shortens the WAIT, never the CHECK" — with fixture U2 guarding
    "URGENT INJECTED INTO A BUSY PROMPT". The question was never open; I failed to
    read the arc before asking it.
  Option C (wake on `inbox.queued`) requires yes. That is a claim about who
  controls a session's attention, not an engineering detail, and it is the
  operator's to make. Options A and B deliberately do not interrupt — they make
  the backlog visible and leave the timing to a human.

- **IW-2: Do we ack on READ, or on ACTION?**
  confidence: 3
  disposition: dissolved
  rationale: Already designed. arc-011 uses a three-rung receipt ladder — sent /
    delivered / read (S5, S6 T-3070) — so "read" is a distinct durable event from
    delivery, and notify-ledger.sh refuses to advance a rung without a receipt read
    back. The false dichotomy was mine.
  AEF's T-3434 ladder stops on ack. If we ack when a message is merely
  SURFACED, their retries stop while the work is still undone — converting a
  loud, correct escalation into silence, which is the failure this whole task
  exists to fix, inverted. Acking only on action keeps their pressure honest but
  leaves the ladder climbing while we are simply busy. Confidence 1 because the
  trade is clear but the right answer depends on IW-1.

- **IW-3: Does a consumer generalise beyond AEF?**
  confidence: 2
  disposition: answered
  rationale: Yes by construction — the rail is topic-prefix driven, not
    peer-specific. journal-mirror.sh enumerates a PREFIX, so generalisation is a
    matter of which prefixes it watches. That reframes the whole task: see
    Findings. Residual care is which inbox: topics to include, since 95 of 232
    records are AEF's ephemeral test identities.
  Measured: 22 `inbox:*` topics exist across 5 real peer roots
  (832-Workflow-designer 54, 010-termlink 49, 1409-sprind 27, AEF 6,
  framework-agent 1). A consumer built around AEF's sidecar.consult convention
  is one that breaks on the second correspondent. Unknown: whether the other
  four peers post with comparable metadata, which is cheap to check and has not
  been checked.

- **IW-4: What is the real inbound rate?**
  confidence: 3
  disposition: dissolved
  rationale: Only mattered as input to IW-1 (interrupt cost). IW-1 is dissolved —
    the injector never interrupts a busy prompt regardless of rate — so the
    measurement no longer gates anything. Rate remains mildly interesting for
    queue sizing; not worth a spike.
  Interrupt cost is a function of volume and NOBODY HAS MEASURED IT. 232 records
  accumulated over an unknown window. Without a rate, option C's cost is a guess
  and IW-1 cannot be answered honestly. This is the cheapest thing exploration
  could settle and it gates the expensive decision.

## Exploration Plan

<!-- How will we validate assumptions? Spikes, prototypes, research? Time-box each. -->

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

**Recommendation:** NO-GO — dissolve. The question was already answered and built.

**Rationale:**

REVISED 2026-09-28, before any spike ran. The opening recommendation (GO to explore
four design options) rested on a false premise: that no consumer existed. One does.
arc-011 is the operator's own ten-step sidecar spec and NINE of its ten slices are
built, with S10 proven live on 2026-09-22 against a receiver's own transcript. IW-1
— "may an inbound message interrupt a working session?" — was ruled by the operator
as SQ-4 and built into S7/S9: the injector checks the prompt is free, and urgent
shortens the WAIT, never the CHECK.

What actually broke is a one-line-class defect, not a design gap.
`scripts/journal-mirror.sh:115` enumerates topics with `--prefix "dm:"` and contains
zero references to `inbox:`. The mirror is the sole ingest point for the journal the
injector reads. AEF moved their consults to `inbox:cacc73ea32b121dd/010-termlink` on
2026-09-22, so the rail has been blind to them ever since — including the message
that announced the move, which was itself sent to the new address.

Dissolve this inception and carry the remediation as a build task against the mirror.
Exploring four alternatives to a rail that already works would have been the most
expensive possible response to a topic-prefix bug.

**Operator command to record the dissolution** (agents cannot run `inception decide`
— Tier-0, and T-2958 confirmed the verb refuses an agent outright):

    cd /opt/termlink && .agentic-framework/bin/fw inception decide T-3200 no-go \
      --rationale "Dissolved: the consumer exists (arc-011, 9/10 slices built, S10 proven live). IW-1 was already SQ-4. Real cause is journal-mirror.sh watching dm: only; carried as a build task."

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

### 2026-09-28T15:45:49Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
