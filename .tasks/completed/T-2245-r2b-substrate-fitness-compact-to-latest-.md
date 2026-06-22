---
id: T-2245
name: "R2b substrate-fitness: compact-to-latest-per-cv_key retention (close T-1991 agent-count scaling)"
description: >
  R2b substrate-fitness: compact-to-latest-per-cv_key retention (close T-1991 agent-count scaling)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-substrate-fitness]
arc_id: arc-substrate-fitness
components: [crates/termlink-bus/src/lib.rs, crates/termlink-bus/src/meta.rs, crates/termlink-bus/src/retention.rs, crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs, crates/termlink-hub/src/channel.rs, crates/termlink-hub/src/router.rs, crates/termlink-protocol/src/control.rs]
related_tasks: [T-2242, T-2244, T-2107]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-22T21:05:35Z
last_update: 2026-06-22T21:19:11Z
date_finished: 2026-06-22T21:19:11Z
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

# T-2245: R2b substrate-fitness: compact-to-latest-per-cv_key retention (close T-1991 agent-count scaling)

## Context

R2b of arc-substrate-fitness (arc-002), the second half of the Q1 presence-retention
decision. R2a (T-2244) shipped the change-retention-on-existing-topic path
(`channel.set_retention`), enabling the interim `days:2` on `agent-presence`. R2b ships
the only retention mode that closes the **T-1991 agent-*count* scaling** problem:
**compact-to-latest-per-`cv_key`** — keep only the most-recent record per distinct
`cv_key` on a topic, so record-count converges to *agent-count* (one per agent), not
*heartbeat-count* (one per 30s beat per agent forever).

The existing `Retention` enum has `Forever / Days(N) / Messages(N) / Latest` (keep-1
globally). `Latest` keeps one record for the WHOLE topic; per-`cv_key` compaction keeps
one record per key — the difference between "1 record" and "K records for K agents."

Design ground-truth (plan §1 FLAG 1, T-2107): producers already emit
`metadata.cv_key=$agent_id` on `agent-presence` heartbeats; the hub-side in-memory
`cv_index` maps cv_key→offset. But the bus SQLite `records` table has **no cv_key
column** — cv_key lives only in the envelope blob in the log file. A storage-layer
compaction sweep that groups by cv_key therefore needs cv_key available at the records
layer (schema migration OR blob-parse during sweep — decided in Decisions below).

See `docs/plans/T-2242-substrate-fitness-ingestion.md` §2 (R2) and §0.5 Q1.

## Acceptance Criteria

### Agent
- [x] `Retention` enum gains a `LatestPerCvKey` variant (latest-record-per-cv_key);
      `kind()`/`value()`/`from_parts()` round-trip it (wire kind `latest_per_cv_key`); CLI
      `parse_retention` accepts `latest-per-cv-key` (+`compact-per-key` alias); hub
      `retention_from_json`/`retention_to_json` + `channel.create`/`channel.set_retention`
      persist it. *(retention.rs, channel.rs CLI+hub)*
- [x] cv_key is available to the storage-layer sweep **without a schema migration** (per
      Decisions): the sweep recovers cv_key by reading each record's envelope blob via the
      existing `envelope_at` path — works on records written before this mode existed.
- [x] The retention sweep enforces `LatestPerCvKey` (`Bus::compact_per_cv_key`): for a topic
      under this policy, after a sweep only the highest-offset record per distinct cv_key
      survives; records with no/empty cv_key are **retained** (never silently dropped).
- [x] Existing readers/late-joiners still resolve current presence after compaction
      (the surviving per-key record is the latest; `subscribe`/`envelope_at` still resolve it).
- [x] **Regression test asserts the PROPERTY (PL-213):** post M heartbeats across K distinct
      cv_keys (M ≫ K) → after sweep, exactly K records remain, each the latest offset for its
      key. Asserts exact count `== K` (bus: `compact_per_cv_key_keeps_exactly_one_record_per_key`;
      hub: `sweep_enforces_latest_per_cv_key_and_reports_pruned`).
- [x] **Sweep trigger exists** — `channel.sweep(topic)` RPC + `termlink channel sweep <name>`
      CLI verb. Discovered the retention sweep had ZERO production callers (the whole subsystem,
      incl. R2a's `days:2`, was inert). Generic — enforces any policy; operator/cron-invokable.
- [x] Backward-compatible: no schema change, so pre-existing meta.db opens unchanged; topics
      under Forever/Days/Messages/Latest are unaffected (verified by existing tests).
- [x] All existing bus + hub tests still pass (bus 79/79, hub 364/364) + live CLI smoke.

## Verification

cargo test -p termlink-bus --lib compact_per_cv_key
cargo test -p termlink-hub --lib latest_per_cv_key
cargo test -p termlink-hub --lib sweep_
cargo build -p termlink

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

### 2026-06-22 — R2b did NOT need a schema migration
- **What changed:** The plan (and the prior session's hand-off) assumed per-cv_key
  compaction required adding a `cv_key` column to the `records` table — establishing a
  brand-new ALTER-TABLE migration pattern (none exists in the codebase). The Explore
  map showed cv_key is fully recoverable from the envelope blob in the log file via the
  existing `Bus::envelope_at` path. So the sweep reads each record's envelope and groups
  by `metadata.cv_key` with **no schema change**.
- **Plan impact:** The "needs records-schema migration for cv_key" flag (plan §2 / prior
  hand-off) is dissolved. Blob-parse is strictly better here: (a) zero migration risk /
  backward-compat is free, (b) it works on the EXISTING ~30k stale records — a NULL
  cv_key column would only populate on new posts, so a migration alone could not compact
  the existing backlog, defeating the point. Cost: O(N) positional reads per sweep, paid
  only on-demand (no background thread), exactly when the operator chooses to shrink.
- **Triggered:** No new sub-task; scope stayed within T-2245 and got simpler.

### 2026-06-22 — the retention sweep had ZERO production callers
- **What changed:** `Bus::sweep` (the retention-enforcement entry point) was invoked only
  from unit tests. `channel.create` / `channel.set_retention` (R2a) only PERSIST a policy;
  the control.rs doc even states set_retention is "storage-only — the hub does not sweep."
  Nothing in hub/CLI/MCP ever triggered enforcement. The entire retention subsystem —
  including R2a's just-shipped `days:2` and the pre-existing `Latest` mode — was **inert in
  production**.
- **Plan impact:** R2b's headline value ("close T-1991 agent-count scaling") could not land
  without a trigger. Added a generic `channel.sweep(topic)` RPC + `termlink channel sweep`
  CLI verb (protocol const + hub handler + router arm + capabilities list + CLI variant +
  dispatch + command). It enforces WHATEVER policy is set, so it retroactively activates
  R2a too. Chose an explicit operator/cron verb over sweep-on-post to stay consistent with
  the substrate's "no background thread, explicit verbs" design (T-1155).
- **Triggered:** None minted — `channel.sweep` is the natural home and belongs to this
  deliverable (a retention mode that can't be triggered is not a deliverable). A follow-up
  worth considering: a periodic/cron sweep recipe in docs/operations (noted, not filed).

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

### 2026-06-22 — cv_key source: blob-parse, not a schema column
- **Chose:** Recover cv_key during the sweep by reading each record's envelope blob
  (`Bus::envelope_at`), grouping by `metadata.cv_key`. No `records`-table column.
- **Why:** Zero migration risk; works on existing records (a backfill-less column can't
  compact the existing backlog); self-contained in the bus crate. Sweep is on-demand so
  the O(N) read cost is paid only when shrinking.
- **Rejected:** Add a persisted `cv_key` column + an ALTER-TABLE migration (no existing
  pattern to follow; only helps future posts; more invasive). Reading the hub's in-memory
  `cv_index` (wrong crate; non-durable; cleared on restart).

### 2026-06-22 — trigger: generic `channel.sweep` verb, not sweep-on-post
- **Chose:** A single explicit `channel.sweep(topic)` RPC + CLI verb that enforces whatever
  policy is set. Operator- or cron-invoked.
- **Why:** Matches the substrate's documented design (T-1155: the bus runs no background
  thread; sweeps are explicit). One generic verb activates ALL retention modes (fixes the
  inert subsystem, incl. R2a's days:2), keeps the post hot-path untouched, and is
  cron-wireable.
- **Rejected:** Sweep-on-post (adds per-post cost; implicit; fights the explicit-sweep
  design). A per-cv_key-only trigger (would leave days/messages/latest still inert).

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-06-22T21:05:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2245-r2b-substrate-fitness-compact-to-latest-.md
- **Context:** Initial task creation

### 2026-06-22T21:19:11Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
