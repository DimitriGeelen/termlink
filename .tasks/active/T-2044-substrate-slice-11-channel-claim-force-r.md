---
id: T-2044
name: "Substrate Slice 11: channel claim-force-release — operator intervention verb (T-2019 extension)"
description: >
  Substrate Slice 11: channel claim-force-release — operator intervention verb (T-2019 extension)

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
created: 2026-06-08T06:32:43Z
last_update: 2026-06-08T06:32:43Z
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

# T-2044: Substrate Slice 11: channel claim-force-release — operator intervention verb (T-2019 extension)

## Context

Slice 11 of the T-2019 (arc-parallel-substrate primitive #1, exclusive-delivery claim semantics) surface. Closes the observability → diagnosis → intervention loop the prior 10 slices opened.

The Slice 8 + 9 + 10 observability axis (`claims-summary --watch` + `--all` + MCP parity) surfaces stuck workers — `expired_count > 0` OR `oldest_active_age_ms > 60_000` annotated `[POTENTIALLY STUCK]`. But there's no operator verb to ACT on the diagnosis. Today's only path to clearing a stuck claim is:

1. Wait for `claimed_until` natural expiry (30s default TTL, up to 1h max)
2. Then another claim attempt triggers lazy-evict on next access (T-2030)

For a wedged worker that keeps renewing but isn't progressing, OR for an operator who needs to recover faster than the TTL allows, this is the missing intervention verb.

**Scope:** Add `channel.force_release(topic, claim_id, reason)` RPC + CLI `channel claim-force-release` + MCP `termlink_channel_claim_force_release`. Hub-side: bypass the `claimed_by != claimer` check that `release_claim` enforces; everything else (cursor non-advance, slot reopening for lazy-evict on next claim) matches `release(ack=false)` semantics — the work returns for retry, not silently consumed.

**Out of scope:** Per-user authorization model. Today the hub trusts any authenticated caller equally (transport-level auth, not user-level — see ADR §2 and §6 #6). force_release is consistent with this — anyone who can reach the hub can break any claim. This is a homelab/single-operator-safe trade; a future multi-tenant scenario would need T-2024 (§6 #6 symmetric auth) PLUS a user/role model (not in §6). Documenting this asymmetry in concerns.yaml is part of this slice (G-064 candidate).

**Symmetry pattern:** `fleet doctor --auto-heal` (T-1683, page-respond mode) is the analog for rotation events; `claim-force-release` is the analog for stuck-worker events.

Companion docs: [docs/architecture/parallel-execution-substrate.md](../../docs/architecture/parallel-execution-substrate.md) (§6 #1, the claim primitive), [docs/operations/substrate-claim-primitive.md](../../docs/operations/substrate-claim-primitive.md) (operator runbook — gets a new "Stuck-worker intervention" section).

## Acceptance Criteria

### Agent
- [x] `crates/termlink-bus/src/meta.rs`: add `force_release_claim(claim_id, reason)` library function — same DELETE path as `release_claim` but no `claimed_by != claimer` check; returns `ReleaseInfo` with `ack=false` semantics
- [x] `crates/termlink-bus/src/lib.rs`: pub wrapper `force_release_claim` on `Bus` + unit test verifying force-release succeeds where `release_claim` fails with `ClaimNotOwned`
- [x] `crates/termlink-hub/src/channel.rs`: `handle_channel_force_release` RPC handler — params `{topic, claim_id, reason?}`, returns `{ok: true, topic, offset, claim_id, forced_by_reason}`
- [x] `crates/termlink-protocol/src/control.rs`: register `CHANNEL_FORCE_RELEASE = "channel.force_release"` constant with doc-block matching existing pattern
- [x] `crates/termlink-hub/src/router.rs`: route `CHANNEL_FORCE_RELEASE` → `channel::handle_channel_force_release` + add to allow-list at line 833-block
- [x] `crates/termlink-cli/src/cli.rs` + `commands/channel.rs`: `channel claim-force-release <topic> <claim_id> [--reason "..."]` CLI verb wrapping the RPC, with `--json` mode
- [x] `crates/termlink-mcp/src/tools.rs`: `termlink_channel_claim_force_release` MCP tool — symmetry per established pattern (T-2043 closure)
- [x] Live verification: spin a fresh hub, claim with claimer A, force-release with caller B + reason, verify (a) RPC returns ok, (b) `channel claims` shows the slot freed, (c) a follow-up `claim` from claimer C succeeds and gets a fresh `claim_id`
- [x] `docs/operations/substrate-claim-primitive.md`: add "Stuck-worker intervention" section under the existing "Stuck-worker pattern" — show the diagnostic-then-fix recipe (`claims-summary --watch` flags → `claim-force-release` clears)
- [x] `.context/project/concerns.yaml`: log G-064 candidate — hub has no per-user authorization model, force-release is operator-equivalent only because anyone with hub access has full control
- [x] Verification block (below) passes

### Human
<!-- Removed: all ACs above are agent-verifiable. -->

## Verification

test -f crates/termlink-bus/src/meta.rs
grep -q "fn force_release_claim" crates/termlink-bus/src/meta.rs
grep -q "fn force_release_claim" crates/termlink-bus/src/lib.rs
grep -q "handle_channel_force_release" crates/termlink-hub/src/channel.rs
grep -q "CHANNEL_FORCE_RELEASE" crates/termlink-protocol/src/control.rs
grep -q "claim-force-release\|ClaimForceRelease" crates/termlink-cli/src/cli.rs
grep -q "termlink_channel_claim_force_release" crates/termlink-mcp/src/tools.rs
grep -q "Stuck-worker intervention" docs/operations/substrate-claim-primitive.md
grep -q "G-064" .context/project/concerns.yaml
cargo build --release -p termlink 2>&1 | tail -3 | grep -q -E "Compiling|Finished|warning"

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

### 2026-06-08 — Slice 11 closes the observability→intervention loop
- **What changed:** Slices 8/9/10 (T-2041/2042/2043) shipped the observability axis — `claims-summary --watch`, `--all` fleet sweep, MCP parity. The diagnostic surface was complete: an operator could see exactly which claim was stuck, on which topic, held by which claimer. But the intervention surface was missing: ordinary `release` requires `claimed_by == claimer`, so for a stuck worker the only path forward was waiting out the TTL (30s default, up to 1h max). Slice 11 adds the missing primitive — `channel.force_release` RPC + `channel claim-force-release` CLI + `termlink_channel_claim_force_release` MCP — that deliberately bypasses the ownership check. Semantics match `release(ack=false)`: cursor untouched, slot reopens for retry. Hub returns audit anchors (`forced_from`, `forced_reason`) in the response.
- **Plan impact:** The T-2018 ADR §6 #1 primitive surface for claims is now structurally complete across detect → diagnose → intervene. No further slices on this primitive are needed before the operator decides on T-2020..T-2028 promotion. The next leverage point for autonomous extension would be claim-event audit log (envelopes on a `<topic>:claim-events` sibling topic walkable via standard `channel subscribe`) — but that's a value-add, not a structural gap.
- **Triggered:** G-064 filed — hub has no per-user authorization model, so `force_release` is operator-equivalent only by virtue of "anyone with hub access has full control." Documented in concerns.yaml + the operator runbook's Authorization Scope section. Status: `watching` until a multi-tenant use case surfaces; revisits after T-2024 (transport-symmetry) lands.

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

### 2026-06-08T06:32:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2044-substrate-slice-11-channel-claim-force-r.md
- **Context:** Initial task creation
