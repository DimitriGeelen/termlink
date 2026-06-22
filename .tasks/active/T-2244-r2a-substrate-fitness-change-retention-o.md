---
id: T-2244
name: "R2a substrate-fitness: change-retention-on-existing-topic path (enables Q1 interim days:2 on agent-presence)"
description: >
  R2a substrate-fitness: change-retention-on-existing-topic path (enables Q1 interim days:2 on agent-presence)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: ["arc:arc-substrate-fitness"]
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-22T19:50:15Z
last_update: 2026-06-22T19:57:18Z
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

# T-2244: R2a substrate-fitness: change-retention-on-existing-topic path (enables Q1 interim days:2 on agent-presence)

## Context

R2 of arc-substrate-fitness (T-2242 ingestion §2). The plan flagged a missing primitive:
`bus.create_topic` REFUSES to change an existing topic's retention (`meta.rs` → `TopicExists`
error on mismatch), and there is **no path to change retention on an already-created topic**.
`agent-presence` already exists with `retention: forever` (the T-1991 monotonic-growth source);
the Q1 decision wants it on `days:2` NOW as the shipped interim (per-cv_key compaction is the
larger R2b follow-up, blocked on a records-schema migration for cv_key). This task adds that
missing change-retention path end-to-end so an operator can move `agent-presence` (or any topic)
off `forever` without recreating it.

Scope boundary: this is the **change-retention mechanism**, not the per-cv_key compaction mode
(R2b) and not the operational act of applying days:2 to the live production hub (an operator
step — agent does not mutate live shared-host state autonomously).

## Acceptance Criteria

### Agent
- [x] `Bus::set_topic_retention(name, retention)` exists: UPDATEs an existing topic's retention
      policy in the meta DB; returns whether the topic existed (false = no-op, not an error);
      a subsequent `topic_retention(name)` reflects the new policy.
- [x] A hub RPC method exposes set-retention so a remote client can change a topic's retention
      (mirrors `channel.create`'s RPC wiring). [Note: `channel.create` itself has no explicit
      per-method scope gate in the router; set-retention mirrors that exactly — no extra gate.]
- [x] CLI `termlink channel set-retention <topic> --retention <forever|days:N|messages:N|latest>`
      drives the RPC; rejects an unknown/missing topic with a clear error (not a silent create).
      [Live round-trip verified: create forever → set days:2 → list shows kind=days; unknown
      topic → -32602 "use channel.create first", exit 1.]
- [x] After set-retention to `days:N`, a `sweep` of that topic enforces the NEW policy (old
      records beyond N days are trimmed) — proves the change actually takes effect, not just stored.
- [x] `cargo test` passes for the touched crates; existing `create_topic` mismatch-refusal
      behaviour is unchanged (set-retention is the explicit opt-in to change it).

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

# T-2244 (R2a): bus primitive (+ set->sweep enforcement) and hub RPC handler.
cargo test -p termlink-bus --lib set_topic_retention
cargo test -p termlink-hub --lib set_retention

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

### 2026-06-22 — R2 had to be sliced; the plan's R1/R2 sizing was optimistic

- **What changed:** Investigating R2 surfaced that the plan's per-cv_key compaction
  ("compact-to-latest-per-key") needs a **records-schema migration** — `cv_key` is not a
  column on the `records` table (only `offset/byte_pos/length/ts_unix_ms`); the envelope (with
  `metadata.cv_key`) lives in the log-file blob. A latest-per-cv_key sweep therefore can't be a
  pure SQLite op without either adding a `cv_key` column on the hot append path or parsing every
  envelope at sweep time. That is an L-sized build of its own.
- **Plan impact:** R2 splits. **R2a (this task)** = the change-retention-on-existing-topic
  mechanism (the plan flagged "no such path exists"), which independently unlocks the Q1
  **interim days:2** decision. **R2b (follow-up)** = the `LatestPerKey`-per-cv_key retention
  mode + records-schema migration. Also confirmed R1 collapses into R3 (the binary `register`
  self-heartbeat only touches an on-disk file for the directory sweep — it does NOT post to the
  `agent-presence` bus topic, so there is no envelope to add `cv_key` to without first making
  that path post heartbeats — which is the frozen-husk / T-2230 area overlapping R3's Sovereign
  T-2025 revisit).
- **Triggered:** R2b (per-cv_key compaction, needs records migration) — not yet minted, depends
  on no Sovereign answer but is L-sized. Surfaced to the human alongside the evolved arc shape.

### 2026-06-22 — set-retention is storage-only by design

- **What changed:** Considered having `set_retention` sweep immediately. Chose storage-only
  (mirrors how `create_topic` + `sweep` are separate): the RPC changes the policy, the operator
  runs a sweep to enforce it. Keeps the verb single-purpose and avoids a surprise mass-delete
  hidden behind a "set policy" call.
- **Plan impact:** AC4 proves the change *takes effect* via an explicit set→sweep test rather
  than folding the sweep into set.
- **Triggered:** none. (MCP parity for `channel.set_retention` is a deliberate out-of-scope
  follow-up — RPC + CLI cover the operator path; the MCP tool can mirror later.)

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

### 2026-06-22T19:50:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2244-r2a-substrate-fitness-change-retention-o.md
- **Context:** Initial task creation
