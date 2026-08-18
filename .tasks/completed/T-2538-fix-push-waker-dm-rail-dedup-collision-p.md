---
id: T-2538
name: "Fix push-waker dm-rail dedup collision (per-topic offsets aliased)"
description: >
  The push-waker dm rail dedups wakes by message_offset alone, but dm offsets are
  per-topic (each dm:self:peer starts at 0), so a second peer's offset-0 wake within
  the TTL is silently dropped as a duplicate — the why-no-response latency class.
  Key dedup on (channel, offset).

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-08T14:55:07Z
last_update: '2026-08-18T18:59:12Z'
date_finished: 2026-08-08T14:57:42Z
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
  - ts: '2026-08-18T18:56:50Z'
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
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:12Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2538: Fix push-waker dm-rail dedup collision (per-topic offsets aliased)

## Context

arc-004's push-waker (`scripts/be-reachable-pushwaker.sh`) has two rails that ring
an idle agent's PTY when a durable message arrives: the **inbox rail** (aggregates
one `inbox:<id>` topic) and the **dm rail** (T-2324, aggregates the single global
`dm.queued` topic, matching `addressee == self_fp`).

**Defect:** the dm rail funnels EVERY `dm:<self>:<peer>` thread through the one
aggregator, but each thread has its OWN offset sequence starting at 0. The rail
dedups wakes with `seen[$offset]` keyed on `message_offset` ALONE
(`be-reachable-pushwaker.sh:183,199-202`). So:

- peerA DMs on `dm:self:peerA` offset 0 → waker rings the PTY.
- Within the TTL window (default 120s), peerB DMs on `dm:self:peerB` offset 0 →
  `pushwaker_dedup_ok` sees `seen[0]` set recently → `continue` → **peerB's wake
  is silently dropped** as a "duplicate," though it is a distinct message from a
  distinct peer on a distinct topic.

Low-traffic dm threads all sit at small offsets (0,1,2…), so collisions are the
COMMON case, not a corner. The design note at lines 173-175 only reasons about
inbox-offset-N vs dm-offset-N (cross-rail) and is blind to dm-topic-vs-dm-topic
WITHIN the one dm rail. The inbox rail is unaffected (single topic → unique offsets).

**Class: LATENCY, not data-loss.** peerB's turn is durably on `dm:self:peerB` and
will be seen on the next `/check-arc` poll — but this rail exists precisely for the
non-live-sender path (raw `channel post` / cron / remote peer / MCP) where nothing
ELSE rings the PTY. For an otherwise-idle agent the wake is delayed indefinitely
until an unrelated message happens to ring it — the exact G-063/G-069 "why is there
still no response?" symptom.

**Fix:** the `dm.queued` emit already carries `"channel": &topic` (channel.rs:856),
as does the inbox emit (channel.rs:212). Key dedup on `(channel, offset)` via a
pure `pushwaker_dedup_key` helper, with bare-offset fallback for channel-less
legacy frames. ~10-line shell change + a load-bearing fixture in
`scripts/test-pushwaker-filter.sh`.

Predecessor: T-2324 (dm rail), T-2316 (pure-helper test harness). Found by an
adversarial push-wake hunter during the T-2468 purpose-review campaign.

## Acceptance Criteria

### Agent
- [x] A pure `pushwaker_dedup_key` helper keys on `(channel, offset)` and falls back to bare offset when the frame carries no `channel`.
- [x] `pushwaker_rail_loop` uses the composite key for its `seen` dedup map (both rails).
- [x] The design note (lines ~173-175) is corrected to reflect per-(channel,offset) dedup.
- [x] A load-bearing fixture in `scripts/test-pushwaker-filter.sh` asserts two distinct dm channels at the same offset get DISTINCT dedup keys (and same channel+offset collide, and a channel-less frame → bare offset key). Removing the channel from the key makes the distinct-channels assertion FAIL (temp-revert proof). Proven: offset-only key → `FAIL: dm-rail collision`, restore → PASS.
- [x] `bash scripts/test-pushwaker-filter.sh` prints `RESULT: PASS`.

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

bash scripts/test-pushwaker-filter.sh 2>&1 | tail -8
bash -n scripts/be-reachable-pushwaker.sh

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

**Symptom:** An idle agent does not get its PTY rung for peerB's DM when peerA
already DMed within the last TTL window and both messages landed at the same
per-topic offset (typically offset 0 on fresh threads). The message is durable
but the wake — the whole point of the non-live-sender rail — never fires.

**Root cause:** the dm rail dedups on `message_offset` alone, but `message_offset`
is per-`dm:<a>:<b>`-topic; the single `dm.queued` aggregator multiplexes many such
topics, so offsets collide across peers. `seen[$offset]` treats peerB@0 as a
duplicate of peerA@0.

**Why structurally allowed:** the design note (be-reachable-pushwaker.sh:173-175)
explicitly reasoned that "an inbox offset N and a dm offset N are distinct messages
on distinct topics, so per-rail dedup is correct and collision-free" — correct for
CROSS-rail, but it never considered that the dm rail itself aggregates MANY topics.
The pure-helper test harness (T-2316) tested `pushwaker_dedup_ok` with scalar
offsets, so the multi-topic aliasing was outside the tested surface.

**Prevention:** the load-bearing fixture asserting distinct-channel/same-offset →
distinct keys fails if dedup ever regresses to offset-only. The dedup key now
derives from the same `channel` field the hub already emits, so key and topic
cannot drift.

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

### 2026-08-08T14:55:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2538-fix-push-waker-dm-rail-dedup-collision-p.md
- **Context:** Initial task creation

### 2026-08-08T14:56:03Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-60f574f3
- **Timestamp:** 2026-08-08T14:57:44Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-08T14:57:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
