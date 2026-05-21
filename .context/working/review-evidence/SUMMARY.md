# Review-Evidence Sweep — 2026-05-22

Read-only evidence gathered for tasks in `fw review-queue` (Human ACs awaiting
verification). **No Human ACs were ticked** — agents gathered evidence only;
ticking is the operator's call. Per-task captured output in `batchN.md`.

## Verdict legend
- **EVIDENCE-CLEAN** — verb ran, sane non-error output; ready for glance-and-tick.
- **EVIDENCE-EMPTY** — verb ran cleanly but fleet-quiet window returned no rows;
  empty-state phrasing is evidenced, but the *populated* table/section the AC
  targets needs a re-glance during active fleet traffic.
- **HUMAN-VISUAL-ONLY** — `--watch` steadiness/no-flicker is inherently a live
  terminal judgment; command starts clean, but text capture can't prove it.
- **HELP-ONLY** — mutating verb with no `--dry-run`; interface evidenced via
  `--help`, live mutation skipped to protect real chat-arc data.
- **MCP-VERIFIED** — MCP tool registrations confirmed in source + live lifecycle.

## Release / infra (checked directly, not via agents)
| Task | Verdict | Evidence |
|---|---|---|
| T-1691 v0.11.0 release | ✅ CLEAN | GitHub `DimitriGeelen/termlink` has v0.11.0 + checksums.txt + darwin aarch64/x86_64 + linux aarch64/x86_64/static. [RUBBER-STAMP] satisfied. |
| T-1673 v0.10.0 release | ✅ CLEAN (artifacts) | Same full asset set on v0.10.0. brew-upgrade sub-step needs an operator host. |
| T-1696 canary cron | ❌ NOT DONE | Cron entry absent from host crontab; freshness check exits 1 (drift). Human must install per task steps. |

## agent-* verb batch (via reviewer subagents)
| Verdict | Count | Tasks |
|---|---|---|
| EVIDENCE-CLEAN | 19 | T-1483,1484,1485,1487,1488,1490,1492,1499,1500,1501,1502,1506,1533,1534,1535,1536,1537,1558,1559 |
| EVIDENCE-EMPTY | 5 | T-1482,1489,1491,1493,1495 (re-glance when fleet active) |
| HUMAN-VISUAL-ONLY | 5 | T-1486,1494,1496,1498,1557 (watch steadiness — eyeball) |
| HELP-ONLY | 4 | T-1529,1530,1531,1532 (mutating; interface verified) |
| MCP-VERIFIED | 1 | T-1570 (poll start/vote/end) |

## Defect / anomaly flags
1. **`agent on-thread <T-XXX>` returns empty even unfiltered**, while
   `agent timeline --thread <T-XXX>` returns matching posts for the same thread.
   Affects only the on-thread arm of T-1499/T-1501 (independently proven on
   recent + timeline). Possibly same thread-key-shape class as T-1502.
   Candidate new bug task.
2. **G-058 mirror drift LIVE:** GitHub branch HEAD is ~115 commits behind OneDev
   despite the 2026-05-18 release tags mirroring across. Owned by T-1695
   (status: issues).
3. **Side-effect during evidence run:** batch5's T-1570 poll lifecycle test
   posted offsets 1809–1813 to the real `agent-chat-arc` topic (a closed
   throwaway poll). Non-destructive but touched shared data.
