---
id: T-2206
name: "Orchestrator claim-transfer schema fix + end-to-end LIVE smoke proof (T-2204 follow-up)"
description: >
  Smoke testing T-2204+T-2205 substrate consumer kit end-to-end uncovered that scripts/orchestrator-backlog-drain.sh's claim-transfer call used positional args ('claim_id' 'to-owner') but the CLI verb requires --claim-id + --to-owner flags. Fix the call, re-run the LIVE smoke proof, capture the success path in the recipe doc as a validation reference.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [substrate, parallel-worker, bug-fix, arc-001, T-2018]
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-13T09:18:44Z
last_update: 2026-06-13T09:18:44Z
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

# T-2206: Orchestrator claim-transfer schema fix + end-to-end LIVE smoke proof (T-2204 follow-up)

## Context

End-to-end smoke testing the T-2204+T-2205 substrate consumer kit (orchestrator + worker scripts) caught one remaining schema mismatch: `scripts/orchestrator-backlog-drain.sh` called `termlink channel claim-transfer "$claim_id" "$target" --by ...` with positional args, but the CLI verb requires `--claim-id` + `--to-owner` flags. Single-line fix + re-run end-to-end + capture the success path as a validation reference.

Validation evidence (after fix):
- Orchestrator (`--live --orchestrator-id fake-orch --limit 1`): posted T-1166 to work-queue offset 3, claimed as fake-orch, transferred to root-claude-dimitrimintdev
- Worker (`--auto-noop --once`): discovered the claim, decoded base64 payload (task=T-1166, classification=closure-ready, ac_count=0, dispatched_by=fake-orch), released --ack with `{topic: work-queue, offset: 3, ack: true}`
- Final state: 0 active claims (kit closed the loop cleanly)

## Acceptance Criteria

### Agent
- [x] `claim-transfer` call in orchestrator-backlog-drain.sh uses `--claim-id` + `--to-owner` flags (not positional args)
- [x] End-to-end LIVE smoke (orchestrator post→claim→transfer + worker discover→decode→release) executes cleanly with `Summary: dispatched=1 failures=0`
- [x] Final hub state shows 0 active claims after worker --auto-noop release
- [x] Recipe doc gains a "Validation evidence" callout under the in-tree consumer section so future readers don't need to re-run the smoke to trust the kit

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

**Symptom:** `scripts/orchestrator-backlog-drain.sh --live` failed at the claim-transfer step with `DISPATCH [TRANSFER-FAIL]` and `unexpected argument 'clm-…-work_queue-2' found`. The claim ITSELF landed; only the transfer to worker failed, leaving fake-orch as the stuck claimer.

**Root cause:** The script's `claim-transfer` call used positional args (`termlink channel claim-transfer "$claim_id" "$target" --by …`) but the CLI verb requires `--claim-id` + `--to-owner` flags. The other verbs (post, claim, release) all happen to use a mix of positional + flag patterns; claim-transfer is flag-only. Easy mismatch when authoring from CLAUDE.md catalogue alone without `--help` cross-check.

**Why structurally allowed:** Smoke testing of the kit happened at the script-validation level (dry-run output, simulated single-step substrate calls) but did NOT exercise the full LIVE pipeline before T-2204 closure. The dry-run path bypasses the actual CLI verbs, so schema bugs survived to first --live attempt.

**Prevention:** Recipe doc's "In-tree consumer" section now ships with explicit "Validation evidence" subsection showing the proven invocation. This is grep-able for future readers and serves as the load-bearing artifact AEF integrators trust before running their own smoke. Future substrate-consumer additions should run a `--live` end-to-end against the local hub at task-close time (added to PL-206 follow-up pattern: don't trust catalogue notes — `--help` is authoritative).

<!-- Original template hint follows:
     REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
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

### 2026-06-13T09:18:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2206-orchestrator-claim-transfer-schema-fix--.md
- **Context:** Initial task creation
