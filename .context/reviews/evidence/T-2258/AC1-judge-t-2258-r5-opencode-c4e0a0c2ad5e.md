# Reviewer evidence — T-2258 AC#1

- dispatch: `judge-t-2258-r5-opencode-c4e0a0c2ad5e`
- worker kind: `opencode` (vendor `zai`, model `zai-coding-plan/glm-5.3`)
- seat: `opencode`
- reviewed revision: `37c945a39df76fc1f82450555b8e5852542a78c2`
- recorded by: the dispatch runtime on the worker's behalf (T-3582)
- result.jsonl sha256: `d6d04396ca169d690b05baad19e7c3a5338247af2e5495bb03a59607f3bd0e45`
- result.md sha256: `e8f78b4c4b0300bf48f4f4a1726abe1a515ac153d63fca34d94ad2ae46f9ae67`

## The worker's verdict block

```
1. [AC] Fixed binary deployed to the .107 hub and the live repro no longer hangs
VERDICT: green
WHY: Judged by the restated title. The recorded Measured output names the commands and shows every element of the act: hub_version 0.12.13+135.gd167da0e2 read via hub.version on 192.168.10.107:9100 (operator runme deploy 2026-11-17 08:50Z); five framework:pickup reads under timeout -s KILL 12 all rc=0 in 0.14–0.38s; post→immediate re-reads on a scratch topic rc=0 0.24–0.48s; and on a 1500-post scratch topic a cold read rc=0 104ms, a post-then-immediate re-read rc=0 178ms, and 4 concurrent reads during 20 landing posts rc=0 112–189ms, with the topic deletion removing 1521 records — exactly 1500 seeds + 1 write + 20 landing posts, corroborating the large-topic condition. No probe post was sent to framework:pickup (read-only there, 3× rc=0 134–166ms), as the title requires. I also independently reproduced the core claim live from this sandbox: the exact criterion command returned rc=0 three times (0.14s/0.17s, 1.34MB) against the live hub today, and the fix (spawn_blocking in both read walks, channel.rs:1619/:1861, plus the t2258 regression guard at :5357) is present in the source at this revision, whose client reports the same 0.12.13+135.gd167da0e2 lineage as the deployed hub. The output is neither missing nor ambiguous and shows the act as stated.
GUIDANCE: None needed; the criterion is met on recorded evidence and live corroboration.
Summary: 1 green, 0 amber, 0 red, 0 escalate, 0 not-evaluated
```

## The worker's full output (result.md)

```
The task file matches the criterion text exactly. The measured evidence is inline in the criterion. Let me corroborate what I can locally: the fix in the source at this revision, and whether the hub is reachable read-only from this sandbox.
Fix confirmed in source at this revision (`spawn_blocking` in both read walks at channel.rs:1619 and :1861; regression test at :5357). Now let me find the right version command and run the criterion's exact read command (read-only, no posts).
The exact criterion command returned rc=0, ~1.3MB, well inside the bound — live from this sandbox. Let me time two more reads and check the hub version line in the output.
Live reads corroborate: 0.17s, 0.14s, rc=0 — matching the recorded 0.14–0.38s. Let me identify the hub's running version via a read command.
All checks are complete. The hub is live and serving; my independent reads corroborate the recorded evidence. Printing the verdict.

---

**Evidence summary of what I checked:**

1. **Recorded Measured blocks** (criterion text, mirrored verbatim in the task file at `.tasks/active/T-2258-fix-hub-read-path-stall--channelstatesub.md:167-194`): hub_version `0.12.13+135.gd167da0e2` read via `hub.version`; five `timeout -s KILL 12 termlink channel state framework:pickup --hub 192.168.10.107:9100` reads all rc=0 (0.14–0.38s); post→immediate re-reads on scratch topic `test-t3814-probe` rc=0 (0.24–0.48s); 1500-post scratch topic: cold read rc=0 104ms, post-then-immediate-re-read rc=0 178ms, 4 concurrent reads while 20 posts land rc=0 112–189ms; topic deletion removed **1521 records = 1500 seeds + 1 write + 20 landing posts** (arithmetic consistent with the 1500+ condition); `framework:pickup` was only **read** (3× rc=0 134–166ms), never posted to — exactly what the restated title demands.
2. **Independent live corroboration from this sandbox** (read-only, no posts sent): the exact criterion command returned rc=0 three times against 192.168.10.107:9100 — 0.17s and 0.14s timed, 1.34MB of state — matching the recorded latencies; `channel info` confirms a large, live, actively-fetched topic (361 posts, 145k fetches).
3. **Fix present at this revision**: `spawn_blocking` in both large-topic read walks (`crates/termlink-hub/src/channel.rs:1619`, `:1861`) plus the regression guard `channel_subscribe_no_hang_under_concurrent_walks_t2258` (`channel.rs:5357`); the sandbox's `termlink` client reports lineage `0.12.13+135.gd167da0e2` — identical to the recorded hub_version.

---

```
1. [AC] Fixed binary deployed to the .107 hub and the live repro no longer hangs
VERDICT: green
WHY: Judged by the restated title. The recorded Measured output names the commands and shows every element of the act: hub_version 0.12.13+135.gd167da0e2 read via hub.version on 192.168.10.107:9100 (operator runme deploy 2026-11-17 08:50Z); five framework:pickup reads under `timeout -s KILL 12` all rc=0 in 0.14–0.38s; post→immediate re-reads on a scratch topic rc=0 0.24–0.48s; and on a 1500-post scratch topic a cold read rc=0 104ms, a post-then-immediate re-read rc=0 178ms, and 4 concurrent reads during 20 landing posts rc=0 112–189ms, with the topic deletion removing 1521 records — exactly 1500 seeds + 1 write + 20 landing posts, corroborating the large-topic condition. No probe post was sent to framework:pickup (read-only there, 3× rc=0 134–166ms), as the title requires. I also independently reproduced the core claim live from this sandbox: the exact criterion command returned rc=0 three times (0.14s/0.17s, 1.34MB) against the live hub today, and the fix (spawn_blocking in both read walks, channel.rs:1619/:1861, plus the t2258 regression guard at :5357) is present in the source at this revision, whose client reports the same 0.12.13+135.gd167da0e2 lineage as the deployed hub. The output is neither missing nor ambiguous and shows the act as stated.
GUIDANCE: None needed; the criterion is met on recorded evidence and live corroboration.
Summary: 1 green, 0 amber, 0 red, 0 escalate, 0 not-evaluated
```
```
