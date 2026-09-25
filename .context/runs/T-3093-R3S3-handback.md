# T-3093 R3S3 handback (procAsFit, round 3 of 4)

**Status:** COMPLETE — one unit of work (T-3134) selected, built, live-proven, and closed;
a second finding (T-3140) filed, scored, and correctly parked (out of the first unit's Scope
Fence); stopped between tasks at ~317K/~39% context, past the mandate's ~300k ceiling.

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

Wrote 6 real Agent ACs on T-3134 (was placeholder-only), then implemented:
- `crates/termlink-cli/src/commands/artifact.rs` (new): `cmd_artifact_put`/`cmd_artifact_get`,
  local-hub-only (mirrors `file send`'s `resolve_hub_paths()` pattern), a fail-fast
  `sha256_args_agree` check run BEFORE any hub contact so a caller-argument mismatch on `get`
  never touches the network, `--timeout`-bounded RPC via `tokio::time::timeout`, and 5 pure
  unit tests for the mismatch-detection helper.
- `crates/termlink-cli/src/cli.rs`: new `Command::Artifact { action: ArtifactAction }` with
  `Put { path, to, json, timeout }` / `Get { sha256, expected_sha256 (clap-required, not
  `Option` — IW-3), output, json, timeout }`, plus 3 clap parse tests in `cli_tests` (put
  parses `--to`; get rejects a missing `--expected-sha256`; get parses `-o`).
- `crates/termlink-cli/src/main.rs`: dispatch wiring.
- `tests/artifact-cli-fixtures.sh` (new, 14 assertions): hermetic, no live hub — `--help`
  surfaces both verbs; missing `--expected-sha256` is a clap error; mismatched
  sha256/--expected-sha256 fails fast with no output file written; matching args reach (and
  fail cleanly on) the absent hub; `put` on a nonexistent file fails cleanly; `--json` mode
  emits parseable `{"ok":false,...}`.

**Unit tests, clippy, and 3 static guard-layer checks all clean:**
`cargo test -p termlink artifact` (10/10), `cargo test -p termlink cli_tests::` (6/6),
`cargo clippy -p termlink --no-deps` (0 new warnings — 86 pre-existing, none in my files),
`check-alloc-sink-clamps.sh` / `check-drain-sink-caps.sh` / `check-silent-exit.sh` all rc=0.

**Live end-to-end proof (not just unit tests):** started an isolated, disposable local hub
(`TERMLINK_RUNTIME_DIR`/`TERMLINK_IDENTITY_DIR` pointed at a scratch dir, unix-socket only, no
`--tcp` — never touched the real production hub already running on this shared host at
:9100/pid 3071124, confirmed untouched at both start and end). `artifact put` of a 200KB
random payload went via the chunked `artifact.put` path (`via: channel.artifact`); `artifact
get` downloaded it byte-identical (`cmp` clean) with `sha256_verified: true`.

**Genuine bug found and fixed live (Hypothesis-Driven Debugging):** first `put` attempt failed
with `channel.post error -32014: sender_id="cli-<pid>" does not match identity fingerprint`.
Hypothesis: `ArtifactManifest.from` (copied verbatim into the signed envelope's `sender_id` by
`send_artifact_via_client`) must equal `identity.fingerprint()`, not an arbitrary label — one
test (switch to `identity.fingerprint().to_string()`, retry) confirmed it. Grep showed the
SAME `format!("<prefix>-{pid}")` pattern in all 3 pre-existing callers (file.rs/remote.rs/
tools.rs) — a real, currently-live defect, just unexercised (the file.rs path is behind a
deprecation warning; none of the three appears to have been proven against a hub that enforces
T-1427). Fixed only in my own new call site (in-scope); filed **T-3140** (captured, BVP-scored
D1=4 D2=2 D3=3 D4=3, NOT worked — fixing the 3 pre-existing sites is outside T-3134's Scope
Fence and outside this unit's selection) for the rest, per "register first, fix second" and
"one bug = one task".

Checked all 6 ACs, added the required `## Evolution` entry (arc-tagged build task) recording
the manifest.from finding, added `related_tasks: [T-3140]`, ran the full `## Verification`
block for real (not just individually), then `fw context focus T-3134` + `fw task update
T-3134 --status work-completed` (completed cleanly, `date_finished` stamped — no T-2833-class
latch). Re-marked arc-011 slice S2 `status: built` with the live-proof note (first edit had a
missing closing YAML quote — caught immediately by `check-arc-slice-drift.sh` erroring with
"does not parse" rather than silently corrupting the register; fixed, re-verified with
`python3 -c yaml.safe_load` before re-running the check, which now reports clean: "12 slice(s)
across 1 in-progress arc(s), 0 acknowledged").

**T-3140 (filed, scored, parked):** "artifact manifest.from must be identity fingerprint, not
cli-pid label" — captured, BVP-scored, `related_tasks` back to T-3134, not started. Correctly
left for a future round/dispatch: fixing 3 unrelated pre-existing call sites is not part of
this unit's ACs, and starting a second unit this late in the context budget risked a mid-task
stop on the very rule this run's carried fixes exist to prevent.

## Stop condition

Stopped between tasks (T-3134 fully closed, verified, and committed; T-3140 deliberately filed
and parked rather than started) at ~316,838 tokens (~39% of context window) via
`checkpoint.sh status` — past the mandate's ~300k ceiling. No task was left mid-flight.

## Commits this step

- `5f5ae0143` — T-3134: `termlink artifact put`/`get` CLI verbs, tests, arc-011 S2 re-marked
  built, T-3140 filed.
- `9bc77c6cb` — fabric component cards for the two new source files.

**Ambient drift observed, deliberately NOT committed here:** the working tree still carries
uncommitted changes to `.tasks/active/T-3139-*.md`, `.tasks/completed/T-2753-*.md`, several
`.context/audits/*` snapshot files, and shared working-memory files (`.hook-counter`,
`feedback-stream.yaml`, `session.yaml`, `.budget-status`). Unlike prior steps' own trailing
residuals, this looks like a genuinely different, currently-active concurrent session on this
shared host — T-3139 (2 commits already landed, small residual diff) and T-2753 (1 commit
landed via the operator's own Watchtower action, larger residual diff with reviewer-verdict
sections) are both unrelated to arc-011/T-3093's scope. Also observed: setting
`fw context focus T-3134` was immediately overwritten back to `T-3093` in `focus.yaml` within
the same few seconds, more than once — consistent with another live process writing the same
shared file. Left uncommitted rather than folded into my own commits: it is not this step's
work to close out, and claiming it in a commit message here would misattribute someone else's
in-progress session. Flagging for the operator / next dispatched step rather than acting on it.

## Objectives advanced (vs. state at this step's start)

- arc-011 (Agent-to-agent message delivery: mailbox to prompt): slice S2 unbuilt -> built,
  live-proven end to end. 3 of 12 slices now built where 2 were before this step (S2 newly
  built; S1 and S12 remain unbuilt, bound to T-3135, Q2, not selected this round).
- A genuine correctness defect (T-1427 sender_id validation vs. the 3 pre-existing artifact
  callers) that would otherwise have shipped invisibly the next time any of file.rs/remote.rs/
  tools.rs' artifact path was actually exercised against an enforcing hub is now a filed,
  scored, traceable task (T-3140) instead of a silent landmine.

## Arc state (arc-011)

- S1 (sender via sidecar API): unbuilt, bound to T-3135 (Q2 hv-hc, carries operator SQ-8:
  systemd-only respawn REJECTED, portable fallback REQUIRED). Not selected this round —
  Q1-before-Q2, and a large/risky unit not worth starting this late in budget.
- S2 (payload may carry a binary blob): **built this step** (T-3134), live-proven.
- S3/S4 (receiver stores/flags): built (pre-existing, T-2300/T-3068).
- S12 (independent respawn supervisor): unbuilt, bound to T-3135 (same Q2 task as S1).
- Remaining slices (S5-S11 etc.): unchanged from R2S3's characterization — see arc-011.yaml
  for current per-slice detail; not re-audited this step (out of this unit's scope).

## What remains in Q1/Q2, per task

- **T-3135** (arc-011 S1+S12, Q2 hv-hc): sidecar API + portable respawn supervisor. NOT
  worked — correctly deferred behind Q1 exhaustion, and its size/risk (a new always-on
  process, SQ-8's portable-fallback requirement) is a poor fit for whatever fraction of a
  fresh dispatch's budget would remain after picking it up mid-round.
- **T-3140** (new this step, Q1-equivalent by BVP profile but not yet quadrant-ranked against
  the wider backlog): fix the 3 pre-existing `manifest.from` sites. Small, single-session,
  well-scoped by this step's own investigation — a strong next-unit candidate for R4S3 or
  whichever step picks arc work next.
- arc-008's ~55-task backlog (captured/started-work, per R3S2's characterization) and arc-009's
  17 remaining tasks: unchanged, out of this step's selected arc, not re-triaged.

## Sovereign questions raised, unresolved

None new. T-3135's SQ-8 constraint (systemd-only respawn REJECTED, portable fallback
REQUIRED) is unchanged and still binds whoever picks up S1/S12.

## Gates that refused, and what was done instead

- `check-active-task` (P-002/Tier-1) refused a read-only command once focus had moved to
  T-3093 while I still had a stray reference to T-3134 in a command — resolved by re-running
  `fw context focus` to the task actually relevant to the command about to run, not by
  bypassing.
- No other gate refused this step's work. `fw task update T-3134 --status work-completed`
  passed the P-011 verification gate on the first real attempt (all 4 lines re-run and
  confirmed passing before invoking it, not just individually rehearsed).

## Cost-vs-estimate deltas worth feeding back into calibration

- T-3134's `cost_estimate_proposed` was `effort: 8` (lines=204, acs=4) at filing time. Actual:
  ~340 new lines across 3 files (artifact.rs 265 + cli.rs additions ~70 + fixtures 95) plus a
  live debugging detour (the manifest.from bug) that cost real budget but produced a second
  filed task rather than blowing the AC count. The estimator's effort figure undercounted the
  fixture-script line count entirely (it wasn't listed as a component with an existing size at
  filing time, since the file didn't exist yet) — worth noting for the estimator: a component
  path that doesn't exist yet contributes 0 to the lines estimate, understating tasks whose
  scope explicitly names a new test file.
- The live-hub-proof step (spin up an isolated hub, round-trip a real payload) cost roughly as
  much context as the entire unit-test-writing phase, but is what caught the one genuine
  defect this step found. Worth recording as a pattern: for build tasks whose ACs describe
  wire-protocol interaction, unit tests alone would have shipped a broken `put` verb — the
  IW-3-mandated verification-before-shipping discipline this run keeps citing (Presenting Work
  for Human Review) applies just as much to "did I actually run it," not only to human review
  quality.
