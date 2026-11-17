**Verdicts**

- Slicing (§2): WEAK — the split at AL-20 is defensible, but P-16's done-criterion is "a PASS line per listed E-n" and slice A can pass neither E-69 (its ACT includes both replacement cases) nor E-70; slice B has no build or test list, and "needs no AL-20" is contradicted by item 11's option to fetch the hub's card.
- 1 (card key): WEAK — it defines the `from_agent` lookup only for msg/stage payloads although item 4 also gates `sidecar.liveness.v1`, compares `card.signing_key` literally with raw `sender_pubkey_hex` although item 15 stores it prefixed, and never validates the attacker-supplied `from_agent` used to find `<agent>.json`.
- 2 (bytes verified): SOUND — keeping `canonical_sign_bytes` and adding key == card key is the coherent reading of AL-33 step 2 and reuses the verify at `channel.rs:906-952` unchanged.
- 3 (project binding): WEAK — `card.project` is stored but never compared with anything, so R-75.e rests wholly on hub-unique names, which a runme put that overwrites a card with a different `project` does not enforce, and no fixture has two projects.
- 4 (message types): WEAK — the gate keys on a poster-chosen `msg_type`, so a hub-secret holder (BP-3, "closed for posting") still posts to `inbox:*` unchecked under any unlisted type, and future arc-011 types fail open.
- 5 (roll-out): WEAK — `off` by default means the build delivers no refusal (R-75.e, FCW-135) until someone flips it; no plan unit owns the move to `enforce`, there is no exit criterion for `log`, and setting changes are not admin-logged.
- 6 (hub public key): SOUND — publishing the key and checking its sha256 against the pin closes the gap for the pinned local hub, though AL-34's "`d.hub_id == declared home hub`" for a remote home hub is not covered.
- 7 (where DEAD lives): WEAK — "recorded as open" is not a settlement: the instance registry, the record's topic or RPC, the hub's answer to a post for a retired instance, and `mail_guard` finality (code map 4d) all stay undesigned, and P-15's form decides none of them.
- 8 (DEAD seq): WEAK — with "seq per project", a sender that has seen seq 9 for one instance will forever refuse the legitimate seq-5 DEAD of another instance it polls later; it also relaxes FCW-136's "its sequence does not rise → refused" without a spec change.
- 9 (who may write): WEAK — `execute` is the general exec scope (FCW-130), not a runme credential, so "a runme act" (FCW-134) is indistinguishable from any same-uid token holder; a silent overwrite raises no attention, and TH-56's "operator approval bound to a digest" is absent.
- 10 (name clash): SOUND — keeping `DEAD` and `DEAD_LETTER` apart matches R-44 and code map 4a, provided slice B really adds `DEAD` to the sender vocabulary at `api.rs:44-57`, which the note does not say.
- 11 (receiver): WEAK — it leaves two sources undecided: register files are host-local and carry no project, and an unsigned card fetched from the hub is not authenticated by a pin of the hub signing key (the extract says that route needs a hub-signed card, hence AL-20).
- 12 (needs-attention): WEAK — in the adversarial case the refused poster is the forger, with no sidecar to raise anything, so the impersonated agent and operator get only a counter nobody polls, against AL-33 step 2 "refuse, admin_log, needs-attention"; an uncoalesced admin-log row per refusal also lets a token holder flood the log.
- 13 (canonical JSON): SOUND — slice A signs nothing, and using P-15's single AL-20 implementation avoids forking the V-REV-pinned form, though no vectors exist yet for `termlink.key-replace/1` or `termlink.dead/1`.
- 14 (two levels): SOUND — two harnesses match E-169 at L2 and E-69 at L4; E-70 (also L4) is simply not planned here.
- 15 (key format): SOUND — it fixes one form for the card as the gap asked, but `ed25519:<64 hex>` remains visually identical to `hub.version`'s `ed25519:<sha256>` fingerprint, and item 6 adds a third, unprefixed form.

**Findings the note misses**

1. **Unvalidated `from_agent` used as a path at the hub and the receiver.**
   - The card lookup is `<runtime_dir>/key-cards/<agent>.json` and the receiver lookup is `/etc/termlink-sidecar/keys/<agent>.pub`, both driven by a payload field the poster controls.
   - A same-host poster can use a name such as `../../tmp/x` to point at a file it wrote carrying its own key, which passes FCW-135 ("no card for the poster → refused") without a registered card.
   - AL-23 step 3 demands "the conversation id is a plain name else refuse"; the note applies no equivalent rule to agent names.

2. **Slice B is named but not designed, so E-70 and half of E-69 have no path to a PASS.** The spec requires all of the following, and the note has no build or test list for any of them:
   - "`runme retire <project> <instance>`: hub writes {schema: termlink.dead/1 … seq: project_seq + 1 …} signed (AL-20); admin_log; attention";
   - "`seq = project_seq + 1` is taken under the hub's per-project lock";
   - E-70.4, "polling stops; unsigned DEAD refused and flagged; the message is never delivered to another instance";
   - E-70.5, "mutant accepting unsigned DEAD";
   - E-69.4, "new-key replacement refused; old-key replacement accepted and logged".

   The code map also says "the hub has no registry of instances and no notion of retired".

3. **No revocation check at the hub on post.** Architecture 6.2.5 says "A message signed by a revoked key is refused at the receiver (and at the hub on post, once K-15's key card exists) and recorded". The card check is that trigger, yet neither slice consults revocation, so a revoked key with a surviving card keeps passing the hub.

4. **Register and card become two unreconciled sources of truth.**
   - The hub trusts the card; item 11's receiver and the sidecar's own start-up trust the register.
   - P7-2 says "a key changes only through a runme action naming the change" and "a key that differs from the register is a refused start".
   - Slice B's old-key replacement therefore moves the card while the register stays put: the hub then accepts a key the receiver refuses and the agent's sidecar cannot start with.
   - Architecture 6.6.5 also labels that route "(C-6a, built-in route from PH-3)", while E-69 demands it in PH-2; the note does not notice the conflict.

5. **Crash and durability rules of AL-33 2.17.f are dropped.**
   - "(2) crash between persist and log: start-up compares card `seq` with the last logged `seq`" is absent from slice A; the note's "admin-log row first" only inverts the window.
   - Cards live under `<runtime_dir>`. If that directory is not durable, a restart loses every card and resets `seq`, against the rollback "check off at the hub; cards kept"; in slice B a reset `seq` would also let old replace statements replay.
   - Behaviour on an unreadable or corrupt card file is unstated, though E-169 arranges "a hub fake failing each read or write in turn".
   - 2.17.f(3), "a post is verified against the card it reads under that lock", is likewise unstated: the note locks only writes.
