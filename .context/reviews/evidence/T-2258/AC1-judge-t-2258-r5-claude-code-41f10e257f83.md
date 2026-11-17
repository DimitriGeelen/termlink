# T-2258 Human AC#1 — independent review (rung-5-panel:claude-code)

Revision reviewed: ccea0a0717d238c5b7974bfa6a28fac0b87c8341. Criterion judged by its restated title
(Restated 2026-11-17, V-20261117-f742f766).

## What the criterion needs
1. Fixed binary deployed to the .107 hub.
2. `timeout -s KILL 12 termlink channel state framework:pickup --hub 192.168.10.107:9100` returns within the bound.
3. On a scratch topic of 1500+ posts on that hub: a read right after a post, and a read during landing posts, return within the bound.
4. No probe post to framework:pickup.

## What I checked myself (read-only, 2026-11-17 ~23:00Z)
- **Deployed binary carries the fix.** `termlink fleet doctor --json` reports for 192.168.10.107:9100:
  `hub_version 0.12.13+135.gd167da0e2, hub_id cacc73ea32b121dd, status ok` — the same version the Measured block records.
  `git merge-base --is-ancestor db7c10a56 d167da0e2` succeeds (db7c10a56 = "T-2258: fix hub read-path stall — spawn_blocking
  for large-topic walks"); `git describe d167da0e2` = v0.12.13-135-gd167da0e2; channel.rs at that commit has 8 `spawn_blocking` uses.
- **Live repro command, run by me 5 times, exactly as the criterion writes it:**
  ```
  read 1 rc=0 241ms lines=20096
  read 2 rc=0 185ms lines=20096
  read 3 rc=0 199ms lines=20096
  read 4 rc=0 140ms lines=20096
  read 5 rc=0 225ms lines=20096
  ```
  rc=0 (not 137/KILL), real content returned ("Canonical state of 'framework:pickup':" + ~20k lines), far inside 12 s.
- **Scratch-topic part: recorded output checked against the script, not re-run** (re-running would post 1521 records to a live
  shared hub; I am read-only). `.context/working/t3814/repro-2258.sh` does what the Measured text says: creates
  `t3814-repro-<epoch>`, 1 + 1499 seed posts, cold read, one post then immediate read, 20 sequential posts in the background
  with 4 concurrent reads started at once, 3 reads of framework:pickup, then delete. Every read is wrapped in
  `timeout -s KILL 12`. `.context/working/t3814/repro-2258.out` matches the block in the task file line for line:
  all reads rc=0, 104–189 ms. "Deleted topic ... (1521 record(s) removed)" = 1500 + 1 + 20, which the hub itself counted, so the
  topic did hold 1500+ posts and all landing posts arrived.
- **No probe post to framework:pickup:** the script posts only to `$T` (the scratch topic); the first Measured block posts to
  `test-t3814-probe`. framework:pickup is only read.

## Remarks (do not change the verdict)
- The script's "hub binary:" line printed empty (its grep on fleet doctor JSON looks for `"version"`, the field is `hub_version`).
  The version is established separately by `hub.version` in the Measured text and by my own fleet doctor read.
- The 4 concurrent reads finish in ~110–190 ms while 20 sequential posts take longer, so the reads overlap the first landing
  posts rather than all 20. That still is a read "during landing posts" as the criterion states.

## Verdict
green — the recorded output shows the act as the criterion states it, and the parts I could repeat without writing to the
shared hub (deployed version, fix ancestry, the framework:pickup read) I repeated with the same result.
