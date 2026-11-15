# T-3189 Human AC#1: the repopulated cost axis produces a sane ranking

Reviewer: independent, rung-1-same-vendor-independent. Revision 3609243551e17a365de724d5ff49cc81f1bda4e2.

## What I ran

1. `cd /opt/termlink && .agentic-framework/bin/fw bvp --include-proposed --quadrant hv-lc` (the criterion's command, as written; it runs).
2. For the top 12 rows: read the latest `cost_estimate_proposed` rationale in each task file, and re-ran
   `_derive_blast_radius_refs` from `.agentic-framework/agents/termlink/bvp-estimator/estimator.py` on each body
   to see exactly which file references produced the cost. Then read each task's Context/ACs.
3. Ran the same derivation over every active task that reaches the derived tier (182 tasks) to tell a
   one-off from a systematic skew.

## Top rows: cost vs what the task involves

| # | Task | COST | blast_radius source | Refs the scorer counted | Reads as |
|---|---|---|---|---|---|
| 1 | T-2203 | 3.2 | derived, 3 refs | `.agentic-framework/bin/fw`, `CLAUDE.md`, `scripts/check-task-finalization-freshness.sh` | Cost plausible. Two of three refs are a command path and a citation, not files changed. |
| 2 | T-2606 | 3.2 | derived, 3 refs | `bus_client.rs`, `envelope.rs`, `dedupe.rs` | Under-costed. Body also names `channel.rs:767-784`, dropped as an ambiguous basename (2 in repo). 4 files = band 5 = cost 4.4 = hv-hc. The body itself says "architectural, not a mechanical fix". |
| 3 | T-2576 | 3.2 | derived | `claim_client.rs`, `parallel_worker.rs` (+ `bin/fw` now) | Plausible: a design decision over two files. |
| 4 | T-2577 | 3.2 | derived | `meta.rs`, `channel.rs`, `lib.rs` (+ `bin/fw` now) | Plausible. |
| 5 | T-2815 | 3.2 | derived, 2 refs | `audit.sh`, `bin/fw` | Plausible. Here `bin/fw:1772` is a real change site. |
| 6 | T-2940 | 1.6 | `components:` (measured, not derived) | one census .md | Not a T-3189 value. Cheapest row in the list, but the work is human triage of 57 tasks. |
| 7 | T-2886 | 3.2 | derived, 2 refs | mcp fragment json, `bin/fw` | Plausible. |
| 8 | **T-2911** | **2.0** | derived, 1 ref | `settings.json` | **Mis-costed.** See below. |
| 9 | **T-2978** | **2.0** | derived, 1 ref | `tl-claude.sh` | **Mis-costed (unknown, not cheap).** See below. |
| 10 | T-3010 | 3.2 | derived, 3 refs | `concerns.yaml`, a value-review report, `revisit-due-scan.sh` | All three are citations; the task changes none of them. It is a dormant reminder, open on purpose until 2026-12-18. |
| 11 | **T-3091** | **2.0** | derived, 1 ref | `.context/runs/T-3089-procasfit-x4.yaml` | **Mis-costed.** See below. |
| 12 | T-3188 | 3.2 | derived, 2 refs | divergence script, fixture file | Plausible (the real change was in estimator.py, not named by path). |

## The three rows that fail the Expected clause

**T-2911 "opencode runs ZERO of the 37 fw hooks".** The body is an untouched template: empty Context,
ACs are `[First criterion]` / `[Second criterion]`. The single ref `settings.json` comes from the H1
title line (`# T-2911: ... settings.json wiring is runtime-specific`), which the derivation reads as body.
Result: blast_radius 1, cost 2.0, the cheapest derived value possible. The work named by the title is
making 37 hooks run under a second runtime, which is broad. This is exactly "cheap because its body
happens to name few files while the work itself is broad". It should be UNMEASURED, or 7+ if costed.

**T-2978 "Re-arm push-wake rail: relaunch unwakeable agents via tl-claude.sh".** Same shape: empty body,
placeholder ACs, the one ref is the title. Cost 2.0. It should be UNMEASURED.

**T-3091 "A dispatched round cannot tell whether a sibling round is mid-flight".** The single ref is
`.context/runs/T-3089-procasfit-x4.yaml`, a run-state file cited in the incident timeline. The task is to
build a predicate a worker consults at the moment of action, a new dispatch mechanism; ACs are still
placeholders. Cost 2.0 from one cited session artifact. It should be UNMEASURED.

## One-off or systematic? Systematic, three causes, measured over the 182 derived-tier active tasks

1. **The title is read as evidence.** 6 tasks get their whole cost from the H1 title line alone:
   T-2823, T-2911, T-2978, T-3180, T-3212, T-3777. All six land at blast_radius 1, the cheapest value.
   A title that names one file is a bug-report subject line, not a reach estimate.
2. **Citations and command paths count as files touched.**
   - `.context/runs/` is missing from `_REF_EXCLUDE_PREFIXES`, though it is a session-artifact tree like
     the eight that are listed (T-3091).
   - `.agentic-framework/bin/fw` and `CLAUDE.md` are counted wherever an AC step says
     `cd /opt/termlink && .agentic-framework/bin/fw ...` or the prose cites a CLAUDE.md rule. For 4 tasks
     they are the only refs (T-1726, T-2566, T-3180, T-3798); for 16 more they move the ladder band
     (T-1426, T-1430, T-1457, T-1722, T-2014, T-2017, T-2198, T-2203, T-2567, T-2570, T-2577, T-2755,
     T-2815, T-2861, T-2886, T-2972). This one inflates rather than cheapens, but it means the count is
     not measuring reach. `docs/reports/*.md` citations behave the same way (T-3010, T-1124).
3. **A body that has not been written still gets a cost.** 16 derived-cost tasks still carry
   `[First criterion]` placeholder ACs. For those the derivation has nothing but a title or a pasted
   symptom to read, and T-3189's own rule is "return None rather than manufacture a value".

A side effect worth knowing: tier is 2 and effort is 8 for almost every row (the template alone is over
200 lines, so `lines=204` gives effort 8 for an empty task), so COST is in practice only the
blast-radius ladder: 2.0 / 3.2 / 4.4. The whole HV-LC list is "3 or fewer refs counted". That leaves no
other term to absorb an error in the ref count.

## Verdict: red

7 of the top 10 rows carry a believable cost, but the Expected clause is specific: no task appears cheap
because its body names few files while the work is broad. Rows 8, 9 and 11 are that case, and the cause
is in the scorer, not in three unlucky task bodies.
