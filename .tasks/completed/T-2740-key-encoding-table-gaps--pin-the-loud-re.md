---
id: T-2740
name: "Key-encoding table gaps — pin the loud refusal, then widen resolve_key beyond its current subset"
description: >
  Key-encoding table gaps — pin the loud refusal, then widen resolve_key beyond its current subset

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-session/src/executor.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-15T14:20:31Z
last_update: 2026-08-15T14:27:24Z
date_finished: 2026-08-15T14:27:24Z
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

# T-2740: Key-encoding table gaps — pin the loud refusal, then widen resolve_key beyond its current subset

## Context

Herdr adoption backlog rank 10 (worker 1, class E). `executor.rs::resolve_key`
is missing Ctrl+I/J/M, Ctrl+[/]/^/_, every F-key, Shift+Tab, PageUp/PageDown,
Insert, and all modified arrows. The backlog ranks it *below* the correctness
items on purpose, because it fails LOUD: `resolve_key` returns `None` and
`resolve_key_entry` turns that into `Unknown key: {name}`. That is a refusal, not
a silent drop, and it is the property that makes this a usability item rather
than a correctness one — so the backlog's instruction is to **pin the loud
failure first**, since that assertion is load-bearing and currently untested,
and only then widen the table.

Reading the table turned up a gap the backlog did not list: matching is on exact
strings, `"Ctrl+A" | "ctrl+a"`. Fully-capitalised and fully-lowercase spellings
work; the natural middle spelling `Ctrl+a` is refused. Every name in the table
has this shape, so the fix belongs at the lookup, not in more `|` arms.

## Acceptance Criteria

### Agent
- [x] The loud-refusal property is pinned first: tests assert `resolve_key` returns `None` for an unknown name and that `resolve_key_entry` surfaces it as `Unknown key: {name}` carrying the offending name
- [x] Every key name that resolves today still resolves to byte-for-byte the same sequence — pinned by a test enumerating the pre-existing table, so widening cannot silently change an existing binding
- [x] Lookup is case-insensitive, so `Ctrl+a`, `CTRL+A` and `ctrl+A` resolve identically to `Ctrl+A`
- [x] The missing control codes resolve: Ctrl+I (0x09), Ctrl+J (0x0A), Ctrl+M (0x0D), Ctrl+[ (0x1B), Ctrl+] (0x1D), Ctrl+^ (0x1E), Ctrl+_ (0x1F), and Ctrl+@ / Ctrl+Space (0x00)
- [x] The aliasing between those and their named equivalents is asserted, not incidental: Ctrl+I == Tab, Ctrl+J == LF, Ctrl+M == Enter, Ctrl+[ == Escape
- [x] F1–F12 resolve, with F1–F4 as SS3 (`ESC O P/Q/R/S`) and F5–F12 as CSI `<n>~` on the standard non-contiguous numbering (15,17,18,19,20,21,23,24)
- [x] Shift+Tab (`CSI Z`), PageUp (`CSI 5~`), PageDown (`CSI 6~`) and Insert (`CSI 2~`) resolve
- [x] Modified arrows and Home/End resolve as `CSI 1;<mod><final>` with the standard modifier arithmetic (1 + shift·1 + alt·2 + ctrl·4), covering combinations, not just single modifiers
- [x] A negative test proves the widening did not make the matcher permissive: plausible-but-unreal names (e.g. `F13`, `Ctrl+`, `Super+Up`, empty string) still refuse
- [x] Load-bearing proof recorded; `cargo test -p termlink-session` green and `scripts/run-guard-layer.sh` clean

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

cargo test -p termlink-session --lib executor
bash scripts/run-guard-layer.sh

## RCA

**Symptom.** `termlink inject`-style callers naming a key outside a narrow
subset got `Unknown key: F5`. No F-key, no PageUp/PageDown/Insert, no Shift+Tab,
no modified arrow, and none of Ctrl+I/J/M/[/]/^/_ resolved — the last group
being control codes that unambiguously exist and simply were not listed.

**Root cause.** `resolve_key` was a hand-maintained match with one arm per name,
listing each name twice (capitalised and lowercase). It covered what the first
author needed.

**Why structurally allowed — and why this is a usability bug, not a correctness
one.** The function fails LOUD: `None` becomes `Unknown key: {name}` naming the
offending key. That refusal is the whole reason the backlog ranked this item
*below* the correctness work, and it is the right design. But **the refusal was
itself untested** — the single property the ranking depended on rested on
nothing. So the first AC here, and the first change made, was to pin it
(`unknown_key_refuses_loudly`), before widening anything. Widening a table while
its refusal path is unpinned is how a loud failure quietly becomes a permissive
guess.

**The gap the backlog did not list.** Matching was on exact strings, so of the
three natural spellings of a modified key only two worked: `Ctrl+A` and
`ctrl+a` resolved, and `Ctrl+a` — arguably the most natural — was refused. That
affected every name in the table simultaneously and was invisible in a reading
of the arms, because each arm *looks* like it handles case. The fix belongs at
the lookup (normalise once), not in a third `|` arm per name.

The temp-revert of that one line is the strongest evidence in this task: it
fails **eight** tests including two that predate this work (`resolve_known_keys`,
`resolve_key_entries`). The table is lowercase-only now, so normalisation is
load-bearing for backward compatibility, not merely for the new spellings.

**Prevention.** `pre_existing_bindings_are_unchanged` enumerates every binding
that resolved before this change and asserts byte-for-byte equality in both
spellings, so a future collapse or re-normalisation of the table cannot silently
move an existing binding. `widened_table_still_refuses_unreal_keys` guards the
other direction with the names a caller would plausibly try (`F13`, `Ctrl+`,
`Super+Up`, empty) — a widening that made the matcher permissive would fail it.
The aliasing assertions are deliberate too: Ctrl+I == Tab and Ctrl+M == Enter
are asserted equal, while Ctrl+H (0x08) and Backspace (0x7F) are asserted
**un**equal, because that last pair is the one someone would "fix" wrongly.

**PL-344 applied.** Having just recorded that a correct answer on one surface
does not propagate to others, I grepped for a sibling key table before closing:
`resolve_key` has exactly one consumer (`handler.rs:573`) and no duplicate
elsewhere in the tree. Clean this time — but checked, not assumed.

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

### 2026-08-15T14:20:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2740-key-encoding-table-gaps--pin-the-loud-re.md
- **Context:** Initial task creation

### 2026-08-15T14:27:24Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
