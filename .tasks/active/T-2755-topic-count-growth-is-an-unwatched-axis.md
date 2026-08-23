---
id: T-2755
name: "topic-count growth is an unwatched axis"
description: >
  T-2754 fixed the producers that leaked one permanent topic per run; this closes the blindness that let it run unseen since 2026-04-27. The local hub holds 771 topics carrying 13705 records total. Both existing growth canaries gate on per-topic RECORD count: T-2252 (topic-growth) fires at 5000 records on four watched name patterns; T-2562 (forever-archival) fires at 50000 records on a Forever topic. 771 topics averaging 18 records each trips neither, so unbounded topic-COUNT accumulation is structurally invisible. Note health:ring20-fedprobe sits at 4971 records - Forever, unwatched by name, and 29 under T-2252's threshold. Design question to settle first: a new canary (T-2562 precedent - same channel list read, orthogonal firing gate) versus a second arm on T-2252. Prefer whichever avoids canary sprawl. Any firing gate must account for legitimately-durable topics and must not latch on debris the operator cannot clear (PL-340 / T-2709).

status: captured
workflow_type: build
owner: agent
horizon: later
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-15T22:27:23Z
last_update: 2026-08-15T22:27:23Z
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

# T-2755: topic-count growth is an unwatched axis

## Context

T-2754 fixed the producers that leaked one permanent topic per run. This task
closes the *blindness* that let it run unseen from 2026-04-27 until it was found
by hand (G-019: fix the symptom, then ask why the framework could not see it).

Measured on the local hub: **771 topics carrying 13,705 records total** — roughly
18 records per topic. Both existing growth canaries gate on per-topic RECORD
count and are therefore structurally blind to this shape:

- **T-2252** (topic-growth) fires at 5,000 records, and only on four watched name
  patterns (`agent-presence`, `agent-listeners-*`, `agent-conv-*`, `dm:*`).
- **T-2562** (forever-archival) fires at 50,000 records on a `Forever` topic.

771 topics averaging 18 records trips neither. Worth noting in passing:
`health:ring20-fedprobe` sits at **4,971** records — `Forever`, unwatched by name,
and 29 records under T-2252's threshold.

### The cleanup path is blind too — measured, not assumed

`scripts/sweep-test-debris.sh` exists (T-2424) and looks like the answer. It is
not, for two independent reasons, and a future reader should not assume otherwise:

1. **Nothing runs it.** No cron, no CI, no CLAUDE.md reference — only its own task
   files and one review doc. The T-2683 shape (an artifact believed to work that
   nothing executes).
2. **Even run, it clears almost nothing.** A dry-run against this hub reports
   `topics=771  debris-candidates=1`. Its allowlist covers only the five T-2426
   debris namespaces (`t-*`, `T-*`, `xhub-*`, `stress-*`, `scratch:*`, `smoke:*`).
   The ~630 topics actually leaked here — `agent-conv-selftest-*`,
   `agent-conv-list-*`, `agent-presence-t2302-*`, `arc004-dbg*`, `dummy-*`,
   `drain-fix-verify-*` — match **none** of those patterns.

So the historical debris has no working remover, and detection has no watcher.
T-2754 stopped the bleeding at the producers; neither of those two gaps is closed.

### Design question to settle before building

A **new canary** (following the T-2562 precedent: same `channel list --json` read,
orthogonal firing gate) versus a **second arm on T-2252**. Prefer whichever avoids
canary sprawl — this would otherwise be the 18th.

Constraints on any firing gate:
- It must not latch on debris the operator cannot clear. That is PL-340 / T-2709
  exactly: a guard firing daily on permanent, unclearable noise is how an operator
  learns to stop reading it. Given the sweeper cannot currently remove the 630,
  a naive "topics > N" gate would fire every day with no available remedy.
- Legitimately-durable topics (`channel:learnings`, `policy-decisions`,
  `framework:pickup`, `broadcast:global`) must be excluded, mirroring the T-2252 /
  T-2562 / T-2057 §5 exclusions.
- Deciding the remedy (widen the sweeper's allowlist, or schedule it, or both)
  probably has to come *first* — a detector for a condition with no fix is the
  thing this codebase keeps warning about.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] [First criterion]
- [ ] [Second criterion]

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

### 2026-08-15T22:27:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2755-topic-count-growth-is-an-unwatched-axis.md
- **Context:** Initial task creation
