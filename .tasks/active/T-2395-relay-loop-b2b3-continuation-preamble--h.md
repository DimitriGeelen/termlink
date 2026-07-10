---
id: T-2395
name: "Relay-loop B2+B3: continuation preamble + hop-budget circuit-breaker"
description: >
  Relay-loop B2+B3: continuation preamble + hop-budget circuit-breaker

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
created: 2026-07-10T16:53:02Z
last_update: 2026-07-10T16:53:02Z
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

# T-2395: Relay-loop B2+B3: continuation preamble + hop-budget circuit-breaker

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

T-2393 GO build 2+3/3 (B1 shipped as T-2394). B1 closed the return-leg RAIL
(reply rings the sender). B2+B3 close the *continuation*: a woken agent must
advance to its next real blocker and either reply-on-rail or declare-and-stop —
never idle silently — AND the loop must not ping-pong forever. Design:
`docs/reports/T-2393-poll-free-self-advancing-agent-exchange-inception.md`
(mechanism §, B2/B3). Plumbing verified: both `agent-send.sh` (turn post,
line ~459) and `agent-respond.sh` (reply post, line ~102) already post with
`--metadata conversation_id`, so `relay_hops` threads the same way. The
continuation contract lives in the RECEIVER's own skill (`check-arc.md` Step 6a)
→ framework-owned, non-spoofable (IW-3). Hop cap default 4, env-tunable
(`TERMLINK_RELAY_MAX_HOPS`) — bounded autonomy is the default (IW-1), so we never
trade the nudging problem for a runaway problem.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] **B3 hop plumbing (send):** `agent-send.sh` accepts `--relay-hops N` and, when set (or when the doorbell is rail-augmented = a relay initiation, defaulting to 1), stamps `--metadata relay_hops=<N>` onto the posted turn. The `--dry-run` RESOLVED line surfaces `relay_hops=<N>` so it is assertable without a hub. A non-relay send (custom `--doorbell-text`, no `--relay-hops`) stamps NO relay_hops (back-compat).
- [x] **B3 hop plumbing (reply):** `agent-respond.sh` accepts `--relay-hops N` and stamps `--metadata relay_hops=<N>` onto the `--reply` turn when set (the caller passes incoming+1). Absent the flag, no relay_hops is stamped (back-compat with existing receipt/reply callers).
- [x] **B3 circuit-breaker helper:** new `scripts/relay-hop-check.sh --topic <t> --cid <c>` reads the latest turn on (topic, cid), extracts `relay_hops` metadata, compares to `TERMLINK_RELAY_MAX_HOPS` (default 4). Emits `verdict=continue hops=<N> cap=<M> next_hops=<N+1>` (exit 0) when under cap, or `verdict=stop hops=<N> cap=<M> reason=hop-budget-exhausted` (exit 10) at/over cap. A hub-independent test seam (`TERMLINK_RELAY_HOPCHECK_TEST_JSON=<file>`) feeds canned turn JSON. Missing/absent relay_hops treated as hops=0 (continue).
- [x] **B2 continuation contract + B3 gate (receive):** `.claude/commands/check-arc.md` Step 6a documents the advance-or-declare contract (woken agent: advance to next REAL blocker → reply on this rail via `agent-respond.sh --reply --relay-hops <next_hops>`, OR declare the blocker + STOP; never idle silently) AND instructs running `scripts/relay-hop-check.sh` BEFORE auto-replying: on `verdict=stop`, surface the hop-budget-exhausted blocker LOUDLY and halt (do not reply); on `verdict=continue`, pass `--relay-hops <next_hops>` to the reply.
- [x] **Tests:** `tests/relay-b3-hop-budget.sh` asserts the helper emits `continue`/`next_hops` below cap and `stop`/`hop-budget-exhausted` at cap (via the test seam); `tests/relay-b2-send-hops.sh` (or extend the B1 test) asserts `agent-send.sh --relay-hops 2 --dry-run` surfaces `relay_hops=2` and a bare relay initiation defaults to `relay_hops=1`. `bash -n` clean on all three edited/new scripts.

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
bash -n scripts/agent-send.sh
bash -n scripts/agent-respond.sh
bash -n scripts/relay-hop-check.sh
bash tests/relay-b3-hop-budget.sh
bash tests/relay-b2-send-hops.sh
grep -q 'advance-or-declare\|advance to' .claude/commands/check-arc.md

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

### 2026-07-10T16:53:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2395-relay-loop-b2b3-continuation-preamble--h.md
- **Context:** Initial task creation
