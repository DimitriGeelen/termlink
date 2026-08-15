---
id: T-2754
name: "cron+test provers reap the topics they mint (topic-count leak)"
description: >
  cron+test provers reap the topics they mint (topic-count leak)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [scripts/agent-conversation-selftest.sh, scripts/lib/reap-topic.sh, scripts/substrate-smoke.sh, scripts/test-agent-conversation-list.sh, scripts/test-agent-conversation-status.sh, scripts/test-agent-respond.sh, scripts/test-agent-send.sh, scripts/test-agent-send-transport.sh, scripts/test-journal-mirror.sh, tests/reap-topic-fixtures.sh]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-15T22:07:21Z
last_update: 2026-08-15T22:25:02Z
date_finished: 2026-08-15T22:25:02Z
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

# T-2754: cron+test provers reap the topics they mint (topic-count leak)

## Context

The local hub holds **771 topics carrying 13,705 records total** — the signature
of *topic-count* growth rather than record growth. 403 are `Forever`. 398 carry an
embedded unix timestamp and accumulate steadily every month since 2026-04-27; the
newest was minted yesterday.

Two prior tasks addressed the record dimension and stopped there. T-2424 swept 851
debris topics **once, by hand**. T-2426 then closed re-accumulation by auto-picking
`Days(7)` at creation — but only for five named namespaces (`t-*`, `T-*`, `xhub-*`,
`stress-*`, `scratch:*`, `smoke:*`), and only for *retention*. Deleting the topic
registry entry was never in scope.

Today's leakers match none of those five patterns, and one of them runs on a
**daily cron**: `.context/cron/fleet-doorbell-mail-canary.crontab` →
`check-fleet-doorbell-mail-health.sh` → `agent-conversation-selftest.sh:76`, which
mints `agent-conv-selftest-$$-<ts>-$RANDOM` per run with no `trap`, no cleanup, no
delete. The script's own JSON output calls the field `ephemeral_topic` — a promise
the code does not keep. A daily guard permanently polluting the substrate it
monitors, which is the T-2709 shape (a guard generating unclearable debris).

`channel delete` has existed since **T-2421** — registry entry, records, cursors,
claims, on-disk log and cv_index, `--yes`-gated. The capability shipped; these
consumers never migrated. `test-agent-conversation-status.sh:118` still carries the
comment *"No cleanup: termlink has no `channel delete` verb"*, which was true when
written and is now false. Same "hardened in one place, siblings not migrated"
divergence as T-2672/T-2673.

Both growth canaries are structurally blind to this axis: T-2252 (topic-growth)
gates on per-topic record count over four name patterns; T-2562 (forever-archival)
fires at 50,000 records on a `Forever` topic. 771 topics averaging 18 records each
trips neither. Detection is deliberately **out of scope here** — this task fixes the
producers; the blindness is filed separately so each lands as one deliverable.

Scope boundary: cron-driven and test-suite provers reap. `demo-*.sh` and `bench-*.sh`
are deliberately excluded — a human runs those interactively and may want to inspect
the topic afterward.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] A shared reap helper exists (`scripts/lib/reap-topic.sh`) exposing `reap_topic <name> [hub-args...]`, sourced by the consumers rather than duplicated per script. It is **best-effort by contract**: a failed or unavailable delete emits one stderr warning and returns 0, so reaping can never turn a passing prover into a failing one.
- [x] The helper degrades safely on a binary predating T-2421 — probes `channel delete --help` once and skips with a named warning rather than erroring (mirrors `sweep-test-debris.sh:53`).
- [x] `TERMLINK_KEEP_TEST_TOPICS=1` opts out of reaping across every migrated script, so an operator debugging a failed run can retain the topic for inspection.
- [x] `scripts/agent-conversation-selftest.sh` (the daily-cron leaker) reaps its `ephemeral_topic` via `trap` on EXIT, so the topic is removed on every exit path — pass, assertion-fail, setup-fail, and interrupt — not only the success path.
- [x] The remaining in-scope provers reap every topic they mint: `test-agent-conversation-status.sh`, `test-agent-conversation-list.sh` (both topics), `test-agent-respond.sh`, `test-agent-send.sh`, `test-agent-send-transport.sh` (both sites), `test-journal-mirror.sh`, `substrate-smoke.sh`.
- [x] The stale `test-agent-conversation-status.sh:118` comment asserting `channel delete` does not exist is replaced with the actual reap call.
- [x] Reaping preserves the prover's own exit code — the `trap` must not overwrite a non-zero verdict with the delete's status.
- [x] Load-bearing proof: running `agent-conversation-selftest.sh` leaves the hub's topic count unchanged, and the same run with `TERMLINK_KEEP_TEST_TOPICS=1` increases it by exactly 1 (the opt-out is what makes the reap observable).
- [x] A fixture suite `tests/reap-topic-fixtures.sh` covers the helper's contract without a live hub: best-effort-on-failure, missing-verb skip, opt-out honoured, exit-code preservation.

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
# The helper exists and is executable-readable.
test -f scripts/lib/reap-topic.sh
# Fixture suite passes (hermetic — no live hub).
bash tests/reap-topic-fixtures.sh
# Every in-scope prover sources the shared helper rather than duplicating the delete.
bash -c 'for f in agent-conversation-selftest test-agent-conversation-status test-agent-conversation-list test-agent-respond test-agent-send test-agent-send-transport test-journal-mirror substrate-smoke; do grep -q "reap-topic.sh" "scripts/$f.sh" || { echo "MISSING reap helper: $f"; exit 1; }; done'
# Every in-scope prover actually invokes the reap (sourcing alone is not reaping).
bash -c 'for f in agent-conversation-selftest test-agent-conversation-status test-agent-conversation-list test-agent-respond test-agent-send test-agent-send-transport test-journal-mirror substrate-smoke; do grep -q "reap_topic" "scripts/$f.sh" || { echo "MISSING reap call: $f"; exit 1; }; done'
# The stale "no channel delete verb" claim is gone from the tree.
bash -c 'test -z "$(grep -rn "no .channel delete. verb" scripts/ 2>/dev/null)"'
# The cron-driven leaker reaps on EXIT, not just on the success path.
grep -q "trap .*reap" scripts/agent-conversation-selftest.sh
# Opt-out is wired in the helper.
grep -q "TERMLINK_KEEP_TEST_TOPICS" scripts/lib/reap-topic.sh
# The guard layer stays green.
bash scripts/run-guard-layer.sh
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

### 2026-08-15T22:07:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2754-crontest-provers-reap-the-topics-they-mi.md
- **Context:** Initial task creation

### 2026-08-15T22:25:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
