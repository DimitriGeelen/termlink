---
id: T-2354
name: "channel info/unread hang indefinitely over --hub <tcp> (list works)"
description: >
  Field-discovered during T-2353 verification: 'termlink channel info <topic> --json --hub 192.168.10.122:9100' and 'channel unread ... --hub <tcp>' hang past 12s (killed by timeout) while 'channel list --hub <tcp>' returns fast on the same hub — a remote read-verb wedge class, plausibly the same as ring20's G-157 ('cross-host reads deadlock'). Suspect: these verbs issue a second/streaming RPC after connect that never completes over TCP. Repro: timeout 8 termlink channel info agent-chat-arc --json --hub 192.168.10.122:9100 => exit 124. agent-send.sh now bounds its scan calls (TERMLINK_SCAN_TIMEOUT, T-2353) so sends degrade loudly instead of hanging; this task is the root-cause fix in the CLI/hub.

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
created: 2026-07-04T13:08:58Z
last_update: 2026-07-04T13:19:55Z
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

# T-2354: channel info/unread hang indefinitely over --hub <tcp> (list works)

## Context

Verb scope narrowed by live probing against 192.168.10.122:9100 (2026-07-04): `channel list --hub` WORKS (fast), `channel subscribe --hub` WORKS (fast), `channel info --hub` HANGS (>12s, killed), `channel unread --hub` HANGS (>8s, killed) — reproducible on multiple topics (agent-chat-arc, agent-presence, dm:*). So the wedge is specific to the info/unread RPC pair, not TCP/TLS/auth (list+subscribe share those and are fine). Suspect a second RPC or receipt/cursor read these two verbs issue after connect that never completes over the TCP path (works over local unix). Plausibly the same class ring20 reported as their G-157 ("cross-host reads deadlock", DM offsets 61-62). Repro: `timeout 8 termlink channel info agent-chat-arc --json --hub 192.168.10.122:9100` → exit 124. Mitigation already shipped: agent-send.sh bounds all scan calls (`TERMLINK_SCAN_TIMEOUT`, T-2353); this task is the root-cause fix.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `termlink_session::client::Client` gains `call_with_timeout(method, id, params, timeout)` — wraps `call` in `tokio::time::timeout`, mapping expiry to `ClientError::Io(TimedOut)` with a message naming the method and the wedged-hub cause (mirror of `connect_addr_with_timeout`, T-1678)
- [x] `rpc_call_authed` (crates/termlink-cli/src/commands/channel.rs) bounds BOTH the `hub.auth` call and the main RPC on the TCP path with a read deadline: env `TERMLINK_RPC_READ_TIMEOUT_SECS` (default 30, clamped 1..=600)
- [x] A hung remote read now errors bounded instead of hanging forever: `channel info <topic> --hub <tcp>` against a wedged hub returns non-zero within the deadline with the timeout message (no more indefinite hang) — LIVE-PROVEN against .122: `TERMLINK_RPC_READ_TIMEOUT_SECS=5` errored at 5.1s ("RPC 'channel.subscribe' response timeout after 5s...") where the pre-fix binary hung >2m16s
- [x] Working verbs unregressed: `channel list --hub <tcp>` (exit 0 vs .122) and local-unix `channel info` (exit 0) behave exactly as before (unix path untouched)
- [x] Unit test covers `call_with_timeout` expiry (server that accepts but never replies → `TimedOut` within bound) and pass-through success — both pass
- [x] Full CLI + session test suites green (session 380+1+16+20+1; CLI `termlink` 958+3+174 — the CLI crate's package name is `termlink`)

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

# new client tests: silent-server bounded error + success pass-through
out=$(cargo test -p termlink-session --lib client::tests::call_with_timeout 2>&1); echo "$out" | grep -q "2 passed"
# suites green
out=$(cargo test -p termlink-session 2>&1); echo "$out" | grep -vq "FAILED"
out=$(cargo test -p termlink 2>&1); echo "$out" | grep -vq "FAILED"

## RCA

**Symptom:** `channel info <topic> --hub <tcp>` and `channel unread ... --hub <tcp>` hang indefinitely (>12s, only killable) against a remote hub, while `channel list --hub` and `channel subscribe --hub` return fast on the same hub/connection parameters.

**Root cause:** Two stacked gaps. (1) `info`/`unread` are the only common verbs issuing a SECOND RPC after their cheap first call — a full record-walk (`channel.subscribe`/`channel.receipts` paging loop, channel.rs walk-loop) that exercises the hub's blocking-I/O `spawn_blocking` record reads, which can stall under concurrent load (the documented T-2258 starvation class in crates/termlink-hub/src/channel.rs:947-960). (2) `Client::call` (crates/termlink-session/src/client.rs) had NO read deadline — `rpc_call_authed` bounded only the connect (10s, T-1678), so a stalled walk response hung the CLI forever instead of erroring.

**Why structurally allowed:** the T-1678 timeout work covered connect-unreachable but nobody added the symmetric read bound; verbs that complete in one RPC (list, single-page subscribe) never expose the gap, so it survived until a verb needing the multi-RPC walk was run against a busy remote hub.

**Prevention:** `call_with_timeout` on the client (mirror of `connect_addr_with_timeout`) + `rpc_call_authed` bounding BOTH reads via `TERMLINK_RPC_READ_TIMEOUT_SECS` (default 30s, clamp 1..=600) — every TCP channel-verb RPC is now deadline-bounded by construction, converting any future hub-side wedge into a bounded, actionable error naming the method. The hub-side walk starvation itself remains tracked by the T-2258 note (separate concern).

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

### 2026-07-04 — Client-side read deadline (not hub-side walk fix) as this task's scope
- **Chose:** bound every TCP RPC read in `rpc_call_authed` via a new `Client::call_with_timeout` (default 30s, `TERMLINK_RPC_READ_TIMEOUT_SECS` override clamped 1..=600); map expiry to `ClientError::Io(TimedOut)` so no caller error-handling changes.
- **Why:** the CLI hanging FOREVER is the operator-facing defect regardless of why the hub stalls; the client must never trust a remote to reply. The hub-side record-walk starvation is a pre-documented separate concern (T-2258 note in termlink-hub channel.rs:947-960) with its own remediation path. 30s default is generous for a single ≤1000-record page while still converting a wedge into an actionable error.
- **Rejected:** new ClientError::Timeout variant (forces match-arm churn in every caller for zero benefit — Io(TimedOut) is idiomatic and matches connect_addr_with_timeout, T-1678); fixing the hub walk here (different subsystem, different failure domain, deserves its own task/testing); per-verb timeouts (the gap is structural in the shared helper — fixing it once covers every current and future multi-RPC verb).

### 2026-07-04 — Stale-binary verification trap (PL-209 replay)
- **Chose:** re-run the live probe after confirming `target/debug/termlink` mtime postdates the edit.
- **Why:** the first live probe ran while `cargo test -p termlink` was still compiling — it exercised the PRE-fix binary and "disproved" a working fix (hung 2m16s). Same class as PL-209 (probing an old in-memory hub image). Check artifact freshness before interpreting a negative probe result.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-07-04T13:08:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2354-channel-infounread-hang-indefinitely-ove.md
- **Context:** Initial task creation

### 2026-07-04T13:19:55Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
