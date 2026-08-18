---
id: T-2603
name: "release_claim has no lease-expiry check — silent success on lapsed lease (asymmetry
  vs renew/transfer)"
description: >
  Verb-3 hunt F1: release_claim (bus meta.rs) lacks expiry gate that renew/transfer
  have

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-bus/src/lib.rs, crates/termlink-bus/src/meta.rs, 
      crates/termlink-cli/src/cli.rs, crates/termlink-hub/src/channel.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-11T10:05:02Z
last_update: '2026-08-18T18:59:13Z'
date_finished: 2026-08-11T13:05:47Z
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
  - ts: '2026-08-18T18:56:53Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 3
      D3: 2
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=3 
      (body:component-silent-failure); D3=2 (body:default-change); D4=3 
      (body:portability-abstraction); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:13Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 5
      tier: 2
      effort: 8
    rationale: blast_radius=5 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2603: release_claim has no lease-expiry check — silent success on lapsed lease (asymmetry vs renew/transfer)

## Context

Found by the T-2468 verb-3 (claim-work) adversarial hunt — finding F1, verified in code.

`Meta::release_claim` (`crates/termlink-bus/src/meta.rs:484-535`) checks ownership
(`claimed_by != claimer` → `ClaimNotOwned`) but has **no lease-expiry gate** — it does
not even take a `now_ms` parameter, so it structurally cannot check `claimed_until`.
Its two siblings both DO gate on expiry and return `ClaimExpired`:
- `renew_claim` (meta.rs:661, guard at :683 `if old_until <= now_ms { … ClaimExpired }`)
- `transfer_claim` (meta.rs:597, guard at :620 `if claimed_until <= now_ms { … ClaimExpired }`)

**Failure scenario:** Worker A claims offset 5 (30s TTL). The lease lapses; no other
worker has yet re-claimed *that exact offset* (reap is lazy — it only fires on a
re-claim of the same `(topic,offset)`), so A's row persists. A finishes late and calls
`release(ack=true)` → returns **SUCCESS** and advances A's cursor past offset 5. During
the expired window offset 5 was legitimately claimable/processable by another worker →
risk of double-processing, and A is never told its lease had lapsed. Violates the
Reliability directive ("no silent failures") + the lease-ownership invariant.

**Why file (not build autonomously):** the fix is not a blind copy of the sibling
guard — it needs a deliberate SEMANTIC decision (see Decisions block) about what a
late `ack` should do, and it changes `release_claim`'s signature (adds `now_ms`) which
ripples to every caller (hub RPC handler + MCP + CLI + any Rust `LeasedClaim`). Core
claim state-machine change → owner:agent, design-first.

## Acceptance Criteria

### Agent
- [x] `release_claim` (or its callers) makes a deliberate decision on the late-ack
      semantics (see Decisions) and implements it consistently with `renew`/`transfer`.
- [x] If the chosen semantics is "loud-fail on expired lease": `release` against an
      expired claim returns `ClaimExpired` (-32018), the cursor is NOT advanced, and the
      stale row is lazily evicted (mirror the renew/transfer reap). If the chosen
      semantics is "late-ack still completes": the behavior is DOCUMENTED as intentional
      at the bus + MCP + CLI layers so it is no longer a silent divergence.
- [x] `release_claim` signature/threading of `now_ms` (if added) updated at ALL call
      sites: hub `channel.rs` release handler, MCP `termlink_channel_release`, CLI
      `channel release`, and any `LeasedClaim`/helper. No caller left passing a stale time.
- [x] A load-bearing unit test in `termlink-bus` proves the new behavior: claim →
      force-expire (advance now_ms past claimed_until) → release → assert the chosen
      outcome (ClaimExpired + cursor-unchanged, OR success + documented). Prove
      load-bearing by temp-reverting the guard and confirming the test FAILS.
- [x] `cargo test -p termlink-bus` passes.

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

out=$(cargo test -p termlink-bus release 2>&1); echo "$out" | grep -q "test result: ok"
grep -q "T-2603 — lease-expiry gate" crates/termlink-bus/src/meta.rs
grep -q "CLAIM_EXPIRED" crates/termlink-hub/src/channel.rs

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

**Symptom:** A worker whose lease lapsed (finished late, after `claimed_until`) could call
`release(ack=true)` and get **SUCCESS** — silently advancing its cursor past an offset that,
during the expired window, was legitimately claimable/re-processable by another worker. The
late worker was never told its lease had lapsed → risk of silent double-processing and a
blessed-but-stale ack.

**Root cause:** `Meta::release_claim` checked ownership (`claimed_by != claimer → ClaimNotOwned`)
but had **no lease-expiry gate** — it did not even take a `now_ms` parameter, so it *structurally
could not* consult `claimed_until`. Its two lifecycle siblings both do: `renew_claim` (guard at
meta.rs:683) and `transfer_claim` (guard at meta.rs:620) each return `ClaimExpired` and lazily
evict the stale row. Release was the one lifecycle verb missing the gate — an asymmetry in the
claim state machine.

**Why structurally allowed:** the expiry check requires the current time, and release's signature
omitted `now_ms` entirely — so the omission was invisible at the type level (nothing forced a
time in). Lazy reap (a claim row is only evicted on a *re-claim of the same offset*) meant an
expired-but-not-yet-reclaimed row persisted and looked releasable. No test asserted the
release-after-expiry path (the existing release tests all release within TTL), so the gap sat
silent. Same "silent success on a state that should refuse" class as T-2604/T-2605.

**Prevention:** The load-bearing test `release_ack_true_on_expired_claim_returns_expired_and_does_not_advance_cursor`
fails the instant the `claimed_until <= now_ms` guard is disabled (proven by temp-revert:
release returns `Ok(ReleaseInfo{ack:true})` and the cursor advances to `Some(1)`). The positive
control `release_within_ttl_still_succeeds_and_advances_cursor` guards against an over-aggressive
gate. Both pin the contract symmetric with the renew/transfer expiry tests.

## Evolution

### 2026-08-11 — the anticipated caller-ripple did not materialize
- **What changed:** The task feared adding `now_ms` to `release_claim` would "ripple to every
  caller (hub RPC + MCP + CLI + any Rust `LeasedClaim`)." In fact the sibling verbs already
  established the pattern that the **public `Bus` wrapper computes `now_ms = now_unix_ms()`
  internally** and passes it to `Meta` (see `Bus::transfer_claim` lib.rs:472, `Bus::renew_claim`).
  Mirroring that, only `Meta::release_claim` (a `pub(crate)` fn) gained the param; the public
  `Bus::release_claim` signature is **unchanged**, so the hub handler, MCP, CLI, and all existing
  test callers compile untouched. now_ms is computed fresh at the Bus boundary on every call, so
  the AC's "no caller left passing a stale time" is satisfied by construction.
- **Plan impact:** The "signature change ripples to all callers" framing that made this a
  design-first FILE was over-cautious — the change turned out contained (one `pub(crate)` param +
  one guard + one error-arm in the handler + doc). The only external-visible change is the new
  `CLAIM_EXPIRED` (-32018) error on the release RPC, which MCP/CLI relay unchanged.
- **Triggered:** No new sub-tasks. Confirms the "compute time at the Bus boundary" convention as
  the right place for lifecycle-verb time — worth remembering for any future claim-state verb.

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

<!-- The core semantic choice this task must resolve BEFORE coding: -->

### 2026-08-11 — RESOLVED (Option A) — release on an expired lease → loud ClaimExpired

- **Chose Option A — loud-fail (`ClaimExpired`, -32018), do NOT advance cursor.** A release
  against a lapsed lease returns `ClaimExpired`, leaves the cursor untouched, and lazily evicts
  the stale row (so the slot becomes cleanly claimable). Expiry is checked BEFORE ownership,
  exactly mirroring `transfer_claim`.
- **Why:** Symmetric with the two sibling lifecycle verbs (renew/transfer already gate on expiry);
  strictly honors "no silent failures"; forces the late worker to re-`claim` and re-verify before
  consuming. The double-process risk already exists the moment the lease lapses — Option A makes
  it VISIBLE rather than silently blessing a stale ack.
- **Rejected Option B (allow the late ack, document it):** keeps the asymmetry with renew/transfer,
  and "honor the ack even if late" still silently advances a cursor past an offset another worker
  may have already processed — the exact silent-double-process the directive forbids. Consistency
  across the claim state machine won.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-11T10:05:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2603-releaseclaim-has-no-lease-expiry-check--.md
- **Context:** Initial task creation

### 2026-08-11T13:00:54Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-db81ae21
- **Timestamp:** 2026-08-11T13:05:48Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-11T13:05:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
