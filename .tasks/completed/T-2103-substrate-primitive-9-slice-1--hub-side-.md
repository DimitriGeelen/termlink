---
id: T-2103
name: "substrate primitive 9 slice 1 — hub-side cv_index module + channel.post wiring (T-2027/T-2089 GO)"
description: >
  substrate primitive 9 slice 1 — hub-side cv_index module + channel.post wiring (T-2027/T-2089 GO)

status: work-completed
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
created: 2026-06-09T20:26:48Z
last_update: 2026-06-09T20:26:48Z
date_finished: 2026-06-09T20:35:57Z
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

# T-2103: substrate primitive 9 slice 1 — hub-side cv_index module + channel.post wiring (T-2027/T-2089 GO)

## Context

Substrate primitive #9 (broadcast-with-replay / current-value key) is the last
operationally-active gap in the §6 build manifest of the parallel-execution-substrate
ADR (T-2018). T-2089 inception decided GO on Design A (tagged-post current-value via
optional `metadata.cv_key`) — see `docs/reports/T-2089-broadcast-with-replay-inception.md`.

The build plan's slice 1 — the foundational hub-side index — has not yet shipped.
This task ships ONLY that piece:

1. New module `crates/termlink-hub/src/cv_index.rs` mirroring the dedupe.rs pattern
   (T-2049): OnceLock global, in-memory `HashMap<topic, HashMap<cv_key, offset>>`,
   `init()` called at hub startup.
2. Wire into `handle_channel_post_with` (channel.rs) after the successful-post path
   — when `env.metadata` contains a `cv_key` field, record `(topic, cv_key, offset)`
   into the index with last-write-wins semantics (matching agent-presence latest-
   heartbeat-per-agent_id model per IW-2).
3. LOUD-refuse symmetry with T-2049 dedupe — cap distinct cv_keys per topic at
   1000 (matches T-2049 A2 assumption). On overflow: refuse insert with an
   internal warning + governor counter increment, NOT a post error. The post
   itself stays atomic; only the cv_index annotation drops.
4. Inline unit tests (matching dedupe.rs structure): empty index, single key,
   key update (last-write-wins), multi-topic isolation, cap-overflow refusal.

**Out of scope for slice 1 (deferred):**
- `channel.subscribe --include-current-value` wire (slice 2)
- CLI/MCP surface (slice 3)
- `channel cv-keys` inspection verb (slice 4)
- Heartbeat producer integration (slice 5)
- Eager hub-startup rebuild — for slice 1 the index is built incrementally as posts
  arrive. Eager rebuild is wired in slice 2 when the read side becomes user-visible
  (no point paying startup cost if no consumer yet reads).

This slice is backward compatible (no API change, no metadata field becomes required,
old posts without `cv_key` skip the index, old subscribers see no behavior change).

## Acceptance Criteria

### Agent
- [x] `crates/termlink-hub/src/cv_index.rs` exists with module-doc explaining T-2027/T-2089 lineage and last-write-wins semantics
- [x] Module exposes `pub fn init()` mirroring dedupe.rs pattern (OnceLock-installed global)
- [x] Module exposes `pub fn record(topic, cv_key, offset)` — append-or-update with last-write-wins
- [x] Module exposes `pub fn current_values(topic) -> Vec<(String, u64)>` for slice 2 to read
- [x] Module exposes `pub fn entries_active() -> usize` for governor surface (T-2048 sibling)
- [x] Per-topic distinct cv_key cap enforced at 1000 with `TERMLINK_CV_INDEX_CAP_PER_TOPIC` env override
- [x] `cv_index::init()` called from hub startup (server.rs) alongside `dedupe::init()`
- [x] `handle_channel_post_with` records cv_key after successful post when metadata.cv_key is present
- [x] Inline unit tests cover: empty, single key, key-update last-write-wins, multi-topic isolation, cap-overflow refusal (8/8 passing)
- [x] `cargo check -p termlink-hub` passes
- [x] No regressions in existing hub tests — 341/341 passing (8 new cv_index + 333 existing)

## Verification

test -f crates/termlink-hub/src/cv_index.rs
grep -q "T-2027\|T-2089" crates/termlink-hub/src/cv_index.rs
grep -q "pub fn init" crates/termlink-hub/src/cv_index.rs
grep -q "pub fn record" crates/termlink-hub/src/cv_index.rs
grep -q "pub fn current_values" crates/termlink-hub/src/cv_index.rs
grep -q "pub fn entries_active" crates/termlink-hub/src/cv_index.rs
grep -q "TERMLINK_CV_INDEX_CAP_PER_TOPIC" crates/termlink-hub/src/cv_index.rs
grep -q "mod cv_index" crates/termlink-hub/src/lib.rs
grep -q "cv_index" crates/termlink-hub/src/channel.rs
grep -q "cv_index::init" crates/termlink-hub/src/server.rs
out=$(cargo check -p termlink-hub 2>&1); echo "$out" | grep -qE "Finished"
out=$(cargo test -p termlink-hub cv_index 2>&1); echo "$out" | grep -q "8 passed"

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

### 2026-06-09T20:26:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2103-substrate-primitive-9-slice-1--hub-side-.md
- **Context:** Initial task creation
