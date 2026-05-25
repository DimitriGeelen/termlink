---
id: T-1798
name: "Fix cli_tests event_watch stack-overflow aborting bin test run"
description: >
  cli::cli_tests::event_watch_without_hub_accepts_targets stack-overflows (SIGABRT) when the bin test suite runs, aborting the whole process before most tests report. Pre-existing; discovered during T-1797. Likely unbounded recursion in the event-watch-without-hub path or the test's setup. Investigate the recursion, add a depth/termination guard, confirm the full suite runs to completion.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/remote.rs]
related_tasks: []
created: 2026-05-22T07:07:04Z
last_update: 2026-05-25T15:02:50Z
date_finished: 2026-05-25T15:02:50Z
---

# T-1798: Fix cli_tests event_watch stack-overflow aborting bin test run

## Context

`cli::cli_tests::event_watch_without_hub_accepts_targets` stack-overflows
(SIGABRT) during the bin test run, aborting the process before most tests
report. Pre-existing; surfaced during T-1797. Restoring a green,
run-to-completion bin test suite is foundational — it gates every TermLink
release.

## Acceptance Criteria

### Agent
- [x] Root cause of the stack overflow identified and documented in the RCA section (the specific recursion/setup gap, not "the code was wrong") — stack-size, not recursion: see RCA
- [x] Fix adds a termination/bound (or corrects the faulty path) so `event_watch_without_hub_accepts_targets` runs to completion without SIGABRT — `.cargo/config.toml` sets `RUST_MIN_STACK=16 MiB` for cargo-invoked test threads
- [x] The previously-crashing test passes in isolation — `cargo test -p termlink --bin termlink event_watch_without_hub_accepts_targets` → 1 passed (via config, no inline env)
- [x] The full bin (`--bin termlink`) test binary runs to completion with no process abort — 782 passed, 1 failed, no SIGABRT/overflow (finished in 30s)
- [x] `cargo check -p termlink` passes
- [x] No unrelated test regressed — the single failure (`manifest::tests::test_is_git_repo_on_temp_dir`) is pre-existing + environment-dependent (`/tmp/.git` present), was masked by the prior SIGABRT, and is filed as T-1801. Not a regression of this change (stack env cannot affect git detection; confirmed by isolation run).

## Verification

cargo check -p termlink
cargo test -p termlink --bin termlink event_watch_without_hub_accepts_targets

## RCA

**Symptom:** `cli::cli_tests::event_watch_without_hub_accepts_targets` overflows its
stack and SIGABRTs (signal 6) when the `--bin termlink` unittests run, aborting the
whole process before most of the 783 tests report.

**Root cause:** Stack size, NOT infinite recursion. The test does nothing but
`Cli::try_parse_from(["termlink","event","watch","alpha","beta"])`. clap builds the
*entire* command tree on every parse, and the termlink CLI is large (22 top-level
subcommands, hundreds of args). That construction's peak stack usage exceeds the
**default 2 MiB stack of a Rust libtest worker thread**. Proof: the test passes
unchanged under `RUST_MIN_STACK=8388608` (8 MiB) and larger; it overflows at the 2 MiB
default. The real `termlink` binary parses on the main thread (8 MiB), so production
has never been affected — this is strictly a test-harness/main-thread stack mismatch.

**Why structurally allowed:** Nothing pinned the test-thread stack size, so it silently
rode the 2 MiB libtest default. As the CLI grew (clap surface expanded over many tasks),
per-parse stack usage crept up until it crossed 2 MiB — with no gate or signal, because
the only place it manifests is a worker thread, never the 8 MiB main thread the binary
actually uses. A single SIGABRT then masks the rest of the suite (and other latent
failures, e.g. T-1801).

**Prevention:** `.cargo/config.toml` `[env] RUST_MIN_STACK = "16777216"` gives all
cargo-invoked processes a 16 MiB thread-stack floor (8 MiB margin over the proven 8 MiB
need; lazily committed, so negligible real cost). Released binaries are not launched via
cargo, so runtime behaviour is unchanged. The fix is durable (committed config, not an
operator-remembered env var), and the now-completing suite surfaces future latent
failures instead of swallowing them in the abort.

Rejected alternatives: shrinking the clap command tree (would gut a legitimately large
CLI for a test-only issue); a per-test big-stack thread wrapper (fixes one test, leaves
every other parse-test one CLI-growth-increment from the same SIGABRT).

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

### 2026-05-22T07:07:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1798-fix-clitests-eventwatch-stack-overflow-a.md
- **Context:** Initial task creation

### 2026-05-25T14:56:13Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.4)

- **Scan ID:** R-1dc934ac
- **Timestamp:** 2026-05-25T15:03:18Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#2 (Agent)** — Fix adds a termination/bound (or corrects the faulty path) so `event_watch_without_hub_accepts_targets` runs to completion without SIGABRT — `.cargo/config.toml` sets `RUST_MIN_STACK=16 MiB` for cargo
  - **AC-verify-mismatch** (narrow, heuristic) — `path=cargo/config.toml in: Fix adds a termination/bound (or corrects the faulty path) so `event_watch_without_hub_accepts_targets` runs to completion without SIGABRT — `.cargo/c`

### 2026-05-25T15:02:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
