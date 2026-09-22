---
id: T-3030
name: "P-002 read-only allowlist gap blocks pure-read guard scripts and command substitution"
description: >
  The P-002 check-active-task gate blocks commands it cannot prove are reads, and
  the read-only allowlist in agents/context/lib/safe-commands.sh does not cover several
  shapes that are unambiguously read-only. Measured five refusals across three sessions
  (2026-09-19, 2026-09-20 x2 runs): (1) python3 - <<PY heredoc doing a pure parse;
  (2) f=$(ls ...) command substitution; (3) a read-only command containing >/dev/null,
  classified as a file write by the redirect pattern; (4) bash scripts/check-stranded-finalized-tasks.sh
  --quiet and bash scripts/check-task-id-collisions.sh --quiet, both documented deploy-time
  READ-ONLY guard checks that detect and never repair; (5) the same guard-script shape
  again while focus sat on a captured task. The gate's own block message names the
  gap each time: 'this command writes nothing the gate can detect ... If it genuinely
  only reads, that is a gap in the allowlist worth filing.' Cost is not the refusal
  itself but where it lands: every occurrence is during wrap-up, immediately after
  a task closes and clears focus, which is exactly when the verification and guard
  reads are required before handover. The same deadlock class as T-2878/T-2052 and
  the one T-2961 fixed for checkpoint.sh, arriving for the guard-layer scripts. Proposed
  fix mirrors T-2961's Category 4c verb-scoped arm: allowlist bash scripts/check-*.sh
  when the argv contains only --quiet/--json/--no-heartbeat style read flags, since
  the entire scripts/check-* family is documented as detect-never-repair; and stop
  classifying >/dev/null as a write. Vendored (G-062) so the fix is upstream or a
  registered local divergence, not a silent patch.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [.agentic-framework/agents/context/lib/safe-commands.sh]
related_tasks: [T-2961, T-2878, T-2052, T-3018]
arc_id: arc-008
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
# demo_target: true               # T-2286: optional — marks task as reserved for an orchestrated demo
#                                 # worker (e.g. arc-010 HM-A dispatches via mcp__fw__work_on). When set,
#                                 # `fw work-on T-XXX` refuses unless --i-am-demo-orchestrator (CLI) or
#                                 # FW_I_AM_DEMO_ORCHESTRATOR=1 (env) is passed. Prevents the parent
#                                 # session from consuming the captured→started-work transition the demo
#                                 # worker expects to drive. Origin OBS-057.
created: 2026-09-20T15:19:15Z
last_update: 2026-09-20T17:05:57Z
date_finished: 2026-09-20T17:05:57Z
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
bvp_scores_proposed:
  - ts: '2026-09-20T15:20:28Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 3
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-20T15:21:17Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 3
      D4: 2
      F-RECALL: 0
      F-ORCH: 4
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=0 (no-signal); F-ORCH=4 (body:rubric-routable)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-20T15:21:17Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (single-component); tier=2 (workflow:build); 
      effort=8 (lines=240,acs=7)
    rubric_sha: e4a00f38e801
---

# T-3030: P-002 read-only allowlist gap blocks pure-read guard scripts and command substitution

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The refusals are reproduced, not recalled. All five shapes were re-run through the real
      classifier (`is_bash_safe_command`) and the real write-predicate (`has_bash_write_pattern`)
      sourced from the live lib, not reasoned about from the regex. Verdicts, measured:
      `bash scripts/check-task-id-collisions.sh --quiet` GATED/no-write;
      `bash scripts/check-stranded-finalized-tasks.sh --quiet` GATED/no-write;
      `bash scripts/check-verification-misfile.sh --json` GATED/no-write;
      `python3 - <<PY` GATED; `f=$(ls .tasks/active | head -1)` GATED;
      `git status --porcelain >/dev/null` write-pattern MATCH.
      The gate's own line is at `check-active-task.sh:364-367`, emitted on exactly this branch:
      "this command writes nothing the gate can detect ... If it genuinely only reads, that is a
      gap in the allowlist worth filing." Original occurrences: 2026-09-19 (shapes 1-3, three
      refusals in one session, also the origin of T-2961); 2026-09-20 run 1 (shape 4,
      `check-stranded-finalized-tasks.sh --quiet`); 2026-09-20 run 2 (shape 4 again,
      `check-task-id-collisions.sh --quiet`, and shape 5 with focus on a `captured` task).
- [x] The read-only claim is established, not assumed. The family is 66 `scripts/check-*.sh`, of
      which **38 carry the `# guard-layer: source` marker** — which CLAUDE.md defines as "safe to
      run anywhere: no live hub, no network, no host state". Headers citing detect-never-repair
      explicitly: `check-pickup-deferred-freshness.sh:40` ("This DETECTS; it never drains"),
      `check-task-id-collisions.sh:60` and `check-pickup-cron-lock.sh:26` ("DEPLOY-TIME / ad-hoc
      check, NOT a cron canary"). Measured: six members run (`check-task-id-collisions`,
      `check-stranded-finalized-tasks`, `check-verification-misfile`, `check-canary-log-hygiene`,
      `check-silent-exit`, `check-version-derivation`), all rc=0, and
      `git status --porcelain | sort | md5sum` identical before and after
      (`5a54d8c25df77549887305c594b4636d` both sides). Tree byte-identical.
- [x] The `>/dev/null` misclassification is isolated, and it is narrower than the filing said.
      `has_bash_write_pattern` (`safe-commands.sh:771`) scores a write on `[^2>&]>[^>&]|>>`.
      Measured: `>/dev/null` and `> /dev/null 2>&1` classify WRITE; `2>/dev/null` does NOT — so the
      defect is specifically the **stdout** discard-redirect, not discard-redirects generally.
      The command refused *solely* by it is `git status --porcelain >/dev/null`, which
      `is_bash_safe_command` independently returns SAFE for: the two predicates disagree, and
      `check-active-task.sh:220` tests the write-pattern FIRST with the safe-list reachable only
      via `elif` at :223, so the write-pattern wins and the command is gated in production.
- [x] The structural ordering conflict is recorded, and the answer is the opposite of the one the
      filing implied. Mechanism: `check-active-task.sh:731-743` blocks outright when focus is on a
      `captured` task ("BLOCKED: Task X has status 'captured' (work not started)") and prints
      `fw work-on X` — i.e. *start it* — as the remedy. Independently, both scoring verbs are off
      the allowlist: `fw bvp estimate T-XXX` GATED and the `estimator.py cost-one` invocation
      GATED, while the pure read `fw bvp --quadrant hv-lc --include-proposed` is SAFE. So scoring a
      captured task is refused from both directions, and the gate's own prescribed remedy is the
      rule violation.
      **The remedy belongs in the RULE, not the allowlist.** The estimator genuinely writes — it
      persists `bvp_scores_proposed:` into the task's frontmatter, visible in this task's own
      frontmatter at ts `2026-09-20T15:20:28Z`. Allowlisting a real write to silence an ordering
      rule would weaken the gate to fit the process. That the bvp *read* is already SAFE shows the
      allowlist has drawn the read/write line correctly here. Either "scored before started"
      accepts that scoring is itself a state change, or the estimator needs a non-persisting
      `--dry-run`. Which of those is a Sovereign decision, surfaced below and not taken here.
- [x] Disposition recorded per G-062, and it splits by provenance rather than being one verdict.
      No local patch was made to `.agentic-framework/` — verified byte-clean in `## Verification`.
      **(a) The guard-script arm is NOT upstream's defect.** `scripts/check-*.sh` and the
      `# guard-layer: source` marker are this project's own convention (T-2684); upstream AEF has
      no such family, so it cannot be expected to allowlist one. This half is a consumer-local
      extension, and its route is a registered `.vendor-divergence.yaml` divergence exactly as
      **T-2961** did for `checkpoint.sh` on this same file (`status: local-only`, with fixtures
      that redden the guard layer if a re-vendor deletes the arm). Filing it upstream would be
      asking upstream to encode a downstream naming convention.
      **(b) The `>` over-classification is already known upstream and deliberately declined.**
      `check-active-task.sh:265-283` — vendored text, therefore upstream's own — documents this
      exact behaviour ("a `>` anywhere on the line ... voids it", with worked examples) and
      concludes "The guard is NOT the bug and is not relaxed here ... Widening it would admit
      `fw work-on X > .claude/settings.json`." Upstream considered it and chose to emit a hint
      instead. Re-filing a deliberated upstream decision as a defect is precisely the error
      **T-3018** made eleven days after T-2950 had closed the same finding.
      **(c) Engaging the T-2950/T-2961 precedent as required.** That precedent says a report
      against a three-month-stale vendored build (1.6.29 @ 2026-06-08 vs upstream v1.6.295) is
      phantom register debt. It applies here with *additional* force, because unlike the
      checkpoint.sh case there is no reason to think upstream shares the gap at all: (a) is ours
      by construction and (b) is upstream's settled decision. Nothing is filed this run. All of it
      re-checks at the already-surfaced sovereign re-vendor decision (T-2950).

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

# ── T-3030 checks (16 lines, all rehearsed under `set -eo pipefail`; lines 1 and 5
# mutant-tested: widening the bash arm reddens line 1, neutering the redirect regex
# reddens line 5 — neither is vacuous). ──
bash -c 'set -eo pipefail; . .agentic-framework/agents/context/lib/safe-commands.sh; ! is_bash_safe_command "bash scripts/check-task-id-collisions.sh --quiet"'
bash -c 'set -eo pipefail; . .agentic-framework/agents/context/lib/safe-commands.sh; ! is_bash_safe_command "bash scripts/check-stranded-finalized-tasks.sh --quiet"'
bash -c 'set -eo pipefail; . .agentic-framework/agents/context/lib/safe-commands.sh; ! has_bash_write_pattern "bash scripts/check-task-id-collisions.sh --quiet"'
bash -c 'set -eo pipefail; . .agentic-framework/agents/context/lib/safe-commands.sh; is_bash_safe_command "bash -n scripts/check-task-id-collisions.sh"'
bash -c 'set -eo pipefail; . .agentic-framework/agents/context/lib/safe-commands.sh; has_bash_write_pattern "git status --porcelain >/dev/null"'
bash -c 'set -eo pipefail; . .agentic-framework/agents/context/lib/safe-commands.sh; ! has_bash_write_pattern "git status --porcelain 2>/dev/null"'
bash -c 'set -eo pipefail; . .agentic-framework/agents/context/lib/safe-commands.sh; ! is_bash_safe_command "python3 - <<PY"'
bash -c 'set -eo pipefail; . .agentic-framework/agents/context/lib/safe-commands.sh; is_bash_safe_command "python3 -c import yaml"'
bash -c 'set -eo pipefail; . .agentic-framework/agents/context/lib/safe-commands.sh; ! is_bash_safe_command ".agentic-framework/bin/fw bvp estimate T-3017"'
bash -c 'set -eo pipefail; . .agentic-framework/agents/context/lib/safe-commands.sh; is_bash_safe_command ".agentic-framework/bin/fw bvp --quadrant hv-lc --include-proposed"'
test "$(grep -c 'Check write patterns FIRST' .agentic-framework/agents/context/check-active-task.sh)" = "1"
grep -q 'gap in the allowlist worth filing' .agentic-framework/agents/context/check-active-task.sh
grep -q 'The guard is NOT the bug and is not relaxed here' .agentic-framework/agents/context/check-active-task.sh
test "$(ls scripts/check-*.sh | wc -l)" -ge "60"
test "$(grep -l '^# guard-layer: source' scripts/check-*.sh | wc -l)" -ge "35"
test -z "$(git status --porcelain .agentic-framework/agents/context/lib/safe-commands.sh)"

## RCA

The gate has two predicates and they answer different questions. `has_bash_write_pattern` asks
"does this line contain a write-shaped token?"; `is_bash_safe_command` asks "is this command on the
read-only allowlist?". `check-active-task.sh:220-223` consults them in that order, write-pattern
first, allowlist only via `elif`. Every one of the five refusals is one of those two saying no:

- Shapes 1, 2, 4, 5 are **allowlist omissions**. The `bash|sh` arm admits exactly one form,
  `bash -n` (`safe-commands.sh:707-712`); the `python3` arm admits exactly `python3 -c`
  (`:697-706`). Everything else in those families falls through and returns "not safe". No write
  was ever detected in any of them — the classifier itself reports `no-write` for all four.
- Shape 3 is a **write-predicate false positive**, and the only one of the five where the two
  predicates actively disagree.

The filing conflated these, attributing the guard-script refusals to "the redirect pattern". They
are not: `bash scripts/check-task-id-collisions.sh --quiet` contains no redirect and is scored
`no-write`. It is gated because no arm claims it. That distinction decides the fix — a redirect
fix would not have unblocked the guard scripts at all.

Why the framework was blind (G-019): the allowlist is a closed vocabulary of *command spellings*,
while the property it is trying to approximate — "this writes nothing" — is a property of
*behaviour*. Thirty-eight of this project's guard scripts already declare that behaviour in a
machine-readable header (`# guard-layer: source`, defined as no live hub, no network, no host
state). The gate cannot see it, because nothing connects a consumer project's declaration to the
framework's vocabulary. The scripts are not merely *probably* safe; they are *annotated* safe, and
the annotation is unread.

## Evolution

Two of this task's five criteria disproved something the task asserted about itself, which is now
the third consecutive arc-008 task where that has happened (T-3025 AC2, T-3018 AC2, T-3030 AC1/AC5).
The shared mechanism is worth naming: each of those criteria was written to force a *measurement*
before a claim was believed, and in every case the measurement contradicted the filing. The filings
were not careless — they were written from a real refusal, observed live. What they got wrong was
the *cause*, inferred from the symptom without reading the code path.

The specific lesson here is narrower and more useful than "measure first". It is that a gate with
two predicates and an `elif` between them has **two** failure modes that present identically to the
user — a false positive in the first predicate and an omission in the second both render as one
block message — and the message cannot distinguish them, because it is emitted after the branch is
already taken. Five refusals across three sessions were read as one defect for that reason. Anyone
diagnosing a gate refusal should establish *which predicate* refused before proposing a fix; here,
four of five refusals would have been untouched by the fix the filing proposed.

Second, and recorded against the G-062 discipline rather than this defect: "file it upstream" is not
automatically the conservative choice. Two of this run's three disposition branches resolve to
*do not file* — one because the convention is ours and upstream cannot be expected to know it, one
because upstream has already considered and declined it in a comment shipped in the vendored file.
Filing either would have added register debt while looking like diligence. The check that caught it
was reading the vendored code's own comments before writing the report, which is the same discipline
T-3018 arrived at from the other direction (grep `completed/` before filing).

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

### 2026-09-20T15:19:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3030-p-002-read-only-allowlist-gap-blocks-pur.md
- **Context:** Initial task creation

### 2026-09-20T15:20:28Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Correction

Two claims in this task's own filing were disproved by executing its acceptance criteria, and are
corrected rather than silently dropped (the convention used on T-3025's AC2 and T-3018's AC4):

1. The filing said the guard-script refusals came from the redirect pattern. Measured: they come
   from a missing `bash|sh` arm. The redirect pattern scores them `no-write`.
2. The filing proposed "allowlist `bash scripts/check-*.sh` when the argv contains only read
   flags". Measured, there is a better key already in the tree: the `# guard-layer: source` marker,
   carried by 38 of the 66 members and already *defined* as run-anywhere-safe. A name-prefix rule
   admits any file someone names `check-*.sh`; the marker is an assertion the script makes about
   itself and that the guard-layer runner already relies on. Recorded for whoever implements the
   fix — which is not this task.

## Scope note — why no fix was written

Not one of the five acceptance criteria asks for a fix. T-3030 is a diagnosis-and-disposition task,
and under the governing rule that an activity not required by an acceptance criterion is not part of
the task, implementing the allowlist arm here would be scope the task does not carry — and would
open a second structural change while this one is ungated. The implementation is a separate task
against `.vendor-divergence.yaml`, carrying the T-2961 shape: local arm + registered divergence +
a fixtures suite whose name matches the guard-layer runner's `*fixtures*.sh` membership convention,
so that a re-vendor deleting the arm reddens the guard layer by construction.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-0c4232ea
- **Timestamp:** 2026-09-20T17:05:59Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-20T17:05:57Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
