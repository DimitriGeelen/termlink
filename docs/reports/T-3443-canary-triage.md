# T-3443 — triage of the five canaries firing since 2026-10-13

Date 2026-10-14. Each finding: what fired, the cause with evidence, what was done.

## 1. canary-aliveness — fixed (stale, not a live fault)

- **Fired on:** 2026-10-06, `notify-sidecar-canary` cron-scheduled but writing no heartbeat.
- **Cause of the lasting red:** that condition was fixed after 2026-10-06 (its heartbeat is fresh: 2026-10-14 07:53), and
  today's sweep (`check-canary-aliveness.sh --all --quiet`) is silent, rc 0. It stayed FIRING because the sweep itself
  wrote no heartbeat, so `/canaries` judged it by log content alone and an old entry kept it red forever.
- **Done:** the sweep writes its own completion heartbeat in `--all` mode (EXIT trap, `--no-heartbeat` opts out),
  fixtures 29/29 (two new). One sweep run as cron runs it: green.

## 2. dead-letter — fixed; reported to AEF

- **Fired on:** 12 rows in the host's shared outbound queue (`/root/.termlink/outbound.sqlite`).
- **Cause:** all 12 were AEF test traffic, signed by sender `d1993c2c3ec44c94`:
  - 11 `sidecar.receipt` posts from AEF test agents `t3684` / `t3745` (messages `legacy-1`, `m1`, from_project
    999-Agentic-Engineering-Framework), enqueued 2026-10-07 05:29-05:30Z to test-only inboxes; the hub refused them as
    unknown topics.
  - 1 `learning-P-001` post ("test learning", PL-235, T-500, origin_project `proj`) queued 2026-10-13 10:21Z for the
    REAL fleet `channel:learnings`. Our flush dead-lettered it as "corrupt/undeserializable"; the row decodes and has
    every field `PendingPost` needs, so why termlink rejected it is NOT determined. Had it parsed, a test learning
    would have gone fleet-wide.
- **Done:** rows backed up in full to `.context/working/dead-letters-backup-20261014.jsonl`, then removed; canary
  healthy (0 pending, 0 dead). Reported to AEF (conversation `tl-aef-test-leak`): their tests must use an isolated
  runtime/HOME or a fake transport.

## 3. framework-pickup — triaged and acked

Four inbound filings on `framework:pickup`, all addressed to AEF (the framework), none to TermLink:

| Offset | Filing | Disposition |
|---|---|---|
| 320 | P-052 (via AEF, from 1409-sprind T-1752): delivery telemetry for all projects | Framework feature; AEF's. No TermLink task. |
| 323 | P-053 HIGH (via AEF, from 1409-sprind T-1799): sessions die at the 95 % budget block instead of a handover | Framework budget gate; AEF's. Affects every consumer, no TermLink code. |
| 324 | P-007 (1409-sprind T-2017): arc "collections" on /arcs | Watchtower feature; AEF's. |
| 325 | P-008 (1409-sprind T-2022): per-session arc focus; `--allowed-tools` is not a boundary for dispatched workers; agents set `FW_SWITCH_FOCUS` themselves | AEF's dispatch script and gates. Point 4 applies to this agent too: it used `FW_SWITCH_FOCUS=1` (logged) several times on 2026-10-13/14 to attribute commits to the right task. |

Marker acked to offset 325 after the triage.

## 4. release-publication — partly fixed; rest filed (T-3444)

- **Fired on:** two WARN guards red > 14 days (escalated as T-3440, T-3441, filed unattended and committed by name in
  this task), 7 WARN members red at once, and `doc-lint.yml` (the guard-layer job) with no green run on main since
  2026-09-30. The release itself (v0.12.4) is not a finding.
- **Fixed here (FAIL-tier members):**
  1. `hub-runtime-dir-fixtures.sh` (from T-3437, this session) failed in CI: the runner is not root, so `chown` to a fake
     uid failed. Now uses the runner's uid when not root; passes as root and as `nobody`.
  2. `runme-fixtures.sh`: 76 of 107 cases failed in CI on runme's non-root refusal (only cases 3-9 were skipped). Fixture
     mode may now opt out (`RUNME_FIXTURE_ALLOW_NONROOT=1`, safe because fixture mode aborts before any real host
     default); case 11 still tests the real refusal. As `nobody` with a private /tmp: 123/125, the two left only fail
     because the local checkout is root-owned.
  3. `canary-log-isolation`: `tests/count-hub-plaintext-fields-fixtures.sh` (P8) named `agent-send.sh` without
     redirecting the woken-silent log; it never runs it, but now redirects. Check clean, fixtures 20/20.
- **Filed as T-3444:** `check-verification-misfile` fires on T-3300; `human-ac-escalation-fixtures` asserts the real
  tree is clean (fails while T-3370 waits on the operator); `pin-events-fixtures` fails in CI only;
  `vector-index-reader-no-rebuild-fixtures` errors in CI (no `ollama` module on the runner).
- **Stays firing** until a guard-layer run on main is green — correctly.

## 5. waker-liveness — the operator's decision

- **Fires on:** `penelope` (fp d1993c2c, this host) LIVE but with no `pty_session`, and "rail-dark": zero LIVE listeners
  on this hub carry a waker. The canary runs with `--expect-armed` for .107.
- **Cause:** agents here are not started through `tl-claude.sh --reachable`, so the arc-004 push-waker never arms; DMs
  wait on the poll floor. No defect in the canary or the rail.
- **Open:** whether to arm agents (relaunch through the launcher with `--reachable`), or to stop expecting the legacy
  rail armed on .107 while arc-011's doorbell (W2) replaces it. That is a choice about how this host runs, so it is put
  to the operator, not decided here.

## 6. forever-archival — fired 2026-10-14 07:51 (new, after this triage began)

- **Fires on:** project inboxes kept forever and growing without bound: `inbox:cacc73ea32b121dd/999-Agentic-Engineering-Framework`
  (1435 records, 66.5/day over 21.6 days) and, in earlier entries, `inbox:cacc73ea32b121dd/010-termlink`.
- **Cause:** project inboxes on this hub have `Retention::Forever`; the charter's non-goal #2 says topics are not a
  system of record.
- **Not done here:** the suggested remedy (cap the inbox and sweep) can drop mail a receiver has not read yet, and one
  inbox is AEF's. A retention for project inboxes is a decision for the operator, coordinated with AEF; arc-011 will
  also move this project's mail to per-agent inboxes (W4).
- **Correction (2026-10-15):** the forever retention is not a policy anyone chose. T-3310 already made "newest 1000"
  the default for `inbox:*`; inboxes auto-created before it fell through to forever (commit e5d758159). Leftover forever
  project inboxes: AEF 1462, ours 1042, 832 746, 055 180, 1409-sprind 150. Ours: newest offset 1041, lowest hub read
  receipt 969, our alert marker 1040. AEF's records no read receipts on the hub.
- **Ruled B (operator, 2026-10-15):** cap ours now behind a reader check (runme action 18: refuses when any reader is
  behind the cut point or none is known); the others only after their owner confirms.
- **Done (2026-10-15/16):** all five owners confirmed (AEF, 832, 1409-sprind by their operators; 055 by its agent).
  runme 2026-10-15T23:29Z capped ours (offsets 0–42 removed), AEF's (0–462, under AEF's stated read position 1461),
  055's and 1409-sprind's (policy only); runme 2026-10-16T21:05Z capped 832's (policy only). The canary is healthy.

## Close (2026-10-17)

What stays open, named so a later reader does not take the close for "all clear":

1. **release-publication still fires**, correctly, until a guard-layer run on main is green; the remaining reds are
   T-3444.
2. **Why termlink judged the queued `learning-P-001` row "corrupt"** is not determined (finding 2). The row decoded
   cleanly and has every `PendingPost` field. It was test traffic, now deleted and backed up; if a real post is ever
   dead-lettered as corrupt, start from `.context/working/dead-letters-backup-20261014.jsonl`.
3. **The waker allowlist is revisited at W2** (decision 5).
4. **The small forever test inboxes** (e2e-*, t3433-*, …, a few records each) were left as they are (decision 6).
