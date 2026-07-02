---
id: T-2307
name: "V1 WS-S3 client WS subscribe with per-topic filter + degrade-to-poll fallback"
description: >
  V1 WS-S3 client WS subscribe with per-topic filter + degrade-to-poll fallback

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
arc_id: push-transport            # arc-004 — WS live-transport build arc (GO output of T-2303)
created: 2026-07-02T16:08:40Z
last_update: 2026-07-02T16:08:40Z
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

# T-2307: V1 WS-S3 client WS subscribe with per-topic filter + degrade-to-poll fallback

## Context

Third build slice (S3) of arc-004 `push-transport`. S2 (T-2306) made an authenticated WS
connection receive the **firehose** — *all* aggregator events. That is unsafe for
production (a client would see other agents' events) and wasteful. **S3 adds the
client-driven per-topic subscription filter**: a WS client calls `hub.ws_subscribe` with
the topics it cares about (its `dm:*`, `agent-presence`, …) and the hub pushes only
matching `hub.event` frames. This also flips the default to **opt-in** — an authed but
un-subscribed connection receives nothing (no accidental firehose).

**Degrade-to-poll** is a protocol property, not new code: the WS is a faster transport for
the *same* aggregator events that remain readable via the existing poll path
(`event.collect` / `channel.subscribe`). If the WS upgrade fails or the socket drops, the
client falls back to polling and misses nothing — the durable substrate stays
authoritative (arc invariant IW-5). This slice proves that contract holds (poll path
unchanged) rather than adding a parallel source of truth.

**S3 scope (this task, hub-side + protocol):** `hub.ws_subscribe` control (auth-gated,
per-connection topic filter with exact + `prefix*` matching); push loop forwards only
matching events; opt-in default; tests. **Out of scope / follow-on:** a dedicated CLI
consumer with an automatic WS-connect→reconnect→poll-fallback loop is a separate consumer
deliverable (S3b, file if/when a live consumer is wired) — bundling it here would violate
one-task-one-deliverable. **Also out of scope:** receipts/journal through WS (S4).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The hub handles a WS-only `hub.ws_subscribe` control message (`params.topics: [..]`) that sets a per-connection topic filter; it is auth-gated (requires an authenticated `observe`-capable connection — an unauthenticated call is refused, not silently accepted) and returns an ack listing the active subscription. `cargo build --release -p termlink-hub` succeeds. *(`maybe_handle_ws_subscribe` — release build green.)*
- [x] After subscribing, the push loop forwards ONLY `hub.event` frames whose `topic` matches the filter; entries match exactly, or as a prefix when written `stem*`. A second `hub.ws_subscribe` replaces the filter. *(`ws_topic_matches` + push-gate; `ws_topic_matches_exact_and_prefix` unit test.)*
- [x] Opt-in default: an authenticated WS connection that has NOT sent `hub.ws_subscribe` receives NO pushes (the S2 firehose is now gated behind an explicit subscribe). The S2 push test was updated to subscribe first, reflecting the tightened contract. *(Empty filter matches nothing; `ws_subscribe_topic_filter` proves the unsubscribed-authed case.)*
- [x] Degrade-to-poll contract: the events delivered over WS are the same aggregator events readable via the existing poll path — a client with no WS (or a dropped WS) still reads them by polling. Verified by all 376 hub tests (line-protocol + event-poll) still passing — the substrate path is untouched.
- [x] A unit test (`server::tests::ws_subscribe_topic_filter`) proves filtering: connect + auth + `hub.ws_subscribe` to a specific topic, inject one matching and one non-matching `AggregatedEvent`, assert only the matching one is pushed; and assert an authed-but-unsubscribed connection receives neither. Passes in CI.

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
cargo build --release -p termlink-hub 2>&1 | tail -3
cargo test -p termlink-hub ws_ 2>&1 | tail -12

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

## Evolution

### 2026-07-02 — S3 built: opt-in per-topic filter, degrade-to-poll as a contract not code
- **What changed:** Added `hub.ws_subscribe` (auth-gated, per-connection `Vec<String>`
  filter) intercepted in the WS read branch before the shared dispatch, `ws_topic_matches`
  (exact + `stem*` prefix), and re-gated the push loop on `authed && filter-match`. This
  flipped S2's firehose to **opt-in** — the S2 push test was updated to `hub.ws_subscribe`
  before expecting a push (contract tightened; documented here per §ACD).
- **Plan impact:** "degrade-to-poll fallback" was realized as a **verified protocol
  contract**, not new fallback code: WS carries the *same* aggregator events the poll path
  (`event.collect`/`channel.subscribe`) already serves, so a client with no/dropped WS
  misses nothing — proven by all 376 hub tests (poll+line path untouched). Building a
  parallel fallback would have added a second source of truth (violates IW-5).
- **Triggered:** The full **CLI consumer** (auto WS-connect → reconnect → poll-fallback
  loop, and wiring `metadata.cv_key`/`dm:*` topic selection) is carved out as **S3b** — a
  distinct consumer-integration deliverable, filed if/when a live consumer is wired
  (one-task-one-deliverable). **S4** (wire delivery-confirm/journal receipts through the WS
  path unchanged) remains the last arc slice. Arc plan otherwise intact.

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

### 2026-07-02T16:08:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2307-v1-ws-s3-client-ws-subscribe-with-per-to.md
- **Context:** Initial task creation
