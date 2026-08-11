---
id: T-2605
name: "Over-length/whitespace client_msg_id silently downgrades exactly-once to at-least-once (filter-to-None, no error)"
description: >
  Verb-2 hunt F2: channel.rs:722 filters invalid client_msg_id to None with no INVALID_PARAMS

status: captured
workflow_type: build
owner: agent
horizon: next
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-11T10:13:18Z
last_update: 2026-08-11T10:13:18Z
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

# T-2605: Over-length/whitespace client_msg_id silently downgrades exactly-once to at-least-once (filter-to-None, no error)

## Context

Found by the T-2468 verb-2 ("exchange durable messages") adversarial hunt — finding F2
(MEDIUM), verified in code.

The hub post handler `filter`s a present-but-invalid `client_msg_id` to `None`, silently
proceeding with NO dedupe and NO error (`crates/termlink-hub/src/channel.rs:718-722`):
```rust
let client_msg_id = params.get("client_msg_id").and_then(|v| v.as_str())
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty() && s.len() <= 128);   // >128 or all-whitespace → None, silently
```
A caller who supplied an idempotency token believes it is protected, but a token >128
chars (or all-whitespace) is dropped and the post proceeds unprotected.

**Scenario:** An MCP `termlink_channel_post` / raw-RPC caller supplies a 200-char
namespaced id; the original commits at N, the ack is lost, the retry replays → the hub
sees no dedupe key → **silent double-append**. (The CLI mints 32-char ids so it is masked
there, but the hub accepts arbitrary callers.) **Secondary:** `trim()` makes `"abc "` and
`"abc"` collide — two intentionally-distinct posts differing only in trailing space would
have the second **silently dropped** (returns the first's cached offset, never appends).

**Violates:** Reliability directive #2 ("no silent failures") — a request that *looks*
idempotency-protected silently isn't.

**Why file (not build autonomously):** it is a wire-contract change (present-but-invalid
input goes from accepted→rejected) plus a `trim()` semantics decision — a deliberate
contract call, though a small one. Cleanly unit-testable once the decision is made.
Note: sibling of T-2606 (F1/F3 durable exactly-once) — F2 lets a caller lose even the
in-memory protection; coordinate the contract wording with T-2606.

## Acceptance Criteria

### Agent
- [ ] A decision is recorded (see Decisions) on how the hub handles a present-but-invalid
      `client_msg_id` (over-length / empty-after-trim) and on the `trim()` collision.
- [ ] Present-but-invalid `client_msg_id` no longer silently downgrades to no-dedupe:
      the hub returns a clear INVALID_PARAMS (-32602) naming the constraint (≤128 chars,
      non-empty), instead of `filter`-ing to `None`. An ABSENT `client_msg_id` still
      proceeds normally (dedupe is opt-in — only present-but-invalid is an error).
- [ ] The `trim()` collision is resolved per the decision (either documented as
      intentional normalization, or trailing/leading whitespace preserved so distinct ids
      stay distinct).
- [ ] A load-bearing unit test proves it: a post with a 200-char `client_msg_id` returns
      INVALID_PARAMS (not `ok:true`); an absent id still succeeds. Prove load-bearing by
      temp-reverting to the `filter`-to-`None` form and confirming the test FAILS.
- [ ] `cargo test -p termlink-hub` passes.

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

### OPEN — reject vs. accept-and-hash for an over-length client_msg_id?

- **Option A — loud-reject (recommended).** Present-but-invalid → INVALID_PARAMS naming
  the ≤128 / non-empty constraint. Honors "no silent failures"; forces the caller to fix
  the token. The comment at channel.rs:716 already says "longer payloads should hash
  before submission" — this makes that guidance enforced instead of silently assumed.
- **Option B — hash over-length ids hub-side.** Deterministically hash a >128-char id to
  a fixed-width key so the caller's intent (dedupe on this string) is honored. More
  forgiving, but hides a caller contract violation and adds a hashing dependency. Weaker.

### OPEN — trim() collision

Does `"abc "` == `"abc"`? Current `trim()` says yes. Either (a) document trimming as
intentional normalization, or (b) drop the `trim()` so leading/trailing whitespace keeps
distinct ids distinct (safer against accidental drop of a legitimately-distinct post).
Lean (b) unless there is a concrete reason callers pad ids.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-11T10:13:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2605-over-lengthwhitespace-clientmsgid-silent.md
- **Context:** Initial task creation
