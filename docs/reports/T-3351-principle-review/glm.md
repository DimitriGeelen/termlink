1. **Soundness and flaws in P1–P4**

1a. The direction is sound: it matches the five-level identity model, answers OQ-7, OQ-10 and OQ-11 in one coherent move, and reuses machinery already ruled in (per-agent keys, signed cards). The fallback rule — never cross a project boundary, wait or dead-letter — is consistent with the existing delivery model.

1b. The mailman analogy understates the hub. A mailman needs no trust; this hub is also the key directory (cards), the recovery store, and the health monitor. The real attack surface is key substitution at first contact, which the analogy hides — the hub needs bounded, stated trust, not "none".

1c. The envelope is richer than a postal one: conversation id + sequence + priority + stage lets the hub reconstruct workflow state and the full who-talks-to-whom graph with timing. That is a permanent metadata leak; the principle should declare it an accepted residual rather than imply confidentiality of relationships.

1d. P1 and P3 contradict each other: P1 grants a project read access to its agents' mail, but P3 encrypts agent-addressed mail only to the agent key. Either agent mail is also wrapped to the project (then every sibling reads it), or P1's grant has no mechanism. This must be resolved before adoption.

1e. P4 is unenforceable today: Tier 0 is a harness hook that does not bind root, and GP-11 means no operator key exists. As written, P4 is a promise, not a mechanism.

2. **P1 grouping**

2a. Some project-level read is load-bearing: mail that falls back to project level must be decryptable there, or fallback is theater.

2b. But blanket project read collapses agent-level confidentiality — every sibling agent reads every other's mail, including credentials mailed to one agent. After per-agent OS accounts land, it would quietly undo them.

2c. Better grouping: the project reads by right only project-addressed mail and dead-lettered/fallback mail; reads of a live agent's mail are logged, deliberate audit acts, never ambient. Or scope read to conversation participants, with project read only on the two failure paths.

3. **P3 mechanism and smallest version**

3a. Right mechanism, and the standard one: a random per-message content key, wrapped to recipient keys. Reuse existing agent keys (add an encryption subkey on the same card). For project-addressed mail, the smallest version is fan-out — wrap the content key to each project agent's key — avoiding a project-key distribution/rotation subsystem entirely at home-lab scale. A real project key can come later; membership change then means re-wrapping message keys, not re-encrypting stores.

3b. Recovery: the hub keeps the encrypted copy (answering OQ-10: content yes, ciphertext only) under a bounded retention window; the recipient re-fetches and decrypts. With no plaintext path at the hub, OQ-7 and OQ-11 close by construction.

3c. Do first, in order: 1) pin each hub's card-signing-key fingerprint out-of-band (fleet pin list in hubs.toml) — without it, a hostile hub voids the whole principle; 2) encrypt agent-to-agent DMs, the dominant confidential traffic, leaving broadcast topics plaintext; 3) project-addressed mail via fan-out; 4) break-glass only after GP-11.

3d. Keep signatures over the plaintext inside the encryption (recipient-verifiable), plus the already-ruled outer signature for routing.

4. **P4 break-glass**

4a. An off-host operator key is the right pattern: host compromise cannot silently use it, and the act of retrieval is itself the approval. It converts today's ambient operator read into a deliberate one — the actual win over OQ-11.

4b. Guarantees: approval equals the physical/interactive retrieval, never a host-side gate; scope is per-recipient or per-conversation unlock, never "decrypt everything"; every use is logged with scope, accepting that root can delete local logs (the off-host record of retrieval is the tamper-evident one); recovery sessions are time-boxed; affected keys rotate after any recovery tied to suspected compromise; the key is never needed for normal operation — and test that.

4c. Simplest construction: wrap every message key additionally to the operator's public key. State plainly that whoever steals that key reads everything, and that until GP-11 delivers it, P4 must be labelled not-yet-real.

5. **Costs and breaks**

5a. Hub content functions degrade to envelope-only: content search, body-based digests, payload-sniffing telemetry. Stuck-message detection and canaries survive — P2 deliberately keeps sequence, stage and acks in the envelope — but verify this per canary.

5b. Debugging gets harder: you cannot inspect a message at the hub. Ship receiver-side "decrypt and show my own mail" tooling (logged), or field diagnosis will route around the principle.

5c. Local root still reads agent keys until per-agent OS accounts land. This principle closes the hub axis, not the local-host axis; say so rather than implying agent-to-agent privacy on a shared-root host.

5d. The ruled requirements survive: signing is orthogonal; urgent-mail downgrade lives on envelope priority; circuits already ruled safe without the agent key should use the same E2E wrapping.

5e. Hub retention of ciphertext needs an explicit bound, or the recovery copy becomes a forever-store with a forever metadata map.

6. **Recommendation**

6a. Adopt with changes. The direction is correct and closes OQ-7/10/11 coherently at home-lab cost; the flaws are boundary conditions, not the core. Required changes: 1) resolve the P1/P3 contradiction via the logged-audit-read model of 2c; 2) make card-fingerprint pinning a prerequisite, not a caveat; 3) bound encrypted-copy retention; 4) mark P4 blocked on GP-11; 5) ship in the 3c order — pinning, then DM encryption, then project mail.
