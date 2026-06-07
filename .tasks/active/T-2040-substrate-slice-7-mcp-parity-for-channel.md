---
id: T-2040
name: "substrate Slice 7: MCP parity for channel.claims_summary (termlink_channel_claims_summary tool)"
description: >
  substrate Slice 7: MCP parity for channel.claims_summary (termlink_channel_claims_summary tool)

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
created: 2026-06-07T21:12:15Z
last_update: 2026-06-07T21:12:15Z
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

# T-2040: substrate Slice 7: MCP parity for channel.claims_summary (termlink_channel_claims_summary tool)

## Context

T-2039 (Slice 6) shipped `channel.claims_summary` aggregate RPC + bus + Rust client + CLI verb across the workspace. Slice 7 closes the substrate-first-primitive arc by adding the MCP wrapper so AI agents can query topic load + stuck-worker state via tool calls instead of shelling out. Same control-plane RPC pattern as T-2038 (Slice 5): `rpc_call` → `unwrap_result`, NO envelope signing (sibling of `termlink_channel_claims`, not `termlink_channel_pin`).

## Acceptance Criteria

### Agent
- [x] `ChannelClaimsSummaryParams { topic: String }` struct added to `crates/termlink-mcp/src/tools.rs` with `#[derive(Deserialize, JsonSchema)]`
- [x] `#[tool(name = "termlink_channel_claims_summary", ...)]` async method added; description explains the active-vs-expired aggregate framing and stuck-worker use case
- [x] Tool delegates to `termlink_session::client::rpc_call` with `method::CHANNEL_CLAIMS_SUMMARY` and unwraps via `unwrap_result` — same shape as `termlink_channel_claims` (Slice 5)
- [x] Help-registry entry added to the channel_admin group: `("termlink_channel_claims_summary", "Aggregate claim state ...")` — searchable via `termlink_help`
- [x] Release binary builds cleanly: `cargo build -p termlink --release` succeeds (Finished release profile in 6m 46s)
- [x] Tool name visible in release binary strings: `strings target/release/termlink | grep termlink_channel_claims_summary` returns the tool name + description text (3 matches: registry entry, tool macro, async fn)

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
grep -q "ChannelClaimsSummaryParams" crates/termlink-mcp/src/tools.rs
grep -q "termlink_channel_claims_summary" crates/termlink-mcp/src/tools.rs
grep -q "CHANNEL_CLAIMS_SUMMARY" crates/termlink-mcp/src/tools.rs
cargo build -p termlink > /tmp/.t2040.build.out 2>&1 && grep -q "Finished" /tmp/.t2040.build.out

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

### 2026-06-07 — slice-7 closes the substrate first-primitive arc
- **What changed:** with Slices 1-7 shipped (T-2029, T-2030, T-2031,
  T-2032, T-2033, T-2034, T-2035, T-2036, T-2037, T-2038, T-2039, T-2040),
  the exclusive-delivery claim primitive now has full coverage at every
  surface: hub RPC (claim/release/renew/claims/claims_summary), Rust client
  (low-level + `LeasedClaim` RAII), CLI verbs, MCP tools (5 tools total),
  runnable example, operator runbook with stuck-worker incident pattern,
  and integration test coverage. Operators get O(1) monitoring; AI agents
  get tool-callable observability; workers get RAII safety; the primitive
  is production-ready.
- **Plan impact:** the substrate first-primitive arc is structurally
  complete. Further refinement (metrics emission, fault-injection chaos
  tests, hub-side rate limiting) becomes optional polish, not blocking
  work. Decision-makers can now reason about the remaining 9 §6 primitives
  (T-2020..T-2028) using this primitive as a concrete reference.
- **Triggered:** none. Next substrate movement requires Tier-0 GO on one
  of the remaining §6 primitives; not autonomous.

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

### 2026-06-07T21:12:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2040-substrate-slice-7-mcp-parity-for-channel.md
- **Context:** Initial task creation
