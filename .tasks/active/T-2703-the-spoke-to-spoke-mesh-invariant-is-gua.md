---
id: T-2703
name: "The spoke-to-spoke mesh invariant is guarded by nothing (T-2569 guards a different edge)"
description: >
  The architecture doc's decisive invariant is 'spokes never connect to one another'. T-2569's tripwire scans only the hub crate and forbids hub-to-hub federation — a different edge. termlink-session ships a generic client that connects to any unix path or TCP host:port, so a spoke-to-spoke mesh could be added and no test would fail (T-2702 F1).

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
created: 2026-08-14T11:27:41Z
last_update: 2026-08-14T11:48:59Z
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

# T-2703: The spoke-to-spoke mesh invariant is guarded by nothing (T-2569 guards a different edge)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] A tripwire test guards the **spoke↔spoke** edge — the invariant T-2569 does NOT cover, since it scans only the hub crate and forbids hub↔hub federation
- [x] It enumerates every outbound-connection construction site in `termlink-session/src`, pinning the count and each site's legitimate purpose, so a NEW outbound path fails the build until deliberately acknowledged (the T-2569 `EXPECTED_OUTBOUND_SITES` idiom)
- [x] Connecting to the **hub** stays legal — the star requires it; the test must distinguish "spoke → hub" from "spoke → spoke", not ban outbound connections
- [x] Connecting to a **local session socket from an operator tool** stays legal — §3 forbids agents meshing with each other, not a CLI reaching a local session; banning that would be a false positive that makes the guard unusable
- [x] The test explains in-file WHY the mesh is forbidden, citing §3's decisive argument (a mesh distributes fragility across N² links with no central durable replay and silent partial-partition divergence), so a future reader knows the cost of "just add a direct channel"
- [x] It documents its own residual: what shape of spoke↔spoke connection it would still miss
- [x] Load-bearing: adding a simulated peer-to-peer connection site makes it fail; removing it returns to green
- [x] Comment/string mentions of a socket path do not trip it — prose about connecting is not connecting
- [x] `cargo test -p termlink-session` green, and `cargo test --workspace` green

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

cargo test -p termlink-session --test no_spoke_mesh_tripwire
cargo test -p termlink-session

## RCA

**Symptom:** the architecture doc the charter calls authoritative declares five
invariants that "must not be violated". The first — *"Strict star; spokes never connect
to one another"*, which §3 defends over forty lines and calls decisive — was enforced by
nothing.

**Root cause:** `no_federation_tripwire.rs` (T-2569) *looks* like it covers this, and
was cited in T-2678 as closing charter non-goal #1. It does — but non-goal #1 is
hub↔hub **federation**, and the invariant here is spoke↔spoke **mesh**. Two different
edges of the same topology. The tripwire scans `CARGO_MANIFEST_DIR/src` of the *hub*
crate only, so the entire client side was unguarded, while `termlink-session/src`
ships a generic RPC client that dials any unix path or TCP host:port with nothing
constraining the target.

**Why structurally allowed:** the two edges share a vocabulary ("strict star", "no
peer-to-peer"), so a guard on one reads as a guard on both. T-2678 built the matrix of
charter **non-goals**; nobody had built one for the architecture document's
**invariants**, and the overlap in wording made the gap invisible from either side.

**Prevention:** `crates/termlink-session/tests/no_spoke_mesh_tripwire.rs` pins the real
structural property — production connections are confined to four enumerated
transport/probe modules (`client.rs` 5, `transport.rs` 3, `tofu.rs` 1,
`ws_consumer.rs` 1), each read and confirmed to dial the hub, a local session control
plane, or a hub-router proxy. A mesh appears either as a new site inside one (count
check) or, far more likely, as a socket opened in a fifth module (containment check).
Proven load-bearing: a simulated peer-to-peer connect in `discovery.rs` fails it;
removing it returns to green. Comment mentions do not trip it.

**Two corrections made during the build, both kept visible:**

1. The tripwire was first written asserting all connects live in `client.rs`. That
   premise came from a `grep` truncated at ten results and was simply false — the check
   failed on its own first run against `transport.rs`, `tofu.rs` and `ws_consumer.rs`.
   The corrected enumeration is a stronger invariant than the guess, because it is the
   actual answer to "what can a spoke dial".
2. The scanner initially removed test code by truncating at the first `#[cfg(test)]`.
   `discovery.rs` has its test module at line 89 of 176, so that discarded **half the
   file unscanned** — and the load-bearing probe appended at the end did not fire.
   Replaced with brace-counting that skips only the guarded item's body. A guard that
   silently reads less than it claims is the exact failure mode this review series
   keeps finding elsewhere (T-2680's scope over-report, T-2699's comment-as-emission
   and digit-blind regex); catching it in my own guard is the same class, not a
   different one.

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

### 2026-08-14T11:27:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2703-the-spoke-to-spoke-mesh-invariant-is-gua.md
- **Context:** Initial task creation

### 2026-08-14T11:28:05Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
