---
id: T-2831
name: "Repair T-2830 verification block landed under Evolution so P-011 ran nothing"
description: >
  Repair T-2830 verification block landed under Evolution so P-011 ran nothing

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-23T11:26:48Z
last_update: 2026-08-23T18:59:45Z
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

# T-2831: Repair T-2830 verification block landed under Evolution so P-011 ran nothing

## Context

T-2830 completed reporting **"Acceptance criteria: 8/8 checked ✓"** and printed no
verification run at all. Its `## Verification` heading (line 83 of the completed file) is
followed only by the template's comment block; the six real commands I wrote landed at
lines 155–166, under **`## Evolution`**. P-011 extracts commands from the `## Verification`
section, found none there, and passed **vacuously**.

This is the exact defect class this session has been reporting all day — a check that
asserts a property adjacent to the one it claims — committed by me, one commit after
writing it up. The tell holds: *if the check would still pass while the subsystem is
entirely broken, it is the wrong check*. P-011 would have passed here even if the merge had
been garbage.

**The evidence is not missing — only the gate is.** Every one of the six commands was run
by hand during T-2830, before completion, and passed:

| command | result |
|---|---|
| `git rev-parse --verify integration/t2687-trial` | rc=0 |
| `git merge-base --is-ancestor origin/main integration/t2687-trial` | rc=0 |
| `git grep -l '^<<<<<<< '` | 0 files |
| `git diff --quiet origin/main -- crates/termlink-mcp/src/tools.rs` | rc=0 |
| `cargo build --release` | rc=0 |
| `bash scripts/verify-register-union.sh` | rc=0, 4 registers, nothing lost |

plus `cargo test --release` (2,944 passed / 0 failed) and all 60 `tests/*.sh` suites. So
T-2830's claims are **true**; what is false is the implication that a gate proved them.
Those two are worth keeping apart, which is the whole point of the gate existing.

**Why the framework was blind (G-019).** A misfiled block is silent in both directions: the
section renders identically in the task file, and P-011 reports nothing when it finds
nothing — "no commands to run" and "all commands passed" are the same output. The
`## Verification` template is a long comment block, so commands appended near the *end* of
a template section land visually plausibly but structurally wrong. Nothing checks that a
task's shell commands are in the section that executes them.

The prevention is the checker in AC#3: shell commands under any heading other than
`## Verification` are almost always a misfile, and it is mechanically detectable.

**Deliberately not done here:** T-2830 was not reopened or re-completed with `--force`. Its
ACs are individually true and independently evidenced above; re-running the gate proves
nothing that this record does not already carry, and forcing a completion to make a gate
look like it ran would be the same dishonesty in the other direction.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] T-2830's verification commands sit under its `## Verification` heading, not `## Evolution`
- [x] Every one of those commands is re-run by hand against the merged tree and passes, so T-2830's 8/8 is backed by evidence rather than by a gate that ran nothing
- [x] A check exists that reports any task file carrying shell commands in a section that is NOT `## Verification`
- [x] That check is proven load-bearing: red against T-2830's broken shape, green after the repair

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
       `bin/fw reviewer T-XXX 2>&1 | grep -q "Overall:.*PASS"` added to ## Verification.
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
# Pipefail/SIGPIPE hint (L-387): P-011 runs each command under `set -eo pipefail`.
# `cmd | grep -q PATTERN` exits 141 (SIGPIPE) when grep matches and closes stdin
# while the upstream is still writing — verification then "fails" even though
# the pattern was present. Safe pattern: capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Or:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
# Origin: L-387, captured 4× (T-1716, T-1838, T-1862, T-1863) before this hint.
#
# Single pipe only — no intermediate tail/awk/sed stages between capture and grep
# (T-2090): `echo "$out" | tail -3 | grep -q PAT` re-introduces the SIGPIPE risk
# the capture step closed off — the middle stage is what `grep -q` slams its
# stdin on. `echo "$out"` is small and immediate; grep scans the whole captured
# string anyway, so the tail-3 was cosmetic. Drop it: `echo "$out" | grep -q PAT`.
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

# The checker exists, is executable, and is a declared guard-layer member
test -x scripts/check-verification-misfile.sh
out=$(bash scripts/run-guard-layer.sh --list 2>&1); echo "$out" | grep -q check-verification-misfile.sh
# GREEN: the repaired corpus scans clean
bash scripts/check-verification-misfile.sh
# RED (load-bearing): it fires on T-2830 as it stood BEFORE the repair, straight from git.
# Pinned to the explicit pre-repair sha, NOT HEAD: HEAD moves when the repair commit
# lands, so a HEAD-relative fixture silently starts fetching the FIXED file and the
# red leg passes for the wrong reason. P-011 caught exactly that here.
rm -rf .tmp-misfile-fixture && mkdir -p .tmp-misfile-fixture/active
git show d2442b603:.tasks/completed/T-2830-pre-resolve-t2687-to-main-integration-on.md > .tmp-misfile-fixture/active/T-2830-prerepair.md
! bash scripts/check-verification-misfile.sh --tasks-dir .tmp-misfile-fixture --quiet
rm -rf .tmp-misfile-fixture
# T-2830 now carries its commands under ## Verification, not ## Evolution
bash scripts/check-verification-misfile.sh --tasks-dir .tasks --quiet
# Fixture suite green
out=$(bash tests/verification-misfile-check-fixtures.sh 2>&1); echo "$out" | grep -q "0 failed"

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

### 2026-08-23T11:26:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/t2687-pickup-failopen/.tasks/active/T-2831-repair-t-2830-verification-block-landed-.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-e07b82f2
- **Timestamp:** 2026-08-23T17:06:03Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** yes
- **Findings:** none

- **Layer-1 escalations:** 1
  1. **destructive-action** (high) — Destructive operation in verification or AC
     - matched: `rm -rf`
