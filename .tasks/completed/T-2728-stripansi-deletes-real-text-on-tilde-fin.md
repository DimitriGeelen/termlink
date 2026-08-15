---
id: T-2728
name: "strip_ansi deletes real text on tilde-final CSI and leaks DCS/APC payloads"
description: >
  strip_ansi deletes real text on tilde-final CSI and leaks DCS/APC payloads

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
created: 2026-08-15T08:06:36Z
last_update: 2026-08-15T08:19:27Z
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

# T-2728: strip_ansi deletes real text on tilde-final CSI and leaks DCS/APC payloads

## Context

Rank-1 item in `.context/upstream/herdr-adoption-backlog.md` — the top MEASURED
defect from the T-2725 herdr adoption research (worker 1, class D). Filed and
scoped but NOT implemented: the budget gate reached `urgent` mid-session, and a
two-crate edit that gets blocked at `critical` halfway through leaves the tree
inconsistent. Everything needed to execute is below.

**Why it matters more than it looks.** This is a "give me the clean text" API,
so its entire contract is fidelity, and it fails in *both* directions: it
deletes real characters AND emits escape-sequence payloads as text. The output
stays plausible, so nothing downstream notices — a silent wrong answer, D2.

### The two defects

Both implementations are near-identical copies:
`crates/termlink-session/src/ansi.rs:5-46` and
`crates/termlink-cli/src/util.rs:4-51`.

**1. CSI terminator is wrong.** Both break the CSI scan on
`ch.is_ascii_alphabetic()` (the CLI copy adds redundant `|| ch == 'h' || ...`
arms that are already alphabetic — cosmetic noise, not a second bug). Per
ECMA-48 a CSI final byte is **`0x40..=0x7E`**, which includes `~`, `@`, `[`,
`\`, `]`, `^`, `_`, `` ` ``, `{`, `|`, `}`. So `"\x1b[3~hello"` does not
terminate at `~`; the loop eats the `h` and yields `"ello"`. Bracketed paste
(`\x1b[200~` / `\x1b[201~`) hits this on every paste.

**2. DCS/SOS/PM/APC payloads leak as text.** The `_ =>` arm skips exactly one
character. `ESC P` (DCS), `ESC X` (SOS), `ESC ^` (PM) and `ESC _` (APC) are
*string* sequences that run until ST (`ESC \`), so their entire payload is
emitted as visible text.

### The fix

In the CSI arm, replace the break condition with the final-byte range:

```rust
if ('\x40'..='\x7e').contains(&ch) { break; }
```

Add a string-sequence arm alongside `Some(']')`, consuming to ST — the OSC arm's
existing `ESC \` handling is the pattern to copy:

```rust
Some('P') | Some('X') | Some('^') | Some('_') => { /* consume through ST */ }
```

### Do not skip this part

**Fix BOTH copies.** The duplication is itself a finding (worker 3 measured 94
duplicated `*_mcp` helpers and an 8.8% parity-test coverage ratio); fixing one
copy is how the two silently diverge. `termlink-cli` already depends on
`termlink-session` (`crates/termlink-cli/Cargo.toml:9`), so promoting
`ansi::strip_ansi_codes` from `pub(crate)` to `pub` and deleting the CLI copy is
viable and preferable — but it is a public-surface change, so make it
deliberately rather than as a side effect. If the copies are kept, put the same
test in both modules so a future divergence fails the suite.

**Write the tests first and watch them fail.** Assert
`strip("\x1b[3~hello") == "hello"`, the bracketed-paste pair, and that a DCS
payload does not appear in the output. A test that passes before the fix is
testing nothing — that check is what proved T-2727's regression test
load-bearing.

---

## STATUS: COMPLETE. Tree is green.

**Do NOT re-apply the change below — it is already in the tree.** This section
is a record of what landed, kept so the diff can be reviewed against its intent.

The work landed across two commits because the context budget hit its critical
gate mid-change:

- `2ec87688c` — both defects fixed, the two implementations consolidated, and
  one pre-existing test (`strip_ansi_bare_escape_consumed`) left RED. The gate
  blocks `git checkout` at critical, so the revert was impossible and the work
  was committed deliberately, with the red test named in the commit message
  rather than left to look finished.
- this commit — corrected that one test. `ESC X` → `ESC 7`.

Measured now: `cargo test -p termlink-session -p termlink` → **1078 + 174 + 439
passing, 0 failing**. (At `2ec87688c` it was 1078 + 174 + 438 passing, exactly 1
failing.)

**Confirmed failing output before the fix** (this is the reproduction, keep it):

```
strip_ansi_csi_non_alphabetic_final_byte   left: "ello"               right: "hello"
strip_ansi_string_sequences_consume_payload left: "aq some payload b" right: "ab"
```

### The change, in four parts

1. `crates/termlink-session/src/ansi.rs` — add
   `fn is_csi_final(ch: char) -> bool { ('\x40'..='\x7e').contains(&ch) }` and
   use it as the CSI break condition.
2. Same file — widen the OSC arm to
   `Some(']') | Some('P') | Some('X') | Some('^') | Some('_') =>`, so the
   string sequences share OSC's run-to-ST discipline.
3. `crates/termlink-session/src/lib.rs` — `pub(crate) mod ansi;` → `pub mod ansi;`,
   and `strip_ansi_codes` → `pub`.
4. `crates/termlink-cli/src/util.rs` — delete the local copy, replace with
   `pub(crate) use termlink_session::ansi::strip_ansi_codes;`. Only one call
   site consumes it (`commands/pty.rs:143`), so this is a small diff. Keep
   duplicate regression tests in the CLI test module: if anyone reintroduces a
   local copy, it must satisfy the same contract.

### THE ONE THING THAT BIT ME — resolved, kept as the reasoning record

An existing test failed after the fix, and **it was the test that was wrong, not
the fix**:

```rust
fn strip_ansi_bare_escape_consumed() {
    assert_eq!(strip_ansi_codes("\x1bXrest"), "rest");   // ← now returns ""
}
```

It was written to pin the catch-all "a bare two-character escape is consumed"
behaviour, and picked `X` as an arbitrary unknown escape character. **`X` is not
arbitrary: `ESC X` is SOS (Start Of String), 0x58** — a string sequence that
runs to ST. With no ST in `"\x1bXrest"` the correct output is `""`, which is what
the fixed implementation returns and what the new
`strip_ansi_string_sequences_consume_payload` test already asserts for the
unterminated-DCS case.

Fixed by choosing escapes that really are bare — `ESC 7` (DECSC, save cursor)
and `ESC 8` (DECRC, restore cursor) — with a comment in the test saying why `X`
was replaced, so nobody "restores" it later. `ESC X` did not simply move; it now
lives in `strip_ansi_string_sequences_consume_payload`, asserting the *opposite*
expectation, which is where an SOS case belongs.

Note the CLI crate's package name is **`termlink`**, not `termlink-cli` —
`-p termlink-cli` errors with "did not match any packages". The `## Verification`
block below uses the correct name; the AC wording that said `termlink-cli` was
itself wrong and is corrected.

**The generalisable lesson.** This test passed for as long as it existed while
asserting behaviour the spec contradicts, because it was written from the
implementation rather than from ECMA-48 — the author picked `X` meaning "some
letter, nothing special", and the implementation agreed because it had the same
gap. A test derived from the code under test cannot detect a defect the code
already has. Same shape as T-2680 and T-2709: a guard whose verdict rests on an
assumption about its input that nobody re-checked.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] CSI sequences terminate on the ECMA-48 final-byte range `0x40..=0x7E`,
      not on `is_ascii_alphabetic()` — so `"\x1b[3~hello"` yields `"hello"`,
      not `"ello"`
- [x] DCS / SOS / PM / APC string sequences (`ESC P`, `ESC X`, `ESC ^`,
      `ESC _`) consume through their ST terminator instead of emitting the
      payload as text
- [x] BOTH implementations are fixed — `termlink-session/src/ansi.rs` and
      `termlink-cli/src/util.rs` — since the divergence risk is the point.
      Satisfied by *removing* the divergence: `util.rs` now re-exports the
      session implementation, so there is one implementation to fix
- [x] Regression tests covering tilde-final CSI, bracketed paste, and a DCS
      payload exist in BOTH modules and FAIL against the pre-fix code
      (measured: `left: "ello"` / `left: "aq some payload b"`)
- [x] `cargo test -p termlink-session -p termlink` reports no FAILED
      (the CLI package is named `termlink`; the original AC's `-p termlink-cli`
      does not resolve — corrected here rather than left to mislead)

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

# The suite must be green. Capture-first per L-387; the CLI package is named
# `termlink`, not `termlink-cli`.
out=$(cargo test -p termlink-session -p termlink 2>&1); ! echo "$out" | grep -q "FAILED"

# There must be exactly ONE implementation. The whole reason both defects
# survived is that a reader fixing one copy left the other wrong, so this
# asserts the CLI re-exports rather than redefines.
grep -q "pub(crate) use termlink_session::ansi::strip_ansi_codes" crates/termlink-cli/src/util.rs

# `ESC X` must never be reinstated as a *bare-escape* expectation: it is SOS,
# and asserting it yields "rest" is precisely the wrong-test this task fixed.
# (It is legitimate elsewhere — the string-sequence test asserts it yields "".)
! grep -qF 'x1bXrest"), "rest"' crates/termlink-session/src/ansi.rs

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

### 2026-08-15T08:06:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2728-stripansi-deletes-real-text-on-tilde-fin.md
- **Context:** Initial task creation
