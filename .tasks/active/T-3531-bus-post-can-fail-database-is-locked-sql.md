---
id: T-3531
name: "Bus post can fail 'database is locked' (SQLITE_BUSY after 5 s) under concurrent
  reclaim / second writer connection"
description: >
  Seen 2026-10-21 in the bus suite under load: reclaim::tests::readers_never_see_a_broken_log_during_posts_trims_and_rewrites
  panicked at reclaim.rs:540 because bus.post returned Sqlite DatabaseBusy 'database
  is locked'; 6/6 isolated reruns pass. A post must never be refused because another
  writer holds the lock briefly; O-2 (stored before accepted) makes a spurious refusal
  a delivery failure. Facts: meta.rs:34 sets busy_timeout=5000 on the bus writer connection
  (mutex-serialised); worker_contract.rs:176 opens a SECOND writer connection on the
  same database since S1 (busy_timeout 5000, BEGIN IMMEDIATE); T-3488 measured write-lock
  holds of 4 s and fsync stalls far longer on this disk. Find which connection held
  the lock and why the post's wait ran out; fix so a post waits out normal contention
  (one writer path, or a bounded retry with a loud error past a much longer bound),
  with a deterministic test that holds the lock from the other connection.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [bug, arc:arc-011]
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
created: 2026-10-21T15:28:51Z
last_update: 2026-10-21T15:45:02Z
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
  - ts: '2026-10-21T15:29:31Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 4
      D3: 3
      D4: 2
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=4 (body:fw-audit-or-doctor); D3=3
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=2 (body:lightly-promoted); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-3531: Bus post can fail 'database is locked' (SQLITE_BUSY after 5 s) under concurrent reclaim / second writer connection

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Root cause named in the RCA with file:line: which connection held the write lock past the post's 5 s busy wait (bus writer, bus reader, worker_contract writer, reclaim), and since which commit the second writer connection exists
- [x] A deterministic failing-first test holds the write lock from the other connection (or simulates a slow commit) for longer than the post's wait and shows the post refused today; after the fix the post waits and succeeds within a bounded time, and past that bound fails with a named, loud error (not a raw SQLITE_BUSY)
- [x] The fix keeps the T-3488 budget: a post is still two durable steps (`a_post_is_two_durable_steps_and_a_create_is_one` unchanged) and the slow-lock log still names the waiting and holding call sites
- [x] `readers_never_see_a_broken_log_during_posts_trims_and_rewrites` passes 20 consecutive runs, including alongside a parallel `cargo build` (2026-10-22: 20/20 while the T-3533 close ran cargo build/test alongside; host load 44–93 during the loop, 22 at its end); `cargo test -p termlink-bus -p termlink-hub` passes; RCA and Evolution filled

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

cargo test -p termlink-bus --lib readers_never_see_a_broken_log_during_posts_trims_and_rewrites > /tmp/.t3531-r 2>&1 && grep -q "test result: ok. 1 passed" /tmp/.t3531-r
cargo test -p termlink-bus --lib a_post_is_two_durable_steps > /tmp/.t3531-d 2>&1 && grep -q "test result: ok. 1 passed" /tmp/.t3531-d
cargo test -p termlink-bus -p termlink-hub > /tmp/.t3531 2>&1 && ! grep -q "test result: FAILED" /tmp/.t3531

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

**Symptom:** `bus.post` returned `Sqlite(DatabaseBusy)` ("database is locked") while a reclaim and trims ran on the same topic. It was seen once in `reclaim::tests::readers_never_see_a_broken_log_during_posts_trims_and_rewrites` (reclaim.rs:540, the poster's `unwrap`). 6/6 isolated reruns passed before the fix.

**Root cause (measured, not inferred).** The post's write transaction was DEFERRED, and a deferred transaction that has already read cannot wait for a lock it needs to upgrade to.
- `crates/termlink-bus/src/meta.rs` `record_append` (the post's commit, meta.rs ~256 before the fix) ran `conn.transaction()`, a DEFERRED transaction. Its first statement, the `SELECT next_offset`, starts a read snapshot. The `UPDATE` then needs the write lock. The same pattern was at all 13 explicit write transactions in `meta.rs` (`create_topic`, `trim_records`, `commit_rewrite` at ~495, and others).
- SQLite runs the busy handler only when the connection is not already inside a transaction. An upgrade from read to write therefore fails at once with SQLITE_BUSY whenever another connection holds the write lock. The message is the same "database is locked" as BUSY_SNAPSHOT, so a stale snapshot is the same class.
- Measured in the failing-first test, which holds the lock from a second connection for 6 s: the post failed in **20 ms**, not after the 5 s `busy_timeout`. The task's premise that it "waited out 5 s then failed" is wrong. A raw experiment agrees: a deferred upgrade failed in 0.000 s, and `BEGIN IMMEDIATE` waited the full 5.005 s. Raising `busy_timeout` alone would have changed nothing on the post path.
- The second connection is `WcStore` (`crates/termlink-bus/src/worker_contract.rs:173-178`, its own `Connection` on `meta.db`). It was added by T-3501 (commit `698d630f5`, 2026-10-20), and it takes `BEGIN IMMEDIATE` and holds the write lock through its commit fsync. It is the one writer the Meta mutex does not serialise against. Its `busy_timeout` was 5 s, the same bound T-3488 measured holds exceeding (multi-second fsync stalls on this disk).
- **Not proven:** which connection held the lock in the one observed flake. In that test only the Meta writer and the read-only connection are active, and `WcStore` is idle, so the logs do not name the holder. The flake was not reproduced before the fix in 6 isolated runs, and not after it in 20 runs under synthetic load. The mechanism above is what makes any contention a refusal. The fix removes that mechanism for every holder.

**Why structurally allowed:** no test ever held the write lock from a second connection. The `busy_timeout` looked like protection in the code, and the single-connection unit tests could not exercise it. The T-3488 slow-lock log measured the wait but could not show a wait that never happened.

**Prevention:** two deterministic tests in `crates/termlink-bus/src/lib.rs`: `a_post_waits_out_a_write_lock_held_by_another_connection` (6 s hold, red before the fix, green after) and `a_post_past_the_write_lock_bound_is_refused_by_name` (named refusal past the bound, nothing written). Every write transaction in `meta.rs` now uses `TransactionBehavior::Immediate`, the same as `WcStore` already did.

**The fix.**
1. `meta.rs`: all 13 explicit write transactions use `transaction_with_behavior(TransactionBehavior::Immediate)`. The lock is taken at BEGIN, where the busy handler applies.
2. `meta.rs` `WRITE_LOCK_WAIT_MS = 30_000` replaces the 5 s bound on the Meta writer, and `worker_contract.rs` `WcStore::open` uses the same constant, so both writers wait the same time. The T-3488 durable-step budget is unchanged: a post is still two durable steps, and `a_post_is_two_durable_steps_and_a_create_is_one` passes unchanged.
3. `meta.rs` `with_conn_mut` maps a busy error past that bound to `BusError::WriteLockTimeout { site, wait_ms }` (error.rs), with a loud stderr line naming the call site. The bound it reports is the one that actually ran (`Meta.write_wait_ms`).

**Scope note.** `WcStore` (worker contract) errors past the bound still surface as `WcError::Sqlite`, not the named variant. Its wait is now the same 30 s, but naming it is not in this slice. The read-only connection keeps 5 s: a WAL reader never waits for a writer.

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

### 2026-10-21 — the premise was the wait; the cause was the upgrade
- **What changed:** The brief said the post waited the 5 s `busy_timeout` and then failed. The first failing-first run showed it failing in 20 ms. A deferred transaction cannot wait for an upgrade to the write lock, so the busy handler never ran. A bigger timeout alone would have fixed nothing on the post path.
- **Plan impact:** The fix is `BEGIN IMMEDIATE` on every write transaction, plus the 30 s bound for both writers, plus a named refusal past the bound. The brief's "raise the wait" was necessary but not sufficient.
- **Triggered:** None. Scope stayed in `crates/termlink-bus`. The worker-contract error path (`WcError`) still surfaces its busy error unnamed, recorded as a scope note in RCA.

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

### 2026-10-21T15:28:51Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3531-bus-post-can-fail-database-is-locked-sql.md
- **Context:** Initial task creation

### 2026-10-21T15:29:31Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-10-21 — fix verified, work-completed NOT set [w-db-locked]
- **Red before:** `a_post_waits_out_a_write_lock_held_by_another_connection` failed with `Sqlite(DatabaseBusy)` in 20 ms (6 s holder on a second connection). Measured, not from the 5 s wait the brief assumed.
- **Green after:** that test passes (6.3 s); `a_post_past_the_write_lock_bound_is_refused_by_name` passes (named `WriteLockTimeout` with call site and bound, nothing written); `a_post_is_two_durable_steps_and_a_create_is_one` passes unchanged.
- **Readers test:** `readers_never_see_a_broken_log_during_posts_trims_and_rewrites` passed 20/20 in a loop run directly from the test binary, with synthetic CPU and fsync load in the background. The load intensity was not measured. Not run alongside a literal parallel `cargo build`, so AC 4's wording is not fully met and that AC stays open.
- **Suite:** `cargo test -p termlink-bus -p termlink-hub` exit 0; bus 199/199, hub lib 556 passed, 0 failed.
- **Open:** the holder in the one observed flake is not identified by the logs (only the Meta writer and the read-only connection are active in that test; WcStore is idle). The mechanism that turned any contention into a refusal is removed for every holder.
- **Commit:** pending, paths named explicitly; the rest of the working tree is framework state, not part of this change.
