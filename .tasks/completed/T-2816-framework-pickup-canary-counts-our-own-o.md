---
id: T-2816
renumbered_from: T-2691  # T-2823 cross-branch collision
name: "framework-pickup canary counts our own outbound filings as unprocessed inbound"
description: >
  The canary's stated contract is to surface filings from PEER projects, but it counts
  every filing on the topic including our own outbound posts. 10 of 14 current filings
  are
  ours. Posting a bug report to AEF makes our own canary fire at us, and the only
  way to
  quiet it (--ack) also acks genuine inbound filings that arrived in the meantime.
status: work-completed
workflow_type: build
horizon: null
owner: claude-code
created: 2026-08-20
last_update: 2026-08-20T18:54:34Z
date_finished: 2026-08-20T18:54:34Z
tags: [governance, canary, g-063, signal-quality]
bvp_scores_proposed:
  - ts: '2026-08-20T15:20:38Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=2 (body:concern-ref); D2=0 (no-signal); D3=0 (no-signal); D4=0
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-08-20T15:21:22Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2816: framework-pickup canary counts our own outbound filings

## Context

`scripts/check-framework-pickup-freshness.sh` exists to close G-063: the `framework:pickup`
topic receives filings from peer projects and termlink has no automatic consumer, so a
high-severity ring20 RCA once sat ~27h unnoticed (T-2229).

The script's own header states the contract:

> The `framework:pickup` hub topic receives bug-reports / feature-proposals / RCAs filed
> **by peer projects** (e.g. ring20).

The implementation does not honour that. It fires on *any* offset above the seen-marker
regardless of who wrote it. `from_project` is read (line 109) but used **only for display**
— never as a filter.

### Observed

At the time of filing, 14 filings sit on the topic. Ten carry
`metadata.from_project = 010-termlink` — they are *our own outbound* posts to AEF and to
Pen (050-email-archive). Only offsets 0, 2, 3 and 7 are genuinely inbound.

The perverse consequence: **filing a bug report upstream makes our own canary fire at us.**
Posting T-2815 to `framework:pickup` this session immediately added another "unprocessed
filing" that termlink is instructed to go and process.

### Why this is worse than noise

The documented workflow is "triage the surfaced filings, then run `--ack`". Because `--ack`
bumps the marker to the current max offset, an operator who acks to clear the echo of their
own posts **also silently acks any real inbound filing that arrived in between**. The canary
built to stop inbound filings being missed can therefore cause exactly that. That is the
G-063 failure mode reintroduced through the mitigation.

### Why the framework was blind (G-019)

The canary was written when the topic was purely inbound. Termlink then became a prolific
*poster* to the same topic (the T-2663/T-2713/T-2714/T-2784/T-2788 filings), and no check
noticed that the read side had never been taught the difference. Nothing asserts the
canary's own contract — that what it reports is what its header claims it reports.

## Approach

1. Self-attribution is an **explicit constant**, `FW_PICKUP_SELF_PROJECT` (default
   `010-termlink`), env-overridable. Deliberately **not** derived from `basename $PWD` —
   that is precisely the T-2815 defect filed upstream this session, and repeating it here
   would make the filter silently wrong inside a worktree.
2. Attribution order: `from_project` → `source_project` → `agent_id` → unknown.
3. A filing attributed to self is classified `own`: counted and reported, but **not firing**.
4. A filing with **unknown** attribution still fires — fail-safe. We cannot prove it is ours,
   and a false fire is cheap while a false silence is the G-063 failure.
5. The report is **loud about suppression**: it always prints how many own filings were
   skipped, so "quiet" can never be confused with "the filter ate something".
6. `--ack` semantics unchanged (still advances to max offset) — with own posts no longer
   firing, the ack-pressure that caused the swallow risk is gone.

## Acceptance Criteria

### Agent
- [x] Self-attributed filings no longer count toward the firing set
- [x] Unknown-attribution filings still fire (fail-safe direction proven by fixture)
- [x] Suppression count is always reported, never silent
- [x] JSON envelope carries `own_count` alongside `unprocessed`
- [x] Self-identity is not derived from a path basename (T-2815 class not repeated)
- [x] Regression fixtures cover: own-only topic reads healthy; inbound fires; unknown fires;
      mixed reports both counts
- [x] Fixtures are host-independent (PL-213 test seam, no live hub required)

## Verification

bash tests/pickup-canary-selffilter-fixtures.sh

## Decisions

**Constant, not derived, self-identity.** A derived slug (`basename $PROJECT_ROOT`) would be
wrong in a worktree and would silently disable the filter — the same class as T-2815. An
explicit constant fails in the safe direction: if the project is ever renamed, the filter
stops matching and our own posts merely become visible again (noisy, not dangerous).

**Unknown attribution fires.** The alternative (treat unattributed as own) would make the
canary blind to any peer that omits the metadata key — reintroducing G-063 for exactly the
least-careful callers. Firing on unknown is the fail-safe direction.

**Fixed here rather than filed upstream.** Unlike T-2815, this script is termlink's own
(`scripts/`), so it is ours to fix.

## Notes

Our own filings should consistently set `metadata.from_project`. The T-2815 filing posted
this session used `agent_id` instead, which is why it shows `from=?`. The attribution chain
added here accepts `agent_id` as a fallback so both conventions attribute correctly.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-54748e99
- **Timestamp:** 2026-08-20T18:54:36Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-20T18:54:34Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
