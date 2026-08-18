---
id: T-2680
name: "charter-drift canary reports a full-surface clean bill it cannot measure (category-blind,
  28 live analytics tools invisible)"
description: >
  charter-drift canary reports a full-surface clean bill it cannot measure (category-blind,
  28 live analytics tools invisible)

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
created: 2026-08-13T23:19:37Z
last_update: '2026-08-18T18:58:39Z'
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
  - ts: '2026-08-18T18:55:36Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 3
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=3 
      (body:component-silent-failure); D3=2 (body:default-change); D4=2 
      (body:env-class-handled); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:39Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2680: charter-drift canary reports a full-surface clean bill it cannot measure (category-blind, 28 live analytics tools invisible)

## Context

`scripts/check-charter-drift-freshness.sh` (T-2483) is the structural guard for charter
non-goal #3 ("not a social / engagement platform"). Its header states its purpose as
detecting when *"the tool surface has drifted from the charter's four verbs"*, and
CLAUDE.md records its result as **"214 live tools scanned, 0 off-charter"**. It emits:

```json
{"ok":true,"firing":[],"checked":214,"live_off_charter":0}
```

That reads as a full-surface charter-traceability clean bill. It is not one. Mechanically
the canary applies a **fixed six-family name regex** (reactions / emoji / stars / pins /
typing / polls) — the exact families P4 happened to delete. `checked:214` counts tools the
regex was *run against*, not tools whose purpose was *assessed*.

28 tools are LIVE right now in categories the binary itself names as analytics:
`agent_rankings` (5), `agent_stats` (10), `agent_thread_health` (8),
`channel_engagement` (5). `termlink_agent_top_repliers` is a social leaderboard — same
class as `top_reacted`/`top_pinners`, which P4 deprecated. The only difference is which
word is in the regex.

Proven with the canary's own PL-213 test hook: a catalog containing live
`termlink_agent_top_reacted` fires (exit 1); one containing live
`termlink_agent_top_repliers` passes clean (exit 0).

Worse than the under-detection is the **false assurance**: those 28 tools are precisely
what human-owned **T-2548** is currently incepting to subtract. So an open decision about
~30 off-charter tools coexists with a daily canary reporting that surface as 0-off-charter.
An operator reading `/canaries` sees green. Directive #2 (no silent failures) violated in
the guard layer — the costliest place, because a guard reporting green is why nobody looks.

**Scope discipline:** this task does NOT delete or deprecate anything. That decision is
T-2548's and is `owner: human`. The fix is to make the canary honest about what it
measured, and to make the pending decision *visible* via an acknowledgement allowlist
(the T-2527/T-2531 idiom) rather than invisible in a regex gap.

Found by the T-2678 charter guard-coverage review (finding F2 / IW-2).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Canary gains a **category** detector alongside the name-pattern detector: a live tool whose catalog category matches the off-charter analytics set (`agent_rankings`, `agent_stats`, `agent_thread_health`, `channel_engagement`, plus the already-empty `*_poll` / `*engagement_metrics`) is off-charter regardless of its name
- [x] `termlink_agent_top_repliers` presented as live is now detected — the exact case that passed clean before
- [x] An **acknowledgement allowlist** at a **git-tracked** path suppresses known-pending sites, one `<tool_name>  # <reason>` per line, `#`-comment and blank lines skipped
- [x] The currently-live analytics tools are acknowledged with a reason citing T-2548, so the canary does not alarm-fatigue daily on an open human decision
- [x] An acknowledged tool does NOT fire; removing it from the allowlist DOES fire (allowlist load-bearing in both directions)
- [x] Healthy output no longer claims full-surface traceability — it names what was scanned, by which detectors, and how many sites are acknowledged-pending
- [x] JSON envelope extended additively (`ok`/`firing`/`checked`/`live_off_charter` keep their meaning); adds `acknowledged_count`, `off_charter_total`, `detectors`
- [x] Fixture suite `tests/charter-drift-check-fixtures.sh` covers: name-detector fires, category-detector fires, acknowledged suppressed, un-acknowledging re-fires, healthy path, unparseable catalog → exit 2
- [x] Fixture suite passes and the live tree returns exit 0 with a truthful acknowledged count
- [x] CLAUDE.md's charter-drift entry corrected — it currently states the "0 off-charter" result this task proves was never a full-surface measurement

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

bash tests/charter-drift-check-fixtures.sh
bash scripts/check-charter-drift-freshness.sh --no-heartbeat
test -f .context/checks/charter-drift-allowlist
git ls-files --error-unmatch .context/checks/charter-drift-allowlist

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

### 2026-08-13T23:19:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2680-charter-drift-canary-reports-a-full-surf.md
- **Context:** Initial task creation
