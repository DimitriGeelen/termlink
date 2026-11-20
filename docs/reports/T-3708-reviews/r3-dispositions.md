# T-3708 review round 3 (FINAL) — dispositions

Reviewer: claude-code — **PASS** (no BLOCKER or MAJOR demonstrable), conditional on findings 1-3. opencode returned no text
(rounds 2 and 3) or a truncated first line (round 1). Per the 3-round cap every finding is closed here: accepted with a
change, rejected with a reason, or deferred with a stated risk. Fixes in the commit "review round 3 (final) fixes".

| # | Finding | Disposition |
|---|---|---|
| 1 | `failed()` stamps before the rewrite; verified→failed untested | **Rejected as a defect, change made anyway.** `failed()` always rewrites the line (`unverified since…`, `stale since…`, ROLLBACK, predates); the existing test `e41_a_failed_pull_is_not_an_empty_list_and_goes_stale_after_pn13` drives verified → failed and asserts `unverified since`. The stamp now follows the rewrite. |
| 2 | pending narrowed by `candidates()` | **Accepted, fixed.** The activity veto is not in `candidates()` (the reviewer's scenario does not occur), but `candidates()` skips messages with an open hand-over attempt and has side effects. Pending now comes from the chain rows alone: refused, denied and held-for-the-operator mail is excluded, open attempts stay pending; the status path no longer calls the deliver path. Test `refused_denied_and_held_mail_is_not_pending`; mutant m37. |
| 3 | rows filtered by `--conv`/`--msg-id` | **Accepted, fixed (was a real MAJOR).** `status_at` filters rows by the request; P-24 now always computes over unfiltered rows. `last_surface` shows the pending count; `t3708_status` asserts mail pending in another conversation is counted under `--conv c1`. Mutant m38. |
| 4 | E-127 compares raw wall stamps | **Accepted, fixed.** Compares corrected ages; held-since stamps are registered when written. Test case with a lawful message carrying a higher wall stamp; mutant m39. |
| 5 | eviction drops the lowest wall value | **Accepted, fixed.** Eviction is by write order (FIFO). Test with descending wall values. |
| 6 | foreign writers get epoch 0 | **Accepted as is (documented).** The daemon is its store's only writer; hooks and CLIs reach the store through its API. Said in `Registry::age_ms`. |
| 7 | CLEAR orders raw wall stamps | **Accepted, fixed.** Window, earliest pending and "hand-over after" all compare corrected ages. Test `clear_orders_by_corrected_age_not_wall_value`; mutant m36. |
| 8 | NTP sampler sleeps on CLOCK_MONOTONIC | **Accepted, fixed.** Waits in 30 s steps against the registry clock (boot time). |
| 9 | fallback mixes time bases; no cfg guard | **Accepted, fixed.** The source is chosen once at construction; CLOCK_BOOTTIME only under `cfg(target_os = "linux")`. |
| 10 | unknown skew holds CLEAR on a hub without `hub_now_ms` | **Accepted, fixed.** A hub that answers without a clock gives skew `absent: the hub reports no time (older than T-3708)`, stamped, so it is a measured value during the rollout. |
| 11 | hook record outside the switch | **Accepted, fixed.** Gated on `enabled()`. The `pull` op is the hook's hand-over path, the only arm that offers mail. |
| 12 | silent-hub rig without deliverer or blocked pass; SKIP as green | **Partly accepted.** E-55 now fails instead of skipping. The status path no longer calls the deliver path (finding 2), so a blocked pass can hold no lock the status path takes; residual risk accepted: lock contention with a pass blocked on the hub is not pinned by a test. |
| 13 | strength text with a shared uid | **Accepted, fixed.** "Separate accounts" needs pairwise-disjoint uid sets; distinct `same_user_separation` text for the checked-but-sessions-unknown case. Mutant m4c. |
| 14 | concurrent rotation loses `.1` | **Accepted, fixed.** Rotate-and-append under one process-wide lock. |
| 15 | jump cap drops corrections for epoch-0 stamps | **Accepted, fixed by redesign.** No cap: each written stamp records the running jump total at write time; its correction is the total's change since. Test with 100 jumps. |

Residual risks carried into the closure record: (a) a wall jump while the sidecar was down is not corrected (the skew
line shows it); (b) suspend handling (CLOCK_BOOTTIME) is by review only, not tested; (c) a failed held-since write is
reported but no mutant can trigger it while tests run as root; (d) lock contention between status and a pass blocked on
the hub is not tested.
