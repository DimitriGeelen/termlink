# Close review 2: four tasks whose remaining criterion is a field observation

Project: TermLink, host .107. Measured 2026-11-15 ~20:35Z by the agent. Every command below was run, and its output is quoted verbatim (long lists trimmed, as marked).

Rule being applied: the agent may close a task when the remaining criterion's field condition is shown, by measurement, to hold now. Judge each task separately, against the criterion as written.

## T-1415 — post-cut cleanup: delete retired legacy primitive handlers and fallback paths
Remaining criteria (both): "Verify production hubs have been running flag-off for ≥7 days" and "Confirm bake metric is clean".
Evidence:
```
$ remote call ring20-management-hub hub.capabilities   (trimmed)
"features": { "legacy_primitives": false }, "hub_version": "0.12.13+32.g023fbc762"
$ remote call ring20-management-hub hub.legacy_usage
{ "audit_present": true, "by_method": {}, "last_legacy_ts_ms": null, "total_legacy": 0, "window_seconds": 604800, "top_callers": [] }
$ termlink fleet doctor --legacy-usage   (trimmed)
  total legacy invocations across fleet: 0
  CLEAN (7d): greenfield, local-hub, local-test, ring20-dashboard, ring20-management, workstation-107-public
  (laptop-141 not evaluated: TOFU violation, being handled separately)
```
Note: the legacy cut happened in 2026-05 (the task's Agent ACs carry 2026-05-31 evidence).

## T-1632 — bump default protocol: control_plane_version 3
Remaining criterion: "On next .122 deploy (after the T-1166 bake clears), `hub.capabilities` returns `control_plane_version: 3` alongside `protocol_version: 1`."
Evidence:
```
$ remote call ring20-management-hub hub.capabilities   (trimmed)
"control_plane_version": 3, "protocol_version": 1, "hub_version": "0.12.13+32.g023fbc762"
```

## T-2258 — hub read-path stall (channel.state/subscribe deadlock under concurrent walks)
Remaining criterion: "Fixed binary deployed to the .107 hub and the live repro no longer hangs."
Evidence:
```
$ git merge-base --is-ancestor db7c10a56 023fbc762 && echo yes     # db7c10a56 = "T-2258: fix hub read-path stall — spawn_blocking"
yes
$ git merge-base --is-ancestor c6e226ee3 023fbc762 && echo yes     # c6e226ee3 = T-2355 server-side walk deadline, same class
yes
$ readlink /proc/<hub pid>/exe; /proc/<hub pid>/exe --version
/root/.cargo/bin/termlink
termlink 0.12.13+32.g023fbc762
$ timeout -s KILL 12 termlink channel state <topic> --hub local-107-hub   (over TCP)
framework:pickup                       rows=361  rc=0  104ms
inbox:cacc73ea32b121dd/010-termlink    rows=1000 rc=0  194ms
```
Also in the task (2026-07-05 update): "LIVE .107 re-test PASSED: fix deployed (0.11.324) + field repro no longer reproduces". Today's run did not post to a shared topic first (the write-then-read variant), to avoid writing to peers' queues.

## T-3685 — runme action 28: long-lived Claude token for the test user testdev
Remaining criterion: "Create testdev's token at the terminal."
Evidence (runme log of 2026-11-15T20:22Z, action 28, which verifies by running one prompt as testdev):
```
28. A long-lived Claude token for the test user testdev — ASKS YOU at the terminal (T-3685; operator ruling A)
  skip    testdev's stored token answers (one prompt as testdev returned OK)
```
