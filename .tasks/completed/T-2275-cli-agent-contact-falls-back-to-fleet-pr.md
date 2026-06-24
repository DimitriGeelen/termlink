---
id: T-2275
name: "CLI agent contact falls back to fleet presence when local find_session misses"
description: >
  T-2267 review item 4, slice 4 (findings 11,16). CLI cmd_agent_contact (agent.rs:804-850) resolves target name via local manager::find_session only (817); a peer on another hub is invisible -> 'Session not found'. Add a fleet-presence fallback resolving name->(hub,fp) via identity_fingerprint (T-2270 foundation), use the resolved hub for the dm post, and on not-found steer toward fleet presence rather than broadcast. Depends on T-2270.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/agent.rs, crates/termlink-session/src/lib.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-24T10:21:39Z
last_update: 2026-06-24T19:46:43Z
date_finished: 2026-06-24T19:46:43Z
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

# T-2275: CLI agent contact falls back to fleet presence when local find_session misses

## Context

T-2267 review item 4, slice 3 (Rust parity for T-2273's shell work). PL-107 /
T-1429 Phase-2 gap: `termlink agent contact <name>` resolves `<target>` via
`manager::find_session` (agent.rs:817) which is LOCAL-ONLY — a peer on another
hub yields "not found", indistinguishable from "offline". The shell layer
(agent-send.sh `--to`, T-2273) already does cross-hub contact-by-name by walking
`agent-listeners-fleet.sh` (agent-presence per hub) for the peer's
`identity_fingerprint` + `hub`. This task brings the SAME fallback natively into
`cmd_agent_contact` so the binary works cross-hub without depending on repo
scripts being present on the deployed host (the professional/reliable solution
chosen over shell-out, 2026-06-24).

**Design (confirmed via code map):** the native post path already supports remote
hubs — `cmd_channel_dm` takes `hub: Option<&str>`, and `parse_hub_addr` +
`resolve_hub_secret_hex` (T-1385/T-2269) reverse-resolve the secret from
`hubs.toml`. `cmd_agent_contact` already threads `hub` into `cmd_channel_dm`. The
ONLY gap is the name→{fp,hub} RESOLUTION when `find_session` misses. The
agent_id→{fp,hub,pty_session} resolver does NOT exist in Rust (only in
agent-listeners-fleet.sh). Factoring: a **shared pure parser** in
`termlink-session` (heartbeat-envelope → match, zero drift) + a **per-crate fleet
walker** (transport, mirroring the existing connect_remote_hub/_mcp duplication).
Parse contract (from agent-listeners.sh): filter `msg_type=="heartbeat" &&
metadata.agent_id!=""`, newest per agent_id by ts, `identity_fingerprint =
sender_id` (envelope top-level), `pty_session = metadata.pty_session`, status from
age vs `2×interval` (LIVE) / `5×interval` (STALE) using `metadata.interval_secs`
(default 30).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Shared pure parser added to `termlink-session` (`fleet_presence.rs`, new module): given `&[serde_json::Value]` agent-presence heartbeats + `agent_id` + `now_ms`, returns the newest matching heartbeat's `{identity_fingerprint (=sender_id), pty_session, status, age_secs}` or `None`. Status classification matches agent-listeners.sh (`2×interval`→LIVE, `5×interval`→STALE, else OFFLINE; default interval 30). Unit-tested: LIVE/STALE/OFFLINE bands, no-match→None, newest-wins-on-duplicate, non-heartbeat-ignored, default-interval. (7 tests pass.)
- [x] `cmd_agent_contact` (agent.rs): when positional `<target>` is given, `--target-fp` is NOT set, and `manager::find_session` returns Err, falls back to a fleet walk — `load_hubs_config()` → per hub `fetch_topic_msgs("agent-presence", Some(addr))` (T-2269 secret reverse-resolution) → shared parser → freshest LIVE match across hubs (dedup by address). Sets `peer_fp = identity_fingerprint` and routes the dm post to the matched hub via `hub = hub.or(fleet_hub)` (explicit `--hub` wins). Per-hub failures `continue` (down hub never aborts the walk).
- [x] No-match path: when neither local `find_session` nor any fleet hub yields a LIVE agent_id, the error names both ("not found locally or as a LIVE peer on any hub in hubs.toml") + points at `agent listeners --fleet`/`/peers`/`--target-fp`; empty/missing hubs.toml → `resolve_contact_via_fleet` returns None early (no panic).
- [x] Regression preserved: `--target-fp <hex>` bypass and a successful local `find_session` hit both leave `fleet_hub = None` (walk only entered on local Err), so routing is unchanged. `cargo check -p termlink` succeeds; `cargo test -p termlink-session fleet_presence` passes (7/7).

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
cargo check -p termlink

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

### 2026-06-24T10:21:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2275-cli-agent-contact-falls-back-to-fleet-pr.md
- **Context:** Initial task creation

### 2026-06-24T18:52:05Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-06-24 — implemented (native Rust cross-hub resolver)
- **Shared parser:** new `crates/termlink-session/src/fleet_presence.rs` —
  `resolve_agent_presence(msgs, agent_id, now_ms) -> Option<PresenceMatch>` +
  `PresenceStatus`. Pure; mirrors agent-listeners.sh classification. 7 unit tests
  pass (`cargo test -p termlink-session fleet_presence`). Registered in lib.rs.
- **CLI walker:** `resolve_contact_via_fleet(agent_id)` in agent.rs — walks
  `load_hubs_config()`, dedups by address, `fetch_topic_msgs("agent-presence",
  Some(addr), 500)` per hub (reuses T-2269 secret reverse-resolution), runs the
  shared parser, returns the freshest LIVE `{identity_fingerprint, hub_address}`.
  Per-hub Err → `continue` (down hub never aborts).
- **Wiring:** `cmd_agent_contact` find_session-Err arm now calls the walker; on a
  hit sets `fleet_hub` + peer_fp; routing shadow `let hub = hub.or(fleet_hub)`
  threads the resolved hub through the dry-run preview, require-online probe, dm
  post, and ack-wait. `--target-fp` + local-hit paths untouched (fleet_hub stays
  None). No-match error names local + fleet.
- **Why native over shell-out:** chosen as the professional/reliable solution
  (2026-06-24) — a shipped binary must not depend on repo scripts being present
  on the deployed host.
- **Build:** `cargo check -p termlink` clean.
- **Next:** T-2274 (MCP parity, reuses this same shared parser) → one release
  build + deploy covers both.

### 2026-06-24 — code complete + pushed; DEPLOY pending
- Commit `fb31f978` (CLI + shared parser) pushed to OneDev. ACs ticked; parser
  7/7; `cargo check -p termlink` clean.
- **DEPLOY pending (operational, next session):** a `cargo build --release` was
  running when the context-window budget gate fired at ~95%. To activate:
  `cargo build --release && install -m755 target/release/termlink ~/.cargo/bin/termlink`
  (mirror to peers via `scripts/fleet-deploy-binary.sh` if fleet-wide). The CLI
  change takes effect on the next `termlink agent contact` invocation.
- Closing this task records code-completion; the binary swap is a deploy step.

### 2026-06-24 — DEPLOY done (CLI live)
- Clean `cargo build --release` from HEAD (fce28982) exit 0; installed
  `target/release/termlink` → `~/.cargo/bin/termlink`. `termlink --version`
  now `0.11.20` (matches VERSION; no more `/preflight` Check-4 stale-binary
  WARN). Fleet-fallback string present in the shipped binary (verified via
  `strings`). **The CLI change is live** — `termlink agent contact <peer>`
  now resolves cross-hub peers by name via the fleet walk.
- A first build had baked a cosmetic-stale `0.11.15` version string (built
  mid-arc before the doc commits); rebuilt from committed HEAD for a
  reproducible, version-matched artifact rather than shipping the ambiguous one.

### 2026-06-24T19:46:43Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
