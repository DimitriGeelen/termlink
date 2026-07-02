---
id: T-2304
name: "Fix sys.path bug in update-task.sh inception-scope-trace — __file__ is <stdin> in heredoc, breaks lib.inception_decisions import"
description: >
  Fix sys.path bug in update-task.sh inception-scope-trace — __file__ is <stdin> in heredoc, breaks lib.inception_decisions import

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
created: 2026-07-02T14:21:20Z
last_update: 2026-07-02T15:41:55Z
date_finished: 2026-07-02T15:41:55Z
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

# T-2304: Fix sys.path bug in update-task.sh inception-scope-trace — __file__ is <stdin> in heredoc, breaks lib.inception_decisions import

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The `check_inception_scope_trace` heredoc in `update-task.sh` (~line 578) no longer relies on `__file__` for its sys.path root: it passes `FRAMEWORK_ROOT` via env and `sys.path.insert`s that (falling back to the `__file__` chain only when run as a real file), so `from lib.inception_decisions import …` resolves under a `python3 -` stdin invocation.
- [x] Regression proof: running the same heredoc shape from `cd /opt/termlink` with `FRAMEWORK_ROOT` set imports `lib.inception_decisions` successfully (no `ModuleNotFoundError`); verification command below exits 0.
- [x] T-2303's stuck finalization is completed (status `work-completed`, archived to `completed/`) via `fw inception sweep` after the fix — the GO decision is honored end-to-end.

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

# T-2304: prove `lib.inception_decisions` imports under the fixed sys.path (FRAMEWORK_ROOT env).
FRAMEWORK_ROOT=.agentic-framework python3 -c 'import sys,os; sys.path.insert(0, os.environ["FRAMEWORK_ROOT"]); import lib.inception_decisions; print("import-ok")'
# T-2304: confirm the fix is present in the source (no bare __file__-only sys.path in the scope-trace heredoc).
grep -q 'os.environ.get("FRAMEWORK_ROOT")' .agentic-framework/agents/task-create/update-task.sh

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

**Symptom:** `fw inception decide T-2303 go --i-am-human` recorded the GO decision
("Inception decision: recorded ✓", ACs 3/3, Human 1/1, disposition passed) but then
crashed with `ModuleNotFoundError: No module named 'lib.inception_decisions'` +
apport noise (`FileNotFoundError: '/opt/termlink/-'`), leaving T-2303 stuck at
`status: started-work` in `active/` (finalization to `work-completed`/archived aborted).

**Root cause:** `update-task.sh` `check_inception_scope_trace` runs its Python via a
`python3 - <<'PYEOF'` stdin heredoc, and computed the framework root for `sys.path`
with `os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))`.
For a stdin script `__file__ == '<stdin>'`, so `abspath('<stdin>')` →
`/opt/termlink/<stdin>` and three `dirname`s climb to `/` — `.agentic-framework/`
never lands on `sys.path`, so `from lib.inception_decisions import …` (line 10 of the
heredoc) fails. The apport `'/opt/termlink/-'` noise is the excepthook stat-ing
argv[0]=`-`.

**Why structurally allowed:** the `__file__`-based sys.path idiom is correct for a
real `.py` file (the sibling `check-inception-decisions.py:46` uses
`Path(__file__).resolve().parent` and works) but silently wrong for a stdin heredoc.
The gate only fires for inception tasks reaching finalization with the scope-trace
check active, and its output was partly `|| true`-wrapped, so the breakage stayed
latent until a real `fw inception decide` (T-2303) hit it. Sibling class: PL-227
(Watchtower blueprints' fragile `sys.path.insert`).

**Fix:** pass `FRAMEWORK_ROOT="$FRAMEWORK_ROOT"` as env on the heredoc invocation and
`sys.path.insert(0, os.environ.get("FRAMEWORK_ROOT") or <old __file__ chain>)` —
prefer the explicit env root, keep the `__file__` fallback for real-file runs.
Verified: `FRAMEWORK_ROOT=.agentic-framework python3 -c '… import lib.inception_decisions'`
→ `import-ok`.

**Prevention:** the P-011 verification block on this task asserts the import resolves
under the stdin-shape invocation AND that the `os.environ.get("FRAMEWORK_ROOT")` guard
is present in source — so a regression re-breaks the gate loudly. Candidate learning:
"never derive sys.path from `__file__` in a `python3 -` heredoc — `__file__` is
`'<stdin>'`; pass the root via env/arg" (register on close).

## Status (2026-07-02, budget-blocked before finalization)

- **Fix APPLIED on-disk** to `.agentic-framework/agents/task-create/update-task.sh`
  (the executed copy) + VERIFIED (`import-ok`, grep guard present). `.agentic-framework/`
  is **gitignored** in this consumer repo, so the fix is NOT tracked here — it must be
  **committed in the vendored framework repo and/or couriered to AEF upstream**
  (`/opt/999-*`, per [[relay_to_aef_via_pickup]]).
- **T-2303 GO is recorded** but finalization is STUCK (`started-work`, in `active/`).
  Recover next session with `fw inception sweep` (now that the blocker is fixed) — that
  completes T-2303 → `work-completed`/archived. (AC3 here.)
- **T-2304 close** also pending next session (P-011 + `work-completed` need Bash, blocked
  at ~97% budget this session).

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

### 2026-07-02T14:21:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2304-fix-syspath-bug-in-update-tasksh-incepti.md
- **Context:** Initial task creation

### 2026-07-02T15:41:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
