---
id: T-2692
name: "Static-check allowlists are untracked — three of the four source-level checks fire 15 false positives in any fresh checkout"
description: >
  The four source-level static checks (alloc-sink T-2527, drain-sink T-2531,
  silent-exit T-2666, busy-spin T-2672) record their confirmed-safe
  acknowledgements — with cited reasons — in `.context/working/.*-allowlist`.
  That directory is gitignored and these four files are NOT among the 115 files
  force-tracked there, so the acknowledgements exist on exactly one machine. In
  any fresh clone, worktree, or CI run the checks fire on every allowlisted site:
  measured 15 false positives (5 alloc-sink + 6 drain-sink + 4 busy-spin).
  CLAUDE.md states each check "scans CLEAN" — a claim that only holds where the
  untracked file happens to exist.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [governance, static-check, reproducibility, bug]
components: [.context/working/.alloc-sink-allowlist, .context/working/.drain-sink-allowlist, .context/working/.busy-spin-allowlist, .context/working/.silent-exit-allowlist]
related_tasks: [T-2527, T-2531, T-2666, T-2672, T-2690]
created: 2026-08-18T21:50:00Z
last_update: 2026-08-18T22:35:04Z
date_finished: 2026-08-18T22:35:04Z
---

# T-2692: Static-check allowlists are untracked

## Context

Found while running the four static checks from a clean worktree (T-2690 work).
`.gitignore:80` ignores `.context/working/` wholesale. The project already
force-tracks 115 files under that path, so "important working state gets
force-added" is an established convention — the four static-check allowlists
simply never got it.

Two distinct costs:

1. **The checks are not reproducible.** From a fresh checkout, `check-alloc-sink-clamps.sh`,
   `check-drain-sink-caps.sh` and `check-busy-spin.sh` fire on 15 sites that are
   confirmed-safe. A check that cries wolf everywhere except one machine is not a
   gate; it is noise, and it trains the reader to ignore it — the same alarm-fatigue
   mechanism T-2690 fixed for `/canaries`.
2. **The judgments are unversioned.** CLAUDE.md requires each allowlist entry to carry
   "a cited reason". Those reasons currently have no history, no review trail, and no
   backup — the record of *why* a potentially-OOMing allocation was deemed safe lives in
   a gitignored file. That is governance state, and governance state belongs in git.

## Acceptance Criteria

### Agent
- [x] Each of the 15 firing sites is **independently re-verified against the source**
      before being acknowledged — the reasons are written from reading the code, not
      copied from CLAUDE.md's summary.
- [x] The four allowlist files are force-added to git (`git add -f`, the same treatment
      the other 115 `.context/working/` files get) with one drift-stable signature per
      line and its cited reason.
- [x] All four checks scan CLEAN from this worktree — proving the restored files carry
      correct signatures, since a wrong signature keeps its site firing.
- [x] CLAUDE.md records that the allowlists are tracked and must stay tracked, so the
      next check author does not reintroduce the gap.

### Human
- [ ] [RUBBER-STAMP] Confirm the restored allowlists match the host's own copies
  **Steps:**
  1. `cd /opt/termlink && git status --short .context/working/ | grep allowlist`
  2. After merging this branch: `bash scripts/check-alloc-sink-clamps.sh && bash scripts/check-drain-sink-caps.sh && bash scripts/check-busy-spin.sh && bash scripts/check-silent-exit.sh`
  **Expected:** all four report `clean`. If the host's pre-existing untracked copies held
  entries beyond the 15 restored here, the merge shows them as a diff rather than losing them.
  **If not:** a site that fires after merge is either a genuinely new unclamped sink
  (fix the code) or a signature that drifted (a renamed enclosing fn re-fires by design).

## Verification

bash scripts/check-alloc-sink-clamps.sh
bash scripts/check-drain-sink-caps.sh
bash scripts/check-busy-spin.sh
bash scripts/check-silent-exit.sh

## Recommendation

**Recommendation:** GO

**Rationale:** No source code changes — this adds four previously-untracked data files to
git and one CLAUDE.md section. The only way it can be wrong is if a restored signature is
inaccurate, and that is self-detecting: a wrong signature leaves its site FIRING, so
"all four checks scan clean" is itself the proof the file is correct. Every one of the 15
entries was re-verified against source rather than copied from the CLAUDE.md summary.

**Evidence:**
- Before: `check-alloc-sink` 5 firing, `check-drain-sink` 6 firing, `check-busy-spin` 4
  firing from a clean worktree. After: all four report `clean`.
- Scanned counts match CLAUDE.md's documented figures exactly (6 drain sinks, 14 long-poll
  loops, 39 non-zero-literal exits), so the checks are seeing the same tree the docs describe.
- Re-verification found the guard the grep cannot see:
  `validate_dispatch_count(p.count)` at `tools.rs:13482` (max `MAX_DISPATCH_COUNT` = 256,
  loud-reject) protects both `count` allocation sites at 13536/13623.
- Drain sinks: each confirmed as `current_exe()` / `bash <repo script>` with
  `kill_on_drop(true)` + `stdin(null)` + `tokio::time::timeout`.
- Busy-spin loops: each confirmed to leave the loop on error — `cmd_wait` `bail!`s;
  `termlink_request` / `termlink_wait` / `termlink_agent_ask` return on a wall-clock deadline.

**What the human is being asked for:** confirm the restored files agree with the host's own
untracked copies. If the host held entries beyond these 15, the merge surfaces them as a
diff rather than losing them — which is the point of tracking them.

## RCA

**Symptom:** Three of four static checks FIRE with 15 findings in a clean worktree, while
CLAUDE.md documents all four as scanning clean.

**Root cause:** The allowlist files live under `.context/working/`, which `.gitignore:80`
ignores wholesale. Every other durable file under that path was force-added; these four
were not, so they never left the machine they were created on.

**Why structurally allowed:** Nothing verifies that a check's own acknowledgement state is
reproducible. The checks were introduced with fixture suites that pass an explicit
`--allowlist` path, so the fixtures kept passing regardless of whether the *real*
allowlist was tracked — the tests proved the mechanism, never the deployment.

**Prevention:** The four files are now tracked, so a fresh clone reproduces "clean", and
any future edit to a safety acknowledgement shows up in review as a diff with its reason.
CLAUDE.md states the tracking requirement for the next check author.

## Decisions

### 2026-08-18 — Re-verify rather than trust the documented reasons
- **Chose:** Read each of the 15 sites in source and write the reason from what the code
  actually does, even though CLAUDE.md already asserts the classes are safe.
- **Why:** An allowlist entry is a standing assertion that a potentially-OOMing allocation
  or a CPU-spinning loop is safe. Restoring it from a prose summary would launder an
  unchecked claim into a machine-enforced exemption. Re-verification also paid for itself:
  it confirmed `count` at tools.rs:13536/13623 is bounded by `validate_dispatch_count`
  (max 256, loud-reject) called at 13482 — a guard shape the grep structurally cannot see,
  which is exactly why the site needs an acknowledgement rather than a code change.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-bd62bd20
- **Timestamp:** 2026-08-18T22:35:45Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-18T22:35:04Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
