# Close review: four tasks whose remaining step is an action in the world

Project: TermLink, host .107. Measured 2026-11-15 ~19:50Z by the agent. Every command below was run on the host, and its output is quoted verbatim.

Rule being applied: a task may be closed by the agent only if the action its remaining criterion describes is shown, by measurement, to have happened. Your job is to judge whether the evidence shows that, for each task separately.

## T-2858 — Install-path drift: /root/.local/bin/termlink was stale behind the other two install paths
Remaining claim: all known install paths serve one version (no install-path drift).
Evidence:
```
$ for f in /root/.cargo/bin/termlink /usr/local/bin/termlink /root/.local/bin/termlink; do echo "$f $($f --version) sha=$(sha256sum < $f | cut -c1-12)"; done
/root/.cargo/bin/termlink termlink 0.12.13+32.g023fbc762 sha=9179611302a9
/usr/local/bin/termlink termlink 0.12.13+32.g023fbc762 sha=9179611302a9
/root/.local/bin/termlink termlink 0.12.13+32.g023fbc762 sha=9179611302a9
```

## T-2938 — cron drift: agentic-audit.crontab differed from the deployed /etc/cron.d copy
Remaining claim: the deployed copy equals the tracked source.
Evidence:
```
$ cmp -s .context/cron/agentic-audit.crontab /etc/cron.d/agentic-audit-termlink && echo identical
identical
$ bash scripts/check-cron-install-drift.sh | tail -1
check-cron-install-drift: healthy (47 installed + matching, 0 acknowledged, 0 skipped)
```

## T-2939 — cron(substrate-smoke-canary) declared but never installed to /etc/cron.d
Remaining claim: the canary is installed, scheduled, and not drifted from the tracked source.
Evidence:
```
$ ls -l /etc/cron.d/termlink-substrate-smoke-canary
-rw-r--r-- 1 root root 1813 Oct 28 13:08 /etc/cron.d/termlink-substrate-smoke-canary
$ cmp -s .context/cron/substrate-smoke-canary.crontab /etc/cron.d/termlink-substrate-smoke-canary && echo identical
identical
$ ls -l --time-style=+%FT%T .context/working/.substrate-smoke-canary.heartbeat
... 2026-11-15T07:37:23 .context/working/.substrate-smoke-canary.heartbeat   (the cron line runs it daily at 07:37)
```

## T-3178 — Tier-1 write-gate bypass: `2> file` and `&> file` classified as not-a-write
Remaining claim: a reply was posted to the `framework:pickup` topic telling 050-email-archive their report was reproduced (the fix itself is merged and was verified separately).
Evidence: posts on the hub topic `framework:pickup` from 010-termlink that mention T-3178:
```
offset 202  from 010-termlink: "SECURITY FIX for the Tier-1 write gate, from 010-termlink (T-3178). Applied and tested here; sending the patch for the framework. This confirms and EXTENDS 050-email-archive's P-006 (their T-2257, offset 188). They foun…"
offset 206  from 010-termlink: "AMENDMENT to offset 202 (T-3178, the Tier-1 write-gate fix) — from 010-termlink, T-3187. Do not take offset 202's redirect rule as sent. It has a false positive we found in live use…"
```
