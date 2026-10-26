# T-3592 stage 4 — what is genuinely the operator's

Built 2026-10-26 from `fw task delegate --dry-run` over every human-owned active task (133),
after stages 1–2 of the sweep (operator ruling A). These are the criteria the delegation
ruleset keeps with the operator: inception GO/NO-GO decisions, acts in the world, and
sovereignty/Tier-0 items. The reviewer-judged criteria (74, stage 3) are NOT here; any of
those the reviewer escalates will be added.

Evidence was gathered for each item so the operator only confirms or decides. Groups are
ordered by effort: A is a glance, D is a real decision. Work through them one at a time.

## A. Already satisfied — confirm and close (a glance each)

| # | Task | Criterion | Evidence (checked 2026-10-26) |
|---|---|---|---|
| A1 | T-1696 | Release-mirror drift canary cron installed on .107 | `/etc/cron.d/termlink-release-mirror-canary` installed, byte-identical to tracked source; cron drift check: 45 installed + matching |
| A2 | T-3052 | Pending notify-rail cron installed | `termlink-notify-sidecar-supervisor`, `-canary`, `termlink-notify-wake-supervisor` installed; same drift check |
| A3 | T-1691 | GitHub Release published with binaries (v0.11.0) | `gh release list`: v0.11.0 published 2026-05-18 |
| A4 | T-1695 | Releases v0.10.0, v0.11.0, v0.11.1 published | `gh release list`: all three published 2026-05-18 |
| A5 | T-1635 | Approve the response artifact | Decision already recorded: **GO**; only the close is outstanding |
| A6 | T-2013 | Fixed binary on ring20 hubs (.122, then .121, .141) | .122 and .121 serve 0.12.1160 (runme 2026-10-26, fleet doctor ok); .141 is off (no route) — confirm .141 is out of scope or still owed |

## B. Needs one host action from you (a proposed runme action; nothing done yet)

| # | Task | What | Evidence |
|---|---|---|---|
| B1 | T-2870 | Remove the legacy unlocked pickup cron `/etc/cron.d/agentic-pickup-termlink` (T-1073) | It runs `fw pickup process` every minute with NO lock, while the audit crontab already runs the same job every minute behind `flock -n /var/lock/agentic-pickup-termlink.lock`. The legacy file is the remaining race. Proposal: runme action that backs it up, removes it, and verifies the locked line still runs |

## C. Not satisfied, or blocked by the world

| # | Task | Criterion | State |
|---|---|---|---|
| C1 | T-2861 | Next termlink session launched under `claude-fw` | This session runs as plain `claude --resume`, not under `claude-fw` |
| C2 | T-1420 | Binary deployed on .141 | .141 has no route (host off) |
| C3 | — | (observation, no task yet) | This session's TermLink registration lives inside the tmux server `fleet-999-Agentic-Engineering-Framework` (AEF's) — the cross-project tmux sharing of SEV-1 T-3549/T-3552 |

## D. Probably obsolete — the T-1166 bake era (T-1166 is completed)

Confirm "obsolete, close as superseded", or say which still matter:
T-1415 (flag-off ≥7 days; bake metric clean), T-1417 (zero `event.broadcast` callers after
reboot), T-1419 (freshness signal post-T-1418), T-1435 (CUT-READY verified under T-1418),
T-1632 (`hub.capabilities` on next .122 deploy), T-1633 (warning visible on next .122 deploy),
T-1795 (live confirm on a populated hub), T-2258 (fixed binary on .107, repro no longer hangs —
.107 now runs 0.12.1160), T-2297 (live end-to-end after rebuilt hub), T-1429, T-1558
(post-ship UX checks on old chat-arc verbs).

## E. Decisions about direction (one brief each, one at a time)

| # | Task | Question |
|---|---|---|
| E1 | T-2389 | When to re-home .107 agents under the T-2388 launcher |
| E2 | T-2409 | Who completes the ring20 host-side work |
| E3 | T-2522 | Does `Days(N)` retention mean hub-receive time or client time |
| E4 | T-2203 | Tier-0 path for the stale-status bulk flip (A: `fw tier0 approve`, B: direct edit) |
| E5 | T-2408 | Close arc mcp-slimming with the demo evidence (arc close is yours) |
| E6 | T-2972 | Post three approved replies to 055's DM topics (outward-facing) |

## F. Inception decisions (26) — what each task recommends

| Task | Recommends | Last touched | Topic |
|---|---|---|---|
| T-1635 | GO (already recorded, see A5) | 05-15 | v2 peer-consult seam |
| T-2197 | CLOSE (audit) | 09-27 | 4 inception limbo tasks |
| T-2198 | CLOSE (audit) | 09-27 | T-2144 empty Recommendation |
| T-2875 | NO-GO | 09-08 | Cross-session reach into a running Claude session |
| T-3320 | GO | 10-17 | TermLink settings surface |
| T-3372 | GO | 10-06 | TermLink under OpenCode |
| T-3385 | GO | 10-10 | OTP-style out-of-band approval |
| T-3524 | GO | 10-22 | wc/0.3 vector capabilities |
| T-2837 | "A" (not a verdict) | 10-26 | Cross-checkout operator actions (t2687 merge) |
| T-2984 | — (no Recommendation) | 09-22 | Structural consumer for framework:pickup |
| T-2486 | DEFER | 08-20 | Charter-core health review |
| T-2725 | DEFER | 08-23 | Herdr adoption research |
| T-2918 | DEFER | 09-08 | Two-way operator-channel proposal |
| T-2992 | DEFER | 09-20 | tools.rs + channel.rs split design |
| T-3169 | DEFER | 09-27 | Triage framework:pickup offsets |
| T-3321 | DEFER | 10-02 | Message compaction |
| T-3330 | DEFER | 10-02 | Receive-side telemetry |
| T-3333 | DEFER | 10-03 | Hub steward agent |
| T-3335 | DEFER | 10-03 | External consultation (055's F10 question lives here) |
| T-3439 | DEFER | 10-17 | Hub-routed PTY stream |
| T-3455 | DEFER | 10-18 | arc-011 PH-3 cross-hub delivery gaps |
| T-3476 | DEFER | 10-20 | Deterministic task-assignment sidecar |
| T-3477 | DEFER | 10-21 | Container isolation for workers |
| T-3534 | DEFER | 10-21 | Signed inject + spawn allowlist |
| T-3548 | DEFER | 10-22 | tmux env leak via termlink spawn |
| T-3553 | DEFER | 10-22 | SEV-1 fix 4: supervised lifetimes |

DEFER decisions need a `revisit_at` date and the evidence that would make the revisit
actionable (CLAUDE.md, T-1451); each brief will propose one.

## G. Misclassified — agent experience, not the operator's

T-1430 (topic self-doc discoverable from a fresh agent's view) and T-1487 (one-liner output
scannable): the ruleset says these belong under `### Agent`. They move to the agent.
