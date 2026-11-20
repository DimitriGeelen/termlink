# T-3700 review round 2 — dispositions

Seats (isolated, both `independent`): opencode (`review-2-opencode.md`) **PASS**: every round-1 A finding CLOSED, every
R/D SOUND, 3 new MINOR. claude-code (`review-2-claude-code.md`) **FAIL**: every round-1 BLOCKER/MAJOR CLOSED, every R/D
SOUND, one new MAJOR (N1) introduced by the round-1 fix for finding 2, 7 new MINOR. A = fixed for round 3, R = reject
(reason), D = defer (risk). Round 3 is the last (cap 2–3); its findings become accepted, rejected or deferred, not
re-reviewed.

| # | Finding | Seat | Disp. | Change / reason |
|---|---|---|---|---|
| N1 | Full table: an unrecorded inbox is answered as a clean empty list (FCW-125 broken) | CC (MAJOR) | A | The skip sets a persisted `dropped_since_ms`; `inbox.consumers` for an inbox absent from the table answers an error ("consumer table full since …") while the window reaches it. Test asserts the error; mutant m27. |
| N2 | A missing topic skips the gate, so a bound inbox is ungated until created | CC | A | The gate (reads, claims, receipts) also applies when the topic is bound; unreadable bindings count as bound. Test + mutant m29. |
| N3 | Liveness fallback looser than "-32001 only" | CC | A | Exactly `SESSION_NOT_FOUND` with "Missing 'target'"; test adds a generic coded error; m21 updated. |
| N4 / OC 3 | Reset marker and row only in memory until a record; a dropped row lost | CC, OC | A | The empty table with `incomplete_since_ms` is saved at the move; the row stays queued until written. Restart test + mutant m30. |
| N5 | Suppressed count lost when its row fails; `capped_row` untested | CC | A | Cleared only after a successful write; unit test + mutant m28. |
| N6 | `why` change forces a full rewrite per call | CC | A | `why` updates in memory and is persisted on the 60 s cadence. |
| N7 / OC 1 | Re-check only when `timeout_ms` parses as u64 > 0 | CC, OC | A | Every gated read re-checks the binding before delivering; m20 updated. |
| N8 | Stale-ack rule refuses honest late receipts | CC | R | Measured: the sidecar posts no `receipt` (`grep '"receipt"' crates/termlink-sidecar/src` is empty); receipts come from operator/agent tools that sign at post time. |
| OC 2 | `key_fp` failure → `""` → own entry shown as a second copy | OC | A | An empty own fingerprint makes the line `unknown (this sidecar's own key fingerprint is unavailable)`; unit test. |
| CC 5 gap | `capped_row` untested | CC | A | See N5. |
| CC 5 gap | Unsigned claim before a bind not exercised | CC | D | The claim gate is the same `consumer_gate` as reads, whose unsigned-before-bind path is tested (E-66.6); m16 kills removing it. |
