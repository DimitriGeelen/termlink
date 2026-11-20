# T-3700 review round 3 (final) — dispositions

Seats (isolated, both `independent`): opencode (`review-3-opencode.md`) **PASS**, 2 MINOR. claude-code
(`review-3-claude-code.md`) **FAIL**: N1 only partly closed (one MAJOR), 5 MINOR; every other round-2 finding CLOSED and
every R/D SOUND. Round 3 is the cap (2–3 rounds): every finding below is accepted with a change, rejected or deferred
here and is not sent round again. Accepted changes are pinned by tests and mutants (`T-3700-mutants.md`).

| # | Finding | Seat | Disp. | Change / reason |
|---|---|---|---|---|
| CC 1 (MAJOR) | A dropped inbox that later enters the table is served as a clean list while the dropped read is in the window | CC | A | The table keeps when each inbox entered it and the first drop of the current run; an inbox absent, or entered at or after that first drop, is an error while the latest drop is inside the window. An inbox present before the run was there through every drop. Test `an_inbox_entering_after_a_drop_is_not_served_clean_inside_the_window` (the reviewer's sequence); mutant m33. |
| CC 2 / OC 1 | Drop marker refreshed only every 60 s | CC, OC | A | The marker moves on every drop; only the save is rate-limited. |
| CC 4 | N2 wiring untested at the handlers | CC | A | L4 `a_bound_inbox_is_gated_before_its_topic_exists` (read, claim, ack → INBOX_BOUND); mutants m31 (gate), m32 (ack branch). |
| CC 5 / OC 2 | Empty own fingerprint hides other readers and hub errors | CC, OC | A | Hub errors are reported first; with an empty fingerprint, readers under another agent's name are still listed and flagged; the line is unknown. Unit test. |
| CC 3 | The reset attention row lives only in memory until a record | CC | D | The marker itself is now saved at the move (round 2), so `inbox.consumers` and every sidecar show `unknown (history incomplete since …)` across restarts: the condition stays visible. Only the admin row can be lost if the hub restarts before any inbox read. Risk: the operator learns of the reset from status rather than the admin log. |
| CC 6 | Re-check only when the gate returned a reader | CC | D | The gate returns no reader only for a non-inbox topic, or an unbound topic that does not exist (nothing to read). A topic created AND bound during one long-poll by a non-proven reader is the residual; a bind is an operator runme step, not a race an outsider controls. |
