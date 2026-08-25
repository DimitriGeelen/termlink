# `/approvals` queue triage — 2026-08-25

http://192.168.10.107:3099/approvals

**125 unchecked Human acceptance criteria across 103 of 211 active tasks.** Nearly half the active backlog is waiting on the operator, not on an agent.

This corrects a claim this arc has been repeating: that T-2837's decisions were *the* bottleneck. They are three rows of 125. The queue was never empty and never small — it was undifferentiated, which is a different problem and has a different fix.

Sorted by what each decision costs you, not by task id.

| bucket | ACs | tasks |
|---|---|---|
| A — Infrastructure: you must touch a machine | 21 | 17 |
| B — Decision: a real choice between alternatives | 57 | 43 |
| C — Evidence: confirm real-world state an agent cannot observe | 10 | 9 |
| D — Subjective sign-off: does it read/feel right | 36 | 36 |
| E — Placeholder: template boilerplate, not a real decision | 1 | 1 |

Only **1** of the 125 is template boilerplate. This queue is real work.

## A — Infrastructure: you must touch a machine

Hard or impossible to reverse. Deploys, cron installs, releases, reboots. Do these deliberately and one at a time. **T-2815 in this bucket is now urgent** — see the duplicate-execution finding recorded on that task.

| task | status | decision |
|---|---|---|
| T-1137 | started-work | [REVIEW] /var/log on proxmox .180 is below 50 % after rotation + daily cron active |
| T-1137 | started-work | [REVIEW] CT 200 (.122) stops rebooting |
| T-1296 | started-work | [RUBBER-STAMP] Apply same migration recipe as T-1294 AC 2 |
| T-1296 | started-work | [RUBBER-STAMP] Re-pin from .102 + fleet doctor green |
| T-1296 | started-work | [RUBBER-STAMP] Verify CT 101 reboot persistence (ground truth) |
| T-1417 | work-completed | [REVIEW] Audit shows zero `event.broadcast` callers from this codebase after production hub rebuild + restart |
| T-1419 | work-completed | [REVIEW] Post-deploy of T-1418, the freshness signal correctly distinguishes live from stale |
| T-1420 | started-work | [RUBBER-STAMP] Binary deployed on .141 — Method A or B |
| T-1420 | started-work | [RUBBER-STAMP] .141 hub restarted on new binary |
| T-1632 | started-work | [REVIEW] On next .122 deploy (after T-1166 bake clears), `hub.capabilities` returns `control_plane_version: 3` alongside `protocol |
| T-1633 | started-work | [REVIEW] On next .122 deploy (post-bake), the warning is visible in hub stderr/journal if the operator forgets to set the env. |
| T-1673 | work-completed | [REVIEW] Confirm release pipeline produced artifacts |
| T-1691 | work-completed | [RUBBER-STAMP] GitHub Release published with macOS + Linux binaries |
| T-1695 | work-completed | [REVIEW] Releases published on GitHub for v0.10.0, v0.11.0, v0.11.1 (the GH Actions auto-trigger) |
| T-1696 | work-completed | [RUBBER-STAMP] Cron entry installed in /etc/cron.d on .107 |
| T-1722 | work-completed | [RUBBER-STAMP] Upstream landed on `/opt/999-AEF` `origin/master`. |
| T-1723 | work-completed | [RUBBER-STAMP] Cron entry installed on .107 so the meta-canary actually fires. |
| T-2013 | work-completed | [RUBBER-STAMP] Operator deploys fixed binary to ring20 hubs (.122 first, then .121 and .141) and confirms 5/5 sequential `channel  |
| T-2258 | started-work | [REVIEW] Fixed binary deployed to the `.107` hub and the live repro no longer hangs (OPERATOR step — the `.107` hub is a live shar |
| T-2297 | work-completed | [RUBBER-STAMP] Live end-to-end after installing the rebuilt hub binary |
| T-2815 | started-work | Decide whether to remove the stray `/etc/cron.d/agentic-audit-agent-a1edeca4dc54e9ac7` |

## B — Decision: a real choice between alternatives

The genuine backlog. Each needs you to choose, and an agent has usually already written a recommendation into the task. Fastest path is to read the `**Recommendation:**` line and either ratify or overrule.

| task | status | decision |
|---|---|---|
| T-1435 | work-completed | [REVIEW] Verification of CUT-READY happens under T-1418, not here |
| T-1457 | started-work | [REVIEW] Decide whether .141 needs a peer Claude attached at all, or whether heartbeat-only target is the desired end state for th |
| T-1457 | started-work | [RUBBER-STAMP] Operator action on .141 (only if peer Claude is required) |
| T-1483 | work-completed | [REVIEW] Verify error messages name the failing input clearly |
| T-1493 | work-completed | [REVIEW] Verify the on-thread reading view scans well |
| T-1502 | work-completed | [REVIEW] Verify the fixed reading verbs surface real chat-arc content |
| T-1559 | work-completed | [REVIEW] Verify both `--watch` views are steady and useful |
| T-1570 | work-completed | [REVIEW] Verify `termlink_agent_poll_*` family is operator-fluent over MCP |
| T-1635 | work-completed | [REVIEW] Review response artifact and approve (or amend) before AEF coordination completes |
| T-1695 | work-completed | [REVIEW] Re-enable OneDev auto-mirror (optional but recommended) |
| T-1695 | work-completed | [REVIEW] Revoke the diagnostic PAT pasted in this session (ends `…7ehL`, ~93 chars long, fine-grained `github_pat_…` prefix) |
| T-1799 | started-work | [REVIEW] Rotate/revoke the compromised PAT on GitHub |
| T-1885 | started-work | [REVIEW] Run `fw independent-review --dry-run` and verify the per-task verdict lines read naturally — operator can scan for which  |
| T-1885 | started-work | [RUBBER-STAMP] At least one FAIL produced an auto-followup task with a useful RCA stub |
| T-2014 | work-completed | [REVIEW] Framework-agent prompt is operator-ready: complete enough that pasting it into the framework agent's session in `/opt/999 |
| T-2015 | started-work | [REVIEW] Framework-agent prompt at `docs/reports/T-2015-fw-upgrade-claudemd-clobber-framework-prompt.md` is operator-ready |
| T-2016 | started-work | [REVIEW] Framework-agent prompt at `docs/reports/T-2016-fw-upgrade-replay-arg-drop-framework-prompt.md` is operator-ready |
| T-2194 | started-work | [RUBBER-STAMP] After agent refreshes evidence, batch-click ripe partial-completes. **Steps:** open Watchtower /home, click through |
| T-2198 | started-work | [REVIEWER] Approve the T-2144 NO-GO decision. **Steps:** 1) read T-2144's updated Recommendation; 2) if you agree with the NO-GO c |
| T-2203 | started-work | [REVIEW] Choose Path A (`fw tier0 approve` + loop) or Path B (direct frontmatter sed). **Steps:** read AC 3's mechanism discovery, |
| T-2211 | work-completed | [REVIEW] The demo is a convincing operator-facing proof of the headline mechanic |
| T-2408 | work-completed | [RUBBER-STAMP] Close arc mcp-slimming with the demo evidence. |
| T-2409 | work-completed | [REVIEW] Decide who completes the ring20 host-side work (the residual for "whole fleet"). |
| T-2470 | started-work | [REVIEW] Bless the canonical purpose sentence |
| T-2486 | started-work | [REVIEW] Review exploration findings and approve go/no-go decision |
| T-2522 | captured | [REVIEW] **DECIDE: does `Days(N)` retention mean "keep N days by hub-receive-time" or "by client content-time"?** This is the gati |
| T-2532 | captured | [REVIEW] Resolve the four OPEN policy decisions in `## Decisions` (cap-at-all / per-host-vs-per-caller / default value / enforceme |
| T-2546 | started-work | [REVIEW] Review exploration findings and approve go/no-go decision |
| T-2550 | started-work | [REVIEW] Decide the fix direction for spawn's default success semantics |
| T-2566 | captured | Decision recorded on Q1 (queue / drop / coalesce presence on hub-unreachable), |
| T-2566 | captured | Decision recorded on Q2 (loop exit-after-N-queued: yes/no, N value, opt-in?), |
| T-2566 | captured | If either decision is "change behaviour", a follow-up build task is filed with |
| T-2567 | started-work | Decide: should `execute_capped` force `exit_code = -1` (or a distinct sentinel) |
| T-2567 | started-work | Consumer-contract audit: enumerate every `exec` / `termlink_run` / `batch_run` |
| T-2567 | started-work | If "force -1" is chosen, file a build task with the code change + a regression |
| T-2570 | captured | Decide whether non-goal #4 warrants any structural guard at all, given it is |
| T-2570 | captured | If a guard is wanted, scope what it would check (e.g. a review checklist item, |
| T-2570 | captured | Record the decision; if "no automated guard", close with that rationale so the |
| T-2576 | captured | Decide the contract shape: an at-risk signal on `LeasedClaim` (e.g. |
| T-2576 | captured | Decide whether the renew loop should also proactively surface this to the |
| T-2576 | captured | If "change the contract", file a build task with concrete ACs + a |
| T-2577 | captured | Decide the remedy among (a) sweep-side guard below min live cursor, (b) |
| T-2577 | captured | If "change behaviour", file a build task with concrete ACs + a load-bearing |
| T-2644 | captured | [REVIEW] Interactive attach surfaces dropped input without terminal corruption |
| T-2706 | started-work | [REVIEW] Decide cleanup vs exclusion for the demo topics |
| T-2708 | captured | [REVIEW] Is laptop-141 meant to be online? |
| T-2710 | captured | [REVIEW] Decide whether the remaining 7 canaries get fixture suites now or later |
| T-2711 | started-work | [RUBBER-STAMP] Decide whether U-001 is filed to the shared `framework:pickup` topic |
| T-2723 | work-completed | [RUBBER-STAMP] Decide whether U-008 is filed to the shared `framework:pickup` topic |
| T-2725 | started-work | [REVIEW] Review exploration findings and approve go/no-go decision |
| T-2753 | captured | [REVIEW] Review exploration findings and approve go/no-go decision |
| T-2819 | started-work | Run the one-time catch-up in the main checkout and review before committing. |
| T-2822 | work-completed | Commit the four static-check allowlists so the checks are clean off this machine. |
| T-2836 | work-completed | [REVIEW] Repaired episodics render correctly in Watchtower, and the two fallback files are readable |
| T-2837 | started-work | [REVIEW] The three `corpus_*.py` files are recovered into the vendored framework and committed from `/opt/termlink` |
| T-2837 | started-work | [REVIEW] T-2690/91/92 are renumbered on the two sibling branches before any merge |
| T-2837 | started-work | [REVIEW] P-043 is disposed of deliberately — acted on or dropped, not left stranded |

## C — Evidence: confirm real-world state an agent cannot observe

You must look at something an agent cannot see — a disk, a dashboard, a host that stopped rebooting. Several may already be satisfied; the agent simply cannot confirm.

| task | status | decision |
|---|---|---|
| T-1415 | started-work | [REVIEW] Verify production hubs have been running flag-off for ≥7 days |
| T-1415 | started-work | [REVIEW] Confirm bake metric is clean |
| T-1420 | started-work | [REVIEW] Full chat arc parity confirmed via fleet check |
| T-1442 | work-completed | [REVIEW] Spot-check by running cmd_spawn against a live hub and confirm `cat /tmp/tl-dispatch/spot/meta.json \| python3 -m json.too |
| T-1795 | work-completed | [REVIEW] Live confirm the fix on a populated hub |
| T-2210 | work-completed | [REVIEW] The captured evidence is sufficient to batch-confirm the rubber-stampable ACs |
| T-2212 | work-completed | [REVIEW] Confirm the cooperative-handoff demo is a faithful, useful proof of the canonical orchestrator pattern documented in `doc |
| T-2213 | work-completed | [REVIEW] Confirm the smoke suite is the right home for the two arc-demo regression gates (vs a separate CI entry). |
| T-2577 | captured | Confirm the boundary vs T-2562 (forever-archival guard) and the existing |
| T-2709 | started-work | [REVIEW] Confirm the recency window is the right semantic |

## D — Subjective sign-off: does it read/feel right

Taste calls on wording, layout and feel. Individually seconds; collectively 36. These are the ones burying everything else. Worth a single batched pass.

| task | status | decision |
|---|---|---|
| T-1426 | started-work | [REVIEW] Verify the warning is informative without being noisy |
| T-1429 | started-work | [REVIEW] Verify the verb's UX from a vendored-agent perspective |
| T-1430 | started-work | [REVIEW] Verify topic self-doc is discoverable from a fresh agent's perspective |
| T-1432 | started-work | [REVIEW] Verify the cut-readiness signal is actionable |
| T-1453 | started-work | [REVIEW] CLI feels right |
| T-1482 | work-completed | [REVIEW] Verify text-mode table is scannable for fleet observability |
| T-1484 | work-completed | [REVIEW] Verify the empty-with-filter message reads naturally |
| T-1485 | work-completed | [REVIEW] Verify timeout error wording is operator-actionable |
| T-1486 | work-completed | [REVIEW] Verify the watch view is steady (no flicker / no row jitter) |
| T-1487 | work-completed | [REVIEW] Verify the one-liner output is operator-scannable |
| T-1488 | work-completed | [REVIEW] Verify thread-filter output makes sense |
| T-1489 | work-completed | [REVIEW] Verify the truncation footer reads naturally |
| T-1490 | work-completed | [REVIEW] Verify the empty-with-thread message reads naturally |
| T-1491 | work-completed | [REVIEW] Verify the by-project table is operator-readable |
| T-1492 | work-completed | [REVIEW] Verify the recent-post output is operator-readable |
| T-1494 | work-completed | [REVIEW] Verify the watch view is steady (no flicker) and readable |
| T-1495 | work-completed | [REVIEW] Verify the overview is operator-readable as a "first command" of a session |
| T-1496 | work-completed | [REVIEW] Verify the live overview is steady and useful as a "leave it running" dashboard |
| T-1498 | work-completed | [REVIEW] Verify the live single-peer view is steady and useful for "babysit one peer" |
| T-1499 | work-completed | [REVIEW] Verify --msg-type filtering output is operator-readable |
| T-1500 | work-completed | [REVIEW] Verify timeline output is operator-readable as fleet "tail -f" |
| T-1501 | work-completed | [REVIEW] Verify --grep filtering output is operator-readable |
| T-1506 | work-completed | [REVIEW] Verify offset rendering reads naturally |
| T-1529 | work-completed | [REVIEW] Verify the verb reads naturally |
| T-1530 | work-completed | [REVIEW] Verify the verb reads naturally |
| T-1531 | work-completed | [REVIEW] Verify the verb reads naturally |
| T-1532 | work-completed | [REVIEW] Verify the verb reads naturally |
| T-1533 | work-completed | [REVIEW] Verify the verb reads naturally |
| T-1534 | work-completed | [REVIEW] Verify the verb reads naturally |
| T-1535 | work-completed | [REVIEW] Verify the verb reads naturally |
| T-1536 | work-completed | [REVIEW] Verify the verb reads naturally |
| T-1537 | work-completed | [REVIEW] Verify the verb reads naturally |
| T-1557 | work-completed | [REVIEW] Verify `agent typers --watch` is steady (no flicker / no jitter) |
| T-1558 | work-completed | [REVIEW] Verify `agent inbox --watch` reads naturally as live unread monitor |
| T-2209 | work-completed | [REVIEW] The five history skills read naturally and are discoverable alongside their base-verb siblings. |
| T-2385 | work-completed | [REVIEW] The loud WARNING wording is clear and actionable in a real send |

## E — Placeholder: template boilerplate, not a real decision

Not a real decision. Template text that was never edited. Safe to strike.

| task | status | decision |
|---|---|---|
| T-2197 | started-work | [REVIEW] Make GO/NO-GO/DEFER decision on each of the 4 inceptions per agent recommendations. **Steps:** read each task's Recommend |

## Method

Population read with the **live** `count_unchecked_human_acs` predicate imported from `.agentic-framework/web/shared.py` — the same function `/approvals` and `fw review-queue` call — never a reimplementation, so this cannot drift from what you actually see.

Classification is keyword-based over the AC text and is a triage aid, not a contract; a handful of rows will sit in the wrong bucket. The counts are exact, the buckets are advisory.

The census that produced it ran six fixtures **before** opening any repo file and aborts if they misclassify, because a census that asserts absence is worthless without a control that fails when the instrument is stubbed. Shape borrowed from 832-Workflow-designer (agent-chat-arc offset 399).

