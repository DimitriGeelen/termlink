---
id: T-2514
name: "Fix UTF-8 char-boundary panic in CLI truncate() (byte-slice on multibyte name/cap/role)"
description: >
  Fix UTF-8 char-boundary panic in CLI truncate() (byte-slice on multibyte name/cap/role)

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/util.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-03T17:35:36Z
last_update: '2026-08-18T18:59:12Z'
date_finished: 2026-08-03T17:39:10Z
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
  - ts: '2026-08-18T18:56:49Z'
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
  - ts: '2026-08-18T18:59:12Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2514: Fix UTF-8 char-boundary panic in CLI truncate() (byte-slice on multibyte name/cap/role)

## Context

`truncate(s: &str, max: usize)` (crates/termlink-cli/src/util.rs:54-60) truncates
a display string with:

```rust
if s.len() <= max { s.to_string() } else { format!("{}…", &s[..max - 1]) }
```

`s.len()` and `max` are BYTE counts, and `&s[..max-1]` is a BYTE-index slice. If a
multi-byte UTF-8 character straddles byte index `max-1`, the slice is not on a
char boundary → **panic** (`byte index N is not a char boundary`). Additionally
`max == 0` underflows `max - 1` (usize) → panic. The strings passed are
user-controlled — session `--name`/`--capability`/`--role`, remote profile
names/ids from `hubs.toml` — reached by everyday read commands: the interactive
session picker `pick_session` (util.rs:167), `commands/metadata.rs`,
`commands/session.rs`, `commands/remote.rs`. A single unicode name (e.g. `é`)
hard-crashes the listing/pick flows. Charter verb affected: control terminal
sessions (session listing/selection).

## Acceptance Criteria

### Agent
- [x] `truncate` counts and slices by `char`, never by raw byte index — no `&s[..n]` byte-slice remains in the function
- [x] `truncate` handles `max == 0` without underflow/panic (returns empty string)
- [x] A regression test truncates a string with a multi-byte char straddling the cut point and asserts the result is a valid `String` of the expected char count — FAILS (panics) against the old byte-slice code (proven load-bearing by temp-revert)
- [x] A regression test covers `truncate("", 0)` and `truncate("abc", 0)` (underflow guard)
- [x] Existing ASCII `truncate` tests still pass; `cargo test -p termlink --bins util` (or the crate's test target) passes; `cargo build --release -p termlink` succeeds

### Ready-to-apply fix (turnkey — if this window's budget blocks source edits, next window applies verbatim)

Replace the body of `truncate` (crates/termlink-cli/src/util.rs:54-60) with:

```rust
pub(crate) fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    if max == 0 {
        return String::new();
    }
    let head: String = s.chars().take(max - 1).collect();
    format!("{head}…")
}
```

Add tests (util.rs `#[cfg(test)] mod tests`):

```rust
#[test]
fn truncate_multibyte_no_panic() {
    // 'é' straddles the byte cut point; byte-slice code panics here.
    let s = "aaaaaaaaaaaaaaaaaé more";
    let out = truncate(s, 19);
    assert_eq!(out.chars().count(), 19);
    assert!(out.ends_with('…'));
}

#[test]
fn truncate_zero_max_no_underflow() {
    assert_eq!(truncate("", 0), "");
    assert_eq!(truncate("abc", 0), "");
}
```

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
cargo test -p termlink --bins util 2>&1 | tee /tmp/t2514-test.log; grep -q "test result: ok" /tmp/t2514-test.log
cargo build --release -p termlink 2>&1 | tail -3

## RCA

**Symptom:** CLI process panics (`byte index N is not a char boundary`) on common
read commands — `sessions`/`list`, the interactive session picker, `remote list` —
whenever a session/profile name, capability, or role contains a multi-byte UTF-8
character that lands on the truncation cut point. Hard denial of the listing/pick
flows, triggerable by a single unicode name.

**Root cause:** `truncate` mixes byte and char units — it compares `s.len()`
(bytes) against `max` and then slices `&s[..max-1]` by raw byte index. Rust `&str`
byte-indexing panics unless the index is a char boundary, and a multi-byte char
straddling `max-1` violates that. Separately, `max == 0` underflows the `max-1`
subtraction (usize).

**Why structurally allowed:** all four existing `truncate` tests use pure-ASCII
inputs, where byte index == char index, so a mid-char split can never occur and
the panic path is never exercised. No test fed a multi-byte string.

**Prevention:** char-aware truncation (count/take by `char`) + an explicit
`max == 0` guard; a regression test with a multi-byte char straddling the cut
(panics against the old code, passes after) + a zero-max test. Captured as PL-299
(a display/format helper that compares a byte length then byte-slices a `&str` is
a latent UTF-8 panic; operate in char units and test with a multi-byte straddling
input, not just ASCII).

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

### 2026-08-03T17:35:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2514-fix-utf-8-char-boundary-panic-in-cli-tru.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-fde252e5
- **Timestamp:** 2026-08-03T17:50:44Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-03T17:39:10Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
