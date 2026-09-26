---
id: T-3166
name: "Examine the arc-008 open backlog against the 8 live audit warnings - which
  are actionable from here"
description: >
  Inception: Examine the arc-008 open backlog against the 8 live audit warnings -
  which are actionable from here

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-09-26T08:00:26Z
last_update: 2026-09-26T08:01:05Z
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
  - ts: '2026-09-26T08:01:05Z'
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

# T-3166: Examine the arc-008 open backlog against the 8 live audit warnings - which are actionable from here

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

arc-008 ("Audit and doctor finding remediation") has never been examined at task level. Its
backlog was believed to be ~17 open tasks; nobody had checked. Meanwhile `fw audit` emits 8
warnings, five of which the audit's own 14-day trend analysis reports as recurring **10 times
each** — every run for ten runs. That is the alarm-fatigue failure (T-2818/T-2833) occurring in
the surface whose job is to notice things, and arc-008 is the arc that exists to end it.

For whom: the operator who runs `fw audit` and needs "Warn 8" to mean something. Why now: the
conversion campaign is exhausted (T-3163), so this backlog is the next frontier, and the arc has
a falsifiable finish line to aim at.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

<!-- T-2190 (T-2186 Slice 4): every IW-N question must be disposed before
     --status work-completed. Disposition gate (agents/task-create/update-task.sh
     check_disposition_gate) refuses on under-disposed inceptions.

     Per-question shape:

       - **IW-N: <question text>**
         confidence: 0-3      (your confidence in your current answer; 0=guess, 3=verified)
         disposition: answered | deferred | dissolved
         rationale: <one-line evidence — file:line, decision id, dialogue ref>

     Never bare yes/no — the gate refuses bare checkboxes. See 050-Inceptions.md
     §Disposition Gate. Bypass: --skip-disposition-gate "rationale" (direct) or
     FW_SKIP_DISPOSITION_GATE=1 (env-var, T-1890 producer/consumer parity).
-->

- **IW-1: Which of the 8 warnings `fw audit` emits today is claimed by no arc-008 task at all?**
  confidence: 3
  disposition: answered
  rationale: None is unclaimed — but #6 (Fabric 297/535 no edges) is claimed by T-3009, which is tagged `arc:arc-009`, so arc-008's member list under-represents its own headline mechanic. Table in docs/reports/T-3166-arc-008-backlog-examination.md.

- **IW-2: Of arc-008's open tasks, which are actionable from a code-only session, which are blocked on vendored code (G-062), and which are human-owned?**
  confidence: 3
  disposition: answered
  rationale: 72 members (not the ~17 I estimated), 40 completed. Actionable-from-here is the three Fabric warnings only; #1/#3/#4 are blocked on vendored audit.sh (filed offset 178); #2 and #5 are owner:human, and 27 of 46 active are human-owned.

- **IW-3: Does the arc's stated bar — "fixed OR acknowledged with a cited reason" — mean the three wrong-root warnings already satisfy it, given they are filed upstream at offset 178 with the reason cited in-audit?**
  confidence: 3
  disposition: answered
  rationale: No — and the reason is the finding. There is NO acknowledgement mechanism for audit warnings; the only silencing is a per-check global kill switch (FW_RETIRE_WHEN_ADVISORY=0) that removes the finding AND the record. So those three are acknowledged in substance yet still land in "Warn 8" indistinguishably from unexamined ones. The arc's success condition is unreachable with the current instrument (F5).

- **IW-4: Is the F-ORCH `retire_when` warning a sovereignty question (value drivers are ACD-gated) rather than agent work?**
  confidence: 3
  disposition: answered
  rationale: Yes. Value drivers live in policy/value-drivers.yaml and are ACD sovereignty-gated; T-3126 is already owner:human. Not agent work at any confidence level.

- **IW-5: Is arc-008's closure percentage a meaningful measure of its progress?**
  confidence: 3
  disposition: answered
  rationale: No — arc-008 is a permanent intake queue by design ("every FAIL and WARNING becomes its own governed task"), so every new finding grows its denominator. 30->72 members against 25->40 completions took the ratio from 0.833 to 0.556. The threshold check can only fire during a quiet spell and then pressures closure of an arc that is never finished. Emergent question, not in the original four.

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

**IN:** measuring arc-008's real membership; mapping each of the 8 live warnings to a claiming
task; classifying each open task actionable / vendored-blocked / human-owned; judging whether the
arc's stated bar is reachable.

**OUT:** fixing anything. No warning is remediated under this task — "one task = one deliverable",
and this one's deliverable is the examination. Also out: touching the 27 human-owned tasks, and
any judgement on F-ORCH (value drivers are §ACD sovereignty-gated).

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
- The 8 warnings are mapped to claiming tasks, and the actionable subset is bounded and named
- A specific structural gap is identified with a fix path that survives a re-vendor (G-062)

**NO-GO if:**
- The backlog turns out to be well-governed and the warnings already acknowledged (nothing to do)
- Every remaining path is human-owned or vendored-blocked, leaving no agent-actionable work

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

**Recommendation:** GO — narrowly, on two items, and explicitly NOT on "work the backlog"

Revised from the filing-time DEFER now that the mapping is measured.

**Rationale:**

The examination found the arc's success condition is **unreachable with the current instrument**,
which is a better finding than any individual warning. arc-008 requires every remaining warning to
be "fixed OR explicitly acknowledged with a cited reason", and there is no acknowledgement
mechanism for audit warnings at all — the only silencing available is a per-check global kill
switch (`FW_RETIRE_WHEN_ADVISORY=0`) that removes the finding *and* the record, which is the
opposite of acknowledgement. This repo already solved this exact problem twelve times on the other
side of the guard layer: every static check carries a git-tracked `.context/checks/*-allowlist`
whose entries are counted and reported but do not fire, each with a cited reason (T-2681, T-2483,
T-2680). The audit — oldest and most-read surface — never got one. Consequence today: three
warnings ARE acknowledged in substance (filed upstream at offset 178, reason printed in the
audit's own Mitigation line) and still land in the operator's "Warn 8" indistinguishably from
unexamined ones.

Second finding: arc-008 is **self-diluting**, so its closure percentage is not a meaningful
progress measure. Its job is to absorb every new audit finding as a task, so every finding grows
the denominator. Membership went 30 -> 72 while completions went 25 -> 40, taking the ratio from
0.833 to 0.556. T-3117 was filed on the 0.833 reading and its premise has since expired. The
threshold check can only fire during a quiet spell, and when it does it pressures closure of an
arc that is by design never finished.

GO is narrow and deliberately excludes the backlog itself: 27 of 46 active tasks are `owner:
human`, the two largest are human judgement, and the only mechanically-actionable warnings are the
three Fabric ones.

**Evidence:**

- `docs/reports/T-3166-arc-008-backlog-examination.md` — full measurement, warning-to-task table
- Membership replicated with the audit's own predicate (audit.sh:6440-6460): 72 members, 40 completed, ratio 0.5556
- Today's audit (`.context/audits/2026-09-26.yaml`) emits NO arc-closure warning for arc-008, confirming T-3117's premise expired
- Per-arc ratios measured: arc-011 .952, arc-parallel-substrate .933, arc-substrate-fitness .917, mcp-slimming 1.0 all still exceed threshold — so T-3118/T-3119/T-3120/T-3121 are NOT stale and must not be batch-closed with T-3117
- `check-stranded-finalized-tasks.sh`: 0 stranded, 72 partial-complete by design — the three work-completed-in-active tasks are T-193 by design, not the T-2833 defect
- Refuted hypothesis recorded: the audit's `^tags:` regex does NOT undercount via block-style YAML lists (0 tasks affected)
- `arc-011.yaml` declares `id: arc-010` — filename/id off-by-one that misled this examination

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

### 2026-09-26T08:01:05Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
