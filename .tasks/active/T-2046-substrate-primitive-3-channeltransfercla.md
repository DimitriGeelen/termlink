---
id: T-2046
name: "Substrate primitive #3: channel.transfer_claim RPC (T-2021 GO build slice)"
description: >
  Implement the T-2021 GO decision per docs/reports/T-2021-pull-assign-rpc-inception.md. ONE new RPC: channel.transfer_claim(claim_id, to_owner, by, reason?). Atomic ownership transfer (cooperative + owner-checked, distinct from T-2044 force_release). Pull-side is pure composition of subscribe + claim — no new code. ~120 LOC across 4 vertical slices.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:arc-parallel-substrate, substrate-primitive, foundation]
components: []
related_tasks: [T-2018, T-2021, T-2019, T-2020, T-2044]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-08T10:49:01Z
last_update: 2026-06-08T13:20:44Z
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

# T-2046: Substrate primitive #3: channel.transfer_claim RPC (T-2021 GO build slice)

## Context

T-2021 GO build (see `docs/reports/T-2021-pull-assign-rpc-inception.md`). Atomic ownership transfer of an existing claim — the missing primitive that turns the orchestrator's "find idle worker, hand them this unit" recipe into a non-racy operation. Distinct from T-2044 `force_release`: transfer is cooperative + owner-checked, force_release is operator-Tier-0 bypass. Single-row UPDATE on the existing claims table — no schema migration.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->

**Slice 1 — Bus library + unit tests:**
- [x] `Bus::transfer_claim(claim_id, to_owner, by, reason)` added to `crates/termlink-bus/src/lib.rs` and `Meta::transfer_claim` to `crates/termlink-bus/src/meta.rs`. Atomic single-row UPDATE under transaction: gates SELECT → check expired (lazy-evict + `ClaimExpired`) → check `claimed_by == by` (`ClaimNotOwned`) → UPDATE `claimed_by = to_owner` → return `TransferInfo` with `from_owner` and `to_owner` distinguished.
- [x] `TransferInfo` struct re-exported from `termlink-bus` with: `claim_id, topic, offset, from_owner, to_owner, claimed_at, claimed_until, reason`.
- [x] Unit tests cover: (a) happy path + lease preservation, (b) claim-not-found → `ClaimNotFound`, (c) expired claim → `ClaimExpired` + row evicted + slot reclaimable, (d) `by` ≠ `claimed_by` → `ClaimNotOwned` + row intact, (e) self-transfer (to_owner == claimed_by) success-path, (f) cursor-advances-belong-to-new-owner (A→B transfer + ack release by B: B's cursor advances past offset, A's stays None). **6/6 tests passing** under `cargo test -p termlink-bus transfer_claim`.
- [x] `cargo check -p termlink-bus` passes.

**Slice 2 — Hub handler + router arm + protocol const:**
- [ ] `CHANNEL_TRANSFER_CLAIM = "channel.transfer_claim"` method constant added to `crates/termlink-protocol/src/control.rs` with doc-comment naming `CLAIM_NOT_FOUND / CLAIM_NOT_OWNED / CLAIM_EXPIRED` errors and the cooperative-vs-force distinction vs `CHANNEL_FORCE_RELEASE`.
- [ ] `handle_channel_transfer_claim` handler in `crates/termlink-hub/src/channel.rs` parses `{claim_id, to_owner, by, reason?}`, calls `Bus::transfer_claim`, maps `BusError::{ClaimNotFound,ClaimNotOwned,ClaimExpired}` to JSON-RPC -32016 / -32017 / -32018, returns `{ok, claim_id, topic, offset, from_owner, to_owner, claimed_until}` on success.
- [ ] Router arm added to `crates/termlink-hub/src/router.rs` + governance allowlist entry. `cargo check -p termlink-hub` passes.

**Slice 3 — CLI verb:**
- [ ] `termlink channel claim-transfer --claim-id C --to-owner W --by B [--reason ...] [--json]` — clap subcommand under `ChannelAction`, dispatch in `main.rs`, impl in `crates/termlink-cli/src/commands/channel_claim_transfer.rs`. Renders human-format one-line summary; `--json` returns the raw RPC envelope.
- [ ] `cargo check -p termlink` passes; live smoke against local hub: claim → transfer → release succeeds end-to-end with distinct owner strings.

**Slice 4 — MCP tool + docs:**
- [ ] `termlink_channel_claim_transfer` MCP tool with `ChannelClaimTransferParams {claim_id, to_owner, by, reason?}` in `crates/termlink-mcp/src/tools.rs` + tool-index entry. `cargo check -p termlink-mcp` passes.
- [ ] `docs/operations/substrate-claim-primitive.md` extended with: (a) row in the surface table, (b) `transfer_claim` lifecycle entry, (c) cooperative-vs-force distinction, (d) end-to-end assign recipe (`agent.find_idle` → `channel.claim` → `channel.post` to `dm:O:W` → worker reads → `channel.transfer_claim` → process → `channel.release`).
- [ ] CLAUDE.md Quick Reference: row added cross-referencing MCP parity, the orchestrator recipe location, and the distinction vs `force_release`.

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

### 2026-06-08T10:49:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2046-substrate-primitive-3-channeltransfercla.md
- **Context:** Initial task creation

### 2026-06-08T13:20:44Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
