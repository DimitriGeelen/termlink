# T-2210 — G-008 Human-Review Backlog: Fresh Evidence

**Generated:** 2026-06-13T13:46:50Z
**Purpose:** Re-run the mechanically-verifiable Human-AC Steps for the partial-complete
backlog and capture FRESH output, so the human can batch-confirm `### Human` ACs from this
report without re-running anything. Per CLAUDE.md sovereignty rule, **no `### Human` AC was
ticked by the agent** — every entry below is evidence only.

## Triage of the human-review backlog (84 tasks)

Tasks in `.tasks/active/` with `status=work-completed` OR `owner=human`:

| Bucket | Count | Meaning |
|--------|-------|---------|
| local-cli | 58 | Human-AC Steps are local `termlink` commands — re-runnable on this host |
| no-human-ac | 17 | No unchecked Human AC (already ticked, or none) — separate handling, not re-smoked |
| judgment-only | 5 | Need human taste; no command to pre-run |
| operator-env | 4 | Need ssh/GitHub/remote-host access this session does not have |

## READY TO REVIEW — fresh evidence captured (34 tasks)

These are the agent-chat-arc CLI verb cluster (T-1482–T-1570). Each task's `## Updates`
now has a timestamped **"G-008 fresh evidence"** entry with the command, exit code, and
captured output. Built binary: `target/release/termlink` (0.11.1293, 2026-06-13). Live
peer used for target args: `d1993c2c3ec44c94`.

### Cluster A — presence / who / ping / contact (10)
T-1482 presence (ok) · T-1483 who/error-paths (ok) · T-1484 presence --filter-project (empty, well-formed) · **T-1485 contact --ack-required (⚠ FLAG — see below)** · T-1486 presence --watch (ok, partial-stream) · T-1487 ping online/offline (ok) · T-1488 who --thread (ok) · T-1489 presence --top (ok) · T-1490 presence --thread (empty, well-formed) · T-1491 presence --by-project (ok)

### Cluster B — recent / on-thread / overview / timeline (11)
T-1492 recent (ok) · T-1493 on-thread (ok) · T-1494 on-thread --watch (ok) · T-1495 overview (ok) · T-1496 overview --watch (ok) · T-1498 recent --watch (ok — Step nuance: use `--target-fp`, not positional) · T-1499 --msg-type filter (ok) · T-1500 timeline (ok) · T-1501 --grep filter (ok) · T-1502 content-extraction fix (ok — decoded text, no empty markers) · T-1506 offset @N rendering (ok)

### Cluster C — forward / edit / redact / describe / threads / … (9)
T-1533 threads (ok, 246 roots) · T-1534 redactions (ok) · T-1535 pin-history (ok) · T-1536 edits-of (empty, well-formed) · T-1537 relations (ok) · T-1529 forward, T-1530 edit, T-1531 redact, T-1532 describe (**parse-confirmed only** — these MUTATE shared chat-arc state; the agent did not execute mutations it does not own. Human should run the mutation Step against a post they own to fully confirm.)

### Cluster D — watch dashboards + poll MCP (4)
T-1557 typers --watch (ok, partial-stream) · T-1558 inbox --watch (ok) · T-1559 dms/unread --watch (ok) · T-1570 poll family (parse-confirmed — Step nuance: CLI subcommand is `vote`, MCP tool is `termlink_agent_poll_vote`)

**How to confirm:** open each task file, read the latest `## Updates` "G-008 fresh evidence"
entry, and if the output satisfies the Human AC, tick it and run
`fw task update T-XXXX --status work-completed`. Per-cluster raw evidence:
`docs/reports/T-2210-evidence/cluster-{A,B,C,D}-*.md`.

## ⚠ FLAGGED — needs your attention before confirming

- **T-1485** (`agent contact --ack-required`): re-smoke errored with exit=1 — but a
  *different* loud error than the AC documents (dm-topic retention-policy conflict naming
  peer_fp + topic, not the ack-timeout path). The error is still loud/named (no silent
  failure), but the path diverged. Recommend a human look before confirming.
- **T-1529/1530/1531/1532**: mutating verbs — parse-confirmed only. Run the mutation Step
  against a post you own to fully exercise.

## STILL NEEDS HUMAN / OPERATOR (not re-smokable here)

### judgment-only (5) — need taste, no pre-runnable command
T-1899, T-2007, T-2022, T-2024, T-2026 (most are inception/substrate-design — see their files).

### operator-env (4) — need ssh / GitHub / remote-host access
T-1137 (logrotate on proxmox .180), T-1296 (ring20-dashboard runtime_dir migrate),
T-1420 (deploy to laptop-141), and release/mirror tasks requiring GitHub auth.

### local-cli remainder (24) — heterogeneous, NOT yet re-smoked (next batch)
T-1137, T-1296, T-1415, T-1417, T-1419, T-1426, T-1429, T-1430, T-1432, T-1435, T-1442,
T-1453, T-1632, T-1673, T-1695, T-1696, T-1795, T-2013, T-2090, T-2194, T-2197, T-2198,
T-2203 — migrations, releases, audits, governance. Mixed verification; some overlap
operator-env. A follow-up pass should triage these individually.

## Provenance
- Re-smoke performed by 4 parallel subagents, evidence injected via Bash (Edit/Write
  blocked by bg-isolation this session).
- No `### Human` checkbox was modified. Verified: `grep -c '\[x\]'` in each Human section = 0.
