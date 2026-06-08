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
last_update: 2026-06-08T14:04:41Z
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
- [x] `CHANNEL_TRANSFER_CLAIM = "channel.transfer_claim"` constant in `crates/termlink-protocol/src/control.rs` with full doc-comment naming the error taxonomy and cooperative-vs-force distinction. Assert added to the const test block.
- [x] `handle_channel_transfer_claim` + `handle_channel_transfer_claim_with` in `crates/termlink-hub/src/channel.rs` — mirrors `handle_channel_renew` shape, parses `{claim_id, to_owner, by, reason?}`, maps `BusError::{ClaimNotFound,ClaimNotOwned,ClaimExpired}` to `-32016 / -32017 / -32018` via `error_code::*`, returns `{ok, claim_id, topic, offset, from_owner, to_owner, claimed_at, claimed_until, reason}`.
- [x] Router arm at `crates/termlink-hub/src/router.rs:114` (between FORCE_RELEASE and RENEW), governance allowlist entry next to FORCE_RELEASE. `cargo check -p termlink-hub` passes.

**Slice 3 — CLI verb:**
- [x] `termlink channel claim-transfer --claim-id C --to-owner W --by B [--reason ...] [--json]` — `ChannelAction::ClaimTransfer` clap variant in `cli.rs`, dispatch in `main.rs`, impl `cmd_channel_claim_transfer` in `commands/channel.rs`, session client `channel_transfer_claim` + `TransferSummary` + `parse_transfer_response` in `claim_client.rs`. Renders one-line human summary `<claim_id> transferred <topic>:<offset> from <from> → <to>` (+ optional reason line + lease-preserved note); `--json` returns the raw RPC envelope.
- [x] `cargo check -p termlink` passes; live smoke 2026-06-08T14:25Z against local hub (post `cargo install --path crates/termlink-cli --force` + `systemctl restart termlink-hub`): claim → transfer → release end-to-end. **All 4 paths green:** (a) happy `orch → worker-A` with reason returned `{from_owner:orch, to_owner:worker-A, claimed_at:1780928755344, claimed_until:1780928815344, reason:"T-2046 live smoke"}` — lease ts preserved; (b) wrong `by=imposter` → "claim ... is held by another claimer" (CLAIM_NOT_OWNED); (c) bogus claim_id → "claim ... not found (never existed, released, or expired)" (CLAIM_NOT_FOUND); (d) release by new owner-A with `ack=true` succeeded — confirms transfer was atomic + owner-bound.

**Slice 4 — MCP tool + docs:**
- [x] `termlink_channel_claim_transfer` MCP tool with `ChannelClaimTransferParams {claim_id, to_owner, by, reason?}` in `crates/termlink-mcp/src/tools.rs` + tool-index entry between FORCE_RELEASE and RENEW. `cargo check -p termlink-mcp` passes.
- [x] `docs/operations/substrate-claim-primitive.md` extended: (a) row in surface table with full description, (b) new diagnostic recipe section "Hand a unit to a specific worker without a race window" with end-to-end shell example (find-idle → claim → channel.post DM → claim-transfer → release), (c) explicit cooperative-vs-force distinction in body, (d) References entry "Primitive #3 (T-2046, T-2021 GO)" closing the substrate primitive series.
- [x] CLAUDE.md Quick Reference row "Transfer claim ownership (ASSIGN)" added between Find-idle and Cross-host-handoff rows. Cross-references MCP parity, the assign recipe doc location, and the claim-force-release distinction.

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
cargo check -p termlink-bus
cargo check -p termlink-protocol
cargo check -p termlink-hub
cargo check -p termlink
cargo check -p termlink-mcp
cargo check -p termlink-session
out=$(cargo test -p termlink-bus transfer_claim 2>&1); echo "$out" | grep -q "6 passed"
test -f docs/operations/substrate-claim-primitive.md
out=$(grep -F "channel.transfer_claim" docs/operations/substrate-claim-primitive.md); echo "$out" | grep -q "atomic"
out=$(grep -F "claim-transfer" CLAUDE.md); echo "$out" | grep -q "ASSIGN"
out=$(./target/release/termlink channel claim-transfer --help 2>&1); echo "$out" | grep -q "claim-transfer"

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

### 2026-06-08 — slice 1: lease timestamps survive transfer; only `claimed_by` mutates
- **What changed:** The T-2021 inception specified the RPC shape but left "what about ttl?" implicit. During slice 1 it became clear that the cleanest semantics are: transfer is purely ownership advance — `claimed_at` and `claimed_until` are preserved. Renew is a separate verb; transfer is not a re-lease. Workers wanting a longer window after receiving a transferred claim call `channel.renew(by=new_owner, additional_ttl_ms=...)` as a follow-up.
- **Plan impact:** None for code shape; clarified the response envelope and the doc story. Lease-preservation IS the natural single-row UPDATE; mixing ttl reset would require a now-ms parameter through the bus layer, defeating the simplicity.
- **Triggered:** No new task. Captured in `TransferInfo` doc-comment + the "Hand a unit to a specific worker" recipe (lease-preserved note in the human format CLI output).

### 2026-06-08 — slice 3: live smoke required cargo install + systemctl restart
- **What changed:** Unlike T-2045 where `./target/release/termlink hub stop && hub start` worked because nothing else was managing the hub, T-2046's hub-side surface (transfer_claim handler) is hosted in the systemd-managed `termlink-hub.service` on this host. The local `./target/release/termlink` binary alone cannot validate the RPC end-to-end — the hub PID must come from the upgraded binary. The deploy step is therefore canonical: `cargo install --path crates/termlink-cli --force` (replaces `/root/.cargo/bin/termlink`) → `systemctl restart termlink-hub` (clean restart with `/var/lib/termlink` persistent runtime_dir → no client re-pin, no broken connections).
- **Plan impact:** Captured the install + restart sequence as the validated path for any substrate hub-side primitive. Future T-2018 primitives that add hub handlers will hit this same wall — surfacing it now means they can pre-budget the 6-minute release build.
- **Triggered:** No new task. Should the `cargo install` step ever become a blocker (e.g. cross-host parity), file a deploy-tooling task. For now the local hub fully validates the protocol.

### 2026-06-08 — slice 4: assign recipe lives in substrate-claim-primitive.md (not a new file)
- **What changed:** Considered splitting the assign recipe into its own `docs/operations/substrate-assign-primitive.md` for symmetry with `agent-find-idle.md`. Decided against — transfer_claim is fundamentally a verb in the claim lifecycle, not a separate primitive's worth of documentation surface. Adding the recipe to the existing claim primitive doc (in the Diagnostic Recipes section, between force-release and the error reference) keeps related material co-located.
- **Plan impact:** Avoided creating a third operations doc. The existing doc gained one row in the surface table, one recipe section, and one References entry — still all sub-300-line growth.
- **Triggered:** No new task.

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
