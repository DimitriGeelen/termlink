# T-2210 batch2-a — G-008 fresh evidence (resmoke-agent)

UTC: 2026-06-13T13:51:52Z  ·  Binary: `target/release/termlink` 0.11.1293 (built 2026-06-13)
Peer fingerprint for target args: `d1993c2c3ec44c94`

All Human-AC checkboxes left UNCHECKED (sovereignty). Evidence below is for batch-confirm.

## T-1415
- **AC:** [REVIEW] prod hubs flag-off ≥7d + bake metric clean.
- **Classification:** AC1 (SSH each prod hub .107/.121/.122 + `journalctl ... grep`) = operator-env. AC2 bake metric = local-runnable.
- **Command:** `.agentic-framework/bin/fw metrics api-usage --cut-ready --json`
- **Result:** exit=0; ok. `{"cut_ready": true, "legacy_attributable": 0, "legacy_unattributable_pre_t1409": 0, "window_days": 7}`
- **Note:** AC1 not re-smokable from this host (needs ssh + journalctl on each prod hub).

## T-1417
- **AC:** [REVIEW] zero event.broadcast callers from this codebase after prod rebuild+restart+7d bake.
- **Classification:** build-release + hub-restart + 7d-bake = operator-env; the audit step is local-runnable.
- **Command:** `.agentic-framework/bin/fw metrics api-usage --cut-ready --json`
- **Result:** exit=0; ok. cut_ready=true, legacy_attributable=0 — zero attributable event.broadcast from this host.
- **Note:** Steps 1-3 (cargo build --release, restart hub, wait 7d) = operator-env.

## T-1419
- **AC:** [REVIEW] last_seen_iso freshness signal distinguishes live vs stale post-T-1418 .143 deploy.
- **Classification:** the `api-usage --json` command is local-runnable; .143 deploy + restart bracketing = operator-env.
- **Command:** `.agentic-framework/bin/fw metrics api-usage --last-Nd 7 --json`
- **Result:** exit=0; empty-well-formed. All three arrays present (`legacy_callers`, `legacy_callers_by_ip`, `legacy_callers_by_pid`), each len=0 in the current clean window — no rows to carry last_seen_iso. Schema intact; rows missing last_seen_iso = 0.
- **Note:** Steps 1-2 (.143 deploy + restart polling agents) = operator-env.

## T-1426
- **AC:** [REVIEW] deprecation warnings informative without noise; suppression flag works.
- **Classification:** fully LOCAL.
- **Command:** `target/release/termlink {event broadcast|inbox status|inbox list|inbox clear|file send|remote push}` + `TERMLINK_NO_DEPRECATION_WARN=1 ... remote push`
- **Result:** exit=0; ok. All 6 legacy verbs emit exactly one `[DEPRECATED]` line citing the right replacement verb + T-1166; suppression flag → grep -c DEPRECATED = 0.
- **Note:** No operator-env steps.

## T-1429
- **AC:** [REVIEW] verb UX from a vendored-agent perspective (fire-forget post, ack-required, require-online, json).
- **Classification:** `--help` parse + `--dry-run` = local. Steps 2-5 (live posts/ack vs named peer `ring20-management-agent`, require-online vs known-down host) need a reachable peer + are mutating — not run.
- **Command:** `target/release/termlink agent contact --help`; `agent contact --target-fp d1993c2c3ec44c94 --message ... --dry-run --json`
- **Result:** exit=0; parse-confirmed-only + dry-run ok. Help documents all Phase-2 flags (T-1425/RFC, --ack-required, --require-online, --target-fp, --json, --dry-run). Dry-run resolves canonical `dm:d1993c2c3ec44c94:d1993c2c3ec44c94`, `dry_run:true`, no post.
- **Note:** Steps 2-5 need reachable named peer; mutating, not executed.

## T-1430
- **AC:** [REVIEW] topic self-doc discoverable from a fresh agent's perspective.
- **Classification:** `channel info` against local hub = local-runnable; step 2 (--hub 192.168.10.107 from a peer) = operator-env.
- **Command:** `target/release/termlink channel info agent-chat-arc | head -20`
- **Result:** exit=0; ok. Description present (agent-chat-arc protocol stack — msg_type, identity-via-whoami, _thread, in_reply_to, inbox.push deprecation), retention=forever, 3199 posts.
- **Note:** Cross-host peer-perspective step = operator-env; local read confirms description present.

## T-1432
- **AC:** [REVIEW] cut-readiness signal is actionable.
- **Classification:** baseline `fleet doctor --legacy-usage` = local-runnable; steps 2-4 (deliberately trigger inbox.push + wait 7d clean) = operator-env.
- **Command:** `target/release/termlink fleet doctor --legacy-usage`
- **Result:** exit=0; ok. Verdict=CUT-READY-DECAYING; 3 hubs CLEAN (7d), ring20-dashboard 1 decay-residue invocation (3d ago); "no live legacy callers (no traffic in last 300s)". Live + actionable.
- **Note:** Steps 2-4 (deliberate trigger + 7d clean re-check) = operator-env.
