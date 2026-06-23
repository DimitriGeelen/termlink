---
id: T-2255
name: "Fix worktree isolation — bootstrap complete vendored framework into git worktrees (gitignored-fw breaks finalize/merge)"
description: >
  Fix worktree isolation — bootstrap complete vendored framework into git worktrees (gitignored-fw breaks finalize/merge)

status: started-work
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
created: 2026-06-23T13:26:14Z
last_update: 2026-06-23T13:26:14Z
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

# T-2255: Fix worktree isolation — bootstrap complete vendored framework into git worktrees (gitignored-fw breaks finalize/merge)

## Context

Recurring operator pain (documented as a memory workaround for 10+ days, never filed): git
worktree isolation is structurally broken in this repo. `.agentic-framework/` is gitignored
(`.gitignore:21`, vendored + `fw upgrade`-managed), and `fw` resolves `FRAMEWORK_ROOT` purely
from the LOCATION of the `.agentic-framework/bin/fw` it is invoked through (T-498 removed the
`framework_path` indirection). So a fresh `EnterWorktree`/`git worktree add` checkout
materializes NO (or an incomplete) `.agentic-framework/` → every `fw` op in the worktree is
crippled:
- `fw task update --status work-completed` dies in `evolution_log.sh` (sources the missing
  `lib/arc_membership.sh`) — verification/recommendation pass first, then the transition aborts.
- The reviewer static-scan errors on missing framework files.
- Merge/cleanup of the worktree branch is Tier-0 (force-delete unmerged) — can't self-approve.

Net: work CAN be done in isolation but cannot be finalized or cleanly merged there.

**Design correction during build (see RCA/Evolution):** the vendored dir is NOT wholly gitignored —
it is PARTIALLY tracked (≈1565 files committed, ≈476 untracked/gitignored incl. `lib/arc_membership.sh`).
So a worktree gets a PARTIAL framework, not none. A symlink-replace would show all 1565 tracked
files as deleted and poison the worktree's git state — the opposite of the goal. The shipped fix is
a tracked `scripts/worktree-bootstrap.sh` that COPIES only the missing (untracked/gitignored)
framework files from the main checkout into the worktree's existing dir — filling the gap without
touching tracked files, so `git status` stays clean and `fw` resolves the full framework. Pairs
with a follow-up to push the structural fix (worktree-aware resolution / honor an absolute
`framework_path`) upstream via `fw pickup` (do NOT patch vendored fw locally).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `scripts/worktree-bootstrap.sh` exists, is executable, and is git-TRACKED (not gitignored), so it rides into every worktree checkout
- [x] Run inside a linked git worktree, it COPIES the framework files present in the main checkout but absent in the worktree (the untracked/gitignored set, incl. `lib/arc_membership.sh`) into the worktree's existing dir; it does NOT symlink/replace the dir (which would delete the tracked files); idempotent on re-run
- [x] Run from the main checkout it is a safe no-op (detects it is the main checkout / not a linked worktree) and exits 0 with a clear message
- [x] After bootstrap, the finalize-breaker `.agentic-framework/lib/arc_membership.sh` is present in the worktree AND the bootstrap leaves `git status` clean for `.agentic-framework` (no spurious deletions/additions — copied files are themselves gitignored) — both asserted by `--self-test` (stands up a throwaway detached worktree, bootstraps, asserts, tears down)
- [x] Errors are loud: not-a-git-repo, missing/incomplete main-checkout framework each exit non-zero with an actionable message (no silent degradation)
- [x] A follow-up task (T-2256) is filed to propose the upstream worktree-aware `fw` resolution fix (the copy is mitigation, not the root structural fix)

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
# T-2255 — each line self-contained (P-011 runs each as a separate set -u shell).
test -x scripts/worktree-bootstrap.sh
git ls-files --error-unmatch scripts/worktree-bootstrap.sh >/dev/null 2>&1
out=$(bash scripts/worktree-bootstrap.sh --self-test 2>&1); echo "$out" | grep -q "SELF-TEST PASS"

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

**Symptom:** Work done in a git worktree (harness `EnterWorktree` / `git worktree add`, used
for parallel/isolated work) cannot be finalized or cleanly merged there: `fw task update
--status work-completed` aborts in `evolution_log.sh` (sources the absent
`.agentic-framework/lib/arc_membership.sh`); the reviewer static-scan errors on missing
framework files; the leftover worktree branch needs a Tier-0 force-delete to clean up.

**Root cause:** `.agentic-framework/` is only PARTIALLY tracked — ≈1565 files are committed, but
≈476 (added/managed by `fw upgrade`, gitignored, incl. `lib/arc_membership.sh`) are NOT. So a
worktree checkout gets a PARTIAL framework. AND `fw` resolves `FRAMEWORK_ROOT` solely from the
on-disk location of the `.agentic-framework/bin/fw` it is run through (T-498 deleted the
`framework_path` indirection from `.framework.yaml`), so there is no path indirection that could
point a worktree at the complete copy in the main checkout. Partial-on-disk + no-indirection =
broken `fw` in every worktree.

**Why structurally allowed:** no test or gate ever exercised a `fw` command from inside a
worktree — the breakage only lives on the rarely-travelled worktree path. Worse, when first
hit it was captured as a *memory workaround* ("finalize in the main checkout") instead of being
filed as a task, so the framework had nothing tracking it and stayed blind for 10+ days (G-019:
fixing the symptom-in-the-moment without asking "why was the framework blind?").

**Prevention:** `scripts/worktree-bootstrap.sh` is git-TRACKED (so it rides into every worktree)
and carries a `--self-test` that stands up a throwaway worktree, bootstraps it, and asserts the
framework is reachable — a regression catch, not just a fix. A follow-up task pushes the real
structural fix (worktree-aware `fw` resolution / honor an absolute `framework_path`) upstream.
The stale memory workaround note is updated to point at the bootstrap.

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

### 2026-06-23 — design pivot: copy-missing, not symlink (the self-test taught me)
- **What changed:** I filed this expecting `.agentic-framework` to be wholly gitignored, so the
  plan was "symlink the worktree's dir → main's complete copy." The `--self-test` immediately
  surfaced that the dir is PARTIALLY tracked (1565 tracked / 476 untracked). A symlink-replace
  would have shown 1565 tracked files as deleted in the worktree — poisoning git state and
  breaking the merge this task exists to fix.
- **Plan impact:** Switched the fix to "copy only the files missing in the worktree" (the untracked
  set), filling the gap without touching tracked files. Added a git-status-clean assertion to the
  self-test as the load-bearing property (proves no merge pollution).
- **Triggered:** the stale `worktree-T-2209-history-skills` worktree (2 unmerged commits + dirty
  tree) was found during testing — surfaced to the human as a separate cleanup decision (not
  auto-merged/deleted: ambiguous completion status + branch-delete is Tier-0). Upstream-fix
  follow-up filed.

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

### 2026-06-23T13:26:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2255-fix-worktree-isolation--bootstrap-comple.md
- **Context:** Initial task creation
