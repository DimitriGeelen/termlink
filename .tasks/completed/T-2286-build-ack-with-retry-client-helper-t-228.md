---
id: T-2286
name: "Build ack-with-retry client helper (T-2285 Design A)"
description: >
  Build ack-with-retry client helper (T-2285 Design A)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/agent.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs, crates/termlink-session/src/ack_retry.rs, crates/termlink-session/src/lib.rs]
related_tasks: [T-2285, T-2049, T-2051, T-1485, T-2323]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-25T23:07:34Z
last_update: 2026-06-25T23:33:18Z
date_finished: 2026-06-25T23:33:18Z
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

# T-2286: Build ack-with-retry client helper (T-2285 Design A)

## Context

Producer-side closure of §9 hard-dep #5 (parallel-execution harness). T-2285
inception (completed, GO recorded `7eb1682b`) ratified **Design A**: ack-with-retry
needs **no hub-side delivery state** — the exactly-once leg already exists (T-2049
dedupe), the durability pattern exists (T-2051 outbound.sqlite), and the recipient-ack
signal exists (the `channel.receipts` frontier `up_to >= offset`). The substrate's
contribution is a small, reversible, **client-side** retry helper + CLI verb +
the documented recipient auto-ack convention for the AEF sidecar. Design doc:
`docs/reports/T-2285-ack-with-retry-inception.md` (§"Design A").

## Acceptance Criteria

### Agent
- [x] A durable **awaiting-ack tracker** persists the post identity + attempts to a SQLite store, reusing the T-2051 `outbound.sqlite` conventions (single-writer `Mutex<Connection>`, `TERMLINK_IDENTITY_DIR`-aware path); a confirmed ack deletes the row. — `AwaitingAckTracker` in `crates/termlink-session/src/ack_retry.rs` (schema `awaiting_ack(dm_topic, msg_offset, client_msg_id UNIQUE, recipient_sender_id, attempts, enqueued_ms)`; `record`/`bump_attempts`/`confirm`/`get`/`list`). See Decisions re: schema refinement.
- [x] An `await_ack_with_retry` helper in `termlink-session` posts with a **stable** `client_msg_id`, polls the receipt frontier until the recipient's `up_to >= offset` OR the deadline, and on deadline **re-posts the SAME `client_msg_id`** (≤ `max_attempts`) — relying on T-2049 dedupe for exactly-once. — `ack_retry::await_ack_with_retry` (generic over post/receipts/clock closures) + `await_ack_with_retry_realtime` wrapper.
- [x] CLI verb surface `channel post --await-ack [--retry] [--ack-timeout-secs N] [--max-attempts N]` wires the helper; **without** `--await-ack` the post path is byte-for-byte unchanged (backward compatible — await-ack is a separate early-return branch). — `cli.rs` flags (with `requires = "await_ack"`, verified live), `main.rs` dispatch, `run_await_ack` in `channel.rs`.
- [x] A test proves retry-after-dead-recipient is exactly-once: the recipient withholds its ack until after ≥1 retry; asserts (a) exactly **one** append across both attempts (dedupe absorbs the retry), (b) the helper returns success once `up_to >= offset`. — `ack_retry::tests::retry_after_dead_recipient_is_exactly_once` (sender half) complements hub-side `dedupe_with_client_msg_id_duplicate_returns_cached_offset` (hub half). See Decisions re: two-halves strategy.
- [x] The **recipient auto-ack convention** is documented for the AEF harness — `docs/operations/substrate-ack-with-retry.md` § "Recipient half" + substrate ADR §6 #5 cross-reference, explicitly flagged as the AEF-layer responsibility.
- [x] Retry-policy defaults align with AEF §6 heartbeat numbers (poll cadence ~5s, deadline ~30s, 3 attempts; documented tunable via `--ack-timeout-secs`/`--max-attempts`). — `RetryPolicy::default()` + `from_operator` + `from_operator_clamps_and_defaults` test.

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

cargo check -p termlink-session -p termlink
cargo test -p termlink-session ack_retry

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

### 2026-06-26 — Orchestration logic lives in termlink-session, generic over closures
- **Chose:** Put both the durable tracker AND the `await_ack_with_retry` loop in `termlink-session`, with the loop generic over injected `post_fn` / `receipts_fn` / clock / sleep closures. The CLI (`run_await_ack`) supplies real closures wrapping `rpc_call_authed`.
- **Why:** The exactly-once invariant becomes provable with **zero hub scaffolding** — a fake `post_fn` that simulates T-2049 dedupe + a fake clock the `sleep_fn` advances make the retry test instant and deterministic. Keeps the receipts-RPC plumbing (CLI-layer) out of the helper.
- **Rejected:** Putting the loop in `termlink-cli` (couldn't unit-test without a live hub) and a hub-side enforced-redelivery design (Design B — makes the hub delivery-stateful; rejected at inception for blast radius).

### 2026-06-26 — Exactly-once proven in two halves, not one live-hub integration test
- **Chose:** Prove the hub dedupe leg via the existing `termlink-hub` test (`dedupe_with_client_msg_id_duplicate_returns_cached_offset`) and the sender reuse leg via `ack_retry::tests::retry_after_dead_recipient_is_exactly_once` (dedupe-honouring fake hub).
- **Why:** The full chain is covered without standing up an in-process hub that implements BOTH dedupe AND `channel.receipts` (neither existing test harness does both). Each half is sharp and fast.
- **Rejected:** Extending the `FakeHub` in `bus_client_integration.rs` to implement dedupe + receipts — more scaffolding for no additional coverage, since the hub dedupe is already independently tested.

### 2026-06-26 — Tracker schema refinement vs the literal AC
- **Chose:** Persist `(dm_topic, msg_offset, client_msg_id, recipient_sender_id, attempts, enqueued_ms)` rather than the AC's literal `retry_deadline_ms`.
- **Why:** The per-attempt deadline is a transient computed from `now + deadline_ms` inside the loop — persisting it would be stale on reload. `recipient_sender_id` (whose ack we await) is the genuinely useful durable field for a recovery sweep. Tracker durability + confirm-on-ack semantics (the AC's substance) are fully met.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-06-25T23:07:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2286-build-ack-with-retry-client-helper-t-228.md
- **Context:** Initial task creation

### 2026-06-25T23:33:18Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
