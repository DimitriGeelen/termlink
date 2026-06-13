---
id: T-2216
name: "align claim error symbols across operator docs+skills with real error_code enum (kill fictional CLAIM_LAPSED/CLAIM_ALREADY_HELD)"
description: >
  align claim error symbols across operator docs+skills with real error_code enum (kill fictional CLAIM_LAPSED/CLAIM_ALREADY_HELD)

status: work-completed
workflow_type: refactor
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-13T15:23:55Z
last_update: 2026-06-13T15:25:13Z
date_finished: 2026-06-13T15:25:13Z
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

# T-2216: align claim error symbols across operator docs+skills with real error_code enum (kill fictional CLAIM_LAPSED/CLAIM_ALREADY_HELD)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The fictional `CLAIM_LAPSED` and `CLAIM_ALREADY_HELD` symbols are removed from all operator surfaces (`docs/`, `.claude/commands/`, `CLAUDE.md`) — a recursive grep returns no match.
- [x] Each corrected location names a REAL `error_code` symbol: `CLAIM_ALREADY_HELD` → `CLAIM_CONFLICT` (-32015), `CLAIM_LAPSED` → `CLAIM_NOT_FOUND` (-32016; protocol's -32018 `CLAIM_EXPIRED` lazy-evicts to not-found). Cross-checked against `crates/termlink-protocol/src/control.rs` + `crates/termlink-session/src/claim_client.rs`.
- [x] The per-primitive `substrate-claim-primitive.md` lapsed-renew row carries the full -32016/-32018 nuance and cross-references the T-2214 live proof (parity with the recipe doc fixed in T-2215).

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

# regression gate: neither fictional symbol may reappear on any operator surface
! grep -rq "CLAIM_LAPSED" docs/ .claude/commands/ CLAUDE.md
! grep -rq "CLAIM_ALREADY_HELD" docs/ .claude/commands/ CLAUDE.md
# the real symbols are present where the taxonomy is taught
grep -q "CLAIM_CONFLICT" docs/operations/substrate-claim-primitive.md
grep -q "CLAIM_NOT_FOUND" docs/operations/substrate-claim-primitive.md

## RCA

**Symptom:** Two fictional claim error symbols — `CLAIM_LAPSED` and `CLAIM_ALREADY_HELD` — were taught across 5 operator surfaces (the orchestrator recipe, the per-primitive claim doc, the `/claim` and `/renew` skills, and the CLAUDE.md catalog) as failure-mode grep targets. Neither token is emitted by any code path (`grep -rn` over `crates/` returns nothing). An operator grepping logs for them after a claim/renew failure finds nothing and mis-diagnoses.

**Root cause:** The error taxonomy was authored from assumed/plausible symbol names, not the authoritative `error_code` enum. The real codes are -32015 `CLAIM_CONFLICT`, -32016 `CLAIM_NOT_FOUND`, -32017 `CLAIM_NOT_OWNED`, -32018 `CLAIM_EXPIRED`. A naturally-lapsed lease lazy-evicts and surfaces as -32016, not the protocol's -32018. The wrong names then propagated by copy from the first doc into every downstream surface.

**Why structurally allowed:** Same class as T-2213/T-2215 — prose/table claims about error vocabulary have no automated cross-check against the source enum. One mistaken symbol in the canonical recipe propagated unchecked to 4 other docs because nothing greps docs against `error_code`.

**Prevention:** This task's `## Verification` adds a repo-wide regression gate asserting neither fictional token reappears on any operator surface AND the real symbols are present where the taxonomy is taught. PL-214 (T-2215) is the durable learning: when touching an error-vocabulary doc, gate it with a grep against the real `error_code` token. A standing audit-tier grep (docs vs `error_code` enum) is the systemic follow-up if this class recurs.

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

## Recommendation

**Recommendation:** Ship. Mechanical token correction across 5 operator surfaces; no code change, no behavior change.

**Rationale:** A fictional error symbol taught consistently across every operator doc is worse than one typo — it trains operators to grep for a string that never appears, guaranteeing mis-diagnosis during a real claim/renew incident. Aligning all surfaces with the real `error_code` enum (and adding a regression gate) closes the systemic drift T-2215 found locally.

**Evidence:** `! grep -rq "CLAIM_LAPSED|CLAIM_ALREADY_HELD" docs/ .claude/commands/ CLAUDE.md` passes (clean); real symbols present in the per-primitive doc; 9 occurrences across 5 files corrected; cross-checked against `control.rs` (-32015..-32018) and `claim_client.rs` ClaimError enum.

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

### 2026-06-13T15:23:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2216-align-claim-error-symbols-across-operato.md
- **Context:** Initial task creation

### 2026-06-13T15:25:13Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
