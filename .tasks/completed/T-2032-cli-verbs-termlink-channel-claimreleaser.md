---
id: T-2032
name: "CLI verbs: termlink channel claim/release/renew (substrate user-facing surface, T-2031 follow-up)"
description: >
  Slice-3 follow-up. The Rust LeasedClaim API is shipped (T-2031); now expose it via CLI verbs so operators can directly claim/release/renew offsets from the command line. Builds on the channel_claim/release/renew helpers in crates/termlink-session/src/claim_client.rs. Pattern: mirror the existing ChannelAction enum + commands/channel.rs cmd_X functions + main.rs dispatch table style used for the ~30 existing verbs.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:arc-parallel-substrate]
components: []
related_tasks: [T-2031, T-2019, T-2018]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-07T16:43:49Z
last_update: 2026-06-07T17:11:31Z
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

# T-2032: CLI verbs: termlink channel claim/release/renew (substrate user-facing surface, T-2031 follow-up)

## Context

T-2031 shipped `crates/termlink-session/src/claim_client.rs` with low-level
async wrappers `channel_claim/channel_release/channel_renew` plus the
`LeasedClaim` RAII helper. This task surfaces the three primitives as CLI
verbs so operators can drive the substrate without writing Rust:

```
termlink channel claim <topic> <offset> --claimer <id> [--ttl-ms N]
termlink channel renew <claim-id> --claimer <id> [--additional-ttl-ms N]
termlink channel release <claim-id> --claimer <id> [--ack]
```

ADR: `docs/architecture/parallel-execution-substrate.md` §6 manifest.
Pattern: mirror the existing ~30 channel verbs (Post / Subscribe / Info /
Pin / Star / etc.) — `ChannelAction` enum variant in `cli.rs`, dispatch arm
in `main.rs`, `cmd_channel_X` async function in `commands/channel.rs`.

## Acceptance Criteria

### Agent
- [x] Three new variants on `ChannelAction` in `crates/termlink-cli/src/cli.rs`:
  `Claim { topic, offset, claimer, ttl_ms, hub, json }`,
  `Release { claim_id, claimer, ack, hub, json }`,
  `Renew { claim_id, claimer, additional_ttl_ms, hub, json }` — flag shapes
  match the existing channel verbs (hub: Option<String>, json: bool).
- [x] Three new `cmd_channel_*` async functions in
  `crates/termlink-cli/src/commands/channel.rs` that call the
  `termlink_session::claim_client::{channel_claim, channel_release,
  channel_renew}` wrappers and render either human-readable output (one
  line per response field) or JSON envelopes.
- [x] Dispatch arms in `crates/termlink-cli/src/main.rs` route each
  ChannelAction variant to its cmd_channel_* function.
- [x] Hub-target resolution mirrors existing verbs (use the
  `target::resolve_hub_target` helper or its equivalent — same address /
  fallback handling as `channel post`).
- [x] Typed `ClaimError` variants map to actionable CLI exit messages
  (Conflict → "offset already claimed", NotFound → "claim not found",
  NotOwned → "not your claim", Expired → "claim lease expired") on stderr
  with non-zero exit codes.
- [x] Integration smoke test against a FakeHub (matching pattern used in
  `tests/claim_client_integration.rs`) exercising the CLI binary
  end-to-end for at least one verb's success + one error case.
- [x] `cargo build --release -p termlink-cli` succeeds.
- [x] `cargo test --release -p termlink-cli` succeeds.
- [x] `target/release/termlink channel claim --help` exits 0 with the new
  verb in the help output (smoke test for clap wiring).

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

cargo build --release -p termlink 2>&1 | tail -3 | grep -q "Compiling\|Finished\|warning"
target/release/termlink channel claim --help 2>&1 | grep -q "claimer"
target/release/termlink channel release --help 2>&1 | grep -q "claim-id\|claim_id"
target/release/termlink channel renew --help 2>&1 | grep -q "claim-id\|claim_id"
grep -q "fn cmd_channel_claim\|fn cmd_channel_release\|fn cmd_channel_renew" crates/termlink-cli/src/commands/channel.rs

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

### 2026-06-07 — Substrate primitive becomes operator-callable
- **What changed:** Wired three CLI verbs on top of the T-2031 Rust client
  surface. Operators can now drive the claim/release/renew substrate
  primitive directly from the command line without writing Rust. This is
  the "last mile" of the §6 manifest's first primitive — Slices 1+2+3 +
  CLI together = end-to-end usable.
- **What changed:** The original AC said hub-target resolution should use
  `target::resolve_hub_target` "or equivalent". The actual existing helper
  used by channel verbs is `hub_socket(hub: Option<&str>)` in
  `commands/channel.rs`. Used that — same address parsing, same default
  to `<runtime_dir>/hub.sock` for no-hub case.
- **What changed:** The CLI test uses tokio::process::Command instead of
  the cli_integration.rs sync `Command` pattern, so it can `await` the
  FakeHub tokio task alongside the subprocess. Cleaner than the
  `assert_cmd::Command` blocking variant for this hybrid scenario.
- **Plan impact:** Original AC said `target::resolve_hub_target` — that
  helper doesn't exist on this code path; the channel-verb-local helper
  is `hub_socket`. AC body still satisfied (same address/fallback
  semantics), just different function name.
- **Triggered:** No new tasks. Next natural slice is MCP parity (a
  `termlink_channel_claim` MCP tool family) so agents can drive the
  substrate via Model Context Protocol the same way operators drive it
  via CLI. Not filed under T-2032 — wait for first consuming agent to
  pull it.

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

### 2026-06-07T16:43:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2032-cli-verbs-termlink-channel-claimreleaser.md
- **Context:** Initial task creation

### 2026-06-07T19:30Z — operator/developer recipe doc
- **Action:** Added `docs/operations/substrate-claim-primitive.md` — the
  worker recipe + diagnostic runbook for the primitive shipped across
  T-2029/T-2030/T-2031/T-2032. Covers the four lifecycle paths, TTL
  selection guidance, Rust worker pattern (LeasedClaim usage + worker
  loop), CLI quick-tour, diagnostic recipes ("is this stuck on a dead
  worker?"), full error-code reference, and the intentional scope cuts
  (no work-distribution, no worker discovery — those are §6 manifest
  primitives still in inception).
- **Output:** `docs/operations/substrate-claim-primitive.md`
- **Context:** Closes the loop on the "is it operator-callable AND
  documented?" gate. Operators have CLI verbs; developers have a worker
  pattern they can compile from. Substrate primitive #1 is genuinely
  ready for first consumer adoption.

### 2026-06-07T19:00Z — CLI verbs shipped
- **Action:** Added three new `ChannelAction` variants (Claim/Renew/Release)
  to cli.rs (~70 new lines of clap argument shapes with doc comments), three
  new `cmd_channel_*` async functions in commands/channel.rs (~115 lines —
  thin wrappers over the T-2031 claim_client surface), three dispatch arms
  in main.rs (~45 lines), and a new CLI integration test
  `tests/channel_claim_cli_integration.rs` (~300 lines, FakeHub pattern).
- **Output:**
  - `crates/termlink-cli/src/cli.rs` (modified)
  - `crates/termlink-cli/src/commands/channel.rs` (modified)
  - `crates/termlink-cli/src/main.rs` (modified)
  - `crates/termlink-cli/tests/channel_claim_cli_integration.rs` (new)
- **Tests:**
  - 3 new CLI integration tests covering claim+release roundtrip via
    subprocess+FakeHub, conflict-exits-nonzero, and renew-extends-claim —
    all pass.
    `cargo test --release -p termlink --test channel_claim_cli_integration`
  - Regression: termlink CLI 817/817 tests pass (--bin termlink full suite).
  - Smoke: `target/release/termlink channel {claim,renew,release} --help`
    all exit 0 with expected flag names in output.
- **Context:** Closes the substrate primitive's last-mile (Rust API +
  CLI verbs). Operators can now drive the claim/release/renew flow from
  the command line.
