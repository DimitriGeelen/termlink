---
id: T-2355
name: "Hub record-walk starvation: channel.subscribe/receipts blocking-I/O wedges under load (T-2258 class, ring20 G-157 seat)"
description: >
  The hub-side seat of the read-hang class T-2354 bounded client-side: handle_channel_subscribe_with / channel.receipts run blocking File::seek/read_exact per record on spawn_blocking (crates/termlink-hub/src/channel.rs:947-960 T-2258 note) — under concurrent large-topic walks the blocking pool starves and the walk never completes; observed live on .122 (channel info agent-chat-arc walk wedged >2m while list/subscribe-single-page stayed fast). Client now errors bounded (T-2354) but the walk still fails. Fix directions to evaluate: chunked/yielding walk (bounded records per spawn_blocking hop), per-request walk deadline server-side with partial-result or explicit error, or async file I/O. Also the true seat of ring20's G-157 (cross-host read deadlock) — reply with findings posted at DM offset 110, cid-g157-rootcause.

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
created: 2026-07-04T13:45:10Z
last_update: 2026-07-04T14:40:45Z
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

# T-2355: Hub record-walk starvation: channel.subscribe/receipts blocking-I/O wedges under load (T-2258 class, ring20 G-157 seat)

## Context

Hub-side seat of the read-hang class T-2354 bounded client-side. `handle_channel_subscribe_with` (channel.rs ~961) and `handle_channel_receipts_with` (~1102) walk topic records on `spawn_blocking` with NO deadline: a slow/huge walk (large topic, slow storage, wedged file I/O) holds its blocking-pool thread indefinitely; under K concurrent walks the pool saturates and reads wedge fleet-wide while O(1) posts stay fast — the exact field symptom on .122 (walk >2m16s pre-T-2354; still wedged, re-proven 2026-07-04 during T-2356 verification at bounded 5.26s client error). Fix: server-side per-walk deadline (`TERMLINK_WALK_DEADLINE_MS`, default 20000, clamp 100..=600000 — deliberately UNDER the T-2354 client read-timeout default of 30s so the client receives the server's structured error, not an opaque client-side timeout). On expiry the walk stops and the RPC returns a LOUD structured error (`WALK_DEADLINE_EXCEEDED`, -32020) with `data: {deadline_ms, records_scanned, next_cursor}` so callers can resume from where the walk stopped. Explicit-error over partial-result chosen because legacy pagers treat a short page as end-of-topic — partial results would silently under-count (violates Reliability directive); see Decisions.

## Acceptance Criteria

### Agent
- [x] `error_code::WALK_DEADLINE_EXCEEDED = -32020` registered in termlink-protocol control.rs with doc comment + stability test assert (`walk_deadline_exceeded_const_is_stable_wire_value`)
- [x] `channel.subscribe` walk stops at the deadline and returns -32020 with `data.next_cursor` = first unwalked offset (resumable), `data.records_scanned`, `data.deadline_ms` — deadline checked at top-of-loop BEFORE processing, so `last_offset` covers only fully-processed records and the resume cursor never skips one
- [x] `channel.receipts` walk stops at the deadline and returns -32020 (no partial receipt map — partial aggregate would silently under-report `up_to` marks; data carries deadline_ms + records_scanned only)
- [x] Deadline read from `TERMLINK_WALK_DEADLINE_MS` (default 20000ms, clamped 100..=600000); walk logic factored into `walk_subscribe_records` / `walk_receipt_records` taking `deadline: Duration`
- [x] Unit tests (6 new): zero-deadline hits before any record (both walks); generous deadline completes full topic identically (regression seam); mid-walk deadline preserves resume cursor (deterministic slow-iter); receipts latest-per-sender aggregation unchanged; env default/clamp/garbage handling
- [x] `cargo test -p termlink-hub` (403 passed, 0 failed) and `cargo test -p termlink-protocol` (102 passed, 0 failed) pass; `cargo check --workspace` clean

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

cargo test -q -p termlink-hub > /tmp/.t2355-hub.out 2>&1
cargo test -q -p termlink-protocol > /tmp/.t2355-proto.out 2>&1
grep -q "WALK_DEADLINE_EXCEEDED" /opt/termlink/crates/termlink-protocol/src/control.rs
grep -q "walk_deadline_from_env" /opt/termlink/crates/termlink-hub/src/channel.rs

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

**Symptom:** `channel info` / `channel unread` (both page `channel.subscribe`) and `channel receipts` against a loaded hub hang indefinitely while `channel post` stays fast. Field case: .122's agent-chat-arc walk wedged >2m16s (pre-T-2354 client hung forever; post-T-2354 client errors bounded but the hub-side walk still never completes — re-proven 2026-07-04 at 5.26s bounded client error).

**Root cause:** the record walks in `handle_channel_subscribe_with` / `handle_channel_receipts_with` run unbounded on `spawn_blocking`. T-2258 fixed WHERE the walk runs (dedicated blocking pool, reactor stays alive) but not HOW LONG it may run — a slow/huge/wedged walk holds its blocking-pool thread forever; under K concurrent walks the pool (cap 512) saturates and all reads wedge fleet-wide.

**Why structurally allowed:** the T-2258 fix was verified against reactor-starvation (posts stay fast during one large walk) — the success criterion never included "a single walk terminates in bounded time". No RPC-level deadline convention existed on the hub side; the client-side deadline (T-2354) came first and masked the gap by converting infinite hangs into opaque client timeouts.

**Prevention:** server-side walk deadline (`TERMLINK_WALK_DEADLINE_MS`, default 20s < the client's 30s read timeout) with LOUD structured refusal `WALK_DEADLINE_EXCEEDED` (-32020) carrying a resumable `next_cursor`; wire-value stability test locks the code; zero/mid/generous-deadline unit tests lock the semantics. Distinct from the fix itself: the layered-deadline rule (server deadline strictly under client timeout so the structured error, not the opaque one, reaches the operator) is now documented on the error-code doc comment.

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

### 2026-07-04 — deadline mechanism (filing option 2 over 1/3)
- **Chose:** per-walk elapsed-time deadline checked at top of each iteration, explicit structured error on expiry
- **Why:** bounds blocking-thread occupancy (the actual starvation resource) with ~ns per-record overhead vs µs per-record file I/O; smallest diff; testable as a pure function
- **Rejected:** chunked/yielding walk (option 1 — blocking-pool threads don't need scheduler yielding, added complexity buys nothing once occupancy is bounded); async file I/O (option 3 — full rewrite of the bus reader layer for the same operator-visible outcome)

### 2026-07-04 — explicit error over partial-result page
- **Chose:** `WALK_DEADLINE_EXCEEDED` (-32020) error with `data.next_cursor`, never a truncated success page
- **Why:** legacy pagers treat a short page as end-of-topic — a deadline-truncated success would silently under-count (`channel info`/`unread` would report wrong totals with no signal). Loud refusal with a resume cursor keeps Reliability ("no silent failures") and still enables incremental progress for callers that opt in
- **Rejected:** `truncated: true` flag on a success page (old clients ignore unknown fields → silent wrong data, the worst failure mode)

### 2026-07-04 — 20s default, clamp 100..=600000
- **Chose:** default 20_000ms, strictly under the T-2354 client read-timeout default (30s)
- **Why:** layered deadlines — the server must refuse BEFORE the client gives up, so the operator sees the server's structured, actionable error instead of an opaque client timeout
- **Rejected:** default ≥ client timeout (server error would never reach the client on default configs)

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-07-04T13:45:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2355-hub-record-walk-starvation-channelsubscr.md
- **Context:** Initial task creation

### 2026-07-04T14:40:45Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-07-04T15:10:00Z — fix-implemented [agent]
- **Action:** Server-side walk deadline: `WALK_DEADLINE_EXCEEDED` (-32020) in termlink-protocol; `walk_deadline_from_env` + factored `walk_subscribe_records` / `walk_receipt_records` in termlink-hub channel.rs; both spawn_blocking walks now bounded; 6 new unit tests + wire-stability test
- **Evidence:** hub suite 403 passed / protocol suite 102 passed / workspace check clean
- **Activation note:** hub-side change — takes effect on hub restart with the new binary. The .107 local hub and ring20's .122 hub (the field-wedged one) both run pre-fix binaries until their operators restart; .122 is ring20's domain (their G-157) — notify via the existing dm cid-g157-rootcause thread
