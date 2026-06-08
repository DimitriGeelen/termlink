---
id: T-2049
name: "Substrate primitive #5 Gap A: client_msg_id + hub LRU dedupe for offline-queue idempotency (T-2023 GO follow-up)"
description: >
  Implement T-2023 Gap A per docs/reports/T-2023-client-reconnect-queue-inception.md §4.A. Closes the double-apply gap: client posts → hub commits at offset N → TCP ack lost → spoke queues retry → hub commits AGAIN at N+1. Fix: client generates client_msg_id (UUID v4 or content-hash), hub maintains short-TTL (~5min) recently-seen LRU keyed by (sender_fingerprint, client_msg_id), no-ops duplicates. ~80 LOC.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-parallel-substrate, substrate-primitive, resilience]
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/agent.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs, crates/termlink-hub/src/channel.rs, crates/termlink-hub/src/lib.rs, crates/termlink-hub/src/router.rs, crates/termlink-hub/src/server.rs, crates/termlink-session/src/bus_client.rs, crates/termlink-session/src/offline_queue.rs, crates/termlink-session/tests/bus_client_integration.rs]
related_tasks: [T-2018, T-2023, T-1439]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-08T10:49:24Z
last_update: 2026-06-08T16:09:12Z
date_finished: 2026-06-08T16:09:12Z
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

# T-2049: Substrate primitive #5 Gap A: client_msg_id + hub LRU dedupe for offline-queue idempotency (T-2023 GO follow-up)

## Context

Closes T-2023 Gap A (IW-4 — idempotency). See
`docs/reports/T-2023-client-reconnect-queue-inception.md` §4.A for the
problem (TCP ack lost → spoke retries → hub double-applies the post at
two offsets) and the recommended shape (`client_msg_id` opaque token +
short-TTL hub-side LRU keyed by `(sender_fingerprint, client_msg_id)`,
silent no-op on dup that returns the cached `{offset, ts}`).

Backward compatible: `client_msg_id` is optional. Old clients keep
working unchanged; opt-in callers gain exactly-once semantics across
hub blips.

## Acceptance Criteria

### Agent
- [x] New module `crates/termlink-hub/src/dedupe.rs` exports a `PostDedupe` struct: TTL-bounded + capacity-bounded LRU keyed by `(sender_id, client_msg_id)`; entries store the cached `{offset, ts_unix_ms}` so a duplicate returns the original success envelope.
- [x] `PostDedupe` API: `try_record_or_lookup(sender_id, client_msg_id, now_ms, offset, ts) -> Outcome { Newly_recorded, Duplicate { offset, ts } }` plus `evict_expired(now_ms)` + accessors `entries_active()` / `hits_total()`.
- [x] Defaults: TTL = 5 min (`DEFAULT_DEDUPE_TTL_MS = 300_000`), capacity = 10_000 entries (`DEFAULT_DEDUPE_CAPACITY = 10_000`). Both override-able via `TERMLINK_DEDUPE_TTL_MS` / `TERMLINK_DEDUPE_CAPACITY` env vars at hub start.
- [x] `OnceLock` global `post_dedupe()` accessor with `init()` matching `governor.rs` pattern; wired into `run_with_tcp` + `run_blocking`.
- [x] `handle_channel_post_with` reads optional `client_msg_id` from params (string, 1..=128 chars). When present AND identity verified: checks dedupe BEFORE `bus.post()`. Cache hit → return the cached `Response::success(id, {offset, ts})` envelope without re-appending. Cache miss → post normally + record after success.
- [x] `hub.governor_status` response gains three sibling fields: `dedupe_entries_active`, `dedupe_hits_total`, `dedupe_ttl_ms`. MCP parity tool returns the same shape automatically (passthrough).
- [x] `PendingPost` (offline_queue) gains optional `client_msg_id: Option<String>` with `#[serde(default, skip_serializing_if = "Option::is_none")]` — old persisted rows deserialize cleanly with `None`, new rows persist + replay the id.
- [x] CLI `cmd_channel_post` accepts an optional `client_msg_id: Option<String>` argument; when absent, mints a fresh random 128-bit hex id at call time (via `rand::thread_rng` in `termlink-session::offline_queue::mint_client_msg_id`). The minted id is passed both directly to the hub AND persisted to `PendingPost` so a queue replay reuses the same id.
- [x] Unit tests: 9 for `PostDedupe` (insert-then-hit-returns-cached, distinct-sender-no-collision, distinct-msg-no-collision, ttl-eviction, lru-eviction-at-capacity, hit-counter-increments, ttl-anchors-to-first-sighting, evict-expired-explicit, zero-ttl-clamps).
- [x] Integration tests in `channel.rs::tests`: 4 (no-id-bypasses-dedupe-and-posts-normally, with-id-first-post-succeeds-and-records, with-id-duplicate-returns-cached-offset, oversized-id-ignored).
- [x] `cargo test -p termlink-hub` passes (330/330). `cargo test -p termlink-session` passes (334+). `cargo check -p termlink` passes (cli).
- [x] Live smoke on test hub (`TERMLINK_DEDUPE_TTL_MS=60000`): `hub.governor_status` returns the three new fields. Two CLI posts with `--client-msg-id T2049-SMOKE-AAA` produce offset=0 both times (deduped=true on second); third with `T2049-SMOKE-BBB` produces offset=1. `dedupe_entries_active=2`, `dedupe_hits_total=1`, bus has exactly 2 envelopes (dedupe absorbed the retry).
- [x] Docs: `docs/operations/substrate-post-idempotency.md` (~200 lines) explains the wire shape, hub TTL, operator probe recipe, end-to-end queue-replay scenario, and explicit non-goals. CLAUDE.md Quick Reference: governor row extended with dedupe fields + new "Post idempotency (EXACTLY-ONCE)" row.

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

cargo check -p termlink-hub
cargo check -p termlink-session
cargo check -p termlink
cargo test -p termlink-hub --lib dedupe::tests --no-fail-fast 2>&1 | grep -q "test result: ok"
out=$(cargo test -p termlink-hub --lib channel::tests::dedupe 2>&1); echo "$out" | grep -q "test result: ok"
out=$(cargo test -p termlink-session --lib offline_queue 2>&1); echo "$out" | grep -q "test result: ok"
test -f docs/operations/substrate-post-idempotency.md
grep -q "TERMLINK_DEDUPE_TTL_MS" docs/operations/substrate-post-idempotency.md
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

### 2026-06-08 — workspace lacked `uuid` crate; minted via `rand` instead

- **What changed:** The captured spec said "UUID v4 via existing uuid crate". The workspace doesn't pull `uuid` — `rand` 0.8 is in `termlink-session` already (the load-bearing crate for the offline queue), so we added a tiny `mint_client_msg_id()` helper that fills 16 bytes from `rand::thread_rng()` and hex-encodes them. Same 128-bit entropy as UUID v4 random variant; opaque to the hub either way.
- **Plan impact:** No spec change — id is opaque on the wire. New `pub fn` in `termlink-session::offline_queue` keeps the call site (in `termlink-cli`) free of a new dependency.
- **Triggered:** Nothing — net positive (no new workspace dep).

### 2026-06-08 — race-window decision: NO pre-reservation, post-success record

- **What changed:** The original sketch had `try_record_or_lookup` pre-reserve a placeholder entry on miss, so two concurrent retries with the same id would both miss the cache, both `bus.post`, and the hub would double-apply. With pre-reservation, the second concurrent call would see the placeholder and wait. WITHOUT pre-reservation, the race window remains for concurrent retries from the SAME sender_id with the SAME client_msg_id.
- **Plan impact:** Accepted the trade-off: clients post serially on a given connection (FIFO offline queue → one in-flight at a time). The realistic case is sequential retry (ack lost → wait → retry), which the post-success record path catches reliably. Pre-reservation adds complexity (placeholder offsets, recovery paths) for a degenerate misbehaving-client case.
- **Triggered:** Documented inline in `try_record_or_lookup`. If a future production incident shows concurrent same-id retries (misbehaving spoke), escalate to pre-reservation via a follow-up task.

### 2026-06-08 — bus offset type is u64 not i64

- **What changed:** `termlink-bus::log::Offset = u64`; first slice 1 draft used `i64` for the dedupe cache's offset field. Compile-fail in slice 2 when wiring to `bus.post()`. Changed `DedupeEntry.offset` and the API to `u64` to match.
- **Plan impact:** None — i64/u64 are interchangeable in JSON serialization. Bus consumers (subscribe response) already render as `u64`.
- **Triggered:** Nothing.

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

### 2026-06-08T10:49:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2049-substrate-primitive-5-gap-a-clientmsgid-.md
- **Context:** Initial task creation

### 2026-06-08T15:46:28Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-06-08T16:09:12Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
