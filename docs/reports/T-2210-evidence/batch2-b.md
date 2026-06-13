# T-2210 — G-008 Fresh Evidence (batch2-b)

Generated: 2026-06-13T13:51:56Z by resmoke-agent (host: /opt/termlink, binary: target/release/termlink built today).
Live peer fingerprint used for target/peer args: d1993c2c3ec44c94.

All Human ACs remain UNCHECKED (sovereignty). Evidence below is for batch-confirm.

## T-1435
- **Class:** LOCAL
- **Command:** `target/release/termlink fleet doctor --legacy-usage --legacy-window-days 1`
- **Result:** exit=0; ok
- Verdict: **CUT-READY**. Total legacy invocations across fleet = 0. CLEAN (1d): local-test, ring20-dashboard, ring20-management, workstation-107-public. laptop-141 timed out (10s) but no legacy callers anywhere. T-1166 cut already landed in T-1415; verdict informational. .143 not present in fleet at all.

## T-1442
- **Class:** LOCAL (regression); live-spawn steps mutating — not executed
- **Command:** `bash tests/test_tl_dispatch_meta.sh`
- **Result:** exit=0; ok
- All pins PASS. Pin 3 (cmd_spawn writes populated meta.json): caseA model_used=haiku/fallback_used=False/task_type=build; caseB per-type=sonnet; caseC default=opus+fallback=True; caseD JSON null both. Live spawn against a running hub (AC steps 2-4) is a mutation and was not executed; regression covers the model-resolution + meta-write logic.

## T-1453
- **Class:** LOCAL
- **Command:** `.agentic-framework/bin/fw task revisit-due`
- **Result:** exit=0; ok (clean output, no debug noise)
- Output: `No revisits due today (2026-06-13 UTC)`. Format matches handover banner. No known-ripe revisit_at on this host today to exercise the populated-row path.

## T-1632
- **Class:** OPERATOR-ENV
- **Command (attempted):** `target/release/termlink remote call --hub 192.168.10.122:9100 --method hub.capabilities` -> exit=2, "unrecognized subcommand 'call'"
- **Result:** operator-env-skip. Two blockers: (1) AC gates on the NEXT .122 deploy after T-1166 bake — fleet doctor shows .122 still on 0.11.806 (pre-deploy, new build not yet there); (2) the AC's `remote call --method` verb form is not present in target/release/termlink (remote subcommands are ping/list/status/inject/send-file/events/push/inbox/doctor/profile/exec). Needs the post-T-1166 binary deployed to .122 plus a hub.capabilities call path.

## T-1795
- **Class:** LOCAL (bug-fix regression — local hub populated)
- **Commands:** `target/release/termlink agent on-thread T-1438 --window-secs 604800` ; `agent timeline --thread T-1438 --window-secs 604800`
- **Result:** exit=0; ok — fix confirmed
- on-thread returned 152 lines of T-1438 posts (no longer "(no posts found)"). timeline --thread T-1438 surfaces the SAME @3117.. posts. Both non-empty and in agreement. (A head-pipe truncation produced a spurious 101 the first run; clean run with no pipe = exit 0.)

## T-2013
- **Class:** OPERATOR-ENV
- **Command:** n/a — operator-env
- **Result:** operator-env-skip. [RUBBER-STAMP] AC deploys the musl-static artifact to ring20 hubs (.122/.121/.141) via `scripts/fleet-deploy-binary.sh --swap-restart` (ssh + remote hub restart) then times 5/5 sequential remote `channel info`. Requires remote production hosts + binary swap — not runnable from this host.

## T-2090
- **Class:** OPERATOR-ENV
- **Command:** n/a — operator-env
- **Result:** operator-env-skip. [REVIEW] inception go/no-go: `fw task review T-2090` opens Watchtower; human reviews recommendation + criteria and records decision via the Watchtower form. Human-judgment + UI; no local smokable command.

---
Summary: 4 re-smoked locally (T-1435, T-1442 regression, T-1453, T-1795) — all exit=0/ok; 3 operator-env skips (T-1632, T-2013, T-2090). No Human checkbox ticked.
