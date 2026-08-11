---
id: T-2606
name: "Exactly-once is in-memory-only (dedupe LRU) under a durable replay queue — double-apply across hub restart/crash (design-first)"
description: >
  Verb-2 hunt F1+F3: PostDedupe LRU wiped on restart but outbound.sqlite replays; Envelope has no client_msg_id token

status: captured
workflow_type: build
owner: agent
horizon: next
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-11T10:13:46Z
last_update: 2026-08-11T10:13:46Z
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

# T-2606: Exactly-once is in-memory-only (dedupe LRU) under a durable replay queue — double-apply across hub restart/crash (design-first)

## Context

Found by the T-2468 verb-2 ("exchange durable messages") adversarial hunt — findings
F1 (HIGH) + F3 (LOW-MED). Both are the SAME root cause (exactly-once is enforced only
in-memory while the replay queue is durable), so they are filed as one task with two
trigger scenarios. **DESIGN DECISION REQUIRED before implementation** (see Decisions) —
architectural, not a mechanical fix.

**Root cause:** the idempotency guarantee is enforced by an **in-memory** LRU
(`PostDedupe`, `crates/termlink-hub/src/dedupe.rs` — `OnceLock<PostDedupe>` /
`Mutex<HashMap>`, ~lines 49/138-152) that is wiped on hub restart, while the offline
queue that replays posts (`~/.termlink/outbound.sqlite`) is **durable across restarts**.
The durable `Envelope` (`crates/termlink-bus/src/envelope.rs:20-29`) has **no
`client_msg_id` field**, so nothing durable ties a replayed append back to its original.

**Scenario F1 (restart):** Client posts guaranteed msg with `client_msg_id=K`; hub
commits at offset N and `record_offset(K→N)` (`channel.rs:767-784`); TCP RST loses the
ack → client enqueues K to the durable queue (`bus_client.rs:184-189`). Hub **restarts**
(deploy/crash) during the outage — precisely the case the durable queue exists to
survive. The LRU is now empty. Flush replays K → dedupe returns `Newly` → hub **appends
again at offset M**. The subscriber sees the message twice, and because `client_msg_id`
is not in the envelope the duplicate is **undetectable downstream** — a silent
double-apply of a *guaranteed* message.

**Scenario F3 (crash between commit and record):** order is reserve `Pending` →
`bus.post` (commit) → `record_offset` (promote to `Committed`). A crash after commit but
before `record_offset` leaves an orphan `Pending` that ages out by TTL
(`dedupe.rs:210-211`); a post-TTL retry of the same id is `Newly` → second append of an
already-committed message. Narrow crash window; same durable-record fix closes it.

**Violates:** the charter's "exchange durable messages" exactly-once claim + Reliability
directive #2 ("no silent failures"). The two guarantees (durable-replay + exactly-once)
are in architectural tension; this task resolves it.

## Acceptance Criteria

### Agent
- [ ] A design decision is recorded (see Decisions) choosing between the durable-dedupe
      approaches, with the trade-offs (storage cost, envelope-format change, consumer-side
      vs hub-side idempotency) evaluated.
- [ ] The chosen approach durably ties a replayed post to its original so a hub
      restart/crash between the two scenarios above cannot silently double-apply a
      guaranteed message. (E.g. persist a `(sender_id, client_msg_id)→offset` record that
      survives restart, OR stamp `client_msg_id` into the `Envelope` so consumers can
      dedupe — note the envelope has a `#[serde(default)]` metadata map already, so a
      backward-compatible carry is possible without a hard format break.)
- [ ] A load-bearing integration test proves it: post with `client_msg_id=K` → simulate
      ack-loss + dedupe-state-wipe (restart) → replay K → assert NO second append (single
      offset). Prove load-bearing by disabling the durable-record path and confirming the
      test FAILS (observes the double-append).
- [ ] The F3 crash-window (commit-before-record) is covered by the same mechanism or
      explicitly documented as out-of-scope with rationale.
- [ ] Relevant crate tests pass (`cargo test -p termlink-hub -p termlink-bus`).

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

### OPEN — how to make exactly-once durable across a hub restart?

- **Option A — persist the dedupe record hub-side.** A durable
  `(sender_id, client_msg_id)→offset` table (sqlite, alongside the meta db) that survives
  restart; the flush-replay path consults it before appending. Keeps the guarantee
  hub-enforced (consumers unchanged). Cost: extra durable write per guaranteed post +
  a TTL/GC policy for the table (else unbounded growth — mirror the LRU's 5-min TTL, but
  durable). Must ensure the record commits atomically with the append (else re-opens F3).
- **Option B — stamp `client_msg_id` into the `Envelope`.** Carry the id (via the
  existing `#[serde(default)]` metadata map for back-compat) so CONSUMERS can dedupe.
  Cheaper hub-side; but shifts the guarantee to every consumer (a behavior/contract change
  for the charter's exactly-once promise — is it still "exactly-once" if the consumer must
  cooperate?). Weaker default.
- **Option C — accept + document at-least-once across restart.** If the true guarantee is
  "at-least-once + best-effort-dedupe-within-TTL", say so honestly in the charter/docs
  rather than claiming exactly-once. Cheapest; but a downgrade of the stated guarantee —
  needs human sign-off since it changes the charter promise.

Recommendation leans **Option A** (keeps the guarantee hub-side and honest), but this is
a genuine architecture decision — likely warrants a short inception/design doc before
coding. Owner to decide.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-11T10:13:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2606-exactly-once-is-in-memory-only-dedupe-lr.md
- **Context:** Initial task creation
