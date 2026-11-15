# T-3800 — Backlog triage of the 89 started-work tasks (2026-11-15)

Two read-only TermLink workers (triage-1, triage-2) classified each task with evidence. Counts: NEEDS-OPERATOR 28, DONE 26, BLOCKED 17, LIVE 13, SUPERSEDED 4, UNCLEAR 1.

After the triage (agent, same day): closed T-3749 and T-3750 through the normal gate (verification passed). 12 of the DONE tasks are owner: human with unticked Human ACs, so only the operator can close them. T-3687 is held by the self-deferral gate (a fulfilled deferral to T-3692/T-3693; bypass is operator-only). T-2858, T-2938, T-2939 and T-3178 are DONE but owner: human.

## Part 1 (triage-1)

| id | short name | class | evidence (≤25 words) | open Agent ACs | open Human ACs | suggested action |
|---|---|---|---|---|---|---|
| T-1137 | logrotate on proxmox .180 | NEEDS-OPERATOR | AC1 done; last update Apr: CT200 still rebooting, journald/backups uncapped; host .180 is operator's | 0 | 2 | Operator re-checks /var/log and CT200 stability, or split journald follow-up |
| T-1415 | T-1166 post-cut cleanup | DONE | All 10 Agent ACs ticked with 2026-05-31 evidence; legacy cut long since live | 0 | 2 | Operator ticks bake ACs and closes |
| T-1420 | deploy 0.9.1591 to .141 | BLOCKED | Binary staged/runbook done; deploy on .141 (WSL laptop) never confirmed; version is long obsolete | 1 | 2 | Re-scope to "current release on .141" or close as obsolete with operator |
| T-1426 | deprecation print on legacy primitives | DONE | All 8 Agent ACs ticked; T-3592 reviewer pass | 0 | 1 | Operator reviews warning, closes |
| T-1429 | agent contact verb | DONE | Phase 1 + most Phase 2 shipped; T-1427 completed. Left: name@hub federation, metadata.from, cli.md (deferred) | 3 | 1 | Close Phase 1; file deferred Phase-2 items as own tasks |
| T-1430 | channel describe self-doc | DONE | All 6 Agent ACs ticked; commit records Human REVIEW evidence verified live (2026-05-04) | 0 | 1 | Operator ticks and closes |
| T-1432 | fleet doctor --legacy-usage | DONE | All 11 Agent ACs ticked; fresh re-smoke 2026-06-06 | 0 | 1 | Operator ticks and closes |
| T-1457 | identity on .141 agent-1 | NEEDS-OPERATOR | Human AC asks whether .141 needs a peer Claude at all; whoami AC depends on that | 2 | 1 | Operator rules on .141 end state |
| T-1632 | control_plane_version 3 | DONE | All 8 Agent ACs ticked (commit e89ecb47); only .122 deploy observation open | 0 | 1 | Operator confirms on .122, closes |
| T-1633 | runtime_dir root warning | DONE | All 7 Agent ACs ticked; re-smoke 2026-06-01, 4/4 unit tests pass | 0 | 1 | Operator ticks and closes |
| T-1665 | escape-order fix upstream | SUPERSEDED | Own 2026-05-26 update: premise disproven, upstream uses single-quoted YAML and needs no fix | 3 | 0 | Close as no-op (premise disproven) |
| T-1699 | framework upgrade 1.6.160 | SUPERSEDED | Task Context: superseded by T-3370 re-vendor (1.8.3) and 1.8.15 upgrade (T-3648 triage) | 5 | 0 | Close as superseded |
| T-1727 | default secret-scan patterns upstream | DONE | Vendored `.agentic-framework/.secret-scan-patterns` exists; `secret-scan.sh` is +x — shipped upstream by re-vendor | 5 | 0 | Verify hook message then close; ACs met by upstream |
| T-1799 | purge leaked GitHub PAT | NEEDS-OPERATOR | All 6 local purge ACs ticked; PAT rotation/revocation on GitHub is operator-only | 0 | 1 | Operator revokes PAT, ticks, closes |
| T-1885 | fw independent-review v0.1 | DONE | 13/14 Agent ACs ticked; only one investigate-T- follow-up file exists, RCA-stub AC unverified | 1 | 1 | Reviewer checks follow-up RCA stub; operator dry-run review |
| T-1907 | auto-commit in inception decide | SUPERSEDED | Vendored lib/inception.sh now commits the decision itself (`git commit --only`, lines 437-475) | 6 | 0 | Close as done upstream |
| T-1908 | Gate 2 grace period | SUPERSEDED | Vendored check-active-task.sh has completed-task grace window (T-4182, 3600s, line 1048) | 7 | 0 | Close as done upstream |
| T-2017 | .141 channel.info wedge (WSL) | BLOCKED | Hypothesis never tested; needs hands on .141 WSL host | 5 | 0 | Park later or close with T-1420 if .141 retired |
| T-2026 | typed agent-launch surface (inception) | NEEDS-OPERATOR | All ACs ticked; operator GO recorded 2026-10-18 via Watchtower; task never finalized/build tasks? | 0 | 0 | Finalize inception; confirm build tasks filed |
| T-212 | Homebrew tap | BLOCKED | All ACs ticked but github.com/DimitriGeelen/homebrew-termlink still 404 (T-3105, G-010) | 0 | 0 | Operator creates the tap repo, or untick Human AC 1 |
| T-2197 | Audit D13 — 4 inception limbo | DONE | T-1635, T-2025, T-2028 completed; T-1898 decided DEFER (horizon later) | 0 | 1 | Close; decisions already recorded |
| T-2198 | Audit D14 — T-2144 empty Recommendation | DONE | T-2144 in completed/ with **Decision**: NO-GO | 2 | 1 | Tick re-run AC after audit, close |
| T-2203 | CTL-028 bulk flip 157 tasks | DONE | Commit 444a7e9b3 flipped 157 files; CTL-028 PASS 2026-08-20; Path B already authorized and taken | 0 | 1 | Operator ticks moot Path choice, closes |
| T-2258 | hub read-path stall fix | DONE | Commit 81b45ca5b: live .107 re-test passed on 0.11.324; T-2355 added walk deadline | 0 | 1 | Operator ticks, closes |
| T-2383 | workflow-designer mirror diagnosis | UNCLEAR | 0 commits; cross-project (832) diagnostic; current mirror state not checked (outside project) | 3 | 0 | Ask 832 whether still needed; else close |
| T-2389 | relaunch .107 agents via tl-claude | NEEDS-OPERATOR | Agent ACs ticked; Human AC is a timing decision on re-homing agents; no tl-claude @reboot entry found | 0 | 1 | Operator decides re-home timing |
| T-2398 | Model-B live validation | LIVE | No report in docs/reports, no ACs ticked; 2 commits only | 5 | 0 | Run or park (horizon later) |
| T-2470 | canonical charter | NEEDS-OPERATOR | Agent ACs ticked; charter sentence unratified (T-3335 trace) | 0 | 1 | Operator blesses purpose sentence |
| T-2486 | charter-core health review (inception) | NEEDS-OPERATOR | Report docs/reports/T-2486-*.md exists; Recommendation DEFER; no decision recorded | 3 | 1 | Operator decides; agent ticks inception ACs |
| T-2550 | spawn ok:true before registration | NEEDS-OPERATOR | 4/5 Agent ACs ticked; fix direction awaits operator ruling | 1 | 1 | Operator picks fix direction |
| T-2567 | exec truncated exit_code design | NEEDS-OPERATOR | Two Human REVIEW decisions open; no build work yet | 1 | 2 | Operator decides |
| T-2620 | reliability defect hunt 2 report | DONE | 7bc5d1709 staged 4 findings; filed/fixed as T-2619, T-2621, T-2624 | 2 | 0 | Tick ACs, close |
| T-2669 | bounded rpc_call sweep | NEEDS-OPERATOR | 12/~145 sites migrated; rest blocked on operator timeout policy for CLASS 2 | 1 | 1 | Operator chooses timeout policy |
| T-2717 | inception decisions contradict rationale | LIVE | Upstream defect records persisted (87bd49234) but no report and pickup filing unconfirmed | 8 | 0 | Confirm framework:pickup post, write report or close |
| T-2719 | inbox unread count mixed indices | LIVE | Latest-half fixed (T-2533); cursor two-store defect not filed; agreement test not shown | 3 | 0 | File cursor task, add over-report test |
| T-2725 | herdr adoption research (inception) | NEEDS-OPERATOR | Recommendation DEFER written; no decision recorded | 3 | 1 | Operator decides |
| T-2815 | audit cron-drift worktree slug | NEEDS-OPERATOR | Stray /etc/cron.d/agentic-audit-agent-a1edeca4dc54e9ac7 still present | 0 | 1 | Operator decides removal (runme action) |
| T-2819 | narrow .agentic-framework gitignore | NEEDS-OPERATOR | All 9 Agent ACs ticked; `.agentic-framework/lib/foo.py` not ignored; catch-up run is Human AC | 0 | 1 | Operator runs catch-up review |
| T-2837 | cross-checkout actions blocking t2687 | NEEDS-OPERATOR | 046529012: two of three blockers stale; merge script deleted (T-2839); P-043 disposal open | 2 | 1 | Reviewer ticks stale ACs; operator disposes P-043 |
| T-2858 | install-path drift | DONE | All ACs ticked: three paths agree on 0.12.1084 (2026-10-24) | 0 | 0 | Close |
| T-2870 | unlocked pickup crons race | NEEDS-OPERATOR | /etc/cron.d/agentic-pickup-termlink still has an unlocked `fw pickup process` line | 0 | 1 | Add flock via runme action |
| T-2872 | channel ack unread off-by-one | NEEDS-OPERATOR | Repro + count_unread test done; fix location awaits Human REVIEW choice | 3 | 1 | Operator chooses approach |
| T-2875 | cross-session reach (inception) | NEEDS-OPERATOR | Report exists; Recommendation NO-GO on new transport; no decision recorded | 3 | 1 | Operator decides |
| T-2938 | agentic-audit crontab drift | DONE | `diff -q` repo crontab vs /etc/cron.d/agentic-audit-termlink: identical | 0 | 0 | Close |
| T-2939 | substrate-smoke canary not installed | DONE | /etc/cron.d/termlink-substrate-smoke-canary exists, identical to tracked source | 0 | 0 | Close |

## Counts per class

| class | count |
|---|---|
| DONE | 17 |
| SUPERSEDED | 4 |
| LIVE | 3 |
| BLOCKED | 3 |
| NEEDS-OPERATOR | 17 |
| UNCLEAR | 1 |
| **total** | **45** |

## Part 2 (triage-2)

| id | short name | class | evidence (≤25 words) | open Agent ACs | open Human ACs | suggested action |
|---|---|---|---|---|---|---|
| T-2940 | D2: 57 tasks >30d in review queue | NEEDS-OPERATOR | Agent half done (1/1 ticked); queue drained in part by T-3592 stage 3. Only the Human REVIEW "queue triaged to readable size" is open | 0 | 1 | Operator judges the remaining queue size, then closes |
| T-2984 | Inception: structural consumer for framework:pickup | NEEDS-OPERATOR | Inception, no decision recorded; Recommendation: NO-GO on the filing's scope, GO on a smaller one | 3 (template) | 1 | Put go/no-go to the operator |
| T-3010 | 90-day revisit_at on artifact.* decision | BLOCKED | Deliverable done (3/3); the task stays in active/ on purpose so the G-053 scan surfaces it on 2026-12-18 (task file line 95) | 0 | 0 | Leave open until 2026-12-18 revisit |
| T-3169 | Triage framework:pickup offsets 168/180/181 | NEEDS-OPERATOR | Inception, Recommendation DEFER, no decision; 1ceb69aaa (T-3178) triaged 163-196 | 3 (template) | 1 | Operator decides DEFER (set revisit_at) or NO-GO |
| T-3178 | Write-gate bypass: 2>/&> as not-a-write | DONE | 7/7 Agent ACs; the rule is kept local in safe-commands.sh and carried through every re-vendor up to be.49 (T-3778, .vendor-divergence.yaml) | 0 | 0 | Run Verification, close |
| T-3188 | target_blast_radius fallback for all types | NEEDS-OPERATOR | One AC records a FAILURE (211/248 unchanged); per the task it closes when T-3189 closes or the operator accepts the AC as superseded | 1 (failed) | 0 | Operator rules on the failed AC (superseded by T-3189?) |
| T-3189 | Estimator derives blast_radius from task evidence | DONE | 10/10 Agent ACs; re-applied in fc4adbd5f (T-3370); quadrant 6→111 per T-3188 | 0 | 1 | Operator ticks REVIEW "sane ranking", close |
| T-3303 | claude-termlink signs with per-agent key | LIVE | AC1/AC2 done (b66343559, cd28cf2cf); DM to pen's owner about the shared host key not evidenced | 1 | 0 | Send the DM to pen's owner, then close |
| T-3330 | Receive-side received/injected/replied telemetry | NEEDS-OPERATOR | Inception, Recommendation DEFER, no decision; voi note says arc-011's sidecar already answers much of it | 3 (template) | 1 | Operator decides (DEFER, or NO-GO as covered by arc-011) |
| T-3335 | External consultation: two-way interactive comms | NEEDS-OPERATOR | Inception, Recommendation DEFER; its content became the arc-011 requirements (44c7eabfc), no decision recorded | 3 (template) | 1 | Operator decides; likely close as answered by arc-011 |
| T-3344 | arc-011 step 1: requirements collector | DONE | docs/design/interactive-agent-communication-01-requirements.md exists; step 1 signed off (0a0ff90ae); T-3348/T-3349 completed | 4 (satisfied, unticked; handback sha not checked) | 2 | Check the handback sha, tick the Agent ACs, hand to operator review |
| T-3350 | GP-11 operator out-of-band channel inventory | BLOCKED | Only 050-email-archive's inventory is recorded (0a0b82238); AEF and ring20-dashboard inventories not received | 3 | 1 | Chase AEF and ring20-dashboard, then put GP-11 to the operator |
| T-3383 | 30 days of hub outage durations | LIVE | Collector installed 2026-10-11 (ed7e86994; runme.sh:281). 30 days have passed but no summary result is recorded | 1 | 0 | Run the summariser, record the result, re-check the credential |
| T-3385 | Inception: OTP-style out-of-band approval | NEEDS-OPERATOR | Inception, no decision; its GO unblocks P-64 (T-3737) | 3 (template) | 1 | Put go/no-go to the operator |
| T-3426 | arc-011 PH-2 planning: architecture + build table | NEEDS-OPERATOR | 5/6 Agent ACs; open: operator rulings on OQ-a..OQ-i recorded in the design | 1 | 0 | Collect the remaining OQ rulings, one at a time |
| T-3466 | R5b daily telemetry rollup | BLOCKED | Built (4cce83c93, 4/5 ACs); "running on .107 and a ring20 host" not shown | 1 | 0 | Get a ring20 host running the rollup, check the topic |
| T-3476 | Deterministic task-assignment sidecar | NEEDS-OPERATOR | Inception (voi 0.8), no decision; ID allocation went upstream as AEF T-4393 (58d84b568) | 3 (template) | 1 | Put go/no-go to the operator |
| T-3484 | Shared dispatched-worker contract | BLOCKED | TermLink agreed v0.2 (9cf9acd00); T-3501..T-3503 done, T-3504 open; answers from the ring20 and AEF operators are pending | 3 | 0 | Wait for the ring20/AEF operators' answers |
| T-3488 | Hub write stalls on .107 | LIVE | Cause traced to btrfs fsync (T-3541 completed, 5d6f5cbb9); the 24 h re-measure after deploy is not done | 1 | 0 | Re-measure for 24 h on .107 |
| T-3494 | Track AEF T-4472 triage of 10 vendored fixes | BLOCKED | Triage recorded (91f863266); by design it stays open until all 8 adopted patches land upstream | 1 | 0 | Flip register entries as AEF lands each patch |
| T-3549 | SEV-1 cross-project env leak via tmux | LIVE | T-3550 and T-3551 completed; docs/reports/T-3549-incident exists; reviews, damage assessment and T-3552 still open | 4 | 0 | Tick the brief AC; finish reviews/damage assessment after T-3552 |
| T-3552 | SEV-1 fix 3: one tmux server per project | LIVE | 4/5 ACs; docs/reports/T-3552-per-project-tmux-interface.md exists; 055's answer not recorded | 1 | 0 | Record 055's answer (or chase it) |
| T-3592 | Staged reviewer sweep of human-owned backlog | NEEDS-OPERATOR | Stage 3 batches done (3fcd46a3d etc.); open stage 3/4 items, and stage 4 is the operator's evidence list | 0 | 2 | Present the stage-4 list to the operator |
| T-3609 | Orchestrated [Review,Audit,procAsFit]x9 | BLOCKED | Only the r01 gatherer ran (T-3610 completed); the operator must confirm the Phase 1 yardstick before any judge runs | 4 | 0 | Ask the operator to confirm the r01 yardstick, or retire |
| T-3615 | Fleet usage + worker-outcome telemetry | LIVE | Design and review synthesis exist (docs/reports/T-3615-*); all 6 Agent ACs open | 6 | 0 | Build per design |
| T-3624 | T-3612 round 4: record operator rulings | DONE | T-3612 GO, and T-3642 applied the adopted draft v0.9 (caaee00ee); both completed | 2 (satisfied, unticked) | 0 | Tick, close |
| T-3638 | AEF ask: per-layer value-driver sets | LIVE | AEF GO on T-4961; per-arc driver sets vendored in T-3650 (be.21); the "wired after re-vendor" AC is not done | 1 | 0 | Wire the product set on the vendored mechanism |
| T-3648 | Backlog triage (done/live/parked) | DONE | docs/reports/T-3648-backlog-triage.md exists; close pass d4a31c54c; T-3800 now re-runs the triage | 3 (satisfied, unticked) | 0 | Tick, close (T-3800 succeeds it) |
| T-3654 | Main-session-only delivery | LIVE | 5/7 Agent ACs; open: non-main copy behaviour (R-67.a1) and plan unit P-35 | 2 | 1 | Continue P-35 / non-main behaviour |
| T-3656 | 74 messages stuck in fw sidecar waiting | LIVE | Messages classified (b76dbdac9); root cause with file:line in AEF's receiver not yet stated | 1 | 0 | State the root cause, propose the fix to AEF |
| T-3663 | Cross-machine file routes under O-1 | NEEDS-OPERATOR | Plan ordered by severity, "ready for the operator" (821ad01c9); T-3667 completed; peers not yet asked | 1 | 0 | Operator reviews the plan; then ask peers |
| T-3665 | WireGuard key over plain HTTP | BLOCKED | Operator ruling A recorded; relayed to ring20 T-2595, which waits for the operator's direct go to ring20 (f45b9adea) | 1 | 0 | Operator gives ring20 the go |
| T-3674 | fleet-doorbell-mail canary FIRING | BLOCKED | Waits on the operator to restart greenfield's hub; latest runme (17:05Z) shows greenfield gives no answer | 1 | 0 | Operator restarts the greenfield hub; agent verifies |
| T-3678 | multi-MB file test times out | LIVE | 3 hypotheses disproved (f4e0faa42); no fix | 3 | 0 | Bisect/fix |
| T-3685 | runme action 28: testdev Claude token | DONE | Action 28 is in runme.sh:3664; latest.log: "skip testdev's stored token answers" | 3 (satisfied, unticked) | 1 | Tick the Agent ACs, operator ticks the Human AC |
| T-3687 | B-27 activity meter | DONE | 5/5 Agent ACs; follow-ups T-3692/T-3693 completed (0077f727f) | 0 | 0 | Run Verification, close |
| T-3694 | P-4 lease tightening, K-34 | BLOCKED | 3/4 ACs (82b6685bf); W1-pair `main status` check needs the release deployed (T-3698) | 1 | 0 | After T-3698 install, check W1 status |
| T-3697 | P-7 doorbell D+ / 300 s watchdog | BLOCKED | 4/6 ACs (692e4ae94); open: the macOS host check and the canary-pair collectors | 2 | 0 | Run on macOS host and canary pair |
| T-3698 | P-8 release install action for .107 | BLOCKED | Action 4e built (6a20f44c2); latest.log: "REL_TAG is empty until G-1 cuts one" | 4 | 0 | Wait for the G-1 release, then run 4e |
| T-3749 | Mac→ring20 message lost silently | DONE | 6/6 Agent ACs; ring20 mails handled (e27534b9e) | 0 | 0 | Run Verification, close |
| T-3750 | Remote-hub inbox mail seen only at start | DONE | 9/9 Agent ACs; bd1cb1950 "close — live-proven", but the file is still in active/ | 0 | 0 | Run Verification, close |
| T-3757 | Callers declare `from` on inject/exec | BLOCKED | Step 1 done (8eec4b092); the 7-day evidence window ends about 2026-11-20 | 2 | 0 | Report after 11-20, then step 2 |
| T-3781 | Non-root dispatch | BLOCKED | 8/8 Agent ACs, live on .107 (132b46acc); waits on 1409's design session for fw dispatch | 0 | 0 | Close after 1409 confirms |
| T-3798 | Mail to an unread nickname inbox | BLOCKED | AEF took T-5148/T-5149/T-5150 (65f15a94f); open until re-vendor | 1 | 0 | Re-vendor once AEF ships, then live proof |

"(template)" means the open Agent ACs are the inception template's placeholders.

## Counts per class

| class | count |
|---|---|
| DONE | 9 |
| SUPERSEDED | 0 |
| LIVE | 10 |
| BLOCKED | 14 |
| NEEDS-OPERATOR | 11 |
| UNCLEAR | 0 |
| **total** | **44** |
