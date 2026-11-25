---
id: T-3859
name: "Pin ring20's pool submit script and pool client by sha256 and run pool jobs
  as a non-root user (T-3858 risk 1)"
description: >
  Mitigation for T-3858 risk matrix risk 1 (third-party code unattended as root with
  our hub identity, provider-host 10) and risk 3 (pool pass trusted, 6): refuse to
  run when either file's sha256 differs from the pinned one; run as a non-root user;
  pool results stay advisory.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:risk]
components: [runme.sh]
related_tasks: [T-3858]
arc_id: risk
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
created: 2026-11-24T12:31:17Z
last_update: 2026-11-25T10:08:52Z
date_finished: 2026-11-25T10:08:52Z
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
  - ts: '2026-11-24T13:45:17Z'
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

# T-3859: Pin ring20's pool submit script and pool client by sha256 and run pool jobs as a non-root user (T-3858 risk 1)

## Context

**Background / trigger:** T-3858 risk matrix (`docs/reports/T-3858-risk-matrix.md`): sending our jobs to the ring20
pool means running ring20's code (`010-pool-submit.py` and `/usr/local/lib/ring20-pool-client/pool-client.py`)
unattended on .107 as root under our hub identity. Risk 1 provider-host scores 10; risk 3 (a pool pass trusted) 6.

**Problem / hypothesis:** an unreviewed update of either file would run with root and our identity; a pool pass
could be taken as proof.

**Goal:** the scheduled run refuses changed code, runs without root, and pool results never gate a close or release.

**Expected outcome:** both rows at 5 or below; the matrix's "needs operator acceptance: none" holds.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The scheduled entry point checks the sha256 of the submit script, the pool client and poold.py against pinned values and exits 2 without running when any differs (fixture with a changed file)
      (`scripts/pool-run.sh` + `scripts/pool-run.pins`; it also re-checks its private copy after copying, sets
      POOL_CLIENT to the pinned client, maps any other exit to 2. `tests/pool-run-fixtures.sh` 10/10; mutant without
      the mismatch refusal fails case 5.)
- [x] ~~Pool jobs run as a non-root user~~ dropped 2026-11-24, see Decisions: it needs a copy of a hub credential for that user, and pinning alone already brings risk 1 to 5
- [x] The pool result is written as advisory; no runme action, release step or task verification treats it as a pass gate (stated where it is wired)
      (stated in scripts/pool-run.sh's header, in every run's ADVISORY.txt, in .context/cron/pool-run.crontab and at
      its runme install line; the only consumer is the cron, which appends a summary line to .pool-run.log)
- [x] The first nightly cron run after the operator's runme writes its summary line and heartbeat (checked from a session)
  - Evidence 2026-11-25 09:35Z: `.context/working/.pool-run.log` has both summary pairs ("COULD NOT RUN ... 20261125T013001Z-test-13d23f60a16d" / "ADVISORY result rc=2", and "ALL PASSED ... 20261125T023001Z-clippy-13d23f60a16d" / "ADVISORY result rc=0"); `.pool-run.log.stderr` empty; heartbeat touched 2026-11-25 03:48 local. The pins held across three ring20 republishes that day (each reviewed and re-pinned before the run).

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

bash tests/pool-run-fixtures.sh > /tmp/.t3859 2>&1 && grep -q "13 passed, 0 failed" /tmp/.t3859

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

### 2026-11-24 ~16:00Z — the pin was checked only at start; a client replaced mid-run would run unreviewed
- **What changed:** ring20 replaced `/usr/local/lib/ring20-pool-client/pool-client.py` at 15:59:32Z (sha256
  e3864632…b274 -> c1675cfb…cf41). The next run was refused (PIN MISMATCH, as designed). But a run started earlier
  had passed its pin check at start, and ring20's submit script calls the client from its LIVE path minutes later:
  that run had already made two `ref-put` calls when stopped, possibly with the changed client. Stopped by hand; no
  pool process left; ring20 asked what changed (msg 84d5ffd8).
- **Plan impact:** `scripts/pool-run.sh` now copies all three pinned files into a private dir, re-checks each copy,
  and points POOL_CLIENT at the private client (poold.py beside it, since the client imports it from its own dir).
  Fixtures 11/11; the previous script fails the two new cases (2: POOL_CLIENT is a private copy; 11: a client
  replaced mid-run is not the one used). My manual test wrapper had the same start-only check and is not used again.
- **Triggered:** review of the new pool client before any re-pin.

### 2026-11-24 ~19:10Z — ring20 republished poold.py; reviewed and re-pinned before the first nightly run
- **What changed:** proxmox-ring20-management (msg 3fc1d5a3) said its node release script also republished
  `/usr/local/lib/ring20-pool-client/poold.py` at ~19:05Z (b7eb1761… -> cc319cf0…e474e6); pool-client.py and the
  submit script unchanged (sha256sum -c: all three OK against the new pins). Their stated change is node-side: a
  narrower locked-uid range per node config (`locked_uids_from`), and release of orphan leases.
- **Review:** no copy of the old poold.py was kept, so this is the same scope as the 12:43Z review, not a diff:
  (1) AST check, read-only and without importing: nothing runs at import (only imports, constants, defs, classes,
  one `__main__` guard); imports are standard library only; (2) read in full what the client calls: `Config` (plain
  dataclass, no `__post_init__`), `validate_job`, `validate_cmd`, `cmd_entry_problem`, `validate_bundle_and_refs`,
  `sha256_bytes`, the regexes HEX64 / TOOLCHAIN_NAME / OCI_ENV / REF_NAME. All pure checks in memory: no writes, no
  subprocess, no network. Our `cmd` path calls only `validate_bundle_and_refs` and the regexes. `locked_uids_from`
  is never called by the client.
- **Plan impact:** pin line updated; the pins-file comment now states the real review scope for poold.py ("read in
  full" was true for the submit script and client, not for the 213 KB daemon). Fixtures 13/0. Without the re-pin,
  tonight's 02:30 run would have refused (exit 2), as designed.
- **Triggered:** none. Keep a reviewed copy next time so a re-pin can be a diff (noted for the ring20 thread).

### 2026-11-24 19:28Z — pool-client.py republished (announced in advance); reviewed as a diff and re-pinned
- **What changed:** ring20 (msg cefc43d2, advance notice) republished pool-client.py at 19:28:14Z
  (c1675cfb… -> 37328a0b…b1b2), as announced. poold.py unchanged (cc319cf0). ring20 now keeps the replaced file
  as `<file>.prev` (msg 0316e3f2); `pool-client.py.prev` hashes to c1675cfb, the reviewed version.
- **Review:** full diff against our own saved copy (`.context/working/pool-reviewed/pool-client.py.c1675cfbfb9c`,
  kept from 19:13Z): 56 lines, additions only. One new read-only subcommand `status-jobs` (`jobs_doc` runs
  `termlink channel subscribe pool:jobs --hub <hub> --cursor 0 --limit 20 --json` and prints; no writes, no posts).
  submit / submit-cmd / wait / withdraw / ref-put unchanged; our submit script does not call the new command.
- **Plan impact:** pin updated; reviewed copies of all three files now kept in `.context/working/pool-reviewed/`
  (untracked) for the next diff.
- **Triggered:** none.

### 2026-11-24 20:59Z — poold.py republished (announced, msg e8f9f0c0); one-line node-side diff, re-pinned
- **What changed:** cc319cf0… -> f7a96c5b…85d9e9 at 20:59:14Z, as announced; `poold.py.prev` = cc319cf0 (our
  reviewed copy); pool-client.py unchanged (37328a0b).
- **Review:** full diff against `.context/working/pool-reviewed/poold.py.cc319cf0e25e`: one hunk at line 1810,
  `Poold.usage_high` (the daemon class, never used by the client): the return now prefers measured utilization
  over Anthropic's `allowed_warning` status. Nothing the client calls changed.
- **Plan impact:** pin updated; reviewed copy kept.

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

### 2026-11-24 — pinning only; no non-root user (agent decision, open to the operator)
- **Chose:** keep the pool jobs under root's TermLink identity and rely on pinning: the entry point refuses unless
  the submit script, pool-client.py and poold.py match their reviewed sha256; a new version is read in full and
  re-pinned in a commit.
- **Why:** root reaches .122 through profile `ring20-management`, whose `secret_file` is that hub's full HMAC secret.
  A non-root user would need a copy of that secret or a separately minted token: a new credential use with its own
  risk. And the matrix (docs/reports/T-3858-risk-matrix.md, risk 1) reaches 5 with pinning alone: pinning sets
  likelihood to 1; non-root would only lower impact, which the matrix already left at 5.
- **Rejected:** a non-root user with a copied hub secret (spreads a full-control credential); a non-root user with a
  minted Interact token (workable, but a second credential lifecycle for no score change). Revisit if the operator
  wants the impact side lowered too.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-11-24T12:31:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3859-pin-ring20s-pool-submit-script-and-pool-.md
- **Context:** Initial task creation

### 2026-11-24T13:45:16Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-11-25T10:08:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

## Reviewer Verdict (v1.5)

- **Scan ID:** R-b902adfc
- **Timestamp:** 2026-11-25T10:08:55Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Scope:** static anti-pattern scan; PASS means no anti-pattern was found, not that the criteria hold
- **Needs Human:** no
- **Findings:** none
