---
id: T-2878
name: "Meta-canary watches 8 of 20 canaries; one canary cannot be watched at all"
description: >
  T-1723's meta-canary detects a canary that stopped firing, but it is wired per-canary as an explicit cron job line and only 8 of 20 canary crontabs declare one. The other 12 are unwatched: if their cron stops, nothing detects it - the exact G-058 silent-failure the meta-canary was built for. One of the 12, hook-counter-integrity, writes no heartbeat at all so it cannot be watched even in principle, and its stale log makes /canaries read it as permanently FIRING with no path back to HEALTHY. Nothing detects this gap: check-cron-install-drift compares host against git, so a meta job git never declared cannot fire.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: [scripts/check-canary-aliveness.sh, scripts/check-hook-counter-integrity.sh, tests/canary-aliveness-sweep-fixtures.sh, tests/hook-counter-integrity-fixtures.sh]
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
created: 2026-09-01T22:08:08Z
last_update: 2026-09-01T22:27:35Z
date_finished: 2026-09-01T22:27:35Z
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

# T-2878: Meta-canary watches 8 of 20 canaries; one canary cannot be watched at all

## Context

Found by reading `/canaries` at session start. `hook-counter-integrity-canary` showed
`FIRING` with `hb= --` — no heartbeat at all — while an ad-hoc run of the same check
exited 0 and reported clean. Two surfaces disagreeing about one canary is the thread.

Pulling it produced three findings that compound, each invisible on its own.

**1. One canary was scheduled without a heartbeat.** `check-hook-counter-integrity.sh`
was the only cron-scheduled canary in the tree that writes a canary log and never writes
a heartbeat. That is not cosmetic. `canary-status.sh::classify` has an explicit
no-heartbeat branch that classifies by log content alone: empty → HEALTHY, non-empty →
FIRING. There is no arm that returns a non-empty log to HEALTHY. So the single real
finding on 2026-08-31 — a genuine duplicate key with the two readers disagreeing 25 vs
67 — latched it red permanently, and no future run could clear it. That is the stuck-on
direction of the same one-bit channel T-2685 unstuck from the other side: there, a
tooling error dirtied a findings log and made it deaf to real findings; here, a real
finding dirtied it and made it deaf to recovery. Both end with an operator who learns the
signal means nothing.

**2. The script says it must not be scheduled, and it is scheduled.** T-2795 wrote the
header: *"NOT a cron canary… Run it when you need the number; do not schedule it,"*
arguing the corruption is continuous so a scheduled reader would be permanently red.
T-2850 landed that script and then, an hour later in `bd4014284`, scheduled it daily —
for a good reason ("a guard nothing runs is the state 577-CashWeb described") — and never
amended the header. The file has been telling every reader it is unscheduled while
`/etc/cron.d/termlink-hook-counter-integrity-canary` ran it every morning.

Measurement settles which side was right, and neither had measured. The corruption is
**intermittent**, not continuous: a duplicate key survives only until the next hook fire
happens to rewrite the file cleanly. Observed directly during this task — clean at 00:04,
corrupt again at 00:13 from this session's own parallel hook fires. A daily sample of an
oscillating fault is exactly what a canary is for, so scheduling was correct and the
premise behind "do not schedule" was never true.

**3. The meta-canary watches 8 of 20 canaries.** T-1723 exists because G-058 ran 16 days
silently — nothing watched the watcher. But it is wired PER CANARY: each watched canary
needs its own cron job line carrying `HEARTBEAT_FILE` / `CANARY_NAME` /
`CANARY_PROBE_CMD` / `CANARY_CRON_PATH`. Measured from real cron job lines (comments,
blanks and `VAR=value` stripped — a script named only in remediation prose is not a
scheduled job): 20 canary crontabs, 8 declare a meta job, 12 do not. Twelve canaries had
nothing watching them.

And nothing could detect that. `check-cron-install-drift.sh` compares the HOST against
GIT, so it fires when git declares a job the host lacks. A meta job **git never declared**
is invisible to it by construction. The gap was structurally undetectable — G-019 in the
layer whose whole purpose is preventing G-019.

The obvious remediation is twelve more job lines, which is what produced the gap: a
per-canary wiring step is a step someone forgets, and forgetting is silent. The twelve
were not a decision, they were one omission repeated twelve times. So the fix is a sweep
that never names canaries individually and therefore cannot be forgotten for one of them.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] **The blind spot is measured, not asserted.** The count comes from parsing real cron JOB lines (comments, blanks and `VAR=value` stripped) out of the git-tracked `.context/cron/*.crontab`, not from grepping whole files — a script named only in a crontab's remediation prose is not a scheduled job. Measured: 20 canary crontabs, 8 declare a `check-canary-aliveness.sh` meta job, 12 do not.
- [x] **`check-hook-counter-integrity.sh` touches a heartbeat.** It is the only cron-scheduled canary that writes a canary log and never writes a heartbeat, so it cannot be watched even in principle. The touch happens BEFORE the check runs (the T-1723 convention: prove the canary ran even on a healthy cycle), and `--no-heartbeat` opts out for the guard-layer runner so a source-level run cannot mask a dead cron.
- [x] **The stale-log latch is released as a consequence, and shown to be.** `canary-status.sh::classify` sends a heartbeat-less canary with a non-empty log to FIRING with no path back to HEALTHY — a stale entry keeps it red forever, which is the stuck-on direction of the T-2685 one-bit channel. With a fresh heartbeat the same file classifies HEALTHY via the `log_mtime >= heartbeat_mtime` arm. Demonstrated by running the classifier before and after.
- [x] **One sweep covers every canary instead of 12 more cron jobs.** `check-canary-aliveness.sh --all` walks every heartbeat under the working dir and fires on any stale one. Per-canary jobs are left in place — they carry a `CANARY_PROBE_CMD` drift-fold the sweep cannot do generically — so the sweep is a coverage net, not a replacement.
- [x] **The sweep is not blind to the canary that has no heartbeat.** A sweep that walks only heartbeats reports "all alive" while a heartbeat-less canary sits unwatched — most confident exactly where it is most wrong (the T-2680 / preflight-Check-1 shape). The sweep therefore ALSO walks canary logs and fires on any whose heartbeat companion is absent, as a distinct `NO-HEARTBEAT` class.
- [x] **A canary that git schedules no cron for is not reported as dead.** The sweep reuses `canary-status.sh`'s own predicate — a genuine canary crontab names the log it appends to — so the source-level static checks that carry a heartbeat but no crontab are excluded rather than read STALE forever. Without this the sweep reproduces the exact noise T-2826 removed from `/canaries`.
- [x] **The sweep fails closed.** A missing working dir, an absent cron source dir, or zero discovered canaries exits 2, never 0 — a sweep reporting "all alive" because it found nothing to look at is the false assurance this task exists to remove.
- [x] **It is load-bearing, proven by mutation.** Fixtures drive the sweep against fixture dirs via `CANARY_SWEEP_WORKING_DIR` / `CANARY_SWEEP_CRON_DIR` (PL-213) with no live cron: a stale heartbeat fires, a fresh one does not, a log with no heartbeat companion fires as `NO-HEARTBEAT`, a heartbeat with no crontab is excluded, and an empty discovery set exits 2. Removing the `NO-HEARTBEAT` walk turns the hook-counter case green — the mutant that proves that arm carries weight.

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
       Conversion: this AC should be moved to ### Agent and
       `bin/fw reviewer T-XXX > /tmp/.rev 2>&1 && grep -q "Overall:.*PASS" /tmp/.rev`
       added to ## Verification. NEVER `... 2>&1 | grep -q ...` — that is the shape the
       Pipefail/SIGPIPE section below forbids, and this line used to prescribe it.
-->

- [ ] [RUBBER-STAMP] Install the sweep crontab so the coverage net actually fires daily.
  **Steps:**
  1. The crontab is committed at `.context/cron/canary-aliveness-sweep.crontab` but installing to `/etc/cron.d` needs root, so until this step runs the sweep is shipped-but-dark — the exact G-069 state it exists to detect in others.
  2. `cd /opt/termlink && sudo cp .context/cron/canary-aliveness-sweep.crontab /etc/cron.d/termlink-canary-aliveness-sweep`
  3. `cd /opt/termlink && bash scripts/check-cron-install-drift.sh`
  **Expected:** step 3 reports no MISSING or UNINSTALLED_JOBS entry for `canary-aliveness-sweep.crontab`.
  **If not:** `/etc/cron.d` files must be mode 0644 and owned by root, and the filename must not contain a dot — cron silently ignores files that violate either rule, which reproduces the same silent-schedule failure this task closes.

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# ── Pipefail/SIGPIPE: grepping a command's output (L-387, T-2090, T-2743, T-2738) ──
#
# THE DEFAULT — redirect to a file, then grep the file:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
#     curl -sf "$(bin/fw watchtower url)/page" -o /tmp/.out && grep -q "PAT" /tmp/.out
# Correct at any output size, and `&&` keeps the PRODUCING command's exit code in
# the verdict. Reach for this first; the alternative below is the special case.
#
# NEVER `cmd | grep -q PAT` (L-387) — why: P-011 runs each line under `set -eo
# pipefail`. When grep matches it exits and closes stdin while cmd is still
# writing, cmd takes SIGPIPE, the pipeline exits 141 — verification "fails" with
# the pattern present. Captured 4× (T-1716, T-1838, T-1862, T-1863).
#
# THE EXCEPTION — capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Valid ONLY while "$out" fits the 65536-byte pipe buffer, and it is on you to
# know that it does. Above that the form inverts and becomes the very failure
# L-387 describes: echo blocks on the full pipe, grep -q exits, echo takes
# SIGPIPE, rc=141 (T-2743 — measured on a 146,366-byte Watchtower page, 3/3 runs,
# deterministic not racy; rendered routes run 50-200KB, so anything that curls a
# page is over the line). It also discards cmd's exit code, so a 404 yields an
# empty capture that grep merely fails to match rather than a failed line.
# If you do use it: single pipe only, no intermediate tail/awk/sed stage between
# capture and grep (T-2090) — the middle stage is what `grep -q` slams its stdin
# on, and grep scans the whole captured string anyway, so the `tail -3` was
# cosmetic. `echo "$out" | grep -q PAT`, nothing between.
#
# TEST RUNNERS need a guard either way (T-2738). `set -e` is suppressed inside the
# `if` condition the gate runs each line in, so in `cmd1; cmd2` only cmd2 is the
# verdict — and the pass marker you grep for survives a partial failure: a suite
# printing "3 failed, 9 passed" satisfies `grep -q "9 passed"`, and generalising
# to `grep -qE "[0-9]+ passed"` matches the same output. Keep the exit code:
#     python3 -m pytest <file> -q > /tmp/.out 2>&1 && grep -q passed /tmp/.out
# or add the guard the exit code used to supply:
#     out=$(python3 -m pytest <file> -q 2>&1); echo "$out" | grep -q passed && ! echo "$out" | grep -q failed
#     out=$(bats <file> 2>&1); echo "$out" | grep -q '^ok 1 ' && ! echo "$out" | grep -q '^not ok'
# The close gate refuses the unguarded form. Bypass: FW_ALLOW_UNJUDGED_TEST_RUN=1.
#
# REHEARSING A LINE BY HAND DOES NOT REHEARSE THE GATE (T-2743). Your interactive
# shell has no `set -eo pipefail`. A line has returned 0 by hand and 141 under
# P-011, from the same directory, the same second. To rehearse for real:
#     bash -c 'set -eo pipefail; <your verification line>'
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

bash tests/canary-aliveness-sweep-fixtures.sh > /tmp/.t2878a.txt 2>&1 && grep -q "25 passed, 0 failed" /tmp/.t2878a.txt
bash scripts/check-canary-aliveness.sh --all > /tmp/.t2878b.txt 2>&1 && grep -q "cron-scheduled canaries alive" /tmp/.t2878b.txt
bash scripts/check-canary-aliveness.sh --help > /tmp/.t2878h.txt 2>&1 && grep -q -- "--all            # sweep every cron-scheduled canary" /tmp/.t2878h.txt
grep -q "HOOK_COUNTER_HEARTBEAT_FILE" scripts/check-hook-counter-integrity.sh
grep -q -- "--no-heartbeat)    HEARTBEAT=0" scripts/check-hook-counter-integrity.sh
test -f .context/working/.hook-counter-integrity-canary.heartbeat
bash scripts/check-canary-log-hygiene.sh > /tmp/.t2878c.txt 2>&1 && grep -q "clean" /tmp/.t2878c.txt
# canary-status legitimately exits 1 while an UNRELATED canary (waker-liveness) is
# firing on live host state, so `&&` would make this line assert the health of the
# whole fleet rather than the one classification it is about. `|| true` then grep is
# deliberate here, not the unguarded test-runner shape the header warns about.
bash scripts/canary-status.sh > /tmp/.t2878d.txt 2>&1 || true; grep -qE "HEALTHY.*hook-counter-integrity-canary" /tmp/.t2878d.txt
bash -n scripts/check-canary-aliveness.sh
bash -n scripts/check-hook-counter-integrity.sh

## RCA

**Symptom:** `/canaries` reported `hook-counter-integrity-canary` FIRING with no
heartbeat, while an ad-hoc run of the same check exited 0 and printed "clean".

**Root cause:** the canary was scheduled (T-2850, `bd4014284`) without the T-1723
heartbeat touch. `canary-status.sh::classify` handles a heartbeat-less canary by log
content alone and has no arm that returns a non-empty log to HEALTHY, so the one real
finding from 2026-08-31 latched it red with no path back.

**Why structurally allowed:** three compounding gaps, none visible alone.
1. **No check couples "is scheduled" to "writes a heartbeat".** The T-1723 convention is
   documented and followed by 19 of 20 canaries, but by discipline only. The 20th was
   added by a task whose ACs were about filing a vendored defect upstream, not about
   canary wiring, so nothing in its review path would have asked.
2. **The meta-canary is opt-in per canary and nothing audits the opt-in list.**
   `check-cron-install-drift.sh` compares host against git and therefore cannot see a job
   git never declared. A canary shipped without a watcher is undetectable by design.
3. **The script's own header was stale in the load-bearing direction.** It said "do not
   schedule it" while cron ran it daily, so the one document that would have told a reader
   this canary needed different handling was actively misleading. The claim was also never
   measured — the corruption is intermittent, not the continuous state the header asserted.

**Prevention:** the `--all` sweep's `NO-HEARTBEAT` class is the structural fix. It fires on
any cron-scheduled canary that writes no heartbeat, so the next one added this way is named
the following morning rather than latching red unnoticed. Deliberately paired with the
`STALE` class in the same walk: a sweep over heartbeats alone would report "all alive"
while a heartbeat-less canary sat unwatched — most confident exactly where it is most
wrong, the T-2680 shape. Proven by mutation: deleting the log walk reddens 5 fixture
assertions, deleting the fail-closed guard reddens 2, deleting the unscheduled-canary
exclusion reddens 4.

**Scope — read a green narrowly.** The sweep answers one question: is every
cron-scheduled canary either heartbeat-fresh or named? It does **not** verify that any
canary's logic is correct, that its findings are acted on, or that a canary git declares
no crontab for should exist. A canary whose crontab was never installed writes neither log
nor heartbeat and is invisible to this sweep — that is `check-cron-install-drift.sh`'s
MISSING class, and the two are complements.

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

**Recommendation:** GO — install the sweep crontab.

**Rationale:** The code half is complete and proven; what remains is one `sudo cp`
that only root can do. Until it runs, the sweep exists but never fires, which is the
same shipped-but-dark state (G-069) this task exists to detect in others — and
`check-cron-install-drift.sh` is already naming it MISSING, so the gap is surfaced by
an existing guard rather than by my assertion.

The install is low-risk and reversible: one read-only daily job at 09:53 UTC that
stats heartbeat mtimes and greps crontabs. It writes only to the shared
`.canary-aliveness.log` that eight per-canary meta jobs already append to, changes no
existing job, and `rm /etc/cron.d/termlink-canary-aliveness-sweep` fully reverts it.

What it buys immediately: 12 canaries currently have nothing watching them, including
`dead-letter`, `stuck-claims`, `waker-liveness` and `task-finalization`. If any of
their crons stopped tonight, the framework would not notice — the G-058 condition that
ran 16 days silently and is the reason the meta-canary was built.

**Evidence:**
- Measured from real cron job lines (comments/blanks/`VAR=` stripped): 20 canary
  crontabs, 8 declare a meta job, 12 do not.
- `check-cron-install-drift.sh` exits 1 naming `canary-aliveness-sweep.crontab` →
  `/etc/cron.d/termlink-canary-aliveness-sweep` as MISSING.
- `check-canary-log-hygiene.sh` clean — 32 findings-log job lines across 26 crontabs
  keep stdout and stderr separate, the new job included.
- Live sweep against the real tree: `all 20 cron-scheduled canaries alive (threshold
  48h; 8 unscheduled heartbeat(s) excluded)`, exit 0.
- Load-bearing by mutation: dropping the log walk reddens 5 fixture assertions,
  dropping the fail-closed guard 2, dropping the unscheduled-canary exclusion 4;
  dropping the heartbeat touch reddens 4 hook-counter assertions.
- Fixtures: 25/25 (sweep) and 33/33 (hook-counter, up from 26 with the stale G2
  prose-pin replaced by substance assertions).
- P-011 gate: 10/10 verification commands passed.
- Latch released on the real tree: `hook-counter-integrity-canary` moved from FIRING
  (permanent, no path back) to HEALTHY.

**What this does NOT claim:** the sweep verifies that every cron-scheduled canary is
heartbeat-fresh or named. It says nothing about whether any canary's logic is correct
or whether its findings get acted on, and a canary whose crontab was never installed
writes neither log nor heartbeat and is invisible to it — that remains
`check-cron-install-drift.sh`'s MISSING class.

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

### 2026-09-01T22:08:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2878-meta-canary-watches-8-of-20-canaries-one.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-1c51a37b
- **Timestamp:** 2026-09-01T22:27:38Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-01T22:27:35Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
