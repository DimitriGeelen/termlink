---
id: T-2411
name: "Bind per-agent identity into reachable claude session (doorbell respond gap + shared-host fp leak, T-1693)"
description: >
  Bind per-agent identity into reachable claude session (doorbell respond gap + shared-host fp leak, T-1693)

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
created: 2026-07-12T19:39:08Z
last_update: 2026-07-12T19:42:04Z
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

# T-2411: Bind per-agent identity into reachable claude session (doorbell respond gap + shared-host fp leak, T-1693)

## Context

Discovered while proving the T-2409 whole-fleet doorbell round-trip on .122
(ring20-concierge). The push-wake rail's WAKE half works cross-fleet (transport
+ discovery + PTY doorbell all fire), but the RESPOND half fails: the woken
claude REPL resolves its own identity to the shared **host** key
(`9219671e28054458`) instead of its advertised agent-id fp
(`88743a9ad59fda39`), so `/check-arc respond` on a rail keyed to the agent-id
sees an identity mismatch and safely refuses to post the ack ("onto a rail whose
local party (88743a9a…) isn't this session's identity — I won't do it without
you saying so explicitly"). Same root cause as the peer-flagged identity leak
(rail posts signing as `d1993c2c` host key, not the per-agent fp). This is the
long-open T-1693 / PL-166 / PL-195 shared-host per-agent-identity gap, now
pinned to its exact mechanism and blocking the doorbell respond half.

Proof of root cause (on .122):
`TERMLINK_AGENT_ID=ring20-concierge termlink agent identity --resolve` →
`88743a9ad59fda39` (the rail's party); ambient (no env) → `9219671e28054458`
(host key). The reachable claude process (pid 1782729) has NO `TERMLINK_AGENT_ID`
in its env — `tl-claude build_claude_cmd` passes `--agent-id` to the
heartbeat/waker but never exports it into the claude process itself.

## Acceptance Criteria

### Agent
- [x] `tl-claude.sh` exports `TERMLINK_AGENT_ID=<agent_id>` into the reachable
      claude process in BOTH launch paths (`build_claude_cmd` PTY-string path and
      `cmd_oneshot` exec path), gated on `REACHABLE=1` and a non-empty AGENT_ID.
- [x] `agent-respond.sh` prefers the deterministic env-respecting self-fp source
      (`termlink agent identity --resolve --json | .fingerprint`) when
      `TERMLINK_AGENT_ID` is set, falling back to the existing PL-195
      `channel info agent-presence .senders[0]` scrape when it is not (preserves
      current behavior for non-agent-id sessions — no regression).
- [x] Hermetic test (`tests/tl-claude-identity-binding.sh`) proves: (a) the
      launcher composes `TERMLINK_AGENT_ID` into the claude command when reachable
      + agent-id set and omits it otherwise; (b) `agent-respond.sh` self-fp
      resolver picks the env-respecting path when `TERMLINK_AGENT_ID` is set. All
      cases PASS; `bash -n` clean on both edited scripts.
- [x] **LIVE cross-hub proof (.107 → .122):** deployed the fix to .122
      fleet-scratch, relaunched ring20-concierge through the fixed launcher (claude
      pid 1988328 confirmed carrying `TERMLINK_AGENT_ID=ring20-concierge` in
      `/proc/<pid>/environ`), sent a doorbell from .107, and the concierge posted a
      reply on the SAME rail signed `sender_id=88743a9ad59fda39` (its own agent-id
      fp — NOT the host key `9219671e`), `conversation_id=cid-1783885903-21757`
      (exact match), `in_reply_to="2"`. Payload: "respond-half confirmed …
      Whole-fleet doorbell RESPOND half works." The prior un-bound concierge signed
      the SAME rail as `9219671e` (offset 1) — the before/after is on one rail.

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
bash tests/tl-claude-identity-binding.sh
bash -n scripts/tl-claude.sh
bash -n scripts/agent-respond.sh

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

**Symptom:** A cross-fleet push-wake doorbell (from .107 to .122's
ring20-concierge) correctly wakes the peer's PTY and runs `/check-arc respond`,
but the peer never posts the ack — "woken-but-silent." The woken claude reasons
that the rail's local party (`88743a9a`, its own advertised agent-id fp) "isn't
this session's identity" and refuses to post.

**Root cause:** The reachable claude REPL's termlink identity is decoupled from
the agent-id the waker advertises. `tl-claude build_claude_cmd` / `cmd_oneshot`
launch claude with `env -u PROJECT_ROOT IS_SANDBOX=1 claude …` but never export
`TERMLINK_AGENT_ID`. Inside the REPL, `termlink agent identity --resolve`
therefore falls back to the ambient/shared **host** session key
(`9219671e`), while the be-reachable waker — launched separately WITH
`--agent-id` — resolves the per-agent fp (`88743a9a`). Compounding it,
`agent-respond.sh` resolves self-fp via `channel info agent-presence
.senders[0]`, which on a shared host returns the first/host sender, not the
agent's own fp.

**Why structurally allowed:** T-1693 / PL-166 / PL-195 documented the shared-host
identity ambiguity but the WAKE arc (T-2316/2324/2387) proved presence + doorbell
delivery without ever exercising a real cross-host RESPOND, so the identity
mismatch on the responder side went unhit. No test launched a reachable claude
and asserted its in-REPL identity == its advertised presence fp.

**Prevention:** (1) the hermetic test asserts the launcher threads
`TERMLINK_AGENT_ID` into the claude command and the responder prefers the
env-respecting resolver; (2) the fix makes identity deterministic per agent-id
rather than dependent on shared-host presence ordering, closing the
"regresses on every fresh session" leak the peer flagged.

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

### 2026-07-12T19:39:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2411-bind-per-agent-identity-into-reachable-c.md
- **Context:** Initial task creation

### 2026-07-12 — fix + live cross-hub proof
- **Fix:** `scripts/tl-claude.sh` (both launch paths export `TERMLINK_AGENT_ID`) +
  `scripts/agent-respond.sh` (env-respecting `agent identity --resolve` self-fp
  preferred when `TERMLINK_AGENT_ID` set). Hermetic test
  `tests/tl-claude-identity-binding.sh` 9/9 PASS. Committed 95ad9db8.
- **Live proof:** deployed to .122 fleet-scratch (sha-matched), full teardown of
  the stale concierge (4 duplicate wakers + dead session cleaned), clean relaunch
  via fixed launcher → claude pid 1988328 carries `TERMLINK_AGENT_ID`. Doorbell
  from .107 → concierge posted reply signed `88743a9a` (own agent-id fp) with
  matching cid + in_reply_to. Before/after on ONE rail: offset 1 = un-bound
  concierge signed host key `9219671e`; offset 3 = bound concierge signed
  `88743a9a`. **Whole-fleet doorbell RESPOND half proven end-to-end.**
- **Residual (separate, smaller):** (1) sender receipt-poll window (3 rings ~40s)
  is too short for a COLD-START claude turn (~80s) — the ack lands but after
  agent-send gives up → spurious "woken-but-silent". Tune ring count / add a
  post-ring grace re-poll. (2) claude's own `whoami` still returns the host key
  `9219671e` (session-identity layer) so its self-NARRATION is confused even
  though the WIRE post is correctly signed — full per-agent session keys = T-1693
  deeper scope, cosmetic here. Neither blocks the doorbell.
