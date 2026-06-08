---
id: T-2045
name: "Substrate primitive #2: hub-owned idle/busy agent registry (T-2020 GO build slice)"
description: >
  Implement the T-2020 GO decision per docs/reports/T-2020-idle-busy-registry-inception.md. Derivation-based: idle_agents = LIVE(presence) \ DISTINCT(claimed_by). Add metadata.capabilities: [string] to heartbeat. Surface as agent.find_idle RPC + termlink agent find-idle CLI. ~150 LOC across 5 vertical slices.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-parallel-substrate, substrate-primitive, foundation]
components: [crates/termlink-bus/src/lib.rs, crates/termlink-bus/src/meta.rs, crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/mod.rs, crates/termlink-cli/src/main.rs, crates/termlink-hub/src/channel.rs, crates/termlink-hub/src/router.rs, crates/termlink-mcp/src/tools.rs, crates/termlink-protocol/src/control.rs, scripts/be-reachable.sh]
related_tasks: [T-2018, T-2020, T-2019, T-2021]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-08T10:48:48Z
last_update: 2026-06-08T13:19:02Z
date_finished: 2026-06-08T13:19:02Z
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

# T-2045: Substrate primitive #2: hub-owned idle/busy agent registry (T-2020 GO build slice)

## Context

T-2020 GO build slice. Server-side derivation: `idle_agents = LIVE(agent-presence) \ DISTINCT(claims.claimed_by)`. No new persistent state — joins existing presence topic + claims table. See `docs/reports/T-2020-idle-busy-registry-inception.md` §5 for the design.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
**Slice 1 — RPC + bus library + unit tests:**
- [x] `agent.find_idle` method constant added to `termlink-protocol` (`control.rs:212`)
- [x] Bus library derivation function: walks `agent-presence` topic, dedups by `agent_id` keeping latest envelope, filters to LIVE (heartbeat newer than 2×interval, default 60s), anti-joins against active claims (`claimed_until > now`), sorts by `last_heartbeat_ms` desc (`Bus::find_idle_agents` in `bus/src/lib.rs`)
- [x] Hub router arm for `agent.find_idle` with params `{role?: string, capabilities?: [string], limit?: u32}` → `{ok, idle: [...]}` (`hub/src/channel.rs::handle_agent_find_idle`, router arm at `hub/src/router.rs:120`)
- [x] Unit tests cover: (a) presence-only-no-claims returns all LIVE, (b) presence-with-claims excludes claimed_by, (c) stale (>2×interval) excluded, (d) role filter, (e) capabilities filter (subset match), (f) empty presence → empty result — plus dedup-by-agent_id and limit-truncates-after-sort tests. **8/8 tests passing** (`cargo test -p termlink-bus find_idle`)
- [x] `cargo check -p termlink` and `cargo check -p termlink-hub` pass

**Slice 2 — CLI verb:**
- [x] `termlink agent find-idle [--role R] [--capability C] [--limit N] [--json]` calls the RPC — `AgentAction::FindIdle` in `cli.rs`, dispatch in `main.rs`, impl in `commands/agent_find_idle.rs`
- [x] Human-format output: one agent per line with id/age/role/capabilities; `--json` returns the raw array
- [x] Live smoke against a real hub returns at least the local-session agent_id — verified 2026-06-08T13:15Z post-hub-restart on the bhg34ttiq build. `/be-reachable start --agent-id smoke-T2045-claude` with `TERMLINK_CAPABILITIES="claude-code,rust,smoke-test"` produced `idle:[{agent_id:smoke-T2045-claude, capabilities:[claude-code,rust,smoke-test], role:claude-code, last_heartbeat_ms:1780924555851}]`. Filters exercised live: `--capability rust` matches; `--capability python` empty; `--role claude-code` matches; human-format renders `smoke-T2045-claude\tage=16s\trole=claude-code\tcapabilities=claude-code,rust,smoke-test`

**Slice 3 — MCP tool:**
- [x] `termlink_agent_find_idle` MCP tool with params `{role?, capabilities?, limit?}` — `AgentFindIdleParams` + handler in `crates/termlink-mcp/src/tools.rs`, registered in tool index. `cargo check -p termlink-mcp` passes.

**Slice 4 — Heartbeat schema:**
- [x] `metadata.capabilities` (comma-separated string) supported on the server side — bus library reads it, treats absent as empty set; covered by `find_idle_capabilities_subset_match` unit test
- [x] `listener-heartbeat.sh` reads `TERMLINK_CAPABILITIES` env (comma-separated) and emits in heartbeat metadata — `--capabilities` flag + env-default fallback; omits the field when empty for backward-compat
- [x] `/be-reachable` wrapper exposes `--capabilities` flag — passes through to listener-heartbeat.sh

**Slice 5 — Docs + example:**
- [x] `docs/operations/agent-find-idle.md` with runnable orchestrator example: find-idle → claim → release — written, mirrors substrate-claim-primitive.md style; includes derivation, filter semantics, end-to-end be-reachable→claim→release loop, "what this is NOT" scoping, related-tasks index
- [x] CLAUDE.md Quick Reference row added — single row after "Register component" cross-referencing MCP parity + producer wiring + companion claim primitive

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
cargo check -p termlink
cargo check -p termlink-hub
cargo check -p termlink-bus
cargo check -p termlink-mcp
test -f docs/operations/agent-find-idle.md
out=$(grep -F "termlink agent find-idle" CLAUDE.md); echo "$out" | grep -q "DISPATCH"
out=$(./target/release/termlink agent find-idle --help 2>&1); echo "$out" | grep -q "find-idle"

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

### 2026-06-08 — slice 1: bus library lives in `termlink-bus`, not `termlink-hub`
- **What changed:** During the slice-1 cut the derivation function landed on `Bus` (`crates/termlink-bus/src/lib.rs`) rather than inside the hub router crate. The presence walk + dedup + LIVE filter is a pure read across topic envelopes + the `meta` table — both already on `Bus`. Putting it there made the function unit-testable against an in-memory bus, and let `handle_agent_find_idle` in `termlink-hub` reduce to a thin parse-and-format wrapper.
- **Plan impact:** Slice-1 LOC budget split ~80/30 between `termlink-bus` and `termlink-hub` instead of the originally implied "all in hub". Net total stayed under target.
- **Triggered:** No new task. Added `Meta::distinct_active_claimers(now_ms)` to `termlink-bus/src/meta.rs` to keep the SQL local to the data layer.

### 2026-06-08 — slice 4: `metadata.capabilities` stays absent (not `""`) when empty
- **What changed:** Producer emits `--metadata capabilities=...` only when non-empty (in both `listener-heartbeat.sh` and the `/be-reachable` passthrough). An empty string would be the wrong signal: the hub reads "absent" as "empty set, never matches a non-empty filter" — exactly what backward-compat agents want.
- **Plan impact:** None. Backward-compat with pre-T-2045 emitters is structural, not folkloric: an old worker keeps showing up unfiltered (and never matches `--capability`), which is the intended graceful-degrade.
- **Triggered:** Added explicit `find_idle_capabilities_subset_match` unit test that pins the AND-subset semantics + the empty-set non-match.

### 2026-06-08 — slice 2: hub restart needed for the live smoke (not a regression, expected)
- **What changed:** Local `target/release/termlink` had the router arm, but the running hub PID was launched from `/root/.cargo/bin/termlink` at 00:59Z — pre-find-idle. The smoke needed `hub stop` + `hub start` with the new binary. Persistent `/var/lib/termlink` runtime_dir made this transparent (no client re-pin needed).
- **Plan impact:** None for T-2045 itself, but reinforces the deploy pattern for substrate primitives: every new RPC ships in a binary that must replace the running hub before validation. Captured in the live-smoke evidence so future substrate work knows the same step is required.
- **Triggered:** No new task — this is canonical hub-upgrade behavior. See the live-smoke AC for the literal sequence.

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

### 2026-06-08T10:48:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2045-substrate-primitive-2-hub-owned-idlebusy.md
- **Context:** Initial task creation

### 2026-06-08T12:28:14Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-06-08T13:19:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
