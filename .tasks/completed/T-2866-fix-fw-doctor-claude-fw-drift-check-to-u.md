---
id: T-2866
name: "Fix fw doctor claude-fw drift check to understand the T-2854 router; register the vendored divergence"
description: >
  Fix fw doctor claude-fw drift check to understand the T-2854 router; register the vendored divergence

status: work-completed
workflow_type: build
owner: agent
horizon: null
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
created: 2026-08-31T11:22:04Z
last_update: 2026-08-31T11:34:08Z
date_finished: 2026-08-31T11:34:08Z
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

# T-2866: Fix fw doctor claude-fw drift check to understand the T-2854 router; register the vendored divergence

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

The fix half of T-2861, which diagnosed why the budget-critical continuous-run loop
had never fired here. T-2861 filed the defect upstream and stopped there, on G-062
grounds. This task takes the other option the project actually supports: patch the
vendored copy AND register the divergence, so the fix is live now and survives — or is
knowingly retired — at the next re-vendor.

The defect: `fw doctor`'s T-2501 claude-fw drift check (`bin/fw:2369-2385`) byte-compares
`command -v claude-fw` against `$FRAMEWORK_ROOT/bin/claude-fw`. Since T-2854 those are
the **router** (3707 b) and the **wrapper** (14971 b) — different files by design — so it
reports drift permanently on a correct install, prints directly beneath the T-2499
"Unsupervised session" WARN where it reads as that WARN's cause, and offers a remediation
(`rm -f <router> && cp <wrapper> <router-path>`) that deletes the router and reinstates the
pinned copy T-2854 existed to remove. Sibling-not-migrated.

Why it was worth fixing rather than only filing: the guard misattributes a cause at
exactly the moment an operator is debugging a dead loop, and its advice makes the system
less recoverable while leaving the loop just as unarmed. A guard that costs attention and
then damages the install is worse than no guard.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `bin/fw`'s claude-fw check recognises the T-2854 router shape and, for a router, asserts the RESOLVED wrapper carries `export FW_CLAUDE_FW_SUPERVISED=1` instead of byte-comparing router against wrapper
- [x] The harmful `rm -f <router> && cp <wrapper> <router-path>` remediation is unreachable when the on-PATH file is the router (it stays for the genuine stale-copy case)
- [x] A missing supervision export in the resolved wrapper still WARNs — the fix narrows a false positive without creating a false negative
- [x] `fw doctor` on this host reports OK for the claude-fw check, with the "Unsupervised session" WARN still firing independently (proving the two checks were never the same signal)
- [x] The local edit is registered in `.vendor-divergence.yaml` as `filed-upstream`, so the next re-vendor does not silently delete it
- [x] `bash scripts/check-vendor-divergence.sh` passes with the new entry classified
- [x] The fix reaches the framework carrying the diff, the rationale and the evidence *including its limits* — not just a pointer
  <!-- AC amended mid-task, deliberately, rather than ticked as written. It originally
       said "handed to the live arc-012 framework sessions". Five were live and on this
       exact arc (tl-arc012-w1-loop-core .. w5-cli-surface); all five ended within ~13
       minutes, between the listing and the send, and both sends failed as unreachable.
       A short-lived worker cannot be a delivery target. Routed to `framework:pickup`
       offset 79 instead, threaded onto the original bug-report at offset 75, with the
       failed direct hand-off recorded in the payload's delivery_note so the recipient
       knows it was attempted. The AC's intent is met; its literal wording was not
       achievable, and ticking it as written would have been false. -->
- [x] The delivery failure itself is recorded rather than silently retried away — a durable rail was used because the direct one proved unusable, and the payload says so

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

# --- T-2866 ---
# The patched file is still valid bash.
bash -n .agentic-framework/bin/fw
# The router branch exists and asserts the resolved wrapper's supervision export.
grep -q "claude-fw-router" .agentic-framework/bin/fw
grep -q "is the T-2854 router; resolved wrapper exports supervision" .agentic-framework/bin/fw
# The byte-compare and its rm/cp remediation survive for the genuine stale-copy case.
grep -q "Installed claude-fw matches repo source" .agentic-framework/bin/fw
grep -q 'Refresh: rm -f' .agentic-framework/bin/fw
# Live: doctor now reports OK for the claude-fw check on this host. `|| true`
# because doctor exits 2 on unrelated warnings; the grep is the verdict.
timeout 200 .agentic-framework/bin/fw doctor > /tmp/.t2866-doctor.out 2>&1 || true; grep -q "is the T-2854 router" /tmp/.t2866-doctor.out
# And the independent supervision WARN still fires — the two were never one signal.
grep -q "Unsupervised session" /tmp/.t2866-doctor.out
# The divergence is registered and the register still parses + passes its checker.
grep -q "task: T-2866" .vendor-divergence.yaml
python3 -c "import yaml,sys; d=yaml.safe_load(open('.vendor-divergence.yaml')); sys.exit(0 if any(e.get('task')=='T-2866' for e in d['divergences']) else 1)"
bash scripts/check-vendor-divergence.sh > /tmp/.t2866-vd.out 2>&1 && grep -q "all registered" /tmp/.t2866-vd.out

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

**Symptom:** `fw doctor` reported `WARN  Installed claude-fw drifted from repo source —
supervision export may be stale` on a correctly-installed host, permanently, and printed
it immediately beneath the T-2499 `Unsupervised session — budget auto-restart will NOT
fire` WARN — where it read as that WARN's cause. Its remediation
(`rm -f <router> && cp <wrapper> <router-path>`) would have deleted the T-2854 router.

**Root cause:** T-2854 changed *what lands on PATH* — from a fixed copy of the wrapper to
a router that resolves the current project and execs that project's own wrapper. The
T-2501 drift check's predicate is `cmp -s "$(command -v claude-fw)"
"$FRAMEWORK_ROOT/bin/claude-fw"`, which encodes the pre-T-2854 assumption that the
on-PATH file IS the wrapper. That assumption became false and was never re-examined. Not
"the code was wrong" — the code was right for the world it was written in, and the world
moved underneath it.

**Why structurally allowed:** three compounding gaps.

1. **The check has no fixture, and cannot have one.** `_cfw_src` is computed from
   `FRAMEWORK_ROOT` inside `fw doctor` with no injection seam, so neither branch is
   reachable from a test. A guard nothing exercises cannot notice that the world it
   describes has changed — the T-2683 "static checks nothing ran" class, one layer down.
2. **Nothing links a change in what ships to the assertions about what ships.** T-2854
   altered the on-PATH artefact; no dependency, card, or check pointed from it to the
   predicates that assert on-PATH shape. The sibling-not-migrated pattern this repo
   catches in source (T-2666, T-2672, T-2747) has no equivalent for *install-shape*
   assumptions.
3. **A WARN-only false positive costs nothing mechanically, so it accrued.** Nothing
   fails, nothing blocks; the line simply appears on every run until it is background
   noise. That is the same attention-erosion this project documented from the other
   direction in T-2818 (150 wrong P-011 blocks teaching operators to `--force`) and
   T-2833 (58 spurious findings teaching operators to stop reading). Here the erosion is
   worse than neutral: the noise carried destructive advice.

**Prevention:**

- *Shipped, and load-bearing:* the predicate is now shape-aware — a router is validated by
  asserting the wrapper it RESOLVES to still exports `FW_CLAUDE_FW_SUPERVISED`; only a
  non-router on-PATH file reaches the byte-compare and the `rm`/`cp` remediation. Mutation
  A (a non-router stale copy first on PATH) still WARNs against the real `fw doctor`, so
  the false positive was narrowed without creating a false negative.
- *Shipped:* registered in `.vendor-divergence.yaml` as `filed-upstream`, so the next
  re-vendor cannot silently delete the fix — the T-2812 mechanism, used as intended.
- *Filed, NOT built — stated plainly rather than counted as prevention:* the real
  structural fix is gap 1, an injectable `_cfw_src` / redirectable `FRAMEWORK_ROOT` seam
  so both branches become genuine fixtures. That seam is in vendored code, so it is filed
  upstream at `framework:pickup` offset 79 as an explicit testability gap. Until someone
  builds it, the branch that WARNs when supervision is genuinely broken — the half that
  must fire when the loop is actually dead — remains proven only at predicate level. That
  is the weakest point of this task and it is not closed.

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

### 2026-08-31T11:22:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2866-fix-fw-doctor-claude-fw-drift-check-to-u.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-39bf7653
- **Timestamp:** 2026-08-31T11:34:57Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** yes
- **Findings:** none

- **Layer-1 escalations:** 1
  1. **destructive-action** (high) — Destructive operation in verification or AC
     - matched: `rm -f`

### 2026-08-31T11:34:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
