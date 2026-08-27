---
id: T-2709
name: "Stuck-claim heuristic is a monotonic latch that never clears"
description: >
  is_potentially_stuck (channel.rs:11475) fires on expired_count>0, but expired claim
  rows are reaped ONLY when someone re-claims that exact (topic,offset) (meta.rs:434).
  On a topic nobody re-claims, one abandoned claim marks it potentially_stuck forever,
  with active_count=0 so nothing is actually held. The T-2556 canary built on it then
  fires daily for the life of the host. Add a recency window so an expired lease —
  which IS the substrate's own auto-recovery mechanism — stops being reported as a
  current fault.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: [crates/termlink-bus/src/claim.rs, crates/termlink-bus/src/lib.rs, crates/termlink-bus/src/meta.rs, crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/commands/substrate.rs, crates/termlink-cli/src/main.rs, crates/termlink-hub/src/channel.rs, crates/termlink-mcp/src/tools.rs, crates/termlink-protocol/src/control.rs, crates/termlink-session/src/claim_client.rs, scripts/agent-conversation-selftest.sh, scripts/check-stuck-claims-freshness.sh, scripts/lib/reap-topic.sh, scripts/substrate-smoke.sh, scripts/substrate-worker-pickup.sh, scripts/test-agent-conversation-list.sh, scripts/test-agent-conversation-status.sh, scripts/test-agent-respond.sh, scripts/test-agent-send.sh, scripts/test-agent-send-transport.sh, scripts/test-journal-mirror.sh, tests/reap-topic-fixtures.sh, tests/stuck-claims-check-fixtures.sh]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-14T16:38:41Z
last_update: 2026-08-27T10:01:51Z
date_finished: 2026-08-27T10:01:51Z
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
bvp_scores_proposed:
  - ts: '2026-08-23T19:13:29Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=3 (body:portability-abstraction); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-08-23T19:13:47Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2709: Stuck-claim heuristic is a monotonic latch that never clears

## Context

Found while dispositioning T-2706, which had been filed as "the stuck-claims
canary fires on 11 test-residue topics — clean them up or exclude them."
Reading the code showed the topics were incidental. The predicate itself could
never return to green.

**The two halves, both in-tree:**

```rust
// crates/termlink-cli/src/commands/channel.rs
fn is_potentially_stuck(summary: &ClaimsAggregate) -> bool {
    summary.expired_count > 0                                   // ← latches
        || summary.oldest_active_age_ms.map(|a| a > 60_000).unwrap_or(false)
}
```

```sql
-- crates/termlink-bus/src/meta.rs — the ONLY reap of expired rows
DELETE FROM claims
 WHERE topic = ?1 AND offset = ?2 AND claimed_until <= ?3
```

The reap is scoped to a single `(topic, offset)` and runs only inside the claim
path. Nobody re-claims an abandoned topic, so nothing ever deletes the row, so
`expired_count > 0` holds forever.

**Live measurement (workstation-107, 770 topics):** 11 flagged stuck, every one
with `active_count: 0` — nothing held, nothing that *could* be stuck — with
expired counts from 1 to 81. Two sampled directly via
`channel claims --include-expired`: `work-queue` and `drain-probe-1425555` both
lapsed ~62 days ago. The canary (T-2556) had been firing daily on two-month-old
history.

**Scope note.** The fix deliberately does not reap expired rows, which was the
other candidate remedy. Reaping would destroy the forensic record
`--include-expired` exists to serve ("who held this offset before it lapsed?"),
and it would trade a false-positive problem for a data-loss one. The rows are
fine; reading them as current state was the defect.

## Post-fix measurement — and why the green number is only half the story

Re-measured with the freshly-built binary against the running hub:

```
$ ./target/release/termlink channel claims-summary --all --only-stuck --json
{"ok":true,"only_stuck":true,"shown":0,"stuck_count":0,"topic_count":770,"topics":[]}
```

`stuck_count` 11 → 0. But checking *why* before claiming credit:

```
$ ./target/release/termlink channel claims-summary work-queue --json
{"active_count":0,"expired_count":1,"newest_expired_at_ms":null, ...}
```

`expired_count: 1` with `newest_expired_at_ms: null` is impossible on a hub that
serves the field — the row exists and has a `claimed_until`. So the running hub
(started ~1 day ago, pre-T-2709) omits it, and the 0 came from the **back-compat
path**, not from the recency window doing its job.

What that live run *does* prove: the new client works against an old hub without
error, and absent-reads-as-not-stuck behaves as designed outside the unit tests.
What it does **not** prove: that the window discriminates recent from ancient.
Only the unit tests cover that, until a hub runs the new binary.

**This surfaced a second defect, fixed here.** Against such a hub the
abandoned-claim arm is not merely conservative — it is *completely inert*, and
the check was reporting a cheerful `healthy (770 topics, 0 stuck)` while
structurally unable to see that entire class. Green because we cannot look is
not green. Two additions:

- `claims-summary --all --json` computes `expired_arm_inert` (some topic has
  expired rows but carries no marker — impossible on a capable hub) and emits a
  stderr note naming the missing field and the remedy.
- The canary appends a `DEGRADED:` clause to its healthy line. Deliberately
  **non-firing** — a capability gap is not a stuck claim, which is PL-219's rule
  — but never silent.

Pinned by four fixtures, including that an older CLI omitting the field reads as
not-degraded rather than crying wolf.

**Live confirmation against the real hub (2026-08-14, post-fix).** The fixtures
prove the logic; this proves the wiring. Run on this host against the running
(pre-T-2709) hub:

```
$ ./target/release/termlink channel claims-summary --all --only-stuck --json
{"expired_arm_inert":true,"ok":true,"only_stuck":true,"shown":0,
 "stuck_count":0,"topic_count":770,"topics":[]}
note: this hub does not serve `newest_expired_at_ms` (pre-T-2709), so the
abandoned-claim half of the stuck check cannot fire — only the oldest-active-age
half is live. Upgrade the hub binary and restart it through its unit to restore
full detection.

$ TERMLINK_BIN=./target/release/termlink bash scripts/check-stuck-claims-freshness.sh --no-heartbeat
check-stuck-claims: healthy (770 topics, 0 stuck) (DEGRADED: hub predates T-2709
and omits newest_expired_at_ms — the abandoned-claim half of this check is inert;
upgrade + restart the hub through its unit to restore it)
exit=0
```

Both halves behave as designed: the canary no longer fires daily on the 11
latched topics, and the `0 stuck` it now reports is explicitly qualified rather
than passed off as a clean bill. Exit 0 is correct here — a stale hub binary is
a capability gap, not a stuck claim (PL-219) — and it is T-2707's problem, not
this one's.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The latch is proven, not asserted: a test shows a topic with only-expired claims reports `potentially_stuck: true` indefinitely under the old predicate, because the lazy reap in `meta.rs` is scoped `WHERE topic = ?1 AND offset = ?2` and therefore never runs for an offset nobody re-claims
- [x] `ClaimsSummary` gains a recency marker (`newest_expired_at_ms` — `MAX(claimed_until)` over rows where `claimed_until <= now`), computed in the SAME single SQL aggregate so the read stays one query
- [x] The marker is plumbed end-to-end and is not silently dropped at any hop: bus `ClaimsSummary` → hub `channel.rs` JSON → session `claim_client::ClaimsAggregate` → CLI predicate → MCP envelope
- [x] `is_potentially_stuck` fires on an expired claim ONLY when the expiry is recent (within a stated window), so the flag self-clears; a long-abandoned topic goes quiet
- [x] The active-claim half of the predicate (`oldest_active_age_ms > 60_000`) is preserved unchanged — that half was never the defect
- [x] Backward compatibility is explicit: a hub predating this change omits the field, and the client treats absent as "no recent expiry" rather than defaulting to stuck (an older hub must not start reporting every topic stuck)
- [x] The semantic argument is recorded in-code: an expired lease is the substrate's OWN documented auto-recovery ("if the holder is dead the lease auto-expires"), so reporting a successfully-expired lease as a current fault contradicts the design it is monitoring
- [x] `cargo test --workspace` green
- [x] The 11 live topics on this host are re-measured after the fix and the new `stuck_count` is reported honestly, whatever it is — measured `stuck_count: 0` (from 11), but see the caveat below: on THIS hub that number comes from the back-compat path, not from the recency window
- [x] The half-inert case is not reported as plain "healthy" — a hub that omits `newest_expired_at_ms` cannot fire the abandoned-claim arm at all, so `stuck_count: 0` there means "could not look", and both the CLI (`expired_arm_inert` + stderr note) and the canary (`DEGRADED:` suffix) now say so

### Human
- [ ] [REVIEW] Confirm the recency window is the right semantic
  **Steps:**
  1. Read the new predicate in `crates/termlink-cli/src/commands/channel.rs` (`is_potentially_stuck`)
  2. Decide whether "a claim expired within the window" is the signal you want, versus dropping the expired half of the predicate entirely
  **Expected:** agreement that a recently-abandoned worker is worth flagging but an ancient one is not
  **If not:** say which semantic you want and the predicate changes to match

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
cargo test -p termlink-bus claims_summary
cargo test -p termlink stuck
cargo test -p termlink expired_claims_only_flag_stuck_while_recent
cargo test -p termlink absent_expiry_marker_is_not_stuck
bash scripts/run-guard-layer.sh

# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

## RCA

**Symptom:** `channel claims-summary --all --only-stuck` reports 11 of 770
topics `potentially_stuck`. Every one has `active_count: 0` — nothing is held —
and expired counts from 1 to 81. The T-2556 canary sits on this predicate and
had been firing daily on all of them.

**Root cause:** `is_potentially_stuck` fired on `expired_count > 0`, and
`expired_count` never decreases. Expired claim rows are reaped lazily by the
`DELETE` inside the claim path (`meta.rs`), scoped
`WHERE topic = ?1 AND offset = ?2` — so a row is only removed when someone
re-claims that *exact* offset. On a topic nobody re-claims, the row lives for
the life of the hub's SQLite file. The predicate was therefore a **monotonic
latch**: once true, true forever, regardless of the system being perfectly
healthy.

**Why structurally allowed:** two gaps, one mechanical and one conceptual.

Mechanically, the unit test asserted exactly the latching behaviour
(`expired_stuck.expired_count = 1; assert!(is_potentially_stuck(...))`) — it
pinned the bug in place as intended behaviour, so every test run confirmed it.
The test could not distinguish "fires on abandonment" from "fires forever"
because it never advanced a clock; there was no time axis in the test at all.

Conceptually, the predicate contradicted the design it monitored. Lease expiry
is the substrate's own documented auto-recovery — "if the holder is dead the
lease auto-expires" — so the code treated the recovery mechanism as the fault.
Nothing in the guard layer checks a *semantic* inversion of that kind; the
static checks ask about resource safety, not "does this alarm mean what its
name says".

The blindness is the more expensive half. This is the G-019 pattern one level
up: a canary was built (T-2556) on a predicate that could never clear, so the
guard's daily firing carried zero information. A guard that fires every day
independent of system state doesn't just fail to help — it consumes the
attention a real firing would need, and trains its operator to stop looking.
That is the same failure this session documented in T-2680 (a canary
over-reporting its scope) and T-2699 (refusals that could never be emitted).

**Prevention:** distinct from the fix.
- The replacement test injects `now_ms` and asserts *both* edges of the window,
  including one millisecond past it. A latch cannot pass it.
- `is_potentially_stuck_at` is a pure clock-free function, so the time-dependent
  behaviour is testable at all — the previous shape made the defect unprovable.
- The corrected semantics are recorded where a reader hits them: the field doc
  on `newest_expired_at_ms`, the predicate doc, the protocol RPC doc, both MCP
  tool descriptions, `docs/operations/substrate-claim-primitive.md` (a new
  "Reading `expired_count` correctly" section), the cron recipe, CLAUDE.md, and
  `.claude/commands/claims.md` — every place that previously taught the wrong
  rule now teaches why it was wrong.
- Back-compat is pinned by its own test: an absent field must read as "not
  stuck", never as stuck, or an older hub lights up every topic at once.

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

## Recommendation

**Recommendation:** GO — keep the recency window as shipped: flag a claim that
expired recently, stay quiet about one that expired long ago.

**Rationale:** The substrate's own documented recovery is "if the holder is dead
the lease auto-expires". Reporting a *successfully* expired lease as a current
fault contradicts the design the canary is monitoring. The old predicate
(`expired_count > 0`) was a monotonic latch: expired rows are reaped lazily and
only when the SAME `(topic, offset)` is re-claimed, so on a topic nobody
re-claims the row persists for the life of the hub's SQLite and the verdict never
clears. A canary that fires daily on permanent debris is precisely how an
operator learns to stop reading it — the failure this task exists to end.

**Evidence:** 11 live topics on this host had latched true under the old
predicate. Every one of them had `active_count: 0` — nothing held, nothing that
*could* be stuck — and one carried 81 expired rows. All 10 Agent ACs are ticked
and `cargo test --workspace` is green. The recency marker
(`newest_expired_at_ms`) is plumbed end-to-end (bus → hub → session → CLI → MCP)
and a hub predating it reports absent, which reads as "no recent expiry", never
as stuck.

**What you are actually deciding.** Not whether the fix works — all 10 Agent ACs
are ticked and `cargo test --workspace` is green. You are deciding a *semantic*:
should a lapsed lease ever count as "stuck"? Three options were live:

| Option | Behaviour | Cost |
|---|---|---|
| Recency window (shipped) | expired-within-window fires; ancient expiry goes quiet | a genuinely abandoned topic goes silent once it ages out |
| Drop the expired arm entirely | only `oldest_active_age_ms > 60_000` fires | a crashed worker's lease is never surfaced at all |
| Keep `expired_count > 0` (the old predicate) | any expiry fires, forever | what this task exists to fix |

**Evidence for the shipped choice.** The substrate's own documented recovery is
"if the holder is dead the lease auto-expires". Reporting a *successfully* expired
lease as a current fault contradicts the design the canary is monitoring. The old
predicate had latched 11 topics true — every one with `active_count: 0`, i.e.
nothing held and nothing that *could* be stuck, one carrying 81 expired rows.
A canary firing daily on permanent debris is how an operator learns to stop
reading it.

**The caveat that WAS load-bearing, and has since cleared.** When AC #10 was
written the measured `stuck_count: 0` came from the **back-compat path**, not
from the recency window: the local hub predated `newest_expired_at_ms` and
omitted it, so the abandoned-claim arm could not fire here at all. AC #10 made
that visible rather than letting `0` read as proven (`expired_arm_inert` on the
CLI, `DEGRADED:` on the canary), and the recommendation originally rested on the
argument rather than on a live measurement.

Re-measured 2026-08-27 against hub binary `0.11.1612`:
`channel claims-summary --all --json` now reports **`expired_arm_inert: false`**
over 19 topics with `stuck_count: 0`, and the canary prints plain
`healthy (19 topics, 0 stuck)` with no `DEGRADED:` suffix. **The recency arm is
live and green.** So the green number is now a real measurement of the new
predicate, not the degraded path — the decision below is better-evidenced than
when it was first written, and the hub upgrade this caveat was waiting on has
already happened.

**Why I should not decide this.** The two rejected options are defensible. Which
signal you want out of the claim rail is a judgement about how you intend to use
it — whether an abandoned work item should eventually stop asking for attention,
or should nag until someone clears it. That is an operator's preference about
noise, not a correctness question I can settle from the code.

**If you disagree:** name the semantic you want and the predicate changes to match
— it is a single function, `is_potentially_stuck` in
`crates/termlink-cli/src/commands/channel.rs`.

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

### 2026-08-14T16:38:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2709-stuck-claim-heuristic-is-a-monotonic-lat.md
- **Context:** Initial task creation

### 2026-08-14T16:39:41Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-a32d5255
- **Timestamp:** 2026-08-27T10:05:35Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-27T10:01:51Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
