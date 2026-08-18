---
id: T-2732
name: "Restore terminal private modes on detach — alt-screen/mouse/paste leak (herdr
  item 4)"
description: >
  Restore terminal private modes on detach — alt-screen/mouse/paste leak (herdr item
  4)

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/pty.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-15T09:54:11Z
last_update: '2026-08-18T18:59:15Z'
date_finished: 2026-08-15T10:31:07Z
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
  - ts: '2026-08-18T18:56:57Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=2 (body:env-class-handled); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:15Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2732: Restore terminal private modes on detach — alt-screen/mouse/paste leak (herdr item 4)

## Context

Herdr adoption backlog item 4 (rank 4, GREP-PROVEN ABSENCE — the strongest
non-measured evidence class in that document).

`cmd_attach` and the data-plane streaming attach in
`crates/termlink-cli/src/commands/pty.rs` both save the operator's `termios`,
enter raw mode, run their loop, and restore `termios` on detach. `termios` is
the *only* thing they restore.

But a child running under the session writes bytes straight through to the
operator's terminal, and those bytes can switch on terminal **private modes**:
alternate screen (`?1049h` / `?1047h` / `?47h`), mouse reporting
(`?1000h`–`?1006h`), bracketed paste (`?2004h`), focus reporting (`?1004h`).
Those modes live in the *terminal emulator*, not in `termios`, so
`tcsetattr` cannot undo them. Detach from a child sitting in `vim`, `less`, or
`htop` and the operator is returned to a shell that is still on the alternate
screen, still emitting escape garbage on every mouse move, still wrapping
pastes in `\e[200~`.

Grep across the tree finds **no emission site for any of these sequences in
product code** — the only hits are `mirror_grid.rs` tests feeding the
*detector*. So TermLink can recognise that a child entered alt screen and
still has no way to leave it. herdr closed its issue #2581 for this class,
which is field proof the class bites; the vocabulary came from there, the
defect is ours.

The backlog named two detach sites; reading found a third (`cmd_mirror --raw`,
byte passthrough with no raw-mode entry, so the `cfmakeraw` grep that located
the other two could not see it). One fix for all three: a single shared helper,
because T-2728 (two copies
of `strip_ansi_codes` carrying the same two defects for as long as they both
existed) is the freshest evidence in this repo that a duplicated terminal
primitive diverges.

## Acceptance Criteria

### Agent
- [x] A single helper emits the private-mode restore sequence; every detach
      site in `commands/pty.rs` calls it — no second copy of the byte string.
      Three sites, not the two the backlog named: `cmd_attach`, the data-plane
      streaming attach, and `cmd_mirror`. The third surfaced while reading —
      `--raw` mirror is byte passthrough, so it leaks identically, and it was
      invisible to the `cfmakeraw` grep that found the other two because it
      never enters raw mode
- [x] The sequence disables, at minimum: alternate screen (all three variants
      `?1049l` / `?1047l` / `?47l`), mouse reporting (`?1000l`–`?1006l`),
      bracketed paste (`?2004l`), focus reporting (`?1004l`)
- [x] Restore is emitted BEFORE `termios` is restored, so it is written while
      the terminal is still in the known raw state
- [x] Restore is emitted on BOTH the normal-return and the error-return path —
      a detach caused by a failed loop leaves the terminal no worse than a
      clean one
- [x] Unit test asserts the exact byte content of the helper's output, so a
      future edit that silently drops a mode fails the suite
- [x] Unit test asserts the ordering property (restore precedes termios
      handoff) at the level the code structure permits
- [x] `cargo test --workspace` passes with no new failures
- [x] `bash scripts/run-guard-layer.sh` stays clean

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

# The targeted unit tests for the restore sequence + its ordering.
cargo test -p termlink private_mode
# Exactly one definition of the helper — the anti-duplication AC (T-2728 lesson).
n=$(grep -c 'fn restore_terminal_private_modes' crates/termlink-cli/src/commands/pty.rs); [ "$n" = "1" ]
# Definition + both detach call sites = at least 3 mentions.
n=$(grep -c 'restore_terminal_private_modes' crates/termlink-cli/src/commands/pty.rs); [ "$n" -ge 3 ]
# Whole workspace still green.
cargo test --workspace
# Static guard layer still clean.
bash scripts/run-guard-layer.sh

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

**Symptom:** Detaching from a session whose child had entered the alternate
screen, enabled mouse reporting, enabled bracketed paste, or hidden the cursor
returned the operator to a terminal still in that state — shell prompt drawn on
the alt screen, escape bytes typed into the command line on every mouse move,
pastes wrapped in `\e[200~`, or no visible cursor. Recovery required `reset`.

**Root cause:** Both detach paths restored `termios` and only `termios`.
`termios` is kernel line-discipline state; private modes are emulator state.
`tcsetattr` cannot reach them, so no amount of correctness in the termios
handling could have fixed this — the restore was complete with respect to the
wrong layer.

**Why structurally allowed:** the tree already had a *detector* for this exact
condition (`PtySession::scan_alternate_screen`, with four unit tests covering
split reads and byte-at-a-time feeds) and no emitter anywhere. Detection was
built to answer `termlink status`; nothing connected "we can see the child
entered alt screen" to "so we must leave it on the way out". Worker 1's cluster
framing names the shared root behind items A/B/C of the herdr review:
**TermLink models a PTY nobody is watching** — which is charter-correct, and is
precisely why nothing in the tree sizes that PTY (item 2, T-2727), tears down
its modes (this item), or answers its queries (item 8, still open). The blind
spot is one assumption, surfacing three times.

**Prevention:** three unit tests pin the byte content, the all-disables
invariant, and the ordering. Dropping any sequence fails the first
(demonstrated: removing `?1047l` fails it). The all-disables test is the one
that matters most — the set is emitted unconditionally, which is only safe
while every sequence turns something off, and a single stray `h` would switch a
mode ON in the terminal of every operator who detaches. The structural fix is
that both sites call one `restore_terminal`, so the ordering lives in one place
and cannot drift the way `strip_ansi_codes` did (T-2728).

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

### 2026-08-15T09:54:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2732-restore-terminal-private-modes-on-detach.md
- **Context:** Initial task creation

### 2026-08-15T10:31:07Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
