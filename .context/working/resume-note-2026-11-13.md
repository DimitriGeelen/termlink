# Resume note (main session 10c2f139, 2026-11-13 ~21:50Z, budget wrap-up after host crash 20:27Z)

First: re-arm the standing runme START watch (CLAUDE.md T-3773). No runme run outstanding.

1. Release v0.12.13 NOT tagged yet. v0.12.12 (a298541ad) blocked on GitHub by 2 guard reds, fixed in T-3784
   (26bf6548c, pushed). Local release-gate run on a clean clone (.context/working/t3753/src at 26bf6548c,
   log .context/working/t3753/gate13.out) was cut off by the budget restart after 185 checks. Reds so far:
   check-arc-claim-drift, check-pin-events, pin-events-fixtures (FAIL), artifact-cli-fixtures (ERROR: probably
   "no termlink binary" in the clone, env-only). Earlier GitHub run passed arc-claim-drift and pin-events, so check
   whether these are clone/env artefacts or real before tagging. Then tag v0.12.13, watch GitHub (macOS job
   passed on v0.12.12), close T-3784.
2. After v0.12.13 publishes: hand operator ONE /opt/termlink/runme.sh run (installs T-3779 etc on .107), then tell
   AEF (conversation termlink-arc011-p44-adapter2) to restart aef-meter once. Installed now: 0.12.11+19.gfbb145cfe
   (no T-3779).
3. Open operator decisions: (a) T-3760 registration-capability expiry: A 7 days (recommended) / B 24 h / C 30 days;
   (b) Homebrew 2A steps (tap repo homebrew-termlink still 404; token; secret HOMEBREW_TAP_TOKEN), then T-3754 check.
4. Next builds: T-3781 (dispatch uses TCP registration for non-root), T-3782 (hub forwarding above observe to TCP
   workers), T-3780 (MCP command.exec vs command.execute), T-3783 (meter report log), T-3757 step 2 after 7 days of
   use-log evidence (clock starts at the next .107 install).
5. AEF trial (T-5054) live: binding bound, origin_checked works. 1409 has not confirmed DISPATCH-REQUEST v1.
