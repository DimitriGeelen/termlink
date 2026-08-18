---
id: T-2699
name: "Nothing detects an error code that is defined and documented but never emitted"
description: >
  23 codes defined in control.rs::error_code; three have zero emission sites — SESSION_BUSY,
  MESSAGE_EXPIRED, PROTOCOL_VERSION_TOO_OLD. check-error-code-docs.sh verifies doc-to-definition
  pairing but never asks whether the code can actually be returned, so the published
  refusal taxonomy can contain fiction indefinitely (T-2698 G1).

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
created: 2026-08-14T08:25:09Z
last_update: '2026-08-18T18:58:39Z'
date_finished:
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
  - ts: '2026-08-18T18:55:37Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 2
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=2 (body:env-class-handled); F-RECALL=2 
      (body:lightly-promoted); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:39Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2699: Nothing detects an error code that is defined and documented but never emitted

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `scripts/check-error-code-emission.sh` flags every `error_code::` constant with no emission site in the product crates — the class `check-error-code-docs.sh` cannot see, because it only verifies doc↔definition pairing
- [x] Detection covers BOTH the symbolic form (`error_code::NAME`) and the bare numeric literal, so a code emitted as `-32004` is not miscounted as dead
- [x] The defining file is excluded from the emission scan — a constant must not count as its own use
- [x] A code whose only "use" is a **builder that itself has no callers** is still reported: `check_protocol_version` is defined, unit-tested, and invoked by nothing, which is exactly the case that must not read clean
- [x] Genuinely-reserved codes are acknowledged in `.context/checks/error-code-emission-allowlist` (git-tracked per T-2681) with a cited reason stating WHY it is not emitted and what would change that
- [x] Carries the `# guard-layer: source` marker so the T-2684 runner executes it
- [x] Exit codes 0 clean / 1 unacknowledged dead code / 2 tooling; `--json`; `--root` + `--allowlist` for fixtures; `--quiet` / `--no-heartbeat`
- [x] Fixture suite `tests/error-code-emission-fixtures.sh` covers: emitted code passes, dead code fires, allowlisted dead code suppressed, numeric-literal emission counts as emitted, defining-file-only use does NOT count, absent root is a tooling error
- [x] The three real dead codes are acknowledged with honest reasons rather than hidden — the allowlist is the ledger of an open question, not a silencer
- [x] `control.rs` annotates each of the three at its definition so the contract stops overstating enforcement; no constant renamed, no value changed
- [x] Load-bearing: removing an allowlist entry re-fires that code
- [x] `cargo test --workspace` green

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

bash tests/error-code-emission-fixtures.sh
bash scripts/check-error-code-emission.sh --quiet
bash scripts/check-error-code-docs.sh
bash scripts/run-guard-layer.sh --quiet
cargo test -p termlink-protocol --lib artifact_not_found

## RCA

**Symptom:** three of the 23 codes in `control.rs::error_code` had no emission site
anywhere — `SESSION_BUSY` (-32002), `MESSAGE_EXPIRED` (-32004),
`PROTOCOL_VERSION_TOO_OLD` (-32011) — while `docs/reports/T-005-message-protocol-design.md`
lists them in the protocol's error table as ordinary refusals. Investigating the
numeric-emission path then found something worse: `termlink-hub/src/artifact.rs`
emitted the bare literal `-32004` at three sites for "artifact not found", so one wire
code carried two incompatible meanings — an expired message and a missing artifact,
which demand opposite client responses.

**Root cause — two mechanisms.** (1) The taxonomy is a set of *assertions* ("this
system will refuse you under condition X") and nothing checked that any of them could
occur; a constant plus a doc line was sufficient to publish a refusal. (2) The named
taxonomy can be bypassed entirely by writing the number by hand, and
`check-error-code-docs.sh` — which verifies doc↔definition pairing — structurally
cannot see a literal inside a handler.

`PROTOCOL_VERSION_TOO_OLD` is the sharpest case and shows why ordinary coverage misses
this: `control.rs` ships `check_protocol_version()` plus a passing test
`check_protocol_version_rejects_when_declared_is_older`. The builder is covered; the
builder has zero callers. **Coverage of a builder says nothing about whether the
builder is called** — the T-2683 pattern (a guard nothing executes) in compiled Rust
rather than in shell.

**Why structurally allowed:** the previous review (T-2694) established that positive
capability claims had provers only 1-in-4; this is the same disease in the error
surface, which nobody had thought of as a set of claims at all. A refusal is an
assertion about behaviour exactly like a capability is.

**Prevention:** `check-error-code-emission.sh` flags any code with no emission site,
counts a bare numeric literal as an emission, strips comments first (prose about a code
is not a use of it), and excludes the defining file so a constant cannot count as its
own use — nor can a builder that sits beside it with no callers. It carries the
`# guard-layer: source` marker, so T-2684's runner and T-2686's per-commit CI job
execute it. The three known-dead codes are declared in
`.context/checks/error-code-emission-allowlist` with reasons, and annotated at their
definitions with ⚠️ **NOT ENFORCED** so a reader of the taxonomy is told. The collision
is fixed by `ARTIFACT_NOT_FOUND` (-32024), pinned by
`artifact_not_found_does_not_reuse_message_expired`.

**Found by the check's own fixtures:** fixture 8 named codes `A1/A2/A3` and revealed
the scanner's constant-name class was `[A-Z_]+`, silently skipping any code containing
a digit — the check would have reported clean on a taxonomy it never fully read.
Widened to `[A-Z0-9_]+`.

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

### 2026-08-14T08:25:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2699-nothing-detects-an-error-code-that-is-de.md
- **Context:** Initial task creation

### 2026-08-14T08:25:49Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
