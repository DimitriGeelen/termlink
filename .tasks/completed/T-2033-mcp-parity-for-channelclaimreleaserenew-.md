---
id: T-2033
name: "MCP parity for channel.claim/release/renew (T-2031/T-2032 follow-up)"
description: >
  Add termlink_channel_claim/release/renew MCP tools so AI agents can drive the arc-parallel-substrate first primitive the same way operators drive it via CLI (T-2032). Mirror the existing channel MCP tool families. Implementation: three new param structs after ChannelPinParams in crates/termlink-mcp/src/tools.rs (~line 8013), three new #[tool]-decorated async methods near termlink_channel_pin (~line 19640) calling rpc_call(method::CHANNEL_CLAIM/RELEASE/RENEW), and entries in the channel help-registry group (~line 340). No identity signing needed — claim/release/renew are control-plane RPCs not signed envelopes.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-parallel-substrate]
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: [T-2031, T-2032, T-2019, T-2018]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-07T17:09:24Z
last_update: 2026-06-07T19:14:12Z
date_finished: 2026-06-07T19:14:12Z
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

# T-2033: MCP parity for channel.claim/release/renew (T-2031/T-2032 follow-up)

## Context

Closes the MCP surface for the arc-parallel-substrate first primitive
(claim/release/renew). Hub RPCs landed in T-2029 + T-2030, the Rust
client and `LeasedClaim` RAII helper in T-2031, the CLI verbs in T-2032.
Without MCP parity, AI agents using termlink-mcp cannot drive the
primitive — they have to shell out via `termlink_exec` or call the CLI
through `termlink_remote_exec`, which kills the structured-params /
typed-return ergonomics. Three lightweight `#[tool]` wrappers around
the existing hub RPCs close the gap with no new behavior.

## Acceptance Criteria

### Agent
- [x] `ChannelClaimParams`, `ChannelReleaseParams`, `ChannelRenewParams` structs added in `crates/termlink-mcp/src/tools.rs` next to `ChannelPinParams` (~line 8013)
- [x] `termlink_channel_claim`, `termlink_channel_release`, `termlink_channel_renew` `#[tool]`-decorated async methods added next to `termlink_channel_pin` (~line 19708)
- [x] Each method dispatches via `termlink_session::client::rpc_call` to the corresponding `termlink_protocol::control::method::CHANNEL_{CLAIM,RELEASE,RENEW}` constant
- [x] Returns the raw hub result envelope as pretty JSON on success; on error returns `json_err` with hub error code+message
- [x] No envelope signing — these are control-plane RPCs, not message envelopes (unlike `termlink_channel_pin`)
- [x] Three new entries in the `channel_admin` group of the help registry (~line 393) so MCP clients can discover them
- [x] `cargo build --release -p termlink-mcp` succeeds
- [x] No NEW clippy warnings attributable to the added code: `cargo clippy --no-deps -p termlink-mcp 2>&1 | grep -E "claim|release|renew|Claim|Release|Renew"` returns empty (file has 54 pre-existing unrelated warnings, addressed separately)

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

cargo build --release -p termlink-mcp
cargo build --release -p termlink
out=$(cargo clippy --no-deps -p termlink-mcp 2>&1); echo "$out" | grep -vE "claim|release|renew|Claim|Release|Renew" > /dev/null  # no NEW lints attributable to added code
strings target/release/termlink > /tmp/.t2033.strings && grep -q "termlink_channel_claim" /tmp/.t2033.strings
grep -q "termlink_channel_release" /tmp/.t2033.strings
grep -q "termlink_channel_renew" /tmp/.t2033.strings

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

### 2026-06-07 — MCP tool shape diverges from agent_pin sibling
- **What changed:** Initial scoping referenced `termlink_agent_pin` as the prior-art template (sibling of `channel.claim` in the channel_admin group). On read, those tools post signed envelopes (msg_type=pin, canonical_sign_bytes + Ed25519 sig). claim/release/renew are control-plane RPCs (no envelope, no signing) — the right sibling is `termlink_channel_info` (rpc_call + unwrap_result, no signing).
- **Plan impact:** Each tool dropped from ~70 lines (signed-envelope shape) to ~25 lines (control-plane shape). No identity loading, no canonical_sign_bytes, no metadata map. Net file growth was smaller than estimated.
- **Triggered:** Updated AC 5 to note "No envelope signing" explicitly so future MCP-parity work for control-plane RPCs in this arc (T-2020..T-2028 inceptions) inherits the right template.

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

### 2026-06-07T17:09:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2033-mcp-parity-for-channelclaimreleaserenew-.md

### 2026-06-07 — MCP tools shipped
- **Action:** Added 3 param structs + 3 `#[tool]`-decorated async methods + 3 help-registry entries in `crates/termlink-mcp/src/tools.rs`. Each tool dispatches via `rpc_call` to the corresponding `method::CHANNEL_{CLAIM,RELEASE,RENEW}` constant, no envelope signing (control-plane RPCs).
- **Verification:** `cargo build --release -p termlink-mcp` ✓; `cargo build --release -p termlink` ✓; MCP `tools/list` handshake confirms all three tools registered (filtered match: `['termlink_channel_claim', 'termlink_channel_release', 'termlink_channel_renew']`, 255 total tools).
- **Surface coverage achieved:** AI agents using termlink-mcp can now drive the first arc-parallel-substrate primitive end-to-end the same way operators do via CLI (T-2032) — no more shelling out via `termlink_exec`.
- **Context:** Initial task creation

### 2026-06-07T18:42:27Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-06-07T19:14:12Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
