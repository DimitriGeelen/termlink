**T-3366: FAIL — `hub adopt-id`, which runme runs automatically before every hub restart, silently re-mints a lost `hub.id` or signing key beside existing state (the refusal FCW-79 and mutant m5 exist to guarantee), and several PASS rows in the evidence map rest on tests or output the bundle does not contain.**

1. **`hub_identity.rs` `adopt_legacy` plus `runme.sh` `adopt_hub_id`: the refusal is undone by the deploy path.**
   - **Key:** with `hub.id` present and `hub.signing.key` missing, `load_or_mint` refuses, but `adopt_legacy` takes the `None => generate_key(dir)` arm and writes a new key. This is mutant m5's behaviour, live in the migration verb. It also contradicts AL-18 2.2.f(4) (key generated only by the hub process at start) and the PH-2 replacement rule (new key signed by the old).
   - **Id:** a migrated hub that later loses `hub.id` after a certificate rotation gets `first16(new certificate)`, or a random id if the certificate is unreadable to the runme user.
   - **Runme hides it:** `adopt_hub_id` runs on every restart where `exe != HUB_BIN`, not "once". It never checks `id_source` or `key_generated`, and prints only `head -1`, so the "hub.signing.key generated" line is dropped.
   - **Local check is vacuous:** `HUB_ID0` is read after the adopt, so "hub.id unchanged" compares the new id with itself; only the fleet path compares against the served id.
   - **Untested:** no test covers adopt with an id present and the key missing; m7 covers only the id.

2. **`hub_key_pin.rs` `observe`: a store read error becomes a re-pin.**
   - The pin is read with `store.read_state(..).ok().flatten()`, so an I/O error looks like "no pin". On the trusted path the `None =>` arm then writes the offered key over the existing pin, breaking 2.2.d(3) and FCW-83. `verified_hub_key` fails closed on the same error; `observe` does not.
   - `admin_row(hub-key-pinned)` (OBS-37) is not implemented and the evidence map does not mention it.
   - The pin is written from `check_pin` in both `send.rs` and `pull.rs` at runtime, not "only by the sidecar's start, before its tick loop" (2.2.f(4)).
   - A hub that stops offering `signing_key` while a pin exists yields `NotOffered` with no attention entry.

3. **Spec readings under Evolution: one is not closed, the others hold.**
   - **"Prior state" (`has_prior_state`):** only half defensible. The spec is self-contradictory: step 1 makes the "certificate present now" branch unreachable, and E-36.3 starts a cert-only directory. But the agent resolved it alone, toward the less fail-closed side, with no change request or operator ruling; it is not under Decisions. A directory holding `hub.cert.pem` and `hub.key.pem` that has lost `hub.secret` and `hub.id` now mints silently.
   - **Topic logs:** the `bus/` location is assumed. The only test creates an empty directory `bus/x` and asserts `is_err()` without checking the message.
   - **Mint before the lock (`server.rs` `prepare_start`):** `load_or_mint` runs before `claim_runtime`, so 2.1.f(3) does not cover the mint. Two concurrent first starts can leave the lock winner serving an id that lost the rename (invariant 3), unless an unquoted earlier check prevents it.
   - **Defensible:** minting before the hub creates its own certificate (CR-6.42), and not holding mail on a key mismatch.

4. **Runme order: safe inside the three restart points, not outside them.**
   - **Covered:** `restart_hub`, `install_hub_unit` and the fleet swap all adopt first and abort on failure.
   - **Window:** the new binary is already at `$HUB_BIN` before step 5 (the log shows sidecars restarted on it first). A hub crash, systemd auto-restart, reboot or runme abort in that window starts the new binary without `hub.id`, and it refuses to start. Nothing in the bundle closes or tests this.
   - **Ownership:** adopt runs as the runme or remote-shell user, while `read_key` requires owner equal to the hub's euid. If the hub runs as a different user, it refuses after the swap (FCW-82). Runme checks only `-s hub.signing.key`.
   - **Fleet compare:** if `fleet_hub_field` returns an empty `id0`, the served-id comparison is skipped. On a mismatch the wrong `hub.id` has already been written and stays on disk.

5. **Evidence map claims the bundle does not back.**
   - **E-126:** the spec asks for status `hub {id, declared, match}` per tick and a fake-status control. Neither is in the diff or tests; the cited runme fixtures and "375 passed" are not quoted.
   - **E-166 third leg:** `fc_8_hub_down_*` and `send_hub_hub_down_*` are named but not quoted.
   - **E-37:** the AR-1 LIMIT run that E-37.5 requires is "not separately run". The SI-25 scan only searches the sidecar store for a seed the sidecar never receives; no hub log, journal or status is scanned. "Every hub-signed object refused" is one gate function with no caller.
   - **AC 5:** no test-run output is quoted, and the mutant "killed" results are a table of assertions. E-38 and E-163 were dropped from plan row P-10 by the agent's own statement, not a ruling.
   - **Commits:** mutants ran on `7ddb462c6`, the fielded binary is `g33e6cd61d`, and nothing ties either to "the close commit", so the deployed binary is not shown to be the reviewed code.
   - **Live check:** `hub.version` is quoted for the local hub only, not the three fleet hubs.
