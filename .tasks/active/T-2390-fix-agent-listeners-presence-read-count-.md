---
id: T-2390
name: "Fix agent-listeners presence read: count-based seek-to-tail blind to live heartbeats under latest-per-cv-key retention"
description: >
  Fix agent-listeners presence read: count-based seek-to-tail blind to live heartbeats under latest-per-cv-key retention

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
created: 2026-07-10T05:13:42Z
last_update: 2026-07-10T05:13:42Z
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

# T-2390: Fix agent-listeners presence read: count-based seek-to-tail blind to live heartbeats under latest-per-cv-key retention

## Context

`scripts/agent-listeners.sh` reads agent-presence via a T-1844 "seek to tail"
computed as `cursor = channel-info.count - limit`. Under `latest_per_cv_key`
retention (agent-presence, T-2245) `channel info.count` is the RETAINED-message
count (~few hundred/thousand), **decoupled from the monotonic tail offset**
(33k+). So `cursor` lands ~32k offsets below the live heartbeats, clamps to the
oldest retained offset (pinned low by dead-agent cv_keys that are never
superseded), and reads days-stale envelopes. Every presence consumer
(`/peers`, `find-idle`, `agent contact` reachability preflight,
waker-liveness canary) reads the fleet as OFFLINE while heartbeats are healthy.
Empirically: presence is correctly readable only for ~24 min after each daily
04:17 sweep, then dark ~23.5h/day. Fix: read the fresh cv_index via
`channel subscribe --include-current-value` (O(K), correct regardless of sweep
cadence); fall back to the legacy tail-walk only when no cv_index exists.

## Acceptance Criteria

### Agent
- [ ] `agent-listeners.sh` reads presence via `channel subscribe --include-current-value` (extracting `.current_values[].msg`) as the primary path, independent of `channel info.count`
- [ ] Legacy `count`-based tail-walk retained as a FALLBACK only when the cv path yields no envelopes (backward-compat for non-cv-tagged topics)
- [ ] Hub-independent test seam `TERMLINK_LISTENERS_CV_TEST_JSON` (mirror of `TERMLINK_LISTENERS_TEST_JSON`, PL-213) feeds canned `--include-current-value` output; a test asserts 4 agents parse LIVE from a fixture whose live offsets sit far above a small count
- [ ] LIVE proof on this hub: `bash scripts/agent-listeners.sh --no-cache --json` reads the 4 .107 agents LIVE even when agent-presence `count > limit` (i.e. WITHOUT a fresh sweep masking the bug)

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
bash scripts/agent-listeners.sh --help >/dev/null 2>&1
bash tests/agent-listeners-cv-read.sh
bash tests/agent-listeners-identity-fp.sh

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

**Symptom:** All 4 .107 production agents (aef, sonnenstall, workshop-designer,
workflow-designer) read OFFLINE via `/peers` / `agent-listeners` / fleet
presence despite their heartbeat processes being alive and posting every 30s.
Surfaced in T-2389 as "presence stale ~2h despite live heartbeats". Presence
was correctly readable only for ~24 min after the daily 04:17 CEST sweep.

**Root cause:** `agent-listeners.sh` (T-1844 seek-to-tail) computes the subscribe
cursor as `channel-info.count - limit`. Under `latest_per_cv_key` retention
(agent-presence, T-2245/T-2107) `channel info.count` is the RETAINED-message
count (~1400), NOT the monotonic tail offset (~33400). The two diverge without
bound between sweeps. So `cursor = 1400-200 = 1200` clamps to the oldest
retained offset (~30810 — pinned there permanently by dead-agent cv_keys like
`arc004-probe`, whose last heartbeat is never superseded), and the subscribe
returns the oldest 200 envelopes (6–8 days stale). The live heartbeats at
offsets 33400+ are never in the read window → every consumer sees the fleet
OFFLINE. `channel info` exposes no tail-offset field, so the script had nothing
but the (invalid-under-this-retention) `count` to anchor on.

**Why structurally allowed:** (1) T-1844 introduced count-based seek-to-tail
when agent-presence used `forever` retention (count ≈ tail offset then — the
assumption held). When R2 (T-2245) switched agent-presence to
`latest_per_cv_key`, the count/offset invariant silently broke; no test covered
the seek under cv-retention. (2) The topic-growth canary (T-2252) fires only at
count > 5000; this read-breakage triggers at count > ~200, far below the
canary's threshold — blind to it. (3) The daily sweep (T-2345) masked the bug
for ~24 min/day, so intermittent LIVE reads made it look like a flaky heartbeat
rather than a deterministic read bug.

**Prevention:** (a) the fix reads the cv_index directly
(`subscribe --include-current-value`), making correctness independent of both
`count` and sweep cadence — the invariant can't re-break. (b) new
`TERMLINK_LISTENERS_CV_TEST_JSON` fixture test locks the cv-path parse with a
fixture where live offsets sit far above a small count (the exact bug shape).
(c) learning registered so the next count-based seek on a cv-retention topic is
caught in review.

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

### 2026-07-10T05:13:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2390-fix-agent-listeners-presence-read-count-.md
- **Context:** Initial task creation

### 2026-07-10 — RCA + fix + airtight live proof
- **Diagnosis:** the T-2389 "presence stale despite live heartbeats" finding was
  NEVER a frozen heartbeat. All 4 .107 heartbeat loops were alive and posting
  every 30s, landing at fresh offsets (verified: cv-keys index showed aef@33409,
  ts≈now). The read side was blind: `channel info.count`=1402 vs true tail
  offset 33377 under `latest_per_cv_key` retention → `cursor=count-limit` clamped
  to oldest offset 30810 (pinned by dead-agent keys) → read days-stale envelopes.
- **Immediate heal:** `channel sweep agent-presence` pruned 1388 (count 1410→22);
  presence instantly recovered (4 agents LIVE age 6-16s). Confirms RCA.
- **Durable fix:** `agent-listeners.sh` now reads the cv_index via
  `channel subscribe --include-current-value` (primary), legacy count-seek as
  fallback for non-cv topics. Correctness is now independent of sweep cadence.
- **Counterfactual proof (count=82 > limit=5):** old-path `subscribe --cursor 77
  --limit 5` → 5 envelopes all 6-8 days stale → **0 live**; fixed
  `agent-listeners.sh --no-cache --limit 5` → **4 LIVE** (aef/sonnenstall/
  workflow-designer/workshop-designer). Both `tests/agent-listeners-cv-read.sh`
  (new) and `tests/agent-listeners-identity-fp.sh` (regression) PASS.
- **Scope:** repairs presence reads fleet-wide for every script consumer of
  agent-listeners.sh (`/peers`, `agent-listeners-fleet`, waker-liveness canary
  T-2387). Fix is live on .107 immediately. Other hosts (.122/.141) pick it up
  on next re-vendor — follow-up (they're unreachable now).
- **Not changed:** daily sweep cron (T-2345) — with the read fix it's now purely
  bloat control (topic-growth canary T-2252 territory), not a correctness gate.
