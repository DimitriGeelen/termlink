---
id: T-2413
name: "wake-confirm misses msg_type=turn replies — false woken-but-silent on the doorbell rail"
description: >
  wake-confirm misses msg_type=turn replies — false woken-but-silent on the doorbell rail

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-17T11:01:33Z
last_update: 2026-07-17T11:05:28Z
date_finished: 2026-07-17T11:05:28Z
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

# T-2413: wake-confirm misses msg_type=turn replies — false woken-but-silent on the doorbell rail

## Context

T-2412 broadened the doorbell consumption-confirmation matcher (`scripts/wake-confirm.sh`)
to count a cid-matched REPLY as CONSUMED alongside the canonical `msg_type=receipt`.
But it scoped the reply set to `note` and `chat` only — and the MOST canonical reply
type on the rail is `msg_type=turn`. `agent-send.sh --await-reply` itself documents
the peer's answer as "first msg_type=turn with offset > the posted turn", so the very
type the send path expects back is the one the confirm path refuses to recognise.

Found live (T-2409 fleet completion, 2026-07-17): a real doorbell to peer `aef` on
rail `dm:0e7ee6cad65137fc:d1993c2c3ec44c94` was genuinely answered — `aef` posted
offset=2, `msg_type=turn`, `metadata.in_reply_to="1"`, signed with its OWN agent fp
(`0e7ee6cad65137fc`, proving T-2411's identity binding is live on the shared host).
`agent-send` nonetheless reported `FAILED — receiver never acked` and escalated a
false woken-but-silent to `.woken-but-silent-canary.log`. Re-running `wake-confirm.sh`
against the captured real rail JSON reproduces `consumed:false` deterministically.

This is the exact failure class T-2412 existed to close, left half-closed. It matters
fleet-wide: a false "silent peer" is what trains agents (and operators) to stop
trusting the rail and fall back to passive waiting.

## Acceptance Criteria

### Agent
- [x] `wake-confirm.sh` counts `msg_type=turn` with `metadata.in_reply_to == since_offset` as CONSUMED
- [x] The `receipt` path (`up_to >= since_offset`) and the T-1808 stale-receipt guard are both preserved
- [x] Our own posted turn (no `in_reply_to`) still does NOT self-match, including when sender and recipient share a host key
- [x] CONSUMED output reports `kind="reply"` for a turn-reply (not `kind="receipt"`)
- [x] Hermetic test covers the turn-reply case + real-rail regression fixture; `tests/wake-confirm-reply-match.sh` passes
- [x] Replaying the REAL captured aef rail (offset 2, turn, in_reply_to=1) through `wake-confirm.sh` yields `consumed:true`

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

bash tests/wake-confirm-reply-match.sh
bash -n scripts/wake-confirm.sh
bash -n scripts/agent-send.sh

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

**Symptom:** A doorbell that the peer genuinely answered is reported
`FAILED — receiver never acked` (exit 3) by `agent-send.sh`, and a false
woken-but-silent is escalated to `.woken-but-silent-canary.log`. Observed live
2026-07-17 on rail `dm:0e7ee6cad65137fc:d1993c2c3ec44c94`: peer `aef` replied at
offset=2 (`msg_type=turn`, `in_reply_to="1"`) and the sender still declared silence.

**Root cause:** `receipt_from_json` in `scripts/wake-confirm.sh` (T-2412) matches the
reply class as `msg_type == "note" or msg_type == "chat"`. It omits `msg_type=turn` —
the canonical reply type that `agent-send.sh --await-reply` itself defines as the
peer's answer ("first msg_type=turn with offset > the posted turn"). Send and confirm
therefore disagree about what a reply IS: the send path expects `turn`, the confirm
path refuses to see it.

**Why structurally allowed:** T-2412's test suite was built from the .122 concierge
incident, whose responder posted `msg_type=note`. Every fixture encoded that one
observed shape, so the suite passed while the most common shape on the rail went
untested. The confirm-side reply taxonomy was never checked against the send-side
`--await-reply` definition — no test asserted the two agreed. Hermetic fixtures
mirrored a single field observation rather than the protocol's own contract.

**Prevention:** (1) The hermetic suite now pins the reply class explicitly per
msg_type — `turn`, `note`, `chat` each asserted CONSUMED — so dropping any one of
them fails the suite rather than silently narrowing the rail. (2) A real-rail
regression fixture (`tests/fixtures/aef-turn-reply.json`, captured from the live
2026-07-17 incident) is replayed on every run, so the exact envelope that fooled the
matcher can never be un-fixed. (3) The negative case (own post, no `in_reply_to`) is
asserted for `turn` too, keeping the anti-self-match guarantee true for the newly
matched type on a shared host where sender and recipient can sign identically.

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

### 2026-07-17T11:01:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2413-wake-confirm-misses-msgtypeturn-replies-.md
- **Context:** Initial task creation

### 2026-07-17 — fixed + live-proven on the real rail

- **Found by:** driving a REAL doorbell round-trip to shared-host peer `aef` during
  T-2409 whole-fleet completion. `aef` answered correctly (offset 2, `msg_type=turn`,
  `in_reply_to="1"`, signed with its OWN fp `0e7ee6cad65137fc`) and `agent-send` still
  reported `FAILED — receiver never acked` + escalated a false woken-but-silent.
- **Fix:** reply class in `scripts/wake-confirm.sh` is now the explicit
  `REPLY_MSG_TYPES='["note","chat","turn"]'`, and `kind` is derived from the receipt
  case rather than enumerated (enumerating is what dropped `turn` in T-2412).
- **Second bug caught mid-fix:** the first jq attempt used `$rt | index(.msg_type)`,
  which pipes the ARRAY into `index` so `.` no longer refers to the envelope —
  jq raised `Cannot index array with string "msg_type"`, and `receipt_from_json`'s
  `2>/dev/null || true` SWALLOWED it, silently breaking note/chat too. The suite
  caught it immediately (6 FAIL). Corrected to an `as`-binding:
  `((.msg_type // "") as $mt | $rt | index($mt)) != null`. Lesson: the error-swallowing
  in `receipt_from_json` turns any jq syntax error into a silent "not consumed" —
  the suite is the only thing standing between that and another false-silent class.
- **Live proof (real hub, not fixture):**
  `wake-confirm.sh --topic dm:0e7ee6cad65137fc:d1993c2c3ec44c94 --since-offset 1`
  → before: `{"consumed":false,...,"reason":"rung-but-not-consumed"}` exit 3
  → after:  `{"consumed":true,"receipt_offset":2,"kind":"reply"}` exit 0
- **Regression pinned:** `tests/fixtures/aef-turn-reply.json` (real 2026-07-17 envelope
  shape, payloads redacted) replayed on every suite run. Suite: 19/19 ALL PASS.
- **Fleet context:** this same run independently confirmed T-2411's identity binding is
  LIVE on the shared host — `aef` signed 7 posts with its own fp `0e7ee6ca` and ZERO
  with the host key `d1993c2c`. The .107 identity collapse is cured in the field.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-59b7e024
- **Timestamp:** 2026-07-17T11:05:30Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-07-17T11:05:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
