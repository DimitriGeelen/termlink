---
id: T-2812
name: "Three local fixes to vendored code will be erased by the next re-vendor, and one is being proposed now"
description: >
  T-2304 (update-task.sh sys.path), T-2469 (budget-gate hotfix) and T-2813 (pickup fail-open) are local modifications to vendored framework code with no recorded upstream landing. This repo has no .vendor-divergence.yaml. History shows a wholesale vendor event roughly every two months, and T-2705 on worktree-charter-review proposes `fw update` now.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [governance, g-062, vendor-divergence, directive-2]
components: []
related_tasks: [T-2813, T-2304, T-2469, T-2705, T-2807, T-2053]
created: 2026-08-20
last_update: 2026-08-20T18:11:32Z
date_finished: 2026-08-20T18:11:32Z
---

# T-2812: Local fixes to vendored code have no survival mechanism

## Context

Found while closing **T-2813** (top HV/LC, BVP 99). Its work is genuinely done — 12/12 fixture
assertions pass against the real `lib/pickup.sh`, exercising every refusal path. But
`lib/pickup.sh` is **vendored**, and the task records no upstream filing. So the fix is
one `fw update` away from being deleted, and the task would have closed as "complete".

That is not hypothetical timing. **T-2705 on `worktree-charter-review-2026-0814` proposes a
re-vendor via `fw update` as its remedy, right now.**

### The history says erasure is routine

`git log -- .agentic-framework/` carries **124 commits**, of which **8 are wholesale vendor
events** — `fw upgrade`, `bootstrap-replace`, `re-vendor`, `fw update v1.6.7 → v1.6.295 (444
files)`. Roughly one every two months. Every local change made before one of those is either
carried by upstream or silently gone.

And this repo has **no `.vendor-divergence.yaml`**. Peer project 832-Workflow-designer cites
theirs in filings ("declared in `.vendor-divergence.yaml` as upstream:fix, GENERIC"); we have
no equivalent, so there is no way to answer "what have we changed locally, and did it land?"
short of reading commit messages.

### The at-risk set, narrowed twice

First pass used a keyword heuristic for "vendor event" and picked `T-2469` as the baseline —
wrong, because its message says re-vendor *recommended*, not performed. That understated the
window. Re-measured from the last unambiguous bulk re-vendor (`8c1cca561`, 2026-06-08, 444
files), **12 commits** touch vendored code since. Triaging them:

| commits | class | at risk? |
|---|---|---|
| T-2806, T-2807, T-2811 | **recovery** — add files that already exist upstream | no; a re-vendor restores them anyway |
| T-2061 | records "(landed upstream)" | no |
| T-2418 | port of an upstream fix ("per AEF 8c07bb091 relay") | no |
| T-2365 | itself a targeted re-vendor | no |
| T-2204, T-2194..2201, T-2155 | artifacts / chmod / episodic capture | negligible |
| **T-2304** | real fix: `update-task.sh` sys.path so `fw inception decide` finalizes | **yes** |
| **T-2469** | real fix: budget-gate hotfix, PL-265 | **yes** |
| **T-2813** | real fix: two fail-open paths in the pickup ingest rail | **yes** |

Three. Small enough to act on properly, which is why it was worth narrowing rather than
reporting "95 local modifications" — the first number this scan produced, and true but useless.

**T-2304 is the sharp one.** CLAUDE.md already records that a stranded pickup envelope (P-043)
named two framework bugs, and that one of them "had been independently re-discovered and
re-fixed as T-2304" 73 days later. That fix cost the project twice already. Losing it to a
re-vendor would cost it a third time.

## Approach

File all three upstream in one report, with enough detail that they can be applied without
re-deriving them — these are small, specific defects with known reproductions.

Create `.vendor-divergence.yaml` so the question "what have we changed, and did it land?" has
an answer that is not "read 124 commit messages". Register the three, and the classes that are
deliberately NOT divergence (recovery commits, ports of upstream fixes).

**Do not re-vendor**, and do not pre-empt T-2705. Their remedy may well be right; the point of
this task is that it should not silently cost three fixes when it runs.

## Scope boundary

Files three defects and creates the register. Does **not** re-vendor. Does **not** modify any
vendored file. Does **not** attempt to establish upstream status for the ~83 pre-2026-06-08
commits — they are either already carried or already lost, and re-deriving that is archaeology
with no forward value.

## Acceptance Criteria

### Agent
- [x] `.vendor-divergence.yaml` exists, registering T-2304, T-2469 and T-2813 with file,
      symptom, and upstream status (`filed-upstream`, offset 27)
- [x] It also records the non-divergence classes — recovery, upstream ports, artifacts, and a
      **mode-only** class the detector surfaced that I had described in prose but never recorded
- [x] All three defects filed upstream on `framework:pickup` **offset 27**, each with the file,
      the failure, and enough detail to apply without re-deriving
- [x] `scripts/check-vendor-divergence.sh` surfaces unregistered local modifications since the
      declared baseline. It found one on its first run — against a register its own author had
      just written
- [x] Fixtures pin the check — **10/10**, weighted toward the FIRING cases, plus three
      fail-closed paths (missing / baseline-less / unparseable register all exit 2)
- [x] CLAUDE.md records the register, the erasure cadence, why the baseline is declared rather
      than detected, and to run the check before proposing or accepting a re-vendor

## Verification

# The register exists and parses.
python3 -c "import yaml,sys; d=yaml.safe_load(open('.vendor-divergence.yaml')); sys.exit(0 if isinstance(d,dict) and d.get('divergences') else 'register missing or empty')"
# All three at-risk fixes are registered.
python3 -c "import yaml,sys; d=yaml.safe_load(open('.vendor-divergence.yaml')); ids={x.get('task') for x in d['divergences']}; missing={'T-2304','T-2469','T-2813'}-ids; sys.exit('unregistered: %s' % sorted(missing) if missing else 0)"
# The detector runs clean now that they are registered.
bash scripts/check-vendor-divergence.sh
# Fixtures pin it.
bash tests/vendor-divergence-fixtures.sh

## Decisions

### 2026-08-20 — Narrow before reporting

- **Chose:** Triage 12 commits to 3, rather than report the 95 the first scan produced.
- **Why:** "95 local modifications to vendored code" is true, unactionable, and would have been
  read as alarmism. Three named fixes with a named erasure event next week is a thing someone
  can do something about. The recovery commits genuinely are not at risk — a re-vendor restores
  those files from upstream — and including them would have inflated the number by a factor of
  four while making it less true.

### 2026-08-20 — Register, do not re-vendor

- **Chose:** File upstream and record the divergence; leave the re-vendor decision alone.
- **Why:** T-2705 owns that call on another branch, and pre-empting it would be exactly the
  duplicated work T-2800 exists to prevent. The problem is not that a re-vendor is wrong — it
  is that it currently costs three fixes silently. Making the cost visible is the deliverable.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-36ac3b3f
- **Timestamp:** 2026-08-20T18:11:35Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-20T18:11:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
