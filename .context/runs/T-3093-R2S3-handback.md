# T-3093 R2S3 handback (procAsFit, round 2 of 4) — IN PROGRESS

**Status:** DRAFT skeleton, being filled as work proceeds. Do not trust this file as complete
until the "Status: COMPLETE" line appears in place of this one.

## Orientation (Step 0)
- Re-read run record + `git log --oneline -15` at start of this step (carried fix #2). Confirmed:
  T-3075 GO and T-3076 GO landed via Watchtower during R2S1/R2S2. arc-011 is no longer blocked.
  check-arc-slice-drift.sh fires (rc=1) on S1/S2/S12 — inceptions went GO, nothing shipped, slices
  are genuinely unbuilt. R2S2 explicitly left task creation for this step.
- T-3133 left at `started-work` by R2S2 (its underlying fix — a commit — already landed; only the
  status transition + Evolution entry remain).
- T-3130 (owner-field clobber bug, live and reproducible) flagged by R2S2 as worth prioritizing early.

## Unit of work 1 — housekeeping close-out (arc-008)

**Selection:** Objective: repo hygiene / task-register truthfulness (arc-008's charter).
Arc: arc-008 (in-progress, reused, not re-opened). Task: T-3133. Quadrant: Q1 (trivial —
the fix already existed as R2S2's own commit; only the status/Evolution bookkeeping
remained). Why this one first: cheapest possible unit, clears an ambiguous
`started-work`-with-nothing-left-to-do state before touching anything bigger.

- Added the required `## Evolution` entry, transitioned T-3133 `started-work` →
  `work-completed` via `fw task update`. Verified: episodic generated, moved to
  `completed/`. Cost: ~1 tool round, far under any estimate (bookkeeping only).

## Unit of work 2 — T-3130 bisect (arc-008, INVESTIGATE)

**Selection:** Arc: arc-008. Task: T-3130 (owner-field clobber, live reproducible bug,
explicitly flagged by R2S2 as "worth prioritizing early"). Quadrant: unscored for
build effort (it's an INVESTIGATE-shaped task; BVP scores D1=4/D3=3/D4=2, no D2/F
signal) but high enough signal (a live data-integrity bug across every task file this
whole run touches) to justify a bounded bisect attempt before parking it further.

- 3 bisect attempts via scratch fixtures matching T-3129's exact shape (folded
  description, same tags/owner, same transitions): (1) `captured→started-work`,
  (2) same on a second fresh fixture, (3) full `→work-completed` finalize path
  (AC-ticking, Evolution, git-mv, episodic-gen). **None reproduced the clobber.**
  Recorded as a negative result on the task (rules out the two leading candidates:
  auto bvp-estimate rewrite, tag-insertion regex; also rules out the finalize path
  in isolation). Per the mandate's "three attempts is context burned, not progress",
  stopped and left it **captured/parked** with the negative result on file rather
  than burning further budget chasing a non-reproducing bug. Scratch fixtures
  (T-3134-then-freed, T-9999) deleted, not committed — confirmed clean.

## Unit of work 3 — arc-011 slice re-pointing (the run record's flagged consequence)

**Selection:** Objective: charter-core "exchange durable messages" (arc-011's own
headline mechanic — agent-to-agent delivery, mailbox to prompt). Arc: arc-011 —
**unblocked this round** by the operator's T-3075/T-3076 GO decisions (the state
change flagged at the top of this step's prompt), and per the mandate ("prefer an
arc already in flight... unless blocked") this now outranks arc-008 the moment it
has real Q1/Q2 work, which is exactly what re-pointing produces. Task: none yet —
this unit's job was creating the right tasks. Quadrant: n/a (task-creation step).

- Verified `check-arc-slice-drift.sh` fired exactly as the run record predicted: S1,
  S2, S12 all read `unbuilt` against now-COMPLETED inception tasks T-3075/T-3076.
- Read both inception tasks in full. Confirmed via direct source read (not trusted
  from the task file alone) that the underlying claims still hold: no `termlink
  artifact` CLI subcommand exists (`grep -rn download_artifact_via_client
  crates/termlink-cli/src` → only `file.rs`); `ARTIFACT_PUT`/`ARTIFACT_GET` are real
  routed protocol methods (`control.rs`, `router.rs`, `artifact.rs` all present and
  matching cited line content — the task's own Recommendation Verdict had flagged
  these as CONTRADICTED/file-not-found, which was a line-number-drift false
  negative in that reviewer pass, not a real problem with the claim).
- Created two build tasks per CLAUDE.md:3098 ("after a GO, create separate build
  tasks and re-point the slices"):
  - **T-3134** — `termlink artifact put/get` CLI verbs (arc-011 S2). Scored via
    `fw bvp estimate` + estimator `cost-one`: blast_radius=3, tier=2, effort=8 →
    **hv-lc (Q1)**.
  - **T-3135** — Sidecar API local-control surface + portable respawn supervisor
    (arc-011 S1 + S12). Carries the operator's SQ-8 ruling verbatim in its
    description (systemd-only respawn REJECTED, portable fallback REQUIRED) so
    that constraint cannot be lost by whoever picks it up next. Also documents
    that S12's remaining blocker (framework-agent-systemd's systemd
    `--allowed-commands`) is a **different project's config** (T-559 boundary) and
    is NOT something this build task can close — flagged as a cross-project
    Sovereign item, not decided or routed around here. Scored: blast_radius=5,
    tier=2, effort=8 → **hv-hc (Q2)**.
- Re-pointed all three slices (S1→T-3135, S2→T-3134, S12→T-3135) in
  `.context/arcs/arc-011.yaml`, with an explicit `RE-POINTED 2026-09-25` note on
  each explaining WHY (the GO was on the analysis, not a build) rather than
  silently rewriting history. Did **not** mark any slice `built` — nothing shipped.
  Verified: `python3 -c "import yaml; yaml.safe_load(...)"` parses clean;
  `check-arc-slice-drift.sh` now reports **clean — 0 firing**.

(next: execute the Q1 task T-3134; T-3135 (Q2) scoping/partial-design only, given
budget and the "one lock at a time" / "solid not fast" operator rulings — full
implementation of a portable respawn daemon is not something to rush inside this
step's remaining budget)
