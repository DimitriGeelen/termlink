---
id: T-3426
name: "arc-011 PH-2 planning: architecture + build table (first operational release)"
description: >
  arc-011 PH-2 planning: architecture + build table (first operational release)

status: started-work
workflow_type: design
owner: agent
horizon: now
tags: []
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
created: 2026-10-12T19:17:19Z
last_update: 2026-10-14T19:23:03Z
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

# T-3426: arc-011 PH-2 planning: architecture + build table (first operational release)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Gap map of PH-2 scope (03 section 6.2.3.a-.f) against what PH-1 built, with file:line evidence (`docs/reports/T-3426-arc011-ph2-gapmap.md`)
- [x] `docs/design/arc-011-ph2-architecture.md` written on the PH-1 model: components, waves W1-W4 with rollback, how each floor item of 6.2.3.c and each exit of 6.2.3.d is proven
- [x] Build table: each row one deliverable, dependency order, sized to one session, citing the R-n/SI-n it closes
- [x] Open operator questions listed in the doc, to be asked one at a time
- [x] arc-011 horizon plan updated with the PH-2 build tasks
- [ ] Operator rulings on OQ-a, OQ-f, OQ-b, OQ-c, OQ-d, OQ-e, OQ-h, OQ-i recorded in the doc section 6 and in Decisions

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

### 2026-10-12 — OQ-a: how the sidecar ships (operator ruling)
- **Chose:** A — fold it into the main binary as `termlink sidecar …` (row P1, T-3365).
- **Why:** OD-9 C ruled "in the binary"; one artifact, one checksum (SI-27).
- **Rejected:** B two binaries (packaging and checksums twice; OD-9 C met only by exception).

### 2026-10-12 — OQ-f: what keeps the sidecar running (operator ruling, shaped in dialogue)
- **Chose:** D+ — per-project `SIDECAR_KEEP_RUNNING` (default off) in `.framework.yaml`, also the key the fleet cockpit sets; project start always starts the sidecar; restarts only when the setting is on; an explicit project stop always kills it and suspends restarts, a session ending does not; systemd template unit where present, else cron as supervisor; plus a 10-minute cron backstop on every host that repairs through systemd and alerts each time it acts. Full text: architecture 6.1.
- **Why:** operator: not every project needs to keep running, the cockpit is not in every project yet; systemd can stop restarting silently (start-limit-hit, as D-Bus did in T-3425's boot failure), so a slower backstop is worth having.
- **Rejected (OQ-f):** A alone (every project kept running; silent give-up), B alone (no storing while away, no R-8), C alone (minute-scale gaps on systemd hosts), cron restarting around systemd (two supervisors).

### 2026-10-12 — OQ-b: agents run as root, no local separation (operator ruling)
- **Chose:** A — accept for PH-2; the status call states "same-user separation absent: agents run as root".
- **Why:** PH-2 is one host, one project, the operator's own agents; real separation needs per-agent accounts (Track A), which needs PH-3's Tier 0 machinery.
- **Rejected (OQ-b):** A+ (also surface every admin act and foreign use of a send socket after the fact; offered, not taken), B (wait for Track A: would fold PH-2 into PH-3).

### 2026-10-13 — FX-5: the artifact store and attachments (operator ruling, after external consult)
- **Chose:** C′ — count the artifact store (baseline 9, correcting the PH-0 measurement); adopt the "workflow" reading of FX-5 (a new hub-readable copy of a content class the hub does not hold today is growth, whatever the field count); P15 attachments encrypted to the recipient's key or sent sidecar to sidecar; threat boundary stated (protects stored hub data and token holders, not against the host operator inspecting a sidecar); check: today's artifact-store writers allowlisted, new writers fail unless via the encryption wrapper, plus a size cap on plaintext message fields.
- **Why:** both reviewers (Codex gpt-6-astra, GLM glm-5.3; `docs/reports/T-3426-fx5-consult/`) picked C amended: the store exists at PH-0 so excluding it understates the baseline; routing agent attachments through it is growth under the workflow reading (otherwise the floor is vacuous); a sidecar-only check is one call site from bypass. The weighted scores (A +22, C +17, D −5, B −18) are recorded as inapplicable: the drivers cannot price confidentiality and FX-5 is a floor.
- **Rejected:** A (capability reading: plumbing taken as exposure), B (partial inventory), D (design would start from the plaintext store). GLM's hub-side refusal of all unencrypted artifacts not adopted as worded: it would break today's `termlink file send`, a baseline rail.
- **Corrections recorded:** sidecar-only check scope; field count is a tripwire, not a disclosure measure; threat boundary missing; scoring framed as a near-loss; the interpretation of FX-5 now settled by this ruling, not by the measurer.
- **Left open:** inventory check (store x form x readers x retention) as the primary FX-5 check (follow-on task); whether the hub later refuses unencrypted artifacts from new writers; a confidentiality value driver.

### 2026-10-14 — P9-1: how the sidecar's supporting tools ship (operator ruling)
- **Chose:** A — embed in the binary: `termlink sidecar tools install` writes the tested supervise script, unit template and backstop crontab to `<prefix>/share/termlink/sidecar/` (paths rewritten to that prefix); `install.sh` keeps the release checksum record and installs the tools; status reports whether the running binary matches the release checksum (yes / no / no record); a Docker clean-host install/upgrade/rollback/restart test.
- **Why:** meets R-39 (start and supervise from the deployment, no checkout) and SI-27 with the files that passed today's fixtures and the live W1 run, inside the one artifact whose checksum is already verified; no new release plumbing. Scored A +37 (C +45 on drivers, but several sessions and a re-proof of P3; B +3; D −15).
- **Rejected:** B release tarball (four places to keep in sync plus per-file checksums); C Rust rewrite now (later, with macOS/launchd R-8.g); D defer (R-39 stays unmet).
- **Left open:** C as a later row; the generic launcher/provisioning form (the W1 provisioning script stays host-specific until W3).

### 2026-10-14 — W1 go (operator ruling)
- **Chose:** B — go now: the operator runs `runme.sh` (action 17 provisions and starts both sidecars under systemd), opens `claude-termlink-alt` through the launcher now; the main session joins at its next natural restart (R-48 "next natural start"). The 7-day window counts from when both agents are in.
- **Why:** gate met (P0-P8 done, drain proven on the estate, W1 readiness "ready" in the 2026-10-13 22:13Z run); B proves provisioning, supervision and the stored-while-closed path today without interrupting the working session. Scored B +40, A +17, C −4.
- **Rejected:** A (relaunch the main session now: interruption for a few hours' head start); C (wait for .132 and the framework upgrade: neither touches the sidecar path; .132 has since recovered).
- **Recorded in:** `runme.sh` `ARC011_W1_GO`. Operator's words: "proceed" after the B recommendation (read as B).
- **Left open:** the kill-switch drill and W2 (gated on 7 clean days with both agents in).

### 2026-10-13 — P7-4: instance ids for the W1 pair (operator ruling; corrects a default)
- **Chose:** A — each pair member gets an instance id minted by the W1 runme action (e.g. `claude-termlink.w1-20261013`; corrected from the `@` form in the brief, which the API's name rule refuses — found by the P7 test), stable across sidecar restarts during W1 so the partner can address it, never reused (re-provisioning mints a new date-stamped id); each sidecar's config carries its partner's id. R-62's "a copy's runtime id is never reused" is read at provisioning granularity for W1; per-copy ids arrive with role resolution (R-67) in a later row.
- **Why:** the instance is fixed per sidecar at start (`daemon.rs:27`), the receiver refuses any other `to_instance` (`wire.rs:323`), and senders must name it (`api.rs:508`); under P7-3 B the sidecar runs before any Claude session exists, so a session id cannot be the instance. A sidecar-owned id is also harness-neutral (operator question: vendors, models, harnesses — only the delivery adapter is harness-specific; `adapter/claude.rs` is the only one).
- **Rejected:** default 5 as first proposed (Claude session id from SessionStart: unknowable to the partner, harness-specific); B (build R-67 role resolution into P7 first: much larger, delays W1).
- **Left open:** R-67 role resolution and per-copy ids; a second harness adapter (Codex/opencode) would be its own row with an adapter interface.

### 2026-10-13 — P7-3: SIDECAR_KEEP_RUNNING for the W1 pair (operator ruling)
- **Chose:** B — on for W1: both sidecars run under the P3 systemd unit (one instance per agent, `termlink-sidecar@010-termlink--<agent>`, sharing the project's one setting), survive crashes and reboots, store mail while their sessions are closed, and stop only on an explicit project stop. The setting is written by the W1 runme action, not before.
- **Why:** an unnoticed dead sidecar would spoil the 7-day W1 evidence; supervision makes W1 also the first live proof of P3 (R-8.e on the real host) and exercises "mail waits while the session is closed, then arrives". Scored B +47, C −4, A −13.
- **Rejected:** A (off: a crash leaves the agent deaf until the next launch); C (off one day then on: splits the window into two half-weeks).
- **Left open:** whether the setting stays on after W1, reviewed with the W1 results.

### 2026-10-13 — P7-2: how the pair's signing keys are registered, PR-1a (operator ruling)
- **Chose:** A — an operator-held key register on this host now: `runme.sh` records each pair member's public key under `/etc/termlink-sidecar/keys/` (root-owned) and builds both rosters from it; in production a sidecar refuses to start when its key file is missing or its key differs from the register; a key changes only through a runme action that names the change. The hub-side agent card (PR-1a in full, with old-key-signed replacement) moves into row P17's scope.
- **Why:** for two agents on one host under one operator the rosters are the real gate, and a register the operator owns is the trusted source PR-1a asks for; it turns today's silent re-mint (`daemon.rs:97` -> `agent_identity.rs:85-88` creates a new key when the file is missing) into a loud refusal, without hub changes and a fleet upgrade in front of W1. Scored A +48, B +37, C −13.
- **Rejected:** B (hub card now: all of PR-1a, but hub change + fleet upgrade before W1); C (defer: leaves the silent re-mint).
- **Left open:** hub card and old-key-signed replacement in P17; until then a replacement is an operator runme approval; revocation per OQ-h.

### 2026-10-13 — P7-1: which two agents form the W1 pair (operator ruling)
- **Chose:** A — `claude-termlink` (the main session, relaunched through the launcher at W1 start) and `claude-termlink-alt` (a second Claude session the operator opens in /opt/termlink through the launcher).
- **Why:** W1 must show mail reaching a real working agent; the main session is the realest traffic and a failure surfaces where the operator already looks. The risk is bounded: the hook is a silent no-op in any session started without `TERMLINK_SIDECAR_SOCKET` (`hook.rs:84`) and fails open, and the kill switch stops delivery within one tick. Scored A +33, C +15, B +11. Brief and gap map: `docs/reports/T-3426-p7-gapmap.md`.
- **Rejected:** B (separate `-alt` + `-verify` pair, main untouched: little real traffic, two extra terminals for a week); C (main + headless `claude -p` worker: never exercises the idle/ready lifecycle).
- **Left open:** P7-2 key registration, P7-3 keep-running for the pair; the nine gap-map choices taken by default are listed for override after P7-3.

### 2026-10-13 — R-37 escalation threshold for routine needs-attention entries (operator ruling, after external consult)
- **Chose:** A′ — per kind host-wide (sidecar attribution kept); >5 routine/24 h highlight, >15/24 h (3x H-20) escalates; 3 occurrence clusters >=24 h apart within 14 days escalates, never reset by "seen"; oldest-unresolved age shown; own log event type and own `/canaries` line; one task per kind + umbrella when >5 kinds escalate at once, kinds within 1 h linked as one incident; clears after 7 quiet days AND task completed, re-files if closed while firing; all numbers are settings retuned after W1.
- **Why:** both external reviewers (Codex gpt-6-astra, GLM glm-5.3; `docs/reports/T-3426-r37-consult/`) backed the dual-trigger shape and rejected B (too slow for mail) and C (misses a daily trickle); A′ adopts their corrections (host-wide unit, fire at 3x the 14-day average not at it, cluster counting, weekly-cadence window, clear rule, no canary conflation). Re-scored A′ +51, C +20, B +15, D −5.
- **Rejected:** original A (per-sidecar, 5/24 h fire line, no clear rule), B, C, D.
- **Corrections recorded:** per-sidecar unit, treating the 14-day average as a daily cap, scoring D as a dark sink, no clear rule, escalation shown as a canary failure — all from the reviewers, quoted in the brief.
- **Left open:** Codex's 72 h unresolved-age trigger (needs per-kind recovery signals); how escalations reach the cockpit. Reviewer disagreement resolved toward GLM on the fire line (15) and window (14 d), toward Codex on cluster counting; task granularity per kind (GLM) over per incident (Codex).

### 2026-10-13 — OQ-i: whose account runs the daily canary (operator ruling)
- **Chose:** A — a dedicated canary pair as the unprivileged `testdev` (own Claude login), on the installed artifact, production-mode sidecars under a testdev-owned root, the real hub; negative control; daily cron installed by runme; staleness checked. The working agents' own path is measured by the H-14 soak on real traffic, not by injected canary messages.
- **Why:** the set-up that passed the PH-1 acceptance (X4: no production credential, inbox, hub or pin touched), now daily; unprivileged, so a canary bug cannot touch root's state (T-3425 lesson); independent of whether working agents are active. Scored A +61, B +36, C +12.
- **Rejected:** B canary as root on the operator's login (daily unattended job holding root state and the operator's login), C canary driving the real working agents as root (spends their turns/context, fails on idle agents until the doorbell exists).
- **Correction recorded:** OQ-3 ("unprivileged testdev with its own login") was never written as ruled though the operator provisioned testdev on 2026-10-12 and the acceptance ran as it; now recorded ruled A in the PH-1 architecture.
- **Left open:** a light probe rotated through the real pairs once the doorbell exists (W2 or later; R-66 "rotated across agent pairs" is met only within the canary pair until then); which subscription testdev's login is billed to (operator knows).

### 2026-10-13 — OQ-h: revoking an agent key in PH-2 (operator ruling)
- **Chose:** A — the operator revokes through the hub's root-only local socket on the hub host via a runme action (root on the hub host stands in for the operator key until K, PH-3); the hub signs the revocation list with its signing key (P10) and serves it as signed, numbered hub state (not a topic: topics are retention-bound and sweepable); self-revocation by the agent's project with a statement signed by the old key stays; every revocation is logged and raises a needs-attention entry; sidecars pull each tick with SI-11's refusals.
- **Why:** the only option closing every RK-1 case (including a misfiled/lost key) in PH-2 without a new key; root-on-host is today's trust anchor for the hub's secret and certificate. Scored A +52, C +18, B −8, D −39 (D1-D4, F-AUTONOMY, arc AS_* drivers).
- **Rejected:** B self-revocation only (lost key unrevocable until PH-3), C interim operator revocation key (an unrevocable key with no confirmation), D move RK-1 to PH-3 (changes the PH-2 floor).
- **Correction recorded:** the list signer (hub signing key, FP 6.2.3.a(4)) and who may revoke (TM RK-1) were already decided; only the operator's path before K was open.
- **Left open:** replace root-on-host by the operator key in PH-3; whether a revocation also requires the operator's confirmation once a confirmation provider exists. Known weakness accepted under OQ-b: root agents could also revoke; visible via log and needs-attention.

### 2026-10-13 — OQ-e: fleet kill switch (operator ruling)
- **Chose:** A — `termlink sidecar admin delivery off|on --why [--all]`; `--all` walks the host's listed admin sockets, flips each, reads the state back, reports per agent, exits non-zero if any failed; a runme action.
- **Why:** no new shared state; each flip logged in the sidecar where it happened; every flip verified.
- **Rejected:** B one fleet flag file (new shared state, one log, permissions across users in PH-3).

### 2026-10-13 — OQ-j: drained mail on rollback (operator ruling)
- **Chose:** A — kept in the store and listed (sender, conversation, id) in the rollback report; the operator reads or forwards what matters. Revisit before W4.
- **Why:** re-post with the original signature is impossible (T-3429 finding: the signature binds the topic; AEF's receiver does not read the format); W1-W3 are one pair of own agents and a rollback is a drill.
- **Rejected:** B forward as new provenance-quoting messages (a second rewrapping path), C read-only sidecar after rollback.

### 2026-10-12 — OQ-d: the tick (operator ruling)
- **Chose:** A — tick = 30 s (R-17), the unit of every PH-2 bound; the 500 ms loop stays inside it for pulling, hand-over and the kill switch.
- **Why:** keeps the signed-off requirement and today's delivery speed; one countable unit makes "within one tick" provable; simulated clock keeps tests fast.
- **Rejected:** B poll = tick (bounds 60x tighter, revocation pull twice a second), C shorter tick (changes R-17).

### 2026-10-12 — OQ-c: when a message is STUCK (operator ruling)
- **Chose:** A — R-44.e as written: not accepted 1 min; not handed over 2 ticks after the agent becomes ready; not answered 1 h (per-message override); urgent 15 s / next tool call or turn end / 5 min; each only while the responsible side is reachable; STUCK names the step. Replaces PH-1's single `STUCK_AFTER_MS` (10 min from STORED). Row P2.
- **Why:** the signed-off requirement; counting from readiness removes false STUCK on a busy agent and detects a ready-but-unserved agent in about 1 min.
- **Rejected:** B keep 10 min (slow and false alarms; GP-12 bounds assume 1 min), C looser numbers (changes R-44.e).

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

### 2026-10-14T19:22Z — W1 started; first real round trip [agent]
- **Provisioned:** runme 2026-10-14T17:16Z rc=0 (action 17): both sidecars in production mode under systemd, instances `claude-termlink.w1-20261014`, `claude-termlink-alt.w1-20261014`. Units were started but not enabled; the backstop enabled them (2 `supervisor-backstop` notes) and `sidecar-supervise.sh start` now enables (commit 367876577).
- **`-alt` joined** through `tl-claude.sh start --name claude-termlink-alt --sidecar --agent-id claude-termlink-alt` (the first suggested command lacked `--name`, ran from another project dir and carried `--resume`; corrected).
- **Message 2b896306…:** SENT 17:23:57.060Z, RECEIVED .269Z, STORED .391Z (0.33 s), WAITING FOR RECIPIENT while `-alt` idled, HANDED_OVER 19:05:29.648Z at `-alt`'s first tool call, REPLIED 19:21:27.961Z after operator approval (`-alt` declined to act on the peer's reply request alone, as it should). Reply b2bb93c5… STORED at the main sidecar, to be handed over when the main session joins.
- **Observed:** `claude` with no arguments opens Claude Code's session manager; prompts become background task sessions, each carrying the sidecar socket and receiving mail.
- **Open in W1:** main session joins at its next natural restart; kill-switch drill; 7 clean days with both agents in.

### 2026-10-12T19:17:19Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3426-arc-011-ph-2-planning-architecture--buil.md
- **Context:** Initial task creation
