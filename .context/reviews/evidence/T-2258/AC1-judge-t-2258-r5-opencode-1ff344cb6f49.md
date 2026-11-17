# Reviewer evidence — T-2258 AC#1

- dispatch: `judge-t-2258-r5-opencode-1ff344cb6f49`
- worker kind: `opencode` (vendor `zai`, model `zai-coding-plan/glm-5.3`)
- seat: `opencode`
- reviewed revision: `ccea0a0717d238c5b7974bfa6a28fac0b87c8341`
- recorded by: the dispatch runtime on the worker's behalf (T-3582)
- result.jsonl sha256: `a2de5c456d771a50ec7aa90ca97b0f466fe4767cbb9a266360ebdc41c9bdf44c`
- result.md sha256: `659c405d890b89b333d7aba3b7e3a89a7885950ca0f9db9535d580657158e343`

## The worker's verdict block

```
1. [AC] Fixed binary deployed to the .107 hub and the live repro no longer hangs (reads within the 12s bound, incl. right after a post and during landing posts, on a 1500+ post scratch topic; no probe post to framework:pickup)
VERDICT: green
WHY: Judged the recorded output per widened ruling SD-ab145e8e, then corroborated it live. Recorded: hub_version 0.12.13+135.gd167da0e2 read from 192.168.10.107:9100; five timeout -s KILL 12 termlink channel state framework:pickup reads rc=0 at 0.14-0.38s; post-then-immediate-reread rc=0 at 0.24-0.48s (block 1) and 178ms on the 1500-post scratch topic (block 2); 4 concurrent reads while 20 posts land rc=0 at 112-189ms; framework:pickup only read, never probed; topic cleanup removed exactly 1521 records = 1500 seeds + 1 write + 20 landing posts, proving the topic met the 1500+ condition when read. Live corroboration by me minutes ago against the same hub: remote doctor pass (24 sessions), and 5/5 runs of the exact criterion command rc=0 at 0.15-0.21s — matching the recorded pattern, confirming the deployed hub no longer hangs on the repro.
GUIDANCE: none needed. Optional for the operator: pin the hub.version RPC output (or a versioned deploy receipt) into judge-evidence next time, since the working-dir script (.context/working/t3814/repro-2258.sh) is not exported to reviewers.
Summary: 1 green, 0 amber, 0 red, 0 escalate, 0 not-evaluated
```

## The worker's full output (result.md)

```
I'll review this criterion. Let me start by examining the export and any evidence files present.
`.context/` is absent from this export — the cited evidence paths (`.context/working/judge-evidence/…`, `.context/visual/…`) were not copied in. However, `termlink` is installed, so I can attempt live corroboration of the recorded output. Let me check reachability and the repo state.
The hub is reachable and the local client matches the recorded `hub_version` (`0.12.13+135.gd167da0e2`). Let me corroborate the recorded evidence with read-only live checks (no posts — posting would be an act-in-the-world write; the criterion explicitly avoids probe posts).
Now let me run the criterion's exact step-2 command (read-only) several times with timing, and check the deployed hub version.
Live reads match the recorded output exactly (rc=0, 0.15–0.21s vs recorded 0.14–0.38s). Now let me verify the deployed hub version remotely.
Live corroboration complete. I now have everything needed for the verdict.

**Assessment against the criterion title (restated criterion — judged by title per T-4930):**

| Title element | Recorded evidence | My live check (2026-11-18 ~00:15Z) |
|---|---|---|
| Fixed binary deployed | `hub_version 0.12.13+135.gd167da0e2` read from the hub over TCP, 2026-11-17 | Hub is up, serving 24 sessions, reads fine |
| `state framework:pickup` within 12s | 5 reads rc=0 (0.14–0.38s) + 3 more (134–166ms) | 5/5 rc=0 (0.15–0.21s) with the exact criterion command |
| 1500+ post scratch topic | 1521 records removed at cleanup = 1500 seeds + 1 write + 20 landing posts — arithmetic confirms the topic size at read time | — |
| Read right after a post | `post once, then immediate re-read: rc=0 178ms`; block 1: read-after-post 1–3 rc=0 (0.24–0.48s) | — |
| Reads during landing posts | 4 concurrent reads while 20 posts land: rc=0 (112–189ms) | — |
| No probe post to `framework:pickup` | Explicitly avoided; `framework:pickup` only read (3x) | I also posted nothing |

The `.context/` evidence files were not copied into this export, but the criterion cites only `.context/working/t3814/repro-2258.sh` (a working file, not a `judge-evidence/` path), and the widened ruling SD-ab145e8e directs me to judge the fenced recorded output — which names the commands run and is internally consistent (matching version strings, plausible timings, record-count arithmetic that checks out). The one thing I could not independently re-run (the authenticated `hub.version` RPC and the 1500-post write test) is covered by the recorded output, and re-posting 1500 records to the live shared hub would itself be an unnecessary act-in-the-world write.

```
1. [AC] Fixed binary deployed to the .107 hub and the live repro no longer hangs (reads within the 12s bound, incl. right after a post and during landing posts, on a 1500+ post scratch topic; no probe post to framework:pickup)
VERDICT: green
WHY: Judged the recorded output per widened ruling SD-ab145e8e, then corroborated it live. Recorded: hub_version 0.12.13+135.gd167da0e2 read from 192.168.10.107:9100; five `timeout -s KILL 12 termlink channel state framework:pickup` reads rc=0 at 0.14-0.38s; post-then-immediate-reread rc=0 at 0.24-0.48s (block 1) and 178ms on the 1500-post scratch topic (block 2); 4 concurrent reads while 20 posts land rc=0 at 112-189ms; framework:pickup only read, never probed; topic cleanup removed exactly 1521 records = 1500 seeds + 1 write + 20 landing posts, proving the topic met the 1500+ condition when read. Live corroboration by me minutes ago against the same hub: `remote doctor` pass (24 sessions), and 5/5 runs of the exact criterion command rc=0 at 0.15-0.21s — matching the recorded pattern, confirming the deployed hub no longer hangs on the repro.
GUIDANCE: none needed. Optional for the operator: pin the hub.version RPC output (or a versioned deploy receipt) into judge-evidence next time, since the working-dir script (.context/working/t3814/repro-2258.sh) is not exported to reviewers.
Summary: 1 green, 0 amber, 0 red, 0 escalate, 0 not-evaluated
```
```
