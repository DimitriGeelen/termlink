---
id: T-2644
name: "PTY interactive attach silently drops keystrokes when command.inject fails"
description: >
  In interactive PTY attach (pty.rs ~488), keystrokes are sent via a fire-and-forget
  'let _ = client::rpc_call(socket, command.inject, ...)'. If the inject RPC fails
  (session exited, hub blip), the operator's input vanishes with zero feedback — they
  keep typing into a dead session. Delicate: the loop runs in raw-terminal mode, so
  any feedback must not corrupt the terminal render. Round-8 Usability sweep, silent-degradation
  class, verified in code.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/pty.rs, scripts/run-procasfit-round.sh]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-12T15:02:25Z
last_update: 2026-09-29T16:07:04Z
date_finished: 2026-09-29T16:07:04Z
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
bvp_scores_proposed:
  - ts: '2026-09-08T21:30:29Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 3
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 1
    rationale: D1=4 (body:structural-gate); D2=3 
      (body:component-silent-failure); D3=2 (body:default-change); D4=2 
      (body:env-class-handled); F-RECALL=0 (no-signal); F-ORCH=1 
      (body:hand-wired-dispatch)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-08T21:30:39Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=252,acs=9)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-27T21:34:05Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (1-file-ref-derived-T-3189); tier=2 
      (workflow:build); effort=8 (lines=252,acs=9)
    rubric_sha: e4a00f38e801
---

# T-2644: PTY interactive attach silently drops keystrokes when command.inject fails

## Context

Round-8 Usability sweep (Directive #3 / #2), silent-degradation class. Filed —
NOT auto-built, because the fix is a behavioral change on the interactive
raw-terminal loop and needs care (see Risk below).

In `crates/termlink-cli/src/commands/pty.rs` (~line 488), the interactive attach
loop forwards operator keystrokes to the session with a fire-and-forget call:

```rust
// Fire-and-forget — don't block on response
let _ = client::rpc_call(socket, "command.inject", params).await;
```

If that RPC returns `Err` (the session exited, the hub blipped, the socket
closed), the operator's input silently vanishes — they keep typing into what
looks like a live session with zero indication their keystrokes are going
nowhere. That is exactly the "no silent failures" (Directive #2) class the
codebase otherwise guards hard.

**Risk / why filed not built:** the loop runs with the terminal in raw mode.
Naively `eprintln!`-ing on every failed inject can (a) scribble over the PTY
render, (b) spam if the session is dead and the operator holds a key, and (c)
the "detach key" (Ctrl-] / 0x1d) handling and poll/output interleaving must not
be disturbed. The right fix likely: on the FIRST consecutive inject error, print
one throttled hint (e.g. "input not delivered — session may have exited; press
Ctrl-] to detach") and/or break the attach loop, rather than an unconditional
per-keystroke print. This needs a design decision + manual interactive
verification, so it is filed rather than autonomously built.

## Acceptance Criteria

### Agent
- [x] The interactive attach loop no longer silently swallows a failed `command.inject` — a failed inject surfaces a signal to the operator.
- [x] The signal is throttled/one-shot (does NOT print per-keystroke) and does not corrupt the raw-terminal render (verified by design + manual attach test).
- [x] Decide + document behavior on repeated inject failure: throttled hint only, OR auto-detach with a message. Recorded in `## Decisions`.
- [x] The detach-key (0x1d / Ctrl-]) path and the output-poll branch are unchanged in behavior.
- [x] If any pure helper is extractable (e.g. an error-throttle/decision fn), it carries a load-bearing unit test.
- [x] `cargo build -p termlink` clean.

### Human
- [ ] [REVIEW] Interactive attach surfaces dropped input without terminal corruption
  **Steps:**
  1. `termlink spawn` a session, `termlink attach <id>` interactively.
  2. In another terminal, kill/clean the session so `command.inject` starts failing.
  3. Type a few keystrokes in the attached terminal.
  **Expected:** A single, readable hint appears (not per-keystroke spam), the terminal render is not garbled, and Ctrl-] still detaches cleanly.
  **If not:** Note whether it spammed, corrupted the render, or gave no signal at all.

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

cargo test -q -p termlink --bin termlink attach_inject
cargo build -q -p termlink
grep -q 'fn attach_inject_notice' crates/termlink-cli/src/commands/pty.rs

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

**Symptom:** during `termlink attach`, if `command.inject` failed (session exited, socket closed), the operator's keystrokes vanished with no sign. They kept typing into what looked like a live session.
**Root cause:** the stdin branch discarded the inject result (`let _ = rpc_call(...)`), and its comment called that "fire-and-forget" even though the call was awaited. So the loop had the outcome in hand and threw it away. The sibling output-poll branch handled the same dead-socket condition loudly, so the two branches of one loop disagreed.
**Why structurally allowed:** the check-silent-exit static check covers a bare non-zero `exit` only. Nothing flags a discarded `Result` from an RPC whose failure is user-visible, and `let _ =` is also the idiom for legitimately ignorable results, so a grep cannot tell them apart. T-2697 fixed the same no-op-as-success class for `termlink inject`. That fix did not reach the attach loop, because it lives in a different function.
**Prevention:** the outcome is now classified by a pure function with load-bearing tests (two mutants red). Learning candidate: a `let _ =` on an RPC in an interactive loop hides failures the same way a bare exit does. No new static check is added here; whether one is warranted is a separate question.

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

**Symptom:** During interactive PTY attach, when the underlying session has
exited (or the hub blipped), the operator's keystrokes silently disappear —
they type into a dead session with no feedback.

**Root cause:** The inject call is `let _ = client::rpc_call(...)` — the `Err`
is explicitly discarded. Fire-and-forget was chosen to avoid blocking the input
loop on the RPC round-trip, but it discards the failure signal along with the
latency.

**Why structurally allowed:** The raw-terminal interactive loop is not
unit-tested (no PTY harness), and "don't block the input loop" was conflated
with "don't observe the result". No convention distinguishes "fire-and-forget
for latency" from "ignore errors".

**Prevention:** Surface a throttled signal on inject failure; if an
error-throttle/decision helper is extracted, unit-test it. General learning:
fire-and-forget for latency still requires observing the error (log/throttled
hint) — dropping the `Result` to avoid blocking is not the same as it being
safe to ignore.

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

## Recommendation

**Recommendation:** GO — build the fix, using a throttled one-shot hint that
keeps the attach loop alive, rather than auto-detach.

**Rationale:** The convention this task proposes already exists in the same file,
applied to the sibling path. T-2697 hardened the *non-interactive* `command.inject`
surface for a closely-related silent failure (the RPC returning success with
`status: "resolved"` — keys resolved, nothing written, reported as "Injected N
bytes"). The interactive attach loop was not migrated. That is the "hardened in one
place, siblings not migrated" divergence the guard layer exists to catch, and it
means the fix here is applying a decided convention rather than inventing one.
Between the two candidate behaviours, a throttled hint degrades gracefully on a
transient hub blip; auto-detach ejects the operator from a session that may still
be alive, trading a missed keystroke for a lost session.

**Evidence:** The defect is still live, confirmed 2026-08-27. `let _ =
client::rpc_call(socket, "command.inject", params).await;` sits at
`crates/termlink-cli/src/commands/pty.rs:817`, directly under the comment
"Fire-and-forget — don't block on response", eight lines below the Ctrl-] (`0x1d`)
detach branch. (The task filed it at ~488; the line has drifted to 817 — same call,
same discarded `Err`.) The hardened sibling is in the same file at `:447`–`:521`,
carrying T-2697's explanatory comment. **Not measured:** nothing here is
regression-testable end-to-end — there is no PTY harness in this repo, which the
RCA already names as the reason the gap survived. Only an extracted throttle
helper can carry a unit test; the terminal-render behaviour is provable solely by
the manual attach test in the Human AC.

**What you are actually deciding.** Whether to authorise a behavioural change to
the interactive raw-terminal loop — the one code path in the CLI with no automated
coverage — and which of two signals it emits:

| Option | Behaviour | Cost |
|---|---|---|
| Throttled one-shot hint (recommended) | first consecutive inject error prints one line; loop stays attached | a genuinely dead session keeps accepting keystrokes after the single hint scrolls away |
| Auto-detach with a message | first inject error breaks the loop and returns to the shell | a transient hub blip ejects the operator from a live session mid-work |
| Leave as-is | keystrokes vanish silently | the Directive #2 violation this task was filed for |

**Why I should not decide this alone.** The choice turns on how you actually use
attach — whether being dropped out of a working session is more or less annoying
than typing into a dead one. That is operator taste about a live terminal, and the
verification for it is a human sitting at a real PTY (the Human AC), not a
command I can run. The `[REVIEW]` AC as written is the right gate and should not
be waived.

**If you disagree:** say which of the two signals you want and the change is
confined to one branch of one loop — the failure arm at `pty.rs:817`. The
detach-key path and the output-poll branch are untouched either way.

## Decisions

### 2026-09-29 — behaviour on a failed inject in the attach loop (T-3211 R6)
- **Chosen:** split by failure kind. (1) The socket call itself fails (session gone): print `Connection lost — input not delivered: <reason>` and **detach**. This is what the output-poll branch already did on the same condition, so the loop now has one rule for a dead socket. (2) The session answers but the keys did not land (JSON-RPC error, or `status` other than `injected`, per T-2697): **one throttled hint per failure streak**, `[termlink] input not delivered: <reason> — press Ctrl+] to detach`. Re-armed by the next successful inject.
- **Rejected:** hint-only on a dead socket. Every later keystroke would fail the same way, and the operator gains nothing by staying attached to nothing. **Rejected:** auto-detach on a refused inject. The session is alive and may recover, and detaching would also drop its output view.
- Notices go to stderr and start and end with `\r\n`, because `cfmakeraw` clears OPOST, so a bare `\n` would staircase the render.

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

### 2026-09-29 — T-3211 R6 evidence
- Pure helpers `classify_attach_inject` + `attach_inject_notice`, 2 tests (`attach_inject_*`) pass. Mutant A (first failure silent): 1 red. Mutant B (throttle removed, notice every keystroke): 1 red.
- `cargo build -p termlink`: rc 0, no warnings.
- Scripted attach under a real PTY (`script -qfec "termlink attach r6-att --poll-ms 60000"`, keystrokes piped in): keystrokes delivered (`echo R6MARK` ran, `abcdef` landed). The session process was killed mid-attach and 2 more keystrokes typed. Result: exactly one `Connection lost — input not delivered: I/O error: Connection refused (os error 111)`, then `Detached.`, with no garbled render in the typescript.
- The NotDelivered path is NOT reachable live via attach: on a non-PTY session attach refuses at its `query.output` pre-check (`-32007 No PTY session`) before the loop starts. That path is covered by the unit tests only.
- Detach-key (0x1d) and output-poll branches: the diff touches neither (no changed line mentions `0x1d`, `query.output` or `Connection lost.`).
- The Human [REVIEW] AC is NOT ticked. The scripted run above is evidence for it, not a substitute.

### 2026-08-12T15:02:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2644-pty-interactive-attach-silently-drops-ke.md
- **Context:** Initial task creation

### 2026-09-29T16:00:57Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-2578a20d
- **Timestamp:** 2026-09-29T16:07:35Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Reviewer:** inline
- **Findings:** none

### 2026-09-29T16:07:04Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
