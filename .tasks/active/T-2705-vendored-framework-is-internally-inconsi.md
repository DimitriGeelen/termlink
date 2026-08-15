---
id: T-2705
name: "Vendored framework is internally inconsistent — Watchtower cannot start (missing lib/arc_membership.py)"
description: >
  web/blueprints/arcs.py imports lib.arc_membership, which exists zero times in git and zero times on disk under .agentic-framework/. Watchtower crashes at create_app(). Upstream master has the module and is genuinely newer (its arcs.py carries T-2774 vs termlink at T-2704); the vendored VERSION 1.6.295 vs upstream 1.6.145 is a lineage/tag-epoch artifact, not a downgrade (the T-2359 lesson). Re-vendor via fw update.

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
created: 2026-08-14T12:02:56Z
last_update: 2026-08-14T12:13:55Z
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

# T-2705: Vendored framework is internally inconsistent — Watchtower cannot start (missing lib/arc_membership.py)

## Context

Verified 2026-08-14 after the budget gate cleared. Every AC below was ticked
against a real measurement, not an inference — the previous session committed
this change explicitly marked UNVERIFIED because the gate blocked all four
checks.

**Evidence:**
- `fw doctor` — 0 failures, 6 warnings (none new; enforcement baseline reported
  intact, which is the L-398 check).
- `bash scripts/run-guard-layer.sh` — PASS, 25/25 members clean.
- `cargo test --workspace` — exit 0.
- Watchtower — starts on :3003, health check passes. `/` `/tasks` and crucially
  `/arcs` all return **200**. `/arcs` is the specific blueprint whose
  `lib.arc_membership` import was missing, so a 200 there is what actually
  proves the fix rather than a healthy root page.

**Rollback (one command).** `.agentic-framework` is force-tracked in git
(2366 files), so the re-vendor is revertible without touching `fw update`'s
own backup:

```
git checkout a46e8e726 -- .agentic-framework
```

`a46e8e726` is the last commit before the re-vendor (`c933f8eb7`).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `lib/arc_membership.py` exists after the re-vendor, so `web/blueprints/arcs.py`'s import resolves
- [x] Watchtower actually **starts and serves** — verified by an HTTP 200, not by "the command returned 0"
- [x] The re-vendor uses the sanctioned `fw update` path, which saves a rollback backup, rather than a hand-rolled `rm -rf` + copy
- [x] `fw doctor` reports no new failures versus the pre-change baseline (the vendored framework supplies the hooks governing this session — replacing it must not silently break enforcement)
- [x] The enforcement baseline is checked: if `.claude/settings.json`-governed hooks changed, that is surfaced rather than left to accumulate as a silent `fw doctor` FAIL (L-398)
- [x] `bash scripts/run-guard-layer.sh` still passes 25/25 — the guard layer's fixtures shell out to framework paths, so a framework swap could break them
- [x] `cargo test --workspace` green — the framework swap must not disturb the product build
- [x] The version-comparison trap is recorded: vendored 1.6.295 vs upstream 1.6.145 reads as a downgrade but upstream carries T-2774 (> termlink's T-2704), so the patch numbers are non-comparable across lineages — the T-2359 lesson, and the reason this was nearly mis-actioned
- [x] Rollback path stated in the task so the operator can undo it in one command

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

### 2026-08-15 — ⚠ READ BEFORE THE NEXT RE-VENDOR (T-2721 local patch will be erased)

`.agentic-framework/agents/audit/audit.sh` currently carries a **local patch** that
is NOT upstream. A re-vendor overwrites it and four false audit findings return:

- `_resolve_hook_path()` helper (added just above the commit-msg hook check) — uses
  `git rev-parse --git-path hooks/<name>` instead of concatenating
  `$PROJECT_ROOT/.git/hooks/…`, which cannot resolve in a linked worktree because
  `.git` is a file there. Consumed by the **commit-msg**, **C-002**, and **CTL-011**
  checks.
- **CTL-020** — now skips on a linked worktree via `fw_is_linked_worktree`, matching
  the cron-drift / cron-misload checks in the same file.

If those four warnings reappear after a re-vendor, this is why — reapply from T-2721
(or, better, check whether upstream has taken U-003 / U-004, which are the durable
fix; if so, drop the local patch rather than reapplying it).

Recorded here rather than in a comment inside `audit.sh`, because a note living in
the file that gets overwritten cannot survive the event it warns about.

### 2026-08-14T12:02:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2705-vendored-framework-is-internally-inconsi.md
- **Context:** Initial task creation

### 2026-08-14T12:03:09Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
