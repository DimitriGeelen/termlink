T-1415: UNCLEAR — zero legacy usage over 7 days is shown for six hubs, but the flag is shown off on only one hub and only as of now, with nothing measuring how long it has been off (the 2026-05 cut is an unquoted note), and laptop-141 is not evaluated.

T-1632: UNCLEAR — the values match (`control_plane_version: 3`, `protocol_version: 1`), but the bundle never shows that ring20-management-hub is the .122 deploy or that the T-1166 bake has cleared.

T-2258: UNCLEAR — deployment of the fix is shown (the running hub binary is 023fbc762, which contains db7c10a56), but today's run was two single reads, not the concurrent-walk or write-then-read repro, and the 2026-07-05 pass is an unquoted claim against an older build.

T-3685: PASS — the 2026-11-15 runme log shows action 28 skipped because testdev's stored token answered a live prompt as testdev, so the token exists and works now.

Remarks:
- T-1415 deserves the most caution because closing it authorises deleting fallback paths; it needs per-hub flag state plus a flip timestamp or config history, and a decision on laptop-141.
- T-2258 could be settled by running the actual concurrent repro against a scratch topic, which avoids writing to peers' queues.
- T-3685's evidence shows a working token, not that it is long-lived; the criterion as written does not require that, but the title does.
