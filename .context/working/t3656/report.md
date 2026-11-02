# T-3656 — the 74 messages in `fw sidecar waiting` (2026-11-02)

Raw data: `waiting.json` and `waiting.txt` (listing), `drop-list.tsv` (44 ids proven droppable, each with its evidence).

## Counts by class (74 = 56 inbound + 18 outbound)
1. Inbound e2e test residue: 9 (b557900e nudges 6-10 and c91c5585 nudges 7-10; e2e-* senders; hub_ts 2026-09-22).
2. Inbound AEF, seen and answered: 35 entries from 11 originals. The evidence is a reply or READ note we posted (session e817a600) or a commit:
   2a. bb562f4b vector-db-rca: closing note 2026-10-06T11:36 (5 entries)
   2b. 0edbab2a, 5fd64b99, 83389faa, 9a187da7 aef-termlink-status: replies 10-02T23:14 and 10-03T10:56 (18 entries)
   2c. 1e429b3f t3325-project-uuid: READ+answered 10-03T10:56 (4 entries)
   2d. c5ee2170 and cd3ad2df: READ notes 10-03T15:19 (2 entries)
   2e. 194bc78c and c4a7b9e8: RECEIVED replies 10-04T00:20; c4a7b9e8 was ported as T-3337 (2 entries)
   2f. a8717ecc consult-010-termlink nudges 7-10: both asks answered 2026-09-28 in commit b9a7744a3 (T-3141, on conv e2e-ab947312). The original itself is already HANDED_OVER (4 entries)
3. Inbound AEF, content acted on, but no proof the message itself was read and no reply: 3 entries for fe30dbda (t3336, the "one version, two hubs on .107" request). Acted as T-3340 (c89240800, 2b042c2d6), which cites AEF T-3779 and pid 2919639.
4. Inbound AEF, never seen (neither the id nor the text appears in any transcript before today): 9 entries for 2 originals (listed in 7-8).
5. Outbound, 18 in all:
   5a. 13 were RECEIVED by the peer's receiver but never handed over or answered (peer has no live recipient, or escalated): 20edd6a5, 8d2e9957, c6d20adc, 87569601 (AEF); c6294ba6, 0bff3343 (832); 6fafccd1, 95a86299 (055); afb9ef9c, ed178a10 (0235); 4fc71b91 (0909-boot); b906e055, 6dfacc8e (0909-aef-greenfield-v22).
   5b. 3 REJECTED by 055 on t3340-second-hub: 39e7dbb8, 3463021e, 7517d5f9.
   5c. 2 hub sends to aef-greenfield-test with no receipt: 05a1e020, a59546c7.
   5d. Did the peer get it another way? No peer reply carries in_reply_to = any of these 18 ids. 055 kept posting on env-leak-t536 on 10-16 and 10-17, so 95a86299 was probably seen. The 3 focus-anchor-survey sends (11-02) are under a day old and genuinely in flight.

## Never-seen or not-acted inbound, for the orchestrator (not answered by this worker)
6. fe30dbda, AEF, t3336-vector-index, 2026-10-03: operator ruling "deployment must always be one version" (two hubs and three binaries on .107). Acted via T-3340; no reply sent.
7. ac9d2de7, AEF, t3336-vector-index, 2026-10-03: ruling refinement. Default is one version and one hub per host; a second hub or binary must be explicitly declared; an undeclared one should FAIL. NEVER SEEN. T-3340 is only partly aligned: `--allow-second-hub` is the declared option, but preflight Check 7 WARNs where the ruling says FAIL.
8. e50761d3, AEF, t3336-vector-index, 2026-10-03: our T-3336 findings are fixed upstream in AEF T-3783 (c62bab1bc, 39769059c; cron-seed now adds index-reindex-hourly). NEVER SEEN. It is informational, but it is the reason to check that our next re-vendor brings the seeded cron job in.

## Root cause (lib paths are under .agentic-framework/lib/sidecar/)
9. The receiver came up on 2026-10-06 at 11:33 and backfilled `inbox:cacc73ea32b121dd/010-termlink` from offset 0 (events.jsonl INGESTED_FROM_HUB, offsets 7-390, hub_ts going back to 09-22). Most of that mail had already been read and answered on the hub rail between 09-28 and 10-04.
10. The receiver logged WAITING_NO_RECIPIENT for each message, because the injector could not find our session: inject.py:528/551 looks it up by tag `claude` plus cwd. That first WAITING event is never superseded. waiting.py:386-397 derives "no-live-recipient" from it for as long as the message stays pending, even after an injection went into e817a600 at 10-06T11:45.
11. Two closure predicates disagree. The injector withholds a message that is "already shown" or "answered" (seen.py:172-193, inject.py:669-672), so it is never handed over. But the waiting list closes only on handed-over, dropped, or answered-by-in_reply_to (waiting.py:279-290). "Already shown" is not a closure, so withheld mail is listed and escalated forever.
12. A hub-rail reply without `in_reply_to=<client_msg_id>` is invisible to answered_ids (seen.py:119-140). Our 10-02 to 10-04 replies quoted the id in the body or used a different conversation, so none of them count.
13. A nudge whose base id is HANDED_OVER stays open (is_closed_inbound tests only the nudge id's own pending file): a8717ecc.

## Recommended fix
14. AEF (upstream, per G-062; vendored code not edited): (a) is_closed_inbound should treat seen.withheld()=="already shown", or a HANDED_OVER base id, as closed; (b) a later HANDED_OVER or INJECT_ATTEMPT should clear the "no-live-recipient" state; (c) a backfill from offset 0 should not raise WAITING or escalate for mail older than the receiver epoch; (d) count a hub-rail reply that quotes the client_msg_id as answered.
15. 010 (ours): always reply with `fw sidecar send --in-reply-to <id>`, never a bare channel post. Start interactive sessions with `claude-fw --termlink` so inject.py finds a session.
16. Operator: `fw sidecar drop` is operator-only (refused under CLAUDECODE=1; this worker did not use `--i-am-human`). Drop the 44 ids in drop-list.tsv (9 e2e + 35 answered), e.g. via a runme action. That leaves 12 inbound (items 6-8) and 18 outbound for real follow-up.
