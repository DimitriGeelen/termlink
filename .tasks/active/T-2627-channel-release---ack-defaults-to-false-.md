---
id: T-2627
name: "channel release --ack defaults to false at raw CLI — silently reopens claimed slot for retry (footgun default)"
description: >
  channel release --ack defaults to false at raw CLI — silently reopens claimed slot for retry (footgun default)

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
created: 2026-08-12T05:53:17Z
last_update: 2026-08-12T06:12:15Z
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

# T-2627: channel release --ack defaults to false at raw CLI — silently reopens claimed slot for retry (footgun default)

## Context

**FILED, NOT BUILT** (usability-hunt Directive #3 Finding #3, LOW-MED
confidence, DESIGN-level — verified in code; filed because this window hit
critical budget AND the safe fix needs a design call).

`crates/termlink-cli/src/cli.rs:3458-3468` — `channel release`'s `ack` flag:

```rust
/// Acknowledge the work as completed — advances cursor past the offset.
/// Without this flag, slot reopens without cursor advance.
#[arg(long)]
ack: bool,
```

At the raw-CLI tier, `termlink channel release --claim-id X --claimer Y`
(no `--ack`) does the SURPRISING thing: the work is returned for retry and
will be re-dispatched, with no cursor advance. This is the >90%-wrong default —
the `/release` skill DELIBERATELY inverts it ("done by default, `--retry` to
opt out", per CLAUDE.md) precisely because reversing it IS a footgun. But
operators/scripts calling the binary directly still hit the dangerous default.

Confidence is LOW-MED because (a) the doc-comment does spell out the behavior
and (b) the project consciously chose to fix this at the skill layer. Still a
real Directive-#3 footgun for anyone not going through the skill.

**This needs a design decision, not just a code change** — hence filed, not
auto-built (choosing between a breaking semantics flip and a non-breaking note
is a judgment call the operator should sanction).

## Acceptance Criteria

### Agent
- [x] Decide (record in ## Decisions) between: (A) non-breaking — emit a one-line stderr note on ack-less release (`note: slot reopened for retry; pass --ack to mark completed`); (B) breaking — flip to `--no-ack` semantics so "done" is the CLI default matching the skill. Default recommendation: (A), non-breaking, unless the operator sanctions a breaking change. **→ Chose A (see ## Decisions).**
- [x] Implement the chosen option
- [x] If (A): the note is emitted only in human (non-`--json`) mode on ack-less release; a unit-testable pure fn builds the note; load-bearing test asserts the note text appears for ack=false and is absent for ack=true
- [x] If (B): update the skill layer + all call sites + docs; regression test for the flipped default — N/A (chose A)
- [x] `cargo test -p termlink --bins` passes

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

cargo test -p termlink --bins release_retry_note

## RCA

**Symptom:** `termlink channel release --claim-id X --claimer Y` at the raw CLI (no `--ack`) silently returns the work for retry (re-dispatch, no cursor advance) — the >90%-unwanted behavior — while an operator would reasonably expect "release" to mean "done".

**Root cause:** `cli.rs:3458` defaults `ack: bool` to false; "done" requires the opt-in `--ack`. The safer default is inverted.

**Why structurally allowed:** the fix was applied at the SKILL layer (`/release` adds `--ack` unless `--retry`), not the CLI, so the underlying binary keeps the footgun default for direct callers and scripts. No CLI-tier note or default-flip guards it.

**Prevention:** either a non-breaking stderr note on ack-less release (option A, recommended) or a breaking `--no-ack` semantics flip (option B); load-bearing test on the chosen path. Failure scenario: a script calls `termlink channel release --claim-id X --claimer Y` expecting completion → the offset silently reopens and the unit is re-dispatched to another worker, doing the work twice.

<!-- (template guidance retained)
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

### 2026-08-12 — ack-less release: advisory note vs breaking flag flip
- **Chose:** Option A — non-breaking. Emit a one-line stderr advisory on the ack-less release path (human/non-`--json` mode only): `note: released WITHOUT --ack — slot reopened for retry (cursor NOT advanced). Pass --ack to mark the work completed.` A pure fn `release_retry_note(ack) -> Option<String>` builds it; it fires on `ack==false`, is `None` on `ack==true`. Keyed off the server-confirmed `r.ack` (truthful about what actually happened), not the input flag.
- **Why:** A breaking semantics flip (option B) would silently change the meaning of every existing `channel release --claim-id X --claimer Y` invocation and every script/skill call site — a wire/UX contract change that needs explicit operator sanction (broad "choose what to work on" delegates INITIATIVE, not AUTHORITY to make breaking changes). Option A closes the Directive-#3 footgun (opaque surprising default) with zero contract change: direct callers now SEE that the slot reopened, while the `/release` skill's inverted default is unaffected. JSON consumers are untouched (note is stderr, human-mode only) so machine parsers don't regress.
- **Rejected:** Option B (flip to `--no-ack` so "done" is the CLI default) — correct in principle but breaking; deferred to an operator-sanctioned change. If the operator later wants B, this advisory becomes redundant and can be removed alongside the flip.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-12T05:53:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2627-channel-release---ack-defaults-to-false-.md
- **Context:** Initial task creation
