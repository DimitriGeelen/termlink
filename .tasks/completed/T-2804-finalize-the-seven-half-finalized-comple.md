---
id: T-2804
name: "Finalize the seven half-finalized completed tasks and generate their missing
  episodics"
description: >
  Seven tasks in `completed/` carry `horizon: now` (audit CTL-030). Four share one
  move-commit timestamp — the G-066 bulk-sweep signature — and also have empty
  `date_finished` and no episodic. The other three were moved individually by Watchtower's
  inception-decide path, which sets status and date but never clears horizon. Repair
  the
  frontmatter from the authoritative git move-commit, generate the four missing episodics,
  and write the summaries the git-mining path leaves blank.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [governance, task-system, g-066, episodic-memory]
components: []
related_tasks: [T-2290, T-2203, T-2160, T-2304, T-1665]
created: 2026-08-20
last_update: 2026-08-20T19:06:44Z
date_finished: 2026-08-20T19:06:44Z
bvp_scores_proposed:
  - ts: '2026-08-20T15:20:38Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 1
      F-ORCH: 0
    rationale: D1=2 (body:concern-ref); D2=0 (no-signal); D3=0 (no-signal); D4=0
      (no-signal); F-RECALL=1 (body:episodic-only); F-ORCH=0 (no-signal)
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

# T-2804: Finalize the seven half-finalized completed tasks

## Context

The full audit reports nine failures. Seven are one check:

```
CTL-030: T-2025 is in .tasks/completed/ but stored horizon='now'
         (expected: null/absent — render derives 'past' from _location, T-2160)
```

Same line for T-2028, T-2229, T-2257, T-2260, T-2261, T-2303. Separately, every handover
for weeks has flagged four tasks with no episodic summary: T-2025, T-2229, T-2303, T-2677.
Three tasks appear on both lists, which is what suggested one mechanism rather than two
coincidences.

Checking the git move-commits — the commit that first added each file to `completed/` —
gives a sharper answer: **there are two mechanisms, not one.**

```
T-2025   2026-07-02T17:41:50+02:00   T-2304: ... T-2303 finalized via sweep ...
T-2028   2026-07-02T17:41:50+02:00   T-2304: ... T-2303 finalized via sweep ...
T-2229   2026-07-02T17:41:50+02:00   T-2304: ... T-2303 finalized via sweep ...
T-2303   2026-07-02T17:41:50+02:00   T-2304: ... T-2303 finalized via sweep ...
T-2257   2026-06-25T08:35:07+02:00   T-2250: correct accidental NO-GO → DEFER ...
T-2260   2026-06-25T08:30:58+02:00   T-2260: inception decision NO-GO (via Watchtower)
T-2261   2026-06-25T08:30:56+02:00   T-2261: inception decision NO-GO (via Watchtower)
```

**Mechanism A — the bulk sweep.** Four tasks share one timestamp to the second, and the
commit message says "finalized via sweep". That is the G-066 signature quoted verbatim in
CLAUDE.md: *"Several shared identical move-commit timestamps, the signature of a bulk
`git mv` / migration that skipped finalization."* These four are exactly the four with empty
or null `date_finished`, and three of them are exactly the ones missing episodics. One sweep,
on 2026-07-02, moved them without running the finalize routine at all.

**Mechanism B — the Watchtower inception-decide path.** The other three were moved
individually, minutes apart, by a GO/NO-GO decision. They *are* properly finalized —
`date_finished` set, episodic present — and only `horizon` was left behind. A narrower defect
in one code path, not a bypass.

### Why the canary that exists for this stayed quiet

T-2290 built the task-finalization canary precisely for G-066. It fires on
`status != work-completed` in `completed/`, and treats empty `date_finished` as a *softer,
non-firing* class — informational unless `--strict`. All seven of these say
`status: work-completed`, so the firing class does not see them, and the four genuinely
bypassed ones sit in the class that prints without insisting.

So the canary saw this in July and, by design, said nothing an operator had to act on. That
is the same shape as this session's other findings — a warning nobody must act on is
indistinguishable from no warning — and it is worth recording rather than just repairing.

## Approach

**Repair from the authoritative source, do not invent.** `date_finished` is set from the
task's own `last_update` — the moment the framework wrote the completion — falling back to the
git move-commit when `last_update` is *later* than the commit, which means the file was edited
after completion and no longer records "finished". `last_update` is also what
`fw context generate-episodic` uses for its `completed:` field, so frontmatter and episodic
agree exactly. No value is guessed, and the move-commit remains the outer bound: a task cannot
have finished after the commit that recorded it.

(The first implementation used the move-commit directly and this section claimed that was the
generator's source. It is not — see Decisions. For T-2025 the two differ by 59 seconds, which
is how the mistake surfaced.)

**Clear `horizon` on all seven**, which is what CTL-030 asks for: `_location` derives "past"
(T-2160), so a stored horizon in `completed/` is stale state, not information.

**Generate the four missing episodics**, then write their summaries. The generator mines git
commit messages, and for these tasks it finds none — `Commits: 0` — so it emits a structurally
complete record with `summary:` blank. An episodic memory with no summary is a file, not a
memory. The summaries are written from each task's own Context and Decisions, and each is
marked as hand-written rather than mined so the provenance stays honest.

## Scope boundary

Repairs the seven instances and the four episodics. Does **not** change
`fw context generate-episodic`, the Watchtower decide path, or the T-2290 canary's firing
classes — all three are candidates but each is its own deliverable, and two of them are
vendored framework code (G-062). The findings are recorded here and filed.

## Acceptance Criteria

### Agent
- [x] All seven tasks have `horizon` cleared (null/absent) in `completed/`
- [x] The audit's CTL-030 failure count goes from 7 to 0
- [x] The four tasks with empty/null `date_finished` get it set from their git move-commit
- [x] No `date_finished` value is invented — each is traceable to a commit timestamp
- [x] Episodics exist for T-2025, T-2229, T-2303, T-2677
- [x] Each generated episodic has a non-empty `summary`
- [x] Hand-written summaries are marked as such, not passed off as git-mined
- [x] The three already-finalized tasks (T-2257/T-2260/T-2261) keep their existing
      `date_finished` and episodics untouched — only `horizon` changes
- [x] Task bodies are otherwise unmodified — this repairs frontmatter, not history
- [x] The two tasks outside the CTL-030 set with the same empty `date_finished`
      (T-1448, T-808) are repaired too — the CTL-030 boundary reflects which check
      happens to fire, not the shape of the defect
- [x] Every repaired episodic still parses as valid YAML
- [x] Every repaired task's frontmatter still parses as valid YAML (13/13)
- [x] `check-task-finalization-freshness.sh --strict` exits 0 — the framework's own
      strictest definition of finalized, not a hand-rolled grep

## Verification

# No completed task stores a horizon — this is what CTL-030 asserts.
out=$(grep -l '^horizon: now' .tasks/completed/*.md 2>/dev/null || true); test -z "$out"
# No completed/ task is unfinalized, by the framework's own strictest definition.
# NOT a hand-rolled grep: an earlier version of this line looked for an empty
# `date_finished:` line and passed while three tasks had the field ABSENT entirely.
# The canary counts absent as unfinalized, and it is right.
bash scripts/check-task-finalization-freshness.sh --strict
# Every repaired episodic parses and carries a non-empty summary.
#
# ONE LINE, deliberately. The P-011 gate executes each non-comment line as its
# own command, so the multi-line form this replaced was torn into fragments —
# and the first fragment, a bare `python3 -c "` with an unterminated quote,
# waited on stdin until the 900s timeout killed it (rc=124). A verification that
# HANGS is worse than one that fails: it looks like a slow build.
python3 -c "import yaml,sys; [sys.exit('%s has an empty summary' % t) for t in ['T-2025','T-2229','T-2303','T-2677'] if not (yaml.safe_load(open('.context/episodic/%s.yaml' % t)).get('summary') or '').strip()]; print('4 episodics parse with non-empty summaries')"

## RCA

**Symptom:** seven tasks in `completed/` store `horizon: now`; four of those also have empty
`date_finished` and no episodic summary.

**Root cause:** two distinct paths. (A) A bulk sweep on 2026-07-02 moved four tasks into
`completed/` without running `fw task update --status work-completed`, so none of the
finalization side-effects ran — no `date_finished`, no horizon clear, no episodic generation.
(B) Watchtower's inception-decide path finalizes correctly but does not clear `horizon`,
leaving three tasks with stale scheduling state.

**Why structurally allowed:** the T-2290 canary built for exactly this class gates on
`status != work-completed`. A sweep that sets the status but skips every other finalize
side-effect produces files that satisfy the firing predicate while being unfinalized in every
other respect. Empty `date_finished` — the one signal that would have caught it — is
deliberately non-firing. The canary has been reporting these as informational since July.

**Prevention:** not delivered here, deliberately. The repair is instance-level; making the
canary fire on empty `date_finished`, or making the sweep path call finalize, are separate
deliverables and one of them is vendored. Filed rather than bundled — see Decisions.

## Decisions

### 2026-08-20 — Derive `date_finished` from git, never from today

- **Chose:** The git move-commit timestamp.
- **Why:** It is the only defensible answer — the moment the task demonstrably entered
  `completed/`. Stamping today's date would make the record say these tasks finished in August
  when they finished in June and July, quietly corrupting duration metrics and the episodic
  timeline for the sake of a green audit.
- **Bonus:** it is the same source `generate-episodic` uses for `completed:`, so frontmatter
  and episodic agree by construction.

### 2026-08-20 — Write the summaries the generator cannot

- **Chose:** Hand-write the four blank summaries, marked as hand-written.
- **Why:** `generate-episodic` mines commit messages; these tasks have none referencing them,
  so it emits `summary:` empty. Leaving it blank satisfies "an episodic exists" while
  delivering nothing — the metric-scoring failure this session keeps finding. Marking them
  hand-written keeps the provenance honest: a future reader can tell which summaries are
  evidence and which are reconstruction.

### 2026-08-20 — Trust the canary's definition over my own grep

- **Context:** after repairing nine tasks, my verification grep reported zero unfinalized
  tasks and the T-2290 canary reported three: T-2505, T-2506, T-2507. The canary was right.
  Those three have **no `date_finished` line at all**, and my pattern only matched a line
  that was present-but-empty.
- **Chose:** Repair the three (inserting the field), and replace the grep in this task's
  Verification with `check-task-finalization-freshness.sh --strict`.
- **Why:** I wrote a narrower check than the condition it was supposed to prove, and it
  would have passed while the defect remained — the precise failure mode this session has
  found in five other places, this time mine. Deferring to the framework's own predicate
  removes the chance of my restating it wrongly a second time.

### 2026-08-20 — Repair the siblings CTL-030 does not name

- **Context:** with the seven fixed, two more completed tasks still had an empty
  `date_finished` — T-1448 (May) and T-808 (March). They were never in the CTL-030 set
  because their `horizon` happened to already be clean, so only half the defect was
  visible to the check.
- **Chose:** Repair them as well.
- **Why:** Fixing exactly the tasks a check happens to name, and leaving two identical
  siblings because a different field was already tidy, is scoring the metric rather than
  fixing the problem — the failure this session has found in five other places. Both
  invariants now read zero across `completed/`, which is a fact about the repo rather than
  about the checker.

### 2026-08-20 — `last_update`, not the git move-commit, after checking both

- **Context:** the first implementation derived `date_finished` from the git commit that
  added the file to `completed/`, and the task record claimed that was "the same source
  `generate-episodic` uses". That was wrong. The generator uses the task's own
  `last_update`, and for T-2025 the two differ: `last_update` 15:40:51Z, move-commit
  15:41:50Z.
- **Chose:** `last_update` when it is at or before the move-commit; the move-commit
  otherwise.
- **Why:** `last_update` is the moment the framework wrote the completion, 59 seconds
  before the commit recorded it — closer to the truth, and it makes frontmatter and
  episodic agree exactly, which was the point of the original (mistaken) claim. The
  move-commit remains the fallback and the outer bound: a task cannot have finished after
  the commit that recorded it. Guarding on `last_update <= move-commit` catches the case
  where a file was edited after completion, where `last_update` no longer means "finished".

### 2026-08-20 — Repair the instances, file the mechanisms

- **Chose:** Fix the seven; do not touch the canary, the sweep path, or the generator.
- **Why:** G-062 — two of the three are vendored and a local edit is erased on re-vendor. And
  bundling three prevention changes into a repair task is how a task stops being reviewable.
- **Noted:** the strongest of the three is making empty `date_finished` fire rather than
  inform. It is the signal that would have caught this in July.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-b7355244
- **Timestamp:** 2026-08-20T19:06:46Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-20T19:06:44Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
