---
id: T-2644
name: "PTY interactive attach silently drops keystrokes when command.inject fails"
description: >
  In interactive PTY attach (pty.rs ~488), keystrokes are sent via a fire-and-forget 'let _ = client::rpc_call(socket, command.inject, ...)'. If the inject RPC fails (session exited, hub blip), the operator's input vanishes with zero feedback — they keep typing into a dead session. Delicate: the loop runs in raw-terminal mode, so any feedback must not corrupt the terminal render. Round-8 Usability sweep, silent-degradation class, verified in code.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/pty.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-12T15:02:25Z
last_update: 2026-08-16T23:22:49Z
date_finished: 2026-08-16T23:22:49Z
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
- [x] The interactive attach loop no longer silently swallows a failed `command.inject` — a failed inject surfaces a signal to the operator. — **DONE: `let _ = rpc_call(...)` replaced by classify-then-report. Covers BOTH the transport `Err` the filing named AND the `status:"resolved"` no-PTY case it did not (see RCA amendment — that second one was never surfaced by anything).**
- [x] The signal is throttled/one-shot (does NOT print per-keystroke) and does not corrupt the raw-terminal render (verified by design + manual attach test). — **DONE (agent half): throttled via `should_warn_inject_failure` — announces at failure 1, then every 25th; suppressed at 2..24, pinned by `does_not_warn_on_every_keystroke`. Render safety by design: the hint leads AND trails with `\r\n` (raw mode has no implicit CR). The sibling handlers lead only, which they can afford because they `break` immediately; this one continues the loop so it returns the cursor itself. Terminal-render half is the Human AC below — not agent-verifiable without a PTY harness.**
- [x] Decide + document behavior on repeated inject failure: throttled hint only, OR auto-detach with a message. Recorded in `## Decisions`. — **DONE: throttled hint, NOT auto-detach. Rationale and the rejected alternatives (incl. why auto-detach is wrong here despite the data-plane loop doing it) recorded in `## Decisions`.**
- [x] The detach-key (0x1d / Ctrl-]) path and the output-poll branch are unchanged in behavior. — **DONE: both branches untouched. The detach check still precedes the inject and returns early, so a Ctrl+] never reaches the new code; the poll branch's `Err(_) => "Connection lost." + break` is byte-identical. The change is confined to the inject arm.**
- [x] If any pure helper is extractable (e.g. an error-throttle/decision fn), it carries a load-bearing unit test. — **DONE: two pure helpers, 8 tests. Load-bearing PROVEN by mutation, not assumed — removing the status check fails 3 tests; making the hint unconditional fails 2. Both mutations run and reverted.**
- [x] `cargo build -p termlink` clean. — **DONE: builds clean; `cargo test -p termlink` 1137+3+174 pass, 0 failed.**

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

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# Pipefail/SIGPIPE hint (L-387, CORRECTED by T-2775 — this task predates the fix
# and carried the superseded advice below it). P-011 runs each command under
# `set -eo pipefail`. NEVER write `cmd | grep -q PATTERN`: it exits 141 (SIGPIPE)
# when grep matches and closes stdin while the upstream is still writing — the
# gate then fails BECAUSE the check succeeded.
#
# The capture-then-pipe form this task's template originally prescribed —
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"     # UNSAFE above ~64KB
# is size-dependent, NOT safe: `echo` is a producer like any other, so once $out
# exceeds the pipe buffer it is still writing when grep -q exits. Measured rc=141
# at 3M lines. Use one of these instead, both measured rc=0 at the same size:
#     out=$(cmd 2>&1 || true); grep -q "PATTERN" <<< "$out"   # herestring (preferred)
#     test -n "$(cmd | grep -m1 PATTERN)"                     # pipeline inside $( )
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

# Run the suite ONCE into a file, then assert against it — four separate
# `cargo test` invocations would be four full runs for the same evidence.
# `cargo test` builds, so a separate `cargo build` line is redundant.
cargo test -p termlink > .context/working/.t2644-test.out 2>&1
# The whole CLI suite is green (the 8 new tests are inside this count).
grep -q "test result: ok. 1137 passed" .context/working/.t2644-test.out
# The named load-bearing tests PASS — asserting the name is present in the source
# would let a test that was renamed away, or is failing, still read as coverage.
grep -q 'no_pty_resolved_is_a_failure_even_though_the_rpc_succeeded ... ok' .context/working/.t2644-test.out
grep -q 'does_not_warn_on_every_keystroke ... ok' .context/working/.t2644-test.out
grep -q 'unclassifiable_envelope_fails_closed ... ok' .context/working/.t2644-test.out
grep -q 'transport_error_is_a_failure_and_carries_its_message ... ok' .context/working/.t2644-test.out
# The fire-and-forget is GONE from the attach loop — the defect itself, asserted
# directly rather than only via the tests covering its replacement.
#
# COMMENT LINES ARE STRIPPED FIRST, and that is load-bearing, not tidiness: the fix
# QUOTES the defect verbatim in two comments (the `// T-2644: this was ...` note at
# the call site, and the test-module header describing what regressed). A naive grep
# matches those and the assertion fails on a tree that is correct — verified: it
# returned rc=1 against the fixed code. Same self-referential trap as T-2776, where
# an assertion matched the task's own prose. An assertion about code must read code.
out=$(grep -v '^[[:space:]]*//' crates/termlink-cli/src/commands/pty.rs); ! grep -qF 'let _ = client::rpc_call(socket, "command.inject", params).await;' <<< "$out"
# Both pure helpers exist.
grep -qF 'fn should_warn_inject_failure' crates/termlink-cli/src/commands/pty.rs
grep -qF 'fn attach_inject_failure_reason' crates/termlink-cli/src/commands/pty.rs
# The status check is DELEGATED to the T-2697 helper, not re-implemented — a second
# copy of the fail-closed rule is exactly how this bug reached a third call site.
out=$(sed -n '/fn attach_inject_failure_reason/,/^}/p' crates/termlink-cli/src/commands/pty.rs); grep -q 'inject_status_is_injected' <<< "$out"
# The guard layer stays clean (no new alloc/drain/silent-exit/busy-spin candidates).
out=$(bash scripts/run-guard-layer.sh 2>&1 || true); grep -q "guard layer: PASS" <<< "$out"

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

**AMENDMENT (2026-08-17, on building the fix) — the filed symptom was the
milder half of the bug.** Reading the loop found a SECOND failure mode the
filing did not describe, and it is strictly worse:

The `command.inject` handler answers two different things with the same RPC
success (T-2697): `status:"injected"` (a PTY took the bytes) and
`status:"resolved"` (no PTY — keys resolved and written NOWHERE, plus a `note`
naming the remedy). The attach loop checked neither.

That splits the defect in two, with opposite detectability:

| mode | RPC | already surfaced? |
|---|---|---|
| transport error (session exited / socket gone) | `Err` | **eventually yes** — the sibling output-poll branch in the same `select!` prints `"Connection lost."` and breaks within one `poll_ms`. What was lost is only the keystrokes typed inside that window. |
| `status:"resolved"` (no PTY) | **`Ok`** | **never.** `query.output` keeps succeeding, so the poll branch stays perfectly happy. The operator types into a live-looking session indefinitely and nothing ever says otherwise. |

So the filed case was time-bounded by an accident of the sibling branch; the
unfiled one is unbounded and completely invisible. Fixing only the discarded
`Err` — the literal thing the filing named — would have left the worse half
untouched while looking complete.

**Why structurally allowed (revised):** this is the "hardened in one place,
siblings not migrated" divergence this codebase keeps producing. T-2580 fixed
the status-blindness on the MCP surface and its comment stated the rule
outright; T-2697 migrated it to `termlink inject`; **neither reached the attach
loop**. The pure helper `inject_status_is_injected` was sitting 350 lines above
the defect, already written and already tested. The gap was not knowledge — it
was that nothing enumerates the call sites a rule is supposed to hold at. Same
class as T-2666/T-2667 (silent-exit fixed in `discover`, left un-migrated in
`session.rs` and `remote.rs`) and T-2670/T-2671/T-2673 (busy-spin).

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

### 2026-08-17 — behaviour on repeated inject failure: throttled hint, NOT auto-detach

- **Chose:** announce the START of a failure streak, then re-announce every 25th
  consecutive failure while it persists; reset the counter on any delivered inject.
  Never auto-detach. Encoded in the pure `should_warn_inject_failure(consecutive)`.
- **Why:** the two ends of the design space both fail, in opposite directions.
  Warning per keystroke fires hardest exactly when it hurts most — an operator
  holding a key down against a dead session would have the PTY render scribbled
  over at the worst moment. Warning once per process is the mirror failure: the
  `status:"resolved"` condition is *permanent*, so a single line said and never
  repeated leaves an operator who glanced away with no way to learn. Re-announcing
  periodically costs one line per 25 keystrokes — perceptually silent while typing
  normally, unmissable while typing into the void.
- **Rejected — auto-detach after N failures.** Tempting, and the sibling data-plane
  loop does exactly that (`break` on write error). Rejected for two reasons.
  (1) It would tear the operator out of a session over what may be a transient hub
  blip, and it cannot distinguish that from a permanent one without waiting — by
  which point the poll branch has already handled the genuinely-dead case itself.
  (2) The AC requires the detach-key and output-poll branches keep their behaviour;
  detaching from the inject branch means two branches now decide when the loop ends,
  and a race between them is a worse bug than the one being fixed. The operator is
  *told* to press Ctrl+] and keeps the choice.
- **Rejected — blocking on the RPC to get the error.** The call is fire-and-forget
  for latency, and that is correct; the mistake was never the non-blocking, it was
  discarding the `Result`. `.await`-ing the outcome to *classify* it does not
  reintroduce the original problem — the loop already awaits this call today.

### 2026-08-17 — reuse `inject_status_is_injected` rather than re-implement the check

- **Chose:** `attach_inject_failure_reason` delegates the "did it land?" question to
  the existing T-2697 helper.
- **Why:** the fail-closed rule (an unclassifiable envelope counts as not-delivered)
  then has exactly one definition. A second copy would be free to drift, and this
  bug exists *because* the rule lived in two places and reached neither third one.
  `unclassifiable_envelope_fails_closed` pins the inherited behaviour so a future
  change to the helper cannot silently loosen it here.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Recommendation

**Recommendation:** GO

**Rationale:** All 6 Agent ACs are met and all 11 verification commands pass. The
one open item is the Human AC, and it is open for a real reason rather than as
paperwork: whether a hint renders cleanly inside a raw-mode PTY cannot be asserted
from a unit test, and this repo has no PTY harness (the four `interactive_integration`
tests are `#[ignore]`d for exactly that reason). Everything that COULD be made
mechanical was: the throttle policy and the failure classification are pure functions
with 8 tests, and those tests were proven load-bearing by mutation rather than assumed
— removing the status check fails 3 of them, making the hint unconditional fails 2.

Two things a reviewer should weigh before accepting:

1. **The fix is larger than the filed defect, deliberately.** The filing named the
   discarded `Err`. Building it surfaced a second, worse failure mode the filing did
   not describe — `status:"resolved"` (no PTY) returns RPC **success**, so the
   sibling poll branch never notices and the operator types into a live-looking
   session indefinitely. Fixing only the literal filed defect would have left the
   worse half in place while looking complete. Full analysis in the RCA amendment.
2. **A behavioural choice was made** (throttled hint, never auto-detach) with the
   rejected alternatives recorded in `## Decisions`. If the reviewer disagrees with
   the every-25th re-announce cadence, that is a one-line change in
   `should_warn_inject_failure` with a test already pinning the shape.

**Evidence:**
- `crates/termlink-cli/src/commands/pty.rs` — `let _ = client::rpc_call(socket, "command.inject", params).await;` replaced by classify-then-report; two new pure helpers (`should_warn_inject_failure`, `attach_inject_failure_reason`).
- `cargo test -p termlink`: **1137 + 3 + 174 passed, 0 failed**, including the 8 new tests.
- Mutation proof: status check removed → 3 tests fail; throttle removed → 2 tests fail. Both mutations executed and reverted; tree restored green.
- P-011: **11/11 verification commands PASS**.
- `bash scripts/run-guard-layer.sh` → PASS (no new alloc-sink / drain-sink / silent-exit / busy-spin candidates).
- Detach-key and output-poll branches verified untouched — the change is confined to the inject arm of the `select!`.

**What a GO does NOT claim:** that the hint has been seen rendering in a real raw-mode
terminal. That is precisely what the Human AC is for; it should not be checked on the
strength of this recommendation.

## Updates

### 2026-08-12T15:02:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2644-pty-interactive-attach-silently-drops-ke.md
- **Context:** Initial task creation

### 2026-08-16T23:01:35Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-08-16T23:22:49Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
