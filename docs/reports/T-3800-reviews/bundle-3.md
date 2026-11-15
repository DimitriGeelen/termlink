# Close review 2, round 2: the same four tasks, with the evidence round 1 asked for

Project: TermLink, host .107. Measured 2026-11-15 ~20:45Z. Commands run on the host; outputs verbatim (trimmed where marked). Host addresses are redacted to their last octet (.107, .122).

Round 1 (two independent seats) asked for the following. Each point is answered below.
- T-1415: how long the legacy path has been off, not only "off now".
- T-1632: that the hub measured is the .122 deploy, and that the T-1166 bake cleared.
- T-2258: the actual write-then-read and concurrent-walk repro against the deployed binary.
- T-3685: that the token was CREATED at the terminal, not just that one exists.

Judge each task separately, against its criterion as written.

## T-1415 — post-cut cleanup: delete retired legacy primitive handlers and fallback paths
Criteria: "Verify production hubs have been running flag-off for ≥7 days" and "Confirm bake metric is clean".
Evidence on duration (the flag cannot be on: the code it gated no longer exists, and has not since 2026-05-31):
```
$ grep -rnc 'handle_event_broadcast|handle_inbox_status|handle_inbox_list|handle_inbox_clear' crates/termlink-hub/src/   (summed)
legacy handler matches now: 0
$ sed -n 1154,1160p crates/termlink-hub/src/router.rs
    // T-1405 / T-1415: legacy_primitives is now hardcoded false (cut landed
    // 2026-05-31). Field retained for downstream consumers that probe
    // post-cut vs pre-cut hubs (returns false on every post-cut hub).
    let features = json!({ "legacy_primitives": false, });
```
The task's own Agent ACs record the deletion "verified 2026-05-31T19:09Z" (handler grep = 0 matches; `LEGACY_PRIMITIVES_ENABLED` grep = 0 matches). Every hub binary built since then is flag-off, which is more than 5 months.
Evidence on the bake metric:
```
$ remote call .122 hub.legacy_usage
{ "audit_present": true, "by_method": {}, "last_legacy_ts_ms": null, "total_legacy": 0, "window_seconds": 604800, "top_callers": [] }
$ termlink fleet doctor --legacy-usage   (trimmed)
  total legacy invocations across fleet: 0
  CLEAN (7d): greenfield, local-hub, local-test, ring20-dashboard, ring20-management, workstation-107-public
```
(laptop-141, an intermittently-on laptop, could not be evaluated today because of a separate certificate issue.)

## T-1632 — control_plane_version 3
Criterion: "On next .122 deploy (after T-1166 bake clears), `hub.capabilities` returns `control_plane_version: 3` alongside `protocol_version: 1`."
```
$ grep -A2 '^\[hubs.ring20-management\]' ~/.termlink/hubs.toml | grep address
address = ".122:9100"        # the hub queried below IS the .122 hub
$ remote call .122 hub.capabilities   (trimmed)
"control_plane_version": 3, "protocol_version": 1, "hub_version": "0.12.13+32.g023fbc762"
```
The T-1166 bake: see T-1415 above. The legacy cut landed 2026-05-31; fleet legacy usage is 0 over 7 days. The .122 hub has since been deployed several times; its current build is 0.12.13+32 (2026-11-13).

## T-2258 — hub read-path stall (channel.state/subscribe deadlock under concurrent walks)
Criterion: "Fixed binary deployed to the .107 hub and the live repro no longer hangs."
Deployment:
```
$ git merge-base --is-ancestor db7c10a56 023fbc762 && echo yes     # db7c10a56 = "T-2258: fix hub read-path stall — spawn_blocking"
yes
$ /proc/<hub pid>/exe --version
termlink 0.12.13+32.g023fbc762
```
The live repro, on a scratch topic (created for this, 1500 posts, deleted afterwards), against the running .107 hub over TCP, each read under `timeout -s KILL 12`. The original bug: a large-topic read hung right after a write to the same topic, and under concurrent walks.
```
cold read (1500 rows):                         rc=0  68ms
post once, then immediate re-read:             rc=0  67ms
4 concurrent full reads while 20 posts land:   rc=0 73ms, rc=0 72ms, rc=0 111ms, rc=0 152ms
$ termlink channel delete <scratch> --yes
Deleted topic 't3800-repro-…' (1521 record(s) removed)
```

## T-3685 — long-lived Claude token for the test user testdev
Criterion: "Create testdev's token at the terminal." (runme action 28 asks the operator to paste the token at the terminal, then verifies it.)
```
runme-20261111T083043Z log, action 28:  FAILED  token stored, but a prompt as testdev did not answer OK; re-run to paste a fresh token
runme-20261111T115459Z log, action 28:  OK      testdev has its own long-lived token (/home/testdev/.config/claude/oauth-token, mode 600, loaded by /home/testdev/.profile); one prompt as t[estdev answered]
runme-20261114T005749Z log, action 28:  OK      (same)
runme-20261115T202209Z log, action 28:  skip    testdev's stored token answers (one prompt as testdev returned OK)
```
