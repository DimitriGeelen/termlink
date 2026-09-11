---
id: T-2956
name: "Five inbound 050-email-archive filings never surfaced while the pickup canary
  was blind"
description: >
  Offsets 100-102 (P-EMAIL-ARCHIVE-001/002/003) and 104-105 (Pen defects re T-2065)
  are genuine inbound peer filings on framework:pickup that were invisible behind
  the T-2954 truncation. They need reading and triaging; this is the measured cost
  of the canary blindness.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-008]
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
# demo_target: true               # T-2286: optional — marks task as reserved for an orchestrated demo
#                                 # worker (e.g. arc-010 HM-A dispatches via mcp__fw__work_on). When set,
#                                 # `fw work-on T-XXX` refuses unless --i-am-demo-orchestrator (CLI) or
#                                 # FW_I_AM_DEMO_ORCHESTRATOR=1 (env) is passed. Prevents the parent
#                                 # session from consuming the captured→started-work transition the demo
#                                 # worker expects to drive. Origin OBS-057.
created: 2026-09-11T19:48:44Z
last_update: 2026-09-11T19:56:05Z
date_finished: 2026-09-11T19:56:05Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── BVP scoring fields (T-1918, arc-006). See docs/reports/T-1915-bvp-inception.md for semantics. ──
# bvp_scores:                     # confirmed per-driver scores 0-5, set by `fw bvp confirm` (T-1924).
#                                 # Sovereignty boundary — only set after human or agent confirmation.
#                                 # Shape: {D1: <int 0-5>, D2: <int 0-5>, D3: <int 0-5>, D4: <int 0-5>, [<free-driver-id>: <int>]...}
# bvp_scores_proposed:            # estimator-proposed scores (T-1922 worker). Persists when ≥2 delta
#                                 # from bvp_scores: on any driver (M3 v2-delta). Shape: list of timestamped entries.
# cost_estimate:                  # F8 composite: 0.6×blast_radius + 0.3×tier + 0.1×effort.
#                                 # Q2 fallback: T-shirt S/M/L/XL mapped to 2/4/6/8 when blast_radius is not yet computable.
bvp_scores_proposed:
  - ts: '2026-09-11T19:50:48Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 3
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2956: Five inbound 050-email-archive filings never surfaced while the pickup canary was blind

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] All five filings (offsets 100, 101, 102, 104, 105) are read in full and
      each one's substance is summarised in this task — not counted, not
      characterised by its subject line, read.
- [x] Each filing has an explicit, stated disposition with its reason: a local
      task is filed, a reply is owed, or no action is warranted. "No action" is
      a permitted outcome and must carry a reason; silence is not.
- [x] The dispositions are recorded durably in the arc-008 census, so a future
      reader can audit what was decided about a peer's report without replaying
      the rail.
- [x] Nothing is acked or drained as a side effect of triage. The canary's ack
      marker is a separate, deliberate act — T-2801's "detect and never drain"
      rule applies here for the same reason: converting a visible backlog into
      an invisible one is the failure being remediated, not the remedy.

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.

     ── Prefix routing (T-1811, T-1878): default to [REVIEWER] if Expected is grep-able ──
     If your Expected clause is grep-able / file-exists / structural (a deterministic
     shell check), prefer [REVIEWER] — that AC should be an Agent AC with the reviewer
     command in `## Verification

```bash
# All five filings are read and dispositioned in this task file.
grep -q 'offset 100 —' .tasks/active/T-2956-five-inbound-050-email-archive-filings-n.md
grep -q 'offset 105 —' .tasks/active/T-2956-five-inbound-050-email-archive-filings-n.md

# Every disposition is explicit — five DISPOSITION lines, none implied.
test "$(grep -c 'DISPOSITION' .tasks/active/T-2956-five-inbound-050-email-archive-filings-n.md)" -ge 5

# The outcome is recorded durably in the census, not only in the task.
grep -q 'F-J triage outcome' .context/audits/arc-008-cycle1-census.md

# The two live defects have local tasks.
test -f .tasks/active/T-2957-seven-pretooluse-gates-are-wired-writeed.md
test -f .tasks/active/T-2958-fw-task-review-hardcodes-go-in-the-decis.md

# Nothing was acked or drained as a side effect: the canary marker is unchanged
# and still fires on the inbound set (T-2801 detect-never-drain).
bash scripts/check-framework-pickup-freshness.sh --json > /tmp/.t2956.json 2>/dev/null || true
python3 -c "import json; d=json.load(open('/tmp/.t2956.json')); assert d['seen_offset']==99, 'ack marker moved: %r' % d['seen_offset']; assert len(d['unprocessed'])>0, 'inbound set was drained'"
```

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

### 2026-09-11 — the triage inverted this task's own premise
- **What changed:** The task was filed as "five peer filings need reading" — a
  courtesy backlog. Reading them showed two are LIVE defects in this repo (one on
  the sovereignty boundary, T-2958) and a third, offset 102, is the ROOT CAUSE of
  a cluster this project spent cycles 2 and 3 mis-diagnosing. The inbound queue
  was not a backlog of other people's problems; it contained the answer to ours.
- **Plan impact:** "Triage the peer filings" was scheduled as the lowest-value
  item remaining and is in fact where the highest-value finding of the arc was.
  The ordering heuristic — own findings before inbound reports — was wrong, and
  it was wrong for a structural reason: our own findings are visible by default
  and inbound ones only after a guard surfaces them, so the unread queue is
  systematically under-weighted precisely when the guard reading it is broken.
- **Triggered:** T-2957 (gate matcher coverage), T-2958 (hardcoded `go` on the
  sovereignty boundary); supersedes P-075's diagnosis and resolves the root-cause
  question in T-2951; a correction to P-075 and a reply to the peer on offset 104
  are both owed and deliberately deferred until F-I establishes whether our
  filings reach upstream at all.

## Recommendation

<!-- T-2945: same shape as inception.md's block — the gate that reads it
     (audit_inception_recommendation, lib/task-audit.sh:117) is shared, so the
     shape is copied rather than reinvented.

     REQUIRED once this task reaches partial-complete: Agent ACs done, at least
     one `### Human` AC still unticked. `lib/review.sh:205-211` (T-2421) BLOCKS
     `fw task review` emission for build/refactor/test/decommission tasks in that
     state with no substantive block here — the operator would otherwise open
     /review/<id> to a blank Recommendation card and be asked to approve a form.

     Not required while every Human AC is ticked or the task has none: the gate
     only fires on the partial-complete transition. It is here from the start so
     you write it while you still have the evidence, not when the gate refuses.

     Format (the parser wants the `**Recommendation:**` line at the start of a
     line; a leading `-` or `*` bullet is also accepted):
     **Recommendation:** GO / NO-GO / DEFER
     **Rationale:** Why (cite evidence — what shipped, what was proven, what remains)
     **Evidence:**
     - Finding 1
     - Finding 2

     DEFER is for evidence gaps, not confidence gaps (CLAUDE.md §Presenting Work
     for Human Review). If the artefact is complete and you still don't want to
     commit, that is a calibration failure — recommend GO or NO-GO.
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

### 2026-09-11T19:48:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2956-five-inbound-050-email-archive-filings-n.md
- **Context:** Initial task creation

### 2026-09-11T19:50:17Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-008

### 2026-09-11T19:50:48Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Triage — all five read 2026-09-11, each with a stated disposition

### offset 100 — P-EMAIL-ARCHIVE-001 · `check-human-ac-tick` is Write|Edit only
`.claude/settings.json` wires `check-active-task` to `Write|Edit|Bash`, but
`check-human-ac-tick` — and six siblings (`check-active-completed-dup`,
`check-arc-id`, `check-heredoc-cmd-sub`, `check-inception-decisions`,
`check-inception-schema`, `check-onboarding-gate`) — to `Write|Edit` only.
**VERIFIED LIVE in this repo.** A task file rewritten from a Bash heredoc ticks
Human ACs with no block and no Tier-2 log entry.

Not abstract: every task-file edit in this session was a python heredoc through
Bash, so the gate protecting the Human-AC sovereignty boundary was never in the
path. What kept those ACs untouched was my own column-0 matching, i.e. agent
discipline — which is precisely what P-002 exists to replace.

**DISPOSITION: local task T-2957.**

### offset 101 — P-EMAIL-ARCHIVE-002 · orchestrator-mcp-scan guaranteed FAIL
Reports an ABSENT framework-mcp manifest as `FRAMEWORK-REGRESSION: 6 tool(s) lost
their gate`, resolving the manifest to a PROJECT_ROOT path that cannot exist in a
vendored consumer. `agents/audit/orchestrator-mcp-scan.sh` IS present here, so we
are in the affected class.

**DISPOSITION: no task this cycle, and the reason is stated rather than implied.**
Presence is confirmed; the failure is not yet reproduced here, and filing a task
on an unreproduced claim is how P-075 came to carry a wrong diagnosis (below).
Carried as an explicit next-cycle item in the census.

### offset 102 — P-EMAIL-ARCHIVE-003 · `fw pickup send` without `--remote` delivers NOTHING
It writes the envelope to the SENDER's own inbox (`lib/pickup.sh:617`
`filepath="$PICKUP_INBOX/$filename"`), prints "Created", exits 0 — and
`fw pickup process` then opens our own outbound report as inbound work.

**This is the root cause of F-B, F-I and T-2951, and it corrects P-075.**
Across cycles 2 and 3 this project independently observed the symptoms — own
filings minted as local tasks, each appearing twice, outbound records stranded in
`auto-deferred/` — and filed P-075 blaming the MINTING path for lacking a
self-filter. That diagnosis is wrong. The envelopes were never sent anywhere; they
were written to our own inbox and re-ingested. P-075's proposed remedy would not
have fixed it.

A peer had diagnosed it correctly on **2026-09-07**, four days earlier, and filed
it here. It was invisible behind the T-2954 truncation. This is the T-2801 P-043
precedent recurring exactly: *the same bug solved twice while its sibling stayed
open* — except this time the second solver (us) got it wrong and filed the wrong
fix upstream.

**DISPOSITION: supersedes P-075's diagnosis; resolves the root-cause question in
T-2951. A correction to P-075 is owed upstream — deliberately NOT sent yet, see
F-I: our filings may not be reaching upstream at all, and this filing explains why.**

### offset 104 — Pen / T-2065 · `fw fabric drift` SIGPIPE false-unregistered
`echo "$registered" | grep -qx "$rel_path"` under inherited `set -euo pipefail`:
grep short-circuits on match, echo takes SIGPIPE (141), pipefail propagates 141,
`!` inverts it, and a registered file is reported unregistered. Fires precisely
when the answer is "registered".

**ALREADY FIXED in our copy** — `agents/fabric/lib/drift.sh:30` carries the
remediation with a comment attributing it to T-2518. This is L-387, which this
project documents at length, corroborated independently by a peer.

**DISPOSITION: no local task. A reply is owed telling them it is fixed and where,
so they do not re-derive it** — the courtesy this task exists because we did not
receive.

### offset 105 — Pen / T-2065 follow-on · `fw task review` hardcodes `go`
The copy-pasteable command interpolates the RATIONALE from the task's own
`## Recommendation` but hardcodes the VERB as `go`. On a task recommending NO-GO
in three places it printed a command recording GO. It inverted a live human
decision on their repo.

**VERIFIED LIVE here:** `.agentic-framework/lib/review.sh:380` emits
`inception decide $task_id go \`.

This sits on the sovereignty boundary. CLAUDE.md requires decisions be handed to
the human as copy-pasteable commands (T-609) and marks mechanical ones
`[RUBBER-STAMP]`; a rubber-stamped command that records the opposite of the
recommendation converts the sovereignty gate into a mechanism for silently
reversing the human's decision.

**DISPOSITION: local task T-2958.**

### What this triage cost, and what it bought
Five filings, four days unread. Two are live defects in this repo, one of them on
the sovereignty boundary. One is the root cause of a defect this project spent two
cycles mis-diagnosing and filed a wrong fix for. One was already fixed here and
the peer is still carrying it.

Not one of them was reachable while the canary reported healthy.


## Reviewer Verdict (v1.5)

- **Scan ID:** R-95a016e5
- **Timestamp:** 2026-09-11T19:56:06Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-11T19:56:05Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
