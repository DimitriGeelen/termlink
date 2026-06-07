---
id: T-2039
name: "substrate Slice 6: channel.claims_summary aggregate RPC + Rust client + CLI"
description: >
  substrate Slice 6: channel.claims_summary aggregate RPC + Rust client + CLI

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
created: 2026-06-07T21:00:58Z
last_update: 2026-06-07T21:00:58Z
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

# T-2039: substrate Slice 6: channel.claims_summary aggregate RPC + Rust client + CLI

## Context

Slices 1-5 (T-2029..T-2038) shipped exclusive-delivery claim semantics end-to-end across hub RPC, bus, Rust client, CLI, MCP, runbook, example, and integration tests. Slice 4 added `channel.claims` for full-list introspection. Operators monitoring topic health under load don't want the full list — they want an **aggregate** signal: "how many claims are active vs lapsed, what's the longest-held one, when's the next free slot?" That's a single SQLite COUNT/MIN over `idx_claims_topic_until` instead of every-row transfer, and it pairs naturally with Slice 4's detail view.

Slice 6 adds the aggregate `channel.claims_summary` RPC + bus method + Rust client + CLI. MCP parity stays a separate slice (matching the T-2037→T-2038 split that worked cleanly).

## Acceptance Criteria

### Agent
- [x] `CHANNEL_CLAIMS_SUMMARY` constant added to `crates/termlink-protocol/src/control.rs` as `"channel.claims_summary"` with doc-block matching CHANNEL_CLAIMS shape; covered by `channel_method_constants_are_stable` test
- [x] `Bus::claims_summary(topic) -> Result<ClaimsSummary>` added to `crates/termlink-bus/src/lib.rs`; `Meta::claims_summary(topic, now_ms)` private helper added to `crates/termlink-bus/src/meta.rs`
- [x] `ClaimsSummary { active_count, expired_count, oldest_active_age_ms?, oldest_active_at_ms?, next_active_expiry_ms? }` struct added to `crates/termlink-bus/src/claim.rs`
- [x] `handle_channel_claims_summary` added to `crates/termlink-hub/src/channel.rs`; router + allowed-methods updated in `crates/termlink-hub/src/router.rs`
- [x] `channel_claims_summary(addr, topic) -> Result<ClaimsAggregate, ClaimError>` added to `crates/termlink-session/src/claim_client.rs` and re-exported via `crates/termlink-session/src/lib.rs` (renamed `ClaimsSummary` → `ClaimsAggregate` on client to avoid collision with existing `ClaimSummary` per-claim type — see Evolution)
- [x] `termlink channel claims-summary <topic> [--hub <addr>] [--json]` CLI verb added; help-text reads naturally; default human format is single-line digest
- [x] Integration test added to `crates/termlink-session/tests/claim_client_integration.rs` covering: empty topic (zeroes), single claim (active=1), released claim (active=0,expired=0), expired claim (active=0,expired=1)
- [x] Runbook `docs/operations/substrate-claim-primitive.md` references `channel.claims_summary` alongside `channel.claims` with "active vs expired aggregate" framing
- [x] All workspace tests pass: `cargo test -p termlink-bus -p termlink-session` shows no regressions (16 integration tests + 100 bus tests + 20 session tests pass)

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
grep -q "pub const CHANNEL_CLAIMS_SUMMARY" crates/termlink-protocol/src/control.rs
grep -q "fn claims_summary" crates/termlink-bus/src/meta.rs
grep -q "pub fn claims_summary" crates/termlink-bus/src/lib.rs
grep -q "handle_channel_claims_summary" crates/termlink-hub/src/channel.rs
grep -q "CHANNEL_CLAIMS_SUMMARY" crates/termlink-hub/src/router.rs
grep -q "channel_claims_summary" crates/termlink-session/src/claim_client.rs
grep -q "^    ClaimsSummary {" crates/termlink-cli/src/cli.rs
grep -q "ChannelAction::ClaimsSummary" crates/termlink-cli/src/main.rs
cargo test -p termlink-bus -p termlink-session --lib --tests -q > /tmp/.t2039.out 2>&1 && grep -q "test result: ok" /tmp/.t2039.out

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

### 2026-06-07 — bus `ClaimsSummary` vs client `ClaimsAggregate` naming
- **What changed:** client `claim_client.rs` already exports a `ClaimSummary`
  struct that mirrors per-claim `ClaimInfo` (T-2031 / T-2037). The bus-side
  aggregate type for Slice 6 is named `ClaimsSummary` (plural — "summary
  over claims") which sits one letter away from the existing client name.
- **Plan impact:** keeping both `ClaimSummary` and `ClaimsSummary` in
  `termlink-session::*` would invite caller mistakes (one letter, wildly
  different shape). Pinned bus name `ClaimsSummary` (idiomatic for the bus
  method `claims_summary`), renamed the client export to `ClaimsAggregate`.
- **Triggered:** none (single-file rename inside the slice; no follow-up
  task). The bus internal name stays consistent with the RPC verb; the
  user-facing client name is unambiguous about purpose.

### 2026-06-07 — slice-6 closes the substrate observability axis
- **What changed:** with `channel.claims` (list, Slice 4) and
  `channel.claims_summary` (aggregate, Slice 6) both shipped, the first
  §6 primitive now has BOTH the per-row detail view (forensics) and the
  O(1) aggregate (monitoring/cron) operators need for stuck-worker
  detection. Pairs with the runbook section "Stuck-worker pattern" added
  in this slice — operator now has a complete loop: "summary detects,
  list identifies, release reopens".
- **Plan impact:** the Slice 7 (MCP parity for claims_summary) becomes a
  small wrapper task next — mirrors the T-2037 (Slice 4) → T-2038 (Slice 5)
  split that worked cleanly. No further surface work is needed on the
  first primitive after Slice 7 ships.
- **Triggered:** T-2040 (Slice 7) — MCP parity wrapper.

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

### 2026-06-07T21:00:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2039-substrate-slice-6-channelclaimssummary-a.md
- **Context:** Initial task creation
