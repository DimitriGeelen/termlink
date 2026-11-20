---
id: T-3837
name: "Worker contract wc/0.3.4: 'withdrawn' outcome (dispatched -> done) in the TermLink
  backend + conformance vectors"
description: >
  From T-3836 (AEF T-5300 consult, ring20 T-2790). Build when AEF publishes shared-worker-contract
  v0.3.4: T3b variant to=done from expected_prior=dispatched with a withdrawn flag
  -> outcome 'withdrawn'; only from actual dispatched, current attempt, matching sgen;
  running -> cas_mismatch; done/lost/unknown -> forbidden (repeat = T8 dedupe); credential
  = the attempt's nonce + supervisor token from dispatch.allocate; evidence_digest
  per v0.3.4 (we proposed sha256 of the reason, result_kind none); closes open answers.
  Vectors: withdraw from dispatched; withdraw vs T2 race (one wins, loser cas_mismatch);
  withdraw from running; repeat deduped. Files: crates/termlink-bus/src/worker_contract.rs,
  crates/termlink-hub/src/dispatch_contract.rs, crates/termlink-hub/tests/wc_conformance.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-bus/src/worker_contract.rs, crates/termlink-bus/src/worker_contract/tests.rs, crates/termlink-hub/src/dispatch_contract.rs, crates/termlink-hub/tests/wc_conformance.rs, crates/termlink-hub/tests/wc_conformance/hub.rs, scripts/run-wc-vectors.sh]
related_tasks: []
# write_set:                      # T-3512: optional — globs (relative to PROJECT_ROOT)
#                                 # naming the files this task intends to write. Declared
#                                 # at CAPTURE, unlike components: which the framework
#                                 # resolves from git history at close. Feeds TWO things:
#                                 #   1. `fw write-set check T-A T-B` — without it the
#                                 #      comparison has nothing to compare and every real
#                                 #      pair exits 2 (undecidable). 0 of 3032 tasks
#                                 #      declared it, so that gate has never had an input.
#                                 #   2. BVP blast_radius before close — the 0.6-weighted
#                                 #      cost term, unavailable for 85% of rankable tasks
#                                 #      because components: only exists once the task is
#                                 #      finished (T-3471).
#                                 # Example: write_set: ["lib/bvp.sh", "tests/unit/t*_bvp*"]
#                                 # An EMPTY list is a real declaration ("writes nothing"),
#                                 # which is not the same as omitting the field. Omitted
#                                 # means unknown, and unknown must never score as cheap.
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
created: 2026-11-20T16:10:03Z
last_update: 2026-11-20T20:21:19Z
date_finished: 2026-11-20T20:21:19Z
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
  - ts: '2026-11-20T17:32:42Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 2
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=2 (body:lightly-promoted)
    rubric_sha: e4a00f38e801
---

# T-3837: Worker contract wc/0.3.4: 'withdrawn' outcome (dispatched -> done) in the TermLink backend + conformance vectors

## Context

**Background / trigger:** shared worker contract v0.3.4 (DRAFT, AEF T-5311) carries three items; the operator ruled on
2026-11-20: item 3 (withdrawn, T3w) A = adopt; item 1 (refusal before rule 1) A = adopt; item 2 (optional dispatch
closure) B = adopt only with TermLink's amendments, after an external review (claude-code + opencode, both "adopt with
amendments").

**Problem / hypothesis:** TermLink's backend (`crates/termlink-bus/src/worker_contract.rs`, hub
`dispatch_contract.rs`) must implement T3w and stay conformant; item 2 as drafted leaves races, authentication,
recovery and conformance to each backend.

**Goal:** build T3w with CC-26..CC-29; put TermLink's item-2 amendment set to AEF and ring20; record the reviews.

**Expected outcome:** our backend passes the v0.3.4 vectors; item 2 is adopted only in its amended form.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Item-2 reviews saved under `docs/reports/T-3837-reviews/`, and TermLink's amendment set (fenced close CAS; closure fences T1 only; declared `closure` capability with split CC-15/CC-19; credentialed close; terminal closure + `retry_of_dispatch_id`; first close wins + history) sent to AEF and ring20 with the operator's adoption of items 1 and 3.
- [x] T3w built in `worker_contract.rs`: dispatched -> done{withdrawn} only from dispatched, current attempt, matching sgen; running -> cas_mismatch; done/lost/unknown -> forbidden_transition; repeat -> T8 dedupe; different-reason withdrawal -> forbidden_transition; late real completion -> done_conflict; closes open answers.
- [x] Conformance vectors CC-26..CC-29 pass on our backend, plus the reader rule (unknown outcome = terminal, not finished).

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
       Pipefail/SIGPIPE section below forbids (T-3370: the 1.8.3 template prescribed it again).
-->

<!--
  Render-touching build tasks (web/templates, web/static, web/blueprints, web/shared.py,
  web/app.py …) add a section headed "Visual Verification" (level-2 heading) after
  Verification, with one checklist table — the check-visual-verification gate reads it
  at commit time (T-4213; T-4202 GO):

    | state | control | 1024 | 1440 | 1920 | note |
    |-------|---------|------|------|------|------|
    | toolbar | Load | ok | ok | ok | |

  Cells: ok / clipped / hidden / overlap / unreachable (non-ok needs a note). Each
  listed state needs screenshots at every UX_SHOT_WIDTHS width:
  fw ux shots <T-ID> <state>=<URL>. This comment is inert: the gate ignores it.
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
# Build/refactor/decommission tasks with NO command here are refused. If nothing can be
# run, add the line `# verification: none — <why nothing can be run>`; it is serviced
# (queued for review: add a check / other evidence / accept). See docs/reports/T-4024-servicing-loop.md.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# ── Mutable-corpus anchor (T-3326) ────────────────────────────────────────────
# Do NOT anchor a verification line (or a unit test it runs) to MUTABLE corpus
# state — an exact live count, or a grep of live `fw audit`/`fw doctor` output
# for a specific corpus entity (a named arc, a task count, a census number).
# The corpus moves under the check, and the line rots: it goes red (or vanishes
# its pattern) for reasons unrelated to the code under test, blocking closes.
# Pin the INVARIANT (categories sum, count > 0, property holds) or run the code
# against a COMMITTED FIXTURE — never the live count or a live-audit line.
# Origin: T-2969 line grepping live audit for one arc's status; T-2871's census
# test pinning exact live counts (56→74 files) — both blocked closes (OBS-377).
#
env -u PROJECT_ROOT TERMLINK_RUNTIME_DIR=/tmp/t3837-rt cargo test -p termlink-bus worker_contract::tests::t3w > /tmp/t3837-bus.out 2>&1 && grep -q "test result: ok. 6 passed" /tmp/t3837-bus.out
env -u PROJECT_ROOT TERMLINK_RUNTIME_DIR=/tmp/t3837-rt cargo test -p termlink-bus worker_contract::tests::reader_rule > /tmp/t3837-rr.out 2>&1 && grep -q "test result: ok. 1 passed" /tmp/t3837-rr.out
env -u PROJECT_ROOT TERMLINK_RUNTIME_DIR=/tmp/t3837-rt cargo test -p termlink-hub dispatch_transition_withdraw_over_rpc > /tmp/t3837-hub.out 2>&1 && grep -q "test result: ok. 1 passed" /tmp/t3837-hub.out
bash scripts/run-wc-vectors.sh --ref 4b1d2304e > /tmp/t3837-wc.out 2>&1 && grep -q "fail=0" /tmp/t3837-wc.out
# ── Pipefail/SIGPIPE: grepping a command's output (L-387, T-2090, T-2743, T-2738) ──
#
# THE DEFAULT — redirect to a file, then grep the file:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
#     curl -sf "$(bin/fw watchtower url)/page" -o /tmp/.out && grep -q "PAT" /tmp/.out
# Correct at any output size, and `&&` keeps the PRODUCING command's exit code in
# the verdict. Reach for this first; the alternative below is the special case.
#
# NEVER `cmd | grep -q PAT` (L-387) — why: P-011 runs each line with PIPEFAIL LIVE
# (errexit is not — see below). When grep matches it exits and closes stdin while cmd is still
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
# ── Asserting an ABSENCE: prove the search could have succeeded (T-3144) ──
# (Restored after the AEF 1.8.3 re-vendor dropped it, T-3370.)
#
# `! grep -q "PATTERN" file` exits 0 when the pattern is absent. It ALSO exits 0
# when the file was renamed, deleted, or is empty — so the leg cannot distinguish
# "the bad thing is not there" from "I could not look", and the gate reports green
# over a check that never ran. Pair every absence assertion with something that
# fails if the search could not happen:
#
#     test -f path/to/file && ! grep -q "PATTERN" path/to/file    # existence first
#     grep -q "KNOWN_MARKER" f && ! grep -q "PATTERN" f           # positive companion
#     cmd > /tmp/.out 2>&1 && ! grep -q "PATTERN" /tmp/.out       # &&-joined producer
#
# Count-equals-zero is the same defect wearing a different hat, and it is the one
# that bites hardest over a COMMAND's output rather than a file:
#
#     [ "$(cargo clippy --workspace 2>&1 | grep -c "^error")" = "0" ]   # WRONG
#
# If cargo is missing, or dies before emitting diagnostics, there are no `^error`
# lines, the count is 0, and the leg passes — a build gate that goes green
# precisely when the build could not run. Keep the producer's exit code in the verdict:
#
#     cargo clippy --workspace > /tmp/.out 2>&1 && ! grep -q "^error" /tmp/.out
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
# ── A SKIPPED BATS TEST REPORTS `ok` (T-3217) ─────────────────────────────────
#
# `! grep -q "^not ok"` does NOT mean the suite ran. Bats emits a skip as
#     ok 6 <name> # skip <reason>
# which is not a `not ok`, so the gate passes and the report says ok while the
# thing the test covers was measured NOWHERE. Origin: T-3213 guarded a test with
# `[ "$(id -u)" -eq 0 ] && skip` — the suite runs as root here and in CI, so it
# skipped on every run that mattered, for as long as it existed.
#
# Add a skip clause to any bats verification line. `# skip` is the marker bats
# writes; counting it is the whole check:
#     timeout 300 bats <file> > /tmp/.out 2>&1 && ! grep -q "^not ok" /tmp/.out
#     test "$(grep -c '# skip' /tmp/.out)" -eq 0
# Two lines, because they answer different questions — "did anything fail" and
# "did everything run". If some skips are legitimate on your host (an optional
# dependency is genuinely absent), assert the COUNT you expect rather than zero,
# and say in the task why that number is right.
#
# Corpus-wide, the same check runs from `bin/fw test lint`
# (tools/bats-silent-skip-lint.py): static mode flags guards that are fixed for
# a deployment rather than probing an optional dependency, and `--tap FILE`
# reports the skips a real run actually fired.
#
# REHEARSING A LINE BY HAND DOES NOT REHEARSE THE GATE (T-2743). Your interactive
# shell has no pipefail. A line has returned 0 by hand and 141 under P-011, from
# the same directory, the same second. To rehearse for real:
#     bash -c 'set -o pipefail; <your verification line>'
#
# NOTE THE MISSING `-e` — it is not a typo (T-3203). This file used to prescribe
# `set -eo pipefail` here, which is NOT the gate: it adds errexit the gate does
# not have, so it FAILS lines the gate PASSES. Measured, 10 lines, 3 diverged:
#     line                            gate    set -eo (old)   set -o (this)
#     false; true                     PASS    FAIL  wrong     PASS  ok
#     cd /nonexistent; echo ok        PASS    FAIL  wrong     PASS  ok
#     grep -q MISS file; true         PASS    FAIL  wrong     PASS  ok
# The divergence is one-directional and that is the trap: the old rehearsal only
# ever fails lines the gate accepts, so it produces false REDS, and an author
# who "fixes" a line to satisfy it is fixing something that was never broken —
# while the line that actually is broken (`cmd1; cmd2` where cmd1 fails) passes
# both. Re-derive rather than trust this table — it is pinned, not asserted:
#     bats tests/unit/t3203_p011_gate_semantics.bats
#
# ── `cmd1; cmd2` IS JUDGED ONLY ON cmd2 (T-3203) ──────────────────────────────
#
# The gate runs each line as the CONDITION of an `if` (update-task.sh:1215), and
# POSIX suppresses errexit for a compound command in an `if` condition — through
# the subshell. So pipefail applies and `set -e` does not, and in a sequence only
# the LAST command's status reaches the verdict. `cd /nonexistent; echo ok` passes.
# 2,644 of 10,997 verification lines in this corpus contain `;` (re-derive with
# the query in docs/reports/T-3203-p011-gate-semantics.md).
#
# SAFE SHAPES — both verified biting, each against a passing control:
#   A. one command whose own status is the verdict (prefer this):
#        out=$(cmd 2>&1); echo "$out" | grep -q PAT && ! echo "$out" | grep -q BAD
#      the leading assignments are setup; the trailing `&&` chain is the verdict.
#   B. an explicit sub-shell, whose errexit the outer `if` cannot reach into:
#        bash -c 'set -eo pipefail; cmd1; cmd2'
#      use when you genuinely need every command in the sequence to count.
#
# The rule of thumb: put the assertion LAST, and make sure it is an assertion.
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

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

### 2026-11-20 — Adopt shared worker contract v0.3.4 item 3 ("withdrawn", T3w)? (operator ruling: "A")
- **Chose:** A — TermLink adopts item 3 as drafted by AEF (T-5311, commit 90202e700 on AEF bleeding-edge): T3w =
  dispatched -> done{withdrawn}, current attempt, matching sgen; running -> cas_mismatch; done/lost/unknown ->
  forbidden_transition; same withdrawal repeated = T8 dedupe; different-reason withdrawal = forbidden_transition; late
  real completion = done_conflict; only the holder of the nonce + supervisor_token from dispatch.allocate (Execute
  scope); hub sweep never withdraws; evidence_digest = sha256(reason), result_kind none; closes open questions; T2 rule:
  launch only after T2 accepted (a launch failure after T2 ends through T3 as failed); vectors CC-26..CC-29; reader rule:
  an unknown outcome is terminal, not finished work.
- **Why:** our own proposal (T-3836), checked against `worker_contract.rs`; the race is settled by the existing CAS;
  ring20 needs it this week. Brief scores: A +26, B +16, D -5, C -17.
- **Rejected:** B (wait for items 1 and 2: delays a self-contained change), C (no withdrawal), D (defer).
- **Left open:** v0.3.4 items 1 and 2 are separate decisions, not yet read.

### 2026-11-20 — Adopt v0.3.4 item 1 (refusal before rule 1)? (operator ruling: "A")
- **Chose:** A — adopt. Rule 1 (hand-back missing/malformed -> handback-malformed) applies only to a worker that ran;
  a run.sh or supervisor refusal yields refused via rule 2 even with no hand-back; rule numbers unchanged.
- **Why:** it states what our backend already does (`worker_contract.rs` derive_outcome: the refused flag outranks the
  hand-back check) and our tests pin (CC-16, CC-21, CC-25, `worker_contract/tests.rs:344`); no build needed.
  Brief scores: A +22, B +10, D 0, C -15.
- **Rejected:** B (tie to item 2), C (spec stays ambiguous), D (defer).
- **Left open:** item 2 (optional dispatch closure), next.

### 2026-11-20 — Adopt v0.3.4 item 2 (optional dispatch closure)? (operator ruling: "B")
- **Chose:** B — adopt only with TermLink's amendments, as our condition: (1) close is one fenced transaction with CAS
  on (last attempt, last state ∈ {done, lost}); commit order decides close vs T1; (2) closure fences T1 only, every
  attempt-level transition stays legal, finishedness computed from attempts; (3) a declared `closure: never|supported`
  capability, `dispatch_closed` only with a readable closure record, CC-15/CC-19 split per arrangement (no "accept
  either"), clients treat an unknown fence reason as a non-retryable refusal; (4) close authenticated by the last
  attempt's supervisor credential or a declared backend policy, actor derived from the credential; (5) closure terminal,
  recovery = new dispatch with `retry_of_dispatch_id`, readers expose closed / finished / last outcome separately;
  (6) first close wins, repeats return the stored record, every close and `dispatch_closed` refusal writes history.
  Proposed vectors CC-30..CC-43 (reviewers' numbering).
- **Why:** an external review (claude-code and opencode, independently) found the same six weaknesses, five high:
  close vs T1 race, late lost->done after close, unauthenticated actor, no recovery from a wrong close, "accept either"
  disarming CC-15/CC-19, repeat/audit. All amendments are additive; our hub's cost is a typed refusal of close and
  tolerance of the new reason. Reviews: `docs/reports/T-3837-reviews/`.
- **Rejected:** A (adopt as drafted: leaves every hard case to each backend), C (minimum 1-3 only), D (not adopt).

### 2026-11-20 — T3w build choices (agent, within the operator's ruling A on item 3)
- **Chose:**
  1. The store implements T3w as a `withdraw: Option<Withdrawal{actor, reason}>` on `TransitionRequest` with
     `to: done`. The digest is computed as sha256(reason); actor and reason go in two new `wc_history` columns,
     shown on that row only.
  2. The RPC (`dispatch.transition`) refuses `withdraw` while `WITHDRAW_LIVE = false`, because the spec says
     no backend emits `withdrawn` before all three owners adopt v0.3.4. Flip it once AEF's operator adopts.
  3. The reader rule is `finished_work()`: done, not attested, and outcome in {succeeded, failed, refused,
     handback-malformed}. Anything else, including an outcome this reader does not know, is `not_finished_work`.
     The adapter's `consume` op calls it, so CC-19 and CC-22 now pass too.
  4. A start (T2) on `lost` is `forbidden_transition` whatever prior it expected (CC-20). Otherwise a CAS miss is
     `cas_mismatch` (CC-27). The adapter's start now always expects `dispatched`.
  5. `run-wc-vectors.sh` keeps v0.3.3 (18c6e0042) as its default until adoption; v0.3.4 runs with `--ref ea8379a56`.
- **Why:** each choice follows the spec text at ea8379a56 (§3.3 T3w, §3.2 submitter, the v0.3.4 status line).
- **Rejected:**
  1. Serving withdraw over the RPC now. That breaks the spec's "no backend emits withdrawn before adoption".
  2. Allowing a worker question while `dispatched` to fit CC-26 answer-channel (see the open finding below).
- **Open finding (sent to AEF):** CC-26's answer-channel variant asks a question on a `dispatched` attempt. Our
  store refuses it (`answer.rs:289`: only a running or unknown worker asks). The spec's own T2 rule says no agent
  child runs before T2, so no worker can have asked. **Resolved:** AEF agreed and fixed it (their T-5319, commit
  4b1d2304e): one un-acked dispatcher answer with no question_id, deadline clock+600, expected state `undelivered`.
  Result at 4b1d2304e: 29 cases, pass 24, fail 0, not-implemented 4, n/a 1; CC-26 passes every variant.
  Earlier result at ea8379a56: pass 23, fail 1 (that variant only),
  not-implemented 4, n/a 1.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-11-20T16:10:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3837-worker-contract-wc034-withdrawn-outcome-.md
- **Context:** Initial task creation

### 2026-11-20T16:45:33Z — status-update [task-update-agent]
- **Change:** horizon: later → now

### 2026-11-20T17:32:40Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-11-20T20:21:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

## Reviewer Verdict (v1.5)

- **Scan ID:** R-095a5734
- **Timestamp:** 2026-11-20T20:22:03Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Scope:** static anti-pattern scan; PASS means no anti-pattern was found, not that the criteria hold
- **Needs Human:** no
- **Findings:** none
