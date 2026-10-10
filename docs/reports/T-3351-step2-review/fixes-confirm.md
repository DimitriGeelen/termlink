# T-3351 step 2: fixes after codex-confirm.md (v0.3.7)

Edited only `docs/design/interactive-agent-communication-02-threat-model.md`. No ruling changed. Replaced text is kept and marked "Superseded" or "earlier text, kept for the record".

| Finding | Change (section) |
|---|---|
| 1a | B′ propagated: 12.3 (points to B′), 15.5.3 (countermeasure is B′; PR-24 a complement), CR-9 (answered by B′), RR-6 (narrowed to envelope metadata), PR-33 (adopted target), BP-3 and 17.3.2 (content encrypted, metadata visible) |
| 1b | SI-31 probe (refuse without out-of-band confirmation, no effect, visible expiry within PN-15); 6.3 item (iii) at line ~242; 15.9.9/TH-64; PR-27; RR-16; CR-14 requirements sentence |
| 1c | PR-34, PN-16, CR-16 marked dropped (OQ-15): 2.2.c, 8.2 last case, 8.2.a, 15.4.3, SI-4 (statement and 18.2 row), 19.1 PN-16 row, 19.2 PR-34 row, RR-10, 21.2.3 |
| 1d | 14.13 denial row, 15.6.16 and 15.6.18/TH-33, 16.2.3/AC-3 follow CR-20 (home hub resolves at once) |
| 1e | TH-11 (15.3.6) and PR-3 follow CR-4 as corrected (sender's own durable counter; hub value only raises it) |
| 1f | BP-14, 17.2, 17.3.1 follow CR-21; PR-29 (17.4 and 19.2) follows the RR-13 ruling |
| 2a / 3a (CR-20) | R-34.o and R-67.e added to CR-20 |
| 2b | CR-15 gains intent field and history-share event; CR-19 gains "may share history" (default no automatic share) |
| 2c | 9.3.c and SI-8 distinguish peer MESSAGE (never an approval) from an authenticated request decided by a CR-18 route; 18.2 SI-8 depends on CR-6, CR-18, CR-19 |
| 2d | SI-2 and PR-32 (19.2 and 9.2.d.4) carry the OQ-16 activity veto and probe; busy-bit audience fixed in CR-5 and the 22.18 OQ-16 row (operator, own project, cockpit; not peers; CR-11 stands) |
| 2e | SI-17 (18.2) and PR-18 (19.2) depend on CR-17 |
| 3a (status) | 19.1 (OQ-3, OQ-9 starting values), 7.3.a (RR-2 accepted for now), 22.3 (OQ-2 ruled, C now, B committed), 21.2.2 (OQ-16 ruled D), RR-18 and BP-19 (interim re-pin audit adopted, T-3384) |
| Version | Row 0.3.7 added in section 0; status line set to v0.3.7 |

## Grep check

Terms: "authority unknown", "notice period", "rebuild", "PN-16", "PR-34", "open by design", "later option", "not yet accepted". Every remaining hit is one of: version-history rows (0.3, 0.3.4, 0.3.7); text marked superseded, dropped, historical or "kept for the record" (8.2 and 8.2.a, 15.4.3, 15.6.16/15.6.18, AC-3, SI-4, SI-31, PN-14 and PN-16 and PR-34 rows, RR-8, RR-10, CR-4, CR-14, CR-16, CR-20, 21.2.3, OQ-15 and its register row); or correct as ruled: "open by design" now applies only to BP-1, BP-2 and BP-13 (BP-14 excluded in 17.2 and 17.3.1). "not yet accepted" has no hits; "later option" appears once, in the PR-33 row, as the marker "was later option" (now the adopted target). The CR-4 "rebuild" hit carries the D4 correction marker.
