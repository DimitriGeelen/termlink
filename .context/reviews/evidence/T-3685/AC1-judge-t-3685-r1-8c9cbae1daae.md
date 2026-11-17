# T-3685 Human AC#1 — Create testdev's token at the terminal

Reviewer: judge-t-3685-r1-8c9cbae1daae (rung-1-same-vendor-independent), revision 7dbd8bfb9d4f3d4595855f2c2fbb8c17d2d58a01
Verdict: **green**

## What I checked (all read-only, run by me on 2026-11-17)

1. **Re-ran the Measured command.** Both log files exist and return the two recorded lines verbatim:
   - `runme-20261111T115459Z-3899365.log:224` — `OK      testdev has its own long-lived token (/home/testdev/.config/claude/oauth-token, mode 600, loaded by /home/testdev/.profile); one prompt as testdev answered OK`
   - `runme-20261117T090743Z-3609132.log:250` — `skip    testdev's stored token answers (one prompt as testdev returned OK)`
2. **Context of the OK line.** Log header: `runme.sh start 2026-11-11T11:54:59Z args=[] uid=0` (a real run, not --dry-run). Line 222 is the action-28 heading, line 223 `running 'claude setup-token' as testdev ON YOUR TERMINAL (not logged): open the link and log in`, line 224 the Expected line. That is the criterion's Expected string, printed by action 28.
3. **The OK line cannot be printed without the act.** `runme.sh` ~3930-3962: the line is reached only after a token starting `sk-ant-oat` was read from the terminal, written to the token file, mode 600 confirmed, the profile marker confirmed, and `testdev_token_answers` (one live prompt as testdev) succeeded.
4. **Live state on the host.** `/home/testdev/.config/claude/oauth-token`: owner testdev, mode 600, 109 bytes, begins `sk-ant-oat` (value not read further). `/home/testdev/.profile:30` exports `CLAUDE_CODE_OAUTH_TOKEN` from that file. Directory is testdev 700.
5. **All action-28 outcomes across the run logs.** 2026-11-11T08:30Z FAILED (stored, did not answer); 2026-11-11T11:54Z OK; 2026-11-12..13 skip (stored token answers); 2026-11-14T00:57Z OK again (terminal flow ran a second time); every run since, including 2026-11-17 08:50Z and 09:07Z, skip (stored token answers).

## Observation (does not change the verdict)

The token file's mtime is 2026-11-14 10:16:13 +0100, matching the end of run `runme-20261114T005749Z-2053325.log`, which also printed the OK line. So the token on disk today is the one the operator created on 2026-11-14, not the 2026-11-11 one the Measured note describes. The Measured block omits that run. The act was therefore done twice at the terminal; the criterion is met either way, and the 2026-11-17 line proves the current token answers. Why the 2026-11-11 token needed replacing three days later is not recorded in the logs I read; if it expired or was revoked, "long-lived" is worth watching, but the current token has answered on every run for three days.

## Limits

- Logs are under `.context/working/` (gitignored), so they are not bound to the reviewed revision; I read them on disk as root-owned files with mtimes matching their run times.
- I did not run a fresh prompt as testdev (an outside call with a credential); I relied on the 09:07Z run of today.
- The brief's `bin/fw` does not exist here; I used `.agentic-framework/bin/fw`.
- Widening SD-ab145e8e (ruling T-3800, tasks include T-3685, pattern includes this criterion's title, expires 2027-02-15) is present in `.context/reviews/standing-delegations.jsonl`.
