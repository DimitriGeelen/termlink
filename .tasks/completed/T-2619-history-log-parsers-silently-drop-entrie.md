---
id: T-2619
name: "history-log parsers silently drop entries with unparseable ts + undercount malformed"
description: >
  parse_find_idle_log + parse_substrate_log drop valid-JSON entries whose ts is unparseable, without counting them as malformed

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/agent_find_idle.rs, crates/termlink-cli/src/commands/substrate.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-11T21:07:00Z
last_update: 2026-08-11T21:41:46Z
date_finished: 2026-08-11T21:41:46Z
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

# T-2619: history-log parsers silently drop entries with unparseable ts + undercount malformed

## Context

`parse_find_idle_log` (crates/termlink-cli/src/commands/agent_find_idle.rs:462) and its
twin `parse_substrate_log` (crates/termlink-cli/src/commands/substrate.rs, ~line 1199)
walk NDJSON audit logs. After confirming a line has valid JSON + string `ts`/`agent_id`/
`kind`, they compute `entry_secs = rfc3339_to_unix_secs_local(ts_str)` and drop entries
older than `cutoff_secs`. But `rfc3339_to_unix_secs_local` returns **0 on ANY parse
failure** ("caller treats 0 as 'very old'", agent_find_idle.rs:530). So a line whose `ts`
value is present but unparseable (e.g. `"not-a-date"`) yields `entry_secs = 0`, which is
`< cutoff_secs` for any positive cutoff, so the entry is silently `continue`'d at
agent_find_idle.rs:517-519 — **without** incrementing `malformed`. The `/find-idle-history`
and `/substrate-history` summaries then report "0 malformed lines skipped" while a corrupt
row vanished. T-2468 Reliability-#2 ("no silent failures; auditable execution") hunt.

Multi-file (both parsers share the shape) → filed rather than built inline. Verified in
code 2026-08-11 (agent_find_idle.rs:482-522, helper at 531).

## Failure scenario

Feed `parse_find_idle_log` a single line `{"ts":"not-a-date","agent_id":"a","kind":"new"}`
with `cutoff_secs = 1_000_000`. Result: `entries` is empty AND `malformed == 0`. A
corrupt-timestamp audit row disappeared and the caller's summary tells the operator the
log was clean. Same for `parse_substrate_log`.

## Acceptance Criteria

### Agent
- [x] A `ts` value that is present but unparseable is classified as **malformed** (counted + skipped), not silently dropped by the cutoff filter — in BOTH `parse_find_idle_log` and `parse_substrate_log`
- [x] Fix distinguishes "ts unparseable" from "ts genuinely old" without a false positive on real old timestamps (do NOT treat a legitimate pre-cutoff entry as malformed). Approach used: both `rfc3339_to_unix_secs_*` helpers now return `Option<i64>` (None ⇒ malformed) instead of the lossy 0-sentinel; a genuinely-parsed old timestamp still age-drops via the cutoff without counting malformed (asserted by the 1970-01-05 case)
- [x] Regression test in each module: valid-JSON-with-required-fields-but-bad-`ts` line ⇒ `entries` empty AND `malformed == 1` (not 0) — `find_idle_history_parse_counts_unparseable_ts_as_malformed` + `parse_substrate_log_counts_unparseable_ts_as_malformed`
- [x] Both tests proven load-bearing via temp-revert (restored `.unwrap_or(0)` swallow → both tests FAILED on `malformed == 1`; restored fix → green)
- [x] Full suites green: `cargo test -p termlink --bins commands::agent_find_idle::tests` (16 passed) and `cargo test -p termlink --bins commands::substrate::tests` (21 passed)

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
cargo test -p termlink --bins commands::agent_find_idle::tests
cargo test -p termlink --bins commands::substrate::tests

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

**Symptom:** `/find-idle-history` / `/substrate-history` report "0 malformed lines skipped"
while a log line with an unparseable `ts` value has silently vanished from the results.

**Root cause:** `rfc3339_to_unix_secs_local` conflates two distinct outcomes into the
single sentinel `0` — "ts failed to parse" and "ts is a genuine ~1970 timestamp". The
caller interprets 0 as "very old" and drops it via the cutoff filter, so a parse failure
is laundered into a silent age-based skip that bypasses the `malformed` counter.

**Why structurally allowed:** The helper's doc comment ("Returns 0 on any parse error
(caller treats 0 as 'very old')") documents the sentinel as intentional, so the swallowing
looked deliberate; the existing tests only exercise non-JSON lines (which ARE counted
malformed), never the valid-JSON/bad-ts case, so the miscount was untested.

**Prevention:** Replace the 0-sentinel with an `Option<i64>` parse variant (None ⇒
malformed classification) in both parsers, plus a regression test per module asserting the
bad-ts line counts as malformed. Load-bearing via temp-revert.

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

### 2026-08-11T21:07:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2619-history-log-parsers-silently-drop-entrie.md
- **Context:** Initial task creation

### 2026-08-11T21:35:21Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-962f3a28
- **Timestamp:** 2026-08-11T21:42:18Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-11T21:41:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
