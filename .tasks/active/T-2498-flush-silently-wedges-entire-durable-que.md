---
id: T-2498
name: "flush silently wedges entire durable queue when head row fails to deserialize — peek Err collapsed into empty-queue break"
description: >
  flush silently wedges entire durable queue when head row fails to deserialize — peek Err collapsed into empty-queue break

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
created: 2026-08-02T11:33:00Z
last_update: 2026-08-02T11:35:56Z
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

# T-2498: flush silently wedges entire durable queue when head row fails to deserialize — peek Err collapsed into empty-queue break

## Context

`BusClient::flush` (`crates/termlink-session/src/bus_client.rs:201`) reads the head
row of the durable offline queue via
`let Ok(Some((id, post, attempts))) = self.queue.peek_oldest_with_attempts() else { break };`.
`peek_oldest_with_attempts` ends in `serde_json::from_str(&json)?`, so it returns
`Err` whenever the head row's `post_json` fails to deserialize (schema drift across a
binary upgrade, a partially-written row, or on-disk corruption). The `let-else`
collapses that `Err` into the SAME silent `break` as the empty-queue `Ok(None)` case —
no log, no counter, no dead-letter. The corrupt row can never reach the poison/
dead-letter machinery (all of which lives AFTER a successful peek), so every flush
tick re-peeks it, breaks, and **nothing behind it ever drains**: the entire durable
queue is permanently wedged behind one undeserializable row, and the charter's core
"exchange durable messages" verb silently stops working. The only eventual symptom is
`enqueue` returning `QueueFull` once cap fills — a loud error whose true root cause is
invisible. Sibling of T-2497 (that hardened the success-path pop; this hardens the
peek — the queue's entry gate). Directive-#2 breach (silent failure + total drain stall).

## Acceptance Criteria

### Agent
- [x] A corrupt/undeserializable head row is dead-lettered (durable, recoverable) instead of silently wedging the queue — the queue drains past it
- [x] The failure surfaces LOUD (`tracing::error!` with the row id + serde error) instead of a silent `break`
- [x] The peek `Err` path is disambiguated from the empty-queue `Ok(None)` path via an explicit match (not a `let-else` that collapses both)
- [x] A non-deserializing `OfflineQueue::peek_head_id()` helper reads the head id so a corrupt row can be quarantined without re-triggering the deserialize failure
- [x] A genuine DB fault (peek_head_id itself errors) breaks the pass LOUD rather than spinning
- [x] Regression test: seed a queue whose head row's `post_json` is corrupt, run `flush()`, assert it dead-letters the row (recoverable in the dead-letter store) and drains — no hang, no wedge
- [x] `cargo test -p termlink-session --lib` passes (existing + new)

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
cargo test -p termlink-session --lib bus_client 2>&1 | tail -5 | grep -q "test result: ok"
cargo test -p termlink-session --lib offline_queue 2>&1 | tail -5 | grep -q "test result: ok"

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

**Symptom:** A single undeserializable head row in the durable offline queue
(`~/.termlink/outbound.sqlite`) permanently wedges the whole queue: `flush()` reads
the head, the deserialize fails, and the loop breaks. Every subsequent tick repeats,
so no message enqueued behind the corrupt row ever drains. The queue looks non-empty
and idle in `queue-status`; the only eventual signal is `enqueue` → `QueueFull` once
cap (default 1000) fills, whose root cause (one bad head row) is invisible.

**Root cause:** `bus_client.rs:201` used
`let Ok(Some((id, post, attempts))) = self.queue.peek_oldest_with_attempts() else { break };`.
`peek_oldest_with_attempts` (`offline_queue.rs:249`) ends in `serde_json::from_str(&json)?`
— an `Err` on a corrupt/schema-drifted row. The `let-else` maps BOTH `Err(..)` and
`Ok(None)` to the same `break`, so a corrupt head row is indistinguishable from an
empty queue. The poison/dead-letter machinery (T-1439/T-2243) that would quarantine a
bad row all lives AFTER a successful peek, so a peek-stage failure can never reach it.

**Why structurally allowed:** `let-else` with a refutable `Ok(Some(..))` pattern is a
common, terse idiom that reads as "get the next item or stop" — but it silently
swallows the discriminant between "no more items" and "the read itself failed." The
hardening effort in this function (T-2450/T-2452/T-2439/T-2243/T-2497) all targeted
the branches AFTER the peek; the peek itself — the queue's entry gate — was never
examined because it "obviously just returns the next row." No lint flags a `let-else`
that discards an `Err`.

**Prevention:** Replace the `let-else` with an explicit 3-arm match that routes the
peek `Err` to the durable dead-letter store (via a new non-deserializing
`peek_head_id()` + the existing `dead_letter()`, which copies the raw `post_json`
blob without parsing), loud `tracing::error!`, and `continue` — so the queue drains
past the corrupt row instead of wedging. A regression test seeds a corrupt head row
and asserts `flush()` dead-letters + drains (no hang). Captured as a learning in the
no-silent-failures class (PL-283/PL-276): a `let-else`/`if-let` over a fallible read
collapses "absent" with "read failed" — match all arms so a read failure is loud and
routed, never mistaken for end-of-data.

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

### 2026-08-02T11:33:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2498-flush-silently-wedges-entire-durable-que.md
- **Context:** Initial task creation
