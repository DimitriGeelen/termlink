---
id: T-2735
name: "Inherited TERMLINK_SESSION_ID trusted without ownership check — short-circuits the T-1303 PID-walk (herdr item 6)"
description: >
  Inherited TERMLINK_SESSION_ID trusted without ownership check — short-circuits the T-1303 PID-walk (herdr item 6)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/metadata.rs, crates/termlink-mcp/src/tools.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-15T12:16:23Z
last_update: 2026-08-15T12:47:06Z
date_finished: 2026-08-15T12:47:06Z
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

# T-2735: Inherited TERMLINK_SESSION_ID trusted without ownership check — short-circuits the T-1303 PID-walk (herdr item 6)

## Context

Herdr adoption backlog item 6 (rank 6), plus a second defect in the same
handler found while reading it.

`TERMLINK_SESSION_ID` is seeded into a spawned session's shell
(`session.rs:277`) and is then inherited by **every descendant of that shell**.
So the variable does not say "this process belongs to session X" — only that
*some ancestor once did*. `whoami` consumed it at `metadata.rs:538`
(`session_hint.or(env_hint).or(name_hint)`) and short-circuited **ahead of** the
T-1303 PID-ancestor walk, returning the claimed identity with no check and full
confidence. A stale or foreign value therefore produced a confident **wrong
answer to the one question the command exists to answer**. Same shape at
`tools.rs:11830` on the MCP surface.

**Second defect, found by reading the handler.** The `termlink_whoami` tool
description advertised the chain as `session_hint → name_hint → env → PID-walk`
while the code is `session_hint.or(env_hint).or(name_hint)` — **env beats
name_hint**. On the MCP surface the description *is* the contract: an agent
choosing between the two parameters was reading an order the code did not
honour. Fixed here rather than filed separately, because it is the same handler
and the same reading pass.

**Scope boundary, stated.** This does NOT change which source wins. The env var
still resolves the query exactly where it did before, and no call that works
today starts failing. The reason is that the variable is not a security
boundary — abusing it already requires a process running inside the session —
so escalating to a refusal would cost working setups more than it protects.
What was missing was not enforcement but *legibility*: the answer never said
where it came from, so a wrong one was indistinguishable from a right one.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] When identity resolves from the **inherited env var** (not an explicit
      `--session` / `session_hint`) and the PID-ancestor walk is available, the
      env claim is cross-checked against the walk instead of being trusted blind
      (`check_env_claim`, CLI + MCP)
- [x] A disagreement between the env claim and the PID-walk is **surfaced, not
      silently resolved**: `resolved_via: "env"` names the source,
      `env_claim_verified` carries the verdict, and `env_claim_conflict` names
      the session that actually owns the process (Directive #2)
- [x] Resolution ORDER is unchanged — env still wins where it wins today. Only
      the winner's PROVENANCE is now carried forward
- [x] The `termlink_whoami` MCP tool description is corrected to
      `session_hint → env → name_hint → PID-walk`, matching the code, and is
      pinned by `mcp_whoami_description_states_the_order_the_code_implements`
- [x] CLI (`metadata.rs`) and MCP (`tools.rs`) fixed on the same commit pair,
      per the T-2687 `parity_topics` lesson
- [x] A test proves the cross-check fires when a stale/foreign
      `TERMLINK_SESSION_ID` names a live session that does NOT own the caller's
      ancestor chain (`env_claim_conflicting_with_ancestor_walk_is_reported`,
      `mcp_env_claim_conflict_is_reported`)
- [x] A test proves the quiet path stays quiet
      (`env_claim_owning_the_ancestor_chain_stays_quiet`,
      `no_registered_ancestor_is_not_evidence_against_the_claim`) — PL-219
- [x] New tests demonstrated load-bearing by temp-revert: reverting
      `check_env_claim` to unconditional trust failed **5** CLI tests; reverting
      the description order failed the MCP description test. Both restored to a
      byte-identical tree
- [x] `cargo test -p termlink --bins commands::metadata::tests` (23 passed) and
      `cargo test -p termlink-mcp --lib` (915 passed) pass
- [x] `bash scripts/run-guard-layer.sh` stays clean (27/27)

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

# `termlink` is a bin-only package — `--lib` errors with "no library targets".
cargo test -p termlink --bins commands::metadata::tests
cargo test -p termlink-mcp --lib
bash scripts/run-guard-layer.sh

## RCA

**Symptom.** `termlink whoami` (and `termlink_whoami`) could confidently report
the wrong session identity. Not an error, not an ambiguity — a clean, complete
identity card naming a session the calling process does not belong to.

**Root cause.** `TERMLINK_SESSION_ID` is an *inherited* value, but was consumed
as if it were an *asserted* one. It is seeded into a spawned session's shell and
inherited by every descendant, so its presence proves only that some ancestor
once belonged to that session. The handler resolved it ahead of the T-1303
PID-ancestor walk — the one mechanism that could have contradicted it — and
returned immediately, so the contradicting evidence was never gathered.

**Why structurally allowed.** Two reinforcing reasons. First, the resolution
chain was written as a single `or`-chain
(`session_hint.or(env_hint).or(name_hint)`), which discards *provenance* by
construction: after that line, nothing downstream can tell whether the answer
came from a flag the caller typed or a variable they inherited without knowing.
The information needed to distinguish trustworthy from inherited was destroyed
one line before the point where it mattered. Second, the tool description on the
MCP surface stated a *different* chain order from the code, so the written
contract could not be used to audit the behaviour — reading the docs would have
confirmed a chain the code did not implement.

**Prevention.** Provenance is now carried through the chain (`from_env`) rather
than discarded, so the cross-check is possible at all; `check_env_claim` is a
pure function over (claim, sessions, ancestors, procfs) with all four branches
unit-tested on both surfaces; the verdict is tri-state so "could not check" can
never render as "checked and fine" (the T-2691 conflation); and the MCP
description is pinned by a test that fails if it drifts from the code again.

**Class note (8th instance).** Same shape as the seven before it — *a guard,
test, or report whose verdict rests on an assumption about its input that no
longer holds*. Here the assumption was "this env var was set by whoever is
asking". The MCP description defect is a close relative: a contract asserting an
order the code does not implement. The recurrence rate across three sessions
(T-2680, T-2709, T-2726, T-2729, T-2731, T-2732, T-2734, and this) is the
argument for registering it as a concern rather than fixing instances — a
sovereignty call, flagged and not taken.

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

### 2026-08-15T12:16:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2735-inherited-termlinksessionid-trusted-with.md
- **Context:** Initial task creation

### 2026-08-15T12:47:06Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
