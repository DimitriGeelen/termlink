---
id: T-2099
name: "/claim-transfer claude-code skill — WRITE-side daily verb for substrate #3 (cooperative handoff, T-2046)"
description: >
  /claim-transfer claude-code skill — WRITE-side daily verb for substrate #3 (cooperative handoff, T-2046)

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
created: 2026-06-09T19:29:43Z
last_update: 2026-06-09T19:32:06Z
date_finished: 2026-06-09T20:31:15Z
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

# T-2099: /claim-transfer claude-code skill — WRITE-side daily verb for substrate #3 (cooperative handoff, T-2046)

## Context

Substrate primitive #3 (T-2046 channel.claim-transfer) is the atomic cooperative
ownership-handoff verb. Today it requires 3 long flags (`--claim-id`, `--to-owner`,
`--by`) with operator-supplied identity on every call. This skill collapses that to
positional `<claim-id> <to-owner>` with auto-resolved `--by` identity, completing the
substrate #1 lifecycle surface at the skill tier (acquire / release / transfer /
observe).

Critical distinction: claim-transfer is the orchestrator-to-worker handoff verb that
ELIMINATES the release-then-claim race window. `/find-idle` → `/claim` (orchestrator
holds) → `/claim-transfer` (atomically hands to worker) — no gap during which a third
party could steal the unit. T-2046's atomic guarantee is the whole point.

Pair design: WRITE-side, mutates state. Loud-refusal taxonomy: CLAIM_NOT_FOUND,
CLAIM_NOT_OWNED (-32017 — `--by` must equal current holder), AUTH_FAIL, RATE_LIMITED.
Auto-resolves `--by` from same T-1857 chain (env → be-reachable.state → refuse).
Distinct from `claim-force-release` (Tier-0 operator bypass); claim-transfer is
COOPERATIVE — current holder volunteers the handoff.

Closes substrate primitive #1 daily-verb surface end-to-end. Natural orchestrator
chain: `/find-idle --capability X` → `/claim <topic> <T-XXX>` → `/claim-transfer
<claim-id> <worker-id>` → worker does work + `/release`.

## Acceptance Criteria

### Agent
- [x] `.claude/commands/claim-transfer.md` exists with 6-step skill structure
- [x] Skill wraps `termlink channel claim-transfer --claim-id ... --to-owner ... --by ...`
- [x] Positional args: `<claim-id> <to-owner>` — operator no longer types two `--flag VALUE` pairs
- [x] Skill auto-resolves `--by` identity via T-1857 chain (env → be-reachable.state → refuse)
- [x] Skill surfaces refusal taxonomy: CLAIM_NOT_FOUND, CLAIM_NOT_OWNED, AUTH_FAIL, RATE_LIMITED with actionable next-step hints
- [x] Skill refuses (does not invent an owner) when `--by` cannot be resolved
- [x] Skill calls out distinction from `claim-force-release` (Tier-0, last resort)
- [x] CLAUDE.md Quick Reference table includes a `/claim-transfer` row paired with `/claim` + `/release`
- [x] Cross-links: T-2018 (ADR), T-2046 (substrate impl), T-2097 (`/claim`), T-2098 (`/release`), T-2093 (`/claims`), T-1857 (resolver), T-2092 (`/find-idle` — pre-claim verb in the orchestrator chain)

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

test -f .claude/commands/claim-transfer.md
grep -q "Step 1: Pre-flight" .claude/commands/claim-transfer.md
grep -q "termlink channel claim-transfer" .claude/commands/claim-transfer.md
grep -q "CLAIM_NOT_OWNED" .claude/commands/claim-transfer.md
grep -q "claim-force-release" .claude/commands/claim-transfer.md
grep -q "T-2046" .claude/commands/claim-transfer.md
grep -q "T-2097" .claude/commands/claim-transfer.md
grep -q "T-1857" .claude/commands/claim-transfer.md
grep -q "/claim-transfer" CLAUDE.md

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

### 2026-06-09T19:29:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2099-claim-transfer-claude-code-skill--write-.md
- **Context:** Initial task creation
