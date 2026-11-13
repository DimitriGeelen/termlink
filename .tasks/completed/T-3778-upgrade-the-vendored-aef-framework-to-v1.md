---
id: T-3778
name: "Upgrade the vendored AEF framework to v1.8.16-be.49 (dispatch scan fix T-5105)
  via one runme action, keeping Watchtower reachable on the network"
description: >
  Operator ruling A 2026-11-13. Vendored copy is fw v1.8.15+671.gc55b395a3; AEF T-5105
  (be.49, commit 18010f8d6) makes termlink.sh _all_wdirs linear and fork-free (60-80
  s -> 0.4 s per dispatch on .107 with 7,371 dirs in /tmp/tl-dispatch), which unblocks
  fw reviewer judge (hard 120 s dispatch timeout) and our T-3356/T-3357 reviews (T-3777).
  Known side effect (AEF be.46): Watchtower binds 127.0.0.1 after upgrade; the action
  must set it back (fw config set HOST 0.0.0.0; fw watchtower restart) and verify
  http://192.168.10.107:3003 answers from the LAN address. Steps: scripts/check-vendor-divergence.sh
  clean (CLAUDE.md rule 2 + .vendor-divergence.yaml checklist), upgrade to be.49,
  CLAUDE.md project section above '## Core Principle' preserved (T-2015), HOST restored,
  verify fw version, Watchtower on 192.168.10.107:3003, a dispatch reaching 'Worker
  spawned' in < 10 s, fw doctor. Rejected: B wait for T-5120 retention; C no upgrade.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [runme.sh, tests/runme-fixtures.sh]
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
created: 2026-11-13T15:01:36Z
last_update: 2026-11-13T18:21:29Z
date_finished: 2026-11-13T18:21:29Z
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
  - ts: '2026-11-13T15:03:22Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=0 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-3778: Upgrade the vendored AEF framework to v1.8.16-be.49 (dispatch scan fix T-5105) via one runme action, keeping Watchtower reachable on the network

## Context

**Background / trigger:** we vendor AEF at be.27 (`1.8.15+671.gc55b395a3`, T-3662). AEF T-5105 (v1.8.16-be.49,
18010f8d6) makes `termlink.sh _all_wdirs` linear and fork-free: 60-80 s became 0.4 s per dispatch on .107 with 7,371
dirs in /tmp/tl-dispatch. Without it `fw reviewer judge` hits its hard 120 s dispatch timeout, which blocks the
T-3356/T-3357 reviews (T-3777). Operator ruling A, 2026-11-13.

**Problem / hypothesis:** every earlier re-vendor (T-3589..T-3662) was run by a worker from a fresh upstream clone.
This one must ship as ONE idempotent, self-verifying `runme.sh` action the operator runs, and it has a known side
effect: since AEF be.46 Watchtower binds 127.0.0.1 after an upgrade, so the LAN URL http://192.168.10.107:3003 dies.

**Goal:** a runme action that preflights (divergence clean, upstream clone has be.49 and our commit is its ancestor),
runs `fw upgrade` from a fresh clone at be.49 with the session env unset, keeps the CLAUDE.md project part and the 12
preserved files, restores `HOST 0.0.0.0` and restarts Watchtower, verifies on disk, commits with a T-3778 message and
pushes to origin only. FAILED rc 1 on any miss; skip when already at be.49+ and verified.

**Expected outcome:** after one operator run, `fw version` reports be.49 (or later), CLAUDE.md above `## Core
Principle` is byte-identical, Watchtower answers on 192.168.10.107:3003, `fw doctor` has no FAIL, and dispatches no
longer time out.

## Acceptance Criteria

### Agent
- [x] `runme.sh` has one T-3778 action that: skips when the vendored version is at or past be.49 AND the post-checks
      pass; otherwise checks divergence clean, clones upstream into a fresh dir, resolves the be.49 tag and requires
      our vendored commit to be its ancestor, runs `fw upgrade --dry-run` then `fw upgrade` with
      FRAMEWORK_ROOT/PROJECT_ROOT/FW_ROUTED_FROM unset
      **Evidence:** runme.sh action 11a (`aefu_upgrade`).
- [x] The action verifies on disk and reports FAILED (rc 1) when any check misses: version at or past be.49 (by
      ANCESTRY of the commit named in VERSION against the tag, since AEF's VERSION is `1.8.15+N.g<sha>` and never names
      be.49; `version-cmp.sh` orders TermLink versions, not this), CLAUDE.md above `## Core Principle` byte-identical
      to the pre-upgrade copy, every preserved file byte-identical to its pre-upgrade copy, HOST 0.0.0.0 set and
      Watchtower answering on http://192.168.10.107:3003, no new `fw doctor` FAIL
      **Evidence:** fixture controls oldver, clobber, keep, wt dead, nohost, doctor: each FAILED rc 1.
- [x] The action commits only the vendored tree and the upgrade's own outputs with a T-3778 message and pushes to
      origin (never GitHub)
      **Evidence:** commit pathspec = paths changed this run, filtered to the upgrade's paths (`aefu_upgrade_paths`);
      `git push origin HEAD:main`; fixture checks the bare origin's main equals HEAD.
- [x] `tests/runme-fixtures.sh` covers it hermetically through RUNME_* seams: skip-when-done, success path, and
      negative controls (CLAUDE.md project part changed, Watchtower not answering, version below be.49, ancestry
      failure) each yield FAILED; no real host state, network or framework touched
      **Evidence:** 17 T-3778 cases (12 controls), scratch git repos under the fixture TMP.
- [x] `bash -n runme.sh` passes and the runme fixture suite passes
      **Evidence:** 344 passed, 0 failed (.context/working/t3778/fx.out).
- [x] The two conflicting preserved-file merges (safe-commands.sh, handover.sh) are resolved and stored with a hash
      manifest in `.context/vendor-merges/T-3778/`; the resolved safe-commands.sh classifies like be.49's own copy
      except the T-3178 trailing `>` (fails closed)
      **Evidence:** MANIFEST.tsv hashes verify; 11 forms measured against be.49's copy, only `echo x >` differs (ours W).
- [x] After the operator's run, the runme log shows every 11a line OK (or skip) and no FAILED line other than the
      known-offline greenfield host; `last_vendor_event` in `.vendor-divergence.yaml` is promoted to the T-3778 commit
      (evidence: run 20261113T165216Z, 11a: upgraded, HOST, committed 5db3d2dbb, pushed, Watchtower answering, doctor no
      new FAIL; rc=1 only from 3 greenfield FAILED lines (.132 offline, ring20 T-2595); last_vendor_event = 5db3d2dbb;
      wording changed from "rc=0" because rc stays 1 while .132 is offline; dispatch start measured 3.8 s after)

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
bash -n runme.sh
bash -n tests/runme-fixtures.sh
TMPDIR=/var/tmp timeout 1500 bash tests/runme-fixtures.sh > .context/working/t3778-fx.out 2>&1 && grep -q 'T-3778: upgrade => at be.49' .context/working/t3778-fx.out && ! grep -q 'FAIL.*T-3778' .context/working/t3778-fx.out
bash -c 'cd .context/vendor-merges/T-3778 && grep -v "^#" MANIFEST.tsv | awk -F"\t" "{print \$4\"  \"\$1\".merged\"}" > ../../working/t3778-sums && test -s ../../working/t3778-sums && sha256sum -c --quiet ../../working/t3778-sums'
bash scripts/check-vendor-divergence.sh > .context/working/t3778-cvd.out 2>&1 && grep -q 'all registered' .context/working/t3778-cvd.out
python3 -c "import yaml; yaml.safe_load(open('.vendor-divergence.yaml'))"
# the operator's run 20261113T165216Z: every 11a line OK, and the only FAILED lines are the offline greenfield host (rc stays 1 while .132 is offline)
grep -q 'OK      fw doctor: no new FAIL; vendored framework at 1.8.15+1122.gda9942919' .context/working/runme-logs/runme-20261113T165216Z-355154.log
grep -E '^  FAILED ' .context/working/runme-logs/runme-20261113T165216Z-355154.log > .context/working/t3778/failed-lines.txt; ! grep -v 'greenfield' .context/working/t3778/failed-lines.txt
grep -q 'T-5105' .agentic-framework/agents/termlink/termlink.sh

## RCA

**Symptom:** every `fw termlink dispatch` on .107 took 60-80 s before 'Worker spawned', so `fw reviewer judge` (hard
120 s dispatch limit) failed 'no seat was dispatched' and the T-3356/T-3357 reviews could not run (T-3777).
**Root cause:** our vendored AEF copy (be.27) predated AEF T-5105; its `_all_wdirs` scanned 7,371 worker dirs in the
shared /tmp/tl-dispatch with two forks each and a quadratic string match.
**Why structurally allowed:** nothing measures dispatch start time, and a vendored framework only changes when the
operator rules an upgrade; the slowdown grew with the shared dir until it crossed the judge's limit.
**Prevention:** upgraded to be.49 (measured 3.8 s after); AEF T-5121 makes the judge limit configurable and names it on
timeout; AEF T-5120 (retention of finished worker dirs) is filed upstream.

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

### 2026-11-13 — upgrade the vendored framework to be.49 (operator ruling)
- **Chose:** A — upgrade now to v1.8.16-be.49 (T-5105 dispatch scan fix), through one runme action that restores
  Watchtower's LAN binding.
- **Operator's words:** "A"
- **Rejected:** B wait for T-5120 retention; C no upgrade.

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

### 2026-11-13T15:01:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3778-upgrade-the-vendored-aef-framework-to-v1.md
- **Context:** Initial task creation

### 2026-11-13T15:03:21Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-11-13T18:21:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

## Reviewer Verdict (v1.5)

- **Scan ID:** R-95c0ce16
- **Timestamp:** 2026-11-13T18:42:39Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Scope:** static anti-pattern scan; PASS means no anti-pattern was found, not that the criteria hold
- **Needs Human:** no
- **Findings:** none
