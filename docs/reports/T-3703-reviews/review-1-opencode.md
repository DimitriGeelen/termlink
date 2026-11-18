## Section 3 verdicts

1. **WEAK** — `hub.version` carrying the key under the `sha256(pk)==pin` check is sound, but also adding `signing_public_key` to the *head* contradicts AL-21's fixed head shape `{schema, hub_id, seq, digest}` and the committed V-HEAD-1 canonical form (151 bytes, digest `cc3632…`), and slice B's own deliverables list only touches `hub.version`.
2. **SOUND** — pins one form (`ed25519:` + hex sha256(pk), per AL-18 2.2.h) computed identically at issuance and at the receiver check, with the card-form conversion tested, exactly what the gap asked.
3. **WEAK** — it contradicts the quoted rule it claims to stay inside: 6.2.4 narrows to "own project's verified roster" with only "urgent mail from others" downgraded and DR-8 A rules "narrow allow-list to own project", and the code map itself says urgent already requires the `Trust::OwnProject` roster, so the "cannot tell apart" premise is unestablished and E-41.3's scenario is redefined rather than delivered (no widened-allow-list seam).
4. **WEAK** — the next-tick bound matches SI-11's reachable bound and FCW-96 covers restores, but the quoted [P] mechanism "every authenticated answer … carries the hub's current revocation head" (11.2.b (i), R-46.a (4e), SI-11) is declined, not delivered, and left to optional P-24 uptake.
5. **SOUND** — nesting E-40.4(a)'s exact text inside the existing `unverified since T (<why>)` slot reconciles the AL-22/E-40.4 contradiction within AL-22's own vocabulary and correctly demotes D-4's ROLLBACK to a transition.
6. **SOUND** — declining to assert a size is the right resolution of a three-way cosmetic discrepancy.
7. **SOUND** — inherits the quoted AR-4/OQ-h ruling (root-on-host stands in for the operator), with log-first ordering, attention, and the runme verb and fixture shipping now.
8. **SOUND** — the request carrying `public_key` with `fingerprint(pk)==key` and signature-by-that-key satisfies FCW-90 without the card dependency the plan doesn't record, and the ±10 min `at` window bounds capture-replay against later re-carding.
9. **WEAK** — "waits" is delivered, but RK-1's [P] sender-facing reason "key revoked, operator action needed" and its needs-attention entry are deferred to P-24, which the plan defines as operator status, and E-39.4's `refused-revoked` sender-record value is settled nowhere.
10. **WEAK** — the deferral is recorded, but 6.2.5 conditions hub-post refusal only on card existence and cards now exist (`key_card.rs`), so PH-2 leaves revoked keys free to post to the hub with no evidence item covering the interval.
11. **SOUND** — with sidecars raising `key-revoked` per (key, seq) on install, every revocation still has its three quoted traces (6.2.2, E-39.4, E-43.4) and AL-21's crash row is covered wherever a puller exists.
12. **WEAK** — start-up reconcile plus a `REVOCATION_RECONCILE` refusal fails closed correctly, but recovery is only "stated" until P-37 and the reconcile assumes admin rows record seq/digest, which AL-21's row `{method, caller, params_digest, by}` does not.
13. **WEAK** — a stated ~3000-entry limit with a 50 % attention leaves the terminal behavior undefined: at cap FCW-88 makes the list unissuable and unfetchable with no compaction, no un-revoke, and no recovery.
14. **SOUND** — the explicit `L.hub_id == h.hub_id == cfg.hub_id` check is the cheap FX-2-conformant closure the gap asked for.
15. **SOUND** — a dedicated `hub predates` status without attention, a default-ON switch whose OFF is the plan's "pull off returns to PH-1 trust (status says so)" rollback, and hub-first shipping resolve all three roll-out questions.
16. **SOUND** — committing 06a 2.4.c.1 plus V-REQ-1 as fixtures and copying the generator closes the gitignored-vector gap for T-3819's reuse.
17. **SOUND** — carrying E-127 on the status call is the option the extract itself allows under DP-3, with the canary drill already marked "Not proven" in the row.
18. **WEAK** — re-running only `revocation.check` delivers E-39.6, but FCW-200 / 06:893 re-run the full receiver checks ("policy, revocation AL-22, switch" — "a policy that no longer admits it"), and the hook integration point is conceded unread ("after reading the hand-over code").

## Section 2 verdict

**Slicing: SOUND** — A→B→C honors the P-10→P-15 security edge (B signs with the existing `hub_identity` key), maps every listed E-n to a slice with tier-appropriate harnesses and mutants, and each slice is green alone; the holes below are settlement-level omissions, not flaws in the split.

## Findings the note misses entirely (most severe first)

1. **Forwarded traffic's revocation check (FCW-76) is delivered by no slice or settlement** — settlement 18 covers only the hook's direct pull path, so a revoked original sender's mail still traverses forwarding. Rests on: "FCW-76 [AFC-22, AFC-25; E-39, E-50] the original sender's key is revoked, its sequence number is refused, or its quota at X is full → refused or held exactly as a direct message would be."
2. **Gate E-39 cannot pass as written** — the asserted sender-record value has no delivering slice and the RK-1 sender-facing behavior is mis-homed. Rests on: "the sender's record shows \"refused-revoked\"" (E-39.4); "the sender sees WAITING FOR RECIPIENT (\"recipient not able to receive\", R-44.a) with the reason \"key revoked, operator action needed\", and a needs-attention entry appears" (RK-1) — against the plan gate "the E-n listed pass at the level 05 states" and P-24 being "status: the operator's view of every later item".
3. **The stale rule's circuit clauses are silently dropped** — no slice delivers them and no settlement records the deferral (contrast gaps 9/10); settlement 3 rewrites the PN-13 effect as urgent-only, losing the PH-5 hook. Rests on: "time without a verified revocation pull after which new circuits stop and urgent delivery is downgraded (11.2.d)" (PN-13 row); "(i) refuses to set up or renew circuits" (11.2.d); "verdict `LIMIT` naming RR-20, with no circuit and no urgent delivery from F" (E-40.4 (c)).
4. **The roster is never given the restore treatment the
