---
id: T-2500
name: "create_topic fabricates Retention::Forever on corrupt stored policy — silent"
description: >
  create_topic fabricates Retention::Forever on corrupt stored policy — silent

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-bus/src/error.rs, crates/termlink-bus/src/lib.rs, crates/termlink-bus/src/meta.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-02T16:34:19Z
last_update: 2026-08-02T16:37:54Z
date_finished: 2026-08-02T16:37:54Z
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

# T-2500: create_topic fabricates Retention::Forever on corrupt stored policy — silent

## Context

`Meta::create_topic` (crates/termlink-bus/src/meta.rs:39) reads an existing
topic's stored `(retention_kind, retention_value)` and parses it with
`Retention::from_parts(&kind, value).unwrap_or(Retention::Forever)`. When the
stored row cannot parse — an unrecognized `kind` (schema drift, a cross-version
fleet hub reading a `latest_per_cv_key` row it predates, or a corrupt/mangled
string) or an out-of-range `value` (e.g. a negative `retention_value`) —
`from_parts` returns `None` and the `unwrap_or` **silently fabricates
`Retention::Forever`**. That fabricated value then drives the idempotency guard:
if the caller requests any non-Forever policy the guard returns
`TopicPolicyMismatch { existing: Forever, .. }` (a phantom policy that was never
stored → operator misdiagnosis); if the caller requests `Forever` the guard
returns `Ok(false)`, silently accepting the corrupt topic as a valid Forever
topic — a topic meant to be bounded (`Messages(1000)`) now grows unbounded
(the T-1991 bloat class the retention system exists to prevent).

The sibling reader `topic_retention` (meta.rs:79) already handles the identical
`from_parts` call correctly — it returns `Ok(None)` on parse failure rather than
fabricating `Forever`. `create_topic` is the odd one out. This is the no-silent-
failures campaign (directive #2 Reliability) — sibling of T-2497/T-2498/T-2499.

## Acceptance Criteria

### Agent
- [x] `create_topic` no longer fabricates `Retention::Forever` on parse failure of an existing row; a `None` from `from_parts` on an existing topic surfaces a loud `BusError` naming the topic + the unparseable kind/value. (meta.rs:39 explicit match → `BusError::CorruptRetention`)
- [x] New `BusError` variant added (`CorruptRetention`) with a `thiserror` message including topic name, stored kind string, and stored value. (error.rs)
- [x] Unit test proves an existing topic row with an unrecognized `retention_kind` yields the loud error (NOT `Ok(false)` and NOT a phantom `TopicPolicyMismatch { existing: Forever }`). (`create_topic_rejects_corrupt_stored_kind`)
- [x] Unit test proves an existing topic row with an out-of-range value (negative `days`/`messages`) yields the loud error. (`create_topic_rejects_corrupt_stored_value`)
- [x] Valid-row idempotent re-create path is unchanged (existing `create_topic` / `TopicPolicyMismatch` tests stay green). (5/5 create_topic tests pass)
- [x] `cargo test -p termlink-bus --lib` passes. (100 passed; 0 failed)

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

out=$(cargo test -p termlink-bus --lib 2>&1); echo "$out" | grep -q "test result: ok"
out=$(cargo test -p termlink-bus --lib create_topic 2>&1); echo "$out" | grep -q "test result: ok"

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

**Symptom:** A topic whose stored retention row cannot be parsed (unknown
`retention_kind`, or out-of-range `retention_value`) is silently treated by
`create_topic` as `Retention::Forever`. Two downstream manifestations: (a) a
re-create with the correct policy returns `TopicPolicyMismatch` reporting a
phantom `existing: Forever` that was never stored — the operator debugs the
wrong policy; (b) a re-create requesting `Forever` returns `Ok(false)`,
swallowing the corruption entirely — a topic meant to be bounded now behaves
as Forever and grows without bound (T-1991 bloat class), silently.

**Root cause:** `Retention::from_parts(&kind, value).unwrap_or(Retention::Forever)`
at meta.rs:39 collapses the `None` (parse-failure) case into a fabricated
`Forever` value. `None` from `from_parts` means "the stored bytes do not
represent any known retention policy" — a corruption/version signal — but
`unwrap_or` discards that signal and manufactures a specific, plausible-looking
policy. The fabricated value is indistinguishable from a genuinely-stored
`Forever` to every caller downstream.

**Why structurally allowed:** `unwrap_or(default)` on a parse result is a common
idiom that reads as harmless, but here the `Option` boundary IS the data-integrity
boundary — the same boundary the sibling reader `topic_retention` (meta.rs:79)
respects by returning `Ok(None)`. The two readers of the same table diverged:
one propagates the parse-failure, the other papers over it. No test exercised
`create_topic` against a corrupt/unrecognized stored row, so the divergence was
invisible.

**Prevention:** (1) The fix replaces `unwrap_or(Forever)` with an explicit match
that returns a loud `BusError::CorruptRetention` on `None` — a corrupt stored
policy can no longer masquerade as a valid one. (2) Two unit tests seed a
topic row with (a) an unrecognized kind and (b) an out-of-range value and assert
the loud error, so a future revert to `unwrap_or` breaks a test. (3) Learning
captures the "`Option` at a data-integrity boundary must not be `unwrap_or`'d
into a fabricated value" rule (sibling of PL-284/PL-285).

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

### 2026-08-02T16:34:19Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2500-createtopic-fabricates-retentionforever-.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-9b9e7a47
- **Timestamp:** 2026-08-02T16:38:01Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-02T16:37:54Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
