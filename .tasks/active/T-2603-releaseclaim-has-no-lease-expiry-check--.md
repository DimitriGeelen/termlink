---
id: T-2603
name: "release_claim has no lease-expiry check — silent success on lapsed lease (asymmetry vs renew/transfer)"
description: >
  Verb-3 hunt F1: release_claim (bus meta.rs) lacks expiry gate that renew/transfer have

status: captured
workflow_type: build
owner: agent
horizon: next
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-11T10:05:02Z
last_update: 2026-08-11T10:05:02Z
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
- [ ] `release_claim` (or its callers) makes a deliberate decision on the late-ack
      semantics (see Decisions) and implements it consistently with `renew`/`transfer`.
- [ ] If the chosen semantics is "loud-fail on expired lease": `release` against an
      expired claim returns `ClaimExpired` (-32018), the cursor is NOT advanced, and the
      stale row is lazily evicted (mirror the renew/transfer reap). If the chosen
      semantics is "late-ack still completes": the behavior is DOCUMENTED as intentional
      at the bus + MCP + CLI layers so it is no longer a silent divergence.
- [ ] `release_claim` signature/threading of `now_ms` (if added) updated at ALL call
      sites: hub `channel.rs` release handler, MCP `termlink_channel_release`, CLI
      `channel release`, and any `LeasedClaim`/helper. No caller left passing a stale time.
- [ ] A load-bearing unit test in `termlink-bus` proves the new behavior: claim →
      force-expire (advance now_ms past claimed_until) → release → assert the chosen
      outcome (ClaimExpired + cursor-unchanged, OR success + documented). Prove
      load-bearing by temp-reverting the guard and confirming the test FAILS.
- [ ] `cargo test -p termlink-bus` passes.

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

<!-- The core semantic choice this task must resolve BEFORE coding: -->

### OPEN — what should `release(ack=true)` do when the lease already expired?

Two defensible options; pick one deliberately (this is why the task is filed, not
auto-built):

- **Option A — loud-fail (`ClaimExpired`), do NOT advance cursor.** Symmetric with
  renew/transfer; strictly honors "no silent failures"; forces the late worker to
  re-`claim` and re-verify before consuming. Risk: a worker that genuinely finished the
  work but ran slightly over TTL now cannot ack it and the offset may be re-processed —
  but that double-process risk already exists the moment the lease lapsed, so this option
  just makes it VISIBLE rather than silently blessing the stale ack.
- **Option B — allow the late ack to complete, but DOCUMENT it as intentional** at all
  three layers. Rationale: `ack` means "I did this work"; honoring it (even late) reduces
  redundant reprocessing when in fact only A touched the offset. Cost: keeps the
  asymmetry with renew/transfer; must be explicitly documented so it is not read as a bug.

Recommendation leans **Option A** (consistency + the directive), but this is a semantic
call for the owner to confirm — do not implement before deciding.

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
