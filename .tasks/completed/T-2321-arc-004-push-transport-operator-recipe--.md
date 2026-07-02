---
id: T-2321
name: "arc-004 push-transport operator recipe — consolidate shipped push-wake path"
description: >
  arc-004 push-transport operator recipe — consolidate shipped push-wake path

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: ["push-transport", "documentation", "operations"]
components: []
related_tasks: [T-2303, T-2316, T-2318, T-2320]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-02T23:38:22Z
last_update: 2026-07-02T23:41:08Z
date_finished: 2026-07-02T23:41:08Z
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

# T-2321: arc-004 push-transport operator recipe — consolidate shipped push-wake path

## Context

The arc-004 `push-transport` arc shipped and closed (`decision: shipped`) across
~12 tasks (T-2303 inception → T-2309/2310 hub WS + client subscribe → T-2313
WS-over-Unix → T-2314 reconnect → T-2315 wake-path GO → T-2316/2317 push-waker →
T-2318 E2E → T-2319 leak fix → T-2320 latency benchmark), each with its own
report. There is **no single operator-facing recipe** tying the shipped pieces
together — an operator asking "how do I get instant push wake, and what happens
when the socket drops?" must read a dozen scattered reports. This task
consolidates them into one master recipe under `docs/operations/`, mirroring the
established `substrate-*-recipe.md` pattern (e.g. T-2124's
`substrate-orchestrator-recipe.md`). **Documentation of what shipped — not a
reopen of the closed arc, no new build.**

## Acceptance Criteria

### Agent
- [x] `docs/operations/push-transport-recipe.md` exists and covers, as an operator
      walkthrough: (a) the mental model (durable substrate underneath, push is a
      faster wake/read TRIGGER only) §1, (b) how to enable push wake
      (`/be-reachable` spawns the registered push-waker) §2, (c) the `channel
      subscribe --push` CLI + its TCP-hub/profile requirement §3, (d) degrade-to-poll
      + active reconnect behaviour on socket drop §4, (e) the measured latency
      (~85–111 ms median vs the ~15 s doorbell floor) §5.
- [x] The recipe includes a **failure-modes / operational-reading** table (§6,
      symptom → meaning → action) covering: waker not spawned, `--push` "no
      hubs.toml profile" error, socket drop → degrade, and no-ring-on-deposit.
- [x] The recipe cross-references the per-task reports (T-2303, T-2314, T-2315,
      T-2316, T-2317, T-2318, T-2320) + the arc registry (§8 map), so it is a
      navigation hub not a replacement — and states the durability invariant
      (§1 + §7: journal / receipts / idempotency / offline-queue unchanged; push
      never becomes source of truth).
- [x] Markdown is well-formed — the `## Verification` link-existence check
      confirms every `docs/reports/T-23*.md` path in the recipe resolves to a
      real file (T-2309/T-2313 referenced by task-ID in prose only, no dead link).

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

test -f docs/operations/push-transport-recipe.md
grep -q "be-reachable" docs/operations/push-transport-recipe.md
grep -q "T-2320" docs/operations/push-transport-recipe.md
# every docs/reports/*.md path referenced in the recipe must exist:
bash -c 'ok=1; for f in $(grep -oE "docs/reports/T-23[0-9]+-[a-z0-9-]+\.md" docs/operations/push-transport-recipe.md | sort -u); do [ -f "$f" ] || { echo "MISSING: $f"; ok=0; }; done; [ "$ok" = 1 ]'

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

### 2026-07-03 — consolidation surfaced the two deferred/human-gated threads
- **What changed:** Writing the map (§8) made explicit that the arc's remaining
  surface is entirely human-gated: (1) webhooks (Candidate B) — deferred at
  T-2303 §10 pending external demand, and NOT an agent-to-agent path; needs a
  human inception. (2) a `dm:<self>:*` direct-push waker — a follow-on that would
  extend push coverage to the dm rail for non-live-sender posters, but carries a
  live-topic-discovery design question (wildcard vs poll-list vs per-topic
  subscribe) that should be an inception, not a blind build.
- **Plan impact:** None to this doc task — both are out of scope here and belong
  to a human GO decision. Recorded so they are not lost as folklore.
- **Triggered:** No sub-task filed autonomously (both need a human inception
  decide, which is sovereignty-gated). Surfaced to the operator instead.

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

### 2026-07-02T23:38:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2321-arc-004-push-transport-operator-recipe--.md
- **Context:** Initial task creation

### 2026-07-02T23:41:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
