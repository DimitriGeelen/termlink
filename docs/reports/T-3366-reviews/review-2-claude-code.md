**T-3366: FAIL — the "Fixed" dispositions 5e and 5b are contradicted by the bundle itself (mutants ran on `d04bfe9f7`, the deployed code is `adf4555e0`), and the accepted reason for 2c does not hold against the code shown.**

1. **Section 7 vs sections 9–15 (disposition 5e, "Fixed"): the mutants are still on a different commit from the deploy.**
   - The mutant report ends "Run 2026-11-16T09:09Z on d04bfe9f7"; the code under review and the fielded binary are `adf4555e0`.
   - Nothing in the bundle says what changed between the two, and the section 14 result lines carry no hash at all ("round-2 source").
   - This is round-1 finding 5e unchanged, so AC 5 ("mutant list all killed") is not shown for the reviewed code.
   - The AC text also still says "12/12 mutants killed" while the report says 19.

2. **`hub_key_pin.rs` `observe`, the `None =>` pin arm (dispositions 2c and 2b, "Accepted"): neither reason survives the code.**
   - **2c:** the reason says both writers can only write "the same offer". But `send.rs` and `pull.rs` each make their own `hub.version` call and pass their own reply to `observe`.
   - `observe` reads the pin and then calls `write_state` with no lock and no no-clobber publish; the disposition itself describes `write_state` as temp + rename.
   - If the hub's key changes between the two calls (restart with a swapped key file, or another process on the socket), both threads see no pin, both write, and the second replaces the first.
   - That is a pin changed without a runme action (2.2.d(3)), with both threads reporting `NewlyPinned`. The hub side got `hard_link` for exactly this race; the sidecar side did not.
   - **2b:** the pin is written first and `alert` only does `eprintln!` if `store.attention` fails, so a pin can exist with no durable record. The spec asks for an admin row and an attention entry.
   - Dropping OBS-37 was decided by the agent with no operator ruling, the same pattern round 1 objected to in 3a.

3. **Sections 6, 14 and 15 (disposition 5b "Fixed", and AC 2): the cited evidence is not in the bundle.**
   - **5b** claims the result lines of `fc_8_hub_down_*` and `send_hub_hub_down_*` are quoted. Only their names appear, plus "full hub+sidecar+CLI suites: all-rc=0".
   - **AC 2** cites `runme-20261116T073957Z`, which is not quoted and was run on the round-1 code, whose local "unchanged" check was vacuous (finding 1d).
   - The quoted round-2 log shows only "kept (already present)" for all four hubs, which shows the files exist, not that the adopted id equals the pre-migration id.
   - The local key-owner fixture has a "skipped as non-root" branch (line 1409), and section 13 lists names from the source, not which branch ran.
   - The 20 unit-test bodies are omitted, so m13, m14, m15, m16 and m19 rest on test names alone. m19 is "killed" by `e36_first_start_with_cert_takes_the_certificate_id`, whose name describes the behaviour ruling B removed.

4. **`runme.sh` `adopt_hub_id` and the fleet block in `upgrade_fleet_hubs` (dispositions 1c, 1d, 4a): fixed only on the main path.**
   - **Fleet output is discarded:** the remote adopt runs with `>/dev/null 2>&1`, so `key_generated` and "minted at random" never reach the log. Finding 1c is fixed for the local hub only, and AL-17 requires the migration to be logged.
   - **The guard skips directories the new binary refuses:** `[ -e hub.secret ] || [ -e hub.id ]` returns 0 ("nothing to migrate") for a directory holding only a certificate, only a signing key, or only `bus/` logs. `has_prior_state` then refuses to start on it (FCW-79) after the binary is installed. This contradicts ruling B's stated cost, "an `adopt-id` step runme already runs".
   - **No socket means no `--expect`:** when `hub.sock` is absent (hub down at step 4), adopt runs blind and `HUB_ID0` is the id just written, so the later compare is the adopted id against itself, as in 1d.
   - **Socket-only legacy hubs cannot be migrated while running:** a served null id fails the 16-hex regex, so runme stops with "not migrating blind" and the CR-6.42 random-id path is unreachable through runme.
   - **The uid check can be silently skipped:** it hardcodes the unit name `termlink-hub` instead of `$HUB_UNIT` and is skipped when `MainPID` is empty.

5. **`hub_identity.rs` `adopt_legacy`, the `(None, None)` arm: the two-file migration is not atomic, and "both absent" does not prove a pre-T-3366 hub.**
   - **Self-inflicted stuck state:** `write_id` succeeds, then `generate_key` fails (ENOSPC, a crash, or `hard_link` unsupported); `hub.id` stays on disk with no key and no rollback.
   - Every later adopt then hits `(Some, None)`, "already migrated and lost its key", and `load_or_mint` refuses too. Its message tells the operator to run `adopt-id`, which refuses. No re-key verb exists, and no test covers this.
   - **Lost both files:** a migrated hub that lost both files after a certificate rotation is indistinguishable from a legacy hub. With the hub down it gets `first16(new certificate)` and a new key, which is round-1 finding 1b by another route. The stated fix, "a key on disk marks a migrated hub", does not cover it.
   - **Same pattern in `load_or_mint`:** a first start that crashes between `write_id` and `generate_key` is refused forever, where 2.2.f(3) says the next start generates.

The round-1 fixes 1a, 2a, 2d, 3b, 3c, 4c and 5a are borne out by the code and tests shown, and 3a is properly closed by operator ruling B. I could not run anything.
