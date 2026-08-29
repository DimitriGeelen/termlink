# T-2849 — Finished-and-waiting triage, measured at the authority

**Measured:** 2026-08-29 · **Tree:** `/opt/termlink` @ main · **Supersedes:** the
T-2848 report written on `worktree-charter-review-2026-0814` (never landed here).

## Why this report exists at all

T-2848 measured this population from a worktree branch ~262 commits behind main and
reported **75**. That report was never merged, so at the authority it does not exist —
its headline number survives only as a quotation in task bodies. PL-368 names the
hazard: a tree N commits behind main is a systemic false-positive generator, and the
remedy is to re-measure at the authority rather than carry the replica's number
forward. This report is that re-measurement.

## The count, and why every prior number was wrong

| measurement | where | total | note |
|---|---|---|---|
| T-2848 | worktree (262 behind) | 75 | stale corpus replica |
| T-2849 first pass | authority | 79 | earlier in the queue's life |
| carried forward in session notes | — | 82 | never re-derived |
| **this report** | **authority, 2026-08-29** | **105 / 65** | see definitions |

Two populations, deliberately kept apart because they answer different questions:

- **105** tasks in `.tasks/active/` carry at least one *open* `### Human` acceptance
  criterion (128 open Human ACs in total). This is the full human-gated surface.
- **65** of those are **finished-and-waiting** in the strict sense: `status:
  work-completed`, agent work done, sitting in `active/` solely because a human box is
  unticked. All 65 are `owner: human`. The other 40 split 30 `started-work` + 10
  `captured` — those are not waiting on a human, they are simply unfinished.

Quoting **105** where **65** is meant overstates the human's backlog by 62%. Quoting
**65** where **105** is meant hides 40 tasks that also carry an unanswerable Human AC.
Neither number is "the" answer; the definition has to travel with it.

Measurement excludes HTML-commented template examples — verified explicitly: 0 of the
128 counted ACs match the template's own `Dashboard renders correctly` /
`Block message names both bypass mechanisms` boilerplate.

### Marker distribution (AC-level, n=128)

| marker | count |
|---|---|
| `[REVIEW]` | 91 |
| `[RUBBER-STAMP]` | 18 |
| `[REVIEWER]` | 1 |
| **no marker at all** | **18** |

The 18 unmarked ACs are the finding that matters most, and the worktree run never
surfaced them. An open Human AC carrying no marker cannot be routed by the convention,
so it is invisible to any triage that sorts by marker — including this one. They cluster
in 8 tasks: **T-2566, T-2567, T-2570, T-2576, T-2577** (each a three-part
decide / audit / file-follow-up shape), plus **T-2815, T-2819, T-2822**.

## Rubber-stamp dispositions — evidence, per item

The Human Task Completion Rule permits *suggesting* a close only with cited evidence.
Ten tasks in the strict finished set carry rubber-stamp-only Human ACs. Each was probed
against live state today. **No box was ticked and no task was closed** — that is the
human's act, and the gate exists precisely for this moment.

### Evidence says SATISFIED — ready for the human to stamp (6)

| task | AC | evidence gathered 2026-08-29 |
|---|---|---|
| **T-1691** | GitHub Release published with macOS + Linux binaries | Release `v0.11.0` published `2026-05-18T20:32:46Z`, `draft: false`, **6 assets**: `termlink-darwin-aarch64` (20.4 MB), `termlink-darwin-x86_64` (24.7 MB), `termlink-linux-aarch64`, `termlink-linux-x86_64`, `termlink-linux-x86_64-static`, `checksums.txt`. Both platforms present. |
| **T-1696** | Cron entry installed in `/etc/cron.d` on .107 | `/etc/cron.d/termlink-release-mirror-canary` present; job line `13 7 * * * root … check-mirror-freshness.sh --quiet`; audit reports PASS; canary log **0 bytes** (empty = healthy). |
| **T-1723** | Cron entry installed on .107 so the meta-canary actually fires | Installed **inside** the release-mirror crontab, second job line: `33 8 * * * root bash scripts/check-canary-aliveness.sh --quiet`. This is the T-2682 UNINSTALLED_JOBS class checked directly — the job line is present, not merely the file. |
| **T-2297** | Live end-to-end after installing the rebuilt hub binary | **Was NOT satisfied; now is.** At first measurement the hub served `0.11.1196` against `VERSION 0.11.1715`, with `/proc/<pid>/exe → (deleted)`. Rebuilt (`0.11.1716`), installed to both the session and cron PATHs, hub restarted THROUGH its systemd unit (G-070). Preflight went 4 pass/2 warn → **6 pass, 0 warn, 0 fail**. The AC's own steps were then run: a TCP post to `agent-presence` came back carrying `observed_addr: 192.168.10.107:50262` — hub-stamped, and never supplied by the client, whose metadata held only `from_project`. Step 4 returns `observed_addr: null` for this session's agent, which is the AC's *documented* correct result (it heartbeats over the Unix socket; only TCP posts are attested), so step 3 is the load-bearing half and it passed. |
| **T-2706** | Confirm closes as superseded-by-T-2709, no topic cleanup | `T-2709` exists at `.tasks/active/T-2709-stuck-claim-heuristic-is-a-monotonic-lat.md`, `status: work-completed`. The supersession is real and the successor landed. |
| **T-2711** | Decide whether U-001 is filed to `framework:pickup` | **It is filed.** All 72 messages on the topic were fetched and base64-decoded; offset **38** reads `T-2711 (010-termlink): revisit-due-scan.sh silent no-op — PROJECT_ROOT mis-resolves in vendored mode, exits 0`. The decision is already executed; the AC records it. |

### Evidence says NOT SATISFIED — do not stamp (3)

| task | AC | what live state actually shows |
|---|---|---|
| **T-2013** | Operator deploys fixed binary to .122, then .121 and .141; confirm 5/5 sequential | Only .122 is current (`0.11.1411`). **.121 serves `0.11.588`** and **.141 does not report a version at all**. Two of the three named hubs are undeployed; the 5/5 confirmation cannot have happened. |
| **T-2408** | Close arc `mcp-slimming` with the demo evidence | `arc-005` is `status: in-progress`, `demo_evidence: null`, `closed_at: null`, `decision: null`. Nothing has been closed. |
| **T-2723** | Decide whether U-008 is filed to `framework:pickup` | **Not filed.** Same exhaustive decode of all 72 messages: three hits for the revisit-due-scan family (offsets 24, 35, 38), **zero** for the handover-commit / focus-gate collision. |

### Evidence unobtainable from here (1)

| task | AC | why |
|---|---|---|
| **T-1722** | Upstream landed on `/opt/999-AEF` `origin/master` | Reading that path is refused by the T-559 project-boundary gate, which is correct behaviour, not an obstacle to work around. Verifying it needs a session rooted in that project (`fw termlink dispatch --project /opt/999-AEF`) or the human's own check. Recorded as unverified rather than assumed either way. |

## What this changes

1. **Six tasks are stamp-ready with citations** — T-1691, T-1696, T-1723, T-2297,
   T-2706, T-2711. Each row above is the evidence the Human Task Completion Rule requires.
2. **Three must not be stamped** — T-2013, T-2408, T-2723 are genuinely
   incomplete. Closing them would record a deployment and an arc closure that did not
   happen. T-2013 remains blocked on the remote half of the same stale-binary fact that
   T-2297 was blocked on: .121 and .141 are not upgradeable from this host.
3. **Eighteen unmarked Human ACs are unroutable** and need a marker before any
   marker-sorted triage can see them.
4. **The "N finished tasks" figure should stop being quoted without its definition.**
   Four different numbers (75 / 79 / 82 / 105) have circulated for what is really two
   populations measured at two different times in two different trees.

## Addendum — what changed while this report was being written

T-2297 moved from NOT SATISFIED to SATISFIED **because the blocker was removed, not
because the standard was lowered.** The stale local binary it was waiting on was the
same fact firing the fleet-binary canary, so one rebuild-and-restart cleared both, and
the AC's own verification steps then passed on live state.

That is the difference worth keeping: four of the ten rubber-stamp items were blocked on
something an agent could actually fix, and only one of them turned out to be so. The
other three (T-2013's remote hubs, T-2408's arc closure, T-2723's unfiled report) are
blocked on decisions or on hosts this session cannot reach.
