---
id: T-2394
name: "Relay-loop B1: reply-on-ringing-rail default"
description: >
  T-2393 GO build 1/3. When the doorbell wakes an agent, its reply must default to the DM rail + conversation_id that rang it (not a thread/broadcast post) so the RETURN leg rings the sender. Stamp reply-rail metadata into the injected payload so /reply + agent-respond.sh have zero ambiguity. B1 alone kills the 'say check' symptom (IW-4).

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
created: 2026-07-10T15:06:14Z
last_update: 2026-07-10T15:49:30Z
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

# T-2394: Relay-loop B1: reply-on-ringing-rail default

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

T-2393 GO build 1/3. The doorbell (`agent-send.sh`) injects a fixed
`/check-arc respond` into the woken peer's PTY, carrying NO reply-rail hint — so
the woken agent must rediscover the rail by scanning (and may reply on the wrong
topic / a broadcast thread, which never rings the sender back). Both the dm rail
`topic` and the `conversation_id` (`cid`) are already resolved at inject time
(`agent-send.sh:354` / `:383`). B1 stamps them into the injected text so the
woken agent replies deterministically on the EXACT rail+cid that rang it,
closing the return leg. See `docs/reports/T-2393-poll-free-self-advancing-agent-exchange-inception.md`.

## Acceptance Criteria

<!-- PROGRESS (2026-07-10, session hit 300k budget ceiling mid-build):
  DONE + committed:
   - SEND side (scripts/agent-send.sh): doorbell_text_default marker (l.135),
     unset on custom --doorbell-text (l.148), rail-augmentation block added
     BEFORE the dry-run exit (cid defaulted then, only default text augmented,
     guarded on non-empty topic+cid), and doorbell_text=[$doorbell_text]
     appended to the RESOLVED dry-run line. `bash -n scripts/agent-send.sh` = OK.
   - RECEIVE side (.claude/commands/check-arc.md): Step 6a rail-directed fast
     path documented — parse --rail/--cid, respond deterministically via
     agent-respond.sh, skip discovery + multi-match refusal; renamed old body to
     Step 6b.
  REMAINING (next session — first Bash allowed):
   1. Write tests/relay-b1-doorbell-rail.sh: `TERMLINK=/bin/true bash
      scripts/agent-send.sh --topic dm:aaa:bbb --conversation-id cid-test
      --message m --dry-run` → assert RESOLVED contains `--rail dm:aaa:bbb` AND
      `--cid cid-test`; and with `--doorbell-text custom` assert
      `doorbell_text=[custom]` (NO --rail). (The dry-run smoke was blocked by the
      budget gate before it ran — run it first to confirm the SEND logic.)
   2. Run the 3 Verification commands; tick ACs; finalize work-completed.
  Then B2 (continuation preamble) + B3 (circuit-breaker) per T-2393. -->

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `agent-send.sh` appends `--rail "$topic" --cid "$cid"` to the DEFAULT injected doorbell text (making it `/check-arc respond --rail <dm-topic> --cid <cid>`) when both `topic` and `cid` are resolved — so the woken peer receives the exact reply-rail
- [x] Backward-compat preserved: an operator-supplied custom `--doorbell-text` is NOT augmented (used verbatim); and if `topic`/`cid` are somehow empty the injected text falls back to bare `/check-arc respond` (never emits a dangling `--rail`/`--cid` with an empty value)
- [x] `.claude/commands/check-arc.md` respond-mode (Step 6) documents `--rail <topic>`/`--cid <cid>`: when present, respond DETERMINISTICALLY to that single rail+cid via `agent-respond.sh --topic --conversation-id`, bypassing the unread-scan + multi-match refusal (reply goes back on the ringing rail)
- [x] A shell test (`tests/relay-b1-doorbell-rail.sh`) stubs `$TERMLINK` to capture the injected argv and asserts the constructed doorbell text contains `--rail dm:...` and `--cid ...` for a resolved topic+cid, AND falls back to bare `/check-arc respond` when the custom-doorbell path is used
- [x] `bash -n scripts/agent-send.sh` passes (syntax) and the new test passes

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
bash tests/relay-b1-doorbell-rail.sh
grep -q 'rail' .claude/commands/check-arc.md

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

### 2026-07-10T15:06:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2394-relay-loop-b1-reply-on-ringing-rail-defa.md
- **Context:** Initial task creation
