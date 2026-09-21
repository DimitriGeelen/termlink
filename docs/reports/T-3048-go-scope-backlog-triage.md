# T-3048 — Is the GO-scope-not-propagated backlog a bulk backfill or 86 decisions?

**Status:** exploration in progress. Created BEFORE the research per C-001 — the
thinking trail is the artifact; conversations are ephemeral, files are permanent.

## The question

`fw audit` has reported, for 9 consecutive runs, that **86 of 168** GO-recorded
completed inceptions have no propagated scope. Nobody has triaged it. One
question: is this a mechanical backfill, or 86 individual decisions?

## What the check actually asserts

Structural, no prose consulted. A completed task qualifies when ALL of:

1. `workflow_type: inception`
2. the `## Decision` block records a GO
3. `related_tasks:` is empty or absent
4. no task in `active/` or `completed/` names it on a `related_tasks:` line
5. no task declares `unlocks_inception_decision:` pointing at it

The report is explicit that these are *candidates for triage, not confirmed
abandoned decisions*. That distinction is the whole question.

## Prior observation that bounds the answer

The 86 findings span **T-002 to T-3012** — the entire project history, including
the second task ever created. So this is a convention that was never followed,
not a recent lapse. Any framing of it as "drift" would be wrong.

## Classification the triage is testing

| class | meaning | remedy |
|---|---|---|
| **linkable** | the GO's slices exist; `related_tasks:` was merely never filled | mechanical backfill |
| **inline** | the inception did the work itself; no slice was ever owed | the CHECK is over-reporting a legitimate shape |
| **unactioned** | GO recorded, nothing built, nothing linked | the only class representing real lost scope |

The three remedies are completely different, which is why the mix has to be
measured before anything is done in bulk.

## Method

Read-only. Sample a spread across eras (recent / mid / early / a consecutive
cluster), read each inception's own Decision and Recommendation, and classify.
No task is closed, no box ticked, no status changed. Research is not
authorization — the disposition is the human's.

## Findings

### Population measurement (all 86, mechanical)

For each finding, the `## Recommendation` / `## Decision` / `## Findings` prose was
scanned for references to any OTHER task ID:

| | count | share |
|---|---|---|
| names >= 1 task that **exists** | **57** | 66% |
| names tasks, none of which exist | 1 (T-956) | 1% |
| names no other task at all | 28 | 33% |
| file not found | 0 | — |

**The linkage data already exists in two thirds of the corpus.** It is sitting in
the prose, in the very section a human reads, and is simply absent from the
`related_tasks:` field the check parses. That is a records-keeping gap, not lost
scope.

### Sample of 12, spanning T-002 to T-3012

Six drawn across eras, six more drawn specifically from the 28 that name nothing:

- **T-2089** names its slices outright — "Build decomposes into 5 independently
  shippable slices (T-2090..T-2094)". All five exist and shipped.
- **T-2698** produced T-2699 (error-code emission check) and T-2700, both live and
  documented in CLAUDE.md; neither is linked back.
- **T-957** states it directly: *"No standalone build task needed — absorbed across
  subsystem work. T-940 / T-942 / T-941 all trace back to this learning."* The GO
  was discharged correctly and the check still counts it.
- **T-1122** produced T-1124, which is sitting in `active/` right now.
- **T-005** ("v0.1 spec produced") is a design inception whose deliverable WAS the
  spec. No slice was ever owed.
- From the no-reference 28: **T-690** (push-based event delivery) is `event.subscribe`
  and the long-poll rail, core to the product today; **T-209** is `fw dispatch`,
  live as `termlink_dispatch`; **T-2419** seeded the T-2468 purpose-review arc.

**Zero of the 12 were abandoned decisions.** In every case the work exists, shipped,
or was never owed a separate task.

### Answering the question

This is **not** 86 individual decisions. It is one convention that was never
followed, from T-002 onward, plus a check that counts a legitimate shape.

The remedy splits cleanly and neither half is 86 decisions:

1. **~57 mechanically backfillable** — extract the task IDs already present in the
   prose into `related_tasks:`. Reviewable in bulk, since the evidence is in the
   same file.
2. **~29 need a judgement, but ONE judgement, not 29** — should an inception whose
   deliverable was the decision or the spec itself (T-005, T-957) count as a finding
   at all? If not, the check should recognise that shape and the residue is small.

### What this does NOT establish (read the green narrowly, T-2680)

The sample is 12 of 86. It is strong enough to rule out "86 abandoned decisions" —
one counterexample would have shown up — but it does not prove every one of the 57
links correctly, nor that the remaining 29 are all benign. T-956 (names tasks that
do not exist) was not investigated and is the single anomaly found.

## Dialogue Log

- **2026-09-21** — Operator asked what to do next; this was proposed as item 2,
  explicitly scoped as "triage a sample to determine whether it's a bulk-backfill
  job or 86 individual decisions", with an undertaking to stop and report if it
  turned out to need per-item human judgement. Operator said "go".
- **2026-09-21** — Four gates fired on the way in and each was answered rather
  than bypassed: `fw inception start` refused a prose `--recommendation` (needs
  GO/NO-GO/DEFER), the task was created `captured` and needed a bare `fw work-on`,
  and the G-067 Open Questions gate refused all file edits until IW-1..IW-4 were
  filed. Recorded because the friction is evidence about the filing path itself,
  which is the subject of this inception.
