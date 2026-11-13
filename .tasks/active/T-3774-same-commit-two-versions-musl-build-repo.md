---
id: T-3774
name: "Same commit, two versions: musl build reports 0.12.1 while the host build reports
  0.12.11 (build.rs version not re-derived when a tag is added)"
description: >
  runme run 20261113T103227Z (HEAD 60f3d57f7 = tag v0.12.11): host binaries report
  'termlink 0.12.11', the musl binary deployed to ring20-management and ring20-dashboard
  reports 'termlink 0.12.1' (target/x86_64-unknown-linux-musl/local-fast/termlink
  --version). 0.12.1 = v0.12.10 + 1 commit, i.e. the musl build.rs ran before the
  v0.12.11 tag existed and was not re-run when the tag was added (rerun-if-changed
  does not track new tags). Effects: identical code reports two versions; '0.12.1'
  collides with the real old release v0.12.1; runme's upgrade check, the fleet binary-freshness
  and capability canaries compare these strings (PL-243). Fix: build.rs reruns when
  refs/tags change (e.g. rerun-if-changed on .git/packed-refs and .git/refs/tags)
  and the version check uses the commit, not only the string; test: tag after build
  -> rebuild picks it up.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [bug, release]
components: []
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
created: 2026-11-13T10:50:12Z
last_update: 2026-11-13T12:33:47Z
date_finished:
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
  - ts: '2026-11-13T12:23:09Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 1
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
    rationale: D1=1 (body:fix-without-learning); D2=0 (no-signal); D3=0 
      (no-signal); D4=0 (no-signal); F-RECALL=0 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-3774: Same commit, two versions: musl build reports 0.12.1 while the host build reports 0.12.11 (build.rs version not re-derived when a tag is added)

## Context

**Background / trigger:** runme run 20261113T103227Z showed the same code reporting two versions
(0.12.11 on the host, 0.12.1 on the musl build); the orchestrator then found the real cause below.

**Problem / hypothesis:** the version scheme itself is broken, not the build cache: dev builds get
numbers that are identical to old releases and that fall at every tag, so every reader that compares
versions (canary floors, --min-version gates, "is this hub current?") can be wrong.

**Goal:** every build reports a version that is unique to its commit and orders correctly against
releases and other dev builds, while a tagged release still reports exactly its tag.

**Expected outcome:** v0.12.11 + 4 commits reports `0.12.11+4.ga2fcb16f8`, never `0.12.4`; the
scripts that compare versions order it after 0.12.11 and before 0.12.12.

Orchestrator re-diagnosis (2026-11-13, supersedes the description's stale-build guess): the
git-derived scheme in the four `crates/*/build.rs` (tag exact = `0.12.11`; N commits after a tag =
`major.minor.N`) produces version strings that COLLIDE with real releases and go DOWN at each new
tag: v0.12.11+4 reports `0.12.4` (= release v0.12.4), v0.12.10+1 reported `0.12.1`, and the
sequence ran 0.12.1610 -> 0.12.61 -> 0.12.11 -> 0.12.4. PL-243 already warned these are not
comparable. Readers: runme upgrade/deploy checks (string equality), the fleet binary-freshness
canary (`version_lt`, floors file), `arc-live-probe.sh --min-version` (`version_ge`),
`check-version-derivation.sh` (presence only, unaffected), docs.

## Acceptance Criteria

### Agent
- [x] One derivation used by all four crates (`crates/version-derive.rs`, `include!`d by each build.rs): exact tag `vX.Y.Z` -> `X.Y.Z`; N>0 commits after it -> `X.Y.Z+N.g<sha9>`; unparseable / no tag -> None (Cargo.toml fallback, as before)
- [x] Unit tests of the mapping (exact tag, after-tag, the v0.12.11+4 collision case, pre-release tag, garbage -> None) pass under `rustc --test` (`tests/version-derive-tests.sh`); negative control: the old `major.minor.N` mapping put back fails 4 of them, incl. the 0.12.4 collision
- [x] Shared bash comparator `scripts/lib/version-cmp.sh` (SemVer precedence, then the commit count N as tie-break; sha never read as a number); `check-fleet-binary-freshness.sh`, `arc-live-probe.sh` and `substrate-preflight.sh` source it instead of their own copies
- [x] Fixture `tests/version-cmp-fixtures.sh` proves the chain 0.12.10 < 0.12.11-rc1 < 0.12.11 < 0.12.11+4.g… < 0.12.11+10.g… < 0.12.12 < … < 1.0.0 pairwise and that sha digits never leak; negative control: the old dotted comparator orders `0.12.11+4.g…` above `0.12.12`
- [x] Readers keep working and gain new-scheme cases: fleet freshness fixture (16, incl. 2 T-3774 with control), arc-live-probe (2 T-3774 cases, one must fire), preflight T6/T7 (the embedded commit decides; same number, different sha = the control), version-derivation fixtures (16)
- [x] Fleet floors rewritten to the same commit in the new form (`0.12.786` = b0ed631de = `0.12.4+786.gb0ed631de`)
- [x] `env -u PROJECT_ROOT cargo build -p termlink` succeeds; the binary reports `0.12.11+6.gc559353a4` at HEAD (6 past v0.12.11); clippy on the four crates clean; session's `recorded_version_is_the_build_version…` test passes
- [x] Decisions records the chosen scheme and the rejected ones; operator-facing consequences in the handback

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

bash tests/version-derive-tests.sh > .context/working/t3774/v1.out 2>&1 && grep -q 'PASS  control' .context/working/t3774/v1.out
bash tests/version-cmp-fixtures.sh > .context/working/t3774/v2.out 2>&1 && grep -q ' 0 failed' .context/working/t3774/v2.out
bash scripts/test-check-fleet-binary-freshness.sh > .context/working/t3774/v3.out 2>&1 && grep -q 'T-3774: same-tag dev build with fewer commits fires' .context/working/t3774/v3.out
bash scripts/test-arc-live-probe.sh > .context/working/t3774/v4.out 2>&1 && grep -q 'T-3774: dev build < next release' .context/working/t3774/v4.out
bash scripts/test-substrate-preflight.sh > .context/working/t3774/v5.out 2>&1 && grep -q 'PASS: T7' .context/working/t3774/v5.out
bash tests/version-derivation-check-fixtures.sh > .context/working/t3774/v6.out 2>&1 && grep -q ' 0 failed' .context/working/t3774/v6.out
bash scripts/check-version-derivation.sh --no-heartbeat > .context/working/t3774/v7.out 2>&1 && grep -q 'clean' .context/working/t3774/v7.out
grep -l 'include!("../version-derive.rs")' crates/termlink-cli/build.rs crates/termlink-hub/build.rs crates/termlink-session/build.rs crates/termlink-mcp/build.rs > .context/working/t3774/v8.out 2>&1 && test "$(wc -l < .context/working/t3774/v8.out)" -eq 4
grep -q '0.12.4+786.gb0ed631de' .context/cron/fleet-version-floors.conf && ! grep -qE '^[a-z0-9*-]+ 0\.12\.786$' .context/cron/fleet-version-floors.conf

## RCA

**Symptom:** the same code reported two versions (host 0.12.11, musl 0.12.1), and a dev build at
v0.12.11+4 reported 0.12.4, the number of an old release. Fleet floors (0.12.786) read new code as stale.

**Root cause:** the build.rs scheme `major.minor.N` (N = commits since the nearest tag) throws away
the tag's patch number. Each new tag resets N, so the numbers fall at every tag and land on numbers
real releases already used. The musl/host split itself was runme's cached build (T-3292) keeping a
binary built one commit before the tag existed: same code, different N, by design.

**Why structurally allowed:** four copies of the derivation and four copies of a dotted-numeric
comparator, none tested across a tag boundary. PL-243 recorded "not comparable" as advice to readers
instead of fixing the writer. check-version-derivation.sh (T-2746) checks only that the override
exists, not what it computes.

**Prevention:** one derivation (`crates/version-derive.rs`) whose unit tests include the collision
case, with a mutation control that puts the old mapping back (`tests/version-derive-tests.sh`); one
comparator (`scripts/lib/version-cmp.sh`) with pairwise-order fixtures and a control.

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

### 2026-11-13 — dev-build version form
- **Chose:** `<tag>+<N>.g<sha9>` (e.g. `0.12.11+4.ga2fcb16f8`); a tagged commit still reports exactly the tag.
- **Why:** unique per commit (sha); names the release it sits on; orders after its tag and before the
  next under `sort -V` and our comparator; same shape the framework uses for FW_VERSION between
  releases (AEF T-4055), so one convention across the stack; valid SemVer.
- **Rejected:** `0.12.12-dev.N+g<sha>` (SemVer-ordered by spec, but claims a next version 0.12.12 that
  may never exist, and GNU `sort -V` puts `0.12.12-dev.4` ABOVE `0.12.12`); four-part `0.12.11.N` (not
  SemVer; breaks 3-part parsers such as homebrew/render-formula.sh and Rust parse_version); keep
  `major.minor.N` and only fix rerun triggers (the description's fix; leaves the collisions, which are the bug).
- **Caveat:** strict SemVer ignores build metadata, so our comparator reads N as an explicit
  tie-break; two builds with equal N on different branches compare equal (tell them apart by string).

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-11-13T10:50:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3774-same-commit-two-versions-musl-build-repo.md
- **Context:** Initial task creation

### 2026-11-13T12:23:07Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
