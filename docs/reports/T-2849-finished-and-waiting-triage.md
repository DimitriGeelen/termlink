# T-2849 — the finished-and-waiting queue, re-measured at the authority

**Date:** 2026-08-29 · **Task:** T-2849 · **Tree:** `/opt/termlink` (authority, branch `main`)
**Supersedes:** `docs/reports/T-2848-finished-and-waiting-triage.md`, which was measured in a
linked worktree and is wrong in its headline number and in every marker count.

## What this is

Tasks in `.tasks/active/` whose **every Agent AC is ticked and none open**, that still carry at
least one open `### Human` AC. Finished work waiting on a human.

**Nothing here ticks a Human AC.** Only the human may (CLAUDE.md, Human Task Completion Rule).
Where evidence shows an AC is already satisfied, that evidence is cited so the human can tick it
in one read instead of re-deriving it.

## Two corrections to the T-2848 report

**1. The count was wrong, and the reason is PL-368.** T-2848 measured 75 from a worktree whose
`.tasks/` replica was ~262 commits behind. The corpus is a registry with global invariants; a
stale replica of it is not a sample, it is a different register. Measured here: **82**.

**2. My own first re-measurement at the authority was also wrong, in a way worth recording.**
It reported 3 unmarked Human ACs correctly but inflated the marker counts, because it parsed
checkbox lines **inside HTML comments**. The task template ships an example block containing
`- [ ] [REVIEWER] Block message names both bypass mechanisms`, and that string is present in
**661 task files**. Any AC census that does not strip `<!-- -->` first counts the template's
own example as an open criterion in every task that still carries the template comment.

This is the same shape as the defects this repo's guard layer exists to catch: a measurement
reporting in language broader than what it actually measured. The corrected parser strips
comments before counting, and recognises a **fourth** marker — `[REVIEWER]` — which appears
in the template but on **zero** live ACs.

## The shape of it

| | count |
|---|---|
| **finished-and-waiting (total)** | **82** |
| — already `status: work-completed` (pure partial-complete) | 64 |
| — still `status: started-work` | 18 |
| **open Human ACs across them** | **91** |
| — `[REVIEW]` (genuine human judgement) | 74 |
| — `[RUBBER-STAMP]` (mechanical) | 14 |
| — **no marker at all** | **3** |

By task: 66 `[REVIEW]`-only · 11 `[RUBBER-STAMP]`-only · 2 mixed · 3 unmarked-only.

## The 3 unmarked — and why they matter more than their count

An open Human AC carrying neither marker cannot be routed by the convention, so it is invisible
to any triage that sorts by marker — including the T-2848 report, which reported zero of them.
All three are *"do this in the main checkout"* actions that were unperformable from a worktree.
**Two are now satisfied**, because the work landed by another route while the AC text stayed put.

| task | AC | measured today | verdict |
|---|---|---|---|
| **T-2819** | Run the vendored-framework catch-up in the main checkout | `check-framework-tracking-drift.sh` **exit 0**; `lib` 173/173, `policy` 19/19, `bin` 9/9, `agents` 159/159 tracked | **satisfied** (done by T-2807) |
| **T-2822** | Commit the four static-check allowlists | all four tracked under `.context/checks/`; all four checks **exit 0** | **satisfied — but do NOT follow its Steps** (below) |
| **T-2815** | Decide on the stray cross-project cron | still firing: **6 × `bin/fw: No such file` in 3h**, exactly the half-hourly rate; line 51 targets a `002-Claude-Partner-Network` worktree | **live — cross-project, needs the peer** |

**T-2822 needs a note, because its outcome is satisfied by a different route than its steps
describe.** The AC says to `git add .context/working/.<name>-allowlist`. T-2681 instead migrated
the allowlists to the git-tracked `.context/checks/`, and the checks resolve **tracked-first**
(`check-alloc-sink-clamps.sh:71-75`), falling back to the legacy path only for an un-migrated
checkout. Following step 4 today would commit a second, duplicate copy at the ignored legacy
path. Proven load-bearing rather than assumed: run with an empty allowlist → **exit 1**; run
with the tracked one → **exit 0**.

## Evidence-supported closes — 5, not 2

| task | Human AC | evidence measured today | verdict |
|---|---|---|---|
| **T-1696** | Cron entry installed in `/etc/cron.d` | `termlink-release-mirror-canary` present; findings log **empty** (healthy) | **satisfied** |
| **T-1723** | Cron installed so the meta-canary actually fires | **12** `check-canary-aliveness` job lines across **9** crontabs | **satisfied** |
| **T-2706** | Stuck-claims canary fires daily on 11 residue topics | canary now **exit 0**, `20 topics, 0 stuck` — the T-2709 predicate fix cleared it | **satisfied** |
| **T-2819** | (above) | exit 0, 360/360 tracked | **satisfied** |
| **T-2822** | (above) | 4/4 tracked, 4/4 checks exit 0 | **satisfied** |

## One operator action still gates four tasks

**T-1420 · T-2013 · T-2297 · part of T-1691.** `.141` is unreachable again this session
(`No route to host`), and the fleet remains version-skewed. Confirmed live, not carried
forward from the earlier report.

## The large families (30 tasks, 2 sittings)

`T-1482..T-1506` (21) and `T-1529..T-1537` (9) are one shipped feature family each, and their
Human ACs are the same sentence repeated — *"verify the verb reads naturally"*. Pure output
taste on the `agent presence / recent / timeline / on-thread / forward / edit / redact /
threads / pin-history / edits-of / relations` surface. **All 11 probed verbs are alive** (T-2848
checked this against P4's 52-tool deletion, and it is the one finding from that report that
needed no correction — it was measured against live binaries, not against the corpus).

`[REVIEW]` is right here: "reads naturally" is not grep-able, so the T-1811/T-1878 conversion
to `[REVIEWER]` does not apply.

## The honest caveat

This triage reads AC **text and status**. It does not re-run each task's original verification,
so a task whose Agent ACs were ticked in error is invisible to it. The value is routing — 82
opaque items reduced to a decision list with 5 closes evidenced and one blocker named — not a
claim that all 82 are correctly finished.
