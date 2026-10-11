# T-3352 step 3, review round 1: fixes (role session `step3-fix-r1`, 2026-10-11)

Document: `docs/design/interactive-agent-communication-03-floor-and-phasing.md`, now DRAFT v0.2. Disposition followed: `disposition-r1.md` (binding). Only that document and `docs/reports/T-3352-role-handback.json` were edited. All 17 rows are fixed; none is left open.

## 1 One line per disposition row

1. Row 1 (E): `may_remove` third value is now `anyone_in_conversations_it_started`, with the ruled wording and a v0.1 note. 10.3.3.
2. Row 2 (E): C-17 deleted from the class table, new 10.2.2.a explains there is no custody-release route outside C-10 and that break-glass stays R-80.a (7); high-impact list is nine classes (10.2.4, 10.2.8); OP-3 notes that a wider release needs a ruling change. 10.2.2, 10.2.2.a, 10.2.4, 10.2.8, 13.2.3.
3. Row 3 (E): C-9c limited to grant records (every rule `start` or `resume`), identical body, unexpired; mixed or other records are never renewed that way; budget never reset or raised on renewal, counters keyed by scope, rule id and window (marked [P]). New 10.3.2.e, class table C-9c, 10.2.3, OP-2.
4. Row 4 (E): C-15 is one event and one digest per account; the batch form is withdrawn; 10.1.5 price table follows; OP-1 notes a batch form would need a ruling change. 10.2.2 (C-15), 10.1.5, 13.2.1.
5. Row 5 (E): `digest` is over the record with both `digest` and `signature` removed; the authorising event's digest is over the unapproved form (no circularity); profile example now carries an approval and `approvals` MUST NOT be empty. 10.3.2.a, 10.3.2.c, 10.3.5, 10.3.9.
6. Row 6 (E): format completed. 10.3.2.c signing payload (domain-separated), 10.3.4.a verbs (fourteen: ten action, four event; v0.1's "ten" was a miscount of eleven names) with `rotate_hub_key`, `replace_agent_key`, `renew_grant`, `hand_over` defined in 10.3.4.c with built-in preconditions and routes; event-rule representation (rules with event verbs only narrow the built-in route; receipts name `policy:<record>#<rule>` or `policy:builtin:<verb>`); 10.3.2.d epoch per (scope, kind), one current record per pair, replace not merge; 10.3.7.f same-level resolution (deny overrides allow stated normatively, smallest limit among allows, order irrelevant); 10.3.7.g events; 10.3.8 defaults table extended.
7. Row 7 (E): PH-3 exit tests C-2a and C-6a by the built-in route; C-10 by fixture only in PH-3 and live at PH-4 exit (new 6.2.5.c (9)); PH-3 scope states the route table is fixed, no configurable rule yet. 6.2.4.a (2), 6.2.4.c (2), 6.2.5.c (4) and (9).
8. Row 8 (E): account pilot (A0) moved from PH-2 to PH-3 after the C-15 Tier 0 route; H-28 measured at PH-3; 4.2 R-82 row, 6.2.3.a, 6.2.3.d (7), 6.2.4.a (12), 6.2.4.c (5), 6.5.2, 9.2 H-28, 9.3, 10.1.8.e. No new risk acceptance.
9. Row 9 (E plus O): SI-15 start reconciliation in PH-1 closed by an estate guard (refuse to start outside a throwaway tree, checksum before STORED), with a probe in X4; no RA needed because PH-1 stores no real mail (3.3.1.d, 5.9.4). SI-7 on the existing rails cannot be fixed by an early local control (vendored code, G-062), so it is RA-9 (11.3.9) and OP-23 (13.2.16), cited in 3.3.1.e and the SI-7 row.
10. Row 10 (O): OP-21 (13.2.14) on the CX-2 hook delivery point, options A to D, recommendation A; BV-5 aligned to the recommendation (turn end) and marked pending, with the consequences if the operator rules B; CX-2 text points at OP-21. 5.5.5, 12.3, 13.2.14.
11. Row 11 (O): OP-22 (13.2.15) on who writes the receive side; 6.6 row and 15.5.5 point at it.
12. Row 12 (E): PH-0 rollback keeps the re-pin recorder and the off switch is removed. 6.2.1.d.
13. Row 13 (E): PH-6 rollback written item by item (six procedures, none turns a control off, nothing decrypted); D-1 has a PH-6 rollback note; text equivalent 6.8.3 updated; render check updated. 6.2.7.b, 6.2.7.c, 6.7, 6.8.3, section 16.
14. Row 14 (E): RA-4 ends in two separately stated parts: confirmation part ends at K5 with the R-76.d probes passed, signed-policy part ends at K4 with `require_operator_signature_from_epoch` set; status call names which part is open. 11.3.4.d.1 to d.3.
15. Row 15 (O-light): OP-10 option A now names the five binding rules 3.1.b to 3.1.f. 13.2.10.
16. Row 16 (E): PH-0 baselines now include the count of plaintext content fields a topic read token returns on the hub, per topic family; written to `docs/reports/`. 6.2.1.a (2), 6.2.1.c (4).
17. Row 17 (E): R-7 wording corrected: after every other P1 delivery item, with R-76's real provider and R-1's operator leg later. 4.4.c, 11.2.

Also done: status line DRAFT v0.2; version row 0.2; coverage (15.1, 15.5.6) and counts updated: 23 operator questions (was 20), 9 risk-acceptance requests (was 8), 9 high-impact classes (was 10), 14 policy verbs, Tier 0 classes C-1 to C-16; floor (39) and hypotheses (28) unchanged. Handback `docs/reports/T-3352-role-handback.json`: status `ready-for-review`, new output sha256 `9b78a1392a4d1c37ce893670a4d05a018d6cdf7aafe5e004a4cc4cbeb9c3b8ba`, summary, open questions and decisions updated; validates against `role-handback-1.schema.json`.

## 2 Renders (section 16 of the document)

`mmdc -p {"executablePath":"/usr/bin/chromium","args":["--no-sandbox"]}` on both Mermaid blocks of the final v0.2 text: D-2 rc 0, 60351-byte SVG (unchanged drawing, re-rendered); D-1 rc 0, 44707-byte SVG. No "syntax error" or "parse error" in any log or SVG. D-1 was rendered to PNG and read: order PH-0 to PH-6 intact, PH-6 rollback note present, no semicolons in labels.

## 3 Rows not fixed

None.

## 4 Things the orchestrator should check

1. Row 6 contains two points beyond the disposition text that I chose and the reviewer should look at: a `deny` event rule only turns a policy route into a human route (no rule can widen a route), and a `profile` is treated as shorthand for agent-level rules in the same-level resolution (10.3.7.f.1).
2. The row 10 recommendation (A) is what BV-5 now follows; if the operator rules B, 12.3 lists the phasing changes.
3. The expired-record rule of 10.3.7.a (an expired record is ignored, so an expired deny falls back to the built-in default) was not in the disposition and is unchanged; it may deserve a look in step 4.
