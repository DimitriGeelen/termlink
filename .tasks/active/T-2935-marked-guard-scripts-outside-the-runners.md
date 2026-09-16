---
id: T-2935
name: "Marked guard scripts outside the runner's globs are silently never run"
description: >
  scripts/fabric-workflow-link.sh carries the guard-layer marker but the runner's
  inventory globs (scripts/check-*.sh, scripts/test-*.sh, tests/*.sh) do not match
  it, so it is never run, never listed by --list, and never reported as unclassified.
  check-guard-runner-coverage.sh inherits the same globs, so the script is invisible
  to the detector as well as the runner: neither covered nor flagged. A guard that
  declares membership and is silently excluded is the failure class the layer exists
  to prevent. Found by reconciling 113 computed members against 112 executed during
  T-2933.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components:
  - scripts/run-guard-layer.sh
  - scripts/check-guard-runner-coverage.sh
  - tests/guard-layer-runner-fixtures.sh
  - tests/guard-runner-coverage-fixtures.sh
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
created: 2026-09-09T07:44:22Z
last_update: 2026-09-16T17:21:07Z
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
  - ts: '2026-09-09T07:46:30Z'
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
cost_estimate_proposed:
  - ts: '2026-09-09T07:46:37Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=207,acs=7)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-09T07:47:26Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 5
      tier: 2
      effort: 8
    rationale: blast_radius=5 (4-components-medium-blast); tier=2 
      (workflow:build); effort=8 (lines=207,acs=7)
    rubric_sha: e4a00f38e801
---

# T-2935: Marked guard scripts outside the runner's globs are silently never run

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## State at park (session S-2026-0909, scored — execution not begun)

Parked **before any execution**, deliberately, on stop condition 2 (context ~241k of the
mandate's ~300k). The task is Q2 (hv-hc, BVP 57 / cost 4.4) with effort=8 and five ACs
including a runner contract change and a ~9 min full-layer comparison. That does not fit
the remaining budget, and the mandate forbids stopping mid-task — so it is parked whole
rather than started and abandoned half-done.

**What is already done:** the finding is measured and reproduced (T-2933 close-out), the
task is created, real ACs are written, `components:` is wired, and both axes are scored
through the estimator — value D1=4 D2=0 D3=3 D4=2 (BVP 57), cost blast_radius=5 tier=2
effort=8 (4.4). Nothing in `scripts/` or `tests/` has been touched.

**What the next session should NOT re-derive:** the defect is understood. `run-guard-layer.sh`
computes membership as (marker ∩ three name globs), while CLAUDE.md documents membership as
the marker alone. `scripts/fabric-workflow-link.sh` is the known casualty. AC1 exists because
one instance is not the class — measure the full excluded set first.

**Calibration note fed back:** cost was `blast_radius=None` until `components:` was populated
by hand, and `fw bvp` reports **132/159 tasks (83%) have no cost at all** for the same reason.
The estimator is honest about it (`no-components-UNMEASURED-not-zero`) rather than scoring
missing as cheap — but it means the quadrant system currently ranks over 17% of the backlog.
That is T-3068 and it is a live distortion of any quadrant-driven selection, including this run's.

## AC1 result (session S-2026-0909b) — measured, closed

**Excluded set size: exactly 1.** `scripts/fabric-workflow-link.sh`, introduced by
`6244af60c` (T-2839, 2026-08-27). AC1 existed because one instance is not the class; the
class is measured at 1 today, but the *mechanism* still permits N — any future marked file
whose name does not match `check-*.sh` / `test-*.sh` under `scripts/`, or which lives
outside `scripts/`+`tests/`, is excluded the same silent way.

Method, and a correction worth carrying: the first measurement compared the 53 marked files
against **all 187 names** `--list` prints, and was unsound — `--list` emits the 112 members
*and* the 75 unmarked non-members, so a marked file sitting in the non-member section would
have been counted as enumerated and the excluded set under-reported. Re-measured against the
member block alone (parsed 112, matching the run) the answer held at 1, but it held for a
checked reason rather than a lucky one.

**The fix is de-risked: the excluded script PASSES.** Run standalone: `rc=0` in **1014 ms**.
So making membership marker-authoritative adds one fast, green member — it does not turn the
layer red, and AC5's "no new failure" should be satisfiable. This was the single most useful
thing to know before touching the runner, which is why it was measured before parking.

**Remaining ACs (2-5) not started.** No file under `scripts/` or `tests/` has been modified.
Parked on context (~262k of the mandate's ~300k). AC2 changes the runner's membership
contract and AC5 needs a ~9 min full-layer comparison; beginning that with ~38k left would
leave the runner altered and its verification unread — reliable-but-ungated, the state the
mandate names as the dangerous one.

## AC2-AC4 result (session S-2026-0916c) — implemented and pinned

**AC2 — the runner is now marker-authoritative.** `run-guard-layer.sh` gains a fourth
discovery pass over `"$SCRIPTS_DIR"/*.sh` (skipping `check-*`/`test-*`, already enumerated
by the name-glob passes), marker-gated. Membership now matches the contract CLAUDE.md has
always stated — "Membership is declared, not guessed. A static check joins the layer by
carrying a marker in its own header" — rather than a name convention that silently
overrode it. `TESTS_DIR` needed no equivalent pass: the third loop already globs
`"$TESTS_DIR"/*.sh` whole, which is why the hole was scripts-only.

Measured before and after, on this tree: **112 → 113 members**, newly enumerated set is
exactly `["fabric-workflow-link.sh"]`, lost set empty, **unclassified unchanged at 75**.
That last number matters beyond bookkeeping: the detector's INVENTORY AGREEMENT gate
exits 2 on any unclassified-count drift between the two, so an implementation that widened
the bucket would have turned its own auditor into a hard error.

**The unclassified bucket is deliberately NOT widened, and that is a judgement worth stating.**
`scripts/` holds 190 `.sh` files, the large majority operator tooling rather than guards.
Reporting every unmarked one would add ~130 entries to a bucket whose meaning is precisely
"looks like a guard but forgot its marker" — alarm fatigue manufactured inside the accounting
layer, the T-2818 failure shape. A name-shaped candidate that forgets its marker is still
caught as unclassified by the `check-*`/`test-*` passes, so nothing is lost.

**AC3 — the detector's blind spot is closed at the invariant, not at the globs.**
`check-guard-runner-coverage.sh` mirrored the runner's legacy name globs, so the excluded
script was in neither bucket: not `unclassified` (it carries a marker), not `covered`, not
`dormant`. Invisible to the guard *and* to the guard's own auditor — the specific thing the
task name describes.

The fix does **not** re-implement the runner's new globs, which would only move the copy.
It widens the MARKED scan to every `scripts/*.sh`, then compares that set against the
runner's **actual `--list --json` member names**, reporting any difference in a new firing
class `unenumerated`. Keying on the invariant ("declared membership is honoured") instead of
on a copy of the enumeration rules means the check stays load-bearing if the globs change
shape again — the same reason the existing agreement gate compares counts with the authority
rather than trusting its own.

Worth naming explicitly: **agreeing on the unclassified count never proved this.** The two
can agree perfectly about what is unmarked while the runner silently drops a marked file,
which is exactly what happened for the 20 days between `6244af60c` and now.

**A correctness bug found while wiring the output.** The firing block ends with disposition
advice that tells the operator to *add the `# guard-layer: source` marker* — actively wrong
for a script that already carries one, which is the entire `unenumerated` class. It is now
gated on `n_dormant > 0`, so an unenumerated-only firing cannot hand out an instruction that
would not help.

**AC4 — pinned by mutation, in both suites.** The load-bearing leg mutates the **runner**,
not the check: a copy with the new pass's glob neutered to `__no_such_glob__*.sh` reproduces
pre-fix behaviour exactly, leaving the three legacy loops intact. Against it the check FIRES
(rc 1) and names both marked-but-dropped scripts; against the fixed runner the class is 0.
Mutating the check instead would have proved only that the assertion runs.

- `tests/guard-runner-coverage-fixtures.sh`: 21 → **28 passed, 0 failed**
- `tests/guard-layer-runner-fixtures.sh`: 40 → **46 passed, 0 failed**, adding the runner-side
  contract (a marked `scripts/*.sh` outside the globs joins, is named in `--list`, is executed
  by a full run) and its false-positive guard (an UNMARKED `scripts/*.sh` is neither a member
  nor unclassified — the ~130-entry flood must not happen).

One incidental fixture-harness defect fixed: the M3 mutant block leaves `set -e` on, so the
first new check invocation — which legitimately exits 1 on a tree with dormant scripts by
design — aborted the suite before the summary line. The suite reported `rc=1` with no
failures printed, which reads as a broken run rather than a failing assertion.

## AC5 result (session S-2026-0916c) — full-layer run compared to the T-2933 baseline

| | T-2933 baseline | after this change |
|---|---|---|
| members | 112 | **113** (+1, exactly the newly enumerated set) |
| PASS | 109 | **109** |
| ERROR | 0 | **0** |
| FAIL | 3 | **4** |

**The newly enumerated member passes:** `fabric-workflow-link.sh` → `rc 0`, `verdict PASS`,
confirming the AC1 standalone measurement (rc 0 in 1014 ms) holds inside the runner.

**PASS held at 109 while members rose by 1, and that arithmetic is the actual finding.**
Adding a passing member should have read 110. It did not, because a *different* member moved
PASS → FAIL in the same window: 109 + 1 (new, passing) − 1 (regressed) = 109. The two changes
cancel exactly, so the headline counts alone would have concealed both. Reading the per-member
verdicts rather than the summary is what separated them.

**The fourth FAIL is not this change.** `check-pickup-deferred-freshness.sh` was already a
member before the change (verified against the pre-change `--list --json`), so this task's
enumeration edit cannot have introduced it. It fires on **host state**: four envelopes in
`.context/pickup/auto-deferred/` carry no breadcrumb, which per T-2801 makes them unpromotable
by construction — `fw pickup promote-deferred` has no blocking task to resolve and
`fw pickup auto-deferred list` prints `blocked-by=?` while reporting nothing wrong.

Two of the four are pointed, given this lineage's recent history: **P-075** reports that the
pickup processor mints local tasks from a project's OWN filings — the same defect measured and
filed upstream at `framework:pickup` offset 124 — and **P-077** is a correction to P-076
concerning T-2882. Both had been sitting unread in the queue the whole time.

Filed as **T-2965** (P-074, P-075, P-077; P-078 is already T-2960's) rather than fixed here.
Dispositioning inbound peer filings is not this task's scope, and per the mandate the
verification of that finding is the next cycle's audit, not an assertion in this file.

**The three baseline FAILs are unchanged and unaddressed by this task:**
`check-installed-binary-drift.sh`, `check-receiver-ack-lag.sh`, `cron-drift-firing-fixtures.sh`.

The claim this AC makes is therefore the narrow one it was written to make — *this change added
no failure* — not *the layer is green*. It is not.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] **The excluded set is measured and named before anything changes.** Every file under `scripts/` and `tests/` carrying `# guard-layer: source` that the runner's current inventory does NOT enumerate is listed with its path and the commit that introduced it, and the count is stated. `fabric-workflow-link.sh` is one known member; the task does not assume it is the only one. A fix sized to one instance when the class has N is the recurring shape this repo keeps finding (T-2667, T-2673).
- [x] **The runner's membership is made marker-authoritative, matching its own documented contract.** CLAUDE.md states "Membership is declared, not guessed. A static check joins the layer by carrying a marker in its own header." The implementation instead intersects the marker with three name globs, so a marked file outside them is silently excluded. After the change `--list` and a full run both enumerate every marked file under the scanned roots. Each newly-included script is run and its verdict recorded — a marked script that does not pass is reported, never quietly dropped to keep the layer green.
- [x] **The blind spot in `check-guard-runner-coverage.sh` is closed too.** The detector inherits the same globs, so an excluded script is today neither `covered` nor `unclassified` — invisible to the guard *and* to the guard's own auditor. After the change a marked-but-unenumerated script is reported in a named class rather than absent from every bucket. Verified against a fixture tree containing one.
- [x] **A fixture pins the defect and is load-bearing.** A fixture tree containing a marked script outside the legacy globs FAILS against the pre-fix inventory logic and PASSES after, proving the fixture detects the regression rather than merely passing. Asserted by mutation, not by inspection.
- [x] **No new failure is introduced, measured against the T-2933 baseline.** A full `run-guard-layer.sh` run is compared to that baseline (112 members, 109 PASS, 0 ERROR, 3 pre-existing host-state FAILs: `check-installed-binary-drift.sh`, `check-receiver-ack-lag.sh`, `cron-drift-firing-fixtures.sh`). Member count changes by exactly the number of newly-enumerated scripts, stated before and after. Any FAIL beyond the 3 is either fixed or recorded as a finding with its cause — the claim is "this change added no failure", not "the layer is green".

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

# ---- T-2935 ----
# Both suites green, including the load-bearing mutation legs.
bash tests/guard-layer-runner-fixtures.sh > /tmp/.t2935-v-rf.out 2>&1
grep -q "46 passed, 0 failed" /tmp/.t2935-v-rf.out
bash tests/guard-runner-coverage-fixtures.sh > /tmp/.t2935-v-cf.out 2>&1
grep -q "28 passed, 0 failed" /tmp/.t2935-v-cf.out
# AC2: the runner enumerates the previously-excluded marked script, and --list names it.
bash scripts/run-guard-layer.sh --list --json > /tmp/.t2935-v-list.json 2>&1
python3 -c "import json,sys; d=json.load(open('/tmp/.t2935-v-list.json')); n=[m['name'] for m in d['members']]; sys.exit(0 if 'fabric-workflow-link.sh' in n and len(n)==113 else 1)"
# AC2 false-positive guard: the unclassified bucket did NOT absorb the ~130 unmarked
# general scripts. 75 is also what the detector's INVENTORY AGREEMENT gate requires.
python3 -c "import json,sys; d=json.load(open('/tmp/.t2935-v-list.json')); sys.exit(0 if d['summary']['unclassified']==75 else 1)"
# AC3: the detector reports the new class, and this tree has none of it.
bash scripts/check-guard-runner-coverage.sh --json > /tmp/.t2935-v-cov.json 2>&1 || true
python3 -c "import json,sys; d=json.load(open('/tmp/.t2935-v-cov.json')); sys.exit(0 if d['summary']['unenumerated']==0 and 'unenumerated' in d else 1)"
# AC3: scope line no longer claims a narrower question than the check now answers.
python3 -c "import json,sys; d=json.load(open('/tmp/.t2935-v-cov.json')); sys.exit(0 if 'enumerate every MARKED script' in d['scope'] else 1)"
# Both edited guards still parse.
bash -n scripts/run-guard-layer.sh
bash -n scripts/check-guard-runner-coverage.sh

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

**Symptom:** `scripts/fabric-workflow-link.sh` carried `# guard-layer: source` and was
never executed by the guard layer, never printed by `--list`, and never reported as
unclassified — for the 20 days between `6244af60c` (T-2839, 2026-08-27) and this fix. It
was absent from every bucket, so no output anywhere said anything about it, true or false.

**Root cause:** `run-guard-layer.sh` computed membership as *(marker ∩ three name globs)*
— `scripts/check-*.sh`, `tests/*fixtures*.sh`, `scripts/test-*.sh` + `tests/*.sh` — while
CLAUDE.md, the runner's own `--help`, and its header comment all document membership as the
**marker alone** ("Membership is declared, not guessed"). A marked file under `scripts/`
named neither `check-*` nor `test-*` satisfies the documented contract and matches no glob,
so it is dropped silently. The glob was never a second condition anyone decided to impose;
it is the residue of the marker having been added later to a name-based inventory.

**Why structurally allowed:** the auditor inherited the same globs from the thing it audits.
`check-guard-runner-coverage.sh` scanned `check-*.sh test-*.sh tests/*.sh` and sorted each
file into `marked` or `unclassified` — so a marked file outside those globs was in neither,
and no bucket was empty in a way anyone would notice. Its one cross-check against the
authority compares the **unclassified count**, which agreed perfectly the whole time:
both sides shared the identical blind spot, so agreement was evidence of nothing. This is
the shape this repo keeps re-finding — a guard that asserts a property *adjacent* to the one
it claims (T-2831), and a green that is read as a full bill of health (T-2680). It surfaced
only by reconciling 113 computed members against 112 executed in T-2933, i.e. by someone
comparing two numbers that were never supposed to differ.

**Prevention** (distinct from the fix): the detector now compares the **marked set against
the runner's actual `--list --json` member names** and reports any difference as a firing
`unenumerated` class. That keys on the invariant — *declared membership is honoured* —
rather than on a copy of the enumeration rules, so it survives the globs changing shape
again, which is precisely how the first version failed. Pinned by mutating the **runner**
(the new pass neutered to a glob that matches nothing) so the fixture proves the check
detects a regressed runner, not merely that an assertion executes. A marked script that is
dropped by any future enumeration change now fires by name instead of vanishing.

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

### 2026-09-16 — the auditor's blind spot was the harder half, and it is not glob-shaped
- **What changed:** at filing, the defect read as "the runner's globs are too narrow" and the
  detector's blindness read as a consequence of inheriting them. Reading both scripts showed
  the auditor's real weakness is one level up: its single cross-check against the runner
  compares the **unclassified count**, and that number agreed perfectly for all 20 days. Two
  components sharing a blind spot produce agreement, and agreement was being read as
  verification. Widening globs on both sides would have restored agreement without restoring
  the property.
- **Plan impact:** AC3 ("the detector inherits the same globs") is satisfied, but not by the
  implementation its wording implies. The detector does not mirror the runner's new glob; it
  diffs the marked set against the runner's real member list. Stated here because a future
  reader comparing AC text to the diff would otherwise find them apparently inconsistent.
- **Triggered:** no new task. The unclassified-count agreement gate is kept as-is — it is
  still the right check for its own question, it was simply never the check for this one.

### 2026-09-16 — a wrong instruction inside a firing message
- **What changed:** the detector's firing output ends with disposition advice whose final
  option is "add the `# guard-layer: source` marker". For the new `unenumerated` class that is
  precisely backwards — every member already carries one. Not a cosmetic issue: a firing guard
  that names the wrong remedy sends the operator to change the one thing that is correct.
- **Plan impact:** none to the ACs; folded into AC3's implementation and gated on `n_dormant`.
- **Triggered:** nothing filed. Recorded because it was found by writing the output rather than
  by any assertion — no fixture would have caught advice that is merely wrong.

### 2026-09-16 — the fixture harness reported a broken run as a failing one
- **What changed:** the M3 mutant block leaves `set -e` enabled. The first new check invocation
  exits 1 by design (the fixture tree contains dormant scripts), so the suite aborted before
  its summary line — `rc=1`, no `FAIL` printed, no count. That is indistinguishable at a glance
  from an assertion failure, and it cost a cycle to tell apart.
- **Plan impact:** none; `set +e` added at the top of the new block, matching the surrounding
  cases' convention.
- **Triggered:** nothing filed — it is one line in one suite. Noted because "no failures printed
  and a non-zero exit" is a shape worth recognising quickly in any of these suites.

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

### 2026-09-16 — the general pass is marker-only; the unclassified bucket stays name-shaped
- **Chose:** the new `scripts/*.sh` pass adds a file to the layer when it carries the marker
  and says nothing at all when it does not.
- **Why:** `scripts/` holds 190 `.sh` files, mostly operator tooling. `unclassified` means
  "looks like a guard but forgot its marker" — a judgement the *name* carries. Extending it
  to every general script would add ~130 entries and destroy that meaning. It would also
  trip the detector's INVENTORY AGREEMENT gate, which exits 2 on unclassified-count drift.
- **Rejected:** report unmarked `scripts/*.sh` as unclassified — symmetrical-looking, and it
  manufactures exactly the alarm fatigue T-2818 documented, inside the accounting layer.

### 2026-09-16 — the detector checks the invariant, not a copy of the runner's globs
- **Chose:** widen only the MARKED scan, then diff it against the runner's actual
  `--list --json` member names, reporting the difference as a firing `unenumerated` class.
- **Why:** the original defect *was* a copy of the globs drifting from the contract. Mirroring
  the new globs into the detector recreates the same coupling one commit later and would go
  stale the same silent way. Comparing against the authority's real output tests the property
  that matters — declared membership is honoured — and survives the globs changing again.
- **Rejected:** mirror the new `scripts/*.sh` glob in the detector. Cheaper, and it would have
  reported clean for exactly as long as the two copies happened to agree.

### 2026-09-16 — the fixture mutates the RUNNER, not the check
- **Chose:** build a pre-fix runner (new pass's glob neutered to `__no_such_glob__*.sh`) and
  point the check at it via `GUARD_COVERAGE_RUNNER`.
- **Why:** AC4 asks whether the fixture detects the regression. The regression lives in the
  runner's enumeration, so the runner is what has to break. Against it the check fires and
  names both dropped scripts; against the fixed runner the class is 0.
- **Rejected:** mutate the check (the suite's existing M1-M3 style). That proves the assertion
  executes, not that it catches the defect — the weaker claim, and the one already covered.

### 2026-09-16 — gate the dormant disposition advice instead of leaving it
- **Chose:** print the "give each dormant script a disposition" block only when `n_dormant > 0`.
- **Why:** it ends by telling the operator to *add the `# guard-layer: source` marker*, which
  is wrong for every member of the new class — those scripts already carry one. Wrong advice
  in a firing message is worse than none: it sends the reader to change the one thing that is
  already correct.
- **Rejected:** leave it unconditional. Found while wiring the output rather than by a failing
  assertion, which is why it is recorded here rather than silently patched.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-09-09T07:44:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2935-marked-guard-scripts-outside-the-runners.md
- **Context:** Initial task creation
