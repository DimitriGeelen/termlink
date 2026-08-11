---
id: T-2609
name: "Bare non-actionable unknown-topic errors on charter-core verbs (usability #3)"
description: >
  Usability #3 (actionable errors): post/subscribe/claim/claims/claims_summary return bare 'unknown topic'/'topic not found' with no next step, while sibling sweep/delete handlers already hint 'create it first'. Enrich to name topic + remediation.

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
created: 2026-08-11T15:36:40Z
last_update: 2026-08-11T15:36:40Z
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

# T-2609: Bare non-actionable unknown-topic errors on charter-core verbs (usability #3)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The bare unknown-topic error sites in `crates/termlink-hub/src/channel.rs`
      (post, subscribe ×3 — the immediate + long-poll + streaming `bus.subscribe`
      branches, claim, claims, claims_summary — **7 sites**) each name the offending
      topic AND a concrete next step (create it first / check the name with
      `channel.list`), matching the sibling `channel.sweep` / `channel.delete`
      convention already in the file. No error code changes — message text only.
      (Filing said 6; the build surfaced a 3rd subscribe branch — see Evolution.)
- [x] A load-bearing unit test per charter verb asserts the message now carries the
      actionable hint substring (`channel.create` for post/subscribe/claim,
      `channel.list` for claims/claims_summary). Proven load-bearing by temp-reverting
      the post message to its bare form and confirming the assertion FAILS.
- [x] `cargo test -p termlink-hub` passes (486/486; existing
      `msg.contains("unknown topic")` assertions still hold — post/subscribe messages
      preserve that substring).

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

cargo test -p termlink-hub unknown_topic_error_names_next_step
cargo test -p termlink-hub
# All bare unknown-topic sites are gone (0 = only the test-comment mention remains):
test "$(grep -c 'format!(\"unknown topic: {t}\")' crates/termlink-hub/src/channel.rs)" = "0"
# The actionable hints are present on the charter-core error paths:
grep -q "channel.post: unknown topic" crates/termlink-hub/src/channel.rs
grep -q "channel.subscribe: unknown topic" crates/termlink-hub/src/channel.rs
grep -q "channel.claim: topic .* not found — create it" crates/termlink-hub/src/channel.rs

## RCA

**Symptom:** The four charter-core verbs (post / subscribe / claim /
claims / claims_summary), when handed a topic that does not exist, returned a
dead-end error — `unknown topic: <t>` or `topic <t> not found` — that named the
topic but gave the operator/agent **no next step**. A caller couldn't tell
whether it was a typo, a never-created topic, or an offset problem, and wasn't
told that topics do not auto-create.

**Root cause:** The remediation-hint convention was applied by hand, not
enforced (PL-306: "caller-facing error actionability is a convention, not
enforced"). The sibling handlers `channel.sweep` (`unknown topic '<t>' (use
channel.create first)`) and `channel.delete` (`… (nothing deleted)`) already
carried hints, but the higher-traffic post/subscribe/claim paths were written
bare — an **intra-file inconsistency**, not a considered choice.

**Why structurally allowed:** A bare `format!("unknown topic: {t}")` compiles
and tests clean; the existing unknown-topic tests asserted only the error
*code*, never the message *actionability*. Nothing failed, so the dead-end
messages persisted. Same class as PL-306 / T-2555.

**Prevention:** (a) All 7 sites now follow the sibling convention (topic +
concrete next step); (b) a **load-bearing test per verb** now asserts the
actionable-hint substring (`channel.create` / `channel.list`), proven to fail if
a message is reverted to its bare form — so a future regression to a dead-end
message is caught by the suite, not by a human noticing. Complements the
existing code-only assertions.

## Evolution

### 2026-08-11 — build surfaced a 7th site the hunt's ≤5-cap missed

- **What changed:** The adversarial hunt (capped at 5 findings) flagged 6 sites.
  During the load-bearing temp-revert, an `Edit` collision revealed a **third**
  `subscribe` branch — the streaming `bus.subscribe(&topic, 0)` path (~:1371, a
  different indent level than the immediate/long-poll arms) — carrying the same
  bare `unknown topic: {t}`. It is the same root cause, so it was folded in
  rather than filed separately (one root cause = one task).
- **Plan impact:** AC count 6 → 7 sites; no scope change (still message-text
  only, still `channel.rs`).
- **Triggered:** No new sub-task. Confirms the value of the collision as a
  completeness check — grepping `format!("unknown topic: {t}")` to 0 remaining
  is now a Verification gate so no bare sibling can hide.

## Decisions

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

### 2026-08-11T15:36:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2609-bare-non-actionable-unknown-topic-errors.md
- **Context:** Initial task creation
