---
id: T-3781
name: "Non-root dispatch: termlink dispatch / fw termlink dispatch use register --hub
  for a non-root caller (T-3760 follow-up)"
description: >
  T-3760 shipped the registration claim, owner-bound hub methods and termlink register
  --hub. Remaining for operator ruling A (T-3759): the dispatch path must, for a non-root
  caller, start the worker as that user and register it with register --hub + the
  user's claim token instead of the local runtime dir; never fall back to root; refuse
  when no claim token is provisioned.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:arc-011, security]
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
created: 2026-11-13T17:55:21Z
last_update: 2026-11-13T23:34:45Z
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
  - ts: '2026-11-13T23:10:20Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 5
      D2: 0
      D3: 0
      D4: 0
      F-TRUST: 3
      F-SURFACE: 0
      F-CONV-RECALL: 0
    rationale: 'D1=4-5 (body:new-class); D2=0 (no-signal); D3=0 (no-signal); D4=0
      (no-signal); F-TRUST=3 (L3:keyword=allow-list,L3:keyword=fail closed); F-SURFACE=0
      (L0: no signal); F-CONV-RECALL=0 (L0: no signal)'
    rubric_sha: e4a00f38e801
    driver_set: product
---

# T-3781: Non-root dispatch: termlink dispatch / fw termlink dispatch use register --hub for a non-root caller (T-3760 follow-up)

## Context

**Background / trigger:** the 1409 design session runs as the non-root user `dimitri-mint-dev` and cannot
dispatch reviewers: `termlink register` refuses or lands invisible, because the hub's runtime dir is root-only
(T-3759, operator ruling A + D, 2026-11-13). T-3760 built the hub side and `register --hub`.
**Problem / hypothesis:** nothing on the dispatch paths uses that yet; `fw` exports the root-only runtime dir and
`termlink dispatch` needs the hub's Unix socket and the hub's disk-only `event.collect`.
**Goal:** a non-root user with a provisioned hub link can dispatch workers that run as that user and are visible
on the main hub, without root and without anyone else's identity.
**Expected outcome:** `fw termlink dispatch` and `termlink dispatch` work for `dimitri-mint-dev` on .107; the
workers appear in `session.discover` owned by that user; without a link the user gets a clear refusal.

Operator ruling A (T-3759): a worker runs as the requesting user and registers on the main hub over TCP with
its own registration claim (built in T-3760: `termlink register --hub --hub-token-file`, owner-bound records).
This task wires that into the paths a non-root user actually uses:

1. `fw termlink dispatch` (vendored) runs `termlink spawn --shell` and then a local `termlink pty inject`. It
   exports the hub's runtime dir (`/var/lib/termlink`, root 0700) first, so the worker's `register` cannot write
   there (T-3759 report, antigravity.md:47).
2. `termlink dispatch` checks the hub on its Unix socket and collects through the hub's `event.collect`, which
   only sees sessions on the hub's own disk (`router.rs:572`), never a TCP-registered worker.

Design (agent initiative within ruling A; recorded in Decisions):
- **Hub link** = a registration-claim token file plus the hub address, provisioned per user:
  `TERMLINK_HUB_REGISTER_TOKEN_FILE` / `TERMLINK_HUB_REGISTER_ADDR`, else `~/.termlink/hub-register.token` and
  `~/.termlink/hub-register.addr`. Files, not only env, because `fw` spawns through an `env -i` allow-list.
- **Runtime dir**: for a non-root caller with a hub link, a `TERMLINK_RUNTIME_DIR` that the caller cannot
  write is ignored (its own default dir is used), so register, spawn, inject and list agree.
- **register** (also `--shell`): with a hub link, the same session is also served on TCP (token auth) and
  registered on the hub with a lease; the hub-side name is the local name, or `<prefix><name>` when the local
  name is not under the claim's prefix. If the hub refuses, the session does not start (fail closed).
- **dispatch**: when the local hub socket is unreachable and the caller is non-root with a hub link, it checks
  the hub over TCP and collects results from the workers' own sockets (same user). Without a link it refuses,
  naming how to provision one. It never runs anything as another user.
- **Provisioning** for `dimitri-mint-dev` (the 1409 design session's user): a runme action mints the claim
  token (7 days, ruling A 2026-11-13) and the weekly re-mint renews it.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Hub-link resolution: env first, then the two default files; one file without the other is an error that
      names the missing one; no link → None. Unit-tested.
- [x] Runtime-dir rule: a non-root caller with a hub link and an unwritable `TERMLINK_RUNTIME_DIR` resolves to
      its own default dir; root, no link, or a writable dir leave the variable alone. Unit-tested.
- [x] Hub-side name: local name kept when the claim permits it, else `<prefix><name>`; a result the claim still
      refuses (bad chars, > 64) is an error. Unit-tested.
- [x] `termlink register` (plain and `--shell`) with a hub link serves TCP with token auth, registers on the hub
      and deregisters on exit (the lease loop is T-3760's, moved unchanged; its answer mapping stays unit-tested);
      `register --hub` (T-3760) keeps working through the same code. Proven by integration tests against a
      throwaway TCP hub with a claim token.
- [x] `termlink dispatch` in linked mode registers N workers on the hub and collects their results from the
      workers' own sockets; without a link and without a local hub it refuses with a provisioning hint. Tested.
- [x] Live smoke on .107 as `dimitri-mint-dev`: `termlink spawn --shell` + `pty inject` + the record visible in
      `session.discover` on the root hub with owner `dimitri-mint-dev`; a negative control without the link
      file is refused or invisible as designed. Evidence in Updates.
- [x] runme action provisions `dimitri-mint-dev`'s link (token 0600 owned by the user, addr file), idempotent and
      self-verifying, with a hermetic case in `tests/runme-fixtures.sh`; the weekly re-mint renews it.
- [x] `cargo test -p termlink` (cli crate) and `cargo clippy --workspace -- -D warnings` clean; mutants of the
      runtime-dir rule and the name rule are killed.

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

env -u PROJECT_ROOT cargo test -p termlink --bin termlink hub_link > .context/working/t3781/v-unit.out 2>&1 && grep -q "test result: ok" .context/working/t3781/v-unit.out
env -u PROJECT_ROOT cargo test -p termlink --test t3781_hub_link > .context/working/t3781/v-it.out 2>&1 && grep -q "test result: ok. 4 passed" .context/working/t3781/v-it.out
grep -q "rootcheck KILLED" .context/working/t3781/mutants.out && grep -q "prefix KILLED" .context/working/t3781/mutants.out && grep -q "halflink KILLED" .context/working/t3781/mutants.out && grep -q "unixscope KILLED" .context/working/t3781/mutants.out && grep -q "localcollect KILLED" .context/working/t3781/mutants.out
grep -q "dimitri-mint-dev-smk owner= dimitri-mint-dev" .context/working/t3781/smoke.out && grep -q "collected 2" .context/working/t3781/smoke.out && grep -q "has no hub link" .context/working/t3781/smoke.out
bash -n scripts/hub-link-provision.sh && bash -n runme.sh

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

### 2026-11-13 — expiry of the non-root registration capability (operator ruling, from T-3760)
- **Chose:** A — 7 days (the existing token ceiling), renewed weekly by the existing re-mint mechanism
  (as for the dimitri-mint-dev token, T-3756).
- **Why:** the capability can only register the user's own prefixed workers and forward nothing above observe;
  7 days adds little risk and needs no new mechanism.
- **Rejected:** B 24 h (daily re-mint, a missed renewal fails sooner); C 30 days (raise the hub ceiling for all tokens).
- Operator words in chat: "A". Provisioning the 1409 user's token becomes a runme action in this task.

### 2026-11-14 — where the non-root route lives (agent, within ruling A)
- **Chose:** inside `termlink` itself: `register` (also `--shell`) links to the hub when the user has a hub link,
  and a non-root caller with a link ignores a `TERMLINK_RUNTIME_DIR` it cannot write. `fw termlink dispatch`
  (vendored) needs no change: it runs `termlink spawn --shell` + a local `pty inject`, which now just work.
- **Why:** a vendored change would be lost at the next re-vendor (G-062) and would need AEF's release; the rule
  in termlink covers every caller (fw, MCP, scripts).
- **Rejected:** a separate `--hub` flag on spawn/dispatch (fw strips unknown env and passes no such flag);
  editing AEF's `_tl_resolve_runtime_dir` (upstream, slower, still leaves other callers broken).

### 2026-11-14 — the link is two files, plus an env override
- **Chose:** `~/.termlink/hub-register.token` + `~/.termlink/hub-register.addr`; env
  `TERMLINK_HUB_REGISTER_TOKEN_FILE` + `TERMLINK_HUB_REGISTER_ADDR` override (both or neither).
- **Why:** `fw` spawns through an `env -i` allow-list (lib/spawn-env.sh), so env alone would not reach a
  worker; HOME does. Half a link is an error that names the missing file, never a silent fallback.

### 2026-11-14 — hub-side name gets the claim's prefix
- **Chose:** a linked session keeps its local name and registers on the hub as that name, or `<prefix><name>`
  when the claim does not permit it. `register --hub` (T-3760) stays strict.
- **Why:** callers such as fw name their workers without knowing the claim; refusing would break every fw
  dispatch. The prefix still binds every hub name to the user, so no impersonation is possible.

### 2026-11-14 — a linked session's own user keeps full access on the Unix socket
- **Chose:** the TCP side needs a token secret (token-only); a new `run_accept_loop_tcp_token_only` keeps
  same-UID Unix connections at Execute (peer credentials already prove the user). `--token-secret` still
  restricts local connections as before.
- **Why:** with the secret, local connections fell to Observe, so the user's own `pty inject` and `exec`
  (what fw dispatch uses) would fail. Found by reading `server.rs:335` before testing; the integration test
  pins it (`exec` as the user succeeds).

### 2026-11-14 — dispatch collects from the workers' own sockets in linked mode
- **Chose:** when linked, `termlink dispatch` polls each worker's own Unix socket (same user) for its result.
- **Why:** the hub's `event.collect` resolves targets with `manager::find_session` on its own disk
  (`router.rs:572`), so it never sees a TCP-registered worker. Forwarding through the hub is T-3782's scope.

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

### 2026-11-13T17:55:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3781-non-root-dispatch-termlink-dispatch--fw-.md
- **Context:** Initial task creation

### 2026-11-13T23:10:19Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
