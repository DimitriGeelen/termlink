---
id: T-2571
name: "GUARD: strict-star no-spoke-to-spoke-dial invariant (charter load-bearing-noun)"
description: >
  Strict-star topology-invariant guard: verify no spoke-to-spoke direct dial path exists; add a load-bearing test if safe-only-by-absence. Filed from T-2468 purpose-review round.

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
created: 2026-08-09T14:37:55Z
last_update: 2026-08-09T14:37:55Z
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

# T-2571: GUARD: strict-star no-spoke-to-spoke-dial invariant (charter load-bearing-noun)

## Context

Filed from T-2468 purpose-review round. The charter's load-bearing noun
"Hub-mediated — a strict star: spokes never talk peer-to-peer, the hub mediates
all coordination" is asserted but may be safe only by ABSENCE-of-feature (like
non-goal #1, which T-2569 addresses). This task first VERIFIES whether the
strict-star invariant is enforced/guarded/absence-only, then — if it is a
buildable-small guard — adds a load-bearing test asserting the client message/RPC
path never opens a direct spoke-to-spoke socket (all coordination routes through a
hub address). If the investigation shows an existing direct-dial path that bends
the claim (e.g. remote_exec dialing an arbitrary host:port), that is a
charter-accuracy finding to surface, not silently build around.

## Acceptance Criteria

### Agent
- [x] Investigation recorded: the strict-star invariant is **guarded-by-construction
      + covered by an existing load-bearing test** — NOT safe-only-by-absence.
      Evidence: every client connect targets a hub address
      (`remote.rs::connect_remote_hub` takes `conn.hub`, a validated `hubs.toml`
      profile; all `remote_*` handlers route through it); cross-host peer contact
      resolves to the peer's declared HOME HUB via `agent.rs::resolve_home_hub`,
      which **deliberately excludes** the peer's process socket
      (`metadata.observed_addr` = host + ephemeral port, T-2297) from routing
      (`agent.rs:744-748`). The guard test is `resolve_home_hub_precedence`
      (`agent.rs:4097`), asserting `observed_addr`-alone → `None` (no peer-process
      routing). This is materially stronger than non-goal #1 federation (T-2569,
      no guard).
- [x] No new low-value test built (AC anticipated this branch). Instead the real
      gap was **discoverability**: the charter noun and its enforcing test were not
      linked. Closed by (a) a note in `docs/ARCHITECTURE.md § Hub Architecture`
      tying the strict-star noun to `resolve_home_hub`'s observed_addr-exclusion +
      its guard test, and (b) a `LOAD-BEARING for the charter strict-star noun`
      doc-comment on the test so a future editor cannot weaken it unknowingly.
      Load-bearing property already proven by the test's own assertion: routing to
      `observed_addr` makes it fail.
- [x] Verdict recorded (see Decisions): guarantee is enforced-by-construction +
      tested; no human architectural input needed; no FILE conversion.

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

cargo test -p termlink --bins resolve_home_hub_precedence 2>&1 | grep -q "1 passed"
grep -qi "strict-star" docs/ARCHITECTURE.md
grep -q "LOAD-BEARING for the charter strict-star noun" crates/termlink-cli/src/commands/agent.rs

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

### 2026-08-09 — strict-star: document-the-guard, do not build a redundant test
- **Chose:** Record the verdict (guaranteed-by-construction + already tested) and
  close the discoverability gap with a doc pointer + a test doc-comment, rather
  than authoring a new "no spoke-to-spoke dial" test.
- **Why:** The routing layer already refuses to use a peer's process socket
  (`resolve_home_hub` excludes `observed_addr`), and `resolve_home_hub_precedence`
  already asserts exactly that. A new test would be redundant and non-load-bearing
  (nothing to newly protect). The real gap was that the charter noun and its guard
  were not linked — a future editor could weaken the guard unknowingly. Linking
  them (both directions) is the antifragile, in-authority fix.
- **Rejected:** (a) Building a fresh topology test — redundant, not load-bearing.
  (b) A HubAddr/SpokeAddr newtype refactor to make the star type-enforced — larger,
  needs human architectural input, and unwarranted given the guarantee already
  holds by construction + test. (c) Filing as needs-human — the verdict is clear;
  no decision is owed to the human.
- **Contrast:** non-goal #1 (inter-hub federation) IS safe-only-by-absence with no
  guard — that is the genuine gap, tracked by T-2569. Strict-star is not in that
  class.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-09T14:37:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2571-guard-strict-star-no-spoke-to-spoke-dial.md
- **Context:** Initial task creation
