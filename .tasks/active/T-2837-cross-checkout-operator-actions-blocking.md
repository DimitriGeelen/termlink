---
id: T-2837
name: "Cross-checkout operator actions blocking the t2687 merge"
description: >
  Two guard-layer FAILs gate eight finished tasks and can only be cleared from checkouts this worktree cannot reach (T-559 project boundary): recover three dangling corpus_*.py into the vendored framework from /opt/termlink, and renumber the T-2690/91/92 ID collisions on two sibling worktrees. Carried as Human ACs so they surface on the /approvals route instead of living only in a session transcript.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-23T21:51:44Z
last_update: 2026-08-23T21:53:59Z
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

# T-2837: Cross-checkout operator actions blocking the t2687 merge

## Context

Four guard-layer checks currently FAIL, and those FAILs gate eight otherwise-finished
tasks from finalizing. Two were cleared from this worktree (T-2836 repaired 29
unreadable episodics; T-2714 filed the audit hook-path defect upstream). The other
two **cannot** be cleared from here, and the reason is structural rather than a
matter of effort:

- **`check-framework-tracking-drift` axis B** reports three DANGLING refs —
  `tools/corpus_spec.py`, `tools/corpus_lint.py`, `tools/corpus_explain.py`, sourced
  from `.agentic-framework/bin/fw:4901-4909` and `designer.sh:302`. Per T-2806/T-2817
  a dangling ref can only be fixed in the checkout that still HAS the files, where
  they show as UNTRACKED instead. That is `/opt/termlink`; a worktree materialises
  only tracked files, so they are not here to commit.
- **`check-task-id-collisions` axis A** reports T-2690/91/92 claimed by two branches
  (`worktree-charter-review-2026-0814`, `worktree-governance-canary-signal`) as
  *different tasks*, plus 25 duplicated files. Renumbering must happen on those
  branches. T-559 keeps this session inside its own project boundary.

Delegating these over termlink was attempted and is **not available**: every session
on this host shares one identity file, so `agent contact termlink-agent` resolved
`peer_fp == my_fp` and the DM looped back to a self-topic. That is the same
identity collapse behind T-2690's ambiguous `whoami`. Cross-host peers have distinct
fingerprints but none of them owns these checkouts.

So the actions are genuinely the operator's, and they are carried here as Human ACs
specifically so `/approvals` section D surfaces them — a session transcript is not a
queue, and both items had already been re-derived more than once.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The task states, for each action, why it cannot be done from this worktree rather than merely that it was not done
- [x] Each Human AC carries a single-line copy-pasteable command prefixed with `cd <path> &&` per T-609
- [x] The task is `owner: human` so it lands in `/approvals` section D (unchecked Human ACs) rather than an agent queue

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

- [ ] [REVIEW] The three `corpus_*.py` files are recovered into the vendored framework and committed from `/opt/termlink`

  **Steps:**
  1. `cd /opt/termlink && bash scripts/check-framework-tracking-drift.sh`
  2. Confirm the three files appear under **UNTRACKED** there (not DANGLING — that is this worktree's view of the same fact).
  3. Read them before committing. Framework files carried across disks can hold machine-local paths or credentials, and a commit is permanent. T-2806's scan flagged a bare 64-hex in `policy/designer-pin.yaml` that turned out to be a `sha256:` reproducibility pin — a true pattern match and a false risk. That judgement is yours; the check surfaces, it does not decide.
  4. `cd /opt/termlink && git add -f .agentic-framework/tools/corpus_spec.py .agentic-framework/tools/corpus_lint.py .agentic-framework/tools/corpus_explain.py`
  5. Commit, then push to **OneDev only** — never GitHub.

  **Expected:** `bash scripts/check-framework-tracking-drift.sh` exits 0 in this worktree afterwards, with `dangling_count: 0`.

  **If not:** axis B's count is a lower bound — recovering these may expose further refs that were invisible while their parent was missing (T-2806 saw exactly this). Expect to converge over two rounds, not one. If a file genuinely does not exist on any disk, say so and the reference should be deleted rather than satisfied.

- [ ] [REVIEW] T-2690/91/92 are renumbered on the two sibling branches before any merge

  **Steps:**
  1. `cd /opt/termlink && bash scripts/check-task-id-collisions.sh`
  2. For each colliding ID, confirm from the printed filenames that the two branches hold *different tasks* (a cherry-pick — same ID, same task — is not reported and needs nothing).
  3. Renumber on the branches that own them, picking IDs above the highest claimed on **any** branch. `fw task create` has no `--id` flag, so this is a manual rename plus a reference sweep. T-229 is the worked example.

  **Expected:** the check exits 0; no ID is claimed by two branches beyond the merge base.

  **If not:** do not merge. Merging a collision silently discards one of the two tasks, which is the failure T-229 recorded in March 2026 and that recurred in August because nothing enforced it. Note also that this task's own ID was allocated by the same racing counter — verify T-2837 is not itself colliding.

- [ ] [REVIEW] P-043 is disposed of deliberately — acted on or dropped, not left stranded

  **Steps:**
  1. `cd /opt/termlink/.claude/worktrees/t2687-pickup-failopen && cat .context/pickup/auto-deferred/P-043-bug-report.yaml`
  2. It reports two framework bugs blocking every inception decision, with file:line references. **Both are already resolved:** BUG 2 was independently re-discovered and re-fixed as T-2304, and BUG 1 (`update-task.sh:791`) is filed upstream at `framework:pickup` offset 17. So its content is spent.
  3. My recommendation is therefore to **drop it** — delete the envelope — rather than file anything new from it.
  4. This is your call, not mine: `check-pickup-deferred-freshness.sh` detects and never drains, precisely because discarding is a judgement about whether the work still matters. An agent auto-draining would turn a visible backlog into a silent one, which is the trade the check exists to reverse.

  **Expected:** `bash scripts/check-pickup-deferred-freshness.sh` exits 0, and the guard layer drops from 3 firing to 2.

  **If not:** if you would rather keep it, that is fine — but it should then carry a breadcrumb naming a real blocking task, otherwise `fw pickup promote-deferred` can never promote it and it is stranded again by construction. The cost of leaving it is measured: this envelope sat 76 days while one of the bugs it named was solved twice.

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

### 2026-08-23T21:51:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/t2687-pickup-failopen/.tasks/active/T-2837-cross-checkout-operator-actions-blocking.md
- **Context:** Initial task creation

### 2026-08-23T21:53:59Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
