## Verdicts

**Section 2 (slicing): WEAK** — the slices are not "each green on its own": E-42.1 needs the `prev_digest` and `seq` checks that live in slice C, E-43.4's "three traces" needs the sidecar-raised attention of item 11 (slice C) while E-43 is booked to slice B, and no slice names the E-40 (a) proxy or the E-40 (c) `LIMIT` harness.

1. **WEAK** — hashing the offered key against the pin is sound, but adding `signing_public_key` to the head breaks V-HEAD-1's 151-byte canonical form unless it sits outside the signed object, and a mismatch must also be a failed pull with `revocation-tamper` (06 4.7 "a signature by an unpinned key"), not only the FCW-83 path.
2. **WEAK** — the fingerprint form matches 6.2.1 and AL-18, but "conversion is one function, tested both ways" is impossible for a SHA-256 fingerprint, and "the vectors already commit" proves nothing: V-REV-1's `3b6a27bc…da29` appears to be the raw ed25519 public key of the all-zero seed, not a hash.
3. **WRONG** — code map item 6 says rostered senders are `Trust::OwnProject`, so the premise "own project cannot be told apart" is false; downgrading own-project urgent contradicts AL-22 "interrupt allow-list = own project's verified roster; urgent from others", and E-41.3 is rewritten instead of passed (02 11.2.d would support it, but that needs a ruling).
4. **WEAK** — it waives SI-11's "the home hub's current head is compared on every contact" by reinterpretation with no ruling, and does not require the head pull to precede message handling in the first pass after start.
5. **WEAK** — E-40.4 (a) asserts the literal status `ROLLBACK refused, kept seq 7`; the note emits lower-case text nested in `unverified since T (...)`, and the widened vocabulary binds P-24 without saying so.
6. **SOUND** — the three stated sizes disagree, so asserting none is right.
7. **WEAK** — it stretches a ruling made for `hub.key_card_put` to an irreversible action (item 13: no un-revoke) without asking, says the RPC "raises attention" although item 11 says the hub cannot, and defers the runme action that extract surface 8 requires.
8. **WRONG** — dropping any card or roster binding lets any caller mint keypairs and self-revoke each one with self-asserted `project`/`agent`, filling the 1 MiB cap of FCW-88 and spamming attention; 6.2.2 (b) limits this to "the agent's own project", and whether the added `public_key` is inside the signed canonical object is unstated.
9. **WEAK** — nothing makes mail to a revoked agent wait (`fx-a`'s sidecar still receives it), so E-39.4's "waits" is either false or vacuous; RK-1's reason text and attention entry are dropped from a PH-2 scope, and the sender record value "refused-revoked" is never settled.
10. **WEAK** — 6.2.5's condition "once K-15's key card exists" is already met per the extract, and the clause is moved to another unit without a change request while P-15's gate is "the B-5 specs pass".
11. **WEAK** — sidecar-raised attention disappears when the pull is off or the hub "predates", is lost if a sidecar crashes between install and raise, floods on a fresh sidecar's first install, and drops AL-21's start-up check.
12. **WRONG** — AL-21 writes the admin row before the list exists, so a crash between row and persist, or the FCW-92 case, leaves the log ahead of the file and the hub refuses heads with no shipped recovery; that contradicts crash-table row 1 ("re-run is idempotent") and E-167.4 ("previous list served").
13. **WEAK** — behaviour at the cap is undefined (the hub must refuse before signing an oversized list), the 50 % attention has no store to live in (item 11), and item 8 makes the cap reachable by an attacker.
14. **SOUND** — a cheap explicit check consistent with FX-2 (the failure class should be stated as a failed pull).
15. **WRONG** — an unauthenticated `method not found` giving "no attention, no narrowing (PH-1 trust)" is a silent downgrade, against 06 4.7 ("a hub error or refusal response on either RPC" is a failed pull) and 6.2.4 ("Messages from keys on the last verified list stay refused"); the OFF flag file is also writable by any same-uid agent with no attention.
16. **SOUND** — tracked fixture plus generator closes the gap (V-NEG-3's full signature must be regenerated, since only its prefix is quoted).
17. **WEAK** — a status string does not perform E-127's ACT/ASSERT ("count refusals after each revocation row"; fire on "a message from a revoked key STORED"); no reader, counter or exit-1 path is delivered.
18. **WEAK** — this is a placeholder ("located in slice C after reading the hand-over code"): it does not establish that the notice is signed, or that both pending sets (AL-5 and AL-8) are cleared so "no ring or watchdog line names it afterwards".

## Findings the note misses

1. **The stored list is trusted on start without re-verification.** AL-22: "on start: held = store.revocation (last verified list, may be from a backup)", then `h.digest == held.digest` → `verified()`. A store file with an entry removed and the `digest` field untouched stays VERIFIED forever, against AL-20 "recomputed, never trusted from the object". The note has no load-time `verify(pin, held)` and no slice C mutant for it.

2. **First contact and the empty list are undefined at the sidecar.** Extract surface 3: "The empty list (seq 0, `prev_digest` 64 zeros…) has to be defined as served state." A fresh sidecar holds nothing, so against a seq-0 hub the `h.seq == held.seq` branch either yields "fork" with tamper or never verifies. Under item 3 that means every urgent message is downgraded for good on a healthy estate with no revocations.

3. **Forwarded messages bypass the check.** FCW-76: "the original sender's key is revoked … → refused or held exactly as a direct message would be" [AFC-22; E-39], and extract surface 5: "Forwarding (06 AL-15, FCW-76) applies the same." No slice, settlement or mutant checks the original sender's key.

4. **Test seams are not gated.** Extract surface 9: "simulated clock and a hub-client stub failing only the revocation RPCs; seams must be estate-gated (06 4.8, AL-16)." The note adds a clock seam and an RPC-failure seam with no gating; an ungated clock seam lets a local agent freeze the PN-13 timer.

5. **FCW-96 and the one-tick bound have no ordering rule and no mutant.** FCW-96 edge case: "the new list is fetched before any urgent delivery from others". E-39.4: "time from revoke to the first refusal ≤ one tick + one pull". E-40.4 (c): "verdict `LIMIT` naming RR-20, with no circuit and no urgent delivery from F". The note runs the pull "in the pass" (which runs on `poll_ms`, not the 30 s tick) without saying it precedes `handle_msg`. Section 4 has no mutant for "start as VERIFIED from the store", for the fork case, or for the `hub_id` check.
