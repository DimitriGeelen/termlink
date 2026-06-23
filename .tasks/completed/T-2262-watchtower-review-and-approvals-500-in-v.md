---
id: T-2262
name: "Watchtower /review and /approvals 500 in vendored mode — blueprints insert PROJECT_ROOT/lib but dispatch_pause lives at FRAMEWORK_ROOT/lib"
description: >
  Watchtower /review and /approvals 500 in vendored mode — blueprints insert PROJECT_ROOT/lib but dispatch_pause lives at FRAMEWORK_ROOT/lib

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
created: 2026-06-23T20:47:27Z
last_update: 2026-06-23T20:51:38Z
date_finished: 2026-06-23T20:51:38Z
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

# T-2262: Watchtower /review and /approvals 500 in vendored mode — blueprints insert PROJECT_ROOT/lib but dispatch_pause lives at FRAMEWORK_ROOT/lib

## Context

Field-discovered 2026-06-23 when the human asked for a Watchtower link: every
`/review/<task-id>` page returns **HTTP 500**. Root cause (confirmed): the review
+ approvals blueprints insert the WRONG lib dir on `sys.path`.

`.agentic-framework/web/blueprints/review.py:21` (and `approvals.py` identically) does:
```python
sys.path.insert(0, str(PROJECT_ROOT / "lib"))   # T-1810
...
from dispatch_pause import format_age, list_paused_dispatches_for_task, truncate
```
But `dispatch_pause.py` lives at **`FRAMEWORK_ROOT/lib/`**, not `PROJECT_ROOT/lib/`.
In a **vendored install** (framework ≠ project, the normal consumer layout here),
`PROJECT_ROOT/lib` = `/opt/termlink/lib` which does not exist → `ModuleNotFoundError:
No module named 'dispatch_pause'` → 500 before any task logic runs (so EVERY
`/review/<id>` and the Tier-0 `/approvals` flow are broken, regardless of task state).

This is the **web-blueprint sibling of PL-222** (which was the inception-close
`python3 -`/`__file__` path bug): framework code computing a lib path that is
correct only when framework==project (the framework's own repo) and wrong in
vendored/consumer mode.

**Proper fix (one line, upstream — vendored, cannot patch locally per PL-022):**
`web.shared` already exports `FRAMEWORK_ROOT = APP_DIR.parent` (shared.py:22).
Both blueprints should `sys.path.insert(0, str(FRAMEWORK_ROOT / "lib"))`.
**Local workaround (no patch):** start Watchtower with
`PYTHONPATH=<project>/.agentic-framework/lib` so `dispatch_pause` resolves
despite the wrong insert. Verified working.

## Acceptance Criteria

### Agent
- [x] Root cause confirmed: review.py:21 + approvals.py:25 insert `PROJECT_ROOT/lib`; `dispatch_pause.py` lives at `FRAMEWORK_ROOT/lib` (PROJECT_ROOT/lib absent in vendored install). Documented above.
- [x] Local workaround applied: Watchtower restarted with `PYTHONPATH=.../.agentic-framework/lib`; `/review/<id>` returns HTTP 200 (was 500). Verified T-2258/T-2259/T-2262/T-2256 + /approvals all 200.
- [x] Upstream fix relayed to AEF via `fw pickup send` → RCA bug-report **P-049** (priority high); verified landed on `framework:pickup` offset 55 (auto-bridge silent-failed per PL-227-adjacent gotcha, force-bridged manually).
- [x] Learning registered: **PL-227** — web-blueprint sibling of PL-222 (vendored-mode lib-path resolution bug class).

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

# T-2262: /review must return 200 (was 500). Reads live Watchtower URL from triple file.
test "$(curl -s -o /dev/null -w '%{http_code}' "$(cat .context/working/watchtower.url)/review/T-2258")" = "200"

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

**Symptom:** Every `GET /review/<task-id>` and the Tier-0 `/approvals` page return HTTP 500 (`ModuleNotFoundError: No module named 'dispatch_pause'`). Total outage of the human-review + approval UI on this (vendored) host — independent of task state.

**Root cause:** `web/blueprints/review.py:21` and `approvals.py` do `sys.path.insert(0, str(PROJECT_ROOT / "lib"))` then `from dispatch_pause import …`, but `dispatch_pause.py` ships at `FRAMEWORK_ROOT/lib`, not `PROJECT_ROOT/lib`. In a vendored install (framework ≠ project) `PROJECT_ROOT/lib` (`/opt/termlink/lib`) does not exist, so the import never resolves.

**Why structurally allowed:** The framework's own CI/dev runs from its OWN repo where `FRAMEWORK_ROOT == PROJECT_ROOT`, so the wrong path coincidentally resolves and the defect is invisible. No test boots the web app in a vendored layout (distinct roots). This is the web-blueprint sibling of PL-222 — the recurring "path correct only when framework==project" class.

**Prevention (distinct from the fix):** (1) PL-227 registered. (2) Recommended upstream (in P-049): a smoke test that boots Watchtower with `PROJECT_ROOT` set to a temp dir distinct from `FRAMEWORK_ROOT` and asserts `/review/<seed>` returns 200; and/or an audit grep flagging `PROJECT_ROOT / "lib"` under `web/`. The fix itself (use `FRAMEWORK_ROOT/lib`) is upstream/vendored — relayed via P-049, not patched locally (PL-022).

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

### 2026-06-23T20:47:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2262-watchtower-review-and-approvals-500-in-v.md
- **Context:** Initial task creation

### 2026-06-23T20:51:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
