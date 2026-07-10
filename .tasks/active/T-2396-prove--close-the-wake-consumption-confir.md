---
id: T-2396
name: "Prove + close the WAKE consumption-confirmation bypass (G-083)"
description: >
  Prove + close the WAKE consumption-confirmation bypass (G-083)

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
created: 2026-07-10T18:22:07Z
last_update: 2026-07-10T18:22:07Z
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

# T-2396: Prove + close the WAKE consumption-confirmation bypass (G-083)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

G-083 / PL-253. Field evidence (aef↔designer T-175) showed a wake "delivered by
PTY" that silently died — recipient never read it. Code read of
`scripts/agent-send.sh` (lines 499–588) revealed the consumption detector
ALREADY EXISTS: it rings up to `max_rings`, waits `timeout` for a receipt whose
`up_to >= post_offset`, and on no receipt FAILS LOUD (`exit 3`,
"receiver never acked", line 586). A receipt is what `/check-arc respond`
(agent-respond.sh) posts — so a manual-mode session that never runs check-arc
produces no receipt → agent-send already reports FAILED. **Therefore the silent
failure is a BYPASS, not a missing mechanism:** raw `termlink inject` and thread
posts skip agent-send's receipt-wait entirely. This task PROVES that empirically,
then closes the bypass by extracting the receipt-wait into a standalone,
reusable **wake-confirm** verb that any delivery path (including a raw ring or a
thread nudge) can call to get the loud rung-but-not-consumed signal.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] **PROOF (prove-first, per operator):** a documented live demonstration on the local hub that (a) `agent-send.sh` to a target that does NOT consume (no `/check-arc respond` runs) FAILS LOUD (`exit 3`, "receiver never acked"), and (b) a raw `termlink inject` to the same target returns success with NO consumption signal — proving the detector exists in agent-send but is bypassed by raw delivery. Captured in the task Updates / a `docs/reports/T-2396-*.md`.
- [x] **Standalone wake-confirm verb:** new `scripts/wake-confirm.sh --topic <t> --cid <c> --since-offset <N> [--timeout S]` that polls for a receipt acking `>= N` and exits 0 (CONSUMED, prints receipt offset+stage) or a distinct non-zero (NOT-CONSUMED, prints the loud "rung but not consumed — recipient busy/manual-mode; message unread at offset N" diagnosis). Extracted from agent-send's receipt-wait so raw-inject / thread-nudge paths can reuse it. Hub-independent test seam (`TERMLINK_WAKECONFIRM_TEST_JSON`).
- [x] **agent-send reuses it (no behavior change):** `agent-send.sh` delegates its receipt-wait to `wake-confirm.sh` (or shares the helper) so there is ONE consumption-confirmation implementation; its existing DELIVERED/FAILED output + exit codes are preserved (B1/existing tests still green).
- [x] **Guidance:** `.claude/commands/check-arc.md` (or agent-handoff/reply docs) note that a bare `termlink inject` / thread post has NO consumption confirmation — deliver via `agent-send.sh` or follow a raw ring with `wake-confirm.sh` to avoid the silent rung-but-not-consumed class.
- [x] **Tests:** `tests/relay-wake-confirm.sh` asserts CONSUMED (receipt present ≥ offset) vs NOT-CONSUMED (no/old receipt) via the seam; `bash -n` clean on `wake-confirm.sh` + `agent-send.sh`; existing `tests/relay-b1-doorbell-rail.sh` + `tests/relay-b2-send-hops.sh` still pass.

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
bash -n scripts/wake-confirm.sh
bash -n scripts/agent-send.sh
bash tests/relay-wake-confirm.sh
bash tests/relay-b1-doorbell-rail.sh
bash tests/relay-b2-send-hops.sh
test -f docs/reports/T-2396-wake-consumption-confirmation-proof.md
grep -q 'wake-confirm' .claude/commands/check-arc.md

## RCA

**Symptom:** Agent-to-agent comms "stops, does not progress without manual
nudging" (operator, twice). A wake "delivered by PTY" to the aef session sat
unread at thread T-175 offset 21 — durably written, never read, with no signal
that anything went wrong.

**Root cause:** The WAKE layer had no *consumption* confirmation on the paths
people actually use. A heartbeat proves the process is alive, not that the
session is listening (the heartbeat comes from `listener-heartbeat.sh`, decoupled
from the claude session). A raw `termlink inject` / thread post rings/writes but
never waits for a receipt, so a busy or manual-accept recipient — whose injected
text lands UNSUBMITTED and is discarded on `--continue` — fails **silently**.

**Why structurally allowed:** The consumption detector existed only *inside*
`agent-send.sh` (its ring+await-receipt loop → exit 3). It was never extracted,
so every other delivery path (raw inject, thread nudge) bypassed it with no loud
signal. The framework checked reachable-BEFORE (T-2385) but nothing checked
consumed-AFTER for those paths. Blind since the doorbell shipped (T-1800).

**Prevention:** (1) `scripts/wake-confirm.sh` extracts the receipt-wait into a
standalone verb any path can call → the loud rung-but-not-consumed verdict is now
reusable, not locked inside agent-send. (2) `agent-send.sh` delegates to it → one
implementation, no divergence. (3) `check-arc.md` sender's-note documents that
raw inject / thread posts have no confirmation and must use agent-send or
wake-confirm. (4) Registered as **G-083** (concern) + **PL-253** (learning) so
the class is tracked, and `tests/relay-wake-confirm.sh` guards the verdict logic
(incl. the T-1808 stale-receipt guard).

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

### 2026-07-10T18:22:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2396-prove--close-the-wake-consumption-confir.md
- **Context:** Initial task creation
