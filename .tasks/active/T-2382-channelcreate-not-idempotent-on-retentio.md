---
id: T-2382
name: "channel.create not idempotent on retention — agent contact DM send aborts when dm topic exists with different retention (arc-004 fleet blocker)"
description: >
  agent contact posts a DM by first calling channel.create with Messages(1000); if the dm topic already exists as Forever the hub returns -32603 and the whole send aborts. Blocks arc-004 fleet DM delivery to any peer whose dm topic is Forever. Fix: make the create-before-post idempotent w.r.t. retention.

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
created: 2026-07-09T08:49:11Z
last_update: 2026-07-09T08:49:11Z
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

# T-2382: channel.create not idempotent on retention — agent contact DM send aborts when dm topic exists with different retention (arc-004 fleet blocker)

## Context

Discovered during T-2381 (arc-004 fleet activation). `termlink agent contact`
delivers a DM by ensuring the peer's dm topic exists before posting — it calls
`channel.create` with retention `Messages(1000)`. If the topic ALREADY exists
with a different retention (observed: `Forever`), the hub returns JSON-RPC
`-32603: channel.create: topic "dm:…" already exists with a different retention
policy (existing=Forever, requested=Messages(1000))` and the ENTIRE send aborts —
the DM is never posted. Reproduced twice: T-2379 landed offset 52 on
`dm:9219671e28054458:d1993c2c3ec44c94` (.122) via `agent contact --target-fp`,
then the T-2381 re-send to the same topic (now `Forever`) was refused; had to
route around via a raw `channel post` (landed offset 56). This is a real arc-004
fleet blocker: any peer whose dm topic is `Forever` cannot be reached via
`agent contact`.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] Root cause located: the `agent contact` dm-topic ensure/create call that hard-codes `Messages(1000)` retention, and the hub-side guard that raises "different retention policy". (file:line recorded in RCA)
- [ ] Fix applied so a DM send via `agent contact` to an EXISTING dm topic does NOT abort on retention mismatch — the ensure-create is idempotent (create-if-absent; when the topic already exists, proceed to post regardless of the existing retention rather than erroring). Chosen fix layer (caller-skip-if-exists vs hub-idempotent) recorded in Decisions with rationale.
- [ ] Genuinely-new dm topic path preserved: contacting a peer whose topic does not yet exist still creates it and posts (no regression).
- [ ] `cargo build --release -p termlink` (crate at crates/termlink-cli/ is package `termlink`) succeeds.
- [ ] Regression coverage: a unit/integration test (or a documented manual repro) proves a second `agent contact` to a topic created with a different retention now succeeds instead of -32603.

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

# NOTE: the crate at crates/termlink-cli/ is package `termlink` (not termlink-cli).
cargo test --release -p termlink create_error 2>&1 | grep -q "test result: ok"

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

**Symptom:** `agent contact` / `channel dm` / `/agent-handoff` / `/reply` to a
peer whose dm topic already exists as `Forever` fails with JSON-RPC `-32603:
… already exists with a different retention policy (existing=Forever,
requested=Messages(1000))` and the ENTIRE DM send aborts — the message is never
posted. Observed twice (T-2379 landed offset 52; the T-2381 re-send to the same,
now-`Forever` topic was refused).

**Root cause:** `ensure_topic()` (`crates/termlink-cli/src/commands/channel.rs`)
picks retention from `is_high_rate_pattern`, so every `dm:*` create requests
`Messages(1000)`. When the topic already exists with a DIFFERENT retention
(`Forever` — created before the T-2126 dm-default landed, or via a direct
`channel.create`), the hub's TopicPolicyMismatch guard
(`crates/termlink-bus/src/meta.rs:38-46`) returns -32603. `ensure_topic`'s `Err`
arm propagated ALL errors, aborting the send.

**Why structurally allowed:** comment-code drift — the fn's doc comment PROMISED
"Idempotent — if create returns 'already exists' we treat it as success," but the
`Err` arm never implemented the already-exists special case. The T-2126 change
that made `dm:*` default to `Messages(1000)` silently created the mismatch
condition against pre-existing `Forever` dm topics, and no test covered "ensure
an existing topic whose retention differs from the default." The hub guard is
correct and intentional (protects real retention-policy edits); the defect was
entirely in the caller trusting its own unimplemented doc comment.

**Prevention:** extracted a pure `create_error_is_already_exists` helper + two
unit tests (`create_error_already_exists_matches_retention_mismatch` locks the
-32603 retention-mismatch → treated-as-exists; `…_rejects_genuine_failures`
proves auth/unreachable/capacity errors still surface). The tests pin the
idempotency contract the doc comment states, so a future retention-default change
cannot silently re-break the ensure path.

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

### 2026-07-09 — fix layer: caller-side (agent contact), not hub-side
- **Chose:** Make `agent contact`'s dm-topic ensure-create non-fatal on
  "already exists" (swallow the -32603 retention-mismatch and proceed to post),
  rather than changing the hub's `channel.create` to be globally idempotent on
  retention.
- **Why:** `channel post --ensure-topic` ALREADY establishes this exact
  caller-side precedent — its help says "Auto-create the topic via idempotent
  channel.create before posting … Failure of channel.create is non-fatal; the
  post proceeds." `agent contact` should follow the same contract. The
  send only needs the topic to EXIST; the existing retention is irrelevant to
  delivery. Surgical, one call-site, no change to hub semantics for other
  callers.
- **Rejected:** Hub-side idempotent create — the "different retention policy"
  guard is deliberate (it protects operators from silently reconfiguring a
  topic's retention via a create call). Removing/loosening it globally is a
  larger blast radius and could mask genuine retention-config mistakes
  elsewhere. Keep the guard; fix the one caller that shouldn't be creating in
  the first place.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-07-09T08:49:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2382-channelcreate-not-idempotent-on-retentio.md
- **Context:** Initial task creation
