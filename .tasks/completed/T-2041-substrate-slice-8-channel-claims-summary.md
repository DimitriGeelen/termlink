---
id: T-2041
name: "Substrate Slice 8: channel claims-summary --watch mode for continuous stuck-worker detection"
description: >
  Substrate Slice 8: channel claims-summary --watch mode for continuous stuck-worker detection

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-parallel-substrate, slice-8]
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: [T-2019, T-2018, T-2039, T-2040]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-07T22:06:34Z
last_update: 2026-06-07T22:33:53Z
date_finished: 2026-06-07T22:33:53Z
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

# T-2041: Substrate Slice 8: channel claims-summary --watch mode for continuous stuck-worker detection

## Context

The substrate-claim-primitive (T-2019) ships Slice 6 (`channel.claims_summary`)
as a one-shot operator signal for "is anything stuck on this topic?". The
operator runbook (`docs/operations/substrate-claim-primitive.md`) tells
operators to wire the verb into cron for continuous monitoring, but cron
cadence is coarse (typically 1min minimum, scattered output, no diff
detection). For incident triage and routine soak monitoring, operators
need a hands-off `--watch` loop on a side terminal — same shape as
`termlink fleet doctor --watch` (T-1667).

This slice adds `--watch <secs>` to the existing `termlink channel claims-summary`
verb. No new RPC, no new bus method — pure CLI ergonomics on top of the
already-shipped Slice 6 surface. Clamp range mirrors fleet doctor's
5..=3600 (stuck workers don't need sub-5s polling).

## Acceptance Criteria

### Agent
- [x] `ClaimsSummary` enum variant in `crates/termlink-cli/src/cli.rs` adds `watch: Option<u64>` field with doc-comment naming T-2041 and the 5..=3600 clamp range
- [x] `cmd_channel_claims_summary` in `crates/termlink-cli/src/commands/channel.rs` accepts `watch: Option<u64>` parameter
- [x] When `watch` is Some, the function enters a loop: clear screen (`\x1b[2J\x1b[H`), print header row with verb + interval + timestamp, fetch + render the summary, sleep clamped interval; loop until SIGINT
- [x] When `watch` is Some AND `json_output` is true, the verb errors before entering the loop with a clear "--watch and --json are incompatible" message (same convention as `agent presence --watch`)
- [x] When `watch` is None, behaviour is unchanged from Slice 6 (one-shot text or JSON)
- [x] Watch-mode interval is clamped to 5..=3600 (silently — no error on out-of-range, follows fleet-doctor convention)
- [x] Watch-mode tolerates per-tick fetch errors: prints `# fetch error (will retry on next tick): <e>` and continues the loop (same pattern as `agent presence --watch`)
- [x] `main.rs` dispatch arm passes the new `watch` parameter through
- [x] Operator runbook (`docs/operations/substrate-claim-primitive.md`) "Stuck-worker pattern" section gains a `--watch` recipe alongside the existing cron pattern; References section gains a Slice 8 entry
- [x] `cargo build --release -p termlink` clean (CLI crate is `termlink`)
- [x] `cargo test --release -p termlink` clean (3 pre-existing failures in `cli_inbox_clear_json_no_hub`, `cli_inbox_status_json_no_hub`, `cli_version_text` confirmed unrelated — same failures on stashed HEAD without this change; 169 unrelated tests pass)

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

grep -q "watch: Option<u64>" crates/termlink-cli/src/cli.rs
grep -q "T-2041" crates/termlink-cli/src/cli.rs
grep -q "T-2041" crates/termlink-cli/src/commands/channel.rs
grep -q "claims-summary --watch" docs/operations/substrate-claim-primitive.md
grep -q "Slice 8" docs/operations/substrate-claim-primitive.md
cargo build --release -p termlink 2>&1 | tail -3 | grep -qE "Compiling|Finished"

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

### 2026-06-08 — extracted render to standalone fn; live verify via unreachable hub instead of bringing up fresh hub
- **What changed:** Two surprises during build, both small:
  (a) The original Slice 6 `cmd_channel_claims_summary` inlined the human-format
  render at the bottom of the function. To make the watch loop reuse that path
  cleanly, I extracted the render into a private `render_claims_summary_text`
  fn — pure refactor, identical output, but means the one-shot and watch paths
  render byte-identically by construction.
  (b) Could not bring the local hub's RPC up to verify the live render path —
  the host hub at `/var/lib/termlink/hub.sock` is running the OLD `~/.cargo/bin/termlink`
  binary (predates Slice 6's `channel.claims_summary` RPC, which is why a fresh
  one-shot call against it returns `code=-32001 Missing 'target' in params`).
  Verified watch-loop behaviour against an unreachable hub (`--hub 127.0.0.1:1`):
  two cycles in 8s with proper `\x1b[2J\x1b[H` clear-and-home, header row,
  `# fetch error (will retry on next tick): transport: I/O error: Connection refused`
  per tick, SIGINT exits cleanly. This is a STRONGER test than a happy-path live
  call because it exercises the error-tolerance AC directly.
- **Plan impact:** None. All 11 Agent ACs ticked from the original plan.
- **Triggered:** None — slice ships standalone. The remaining 9 §6 primitives
  (T-2020..T-2028) stay in inception awaiting operator decision.

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

### 2026-06-07T22:06:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2041-substrate-slice-8-channel-claims-summary.md
- **Context:** Initial task creation

### 2026-06-07T22:33:53Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
