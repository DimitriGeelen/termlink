# T-3093 R3S3 handback (procAsFit, round 3 of 4) — IN PROGRESS

**Status:** IN PROGRESS — skeleton written at session start per carried fix #3.

## Orientation (done)

- Re-read run record + `git log --oneline -15` at start (carried fix #2).
- Context at start: 174,762 tokens (~21%) via `checkpoint.sh status`.
- Confirmed focus.yaml already points at T-3093 (session S-2026-0919-2056, the long-running
  session spanning this whole orchestrated run).
- Noted ambient uncommitted drift in the working tree at start: T-2753 (operator's own GO
  decision via Watchtower, commit b53bee358) and T-3139 (Tier-0 defect work, commit 8afd529d8)
  both have small residual uncommitted diffs (timestamps, reviewer-verdict sections) — this is
  NOT my work in progress; it is ambient session-state drift of the same class R3S2 already
  flagged and closed with a residual commit. Will fold into my own closing commit unless it
  turns out to be someone else's live in-flight work.
- NEW since R3S2 dispatched: T-3139 (2 commits) landed — a Tier-0 defect fix/detection task,
  outside this run's arc scope, executed independently. Also observed but not yet reconciled.

## Selection — unit of work 1

- **Objective:** charter verb "exchange durable messages" — the notify-rail's mailbox-to-prompt
  delivery capability (arc-011's headline mechanic).
- **Arc:** arc-011 ("Agent-to-agent message delivery: mailbox to prompt"), status in-progress,
  UNBLOCKED as of the operator's 2026-09-24 GO on T-3075/T-3076 (per state_changes_during_run
  in the run record) and R2S3's re-pointing of slices S1/S2/S12 at new build tasks. Preferred
  over continuing arc-008 (audit-remediation housekeeping — R3S2 already worked its one cycle
  of genuine capacity this round) and arc-009 (value-review execution — archival/cleanup, lower
  order than shipping the founding message-delivery capability) per the mandate's "prefer an
  arc already in flight... unless blocked" rule: arc-011 is in-flight and no longer blocked,
  and its completion (a working sender->receiver->inject round trip) moves the project's core
  charter objective furthest of the three live arcs.
- **Task:** T-3134 ("artifact CLI verbs: termlink artifact put/get"), arc-011 slice S2. Only
  task in arc-011 with a BVP quadrant already computed and no unresolved Sovereign blocker.
- **Quadrant:** Q1 (hv-lc) — `bvp_scores_proposed` D1=4 D2=0 D3=3 D4=2, `cost_estimate_proposed`
  blast_radius=3 tier=2 effort=8 (small, single-session, scoped by T-3076's own Scope Fence
  to "two thin CLI wrappers, no protocol/hub changes"). T-3135 (S1+S12, sidecar API + portable
  respawn) is Q2 (hv-hc) — deferred to "work second" per the mandate, and its portable-respawn
  build (SQ-8) is exactly the kind of larger, riskier unit that should not be started with an
  unproven amount of remaining budget in a single dispatch.
- **Why this one over the next candidate:** T-3134 had only placeholder ACs (`[First
  criterion]`/`[Second criterion]`) — captured but not build-ready. Per "Scored before started"
  it already has a BVP score; per Task Sizing and the Pickup rule, real ACs are written before
  any source file is touched (below), rather than executing against placeholders.

## Work log

### Unit 1 — T-3134: `termlink artifact put`/`termlink artifact get`

Read `crates/termlink-session/src/artifact.rs` (`send_artifact_via_client`,
`download_artifact_via_client` signatures + the 3 existing call sites in file.rs/remote.rs/
tools.rs) and T-3076's completed inception (IW-3: `--expected-sha256` MANDATORY on `get`,
confirmed via `.tasks/completed/T-3076-*.md`) to scope precisely before writing ACs.

## Stop condition

TODO.

## Commits this step

TODO.
