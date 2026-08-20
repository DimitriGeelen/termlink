---
id: T-2814
renumbered_from: T-2689  # T-2823 cross-branch collision
name: "stale gitignore silently untracks new vendored-framework files — fw bvp and
  the whole scoring policy are unrecoverable from git"
description: >
  stale gitignore silently untracks new vendored-framework files — fw bvp and the
  whole scoring policy are unrecoverable from git

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-19T23:46:16Z
last_update: 2026-08-20T18:54:29Z
date_finished: 2026-08-20T18:54:29Z
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
  - ts: '2026-08-20T15:20:37Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=2 (body:env-class-handled); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-08-20T15:21:22Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2814: stale gitignore silently untracks new vendored-framework files — fw bvp and the whole scoring policy are unrecoverable from git

## Context

`.gitignore:21` ignores `.agentic-framework` under the comment "Framework symlink
(machine-specific)". That comment was true once; it is not true now. The framework is
**vendored**, and 1565 files under `.agentic-framework/` are tracked — including
`bin/fw` itself, 103 `lib/*.sh`, 105 `agents/`, 147 `web/`.

Git ignore rules do not apply to already-tracked files, so the tracked 1565 keep
working and nothing looks wrong. But every framework file added *after* the rule landed
(2026-06-11, commit `c73815977`) is silently untrackable: `git add -A` skips it, and
`git status` never mentions it, because ignored files are not reported. The repository
therefore drifts further from the working framework with each upgrade, invisibly.

The concrete casualty found today is the **BVP prioritisation rail** — the mechanism the
operator uses to decide what to work on:

| Path | On disk in the checkout | Tracked |
|---|---|---|
| `.agentic-framework/lib/bvp.sh` | yes | **no** |
| `.agentic-framework/policy/value-drivers.yaml` | yes | **no** |
| `.agentic-framework/policy/bvp-scoring-rubric.md` | yes | **no** |
| `.agentic-framework/agents/termlink/bvp-estimator/*` | yes | **no** |

The entire `policy/` directory has **0** tracked files. Reproduced directly: in a
worktree checked out from `HEAD`, `fw bvp estimate T-2813` fails with
`.agentic-framework/lib/bvp.sh: No such file or directory` — `bin/fw` is tracked and
still routes the `bvp` subcommand, but the library it routes to is not there.

Two consequences, in increasing order of seriousness:

1. A clean clone yields a framework whose documented `fw bvp` subcommand is broken, with
   a raw bash error rather than a diagnostic.
2. `policy/value-drivers.yaml` holds the driver weights that `fw bvp weight --set
   ... --i-am-human` mutates — a **§ACD sovereignty-gated policy decision**. Those
   human-blessed governance decisions, and the rubric that scores every task, exist only
   on this host's disk. They are not recoverable from version control.

This task ships the **detection** half (making the invisible drift visible) and records
the remediation; the remediation itself edits `.gitignore` and adds files that exist only
in the operator's main checkout, so it is deliberately left as an explicit operator step
rather than guessed at from a worktree.

## Acceptance Criteria

### Agent
- [x] `scripts/check-framework-tracking-drift.sh` reports files present under
      `.agentic-framework/` that are unreachable from git
- [x] It FIRES (exit 1) when an untracked file sits under a load-bearing subtree
      (`bin/`, `lib/`, `policy/`, `agents/`), which is what breaks a clean clone
- [x] Generated/cache noise (`__pycache__`, `*.pyc`, `*.pyo`, `.git`) is excluded so the
      signal is not swamped
- [x] `docs/` and `web/` drift is reported as informational, never firing (bulk content,
      not clean-clone-breaking)
- [x] `--json` emits `{ok, firing[], informational_count, checked}` for scripting
- [x] `--quiet` renders only firing entries (cron-friendly convention)
- [x] A fixture test drives firing, informational-only, and clean trees against a scratch
      root and passes
- [x] `bash -n scripts/check-framework-tracking-drift.sh` passes

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

bash -n scripts/check-framework-tracking-drift.sh
bash tests/framework-tracking-drift-fixtures.sh

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

**Symptom:** `fw bvp estimate T-2813` in a `HEAD` worktree dies with
`.agentic-framework/lib/bvp.sh: No such file or directory`. `bin/fw` is present and
still routes the subcommand; the library it routes to was never committed. Widening the
check: `policy/` has 0 tracked files, so the scoring rubric and the §ACD-gated driver
weights are absent from version control entirely.

**Root cause:** `.gitignore:21` blanket-ignores `.agentic-framework`, a rule written when
that path was a machine-specific symlink. The tree is now vendored. Because ignore rules
do not apply to already-tracked files, the 1565 files tracked before the rule landed
(2026-06-11, `c73815977`) kept working — so the rule produced no visible breakage at the
moment it became wrong. Everything added since is silently untrackable.

**Why structurally allowed:** the failure mode is specifically invisible to the tool the
operator would use to notice it. `git status` does not report ignored files, and
`git add -A` skips them without a word, so the usual "is anything uncommitted?" check
returns clean while the divergence grows with every `fw upgrade`. Nothing compared what
is on disk against what is reachable from git.

**Prevention:** `scripts/check-framework-tracking-drift.sh` performs exactly that
comparison and fires on untracked files under the load-bearing subtrees (`bin/`, `lib/`,
`policy/`, `agents/`), which are what break a clean clone; `docs/`/`web/` drift is
counted but non-firing. `tests/framework-tracking-drift-fixtures.sh` pins the behaviour,
including that tracking a file clears its firing.

**Note on the check's own first implementation** — worth recording, because it is the
same class: the membership test was
`printf '%s\n' "$tracked" | grep -qxF "$f"` under `set -o pipefail`. `grep -q` exits on
first match and closes the pipe while `printf` is still writing, so the pipeline status
is 141 (SIGPIPE) *on success*, and the test reported "untracked" for every tracked file
— 110 false positives, `bin/fw` among them. This repo already documents that footgun as
L-387 in the task template. It was caught only because the first run's output was
checked against a known-tracked path rather than taken at face value. The fix replaced
the pipeline with an associative-array lookup (no pipeline, no per-file subprocess), and
fixture assertion 1 now pins it.

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

### 2026-08-19T23:46:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/t2687-pickup-failopen/.tasks/active/T-2814-stale-gitignore-silently-untracks-new-ve.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-c71f1441
- **Timestamp:** 2026-08-20T18:54:30Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-20T18:54:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
