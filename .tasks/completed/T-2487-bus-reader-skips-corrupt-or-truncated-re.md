---
id: T-2487
name: "bus reader skips corrupt or truncated records instead of walling topic replay (round-16 F1 reader-resilience)"
description: >
  bus reader skips corrupt or truncated records instead of walling topic replay (round-16 F1 reader-resilience)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-bus/src/log.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-02T06:10:28Z
last_update: 2026-08-02T06:15:33Z
date_finished: 2026-08-02T06:15:33Z
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

# T-2487: bus reader skips corrupt or truncated records instead of walling topic replay (round-16 F1 reader-resilience)

## Context

TermLink's #1 charter noun is the durable append-log ("survives a blip and replays").
`crates/termlink-bus/src/log.rs:101-119` (`ReaderIter::next`) returns `Some(Err(..))` on any
per-record read/decode failure, and every consumer propagates that first `Err` with `?`
(`lib.rs:349-351` `envelope_at`; `lib.rs:554-555` `find_idle_agents`, the canonical subscribe
pattern). Net effect: a **single** corrupt or truncated record permanently walls off a topic's
replay for **every** consumer at or after that offset — and re-subscribing re-hits the same wall
forever. Identified HIGH in the round-16 reliability hunt (T-2464); this is the in-authority
"reader-resilience" half. The sibling durability half (fsync-before-index in `append`) stays in
T-2464 as a perf/ADR decision (human-gated) — out of scope here.

## Acceptance Criteria

### Agent
- [x] `ReaderIter::next` skips a poison record (payload read failure OR decode failure) instead of
      returning an aborting `Err` — loops to the next `RecordLoc` so replay continues past the gap.
      (log.rs:101-153 — loop with UnexpectedEof + decode-error skip arms.)
- [x] Each skipped record emits a loud, offset-tagged signal to stderr (`eprintln!`, no new dep) —
      the skip is observable, never silent (directive #2: no silent failures). (log.rs:132, 144.)
- [x] `envelope_at` returns `Ok(None)` (not `Err`) when the single requested offset is a poison
      record that got skipped (offset behaves as swept, not as a topic-brick). Satisfied with zero
      code change: the reader now yields the NEXT offset, so lib.rs:352-354 `Some(Ok(_)) => Ok(None)`
      fires; the `Some(Err(e)) => Err(e)` arm now only sees genuine systemic faults.
- [x] New bus unit test: a log with one truncated/corrupt record in the middle → reader yields the
      records BEFORE and AFTER it, skipping only the poison one. (reader_skips_undecodable_middle_*,
      reader_skips_truncated_tail_*, reader_all_valid_records_unaffected.)
- [x] `cargo test -p termlink-bus --lib` passes (existing + new tests green). (98 passed / 0 failed.)

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
cargo test -p termlink-bus --lib

## RCA

**Symptom:** A single corrupt/truncated record in a topic log makes the entire topic
un-replayable for every consumer at or after that offset — `channel subscribe`, `find-idle`
(cv_index walk), and `envelope_at` all abort. Re-subscribing re-hits the same wall; the topic is
effectively bricked forever from that offset onward.

**Root cause:** `ReaderIter::next` (log.rs:101-119) surfaces any per-record read/decode failure as
`Some(Err(..))`, and every consumer uses `item?` / `match … Some(Err(e)) => Err(e)`, so the first
bad record aborts the whole iteration. The reader had no gap-tolerance: a per-record data fault was
treated as a fatal stream fault.

**Why structurally allowed:** the append path fsyncs the SQLite index but not the payload
(T-2464) — so a crash mid-append can leave the index pointing at bytes that never fully hit disk,
producing exactly the poison record the reader then chokes on. The reader was written for the
happy path (index and log always consistent) with no consideration of the index-ahead-of-payload
window. No test exercised a corrupt record, so the brick behaviour was invisible.

**Prevention:** the new unit test (corrupt record in the middle → reader skips only it) locks the
gap-tolerant contract in place; the loud `eprintln!` per skip makes any real-world corruption
observable in hub stderr rather than a silent empty replay. (The durability root — fsync the
payload before the index commits — is tracked separately in T-2464 as a perf/ADR decision.)

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

### 2026-08-02T06:10:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2487-bus-reader-skips-corrupt-or-truncated-re.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-ccb1d13a
- **Timestamp:** 2026-08-02T06:15:40Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-02T06:15:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
