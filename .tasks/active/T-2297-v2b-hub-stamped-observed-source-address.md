---
id: T-2297
name: "V2b: hub-stamped observed source address"
description: >
  Arc-003 V2 follow-up (sliced from T-2293). Hub stamps the OBSERVED TCP source address it saw onto agent-presence heartbeats / registrations, so the discovery registry can prefer a hub-attested host:port over the agent's self-report (defeats stale/spoofed self-reported hostnames). peer_addr is already available at server.rs:640/670 and threaded into process_request; the work is to thread it through route_request -> handle_channel_post and inject observed_addr into the stored envelope, then have fleet_presence/agent-resolve prefer it. Consumed by V6 (T-2296) direct transport, which needs the agent's real host. Self-report addr (the AC2 fallback) already ships in T-2293.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [arc:reliable-comms]
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/agent.rs, crates/termlink-cli/src/main.rs, crates/termlink-hub/src/channel.rs, crates/termlink-hub/src/router.rs, crates/termlink-hub/src/server.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-27T17:52:01Z
last_update: 2026-07-02T08:15:05Z
date_finished: 2026-07-02T08:15:05Z
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

# T-2297: V2b: hub-stamped observed source address

## Context

Arc-003 reliable-comms **V2b** (sliced from T-2293) — the LAST open arc task. The hub
STAMPS the observed TCP source address it saw (`peer_addr`) onto channel.post envelopes
it accepts over TCP, as a hub-attested `metadata.observed_addr`. The discovery read path
(`fleet_presence::PresenceMatch`, `agent resolve`) then PREFERS this attested address
over the agent's self-reported `metadata.addr` (T-2293), defeating stale/spoofed
self-reports. Consumed by V6 direct transport (needs the peer's real host); V6 already
ships on the self-report addr, so this is a **hardening upgrade, not a blocker**.

**Code path (mapped, T-2297 Explore):** `peer_addr` is captured at `server.rs:640/670`,
threaded into `handle_connection` (`:756`) but DROPPED at the `router::route` call sites
(`:846`/`:891`). Work: thread `peer_addr: Option<&str>` through `router::route`
(`router.rs:59`) → `handle_channel_post` / `handle_channel_post_with`
(`channel.rs:503/511`), and stamp it into the envelope `metadata` (built `channel.rs:653`).
**Metadata is NOT in the signed canonical bytes** (`channel.rs:579`) and `Envelope.metadata`
is an existing `BTreeMap<String,String>` (`envelope.rs:20`) — so server-side injection needs
NO schema/serde change and cannot break signature verification (**PL-122 safe**). Read side:
add `observed_addr` to `PresenceMatch` (`fleet_presence.rs:57/144`) and prefer it at
`agent.rs:867`.

**Attestation invariant (load-bearing):** `observed_addr` MUST always be hub-attested or
absent — never client-forgeable. The handler OVERWRITES any client-supplied `observed_addr`
when `peer_addr` is `Some` (TCP), and STRIPS it when `None` (Unix socket / local, not
attestable).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] A pure, unit-tested helper enforces the attestation invariant on envelope metadata: `observed_addr` is set to the hub-observed TCP `peer_addr` when present; any client-supplied `observed_addr` is OVERWRITTEN (TCP) or STRIPPED (Unix/`None`). Unit test covers all three cases (attested-overwrite, forged-value-overwritten, strip-when-none) — **`apply_observed_addr` (channel.rs); `cargo test -p termlink-hub observed_addr` = 3/3 pass** (commit fcf18a44).
- [x] `peer_addr` is threaded from the accept site through `router::route` → `handle_channel_post(_with)`, the helper is applied before the envelope is stored, and the workspace compiles (`cargo build`) — threading DONE; `handle_channel_post_with` marked `#[cfg(test)]` (production Unix path goes through `_with_peer(None)`), dead-code warning cleared; **full-workspace `cargo build` = exit 0**.
- [x] Read side prefers the attested address: `PresenceMatch` carries `observed_addr` (parsed from `metadata.observed_addr`), and `agent resolve` resolves host as `observed_addr ?? self-reported addr ?? profile address` (fleet_presence.rs + agent.rs `hub = observed_addr.or(addr).or(profile)`, JSON+human output carry `observed_addr`); **`cargo check -p termlink-session -p termlink` = exit 0** (CLI package is `termlink`).
- [x] Backward-compatible: envelopes with no `observed_addr` (Unix posts, old clients) behave exactly as before — metadata is unsigned; strip-on-`None` proven by unit test (`observed_addr_stripped_when_not_attestable`); confirmed on final workspace build.

### Human
- [ ] [RUBBER-STAMP] Live end-to-end after installing the rebuilt hub binary
  **Steps:**
  1. Rebuild + install: `cargo build --release -p termlink && install -m 755 target/release/termlink ~/.cargo/bin/termlink`
  2. Restart the local hub so it runs the new binary (per CLAUDE.md §volatile-runtime_dir, confirm `TERMLINK_RUNTIME_DIR` is off `/tmp` first so the restart does not rotate the secret).
  3. From another host (or a TCP loopback post), run: `termlink channel post agent-presence --hub <this-host-ip>:9100 --payload hb --json` then `termlink channel subscribe agent-presence --hub <this-host-ip>:9100 --limit 1 --json | jq '.metadata.observed_addr'`
  4. Run `termlink agent resolve <agent_id> --json | jq '{observed_addr, self_reported_addr, hub}'`
  **Expected:** step 3 shows a hub-stamped `observed_addr` (the caller's real IP:port, NOT what the client put in metadata); step 4 shows `hub` resolved from `observed_addr` when present.
  **If not:** confirm the running hub is the rebuilt binary (`/proc/$(pgrep -f 'termlink hub')/exe` not `(deleted)`); a Unix-socket post correctly has NO `observed_addr` (only TCP posts are attested).

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

# The pure attestation helper's unit test (proves overwrite/strip invariant) + read-side compile.
cargo test -p termlink-hub observed_addr
cargo check -p termlink-session -p termlink

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

### 2026-07-02 (cont.) — closed; the four Agent ACs met, arc-003 complete
- **What changed:** Fresh-budget session finished the three bankable steps left by the
  budget gate: (1) `handle_channel_post_with` marked `#[cfg(test)]` — grep confirmed ALL
  ~40 callers live in `#[cfg(test)]` modules and production posts route through
  `handle_channel_post_with_peer`, so the dead-code warning fired only in the non-test lib
  compilation; the attribute clears it cleanly. (2) `cargo check -p termlink-session -p termlink`
  = exit 0 (read side compiles under the corrected package name). (3) full `cargo build`
  = exit 0; `cargo test -p termlink-hub observed_addr` = 3/3, no warnings.
- **Plan impact:** none — the Explore map held end-to-end. This closes arc-003 (V2b was the
  last open arc task: V1/V2/V2b/V3a/V3b + V6 apex S1–S5 all now work-completed).
- **Triggered:** Human [RUBBER-STAMP] live end-to-end (needs hub rebuild+restart) remains
  the one operator-gated item; it is a hardening-confirmation, not a blocker (V6 already
  ships on the self-report addr).

### 2026-07-02 — implemented; stopped at budget gate one step from close
- **What changed:** The Explore map was exact — the change is small and PL-122-safe
  because envelope `metadata` is NOT in the signed canonical bytes (`channel.rs:579`),
  so a server-stamped `observed_addr` needs no schema/serde change and cannot break
  signature verification. Threaded `peer_addr: Option<&str>` through `router::route`
  (+2 server.rs call sites, +4 router test call sites → `None`) into a new
  `handle_channel_post_with_peer`; kept `handle_channel_post_with` as a wrapper so the
  ~40 existing test callers stay green (no test churn). Stamping is a pure helper
  `apply_observed_addr` (overwrite-on-TCP / strip-on-None) with 3 unit tests (all pass).
  Read side: `PresenceMatch.observed_addr` + `agent resolve` prefers
  `observed_addr ?? self_reported ?? profile`.
- **Plan impact:** `handle_channel_post_with` became test-only (production Unix path goes
  through `_with_peer(None)` directly) → a dead-code warning; the `#[cfg(test)]` cleanup
  edit was BLOCKED when the budget gate hit critical (~96%). Also caught the recurring
  package-name gotcha: the CLI crate is `termlink`, not `termlink-cli` — Verification
  corrected. Code compiles (hub test-build exit 0, 3/3 helper tests); the read-side
  `cargo check -p termlink-session -p termlink` was not run with the corrected name.
- **Triggered:** NEXT SESSION (all bankable, ~10 min): (1) add `#[cfg(test)]` to
  `handle_channel_post_with`; (2) `cargo check -p termlink-session -p termlink` (read
  side) + full `cargo build`; (3) tick AC2–AC4 + close T-2297 → **arc-003 fully
  complete**; (4) the Human [RUBBER-STAMP] live proof needs a hub rebuild+restart.

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

## Recommendation

**Recommendation:** GO

**Rationale:** All 4 Agent ACs are met and machine-verified (P-011 green). The change
is PL-122-safe by construction — envelope `metadata` is NOT in the signed canonical
bytes (`channel.rs:579`), so server-stamping `observed_addr` needs no schema/serde
change and cannot break signature verification. The attestation invariant
(overwrite-on-TCP / strip-on-`None`, never client-forgeable) is enforced by the pure
`apply_observed_addr` helper and proven by 3 unit tests. Read-side prefers the attested
address with correct fallback (`observed_addr ?? self_reported ?? profile`), and old
clients / Unix posts are byte-for-byte unaffected. This closes arc-003 reliable-comms
(V2b was the last open arc task). The single Human [RUBBER-STAMP] AC is a live
end-to-end confirmation that requires a hub rebuild+restart — it is a hardening
confirmation, not a blocker (V6 direct transport already ships on the self-report addr).

**Evidence:**
- `apply_observed_addr` helper + `mod observed_addr_tests` — `cargo test -p termlink-hub observed_addr` = **3/3 pass** (overwrite-when-attested, forged-value-overwritten, strip-when-none).
- Threading: `peer_addr: Option<&str>` from `server.rs` accept site → `router::route` → `handle_channel_post_with_peer`; `handle_channel_post_with` now `#[cfg(test)]` (all ~40 callers are in test modules — grep-confirmed).
- Read side: `PresenceMatch.observed_addr` (fleet_presence.rs) + `agent resolve` host = `observed_addr.or(addr).or(profile)` (agent.rs), JSON + human output carry `observed_addr`.
- Compile: `cargo check -p termlink-session -p termlink` = **exit 0**; full `cargo build` = **exit 0**; no dead-code warnings.
- Commit `43457420` (source + task); prior WIP `fcf18a44`.

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

### 2026-06-27T17:52:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2297-v2b-hub-stamped-observed-source-address.md
- **Context:** Initial task creation

### 2026-07-02T07:40:34Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-07-02T08:15:05Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-07-06 — Human [RUBBER-STAMP] AC evidence gathered (agent-assisted, NOT ticked — operator verifies + closes)
- **Feature is LIVE on the running .107 hub:** commit fcf18a44 is an ancestor of HEAD; running hub binary (pid 3475796, /root/.cargo/bin/termlink, installed 2026-07-04) contains the `observed_addr` symbol. So the AC's rebuild+restart steps (1–2) are already satisfied — no restart needed.
- **Genuine remote-host TCP post verified** (AC step 3, using the .122 termlink session as the "another host"): `.122` posted to `agent-presence` on `192.168.10.107:9100`; the .107 hub stamped `observed_addr=192.168.10.122:45408` at offset 30806 (sender fp `9219671e…`, payload `t2297-from-122`). This is the attestation invariant proven end-to-end: the hub-observed remote peer_addr is stamped, not client-supplied.
- **Local/Unix post backward-compat confirmed** (AC step 4 / AC4): a local post carries NO `observed_addr` (correctly stripped for non-TCP/None), unchanged behavior.
- **Operator action to finalize:** verify the above, tick the `[RUBBER-STAMP]` Human AC box, then `cd /opt/termlink && .agentic-framework/bin/fw task update T-2297 --status work-completed`. (Agent may not check `### Human` ACs — G-068.)
