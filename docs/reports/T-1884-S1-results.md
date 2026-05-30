# T-1884 S1 — classifier output

Total tasks scanned with unchecked ### Human ACs: 63
Total unchecked ACs: 72

## Class distribution

| Class | Count | Confident | Confident% |
|---|---:|---:|---:|
| REVIEW-CLI | 32 | 30 | 94% |
| RUBBER-STAMP-MECHANICAL | 9 | 9 | 100% |
| REVIEW-RENDER | 8 | 7 | 88% |
| OBSERVE-INFRA | 7 | 7 | 100% |
| OPERATOR-ACTION | 6 | 6 | 100% |
| OTHER | 6 | 0 | 0% |
| TIME-GATED | 3 | 3 | 100% |
| RUBBER-STAMP-RELEASE | 1 | 1 | 100% |

**Overall confidence: 63/72 = 87.5%**

GO threshold = ≥80%. PASS

## Per-AC classification

| Task | Prefix | Class | Conf | AC preview |
|---|---|---|:---:|---|
| T-1137 | `REVIEW` | OPERATOR-ACTION | ✓ | /var/log on proxmox .180 is below 50 % after rotation + daily cron activ |
| T-1137 | `REVIEW` | OBSERVE-INFRA | ✓ | CT 200 (.122) stops rebooting |
| T-1296 | `RUBBER-STAMP` | RUBBER-STAMP-MECHANICAL | ✓ | Apply same migration recipe as T-1294 AC 2 |
| T-1296 | `RUBBER-STAMP` | OBSERVE-INFRA | ✓ | Re-pin from .102 + fleet doctor green |
| T-1296 | `RUBBER-STAMP` | OBSERVE-INFRA | ✓ | Verify CT 101 reboot persistence (ground truth) |
| T-1415 | `REVIEW` | TIME-GATED | ✓ | Verify production hubs have been running flag-off for ≥7 days |
| T-1415 | `REVIEW` | REVIEW-CLI | ✓ | Confirm bake metric is clean |
| T-1417 | `REVIEW` | OBSERVE-INFRA | ✓ | Audit shows zero `event.broadcast` callers from this codebase after prod |
| T-1419 | `REVIEW` | OBSERVE-INFRA | ✓ | Post-deploy of T-1418, the freshness signal correctly distinguishes live |
| T-1420 | `RUBBER-STAMP` | RUBBER-STAMP-MECHANICAL | ✓ | Binary deployed on .141 — Method A or B |
| T-1420 | `RUBBER-STAMP` | OBSERVE-INFRA | ✓ | .141 hub restarted on new binary |
| T-1420 | `REVIEW` | OBSERVE-INFRA | ✓ | Full chat arc parity confirmed via fleet check |
| T-1426 | `REVIEW` | OTHER | ? | Verify the warning is informative without being noisy |
| T-1429 | `REVIEW` | REVIEW-CLI | ✓ | Verify the verb's UX from a vendored-agent perspective |
| T-1430 | `REVIEW` | REVIEW-CLI | ✓ | Verify topic self-doc is discoverable from a fresh agent's perspective |
| T-1431 | `RUBBER-STAMP` | RUBBER-STAMP-MECHANICAL | ✓ | Verify the skill works end-to-end from a real session |
| T-1432 | `REVIEW` | OTHER | ? | Verify the cut-readiness signal is actionable |
| T-1435 | `REVIEW` | REVIEW-RENDER | ✓ | Verification of CUT-READY happens under T-1418, not here |
| T-1442 | `REVIEW` | OTHER | ? | Spot-check by running cmd_spawn against a live hub and confirm `cat /tmp |
| T-1453 | `REVIEW` | REVIEW-CLI | ✓ | CLI feels right |
| T-1457 | `REVIEW` | OPERATOR-ACTION | ✓ | Decide whether .141 needs a peer Claude attached at all, or whether hear |
| T-1457 | `RUBBER-STAMP` | RUBBER-STAMP-MECHANICAL | ✓ | Operator action on .141 (only if peer Claude is required) |
| T-1482 | `REVIEW` | REVIEW-RENDER | ✓ | Verify text-mode table is scannable for fleet observability |
| T-1483 | `REVIEW` | REVIEW-CLI | ✓ | Verify error messages name the failing input clearly |
| T-1484 | `REVIEW` | REVIEW-CLI | ✓ | Verify the empty-with-filter message reads naturally |
| T-1485 | `REVIEW` | REVIEW-CLI | ✓ | Verify timeout error wording is operator-actionable |
| T-1486 | `REVIEW` | REVIEW-RENDER | ✓ | Verify the watch view is steady (no flicker / no row jitter) |
| T-1487 | `REVIEW` | REVIEW-CLI | ✓ | Verify the one-liner output is operator-scannable |
| T-1488 | `REVIEW` | REVIEW-CLI | ✓ | Verify thread-filter output makes sense |
| T-1489 | `REVIEW` | REVIEW-CLI | ✓ | Verify the truncation footer reads naturally |
| T-1490 | `REVIEW` | REVIEW-CLI | ✓ | Verify the empty-with-thread message reads naturally |
| T-1491 | `REVIEW` | REVIEW-CLI | ✓ | Verify the by-project table is operator-readable |
| T-1492 | `REVIEW` | REVIEW-CLI | ✓ | Verify the recent-post output is operator-readable |
| T-1493 | `REVIEW` | REVIEW-CLI | ✓ | Verify the on-thread reading view scans well |
| T-1494 | `REVIEW` | REVIEW-RENDER | ✓ | Verify the watch view is steady (no flicker) and readable |
| T-1495 | `REVIEW` | REVIEW-CLI | ✓ | Verify the overview is operator-readable as a "first command" of a sessi |
| T-1496 | `REVIEW` | REVIEW-RENDER | ✓ | Verify the live overview is steady and useful as a "leave it running" da |
| T-1498 | `REVIEW` | REVIEW-RENDER | ✓ | Verify the live single-peer view is steady and useful for "babysit one p |
| T-1499 | `REVIEW` | REVIEW-CLI | ✓ | Verify --msg-type filtering output is operator-readable |
| T-1500 | `REVIEW` | REVIEW-CLI | ✓ | Verify timeline output is operator-readable as fleet "tail -f" |
| T-1501 | `REVIEW` | REVIEW-CLI | ✓ | Verify --grep filtering output is operator-readable |
| T-1502 | `REVIEW` | REVIEW-CLI | ? | Verify the fixed reading verbs surface real chat-arc content |
| T-1506 | `REVIEW` | REVIEW-CLI | ✓ | Verify offset rendering reads naturally |
| T-1529 | `REVIEW` | REVIEW-CLI | ✓ | Verify the verb reads naturally |
| T-1530 | `REVIEW` | REVIEW-CLI | ✓ | Verify the verb reads naturally |
| T-1531 | `REVIEW` | REVIEW-CLI | ✓ | Verify the verb reads naturally |
| T-1532 | `REVIEW` | REVIEW-CLI | ✓ | Verify the verb reads naturally |
| T-1533 | `REVIEW` | REVIEW-CLI | ✓ | Verify the verb reads naturally |
| T-1534 | `REVIEW` | REVIEW-CLI | ✓ | Verify the verb reads naturally |
| T-1535 | `REVIEW` | REVIEW-CLI | ✓ | Verify the verb reads naturally |
| T-1536 | `REVIEW` | REVIEW-CLI | ✓ | Verify the verb reads naturally |
| T-1537 | `REVIEW` | REVIEW-CLI | ✓ | Verify the verb reads naturally |
| T-1557 | `REVIEW` | REVIEW-RENDER | ✓ | Verify `agent typers --watch` is steady (no flicker / no jitter) |
| T-1558 | `REVIEW` | REVIEW-CLI | ✓ | Verify `agent inbox --watch` reads naturally as live unread monitor |
| T-1559 | `REVIEW` | REVIEW-RENDER | ? | Verify both `--watch` views are steady and useful |
| T-1570 | `REVIEW` | REVIEW-CLI | ✓ | Verify `termlink_agent_poll_*` family is operator-fluent over MCP |
| T-1632 | `REVIEW` | TIME-GATED | ✓ | On next .122 deploy (after T-1166 bake clears), `hub.capabilities` retur |
| T-1633 | `REVIEW` | TIME-GATED | ✓ | On next .122 deploy (post-bake), the warning is visible in hub stderr/jo |
| T-1635 | `REVIEW` | OPERATOR-ACTION | ✓ | Review response artifact and approve (or amend) before AEF coordination  |
| T-1673 | `REVIEW` | REVIEW-CLI | ? | Confirm release pipeline produced artifacts |
| T-1691 | `RUBBER-STAMP` | RUBBER-STAMP-RELEASE | ✓ | GitHub Release published with macOS + Linux binaries |
| T-1695 | `REVIEW` | OPERATOR-ACTION | ✓ | Re-enable OneDev auto-mirror (optional but recommended) |
| T-1695 | `REVIEW` | OPERATOR-ACTION | ✓ | Revoke the diagnostic PAT pasted in this session (ends `…7ehL`, ~93 char |
| T-1695 | `REVIEW` | OTHER | ? | Releases published on GitHub for v0.10.0, v0.11.0, v0.11.1 (the GH Actio |
| T-1696 | `RUBBER-STAMP` | RUBBER-STAMP-MECHANICAL | ✓ | Cron entry installed in /etc/cron.d on .107 |
| T-1722 | `RUBBER-STAMP` | RUBBER-STAMP-MECHANICAL | ✓ | Upstream landed on `/opt/999-AEF` `origin/master`. |
| T-1723 | `RUBBER-STAMP` | RUBBER-STAMP-MECHANICAL | ✓ | Cron entry installed on .107 so the meta-canary actually fires. |
| T-1795 | `REVIEW` | OTHER | ? | Live confirm the fix on a populated hub |
| T-1799 | `REVIEW` | OPERATOR-ACTION | ✓ | Rotate/revoke the compromised PAT on GitHub |
| T-1836 | `RUBBER-STAMP` | RUBBER-STAMP-MECHANICAL | ✓ | MCP listing shows the three new tools |
| T-1841 | `RUBBER-STAMP` | RUBBER-STAMP-MECHANICAL | ✓ | Skill discoverable and invokable from Claude Code |
| T-1884 | `REVIEW` | OTHER | ? | Review exploration findings and approve go/no-go decision |

## OTHER bucket (manual sort needed)

- T-1426: `REVIEW` — Verify the warning is informative without being noisy
- T-1432: `REVIEW` — Verify the cut-readiness signal is actionable
- T-1442: `REVIEW` — Spot-check by running cmd_spawn against a live hub and confirm `cat /tmp
- T-1695: `REVIEW` — Releases published on GitHub for v0.10.0, v0.11.0, v0.11.1 (the GH Actio
- T-1795: `REVIEW` — Live confirm the fix on a populated hub
- T-1884: `REVIEW` — Review exploration findings and approve go/no-go decision

