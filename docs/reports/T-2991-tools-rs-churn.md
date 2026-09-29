# T-2991 — `tools.rs` per-tool churn ranking (180 days)

Measured 2026-09-29 on HEAD `ce37f2ddf`. Input: `crates/termlink-mcp/src/tools.rs`,
**374 commits** in the last 180 days (`git log --since=180.days`).

## Method

For each commit: take every diff hunk (`git show -U0 <sha> -- tools.rs`), map the hunk's
new-file start line to the MCP tool whose `#[tool(name = "termlink_…")]` marker most recently
precedes it **in that commit's version of the file**, and count the tool once per commit.
Ranking = distinct commits per tool region. Lines = sum of hunk sizes attributed.

**Attribution caveat (stated, not hidden).** A "region" runs from one marker to the next, so
free helper fns that live between two tools are attributed to the PRECEDING tool. That
over-counts tools followed by a big helper block — `termlink_chat_arc_broadcast` (90 commits,
7386 lines) is the clearest case; `termlink_channel_queue_status` and
`termlink_channel_ack_status` are likely inflated the same way. The ranking is therefore a
"which neighbourhood of the file moves most" signal, not a per-handler blame. Good enough for
choosing where parity assertions pay off first; not good enough to cite as a per-tool defect
rate.

## Script

```python
import subprocess,re,collections,bisect
path='crates/termlink-mcp/src/tools.rs'
shas=subprocess.run(['git','log','--since=180.days','--format=%h','--',path],capture_output=True,text=True).stdout.split()
marker=re.compile(r'name\s*=\s*"(termlink_[a-z0-9_]+)"')
per_tool=collections.Counter(); lines_per_tool=collections.Counter()
for sha in shas:
    diff=subprocess.run(['git','show','--format=','-U0',sha,'--',path],capture_output=True,text=True).stdout
    hunks=[(int(m.group(1)),int(m.group(2) or 1)) for m in re.finditer(r'^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@',diff,re.M)]
    if not hunks: continue
    content=subprocess.run(['git','show',f'{sha}:{path}'],capture_output=True,text=True).stdout.split('\n')
    marks=[(i,m.group(1)) for i,l in enumerate(content,1) if (m:=marker.search(l)) and l.lstrip().startswith(('#[tool','name'))]
    starts=[m[0] for m in marks]; touched=set()
    for start,n in hunks:
        j=bisect.bisect_right(starts,start)-1
        if j>=0: touched.add(marks[j][1]); lines_per_tool[marks[j][1]]+=max(n,1)
    for t in touched: per_tool[t]+=1
for t,c in per_tool.most_common(45): print(f"{c:3d} commits {lines_per_tool[t]:5d} lines  {t}")
```

## Result — top 45 of 262 tool regions touched

```
 90 commits  7386 lines  termlink_chat_arc_broadcast      (helper-block inflated, see caveat)
 44 commits   239 lines  termlink_help
 38 commits  4396 lines  termlink_channel_queue_status    (already asserted, PAIR 5)
 37 commits  2639 lines  termlink_channel_ack_status
 11 commits   111 lines  termlink_dispatch
 11 commits   597 lines  termlink_fleet_doctor
 10 commits  1111 lines  termlink_agent_search
  9 commits   259 lines  termlink_batch_tag
  8 commits   289 lines  termlink_agent_contact
  8 commits   220 lines  termlink_channel_subscribe
  8 commits   465 lines  termlink_channel_thread
  7 commits   128 lines  termlink_agent_inbox
  7 commits   138 lines  termlink_batch_exec
  7 commits   468 lines  termlink_file_send
  7 commits   198 lines  termlink_broadcast
  6 commits   441 lines  termlink_channel_state
  6 commits   116 lines  termlink_agent_dms
  6 commits    73 lines  termlink_channel_typing_emit
  6 commits   233 lines  termlink_inbox_list
  6 commits   105 lines  termlink_doctor
  6 commits   427 lines  termlink_deregister
  6 commits    57 lines  termlink_spawn
  5 commits   342 lines  termlink_agent_ask
  5 commits   209 lines  termlink_wait
  5 commits    37 lines  termlink_channel_claims_summary_all
  5 commits   102 lines  termlink_channel_claims_summary
  5 commits    94 lines  termlink_agent_chat_arc_recent
  5 commits   156 lines  termlink_agent_unanswered
  5 commits     6 lines  termlink_agent_threads_by
  5 commits    23 lines  termlink_exec
  5 commits    94 lines  termlink_channel_post
  5 commits   100 lines  termlink_channel_relations
  5 commits    50 lines  termlink_agent_reactions_of
  5 commits   147 lines  termlink_channel_pinned
  5 commits    33 lines  termlink_channel_reactions_of
  5 commits   103 lines  termlink_channel_pin
  5 commits    75 lines  termlink_channel_typing_list
  5 commits    50 lines  termlink_channel_poll_end
  5 commits   147 lines  termlink_agent_star
  5 commits   209 lines  termlink_channel_starred
  5 commits    49 lines  termlink_channel_unread
  5 commits    82 lines  termlink_agent_react
  5 commits   179 lines  termlink_channel_pin_history
  5 commits    53 lines  termlink_agent_timeline
```

## Selection for this slice

Hub-independent (or hub-DOWN-envelope) cases are the ones the harness can assert without a
live fixture, so the first slice takes those from the top-25 and the top-45 tail:
`termlink_help`, `termlink_doctor`, `termlink_channel_ack_status`, `termlink_channel_state`,
`termlink_channel_subscribe`, `termlink_channel_thread`, `termlink_channel_unread`,
`termlink_inbox_list`, `termlink_agent_search`. Tools needing a live hub or a peer
(`chat_arc_broadcast`, `dispatch`, `fleet_doctor` with hubs, `agent_contact`, `file_send`,
`batch_*`) stay on the allowlist for a later T-2748 slice. CLI verbs with no `--json` flag at
all (`batch tag`, `batch exec`, `deregister`, `agent chat-arc-recent`) cannot be asserted by
this harness until the CLI grows one — that is a CLI gap, noted for T-2748.

## Pre-measurement of the CLI side (hub down, empty runtime dir, empty HOME)

| verb | rc | stdout |
|---|---|---|
| `channel ack-status t1 --json` | 1 | `{"error":"Hub is not running …","ok":false}` |
| `channel state t1 --json` | 1 | same shape |
| `channel unread t1 --json` | 1 | same shape |
| `channel subscribe t1 --json` | 1 | same shape |
| `inbox list sess1 --json` | 1 | same shape (+ deprecation line on stderr) |
| `agent search needle --json` | 1 | **EMPTY** — anyhow error on stderr only |
| `doctor --json` | 0 | structured checks |
| `help --json` | 0 | category map |

`agent search --json` on hub-down prints no JSON: the T-1914 class (CLI early error path does
not honour `--json`) — see the T-2991 Evolution for the task that owns it.
