---
id: T-2954
name: "framework-pickup canary is blind past offset 99 and reports healthy"
description: >
  The pickup canary drains channel subscribe with no --limit, so it sees only the
  default first 100 envelopes (offsets 0-99) and reports ok:true while 22 newer filings
  sit unsurfaced. G-063 reproduced inside the guard built to prevent it.

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
created: 2026-09-11T19:39:48Z
last_update: 2026-09-11T19:44:58Z
date_finished: 2026-09-11T19:44:58Z
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
  - ts: '2026-09-11T19:41:37Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-11T19:41:41Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=216,acs=7)
    rubric_sha: e4a00f38e801
---

# T-2954: framework-pickup canary is blind past offset 99 and reports healthy

## Context

Measured 2026-09-11 while closing T-2951.

`scripts/check-framework-pickup-freshness.sh:102` drains the topic with:

    termlink channel subscribe "$TOPIC" --since "$SINCE_MS" --json

No `--limit` is passed, so the verb applies its documented default of **100**.
Replicating the canary's own call returns exactly 100 envelopes, offsets 0-99,
while the live topic is at offset 121.

The canary therefore reports:

    framework-pickup canary: healthy - all filings surfaced (acked up to offset 99)
    {"ok": true, "seen_offset": 99, "max_offset": 99, "unprocessed": [], "own_count": 0}

`max_offset: 99` is not the topic's max offset. It is the last envelope of the
first page. `own_count: 0` shows nothing was suppressed by the T-2816 self-filter
-- the 22 newer filings were never seen at all.

This does not self-correct. The window is pinned to the first 100 envelopes for
the life of the topic, so every future filing is invisible and the canary reports
healthy forever.

That is G-063 -- the miss this canary was built to prevent -- reproduced inside
the canary. It is strictly worse than the original: G-063 was "no consumer
exists"; this is "a consumer exists, runs daily, and affirms health while blind".
A guard reporting green is why nobody looks.

Concretely stranded behind the window right now: offset 120 (our own P-076, see
T-2955) and the inbound `opencode` learning envelope P-078, whose local copy sits
in `.context/pickup/auto-deferred/` with no breadcrumb (T-2951).

The script is PROJECT-OWNED (`scripts/`, not `.agentic-framework/`), so G-062
does not apply and the fix belongs here.

## Acceptance Criteria

### Agent

- [x] The canary reads the whole window it claims to read: the subscribe call is
      bounded explicitly rather than inheriting the verb's default page size, and
      a topic larger than one page no longer truncates silently.
- [x] `max_offset` reported by `--json` equals the true maximum offset on the
      topic, verified against an independent read of the hub.
- [x] The previously-hidden filings become visible: the canary's unprocessed set
      is computed over every envelope newer than the acked offset, not over the
      first page only.
- [x] Truncation can never again be silent: if the drain returns exactly the
      page limit, the canary treats the window as incomplete and says so rather
      than reporting a bounded `max_offset` as final.
- [x] The fixture suite still passes, and gains a case pinning the regression.
      It is a STRUCTURAL pin, not a behavioural one, and is labelled as such in
      the fixture: the `FW_PICKUP_TEST_NDJSON` seam replaces the drain wholesale,
      so nothing routed through it can exercise the subscribe call. Feeding it a
      >100-line NDJSON would assert the PARSER handles a large input -- which was
      never the defect, and would be a green covering something other than what
      it appears to cover (the T-2680 / T-2747 failure this task is an instance
      of). The pin is proven load-bearing against a mutant with the fix reverted.

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.

     ── Prefix routing (T-1811, T-1878): default to [REVIEWER] if Expected is grep-able ──
     If your Expected clause is grep-able / file-exists / structural (a deterministic
     shell check), prefer [REVIEWER] — that AC should be an Agent AC with the reviewer
     command in `## Verification

```bash
# The canary's own drain must reach the true head of the topic, not the first page.
bash scripts/check-framework-pickup-freshness.sh --json > /tmp/.t2954-canary.json 2>/dev/null || true
python3 -c "import json; d=json.load(open('/tmp/.t2954-canary.json')); assert d['max_offset'] > 99, 'max_offset still pinned to the first page: %r' % d['max_offset']"

# max_offset must agree with an INDEPENDENT read of the hub head.
timeout 60 termlink channel subscribe framework:pickup --cursor 100 --limit 1000 --json > /tmp/.t2954-hub.txt 2>/dev/null
python3 -c "import json; offs=[json.loads(l)['offset'] for l in open('/tmp/.t2954-hub.txt') if l.strip()]; d=json.load(open('/tmp/.t2954-canary.json')); assert offs, 'independent hub read returned nothing'; assert d['max_offset']==max(offs), 'canary %r != hub head %r' % (d['max_offset'], max(offs))"

# The bound is explicit and the drain advances a cursor -- the default must not be inherited.
grep -q -- '--limit "$PAGE_LIMIT"' scripts/check-framework-pickup-freshness.sh
grep -q -- '--cursor "$cursor"' scripts/check-framework-pickup-freshness.sh

# Fixtures green, including the structural regression pin.
bash tests/pickup-canary-selffilter-fixtures.sh > /tmp/.t2954-fix.txt 2>&1
grep -q '11 passed, 0 failed' /tmp/.t2954-fix.txt
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

Found while closing T-2951, by asking why a genuine inbound filing (P-078, from
`opencode`) was sitting unprocessed in `auto-deferred/` while the canary that
exists to surface exactly that reported healthy.

The escalation is Level C/D, not A. The symptom is one missing flag. The
structural fault is that the canary derived its own completeness claim from a
paged read without ever checking whether the page was full -- so "I read
everything" and "I read the first hundred" were the same code path and produced
the same green output.

G-019 asks why the framework was blind. It was blind because the guard's health
signal was computed from its own truncated input, with no independent check that
the input was complete. The meta-canary (T-1723) watches whether canaries still
*run*; nothing watched whether a canary that runs can still *see*. That is a
general shape and this project has hit it before (T-2680: a canary that reported
`live_off_charter: 0` across "214 tools" while its detector knew only six
families; T-2747: a 24-of-260 parity suite whose green read as full coverage).

The durable lesson is the one those two already state and this instance
confirms a third time: a guard must report the SCOPE of what it examined
alongside its verdict, and must fail closed when that scope is provably partial.
A bounded read that cannot distinguish "no more data" from "no more page" is not
a measurement.

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

### 2026-09-11T19:39:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2954-framework-pickup-canary-is-blind-past-of.md
- **Context:** Initial task creation

### 2026-09-11T19:40:54Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-008

### 2026-09-11T19:41:49Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-d448152d
- **Timestamp:** 2026-09-11T19:45:00Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-11T19:44:58Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
