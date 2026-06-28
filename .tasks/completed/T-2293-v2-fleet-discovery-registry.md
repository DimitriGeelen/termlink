---
id: T-2293
name: "V2: fleet discovery registry"
description: >
  RC2 fix. Resolve agent_id -> {host:port (hub-stamped observed addr, self-report fallback), hub, topics-read, liveness}. remote_store.rs RemoteEntry{host,port,TTL=300s,last_heartbeat} supplies the schema — populate + add fleet-rollup reader over hubs.toml with short TTL. Symmetric (both directions: recipient resolves sender too). Tier-2 only; Tier-1 LAN-broadcast/mDNS DEFERRED (T-006 already rejected mDNS). ACs: registry resolves agent_id->host:port+topics-read; addr is hub-stamped (self-report fallback); rollup walks hubs.toml; reverse lookup works; G-155 false-green probe fixed to test the intended correspondent.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:reliable-comms]
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/agent.rs, crates/termlink-cli/src/main.rs]
related_tasks: [T-2291, T-2292, T-2297]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-27T17:06:08Z
last_update: 2026-06-27T17:55:07Z
date_finished: 2026-06-27T17:55:07Z
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

# T-2293: V2: fleet discovery registry

## Context

Arc-003 (reliable-comms) slice. RC2 from the T-2291 inception RCA: there is no registry mapping agent_id → hub, and hubs don't federate, so a sender can't resolve which hub a peer reads. `crates/termlink-hub/src/remote_store.rs` `RemoteEntry{host,port,TTL=300s,last_heartbeat}` already supplies the schema. Depends on T-2292 (per-agent identity is the registry key). Design trail: `docs/reports/T-2291-cross-agent-comms-inception.md`.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Registry resolves `agent_id` → `{host:port, hub, topics-read, liveness}` for a registered peer — `termlink agent resolve <agent_id> [--json]` (new) via `resolve_agent_registry_via_fleet`. Live proof: resolved `v2-selftest` → LIVE, hub `192.168.10.107:9100`, topics `dm:v2-selftest:*, agent-chat-arc`, fingerprint+host+role.
- [x] Observed address is self-reported by the agent (`metadata.addr`), with the resolver falling back to the hub it read the heartbeat from when absent — **self-report baseline shipped** (heartbeat now emits `addr=<hub>`; resolver prefers it, else stamps the found-on hub). The stronger **hub-stamped observed source addr** (hub attests the addr it saw, defeating self-report staleness/spoofing) is sliced to **T-2297 (V2b)**, which V6 (direct transport) consumes — see Decision below.
- [x] A fleet-rollup reader walks `hubs.toml` and merges per-hub registries with a short TTL — the resolver walks all hubs (dedup by address), bounded by an 8s per-hub timeout so a dead hub never stalls the walk; liveness TTL via the `agent-presence` 2x/5x interval bands (`fleet_presence`).
- [x] Reverse lookup works: a recipient can resolve the SENDER's address (symmetric, both directions) — `agent resolve` works for ANY agent_id including the caller's own; symmetric by construction (same code path).
- [x] The G-155 false-green probe is fixed: `agent resolve` returns "found" only when the intended correspondent actually has a heartbeat (LIVE/STALE/OFFLINE) on a hub — NOT when a configured hub's TLS merely handshakes (exit 4 on truly-absent). Unlike `fleet verify`/`fleet doctor`, a configured-but-empty hub yields not-found here.

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
cargo build -p termlink
cargo test -p termlink-session --lib fleet_presence
out=$(target/debug/termlink agent resolve __definitely_not_an_agent__ --json 2>&1); echo "$out" | grep -q '"ok": false'

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

### 2026-06-27 — the "hub" answer is resolver-stamped, not self-reported

- **What changed:** Realized the registry's `hub` field (which hub a peer reads,
  for routing a DM) doesn't need self-reporting at all — when the resolver FINDS
  agent X's heartbeat on hub H, then H *is* the hub X reads. The resolver stamps
  it. Self-reported `metadata.addr` is only an override/refinement (e.g. an
  agent on the default local hub that knows its external addr).
- **Plan impact:** Simplified AC1 — `host:port/hub` is mostly resolver-derived;
  the heartbeat only needed to ADD `addr` for the cases where the agent reads a
  hub by an address the resolver should prefer.
- **Triggered:** Decision below on the self-report-vs-hub-stamp split.

### 2026-06-27 — dead hub stalled the walk (Antifragility fix)

- **What changed:** Live testing surfaced that one unreachable hub in `hubs.toml`
  hung the entire `agent resolve` walk — `fetch_topic_msgs` has no internal
  bound, and the prior-art `resolve_contact_via_fleet` shares this latent risk.
- **Plan impact:** Added an 8s `tokio::time::timeout` per hub (matching the
  T-2062 fleet-governor convention) so a stalled hub is skipped, not fatal —
  "per-hub failures never abort the walk" now holds for hangs, not just errors.
- **Triggered:** Noted the same latent hang in `resolve_contact_via_fleet`
  (contact path) — a candidate follow-up hardening, not fixed here (out of scope).

### 2026-06-27 — AC2 hub-stamping sliced to T-2297

- **What changed:** AC2's "hub records the source addr it saw" requires threading
  `peer_addr` through `route_request` → `handle_channel_post` → envelope storage
  (multi-signature hub change). The self-report baseline (which AC2 names as the
  fallback) fully satisfies the routing need today.
- **Plan impact:** V2 ships the complete self-report registry (AC1/3/4/5 + the
  fallback half of AC2). The hub-attested-addr upgrade is **T-2297 (V2b)**.
- **Triggered:** Filed T-2297; V6 (T-2296 direct transport) is its primary
  consumer (it needs the agent's real host, hub-attested to defeat spoofing).

## Decisions

### 2026-06-27 — split observed-addr into self-report (now) + hub-stamp (T-2297)

- **Chose:** Ship the self-reported `metadata.addr` + resolver-stamped `hub` as
  V2's address answer; slice the hub-attested observed source addr to T-2297.
- **Why:** Self-report is the working baseline AC2 itself names as the fallback;
  it unblocks the resolver, V3 (notify needs to know which hub), and most of V6
  immediately. Hub-stamping is an authoritative *upgrade* whose primary consumer
  (V6 direct transport) is still `later` — sequencing it as its own small task
  keeps V2 a clean, shippable slice (Task Sizing: decompose when too big) instead
  of bundling a multi-signature hub change that risks a half-built V2.
- **Rejected:** (a) Blocking V2 on the full hub-stamp — would balloon one task
  into two deliverables. (b) Dropping AC2 silently — instead it's re-scoped to
  the delivered baseline with the upgrade tracked in T-2297.

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

### 2026-06-27T17:06:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2293-v2-fleet-discovery-registry.md
- **Context:** Initial task creation

### 2026-06-27T17:39:15Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-06-27T17:55:07Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
