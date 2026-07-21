---
id: T-2445
name: "Round-8 review residuals — WS push-transport + client-delivery MED/LOW findings"
description: >
  Backlog capture of round-8 adversarial-review findings not fixed in T-2442/2443/2444; horizon later for round-9 decomposition (one bug = one task on pickup).

status: captured
workflow_type: build
owner: agent
horizon: later
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-21T21:47:07Z
last_update: 2026-07-21T21:47:20Z
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

# T-2445: Round-8 review residuals — WS push-transport + client-delivery MED/LOW findings

## Context

Backlog capture of the round-8 adversarial review (arc-004 WS push-transport +
client-side durable delivery). The HIGH + top-MED findings were fixed this round
(T-2442 conn-cap DoS, T-2443 await-ack durability, T-2444 late-ack false
negative). The findings below are the un-actioned residuals. **This is a
capture, not a build** — `horizon: later`, not started. On pickup, decompose:
one bug = one task (Task Sizing Rules).

### WS push-transport residuals (`crates/termlink-hub/src/{server,aggregator}.rs`, `crates/termlink-session/src/ws_consumer.rs`, `crates/termlink-cli/src/commands/channel.rs`)

- **[MED] WS#4 — client push consumer hangs forever on a silently-dead hub; never degrades to poll.** `run_ws_session` (`ws_consumer.rs:186-203`) streams via `source.next().await` with no read timeout and sends no pings; on a half-open hub link the consumer blocks indefinitely, the CLI reconnect loop (`channel.rs:643`) never fires, and push silently wedges with no fallback to poll. Client-side mirror of the hub-side fix already shipped in T-2442. **Strongest remaining candidate.**
- **[MED] WS#2 — slow WS consumer loses events silently.** Bounded broadcast (aggregator cap 1024) returns `Lagged` handled by only a server-side `tracing::warn` (`server.rs` push arm); dropped events are never signalled to the client (no gap marker). For a raw `ws_consumer` user with no poll backstop this is a permanent silent hole.
- **[MED] WS#3 — every WS reconnect re-renders all live-delivered events (duplicates).** `run_ws_push` (`channel.rs:462-512`) renders pushed events but never advances `*cursor`; on drop, `ws_poll_catchup` re-fetches from the unadvanced cursor and re-renders everything already shown live. `ws_poll_catchup`'s own doc claims it "can't miss OR double-render" — the double-render half is false for live-phase events (doc is wrong).
- **[LOW] WS#5 — aggregator re-subscribe replays old events.** `add_session` resets `cursor = 0` on every (re)add (`aggregator.rs:80`); a flapping session re-fetches from seq 0 and re-sends old events to all connected WS clients.
- **[LOW/MED] WS#6 — auth scope cached for connection lifetime; no per-message/TTL re-check.** Once `hub.auth` sets `granted_scope` it is never re-validated against token expiry/revocation; an expired token keeps receiving pushes until disconnect. Compounds with the (now-fixed) idle-leak. Overlaps G-064 (no per-user authz) / T-2422 authz inception — likely folds into that operator-owned work.

### Client durable-delivery residuals (`crates/termlink-session/src/{bus_client,offline_queue}.rs`, `crates/termlink-cli/src/cli.rs`)

- **[MED] client#1 — duplicate append after crash + dedupe-TTL expiry.** If the process dies between hub-append and queue-row `pop`, the row replays on restart reusing the persisted `client_msg_id` (dedupe absorbs it) — but only within the hub's 5-min dedupe TTL (`dedupe.rs` `DEFAULT_DEDUPE_TTL_MS = 300_000`). Offline > 5 min between accept and replay → the dedupe entry aged out → double append. Narrow window; candidate mitigations: longer TTL for queued rows, or a persisted "delivered-pending-pop" marker.
- **[LOW/MED] client#4 — exhausted-then-acked awaiting-ack rows never reconciled.** On exhaustion the tracker row is retained (correct) but if the recipient acks LATER nothing re-polls receipts to `confirm()`+delete it; `channel awaiting-ack` is read-only. Rows accumulate as false-positive "outstanding" obligations forever. Wants a resume/sweep verb (`channel awaiting-ack --reconcile`).
- **[LOW] client#6 — legacy pre-T-2049 queued rows replay with no dedupe.** Rows persisted by an older binary have `client_msg_id: None`; a crash-replay of such a row (client#1 path) has no dedupe key → guaranteed duplicate. Only affects pre-T-2049 rows; self-heals as the field upgrades.
- **[LOW] client#5 — persistent `bump_attempts` write failure stalls poison escalation.** Already logged (T-2439b warns); a row whose counter never advances never reaches `POISON_THRESHOLD`. Requires a persistently-failing SQLite write (which would also break `dead_letter`). Lowest priority.

## Acceptance Criteria

### Agent
- [ ] (Tracking task — no direct ACs.) On pickup, decompose the residuals above into one-bug-one-task build tasks and close this tracker. Suggested priority order: WS#4, client#1, WS#3, client#4, then the LOWs.

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

### 2026-07-21T21:47:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2445-round-8-review-residuals--ws-push-transp.md
- **Context:** Initial task creation

### 2026-07-21T21:47:20Z — status-update [task-update-agent]
- **Change:** horizon: now → later
