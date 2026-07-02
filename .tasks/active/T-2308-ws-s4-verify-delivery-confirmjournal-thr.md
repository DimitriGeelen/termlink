---
id: T-2308
name: "WS-S4 verify delivery-confirm+journal through WS push path unchanged"
description: >
  WS-S4 verify delivery-confirm+journal through WS push path unchanged

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-hub/src/server.rs, crates/termlink-hub/src/channel.rs]
related_tasks: [T-2305, T-2306, T-2307, T-2303]
arc_id: push-transport            # arc-004 — WS live-transport build arc (GO output of T-2303); S4 (final slice)
created: 2026-07-02T16:50:30Z
last_update: 2026-07-02T16:50:30Z
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

# T-2308: WS-S4 verify delivery-confirm+journal through WS push path unchanged

## Context

Final build slice (S4) of arc-004 `push-transport`. S1–S3 built the live WS transport:
a hub→client WebSocket that pushes `hub.event` frames for topics the client subscribes to
(`hub.ws_subscribe`), opt-in, degrading to poll. S4 closes the arc by proving the arc's
central **durability invariant** holds over the push path: *the WS carries only a faster
pointer to durable state; delivery-confirmation (receipts) and the journal remain the
existing, unchanged path.*

**The seam (established by reading the code, not assumed):** the durable wakeup event that
carries the offset pointer is `inbox.queued` — emitted on `channel.post` to an `inbox:*`
topic (`channel.rs:753`), payload `{addressee_session_id, channel, message_offset,
enqueued_at}`. That event is injected into the aggregator, so an authenticated WS client
subscribed to the `inbox.queued` topic receives the doorbell **instantly** with the durable
`message_offset`. The client then reads the durable `channel` topic at that offset and acks
through the **existing** `channel.ack` / receipts path — no receipt or journal code runs on
the WS path itself. This is the arc's "durability layer unchanged" framing made concrete.

**S4 scope (this task):** a verification slice (mirroring how S3's degrade-to-poll was a
*verified contract*, not new code) — an end-to-end hub test proving that a `channel.post`
whose delivery emits `inbox.queued` produces a WS push whose `params.payload.message_offset`
equals the durable offset returned by the post, AND that reading the durable topic at that
offset yields the posted message (i.e. the pushed pointer maps to an ackable durable
position). If a gap is found (WS push drops the offset field, or the confirm path is
unreachable from the pushed frame), close the minimal gap. **Out of scope:** a WS-native ack
frame (receipts stay on the existing RPC path by design — that is the whole point);
the live CLI consumer (S3b).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] A `channel.post` to an `inbox:<id>` topic returns a durable `offset`, and the same post causes an `inbox.queued` `AggregatedEvent` to be injected into the aggregator carrying `payload.message_offset == offset` and `payload.channel == "inbox:<id>"`. *Proven in `ws_delivery_offset_through_push`: the real forward-path `handle_channel_post_with` returns `offset`; the WS-observed `inbox.queued` frame carries `payload.channel == inbox:<target>` and `payload.message_offset == offset`.*
- [x] An authenticated WS client subscribed (via `hub.ws_subscribe`) to the `inbox.queued` topic receives a `hub.event` frame for that post whose `params.payload.message_offset` equals the durable offset — proving the confirm/read pointer travels over the WS push path intact (not truncated or re-sequenced). *`server::tests::ws_delivery_offset_through_push` — `assert_eq!(pushed_offset, durable_offset)`.*
- [x] The durable topic read at the pushed `message_offset` yields the originally posted message body — i.e. the pushed pointer resolves to the exact durable position the recipient would ack, proving receipts/journal remain reachable and authoritative through the existing (unchanged) path. *Same test: `Bus::envelope_at(inbox_topic, pushed_offset).payload == body`.*
- [x] No change to receipt/journal/ack code paths: the WS path adds no new receipt-write or journal-write. The diff touches only the push-observation test surface (`server.rs` test module) — no production receipt/journal code changed. All existing hub tests (incl. receipts/ack) still pass: `cargo test -p termlink-hub` → **377 passed; 0 failed**.
- [x] `cargo build --release -p termlink-hub` (green, 18.14s) and `cargo test -p termlink-hub ws_` (5 passed) both pass; the arc's degrade-to-poll invariant is preserved (the durable `inbox.queued` + `channel.subscribe`/`envelope_at`/`ack` path a non-WS client uses is untouched — the same durable read the test uses for AC3).

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
cargo build --release -p termlink-hub 2>&1 | tail -3
cargo test -p termlink-hub ws_ 2>&1 | tail -15

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

### 2026-07-02 — S4 seam was in the code, not a new mechanism
- **What changed:** Filing framed S4 as "wire delivery-confirm/journal through WS." Reading the code showed the wiring already exists: `channel.post` to an `inbox:*` topic (T-1637, `channel.rs:753`) injects an `inbox.queued` `AggregatedEvent` carrying `{channel, message_offset}` into the process-global aggregator — the exact stream S2/S3's WS push loop drains. So the "durable delivery pointer over WS" is realized by *subscribing a WS client to the `inbox.queued` topic*, not by adding a receipt path to the socket.
- **Plan impact:** S4 collapsed from "add code" to "prove the contract" (same shape as S3's degrade-to-poll being a verified contract, not new code). Deliverable became a single honest end-to-end test through real production code (`handle_channel_post_with` → aggregator inject → WS push → `Bus::envelope_at`), not a new WS-native ack frame. A WS-native ack frame is explicitly the *wrong* design here — receipts stay on the existing RPC path by the arc's "durability layer unchanged" invariant.
- **Triggered:** No new sub-task. Confirmed the two remaining arc follow-ons stay carved out: **S3b** (live CLI consumer with WS-connect→reconnect→poll-fallback loop) and the observation that a plain `dm:*` topic post does *not* itself inject an aggregator event — only `inbox:*` does — so the live-agent wake path is the doorbell-on-`inbox.queued`, which S4 verifies. arc-004's four build slices (S1–S4) are now complete; the arc is ready to close pending the S3b consumer decision + demo evidence.

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

### 2026-07-02T16:50:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2308-ws-s4-verify-delivery-confirmjournal-thr.md
- **Context:** Initial task creation
