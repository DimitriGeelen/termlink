---
id: T-2790
name: "Close the T-2685 deployment gap — verify installed crontabs carry the stderr split and un-strand the fix from the worktree branch"
description: >
  T-2685 split canary stderr out of the findings logs, but the commit (124a67e35) sits on the worktree branch and never reached main, so /opt/termlink still ships the unfixed crontabs. A deploy loop run from the main checkout copied unfixed over unfixed and exited 0 — a silent no-op. Verify the corrected deploy took effect, confirm check-cron-install-drift.sh clears its 21 JOB_DRIFT findings, and resolve the stranding so the next deploy from main is not another no-op.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-18T12:26:59Z
last_update: 2026-08-18T12:30:03Z
date_finished: 2026-08-18T12:30:03Z
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
---

# T-2790: Close the T-2685 deployment gap — verify installed crontabs carry the stderr split and un-strand the fix from the worktree branch

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Installed crontabs carrying the T-2685 stderr split (`grep -l 'log.stderr' /etc/cron.d/termlink-*`)
      equals the git-declared count in this worktree (21), measured — not assumed from a
      zero-exit `cp`. — **21 installed / 21 declared**; 2 remaining `2>&1` are the crontabs
      git itself does not declare a split for; 24 total; `systemctl is-active cron` → active.
- [x] `scripts/check-cron-install-drift.sh --json` reports `job_drift_count: 0`
      (was 21 before the deploy; the check is the independent witness, not the `cp` exit code).
      — **`{"ok":true,"missing_count":0,"uninstalled_jobs_count":0,"job_drift_count":0,
      "drift_count":0,"ok_count":24}`**. `uninstalled_jobs_count: 0` is the load-bearing one:
      the two genuinely-unscheduled meta-canary job lines T-2787 un-buried
      (fleet-doorbell-mail, substrate-preflight — the jobs that detect a canary going dark)
      are now scheduled as well. The deploy closed both classes at once.
- [x] The stranding root cause is stated in this task: T-2685 is commit `124a67e35` on
      `worktree-charter-review-2026-0814`, absent from `main`, and `main` is what
      `/opt/termlink` (and therefore cron) serves — PL-357.
      — Confirmed by `git grep -l 'log.stderr' main -- '.context/cron/*.crontab'` → **0**,
      same query against `HEAD` → **21**.
- [x] The recurrence is recorded: PL-357 already named this class after T-2764 and did not
      prevent it. Either a learning entry notes the recurrence, or a structural check is
      filed — a second identical instance is the argument that documentation is not a control.
      — **PL-362** records the recurrence and the two compounding properties (`cp` cannot
      witness a deployment; the branch/checkout mismatch is invisible from the deploy command).
- [x] No merge to `main` performed on agent initiative; if un-stranding requires it, it is
      surfaced to the human as an explicit action rather than done.
      — Surfaced, not performed. `/etc/cron.d` is correct NOW, but `main` still carries the
      unfixed crontabs, so the next deploy run from `/opt/termlink` would silently revert all
      21. See the Human AC below.

### Human
- [ ] [REVIEW] Decide how to un-strand T-2685 (and the rest of this branch) from `main`
  **Steps:**
  1. `cd /opt/termlink && git log --oneline main..worktree-charter-review-2026-0814 | wc -l`
     — see how many commits are stranded.
  2. Decide: merge the branch to `main`, or cherry-pick just `124a67e35` (the crontab fix).
  3. Whichever you choose, re-run the witness afterwards:
     `bash scripts/check-cron-install-drift.sh --json`
  **Expected:** `git grep -l 'log.stderr' main -- '.context/cron/*.crontab' | wc -l` → 21,
  and the drift check still reports `job_drift_count: 0`.
  **If not:** the fix is still branch-local; anyone deploying from `/opt/termlink` reverts
  today's work with a command that exits 0 and says nothing (PL-362).

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.

     ── Prefix routing (T-1811, T-1878): default to [REVIEWER] if Expected is grep-able ──
     If your Expected clause is grep-able / file-exists / structural (a deterministic
     shell check), prefer [REVIEWER] — that AC should be an Agent AC with the reviewer
     command in `## Verification` instead of a Human AC here. Only keep [REVIEW] if
     verification genuinely needs human taste (tone, feel, layout rhythm).
     See CLAUDE.md §AC Classification Guidance for the conversion rule.

     [REVIEW] example (genuine human judgment):
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error

     [REVIEWER] example (static-scan-verifiable — convert to Agent AC + Verification):
       - [ ] [REVIEWER] Block message names both bypass mechanisms
         **Steps:**
         1. Run `bin/fw reviewer T-XXX`
         **Expected:** Verdict: PASS; no findings on `block-message-completeness`
         **If not:** Inspect hook block-message string and add missing mechanism
       Conversion: this AC should be moved to ### Agent and this line added to
       ## Verification (herestring, not a pipeline — see the L-387 hint below):
         out=$(bin/fw reviewer T-XXX 2>&1 || true); grep -q "Overall:.*PASS" <<< "$out"
-->

## Verification

# The independent witness — never the deploy's own exit code (PL-362).
out=$(bash scripts/check-cron-install-drift.sh --json 2>&1 || true); grep -q '"job_drift_count":0' <<< "$out"
out=$(bash scripts/check-cron-install-drift.sh --json 2>&1 || true); grep -q '"uninstalled_jobs_count":0' <<< "$out"
out=$(bash scripts/check-cron-install-drift.sh --json 2>&1 || true); grep -q '"missing_count":0' <<< "$out"
# Installed reality matches what git declares — a count, not a zero exit.
n=$(grep -l 'log.stderr' /etc/cron.d/termlink-* 2>/dev/null | wc -l); test "$n" -eq 21
# cron is actually running, or none of the above is scheduled at all.
out=$(systemctl is-active cron 2>&1 || true); grep -q '^active$' <<< "$out"
# The recurrence is on record.
out=$(grep -c 'PL-357 RECURRED' .context/project/learnings.yaml 2>/dev/null || echo 0); test "$out" -ge 1

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# Pipefail/SIGPIPE hint (L-387, corrected by T-2775): P-011 runs each command
# under `set -eo pipefail`. NEVER write `cmd | grep -q PATTERN`: it exits 141
# (SIGPIPE) when grep matches and closes stdin while the upstream is still
# writing — verification then "fails" BECAUSE the check succeeded, and the
# earlier the match, the more reliably it fails.
#
# USE ONE OF THESE — both measured rc=0 at 3M lines:
#     out=$(cmd 2>&1 || true); grep -q "PATTERN" <<< "$out"   # herestring (preferred)
#     test -n "$(cmd | grep -m1 PATTERN)"                     # pipeline inside $( )
#
# The herestring is preferred: a herestring spawns no producer process, so there
# is nothing to SIGPIPE and it cannot regress as output grows. In the second form
# the pipeline sits inside a command substitution, whose status is discarded — the
# OUTER `test` decides.
#
# DO NOT capture-then-pipe. This template previously prescribed
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"     # UNSAFE above ~64KB
# and it is size-dependent, not safe: `echo`/`printf` is a producer like any
# other, so once $out exceeds the pipe buffer it is still writing when `grep -q`
# exits and pipefail propagates 141. The capture bounds the DATA but does not
# remove the PRODUCER. Anything wrapping `cargo test`, `fleet doctor --json`, or a
# full log is already in that size range. (T-2775 measured this; 999-AEF L-613 and
# 050-email-archive PL-161 published the capture-then-pipe form before the
# correction — both have since adopted the herestring.)
#
# Corollary (T-2090): intermediate stages are just as fatal — `... | tail -3 |
# grep -q PAT` re-introduces the same risk. With a herestring the question does
# not arise; grep scans the whole captured string anyway.
#
# Origin: L-387, captured 4× (T-1716, T-1838, T-1862, T-1863) before the hint;
# T-2775 then measured 1490 exposed lines across 802 tasks despite the hint, which
# is why `scripts/check-verification-pipefail.sh` now enforces it structurally.
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

## Recommendation

**Recommendation:** GO — un-strand the branch (merge to `main`, or cherry-pick `124a67e35`).

**Rationale:** `/etc/cron.d` is correct as of today, but that state is *not durable*. `main`
still carries the unfixed crontabs, and `main` is what `/opt/termlink` serves. So the next
person who runs the documented deploy loop from the main checkout silently reverts all 21
crontabs — with a command that exits 0 and prints nothing. That is not a hypothetical: it is
exactly what happened this session, and PL-357 had already named the class after T-2764
without preventing it. Leaving the fix branch-local means the *second* recurrence is set up
in advance.

The narrow cherry-pick is sufficient for the crontabs and lower-risk than merging the whole
branch; the merge is the better answer if the branch is otherwise ready. That choice is the
human's — it is a `main`-branch action, and agent initiative does not extend to merging.

**Evidence:**
- `git grep -l 'log.stderr' main -- '.context/cron/*.crontab'` → **0**; same against `HEAD` → **21**.
- Deploy run from `/opt/termlink` copied unfixed→unfixed, exit 0, no output: installed split
  count stayed **0/24** across the attempt.
- Deploy re-run from the worktree: **21** installed, `systemctl is-active cron` → active.
- Independent witness `scripts/check-cron-install-drift.sh --json`:
  `{"ok":true,"missing_count":0,"uninstalled_jobs_count":0,"job_drift_count":0,"drift_count":0,"ok_count":24}`
  — down from `job_drift: 21, uninstalled_jobs: 2`.
- `uninstalled_jobs_count: 0` also means the two meta-canary job lines T-2787 un-buried
  (fleet-doorbell-mail, substrate-preflight) are scheduled for the first time.
- Recurrence recorded as **PL-362**; verification block asserts the witness, not the deploy.

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

**Symptom:** The operator ran the documented crontab deploy loop from `/opt/termlink`. It
exited 0 and printed nothing. Nothing changed: installed crontabs carrying the T-2685 stderr
split stayed at **0 of 24**, and `check-cron-install-drift.sh` kept reporting
`job_drift: 21, uninstalled_jobs: 2`.

**Root cause:** T-2685 (`124a67e35`) was committed on the long-lived worktree branch
`worktree-charter-review-2026-0814` and never reached `main`. `/opt/termlink` — the checkout
the deploy loop reads, and the one cron's job lines `cd` into — is on `main`, whose
`.context/cron/*.crontab` still carried `2>&1`. The loop therefore copied *unfixed over
unfixed*: a correct execution of a correct command against the wrong source tree.

**Why structurally allowed:** two properties compound, and neither is visible from the deploy
command itself.
1. `cp` cannot witness a deployment. It returns 0 for a copy that changes nothing, so exit
   status is indistinguishable between "installed the fix" and "reinstalled the bug".
2. The source-of-truth branch is implicit. The deploy loop names a *directory*, not a
   revision, so a fix living on another branch is invisible at the point of use — the operator
   sees `.context/cron/*.crontab` and has no signal that these are not the fixed ones.
Compounding both: the fix being *committed* and the CI guard-layer being *green on the branch*
made every conventional signal read healthy while the host ran the old configuration. Shipped,
not live (G-069), one layer below the binary where that gap is usually watched.

**Prevention:** not another instruction — PL-357 already stated this precisely after T-2764
and did not prevent the recurrence, which is the standing argument (cf. T-2746, T-2775) that a
learning is not a control. What actually caught it was
`scripts/check-cron-install-drift.sh` — an independent post-condition witness that compares
installed reality against what git declares, and which T-2787 had just taught to distinguish
`JOB_DRIFT` from `UNINSTALLED_JOBS`. That check is now the assertion in this task's
`## Verification` block (`job_drift_count:0`, `uninstalled_jobs_count:0`, `21` installed),
so the deploy's own exit code is never the evidence. **Residual gap:** the check compares
against the *working tree*, so it cannot see that the working tree itself is a branch cron
never reads — it would have reported clean had this been run from `/opt/termlink`. Closing
that means teaching it (or `fw doctor`) to warn when guard-layer sources are ahead of the
branch the host serves. Recorded as PL-362; filing the structural check is the human decision
in `## Recommendation`.

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
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

### 2026-08-18T12:26:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2790-close-the-t-2685-deployment-gap--verify-.md
- **Context:** Initial task creation

### 2026-08-18T12:30:03Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
