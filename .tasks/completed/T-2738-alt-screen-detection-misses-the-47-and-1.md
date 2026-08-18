---
id: T-2738
name: "Alt-screen detection misses the ?47 and ?1047 variants — a session reports
  normal screen while in alt-screen"
description: >
  Alt-screen detection misses the ?47 and ?1047 variants — a session reports normal
  screen while in alt-screen

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-session/src/pty.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-15T13:59:44Z
last_update: '2026-08-18T18:59:15Z'
date_finished: 2026-08-15T14:06:38Z
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
  - ts: '2026-08-18T18:56:58Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 2
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=2 
      (body:telemetry-or-audit-entry); D3=2 (body:default-change); D4=2 
      (body:env-class-handled); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
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

# T-2738: Alt-screen detection misses the ?47 and ?1047 variants — a session reports normal screen while in alt-screen

## Context

Herdr adoption backlog rank 16 (worker 1, class J; GREP-PROVEN ABSENCE).
`PtySession::scan_alternate_screen` matches only `\x1b[?1049h/l`. The older
DECSET variants `\x1b[?47h/l` (6 bytes) and `\x1b[?1047h/l` (8 bytes) are not
recognised, so a session driven by an older or simpler TUI enters the alternate
screen while `terminal_mode()` keeps reporting `alternate_screen: false` — a
confidently wrong answer, not a refusal.

Adding a 6-byte sequence is not a one-line table change. The current scanner is
built on a **fixed** 8-byte window, and its doc comment argues the
no-double-count property from that fixed length: "because the carry is strictly
shorter than a full sequence, no 8-byte window can lie entirely within it". Once
a 6-byte sequence exists that argument is false — the 7-byte carry can hold a
complete `\x1b[?47h` that was already applied on the previous read. So the
invariant has to be enforced in code rather than inherited from a length
coincidence, and the comment has to be rewritten to state what is actually
guaranteed. Leaving the old comment in place would reproduce PL-343 (a comment
asserting a property the code no longer has) in the very function being fixed.

## Acceptance Criteria

### Agent
- [x] `scan_alternate_screen` recognises all six sequences: `?1049h/l`, `?1047h/l`, `?47h/l`, driven from one table rather than open-coded pairs
- [x] The scan handles variable-length sequences; the carry is sized from the longest sequence, not from a hardcoded 8
- [x] The no-double-count invariant is enforced explicitly — a match is only applied if it extends past the carry into the new chunk — rather than being implied by a fixed window length
- [x] The doc comment states the invariant the code enforces; the stale "no 8-byte window can lie entirely within it" argument is gone
- [x] Tests cover the `?47h` → `?47l` toggle
- [x] Tests cover the `?1047h` → `?1047l` toggle
- [x] A split-read test covers the 6-byte variant specifically (the length the old carry logic could not have handled)
- [x] A negative test proves `\x1b[?1048h` (save-cursor, not alt-screen) does not toggle the flag — the scanner discriminates rather than matching anything `?10..`
- [x] An observable test proves a sequence sitting entirely inside the carry is not re-applied on the next chunk
- [x] Load-bearing proof recorded for both halves (sequence table and carry invariant); `cargo test -p termlink-session` green and `scripts/run-guard-layer.sh` clean

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

cargo test -p termlink-session --lib pty::tests
bash scripts/run-guard-layer.sh

## RCA

**Symptom.** A session driven by a TUI that uses `\x1b[?47h` or `\x1b[?1047h`
entered the alternate screen while `terminal_mode()` kept reporting
`alternate_screen: false`. Not a refusal, not an error — a confident wrong
answer, which is the shape that costs the most downstream.

**Root cause.** `scan_alternate_screen` hardcoded the single pair
`\x1b[?1049h/l` and scanned with a fixed 8-byte window. The other two DECSET
pairs were never in the table.

**Why structurally allowed — this is the interesting part.** The tree *already
knew*. `crates/termlink-cli/src/commands/pty.rs:1447` — the terminal-restore
path from T-2731 — handles all three variants and carries a test comment
spelling out the exact reason: "`?1047` and `?47` are what older curses apps
actually emit, and a terminal left on the alt screen by `?47h` is not returned
by `?1049l`… handling only the variant you expected is indistinguishable from
handling none of them, from the operator's seat." So one half of the codebase
had the knowledge written down as a tested invariant while the other half, doing
the mirror-image job on the same three sequences, knew about one. Nothing
connects a restore-side fact to a detect-side omission: they are different
crates, different functions, and no test spans both. The knowledge did not have
to be discovered — it had to be *propagated*, and nothing propagates it.

**Second finding, filed separately.** The same grep turned up a third surface:
`crates/termlink-cli/src/commands/mirror_grid.rs:666` matches `1049` alone, so
`?47h` / `?1047h` fall into its `_ => unhandled_csi += 1` arm and the mirror
renders alt-screen output into the primary buffer. One bug = one task, so that
is **T-2739** rather than scope creep here.

**Prevention.** Six sequences now come from one table, and the tests cover each
pair, the 6-byte split-read, and a negative case (`?1048`, a cursor save, must
not toggle) so the scanner is pinned as *discriminating* rather than merely
permissive.

The subtler prevention is the carry invariant. Adding a 6-byte sequence silently
invalidated the doc comment's argument for why nothing is double-counted — that
argument was "no 8-byte window can lie entirely within a 7-byte carry", which is
true only while every sequence is 8 bytes. A 6-byte sequence fits in that carry
with a byte to spare. Rather than leave a comment asserting a property the code
no longer had — PL-343, reproduced inside the function being fixed — the rule is
now enforced in code (`end <= carry_len` skips a match already applied) and the
comment describes what is enforced. The `alternate_screen_does_not_reapply_a_
sequence_held_in_carry` test makes the invariant observable, which it was not
before: the stale match would have rewritten the same value and hidden.
`alt_screen_max_seq_len_matches_table` closes the remaining edge — adding a
longer sequence later cannot silently under-size the carry.

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

### 2026-08-15T13:59:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2738-alt-screen-detection-misses-the-47-and-1.md
- **Context:** Initial task creation

### 2026-08-15T14:06:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
