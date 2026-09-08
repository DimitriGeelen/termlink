---
id: T-2928
name: "T-2870's flock regression check passes while pickup duplicates are still being created — 4 envelopes became 8 tasks today"
description: >
  T-2870's flock regression check passes while pickup duplicates are still being created — 4 envelopes became 8 tasks today

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
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
created: 2026-09-08T19:50:25Z
last_update: 2026-09-08T19:50:25Z
date_finished: null
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

# T-2928: T-2870's flock regression check passes while pickup duplicates are still being created — 4 envelopes became 8 tasks today

## Context

T-2870 diagnosed that `fw pickup process` runs from more than one cron entry on
this host without `flock`, so two overlapping runs each create a task for the same
envelope before the dedup marker is written. Its agent ACs are all checked —
including AC2, *"A regression check fails if any `fw pickup process` cron line on
this host lacks flock"* — and its one remaining AC is a `[RUBBER-STAMP]` human step
to install the serialised lines.

Duplicates were nonetheless produced **today**, after that guard shipped:

| distinct content | tasks created (`created:` stamp) | copies |
|---|---|---|
| opencode ignores `.mcp.json` | T-2920 (18:23:01) | 1 |
| aef-guards.ts block messages | T-2921 (18:23:56), T-2922 (18:24:01) | 2 |
| vendored-mode ask.py | T-2923 (19:26:01), T-2924 (19:27:02) | 2 |
| stale daemon lock | T-2925 (19:28:02), T-2926 (19:28:03), T-2927 (19:29:01) | 3 |

Four distinct pickups, eight task files. `diff T-2921 T-2922` shows only the `id:`
line, the `created`/`last_update` stamps and the self-referencing filename —
byte-identical content otherwise. The gaps between copies are diagnostic: T-2925 /
T-2926 are **one second** apart (two runs genuinely overlapping), while T-2926 /
T-2927 are 58 s apart (the next `* * * * *` tick re-creating it because the dedup
marker still had not landed).

Two questions follow, and the second is the one that matters:

1. Why is the duplication still happening — is the fix merely un-installed
   (T-2870's pending human stamp), or is the shipped guard looking somewhere the
   live cron line is not?
2. If the guard passes while the defect it names is actively firing, the guard is
   the more serious defect. That is the G-019 shape this repo treats as primary:
   a green check is why nobody looks.

This task answers (1), fixes whichever of the two it turns out to be if the fix is
agent-ownable, and purges the duplicate task files already in the register. It
deliberately does **not** self-stamp T-2870's human AC.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] **The live `fw pickup process` cron lines are enumerated from the host, not from git.** Every location that can schedule the job — `/etc/cron.d/*`, the root user crontab, and any project-local registry — is read and recorded, with the flock status of each line. The count of *live* lines is stated, because T-2870's premise ("two lines") is itself an inherited claim that this task re-measures rather than assumes.
- [x] **The reason the shipped guard did not fire is established and written down.** Exactly one of: (a) the guard is correct and the fix is simply not installed yet (T-2870's pending human AC), so duplication is expected until it is stamped; or (b) the guard reads a source that does not contain the live line, in which case it reports green over a firing defect. The evidence for whichever it is — the guard's own source, the path it reads, and its exit code when run now — is recorded in this task.
- [x] **If (b), the guard is corrected so it fails against the host as it stands today.** Load-bearing test: the corrected check exits non-zero right now, before any cron change, because an unflocked line is genuinely live. If (a), no code change is made and this criterion records that instead — resisting the urge to invent work is part of the answer.
- [x] **The four duplicate task files are removed, and the surviving ID for each envelope is stated.** `T-2922`, `T-2924`, `T-2926`, `T-2927` are byte-identical re-creations of `T-2921`, `T-2923`, `T-2925` (the last twice). Removal is verified by a re-run of the duplicate scan reporting zero duplicate groups across `.tasks/**`.
- [x] **The purge is proven to have removed only duplicates.** For each removed file, a diff against its surviving twin is recorded showing the only differences are the `id:` line, the `created`/`last_update` timestamps, and the file's self-referencing name — so no unique content was destroyed.

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

# AC1/AC2 — the guard reads the LIVE host and correctly names both unflocked lines.
# Asserted as FIRING on purpose: until T-2870's human stamp installs the flock, a
# green here would mean the check had stopped seeing the defect. `|| true` because
# the script's correct verdict today is exit 1, and P-011 runs lines under `set -e`.
bash scripts/check-pickup-cron-lock.sh > /tmp/.t2928-lock.txt 2>&1 || true
grep -q "2 pickup cron line" /tmp/.t2928-lock.txt
grep -q "agentic-pickup-termlink: pickup line has NO flock" /tmp/.t2928-lock.txt
grep -q "agentic-audit-termlink: pickup line has NO flock" /tmp/.t2928-lock.txt

# AC2 — the guard is executed by nothing: it appears in no crontab under .context/cron/.
# This line is EXPECTED TO BE DELETED by the follow-up task that gives it a runner;
# it exists to pin the finding as measured, not as a permanent invariant.
test -z "$(grep -rl 'check-pickup-cron-lock' .context/cron/ 2>/dev/null)"

# AC3 — the guard's own fixtures stay green (no change was made to it; this proves that).
bash tests/pickup-cron-lock-fixtures.sh > /tmp/.t2928-fix.txt 2>&1 && grep -q "0 failed" /tmp/.t2928-fix.txt

# AC4 — the four duplicate files are gone and the three survivors remain.
test ! -e .tasks/active/T-2922-pickup-aef-guardsts-block-messages-never.md
test ! -e .tasks/active/T-2924-pickup-vendored-mode-defects-askpy-proje.md
test ! -e .tasks/active/T-2926-pickup-stale-daemon-lock-dead-pid-blocks.md
test ! -e .tasks/active/T-2927-pickup-stale-daemon-lock-dead-pid-blocks.md
test -e .tasks/active/T-2921-pickup-aef-guardsts-block-messages-never.md
test -e .tasks/active/T-2923-pickup-vendored-mode-defects-askpy-proje.md
test -e .tasks/active/T-2925-pickup-stale-daemon-lock-dead-pid-blocks.md

# AC4 — normalised content-hash over .tasks/active/ reports zero duplicate groups.
python3 -c "import re,glob,hashlib,collections;g=collections.defaultdict(list);[g[hashlib.sha256(re.sub(r'\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}\S*','<TS>',re.sub(r'^(id|created|last_update|date_finished):.*$','',open(p,encoding='utf-8',errors='replace').read(),flags=re.M).replace(re.search(r'^id:\s*(\S+)',open(p,encoding='utf-8',errors='replace').read(),re.M).group(1),'<ID>')).encode()).hexdigest()].append(p) for p in glob.glob('.tasks/active/*.md') if re.search(r'^id:\s*\S+',open(p,encoding='utf-8',errors='replace').read(),re.M)];d=[v for v in g.values() if len(v)>1];assert not d, d;print('0 duplicate groups')"

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

**Symptom.** Four pickup envelopes produced eight task files on 2026-09-08, three
duplicate groups, one of them a triple — a day *after* T-2870 shipped a regression
check whose stated job is to fail when exactly this can happen.

**Measured ground truth (AC1).** Three `pickup process` cron lines exist on this
host. Two are this project's and **neither carries `flock`**:

| installed file | schedule | flock |
|---|---|---|
| `/etc/cron.d/agentic-pickup-termlink:5` | `* * * * *` | **no** |
| `/etc/cron.d/agentic-audit-termlink:39` | `*/15 * * * *` | **no** |
| `/etc/cron.d/agentic-audit-999-agentic-engineering-framework:48` | `* * * * *` | yes (different project, own lock) |

T-2870's inherited premise — two lines, neither locked — is re-measured and
**confirmed**, not assumed. The root user crontab carries no pickup line.

Note the git-tracked source (`.context/cron/agentic-audit.crontab:39,42`) *does*
carry `flock` on both. So the fix exists in version control and is absent from the
host: the `check-cron-install-drift.sh` DRIFT/UNINSTALLED_JOBS class (T-2561/T-2682).

**Root cause of the duplication.** `fw pickup process` writes the dedup marker
*after* creating the task, so two unserialised runs each pass the "already
processed?" test and each create a file. The one-second gap between T-2925 and
T-2926 is genuine overlap; the 58-second gap to T-2927 is the next tick finding the
marker still unwritten. The window is real and the every-minute schedule keeps
re-entering it.

**Why the guard did not surface it — and the answer is *not* the one the ACs
anticipated (AC2).** Both drafted branches were wrong:

- Not (b). `scripts/check-pickup-cron-lock.sh` was run against the host during this
  task and exits **1**, naming both unflocked lines correctly. It reads
  `/etc/cron.d` — the live state, not git. The guard is right.
- Not simply (a) either. "The fix is merely un-installed" is true but incomplete,
  and stated alone it is exculpatory in a way the evidence does not support.

The actual finding is a third thing: **the guard is correct, is firing, and is
executed by nothing.** `grep -rn check-pickup-cron-lock` across the repo returns its
own task file, session handovers, and one comment in `.context/cron-registry.yaml`
— no cron entry, no CI job, no caller. It carries no `# guard-layer:` marker, so
`run-guard-layer.sh` lists it among **85 unclassified** scripts and never runs it.

That distinction matters. Under (a) the story is "we are waiting on a stamp, and
the moment it lands we are safe." What is actually true is that the stamp was
pending for a day, the defect fired four times, the detector was sitting on disk
returning exit 1 the whole time, and **no surface anywhere would have said so.**
Installing the flock closes this instance; it does nothing about the next
regression, because the thing that would notice still runs on nobody's schedule.

This is the T-2683 shape the guard layer was built to end, recurring in the layer
itself: a check whose existence was mistaken for its enforcement.

**Prevention.** Two parts, deliberately separated:

1. *This instance* — T-2870's pending `[RUBBER-STAMP]` human AC installs the
   serialised lines. **Not stampable by an agent** (CLAUDE.md §Autonomous Mode
   Boundaries; it is a Human AC on another task). Surfaced to the operator instead.
2. *The class* — `check-pickup-cron-lock.sh` needs a runner. Filed as its own task
   rather than bolted on here, because the honest scope is wider than one script:
   it is one of 85 unclassified checks, and at least one sibling
   (`check-cron-install-drift.sh`) is documented as ad-hoc-only for a reason that
   may or may not still hold. Picking a runner for the whole family is a separate
   deliverable, not a footnote to a cleanup.

**What this task does not claim.** The purge is not durable. The cron still runs
unserialised every minute, so the next inbound envelope can duplicate again before
the stamp lands. The register is clean as of this commit and that is all.

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

### 2026-09-08T19:50:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2928-t-2870s-flock-regression-check-passes-wh.md
- **Context:** Initial task creation
