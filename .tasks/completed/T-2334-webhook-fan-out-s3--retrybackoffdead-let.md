---
id: T-2334
name: "Webhook fan-out S3 — retry/backoff/dead-letter for outbound dispatch (arc-004, follows T-2333)"
description: >
  Slice 3 of the T-2331 GO webhook feature. Slices 1-2 shipped the signed+allowlisted dispatch primitive (T-2332) and the event->dispatch fan-out wiring (T-2333). S3 adds delivery resilience: an in-memory bounded retry queue with per-entry exponential backoff + jitter, poison->dead-letter after N attempts, drained by one hub-startup-spawned loop (mirrors governor::spawn_rate_evict_loop). Reuses the SHAPE of the T-2051 offline-queue flush loop (attempts column + jittered periodic drain + poison->dead-letter) WITHOUT its sqlite store — webhook delivery is best-effort/opt-in and durability-across-hub-restart is lower priority than keeping a Mutex<Connection> off the hot channel.post path. Classify HTTP outcomes: 2xx=success, 4xx=permanent-drop, 5xx/transport=retryable. Observability counters (retry_enqueued/retry_success/dead_letter_total + queue depth) for Slice 4 to surface. See docs/reports/T-2331-webhooks-external-fan-out-inception.md.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-hub/src/server.rs, crates/termlink-hub/src/webhook.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-03T13:39:30Z
last_update: 2026-07-03T13:47:21Z
date_finished: 2026-07-03T13:47:21Z
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

# T-2334: Webhook fan-out S3 — retry/backoff/dead-letter for outbound dispatch (arc-004, follows T-2333)

## Context

Slice 3 of the T-2331 GO webhook feature (arc-004). Slices 1–2 shipped the
signed+allowlisted dispatch primitive (T-2332) and the event→dispatch fan-out
(T-2333). S3 adds delivery resilience so a transient endpoint failure isn't a
silent single-shot drop. In-memory bounded retry queue (VecDeque + per-entry
exponential backoff with jitter), poison→dead-letter after N attempts, drained
by one hub-startup-spawned loop mirroring `governor::spawn_rate_evict_loop`.
Reuses the SHAPE of the T-2051 offline-queue flush loop (`attempts` + jittered
periodic drain + poison→dead-letter) WITHOUT its sqlite store — keeps a
`Mutex<Connection>` off the hot `channel.post` path. **Deliberate tradeoff
(PL-111):** in-memory ⇒ in-flight retries do NOT survive a hub restart; webhook
delivery is best-effort/opt-in, so this is acceptable for v1 and can graduate to
a durable runtime_dir-backed table later. See
`docs/reports/T-2331-webhooks-external-fan-out-inception.md`.

## Acceptance Criteria

### Agent
- [x] HTTP outcome classification: pure `classify_outcome` maps a dispatch result to `Success` (2xx) / `PermanentDrop` (4xx + config errors) / `Retryable` (5xx / transport error). Unit-tested across all classes (`classify_outcome_maps_status_and_errors`).
- [x] Exponential backoff: `backoff_base_ms(attempts)` monotonic non-decreasing, capped at `BACKOFF_CAP_MS` (60s), shift-overflow-guarded; `jitter_ms(base)` applies ±25% via a wall-clock-nanos entropy source (no new crate). Unit-tested (`backoff_base_is_monotonic_and_capped`, `jitter_stays_within_25_percent_bounds`).
- [x] In-memory `RetryQueue` (bounded by `TERMLINK_WEBHOOK_RETRY_CAP`, default 1000): `enqueue` rejects (loud, `dropped_full_total`-counted) when full; `drain_due(now)` returns only due entries; `schedule_retry` dead-letters (terminal drop + `dead_letter_total` + bounded observability ring) at `WEBHOOK_MAX_ATTEMPTS`. Unit-tested (`retry_queue_enqueue_rejects_when_full`, `retry_queue_drain_due_selects_only_ready_entries`, `dead_letter_records_and_counts`, `dead_letter_ring_is_bounded`).
- [x] `fan_out` integration via shared `dispatch_once_and_handle`: first attempt inline (happy path unchanged); a `Retryable` failure enqueues a retry (bumped attempt count); a `PermanentDrop` is dropped + logged, never retried.
- [x] `webhook::spawn_retry_loop()` (mirror of `spawn_rate_evict_loop`, `TERMLINK_WEBHOOK_RETRY_INTERVAL_MS` clamped 250..=60000, default 2000) drains due entries each tick and re-dispatches; wired into BOTH server.rs startup paths after `webhook::init()`; idles when disabled.
- [x] Observability counters (`enqueued_total`, `retry_success_total`, `dropped_full_total`, `dead_letter_total`, `depth`, `dead_letters` snapshot) exposed via accessors for Slice 4. `cargo build -p termlink-hub` + `cargo test -p termlink-hub webhook` green (18 tests); full hub suite 397 green, clippy clean, no new deps.

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

grep -q "fn backoff_delay_ms" crates/termlink-hub/src/webhook.rs
grep -q "fn spawn_retry_loop" crates/termlink-hub/src/webhook.rs
grep -q "struct RetryQueue" crates/termlink-hub/src/webhook.rs
grep -q "crate::webhook::spawn_retry_loop" crates/termlink-hub/src/server.rs
cargo build -p termlink-hub
cargo test -p termlink-hub webhook

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

### 2026-07-03 — in-memory retry queue vs durable sqlite (T-2051 reuse)
- **Chose:** in-memory bounded `VecDeque` retry queue reusing the *shape* of the T-2051 offline-queue flush loop (attempts counter + jittered periodic drain + poison→dead-letter), NOT its SQLite store.
- **Why:** the hub is long-running so durability-across-restart is a real but low-value benefit here — webhook fan-out is best-effort/opt-in and external endpoints are expected to be idempotent. In-memory avoids a `Mutex<Connection>` on the hot `channel.post` path, a new sqlite file, and schema/migration burden. PL-111 flags that restart-surviving state belongs in runtime_dir; this is a deliberate, documented tradeoff (in-flight retries are lost on hub restart) that can graduate to a durable table later if dropped-on-restart webhooks prove to matter.
- **Rejected:** durable `OfflineQueue`-style sqlite table keyed on target+topic — correct only if webhook delivery were governance-critical (T-2243 dead-letter class); it isn't for v1.

### 2026-07-03 — jitter entropy without a new crate
- **Chose:** derive ±25% backoff jitter from `SystemTime::now().subsec_nanos()`.
- **Why:** decorrelates retries across targets (T-2055 thundering-herd guard) without adding the `rand` crate to the hub's dependency surface (Directive 4 — minimal deps). subsec_nanos is more than enough entropy to spread retry times across a fleet of targets.
- **Rejected:** adding `rand` — unnecessary weight for a non-cryptographic jitter; deterministic-only backoff — reintroduces the thundering-herd risk T-2055 fixed for the offline queue.

### 2026-07-03 — retry classification (which failures retry)
- **Chose:** 2xx=Success, 4xx + config errors (bad URL / non-allowlisted host)=PermanentDrop, 5xx/3xx/1xx + transport errors=Retryable.
- **Why:** standard webhook semantics — a 4xx or a config error will never succeed on retry (retrying just wastes attempts and hammers the endpoint), whereas 5xx/transport failures are transient. Prevents a misconfigured target from consuming the whole retry budget.
- **Rejected:** retry everything — would loop a permanently-broken 4xx target to the dead-letter for no benefit.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-07-03T13:39:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2334-webhook-fan-out-s3--retrybackoffdead-let.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-e6abb42e
- **Timestamp:** 2026-07-03T13:47:31Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-07-03T13:47:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
