---
id: T-2274
name: "MCP termlink_agent_contact gains hub param + fleet fallback resolution"
description: >
  T-2267 review item 4, slice 3 (findings 12). MCP termlink_agent_contact (tools.rs:17569) hardcodes the local socket (17596) and AgentContactParams (7388) has no hub field, so it cannot reach cross-hub at all. Add hub/hubs_file params; when local find_session (17621) misses, fall back to fleet presence resolution name->(hub,fp) via identity_fingerprint (T-2270 foundation). Depends on T-2270.

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
created: 2026-06-24T10:21:32Z
last_update: 2026-06-24T19:08:09Z
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

# T-2274: MCP termlink_agent_contact gains hub param + fleet fallback resolution

## Context

T-2267 review item 4, slice 4 — MCP parity for T-2275. The MCP handler
`termlink_agent_contact` (tools.rs:17569) resolves the target name via
`manager::find_session` IN-PROCESS (tools.rs:17621, local-only) and posts to the
LOCAL hub via `hub_socket_path()` (tools.rs:17596). `AgentContactParams`
(tools.rs:7388) has NO `hub` field. So an agent calling `termlink_agent_contact`
cannot reach a peer on another hub. This task adds the `hub` param + the same
cross-hub fleet fallback T-2275 adds to the CLI, reusing the shared
`termlink-session` parser (no second parse implementation). MCP already has its
own `connect_remote_hub_mcp` (tools.rs:6770) mirroring the CLI's, so the
per-crate transport pattern is already established here.

**Depends on T-2275** (the shared parser lands there). This task is the MCP
transport walker + param wiring.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `AgentContactParams` gains `hub: Option<String>` (documented JsonSchema doc-comment). When set, the dm post routes to that hub via the new `ContactHub::Remote` (`connect_remote_hub_mcp` + authed `channel.post`) instead of the local `hub_socket_path()` UDS path. Explicit `hub` wins over the auto-resolved fleet hub.
- [x] `termlink_agent_contact`: when `target` is given, `target_fp` is NOT set, and `manager::find_session` misses, falls back to `resolve_contact_via_fleet_mcp` — `list_all_hub_profiles()` → per hub `connect_remote_hub_mcp(observe)` + `ContactHub::fetch_recent("agent-presence")` → the SHARED `termlink_session::fleet_presence::resolve_agent_presence` parser (same fn T-2275 added — no second parse) → freshest LIVE match. Resolves `{identity_fingerprint, hub}` and posts to that hub. Per-hub failures `continue` (down hub never aborts).
- [x] No-match + regression: local `find_session` hit and `target_fp` bypass leave `fleet_hub=None` → `ContactHub::Local` (unauthenticated UDS, unchanged path); the fleet walk is entered only on a local Err; no-match returns `json_err` naming both local + fleet + pointing at `termlink_agent_listeners_fleet`. `cargo check -p termlink-mcp` clean. The whole create/post/probe/ack path now routes through one `ContactHub` so local + remote share logic.
- [x] The new `hub` property is in the tool input-schema by construction (`#[derive(JsonSchema)]` on `AgentContactParams` auto-includes the `pub hub` field with its doc-comment). Dry-run response now also echoes `hub` + `routing` (`local`/`remote`) for caller transparency.

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

cargo test -p termlink-session fleet_presence
cargo check -p termlink-mcp

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

### 2026-06-24T10:21:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2274-mcp-termlinkagentcontact-gains-hub-param.md
- **Context:** Initial task creation

### 2026-06-24T18:59:45Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-06-24 — implemented (MCP parity, shared parser reused)
- **`hub` param** added to `AgentContactParams` (JsonSchema doc-commented).
- **`ContactHub` enum** (tools.rs, near connect_remote_hub_mcp): `Local(PathBuf)`
  (unauthenticated UDS, legacy) | `Remote(Box<Client>)` (authed remote). Methods
  `rpc(method, params) -> unwrapped result` + `fetch_recent(topic, slice)`. The
  whole agent_contact create/post/probe/ack path now routes through one `conn`,
  so local + remote share logic (no dual code path duplication).
- **`resolve_contact_via_fleet_mcp(agent_id)`**: walks `list_all_hub_profiles()`,
  dedups by address, `connect_remote_hub_mcp(observe)` + `fetch_recent(
  "agent-presence")` per hub, runs the SHARED
  `termlink_session::fleet_presence::resolve_agent_presence` (same fn the CLI uses
  — no second parse), returns freshest LIVE `(fp, hub_address)`. Per-hub Err →
  continue.
- **Wiring:** find_session-miss arm calls the walker; on hit sets `fleet_hub` +
  peer_fp. `target_hub = p.hub.or(fleet_hub)` selects the transport; explicit
  `hub` wins. Local existence-check moved into the `ContactHub::Local` arm (a
  pure-remote contact no longer needs a running local hub). Dry-run echoes
  `hub` + `routing`.
- **Build:** `cargo check -p termlink-mcp` clean; shared parser 7/7.
- **Runtime note (honest):** the local + dry-run paths are fully exercisable
  here; the *remote* post path is compile-verified + logic-mirrors the CLI
  (T-2275, runtime-proven via T-2273's shell equivalent) + relies on the
  unit-tested shared parser. A live cross-hub MCP post is an operator field-test
  (cannot safely post to a remote shared hub from this session).
- **Deploy:** one `cargo build --release` covers CLI + MCP; install + MCP
  reconnect is the operational step.

### 2026-06-24 — code complete + pushed; DEPLOY pending
- Commit `4546f412` (MCP) pushed to OneDev. ACs ticked; `cargo check -p
  termlink-mcp` clean; shared parser 7/7.
- **DEPLOY pending (operational, next session):** release build was running when
  the context-window gate fired at ~95%. To activate the MCP change:
  `cargo build --release && install -m755 target/release/termlink ~/.cargo/bin/termlink`,
  then **reconnect the termlink MCP server** (the running server holds the old
  binary in memory until restarted). After reconnect, `termlink_agent_contact`
  with a cross-hub `target` (or explicit `hub`) routes correctly.
- **Field-test (operator):** from an MCP client, `termlink_agent_contact
  {target: "<peer-on-another-hub>", message: "...", dry_run: true}` should return
  `routing: "remote"` + the resolved `hub`; a non-dry-run delivers cross-hub.
