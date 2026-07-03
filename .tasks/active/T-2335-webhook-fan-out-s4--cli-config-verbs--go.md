---
id: T-2335
name: "Webhook fan-out S4 — governor_status telemetry (RPC + MCP + hub status --governor)"
description: >
  Observability slice of the T-2331 GO webhook feature. Slices 1-3 shipped: signed+allowlisted dispatch primitive (T-2332), event->dispatch fan-out (T-2333), retry/backoff/dead-letter with observability counters (T-2334). S4 surfaces the T-2334 RetryQueue counters to operators via the established governor-telemetry channel: wire webhook_enabled + webhook_target_count + the RetryQueue counters (depth, enqueued_total, retry_success_total, dropped_full_total, dead_letter_total) into hub.governor_status JSON-RPC, following the exact cv_index telemetry pattern (T-2110/T-2119). termlink_hub_governor_status MCP is pass-through (clones the RPC result verbatim) so it inherits the fields for free; 'hub status --governor' local CLI render (render_governor_section in infrastructure.rs) gets an explicit webhook row. RetryQueue accessors already exist on webhook::retry_queue(); adds one pub fn target_count() to webhook.rs. CLI config verbs ('termlink webhook add/list/test') were DECOMPOSED into follow-up T-2336 (one task = one deliverable). See docs/reports/T-2331-webhooks-external-fan-out-inception.md and crates/termlink-hub/src/webhook.rs.

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
created: 2026-07-03T13:48:46Z
last_update: 2026-07-03T13:55:20Z
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

# T-2335: Webhook fan-out S4 — CLI config verbs + governor_status counters (arc-004, follows T-2334)

## Context

Final observability slice of the webhook fan-out feature (arc-004 push-transport,
T-2331 GO). Surfaces the T-2334 in-memory RetryQueue state to operators through
the same `hub.governor_status` channel every other substrate primitive uses
(cv_index T-2110, dedupe T-2049, governor T-2048). CLI config verbs split to T-2336.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `webhook::target_count() -> usize` added — returns configured target count (0 when disabled)
- [x] `handle_hub_governor_status` (router.rs) emits `webhook_enabled` (bool), `webhook_target_count`, `webhook_retry_depth`, `webhook_enqueued_total`, `webhook_retry_success_total`, `webhook_dropped_full_total`, `webhook_dead_letter_total`
- [x] router.rs governor_status test asserts the 7 new `webhook_*` fields are present in the response
- [x] `render_governor_section` (infrastructure.rs) prints a `webhook:` row sourced from the new fields (n/a for pre-slice hubs, consistent with cv_index render)
- [x] `cargo build -p termlink-hub -p termlink` succeeds (66s, clean)
- [x] full `cargo test -p termlink-hub` suite run: 396 pass + 1 load-flaky (T-2258 10s-bound stress test, passes in isolation — 82s; slice touches no channel/TLS path). Slice tests deterministic-green: `governor_status_exposes_cv_index_counters` + both `render_governor_section` tests.

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

grep -q "pub fn target_count" crates/termlink-hub/src/webhook.rs
grep -q "webhook_dead_letter_total" crates/termlink-hub/src/router.rs
grep -q "webhook_retry_depth" crates/termlink-hub/src/router.rs
grep -q "webhook: enabled=" crates/termlink-cli/src/commands/infrastructure.rs
cargo build -p termlink-hub -p termlink 2>&1 | tail -1
# Deterministic targeted tests (the slice touches no TLS/reqwest code, so PL-238's
# "full crate suite" discipline was satisfied manually; the full suite carries a
# load-sensitive 10s-bound stress test T-2258 that can flake under gate parallelism).
cargo test -p termlink-hub governor_status_exposes 2>&1 | tail -3
cargo test -p termlink render_governor_section 2>&1 | tail -3

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

## Decisions

- **Decomposed the captured S4 into telemetry (this task) + CLI config verbs (T-2336).**
  The original capture bundled two independent deliverables. "One task = one
  deliverable" — telemetry is low-risk, pattern-following, fully testable; CLI
  verbs involve config-file read/merge/write + a live dispatch test (more surface,
  more risk). Splitting lets the higher-value observability half land cleanly.
- **Wired telemetry through `hub.governor_status`, not a bespoke `webhook.status` RPC.**
  Every other substrate primitive (cv_index T-2110, dedupe T-2049, governor T-2048)
  surfaces its counters here; a new RPC would fragment the operator's mental model
  and miss the fleet-rollup + MCP pass-through for free.
- **Relied on the MCP tool's pass-through shape** (`termlink_hub_governor_status`
  clones the RPC `result` verbatim + inserts `ok:true`), so the 7 new fields reach
  MCP consumers with zero MCP-side edits. Only the local `hub status --governor`
  render (explicit `g()` field list) needed a row.
- **`webhook_enabled` (bool) + `webhook_target_count` (u64) both emitted.**
  `is_enabled()` requires ≥1 target by construction, so the pair disambiguates
  "config path unset" (enabled=false) from a loaded-but-inspectable config — more
  informative to an operator than either field alone.

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

### 2026-07-03T13:48:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2335-webhook-fan-out-s4--cli-config-verbs--go.md
- **Context:** Initial task creation

### 2026-07-03T13:55:20Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
