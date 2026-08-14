---
id: T-2691
name: "whoami identity auto-resolution silently degrades on non-Linux"
description: >
  read_ppid_from_proc reads /proc/<pid>/stat with .ok()? on both the CLI (metadata.rs) and MCP (tools.rs whoami_helpers) surfaces. On macOS the ancestor chain collapses to [self] and whoami always reports ambiguous with every candidate, never signalling that auto-resolution is structurally impossible there (T-2690 F2/G2).

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
created: 2026-08-14T07:26:51Z
last_update: 2026-08-14T07:41:15Z
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

# T-2691: whoami identity auto-resolution silently degrades on non-Linux

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `whoami` distinguishes "auto-resolution is unavailable on this platform" from "you genuinely have several sessions" — the two currently render identically
- [x] The platform limitation is surfaced on BOTH surfaces: CLI (`metadata.rs`) and MCP (`tools.rs::whoami_helpers`), which carry the same duplicated logic
- [x] Human output names the actual remedy (`TERMLINK_SESSION_ID=<id>`), not the one the ambiguous path implies (pick a candidate) — the implied action does not work on a platform where the walk can never succeed
- [x] JSON output carries a machine-readable signal so an MCP consumer can branch on it, not just prose
- [x] Detection is a RUNTIME probe of `/proc` availability, not `#[cfg(target_os)]` — a cfg makes the fallback unreachable on Linux and therefore unprovable from this host
- [x] Unit tests prove both branches from Linux: probe-available yields the normal path, probe-unavailable yields the platform-limited path
- [x] The existing ambiguous-with-multiple-sessions path is unchanged when `/proc` IS available — this must not turn a working Linux flow into a warning
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

cargo test -p termlink --bins procfs
cargo test -p termlink-mcp --lib procfs
cargo build -p termlink -p termlink-mcp

## RCA

**Symptom:** on macOS, `termlink whoami` never auto-resolves the caller's session.
It always prints "Multiple candidate sessions on this hub — which one are you?" with
the full list, even when exactly one obvious ancestor session exists — identical
output to a genuine multi-session ambiguity.

**Root cause:** the PID-ancestor fallback (T-1303) resolves identity by parsing
`/proc/<pid>/stat`. `read_ppid_from_proc` swallows the failure with `.ok()?`, so on a
host with no procfs the walk breaks out immediately and `walk_ancestor_pids` returns
`vec![start]` — the termlink process's own PID, which is never a registered session's
PID. Execution falls through to the ambiguous branch. The same logic exists twice: the
CLI (`metadata.rs`) and a deliberate duplicate in the MCP crate
(`tools.rs::whoami_helpers`), so both surfaces carried the defect identically.

**Why structurally allowed:** three things at once. (1) `.ok()?` converts a *platform
capability* question into an *absent value*, which is indistinguishable downstream
from "no match found" — the type system actively erased the distinction. (2) The
function's own doc comment listed `non-Linux` among its failure modes, so the
limitation was known at authoring time and simply never surfaced to the user. (3) No
CI job has ever run on macOS (T-2692), so nothing could have observed the behaviour;
and no check existed for platform-locked primitives (T-2693), so nothing could have
observed the *cause* either. Directive #4 Portability had never been the subject of a
review at all — the blind spot was in the review series, not just the code.

**Prevention:** two layers. `procfs_available()` is a RUNTIME probe (deliberately not
`#[cfg(target_os)]`, which would make the non-Linux branch unreachable and therefore
unprovable from a Linux host — exactly how the original survived), and both surfaces
now branch on it to name the limitation and the remedy that actually works
(`TERMLINK_SESSION_ID`), with `auto_resolution: "unavailable-no-procfs"` in JSON so an
MCP consumer can branch without parsing prose. Structurally, `check-platform-lock.sh`
(T-2693) now flags any new `/proc`, `/sys`, or Linux-only-command dependency in the
product crates and runs inside the T-2684 guard layer. On its first run it caught a
defect in this very task's own test — an assertion that `/proc` exists, which would
have failed on the macOS runner T-2692 added.

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

### 2026-08-14T07:26:51Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2691-whoami-identity-auto-resolution-silently.md
- **Context:** Initial task creation

### 2026-08-14T07:27:24Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
