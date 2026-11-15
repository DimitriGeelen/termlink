T-1415: PASS — The gated handlers were deleted 2026-05-31 (handler grep = 0, task AC verification quoted), making every post-cut binary structurally flag-off for >5 months, and the bake metric is clean (`.122` legacy_usage: total 0 over a 604800s window, last_legacy_ts null; fleet doctor CLEAN 7d).

T-1632: PASS — hubs.toml ties ring20-management to .122:9100, and that hub (build 0.12.13+32, deployed 2026-11-13, long after the May cut with legacy usage 0 over the 7d window) returns `control_plane_version: 3` alongside `protocol_version: 1`.

T-2258: PASS — The fix commit db7c10a56 is an ancestor of the running .107 binary (version read from `/proc/<hub pid>/exe`, not just the latest build), and the live repro — write-then-read plus 4 concurrent reads against 20 landing posts — returned rc=0 every time with no hang.

T-3685: PASS — Action 28 is by definition paste-at-terminal-then-verify; it FAILED at 08:30 ("re-run to paste a fresh token") and returned OK at 11:54 and again 11-14, with the distinct "skip" outcome on 11-15 showing the terminal-pasted token persistently answers.

Remarks:
- T-3685 rests on runme one-line summaries; a verbatim paste/verify transcript (or token-file mtime matching the OK run) would make creation-at-terminal airtight rather than inferred from the OK/skip distinction.
- laptop-141 is unevaluated (cert issue) and is assumed non-production per its description; if it counts as a production hub, T-1415's fleet-clean claim has a hole.
- The bundle answers each round-1 gap directly; no internal contradictions found.
