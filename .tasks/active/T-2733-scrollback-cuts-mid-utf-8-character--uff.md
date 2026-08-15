---
id: T-2733
name: "Scrollback cuts mid-UTF-8 character — U+FFFD corruption on --bytes and ring overflow (herdr item 5)"
description: >
  Scrollback cuts mid-UTF-8 character — U+FFFD corruption on --bytes and ring overflow (herdr item 5)

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
created: 2026-08-15T10:45:58Z
last_update: 2026-08-15T10:47:54Z
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

# T-2733: Scrollback cuts mid-UTF-8 character — U+FFFD corruption on --bytes and ring overflow (herdr item 5)

## Context

Herdr adoption backlog item 5 (rank 5).

`ScrollbackBuffer` is a `VecDeque<u8>` — a byte ring, deliberately, because it
stores raw terminal output including ANSI sequences. Three of its cuts land on
arbitrary byte offsets:

- `last_n_bytes(n)` starts at `len - n` (`scrollback.rs:41`)
- `append` drains `..overflow` when the ring is full (`:33`)
- `append` keeps `data[len - max_bytes..]` when a single write exceeds capacity
  (`:26`)

A multi-byte character straddling any of those offsets is cut in half.
`handler.rs` then runs the bytes through `from_utf8_lossy`, so the partial
character surfaces as U+FFFD — a replacement glyph in output the operator is
reading to decide what happened. Nothing errors; the text is simply wrong at
that one position.

**Why this is rank 5 and not higher, stated honestly:** the *default* read path
is safe. `last_n_lines` cuts on `\n`, which is ASCII and therefore always a
character boundary. Only `--bytes` and ring overflow corrupt. But `cmd_interact`
polls with `bytes: 131072` (`commands/pty.rs:104–107`), so the corrupting path
is on a live verb, not a theoretical one.

The fix already half-exists in-tree: `char_boundary_floor`
(`commands/pty.rs:1094`). It cannot be reused as-is — it takes `&str` (we hold
bytes that are not guaranteed valid UTF-8) and it moves **backward**, which is
correct for truncating a tail and wrong for a front cut. Moving backward from a
start offset would *include* the orphaned lead byte instead of dropping it.
This needs the ceiling direction: advance forward past continuation bytes.

## Acceptance Criteria

### Agent
- [x] A boundary helper in `scrollback.rs` advances a start offset FORWARD past
      UTF-8 continuation bytes (`0b10xxxxxx`), so a front cut never begins
      mid-character
- [x] The helper is bounded — it advances at most 3 bytes and then yields the
      original offset unchanged, so non-UTF-8 / binary terminal output is never
      trimmed arbitrarily
- [x] All three cut sites use it: `last_n_bytes`, the `append` overflow drain,
      and the `append` oversized-write tail-keep
- [x] `last_n_lines` is left alone, with a comment stating why (`\n` is ASCII,
      so its cut is already a boundary) — so the next reader does not "fix" it
- [x] Test: a buffer of multi-byte characters read via `last_n_bytes` at an
      offset that splits one returns valid UTF-8 with no U+FFFD
- [x] Test: ring overflow that splits a multi-byte character leaves the buffer
      starting on a boundary
- [x] Test: an oversized single write that splits a character keeps a boundary
      start
- [x] Test: binary (non-UTF-8) content is not over-trimmed by the bounded scan
- [x] Each test is demonstrated load-bearing — reverting the fix makes it fail
- [x] `cargo test -p termlink-session` passes
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

# The scrollback boundary tests.
cargo test -p termlink-session scrollback
# The helper exists and every cut site routes through it (3 sites + definition).
n=$(grep -c 'utf8_boundary_ceil' crates/termlink-session/src/scrollback.rs); [ "$n" -ge 4 ]
# Whole session crate still green.
cargo test -p termlink-session
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

**Symptom:** Terminal output read back through `--bytes` (and any read after the
scrollback ring had wrapped) could contain a U+FFFD replacement glyph where a
real multi-byte character had been. No error, no warning — the operator simply
read one wrong character in output they were using to decide what happened.

**Root cause:** `ScrollbackBuffer` is a byte ring by deliberate design (it
stores raw terminal output including ANSI sequences, which is not text). Three
of its cuts — `last_n_bytes`'s `len - n`, `append`'s overflow `drain(..)`, and
`append`'s oversized-write tail-keep — used arbitrary byte offsets, so a
character straddling one was halved. `handler.rs` then applied
`from_utf8_lossy`, which is precisely the call that converts a truncation
mistake into a plausible-looking wrong character instead of an error.

**Why structurally allowed:** the byte/character boundary distinction was
already understood in this codebase — `commands/pty.rs::char_boundary_floor`
exists and does exactly this job. It just lives in a different crate, takes
`&str`, and moves in the opposite direction, so it was not reusable and nobody
carried the idea across. Two contributing factors kept it invisible: the
*default* read path (`last_n_lines`) cuts on `\n` and is safe by construction,
so ordinary use never showed the bug; and `from_utf8_lossy` is a silent
downgrade — the one API choice that guarantees a truncation defect surfaces as
content rather than as a failure. Directive #2 is "no silent failures", and a
lossy decode is a silent failure by construction.

**Prevention:** five unit tests, each pinned to one cut site and each
demonstrated to fail when that site's guard is removed. They sweep every
offset and every ring capacity rather than sampling one, because the defect is
offset-specific — a single-offset test would have passed against the broken
code for most choices of offset. The bounded-scan test pins the *limit* as
well as the behaviour, since an unbounded scan would fix UTF-8 output by
corrupting binary output.

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

### 2026-08-15T10:45:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2733-scrollback-cuts-mid-utf-8-character--uff.md
- **Context:** Initial task creation
