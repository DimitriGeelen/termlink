---
id: T-2803
name: "One-line remediation script for the main-checkout steps, executed via TermLink"
description: >
  The three mechanical main-checkout steps (track the vendored framework subset, track
  the
  static-check allowlists, clear stale task files) required the operator to hand-run
  six
  commands and eyeball a file list for secrets. Ship one idempotent script that performs
  the
  safety review mechanically, refuses on anything suspicious, never touches the Tier
  0 push,
  and writes a machine-readable report — then execute it in the main checkout through
  a
  TermLink session rather than handing it back as homework.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [governance, remediation, termlink, operator-ux]
components: []
related_tasks: [T-2819, T-2822, T-2815, T-559]
created: 2026-08-20
last_update: 2026-08-20T18:39:06Z
date_finished: 2026-08-20T18:39:06Z
bvp_scores_proposed:
  - ts: '2026-08-20T15:20:38Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 1
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=0 (no-signal); 
      D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=1 
      (body:hand-wired-dispatch)
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

# T-2803: One-line remediation script for the main-checkout steps

## Context

Three fixes shipped on this branch are only half-done, because each needs a commit in the
main checkout at `/opt/termlink` and this session is confined to a worktree by T-559:

1. **T-2819** — the `.gitignore` rule was narrowed so the vendored framework subset is
   trackable, but the files themselves (`lib/bvp.sh`, `policy/`) are still uncommitted. Until
   they are, `fw bvp` fails in every worktree and every clean clone, which is why the BVP
   estimator could not be run this session despite being asked for repeatedly.
2. **T-2822** — same shape for `.context/working/.*-allowlist`. Until committed,
   `check-alloc-sink-clamps` and `check-drain-sink-caps` fire in any fresh checkout on eleven
   sites a human already reviewed and cleared.
3. Stale task files in `/opt/termlink/.tasks/active/` that make the task gate refuse to let
   an agent work in the main checkout at all.

The operator was handed six commands and an instruction to "read that list — confirm nothing
machine-local or secret-bearing appears". That review is the right requirement and the wrong
mechanism: a human scanning a 30-file list for secrets is exactly the kind of check that gets
skimmed on the third repetition. Their reply — *"can you put all in one script i can run from
one line"*, then *"or use termlink to get it done"* — is correct on both counts.

## Approach

**Automate the review, not away the review.** The script scans every file it is about to
stage for high-confidence secret markers (PEM blocks, bare 64-hex secrets matching the
`hub.secret` format, populated `password:`/`api_key:`/`token:` fields) and **refuses to commit
anything if it finds one**, naming the file and line. Softer signals (absolute home paths,
host addresses) are reported for the operator's eye but do not block, because this repo
legitimately discusses `192.168.10.x` throughout. The human judgement is preserved as a
mechanical gate that cannot be skimmed.

**Never touch the push.** Item 4 on the operator's list — `git push --no-verify` past the
failing pre-push audit (T-2815) — is Tier 0 and stays out. The script reports the unpushed
commit count and prints the command; it does not run it, and no flag makes it.

**Idempotent.** Every step is a no-op when already applied, so re-running is safe and the
report tells you which steps did nothing.

**Executed through TermLink, not handed back.** `termlink spawn --cwd /opt/termlink` runs the
script in a session rooted in the main checkout — the path the T-559 hook itself names for
legitimate cross-boundary work, precisely because the target enforces its own governance in
its own process. The script is still committed here as a reviewable, re-runnable artifact
rather than a one-off command stream, so what ran is auditable afterwards.

**Report where the agent can read it.** The run writes JSON into this worktree's
`.context/working/`, which this session can read directly — so the outcome does not depend on
the operator copying terminal output back.

## Acceptance Criteria

### Agent
- [x] One script performs all three mechanical steps and is safe to re-run (idempotent)
- [x] It scans every file it would stage and REFUSES to commit if a high-confidence secret
      marker is found, naming file and line
- [x] Soft signals (home paths, host addresses) are reported but do not block
- [x] It never pushes, and no flag makes it push — Tier 0 stays with the human
- [x] It reports the unpushed commit count and prints the push command without running it
- [x] `--dry-run` shows exactly what would happen and changes nothing
- [x] It writes a machine-readable JSON report this session can read back
- [x] Each step reports applied / already-done / skipped-with-reason — never silent success
- [x] Verification of the outcome is part of the run (the tracking-drift and static checks are
      re-run afterwards and their exit codes recorded)
- [x] Fixtures prove the secret-refusal path on a scratch repo, host-independent (PL-213)
- [x] Executed against the real main checkout via TermLink, with the report read back here

## Verification

bash tests/remediate-main-checkout-fixtures.sh
bash scripts/remediate-main-checkout.sh --dry-run --root .

## Evolution

### 2026-08-20 — the first live dry-run found a false success in this script

- **What changed:** run against the real `/opt/termlink`, the script reported steps 1 and 2 as
  "already done (nothing new to track)". They were not done. `git add --dry-run` on a path
  that is still ignored exits **1** and writes "The following paths are ignored by one of your
  .gitignore files" to **stderr**, printing nothing on stdout — and this script read stdout
  only, ignoring the return code. Empty output was interpreted as success.
- **Plan impact:** two things, and the second is larger than the script.
  First, the script now distinguishes ignored-and-blocked from already-tracked from
  nothing-there, and BLOCKS with the active rule quoted. Second — and this invalidates the
  instructions the operator was given three turns ago — **steps 1 and 2 cannot run in the
  main checkout at all yet.** The `.gitignore` narrowing that makes those paths trackable
  lives on this branch; `main` still carries the blanket rules at `.gitignore:21` and
  `.gitignore:80`. The `git add` commands the operator was handed would have failed for them
  exactly as they failed here.
- **Triggered:** steps 1 and 2 are downstream of the merge, which is downstream of the Tier 0
  push. Only step 3 was actionable, and it has been applied.
- **Worth noting:** a dry run against reality caught a false-success bug that 21 green
  fixtures did not, because every fixture built a repo where the paths were stageable. The
  case that mattered was the one the fixtures could not imagine — the precondition being
  absent. It is now assertion 8b.

## Decisions

### 2026-08-20 — Mechanise the secret review rather than dropping it

- **Chose:** A blocking scan for high-confidence markers; soft signals reported only.
- **Why:** I asked the operator three times to eyeball a file list. A review that is requested
  repeatedly and never fails is a review that has already stopped happening. Encoding it makes
  it real; keeping the soft signals visible keeps the human judgement where it is actually
  needed.
- **Rejected:** Blocking on any occurrence of the word "secret". This repo's framework code is
  full of legitimate discussion of `hub.secret` and secret rotation; a scanner that cried wolf
  there would be turned off immediately.

### 2026-08-20 — TermLink executes; the script is still committed

- **Chose:** Ship the script as a repo artifact, then run it through a TermLink session rooted
  at `/opt/termlink`.
- **Why:** Doing it as an ad-hoc command stream would get it done and leave nothing to review;
  committing the script means the next person can see exactly what was run, and re-run it.
- **Note:** this is the path the T-559 boundary hook itself names — the target directory
  enforces its own governance in its own process, so nothing is bypassed.

### 2026-08-20 — The push stays out

- **Chose:** Report it, print the command, never execute.
- **Why:** Tier 0 is not delegable by a broad "get it done" instruction. The framework's own
  rule is explicit that a structural gate exists precisely for moments when an agent has been
  told to proceed. The operator can approve it in one command; I cannot approve it for them.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-dc38d322
- **Timestamp:** 2026-08-20T18:39:26Z
- **Catalogue:** v1.3-seed
- **Overall:** FAIL
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **skip-as-pass** (severe, deterministic) @ Verification:line 2
     - evidence: `bash scripts/remediate-main-checkout.sh --dry-run --root .`

### 2026-08-20T18:39:06Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
