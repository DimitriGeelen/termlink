---
id: T-2510
name: "channel renew can SHORTEN an active claim lease (renew_claim absolute set,
  not monotonic) - double-dispatch"
description: >
  channel renew can SHORTEN an active claim lease (renew_claim absolute set, not monotonic)
  - double-dispatch

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-bus/src/lib.rs, crates/termlink-bus/src/meta.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-03T12:23:16Z
last_update: '2026-08-18T18:59:12Z'
date_finished: 2026-08-03T12:25:57Z
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
  - ts: '2026-08-18T18:56:49Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=2 (body:env-class-handled); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:12Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 8
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2510: channel renew can SHORTEN an active claim lease (renew_claim absolute set, not monotonic) - double-dispatch

## Context

`termlink_bus::meta::renew_claim()` (meta.rs:655) sets `claimed_until = now_ms + additional_ttl_ms`
ABSOLUTELY, discarding the existing `old_until`. Its own doc comment (meta.rs:606) states the
contract is "extend the lease by additional_ttl_ms", the param is named `additional_ttl_ms`
("additional" = add to existing), and the CLI `/renew` skill presents it as "extend / keep alive".
So a renew with a small `additional_ttl_ms` on a long-lived lease SHORTENS it — defeating the
claim primitive's purpose (double-dispatch). The existing test only covers additional >> original
TTL (where now+additional happens to exceed old_until), so the shortening case is untested.

## Acceptance Criteria

### Agent
- [x] `renew_claim` computes `new_until = (now_ms + additional_ttl_ms).max(old_until)` — a renew
      never moves the deadline earlier (monotonic-forward, matching the cursor MAX-upsert invariant)
- [x] Regression test: claim with a LONG ttl, renew with a SMALL additional_ttl_ms shortly after;
      assert `renewed.claimed_until >= initial.claimed_until` (today it is SMALLER)
      (`renew_claim_never_shortens_long_lease`; proven load-bearing — FAILS without the clamp)
- [x] Existing `renew_claim_extends_claimed_until_past_original_deadline` still passes
- [x] `cargo test -p termlink-bus --lib` passes (101 passed)

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

cargo test -p termlink-bus --lib renew
grep -q "max(old_until)" crates/termlink-bus/src/meta.rs

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

**Symptom:** A worker holding a long lease (e.g. `claim ... --ttl-ms 3600000`) that calls
`channel renew` with the default small extension (`/renew` defaults `additional_ttl_ms=30000`)
has its lease *shortened* from 1h to 30s. The slot then lazily expires ~30s later while the
worker is still processing; another `find-idle`→`claim` worker acquires the same offset and the
unit is processed twice — the exact double-dispatch the claim primitive exists to prevent.

**Root cause:** `renew_claim` set `claimed_until = now_ms + additional_ttl_ms` absolutely,
discarding the existing `old_until`. "renew/extend" is only a true extension when
`additional_ttl_ms` exceeds the *remaining* lease; for a long lease + small renew it moves the
deadline backward. The doc comment's own headline said "extend the lease by additional_ttl_ms"
and the param name says "additional" — the implementation contradicted its stated contract.

**Why structurally allowed:** the sole renew test (`..._extends_..._past_original_deadline`)
claimed a *short* 200ms TTL and renewed with a *large* 60s extension — the one regime where
`now+additional` always exceeds `old_until`, so the absolute-set and the correct monotonic
behavior are indistinguishable. The shortening regime (long lease, small renew) was never exercised.

**Prevention:** `renew_claim_never_shortens_long_lease` claims a 1h TTL and renews with the 30s
default, asserting `claimed_until` never regresses — it FAILS if the `.max(old_until)` clamp is
removed (verified). Guards the "renew is monotonic-forward" invariant that mirrors the cursor
MAX-upsert and receipt monotonic-frontier already in the codebase (PL-296).

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

### 2026-08-03T12:23:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2510-channel-renew-can-shorten-an-active-clai.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-921fc353
- **Timestamp:** 2026-08-03T12:25:58Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-03T12:25:57Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
