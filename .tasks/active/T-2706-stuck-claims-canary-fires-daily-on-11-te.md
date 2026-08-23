---
id: T-2706
name: "Stuck-claims canary fires daily on 11 test-residue topics"
description: >
  substrate_status --only_pressured reports stuck_count 11 of 770 topics. Every one has active_count 0 with only expired claims, and all are demo/test residue (substrate-drain-demo x8, drain-fix-verify, drain-probe, work-queue). The T-2556 canary fires daily on debris, training operators to ignore it — the guard-gets-deleted failure mode. Clean the topics or exclude them.

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
created: 2026-08-14T15:10:25Z
last_update: 2026-08-14T15:10:25Z
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

# T-2706: Stuck-claims canary fires daily on 11 test-residue topics

## Context

**CORRECTION (2026-08-14) — this task's framing was wrong, and the correction
matters more than the original finding.**

Filed as "11 test-residue topics need cleaning up or excluding." Reading the
code showed the topics were not the problem at all. `is_potentially_stuck`
fired on `expired_count > 0`, and expired claim rows are reaped only when the
SAME `(topic, offset)` is re-claimed (`meta.rs`, `WHERE topic = ?1 AND offset =
?2`). On a topic nobody re-claims, the row — and therefore the "stuck" verdict —
persists for the life of the hub's SQLite. The predicate was a **monotonic
latch**.

So the proposed remedies were both wrong:
- *Cleaning up the 11 topics* would have worked exactly once. The next
  abandoned claim on any topic — including a real production one — latches it
  permanently, and the canary is noisy again with no one the wiser.
- *Excluding them* would have written the 11 names into an allowlist and
  declared the matter closed, hiding a defect that affects every topic.

Both would have removed the symptom while leaving a guard that can never
return to green. That is the "guard gets deleted" failure mode this task's own
ACs warned about — arrived at from the other direction.

**Root cause is fixed in T-2709**, which narrows the expired arm to a recency
window (`newest_expired_at_ms`) so the flag self-clears. The 11 topics need no
disposition: their expiries are days old, so they fall outside the window and go
quiet on their own once the fixed binary is deployed.

**What remains for this task** is only the verification: after the new binary is
installed and the hub restarted, re-measure `stuck_count` and confirm it
reflects genuine stuck work. Recorded honestly either way.

**Live evidence that the fix silences these (measured 2026-08-14, before
deploy).** Two representative topics, read via `channel claims
--include-expired`:

| topic | `claimed_until` | age at measurement |
|---|---|---|
| `work-queue` | 1781341864363 | ~62 days |
| `drain-probe-1425555` | 1781359269709 | ~62 days |

Both lapsed roughly two months ago — far outside the 15-minute recency window —
so both fall silent once the fixed binary is serving.

Note this also settles the AC that asked for `work-queue` to be judged
SEPARATELY from the demo topics, on the theory it might be a real work topic.
It isn't currently live work: its single claim lapsed ~62 days ago under
claimer `root-claude-dimitrimintdev`. It was history too. The instinct to
judge it separately was right; the conclusion is that it needs no action
either.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] The 11 flagged topics are dispositioned — each is either cleaned up or explicitly excluded, with the reason recorded
- [ ] The distinction is preserved: all 11 have `active_count: 0` with only EXPIRED claims, so nothing is genuinely held; the T-2042 heuristic fires on `expired_count > 0`, which is correct for real work and wrong for abandoned test topics
- [ ] Demo/test residue is identified by name and dealt with as a class — `substrate-drain-demo*` (×8), `drain-fix-verify-*`, `drain-probe-*` — rather than one-off
- [ ] `work-queue` (1 expired claim) is judged SEPARATELY from the demo topics: it is a plausibly-real work topic and must not be swept up in a bulk cleanup
- [ ] After the change, `substrate_status --only_pressured` reports a stuck_count that reflects genuine stuck work, so a firing canary is worth reading
- [ ] If exclusion is chosen over cleanup, the exclusion is declared in the canary's own config with a cited reason — not hidden in a threshold bump
- [ ] The scale is recorded for context: 11 of 770 topics, i.e. the canary's signal was ~100% noise on this host

<!-- Origin: T-2705 session, live MCP diagnostics under a critical budget gate.
     Why this matters beyond tidiness: a canary that fires daily on debris trains
     operators to ignore it, which is the "guard gets deleted" failure mode this
     session documented repeatedly (T-2680 scope over-report, T-2699 dead refusals).
     A guard whose firing means nothing is worse than no guard, because it also
     consumes the attention a real firing would need. -->

### Human
- [ ] [REVIEW] Decide cleanup vs exclusion for the demo topics
  **Steps:**
  1. `termlink channel claims-summary --all --only-stuck`
  2. Confirm the `substrate-drain-demo*` / `drain-*` topics are disposable test residue
  **Expected:** agreement that they can be removed or permanently excluded
  **If not:** say which must be retained and why, so the exclusion list cites a real reason

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

### 2026-08-14T15:10:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2706-stuck-claims-canary-fires-daily-on-11-te.md
- **Context:** Initial task creation
