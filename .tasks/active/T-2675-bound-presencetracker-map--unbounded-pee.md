---
id: T-2675
name: "Bound PresenceTracker map — unbounded peer-keyed in-memory growth OOM risk"
description: >
  Bound PresenceTracker map — unbounded peer-keyed in-memory growth OOM risk

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
created: 2026-08-13T08:25:33Z
last_update: 2026-08-13T08:25:33Z
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

# T-2675: Bound PresenceTracker map — unbounded peer-keyed in-memory growth OOM risk

## Context

The hub's `PresenceTracker` (`crates/termlink-hub/src/channel.rs`) keeps a
process-global `HashMap<(conversation_id, agent_id), last_seen_ms>` (`static
PRESENCE`, hub-lifetime). It was **insert-only** — `record()` (called on every
`channel.post` carrying `metadata.conversation_id`, channel.rs:874) inserted and
never removed; `snapshot()` only reads. `conversation_id` is an arbitrary
peer-supplied `metadata` string, so a peer looping posts with a fresh random cid
grows the map one entry per post, forever → memory exhaustion of the long-lived
hub daemon (Directive #1 antifragility, #2 reliability). It was the lone
unbounded peer-keyed in-memory map: the siblings cv_index (per-topic cap),
dedupe (TTL+LRU), rate buckets (evict_idle), and remote_store (reaper) are all
already bounded. Found by an adversarial unbounded-growth hunt (round 20 of the
T-2468 charter-review campaign) and verified in code before fixing.

## Acceptance Criteria

### Agent
- [x] `PresenceTracker` gains a TTL prune (opportunistic, on `record`, relative
      to the incoming post's ts — not wall-clock — so it stays deterministic)
      plus a hard-cap backstop that evicts oldest-first, both env-tunable
      (`TERMLINK_PRESENCE_TTL_MS` default 3_600_000, `TERMLINK_PRESENCE_MAX_ENTRIES`
      default 10_000), mirroring the sibling-cap convention.
- [x] A new unit test proves BOTH bounds: a stale entry is pruned by TTL on the
      next record, AND a flood of fresh cids past the cap evicts oldest-first
      down to the cap (constructed via `PresenceTracker::with_bounds` for
      env-independent determinism). Load-bearing proven via temp-revert (test
      fails at the stale-prune assertion when both bounds are stripped).
- [x] The existing `dialog_presence_*` tests still pass unchanged (4/4 green —
      the fix does not regress presence semantics for fresh/recent entries).
- [x] `cargo build -p termlink-hub` succeeds.

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

cargo build -p termlink-hub 2>&1 | tail -3
cargo test -p termlink-hub --lib presence 2>&1 | tail -5
cargo test -p termlink-hub --lib dialog_presence 2>&1 | tail -5

## RCA

**Symptom:** The hub's process-global presence map (`static PRESENCE`,
`HashMap<(conversation_id, agent_id), last_seen_ms>`) grows without bound. A
peer that posts to any topic with a fresh, arbitrary `metadata.conversation_id`
each time adds one permanent map entry per post — the map never shrinks, so
sustained (or malicious) traffic drives the long-lived hub toward OOM.

**Root cause:** `PresenceTracker::record` was insert-only and `snapshot` is
read-only; no method ever removed, capped, aged-out, or swept entries. The key's
`conversation_id` component is fully peer-controlled and unbounded in
cardinality, so map size tracks lifetime distinct-cid count with no ceiling.

**Why structurally allowed:** Presence was added (T-1286/T-243) as a small
"who's active in this dialog?" helper and never given a retention bound, while
every *sibling* peer-keyed in-memory map got one later (cv_index cap T-2103,
dedupe TTL+LRU T-2049, rate buckets evict_idle T-2137, remote_store reaper).
The topic-growth canary (T-2252) and forever-archival canary (T-2562) watch
**on-disk topic record counts**, not **in-memory hub maps**, so nothing observed
this map's size — the blindness persisted from T-1286 to now.

**Prevention:** The fix is itself the prevention — the map is now structurally
incapable of unbounded growth (opportunistic TTL prune on every `record` +
hard-cap backstop that evicts oldest-first). Both are env-tunable, matching the
sibling-cap convention so the pattern is discoverable. A load-bearing unit test
(`presence_tracker_ttl_prune_and_hard_cap`) fails if either bound is removed.
Captured as a learning (audit every long-lived peer-keyed in-memory map against
the negative-trail of already-capped siblings). A general static check for
"unbounded peer-keyed hub map" is a candidate follow-up but out of scope here
(the concrete leak is closed).

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

### 2026-08-13T08:25:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2675-bound-presencetracker-map--unbounded-pee.md
- **Context:** Initial task creation
