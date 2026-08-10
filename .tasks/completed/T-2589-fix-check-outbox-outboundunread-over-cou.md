---
id: T-2589
name: "fix check-outbox outbound_unread over-count (peer receipt makes it always >=1)"
description: >
  check-outbox.sh:234 computes outbound_unread=(count - 1 - peer_acked) where count is the WHOLE dm topic envelope count (both senders posts + both sides receipt envelopes) but peer_acked is the peer max receipt up_to. A receipt acking up_to=N is posted at an offset greater than N, so the peers own latest receipt forces count-1-peer_acked to be at least 1 even when the peer has read every DM you sent. Result: the all-caught-up state is nearly unreachable once a peer acks, and the operator-facing unread=N line (line 328/331) counts the peers own posts/receipts as your unread mail, prompting needless nudges. The header comment (228-233) self-documents it as an approximation but does not acknowledge the always->=1 false positive. Fix: count only self-authored non-receipt envelopes with offset > peer_acked (filter sender_id==self_fp and msg_type != receipt in the per-topic scan) instead of the whole-count subtraction. Semantics: decide what outbound_unread should mean (posts I sent the peer has not acked). From T-2468 verb-2 hunt.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [bug]
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-09T22:38:21Z
last_update: 2026-08-10T19:17:51Z
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

# T-2589: fix check-outbox outbound_unread over-count (peer receipt makes it always >=1)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `scripts/check-outbox.sh` no longer computes `outbound_unread` as `count - 1 - peer_acked` (the whole-topic subtraction that counts the peer's own posts + the peer's own receipt envelope).
- [x] `outbound_unread` is computed by a per-envelope tail scan: subscribe from `cursor = peer_acked + 1` and count only envelopes where `sender_id == self_fp` AND `msg_type != "receipt"` AND `offset > peer_acked` (the semantic "DMs I sent that the peer has not acked").
- [x] The peer's latest-receipt-always->=1 false positive is eliminated: a topic where the peer has acked every self-authored post now reports `outbound_unread = 0` and drops out of the report (all-caught-up reachable).
- [x] A load-bearing fixture test (`tests/check-outbox-fixtures.sh`) drives the whole script through a mock `termlink` (`TERMLINK_BIN`) and asserts the new count. It FAILS against the old `count-1-peer_acked` formula (proven by temp-revert) and PASSES against the fix.
- [x] The header comment (lines ~228-233) is updated to document the exact semantic instead of the "approximation, close enough" note.

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

bash tests/check-outbox-fixtures.sh
bash -c 'out=$(cat scripts/check-outbox.sh); echo "$out" | grep -qF "outbound_unread=\$((count" && exit 1 || exit 0'
bash -n scripts/check-outbox.sh

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

**Symptom:** `/check-outbox` reported `outbound_unread >= 1` for a peer even after
that peer had acked every DM the operator sent them — the "all peers caught up"
state was nearly unreachable once any peer posted a single receipt, prompting
needless `/agent-handoff` nudges.

**Root cause:** `outbound_unread` was `count - 1 - peer_acked`, where `count` is
the WHOLE dm-topic envelope count (both senders' posts + both sides' receipts +
reactions/pins/edits) but `peer_acked` is only the peer's receipt `up_to` offset.
A receipt acking `up_to=N` is itself an envelope stored at an offset **> N**, so
the peer's own latest receipt is always included in `count` and never subtracted —
forcing `count-1-peer_acked >= 1` even when zero self-authored posts are unacked.
The subtraction also miscounts the peer's own posts and both sides' non-content
envelopes as "your unread mail".

**Why structurally allowed:** the header comment (228-233) self-documented the
formula as an "approximation … close enough as a backpressure signal" — the
always->=1 false-positive was accepted as approximation noise rather than
recognized as a correctness bug, and there was no test asserting the caught-up
(`outbound_unread == 0`) state was reachable. An approximation that can never
return 0 is not an approximation of the true value; it is a constant-offset bug.

**Prevention:** replaced the whole-count subtraction with an exact per-envelope
tail scan (`sender_id == self && msg_type != receipt && offset > peer_acked`) AND
added `tests/check-outbox-fixtures.sh` — a mock-`termlink` fixture that asserts a
fully-acked topic yields `outbound_unread = 0` and a partially-acked topic yields
the exact self-authored count. The test fails against the old formula (temp-revert
proven), so the caught-up-is-reachable invariant is now load-bearing.

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

### 2026-08-09T22:38:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2589-fix-check-outbox-outboundunread-over-cou.md
- **Context:** Initial task creation

### 2026-08-10T19:17:51Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
