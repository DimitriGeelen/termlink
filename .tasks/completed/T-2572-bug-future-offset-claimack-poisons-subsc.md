---
id: T-2572
name: "BUG: future-offset claim+ack poisons subscriber cursor (silent data loss)"
description: >
  BUG: channel claim at an offset beyond the topic frontier, then release --ack, advances
  the subscriber cursor arbitrarily far past all real messages — silent unbounded
  data loss. Fix: reject a claim whose offset is beyond the current frontier. Found
  in T-2468 purpose-review verb-2 hunt.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-bus/src/error.rs, crates/termlink-bus/src/lib.rs, 
      crates/termlink-bus/src/meta.rs, crates/termlink-hub/src/channel.rs, 
      crates/termlink-protocol/src/control.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-09T14:45:34Z
last_update: '2026-08-18T18:59:13Z'
date_finished: 2026-08-09T14:52:29Z
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
  - ts: '2026-08-18T18:56:52Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 2
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=2 
      (body:telemetry-or-audit-entry); D3=2 (body:default-change); D4=2 
      (body:env-class-handled); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:13Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 5
      tier: 2
      effort: 8
    rationale: blast_radius=5 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2572: BUG: future-offset claim+ack poisons subscriber cursor (silent data loss)

## Context

Found in T-2468 purpose-review verb-2 hunt (durable-message path). `claim_offset`
(crates/termlink-bus/src/meta.rs:400) enforces only `UNIQUE(topic,offset)` + TTL —
it does NOT check the claimed offset against the topic frontier (`next_offset`).
`release_claim` with `ack=true` (meta.rs:489-497) advances the claimer's cursor to
`MAX(last_offset, offset+1)` — monotonic, never rewinds. So a claim on an offset
far beyond the frontier, then `release --ack`, poisons the subscriber cursor
arbitrarily far ahead; every real message up to that offset is then silently
skipped forever (`records_from(cursor)` returns nothing for offsets < cursor).
`gap_before` only fires when the cursor fell BEHIND `oldest_offset`, never when it
is AHEAD of the frontier — so nothing detects it. Charter-core "delivery is the
product" data-loss, silent.

Fix: reject a claim whose `offset >= next_offset` (you cannot claim work that has
not been posted). Valid claimable offsets are `[0, next_offset)`.

## Acceptance Criteria

### Agent
- [x] `claim_offset` rejects a claim whose `offset >= next_offset` (the topic
      frontier) with a distinct, mapped error — a claim beyond the frontier can
      never succeed. Offsets within `[0, next_offset)` (including swept-away
      offsets) still claim normally.
- [x] The hub RPC claim handler maps the new rejection to a stable JSON-RPC error
      code (sibling to CLAIM_CONFLICT), so a client sees a loud, typed refusal
      rather than silent cursor poison.
- [x] Load-bearing unit test(s): (a) claim beyond frontier is rejected; (b) claim
      within frontier + `release --ack` advances the cursor correctly and does NOT
      poison it; (c) proven load-bearing — removing the frontier guard makes test
      (a) fail (documented via the assertion, or a temp-revert note).
- [x] `cargo test -p termlink-bus` green.

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

cargo test -p termlink-bus claim_beyond_frontier_is_rejected_preventing_cursor_poison 2>&1 | grep -q "1 passed"
cargo test -p termlink-bus claim_on_empty_topic_is_rejected 2>&1 | grep -q "1 passed"
cargo build -p termlink-hub 2>&1 | grep -q "Finished"
grep -q "CLAIM_OFFSET_BEYOND_FRONTIER" crates/termlink-protocol/src/control.rs

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

**Symptom:** A `channel claim <topic> <offset>` at an offset beyond the topic
frontier succeeds; a subsequent `release --ack` advances the claimer's subscriber
cursor to `offset+1`. All genuine records up to that offset are then silently
skipped by every future `subscribe`/resume for that subscriber — unbounded data
loss on the charter-core "durable messages" path, with nothing firing.

**Root cause:** `meta.rs::claim_offset` validated only `UNIQUE(topic,offset)` and
TTL — it never bounded the offset against `next_offset`. Combined with the
monotonic-MAX cursor upsert in `release_claim(ack=true)` (which by design never
rewinds), an out-of-range offset became a permanent forward cursor poison.

**Why structurally allowed:** The claim primitive was designed around exclusivity
(one owner per offset) and lease expiry; "is this a real offset?" was an implicit
assumption never encoded. The gap-detection primitive (`gap_before`, T-1285/T-2463)
only detects a cursor that fell BEHIND `oldest_offset` (retention eviction) — it is
structurally blind to a cursor AHEAD of the frontier, so no existing observability
covered this direction.

**Prevention:** (1) The fix itself — `claim_offset` now rejects `offset >=
next_offset` with a typed, RPC-mapped `CLAIM_OFFSET_BEYOND_FRONTIER` (-32022) LOUD
refusal. (2) Two regression tests proven load-bearing via temp-revert
(`claim_beyond_frontier_is_rejected_preventing_cursor_poison`,
`claim_on_empty_topic_is_rejected`) — deleting the guard makes them fail. (3) PL
captured on the class: any monotonic-never-rewind state advance must validate its
input against the live frontier at the write boundary, because a downstream
gap-detector that only looks one direction cannot recover it.

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

### 2026-08-09T14:45:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2572-bug-future-offset-claimack-poisons-subsc.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-6da1ca1d
- **Timestamp:** 2026-08-09T14:52:33Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 3

**Verification-level findings:**

  1. **l387-sigpipe-risk** (partial, heuristic) @ Verification:line 1
     - evidence: `cargo test -p termlink-bus claim_beyond_frontier_is_rejected_preventing_cursor_poison 2>&1 | grep -q "1 passed"`
  2. **l387-sigpipe-risk** (partial, heuristic) @ Verification:line 2
     - evidence: `cargo test -p termlink-bus claim_on_empty_topic_is_rejected 2>&1 | grep -q "1 passed"`
  3. **l387-sigpipe-risk** (partial, heuristic) @ Verification:line 3
     - evidence: `cargo build -p termlink-hub 2>&1 | grep -q "Finished"`

### 2026-08-09T14:52:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
