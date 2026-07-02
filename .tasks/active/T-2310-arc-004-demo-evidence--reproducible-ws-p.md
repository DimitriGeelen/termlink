---
id: T-2310
name: "arc-004 demo evidence — reproducible WS push sub-second + degrade-to-poll proof"
description: >
  arc-004 demo evidence — reproducible WS push sub-second + degrade-to-poll proof

status: started-work
workflow_type: test
owner: agent
horizon: now
tags: ["arc:push-transport"]
components: []
related_tasks: ["T-2303", "T-2305", "T-2306", "T-2307", "T-2308", "T-2309"]
arc_id: push-transport
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-02T18:26:30Z
last_update: 2026-07-02T18:26:30Z
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

# T-2310: arc-004 demo evidence — reproducible WS push sub-second + degrade-to-poll proof

## Context

arc-004 `push-transport` build is complete end-to-end (hub S1–S4: T-2305/06/07/08,
plus live CLI consumer S3b: T-2309). The arc registry field `demo_evidence` is still
`null`, which is the last artifact the human needs before `fw arc close` (sovereignty-gated).
This task produces a **reproducible** demonstration of the arc headline mechanic — a live
agent receiving a DM the instant it is posted via a hub→client WebSocket push (sub-second),
and cleanly degrading to polling when the socket drops — captured as a committed artifact so
the close is evidence-backed rather than folklore. Prior live smoke (T-2309) proved the path
once by hand; this task turns that into a scripted, re-runnable proof against an isolated hub.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Reproducible demo script `scripts/demo-ws-push.sh` exists, is executable, and is self-contained: starts an isolated hub under a temp `TERMLINK_RUNTIME_DIR`, attaches a `channel subscribe inbox.queued --push` consumer, posts a DM to an `inbox:*` topic, and tears the hub down on exit (no touch of the shared :9100 hub).
- [x] Running the demo captures a WS push frame carrying the durable `message_offset` for the posted DM, and the measured post→push latency is sub-second (< 1000 ms) — the arc headline mechanic. **Evidence:** 91/99/93 ms across 3 runs; frame `{"...","message_offset":0,...}`.
- [x] The demo also exercises degrade-to-poll: after the WS path ends/drops, the consumer falls back to the existing poll loop (captured in the artifact as the observed transition or the verified contract). **Evidence:** `[push] WS unavailable (…) — degrading to poll` observed on hub stop.
- [x] Demo evidence artifact `docs/reports/T-2310-arc-004-ws-push-demo.md` records the exact commands, the captured push frame, the measured latency number, and the degrade-to-poll behaviour.
- [x] arc-004 registry `demo_evidence` field references the artifact (no longer `null`).

<!-- All criteria agent-verifiable; no Human section. -->

<!-- HUMAN-SECTION-REMOVED
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
HUMAN-SECTION-REMOVED -->

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

test -x scripts/demo-ws-push.sh
test -f docs/reports/T-2310-arc-004-ws-push-demo.md
out=$(cat docs/reports/T-2310-arc-004-ws-push-demo.md); echo "$out" | grep -q "message_offset"
ev=$(grep '^demo_evidence:' .context/arcs/push-transport.yaml); echo "$ev" | grep -q "T-2310"

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

### 2026-07-02 — demo evidence turns the S3b hand-smoke into a re-runnable proof

- **What changed:** The T-2309 live smoke proved the push path once by hand against an
  ad-hoc :9199 hub. For the arc close to be evidence-backed (not folklore), the proof
  must be reproducible. This slice scripts it: isolated temp-runtime hub, `--push
  inbox.queued` consumer, timed `inbox:*` post, captured push frame + measured latency.
- **Plan impact:** None to the build — S1–S4 + S3b are unchanged. This is a
  verification/evidence deliverable that populates the arc's `demo_evidence` field,
  the last non-gated artifact before the human's `fw arc close`.
- **Triggered:** No new build sub-tasks. Confirms the documented follow-ons (WS-over-Unix,
  active reconnect-with-backoff) remain out-of-scope for arc-004's GO(scoped) surface.

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

### 2026-07-02T18:26:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2310-arc-004-demo-evidence--reproducible-ws-p.md
- **Context:** Initial task creation
