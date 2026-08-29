---
id: T-2855
name: "Template-planted required field makes the inception gate incapable of failing"
description: >
  Template-planted required field makes the inception gate incapable of failing

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [scripts/check-planted-default-gate.sh, tests/planted-default-gate-fixtures.sh]
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
created: 2026-08-29T14:14:16Z
last_update: 2026-08-29T14:23:47Z
date_finished: 2026-08-29T14:23:47Z
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

# T-2855: Template-planted required field makes the inception gate incapable of failing

## Context

832-Workflow-designer named this class on agent-chat-arc @769 and asked us to sweep for
it: **a field that is BOTH required by a gate AND pre-filled by a template converts a
presence gate into a gate that is structurally incapable of failing — it can only ever
confirm the default it planted.**

It reproduces here in full, on all four links:

1. `.tasks/templates/inception.md` plants `voi_score: 0.5` AND `target_blast_radius: 3`.
2. `check-inception-schema.py:86` requires `voi_score` present and in 0..1. `0.5` passes.
3. `estimator.py::_score_inception_voi` makes that single field THE composite for all nine
   drivers — and `int(round(0.5*5))` is 2 under banker's rounding, the SAME value the
   `voi is None` grandfathered branch returns at line 2258. Both branches return 2; only
   the evidence string differs, and the ranking table prints neither.
4. The gate's own remediation argues with itself: line 163 says "so the BVP estimator can
   rank them" and line 172 hands the author `voi_score: 0.5` as the worked example.

Measured on this tree (`templates/` excluded — that exclusion IS the census trap from
@764, and 832 hit it first with three template files in their count):

    inception tasks .................... 203
    voi_score = planted 0.5 ............  61
    voi_score deliberately set .........   1   (T-2838, 0.9 -> 4, completed)
    voi_score absent (pre-T-2188) ...... 141   -> grandfathered branch, also 2

So **202 of 203 inceptions score 2 on every driver**, and every ACTIVE one ranks
identically. Nothing is red anywhere and the ranking output carries no information.

All three defective links are VENDORED (`.agentic-framework/`), so per G-062 they are
filed upstream rather than patched — a local edit is one `fw upgrade` from deletion.
What ships HERE is the detection that survives a re-vendor: the same division of labour
T-2854 used.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `scripts/check-planted-default-gate.sh` exists, carries a `# guard-layer:` marker, and implements the exit contract 0=clean / 1=firing / 2=tooling, fail-closed (a missing template dir or empty task corpus exits 2, never a vacuous clean)
- [x] Against the REAL tree with an empty ledger it fires on both `voi_score` and `target_blast_radius`, naming the domination ratio — load-bearing against real state, not only against fixtures
- [x] `tests/planted-default-gate-fixtures.sh` passes, and includes at least one fixture shape that does NOT occur in our corpus (577 @766 / 832 @769: red-then-green on real state proves the discriminator handles the instance, never the class)
- [x] A mutation that weakens the discriminator is caught BY THE FIXTURES — recorded in this task with the observed failure text, not asserted
- [x] `.context/checks/planted-default-gate-allowlist` is git-tracked (T-2681) and each entry carries a cited reason; the two exposed fields are acknowledged pending the upstream fix so the check runs clean while staying a visible ledger
- [x] The vendored defects are filed at `framework:pickup` (G-062) with the measurement, and the offset is recorded here — **filed at offset 71**, `metadata.from_project=010-termlink` so the T-2816 self-filter keeps it from firing our own pickup canary
- [x] `bash scripts/run-guard-layer.sh` reports the new member PASS and no new ERROR
## Mutation evidence (observed, not asserted)

Two mutations, each restored afterwards and the suite re-run green (26/26).

MUTANT 1 — domination gate removed (`if ratio < THRESHOLD:` -> `if False:`), so mere
exposure fires:

    FAIL B1 a spread field does not fire — expected rc=0 got rc=1
    FAIL J3 real tree is clean with the tracked ledger — expected rc=0 got rc=1
    planted-default-gate-fixtures: 24 passed, 2 failed

MUTANT 2 — min-population floor removed (`if pop < MIN_POP:` -> `if False:`):

    FAIL F1 100% domination over 4 tasks does not fire (min-pop) — expected rc=0 got rc=1
    planted-default-gate-fixtures: 25 passed, 1 failed
    real-tree rc=0

**Mutant 2 is the load-bearing one.** It is caught by exactly ONE fixture — F1, an
INVENTED shape that does not occur anywhere in this corpus — and the real tree stays
GREEN through it. That is 577 @766's "red-then-green on real state is necessary and not
sufficient" and 832 @769's "a corpus-only test stays green through that mutation",
reproduced independently on this instrument the same day. Had the suite been built only
from real state, the min-population floor could have been deleted silently.

## Near-miss while writing this task (recorded because it is the same class)

Writing the Verification block, a `s.index('## Verification')` matched the FIRST
occurrence of that string — inside the Human-AC comment block, where the template quotes
it as prose — and truncated the file there, gluing the header onto the end of a comment
line. Result: no `## Verification` header at column 0, so **P-011 would have extracted
ZERO commands and passed vacuously**, exactly the T-2830/T-2831 defect.

Caught by running the block by hand first: it reported `VERIFICATION_FAIL=0` while
printing no PASS lines at all. A runner that executes nothing and reports no failures is
indistinguishable from one that passed, which is the whole T-2831 finding.

Worth noting what did NOT catch it: `check-verification-misfile.sh` detects commands in
the WRONG section, and these were in NO section (inside an unclosed HTML comment). Its
own documented residual gap — "a task with an empty `## Verification` passes this check
and still gates on nothing" — is precisely this, and this is a live instance of it rather
than a hypothetical. Not fixed here; recorded so it is evidenced when someone closes it.

## Verification

# Safe idioms only: file-redirect then grep the FILE. Never `cmd | grep -q` (L-387),
# and never `echo "$out" | grep -q` for output that may exceed the 64KiB pipe (T-2852).

test -x scripts/check-planted-default-gate.sh
grep -q '^# guard-layer:' scripts/check-planted-default-gate.sh

# The check is clean against the tracked ledger...
bash scripts/check-planted-default-gate.sh > /tmp/.t2855-clean.out 2>&1
grep -q 'clean — 0 unacknowledged' /tmp/.t2855-clean.out

# ...and both acknowledged fields stay VISIBLE on the clean path, never silenced.
grep -q 'acknowledged: voi_score' /tmp/.t2855-clean.out
grep -q 'acknowledged: target_blast_radius' /tmp/.t2855-clean.out

# LOAD-BEARING: with an empty ledger it must actually fire on the real tree.
PLANTED_ALLOWLIST=/dev/null bash scripts/check-planted-default-gate.sh > /tmp/.t2855-fire.out 2>&1 || true
grep -q 'voi_score' /tmp/.t2855-fire.out
grep -q 'target_blast_radius' /tmp/.t2855-fire.out

# Fail-closed: a missing template dir must be rc=2, never a vacuous clean.
bash scripts/check-planted-default-gate.sh --templates /nonexistent-xyz > /tmp/.t2855-fc.out 2>&1; test $? -eq 2

# Fixtures, including the shapes absent from our corpus and the negative control.
bash tests/planted-default-gate-fixtures.sh > /tmp/.t2855-fix.out 2>&1
grep -q '26 passed, 0 failed' /tmp/.t2855-fix.out

# The ledger is git-TRACKED (T-2681) — an untracked allowlist makes a guard's green
# depend on unversioned local state, the defect T-2681 exists to prevent.
git ls-files --error-unmatch .context/checks/planted-default-gate-allowlist > /dev/null 2>&1

# Both members are adopted by the guard-layer runner.
bash scripts/run-guard-layer.sh --list > /tmp/.t2855-list.out 2>&1
grep -q 'check-planted-default-gate.sh' /tmp/.t2855-list.out
grep -q 'planted-default-gate-fixtures.sh' /tmp/.t2855-list.out

## Reviewer Verdict (v1.5)

- **Scan ID:** R-56fe7305
- **Timestamp:** 2026-08-29T14:23:52Z
- **Catalogue:** v1.3-seed
- **Overall:** FAIL
- **Needs Human:** no
- **Findings:** 2

**Verification-level findings:**

  1. **swallowed-errors** (severe, deterministic) @ Verification:line 16
     - evidence: `PLANTED_ALLOWLIST=/dev/null bash scripts/check-planted-default-gate.sh > /tmp/.t2855-fire.out 2>&1 || true`
  2. **empty-output-success** (partial, heuristic) @ Verification:line 29
     - evidence: `git ls-files --error-unmatch .context/checks/planted-default-gate-allowlist > /dev/null 2>&1`

### 2026-08-29T14:23:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
