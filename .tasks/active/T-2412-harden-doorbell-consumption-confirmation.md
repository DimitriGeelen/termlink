---
id: T-2412
name: "Harden doorbell consumption-confirmation — count cid-matched reply (in_reply_to) as CONSUMED + grace poll for cold-start peers"
description: >
  Harden doorbell consumption-confirmation — count cid-matched reply (in_reply_to) as CONSUMED + grace poll for cold-start peers

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
created: 2026-07-12T21:04:31Z
last_update: 2026-07-12T21:09:51Z
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

# T-2412: Harden doorbell consumption-confirmation — count cid-matched reply (in_reply_to) as CONSUMED + grace poll for cold-start peers

## Context

Surfaced by the T-2409/T-2411 whole-fleet doorbell proof on .122. The
consumption-confirmation matcher (`wake-confirm.sh::receipt_from_json`, mirrored
inline in `agent-send.sh`) counts ONLY `msg_type=="receipt"` with
`up_to >= since_offset`. But a fresh/non-framework responder confirms consumption
by posting a `msg_type=note` REPLY (via `/agent-handoff` or agent-respond's reply
path) carrying `conversation_id==cid` + `in_reply_to==<posted-offset>` WITHOUT the
separate receipt envelope — so a genuinely-answered doorbell reports a false
"woken-but-silent" (observed live: concierge reply at offset 3, in_reply_to=2, no
receipt). Two compounding causes: (1) the receipt-only filter misses the reply;
(2) the poll window (3 rings ~40s) is too short for a COLD-START claude turn
(~82s observed), so even a receipt would land after the sender gives up. Matching
on `in_reply_to` is identity-agnostic — it fixes the shared-host .107 case too
(peer reply signs as host key, but our own post carries no in_reply_to, so no
self-match).

## Acceptance Criteria

### Agent
- [x] `wake-confirm.sh` matcher counts a cid-matched REPLY as CONSUMED: an
      envelope with `msg_type in {note,chat}` AND
      `metadata.in_reply_to == since_offset`, in addition to the existing
      `msg_type=receipt` + `up_to >= since_offset` path. Backward compatible (the
      receipt path is unchanged; T-1808 stale-receipt guard preserved).
- [x] `agent-send.sh` inline consumption-wait mirrors the same broadened matcher
      (or delegates to wake-confirm.sh) so the primary ring loop benefits, not
      just the standalone verb. (Delegates to wake-confirm.sh at line ~570 — it
      inherits the fix automatically.)
- [x] A post-ring GRACE poll: after the final ring, `agent-send.sh` keeps polling
      for confirmation for an extended window (default ≥60s, tunable via env) so a
      cold-start / slow peer whose reply lands after the ring cadence is caught
      before declaring woken-but-silent. (`AGENT_SEND_GRACE_SECS`, default 60.)
- [x] Hermetic test (`tests/wake-confirm-reply-match.sh`) via the
      `TERMLINK_WAKECONFIRM_TEST_JSON` seam proves: a note-reply with matching
      in_reply_to → CONSUMED; a receipt (up_to>=so) → CONSUMED (unchanged); an
      original post (no in_reply_to) → NOT self-matched; unrelated traffic →
      NOT CONSUMED. `bash -n` clean on both edited scripts. (12/12 PASS.)
- [x] **LIVE real-data proof:** ran the fixed `wake-confirm.sh` against the actual
      .122 concierge reply that the OLD matcher missed (topic
      `dm:88743a9a:d1993c2c`, cid `cid-1783885903-21757`, since-offset 2) →
      `{"consumed":true,"receipt_offset":3,"kind":"reply"}`. The exact
      woken-but-silent case now correctly reports CONSUMED.

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
bash tests/wake-confirm-reply-match.sh
bash -n scripts/wake-confirm.sh
bash -n scripts/agent-send.sh

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

### 2026-07-12T21:04:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2412-harden-doorbell-consumption-confirmation.md
- **Context:** Initial task creation
