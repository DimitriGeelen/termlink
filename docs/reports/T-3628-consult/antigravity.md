I will start by reading `crates/termlink-session/src/tofu.rs` and `scripts/check-pin-events.sh` to examine the `KnownHubStore` implementation, how `known_hubs` is loaded and saved, and how pin events are recorded and checked.
I will now read the remainder of `crates/termlink-session/src/tofu.rs`, as well as `registration.rs` and `known_peers.rs` referenced in comments, to see how atomic writes and trust stores are handled across the crate.
I will search the repository for all references to `KnownHubStore`, `known_hubs`, `build_tofu_connector`, and `tofu` to locate every code path and test that reads or writes `known_hubs`.
I will use `grep` to find all references to `KnownHubStore`, `known_hubs`, `build_tofu_connector`, `tofu`, and file locking across the repository.
I will now read the full grep output and inspect `crates/termlink-session/src/client.rs`, `crates/termlink-cli/src/commands/infrastructure.rs`, `tests/pin-events-fixtures.sh`, and other persistence modules in `crates/termlink-session/src/`.
I will now inspect `offline_queue.rs`, `ack_retry.rs`, `remote.rs`, `mcp/src/tools.rs`, and search for all uses of `TERMLINK_IDENTITY_DIR`, `TERMLINK_PIN_EVENTS_FILE`, and file locking across `crates/`, `scripts/`, and `tests/`.
I will now read the relevant sections of `crates/termlink-mcp/src/tools.rs`, `crates/termlink-cli/src/commands/remote.rs`, and `crates/termlink-sidecar/src/store.rs` to inspect all callers of `KnownHubStore` and the codebase's existing `flock` patterns.
I will read the fleet status and doctor implementations in `crates/termlink-mcp/src/tools.rs` and `crates/termlink-cli/src/commands/remote.rs`, as well as atomic write helpers in `crates/termlink-sidecar/src/store.rs`, to see how concurrent connections and stores behave.
I will search for all callers of `build_tofu_connector`, `connect_remote_hub`, and `connect_addr` to trace both single-process (in-process multi-task) and multi-process concurrency paths.
I will read the full grep output from step 35 to check all callers of `connect_remote_hub` and `connect_addr`, including parallel task spawns inside single processes.
I will read `crates/termlink-cli/src/commands/remote.rs` around lines 2480-2550 and 2890-2950, and `crates/termlink-mcp/src/tools.rs` around lines 8070-8220, to see how multi-hub commands connect to hubs within a single process.
I will read `crates/termlink-cli/src/commands/remote.rs` around line 10044 and search `tests/` and `scripts/` for any tests or scripts that touch `known_hubs`, `hubs.toml`, or run fleet/remote commands.
I will search the entire repository (including `runme.sh`, `.agentic-framework`, `crates`, `scripts`, and `tests`) for every occurrence of `known_hubs` or `pin-events`.
I will read the grep output from step 50 to see every reference to `tofu` and `known_hubs` across `crates/` and `tests/`.
I will read `crates/termlink-mcp/tests/parity.rs` and `crates/termlink-cli/tests/cli_integration.rs` around the `tofu` tests to check how test directories and environment variables (`HOME`, `TERMLINK_IDENTITY_DIR`, `XDG_STATE_HOME`) are isolated during test runs.
I will now read `crates/termlink-test-utils/src/lib.rs` and lines 670-1170 of `crates/termlink-mcp/tests/parity.rs` to inspect how `termlink_cmd` and the parity tests manage `HOME`, `TERMLINK_IDENTITY_DIR`, and `XDG_STATE_HOME`.
I will read `meta.json` and `env.json` in the dispatch directory to confirm the exact path expected for `result.md`.
I will check `/tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/result.md` to see how the harness captures output.
I will read the end of `/tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/run.sh` to check how `result.md` is handled on exit.
I will read lines 450-550 of `run.sh`.
I will read lines 350-450 of `run.sh` to verify how `result.md` is captured.
I will read lines 310-368 of `run.sh`.
# Independent Review (T-3580): TOFU Pin Loss & `KnownHubStore` Concurrency Analysis

## 1. Numbered Findings & Verification of Suspected Cause (Question 6)

**Verdict on Q6:** The suspected cause in §5 is **confirmed and necessary**, and a **third amplifier in `KnownHubStore::accept` (`tofu.rs:285-291`)** is the missing piece that explains why read-only health checks and already-pinned hubs trigger continuous destructive writes.

1. **Finding 1 — Every TLS handshake to an already-pinned hub rewrites `known_hubs` (the hidden write-amplifier):**
   In [`crates/termlink-session/src/tofu.rs`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/crates/termlink-session/src/tofu.rs#L285-L291), when `KnownHubStore::accept` verifies a hub whose `fingerprint` already matches `existing.fingerprint`, it mutates `existing.last_seen = now` and unconditionally calls `self.save()`. Because [`Client::connect_addr`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/crates/termlink-session/src/client.rs#L88) and [`Client::connect_tls_stream`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/crates/termlink-session/src/client.rs#L142) call [`build_tofu_connector`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/crates/termlink-session/src/tofu.rs#L412-L422) on every remote connection, **every routine connection is a full-store writer**, even when the pin set has been completely static for days.

2. **Finding 2 — Fixed temp path (`known_hubs.tmp`) truncation race wipes the live store to 0 bytes:**
   [`KnownHubStore::save_result`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/crates/termlink-session/src/tofu.rs#L205-L234) writes to a fixed path `self.path.with_extension("tmp")` (`known_hubs.tmp`, line 231) via `std::fs::write` (`open(O_CREAT|O_TRUNC)` + `write` + `close`, line 232) followed by `std::fs::rename` (line 233). Under concurrent saves (Processes $P_1, P_2$ updating `last_seen`):
   - $P_1$ finishes `write("known_hubs.tmp")`.
   - $P_2$ calls `open("known_hubs.tmp", O_CREAT | O_TRUNC)`, truncating the shared `known_hubs.tmp` inode to **0 bytes** before $P_2$'s `write()` runs.
   - $P_1$ executes `rename("known_hubs.tmp", "known_hubs")`, atomically replacing `known_hubs` with $P_2$'s **0-byte (or partially written) inode**. ($P_2$ then writes into the now-live `known_hubs` fd non-atomically and its subsequent `rename` fails with `ENOENT`.)

3. **Finding 3 — Silent read-error / torn-line ignore in `load()` converts a transient 0-byte race into permanent pin deletion:**
   [`KnownHubStore::load`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/crates/termlink-session/src/tofu.rs#L158-L183) silently ignores `fs::read_to_string` errors (`Err(_) => return`, line 161) and silently drops truncated lines (`if parts.len() >= 4`, line 171). Any process $P_3$ calling [`KnownHubStore::default_store`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/crates/termlink-session/src/tofu.rs#L153-L155) during the truncation window loads `{}` (or a 1-hub subset), connects to one hub (e.g., Hub E), and calls `self.save()`, overwriting `known_hubs` with **only Hub E** and permanently erasing Hubs A–D.

4. **Finding 4 — Stale-snapshot lost updates and why one PID pins A, B, C, D within the same second:**
   [`KnownHubStore::new`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/crates/termlink-session/src/tofu.rs#L143-L150) loads `known_hubs` before the TCP/TLS handshake, and `save_result` writes back only that in-memory `HashMap` after the handshake completes—with no lock and no re-read/merge. Once `known_hubs` is reduced to `{E}` (or `{}`), the next sequential multi-hub sweep command in a single process—such as `termlink fleet status` ([`remote.rs:2480-2505`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/crates/termlink-cli/src/commands/remote.rs#L2480-L2505)), `fleet governor-status` ([`remote.rs:2904-2916`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/crates/termlink-cli/src/commands/remote.rs#L2904-L2916)), `fleet doctor` ([`remote.rs:5290-5309`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/crates/termlink-cli/src/commands/remote.rs#L5290-L5309)), `net test` ([`remote.rs:6373-6382`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/crates/termlink-cli/src/commands/remote.rs#L6373-L6382)), or MCP `termlink_fleet_status` ([`tools.rs:16943-16953`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/crates/termlink-mcp/src/tools.rs#L16943-L16953))—iterates over hubs A, B, C, D in <1s, calling `build_tofu_connector` per hub and emitting four `tofu-pin` events with the same `pid`. If a concurrent process that loaded `{E}` before that sweep finishes its handshake slightly later, its `last_seen` save overwrites `known_hubs` back to `{E}`, repeating the cycle on the next burst (~7 minutes later).

5. **Finding 5 — Other paths checked:**
   - [`KnownHubStore::remove`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/crates/termlink-session/src/tofu.rs#L253-L264) and [`clear_all`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/crates/termlink-session/src/tofu.rs#L266-L277) unconditionally call `record_pin_event("tofu-clear" / "tofu-clear-all", ...)`; the absence of clear events rules out CLI/MCP clear commands.
   - [`scripts/deploy-remote.sh:125-129`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/scripts/deploy-remote.sh#L125-L129) also writes to fixed `${KNOWN_HUBS}.tmp` and `mv`s over `known_hubs` without logging to `pin-events.jsonl` (a separate bypass that should use `termlink tofu clear`).
   - [`crates/termlink-session/src/known_peers.rs:125-146`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/crates/termlink-session/src/known_peers.rs#L125-L146) has the same fixed `.toml.tmp` + unlocked snapshot save bug.

---

## 2. Should the Operator Acknowledge the 21 Events Now? (Question 7)

**Answer: YES — acknowledge the 21 events now, subject to two immediate verification conditions; do NOT leave the check red waiting for the code fix.**

- **Why live-cert match alone is not quite sufficient:** Comparing `new_fingerprint` in `pin-events.jsonl` against the cert served *live over the wire* (`termlink tofu verify`) proves no transient MITM substituted a different cert during the 21 re-pin windows, **provided** the current live cert is authentic. It does not by itself prove a hub wasn't replaced before the first observed event, nor that `pin-events.jsonl` wasn't truncated (`check-pin-events.sh:43`).
- **Required conditions before running `bash scripts/check-pin-events.sh --ack`:**
  1. **Out-of-band / local-disk fingerprint check (or pre-incident baseline match):** Confirm each of the 5 fingerprints matches either the previously acknowledged pin in earlier lines of `pin-events.jsonl` / `.context/project/learnings.yaml` (`remote.rs:6817`) or the on-disk `hub.cert.pem` on each hub host (`termlink hub fingerprint`).
  2. **Verify current store completeness:** Confirm `termlink tofu list` currently shows all 5 hubs (A–E) with those exact fingerprints and `check-pin-events.sh --json` reports `truncated: false`.
- **Why not wait for the fix:** Leaving `check-pin-events.sh` red until the fix lands blinds the operator to any *22nd* event (alert fatigue / masked signal). Acknowledge the verified 21 events now so the tripwire resets to green and immediately catches any new event while the fix is developed.

---

## 3. Fix Design & Pitfalls (Question 8)

A complete fix in [`crates/termlink-session/src/tofu.rs`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/crates/termlink-session/src/tofu.rs) requires four coordinated changes:

1. **Advisory lock (`flock`) + Re-read/Merge-on-Save transaction:**
   - Open a dedicated sidecar lockfile `<known_hubs>.lock` (`0600`, never unlinked) and acquire an exclusive advisory lock (`libc::flock(fd, LOCK_EX)`, matching [`crates/termlink-sidecar/src/store.rs:304-320`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/crates/termlink-sidecar/src/store.rs#L304-L320)) around the **critical section inside `accept()`, `remove()`, and `clear_all()`**.
   - While holding `LOCK_EX`: re-read the on-disk `known_hubs` and **merge**:
     - Keep all on-disk `host_port` entries not explicitly removed in this call (**never implicitly drop entries** on `accept`).
     - If `accept(host_port, fp)` finds `host_port` was added on disk by a concurrent process while our handshake was in flight: if `disk.fingerprint == fp`, treat as `is_new = false` (do **not** emit a duplicate `tofu-pin` event; preserve `min(first_seen)` and `max(last_seen)`); if `disk.fingerprint != fp`, return `Err(TOFU VIOLATION)`.
     - Fail closed on corrupt/unreadable non-empty `known_hubs` during merge rather than silently replacing with an empty map.
2. **Unique, `0600`, `fsync`ed temp file before `rename`:**
   - Replace `self.path.with_extension("tmp")` (`tofu.rs:231`) with a unique sibling path in the same directory: `.known_hubs.tmp.<pid>.<counter>` opened with `OpenOptions::new().write(true).create_new(true).mode(0o600)`, call `file.write_all(...)`, `file.sync_all()`, `rename(&tmp_path, &self.path)`, and `fsync` the parent directory (`store.rs:278-291`).
   - Thus, lockless readers (`get`, `list_all`) never need a lock and always observe either the complete old file or complete new file via POSIX atomic `rename`.
3. **Eliminate unnecessary writes on `last_seen`:**
   - In `accept()` (`tofu.rs:285-291`), do **not** rewrite `known_hubs` on every matching handshake; either stop persisting `last_seen` on every connection or throttle `last_seen` disk writes to e.g. once per hour (`now_secs - last_seen_secs >= 3600`). This removes >99% of concurrent writes during fleet sweeps.
4. **Fix `scripts/deploy-remote.sh:125-129`:**
   - Replace the raw `grep -v ... > known_hubs.tmp && mv` with `"${LOCAL_BIN}" tofu clear "${HOST}:${PORT}"`.

**Pitfalls & Portability:**
- **Crashes holding a lock:** Kernel-level `flock` / `LockFileEx` is tied to the open file description and is automatically released by the OS on process exit or `SIGKILL`. (Never use stale-prone dotlock files without `flock`).
- **Locking the data file vs. `.lock` sidecar:** Because `save_result` replaces `known_hubs` via `rename` (new inode), locking `known_hubs` directly would lock an unlinked inode. Always lock a persistent `<known_hubs>.lock` file that is never unlinked.
- **Portability (Linux/macOS/Windows) & NFS:** `flock` works on Linux (`>= 2.6.12` on NFSv4) and macOS; on Windows use `LockFileEx` (or the `fs4`/`fd-lock` crate). Using unique temp filenames (`pid + atomic counter`) ensures that even if advisory locking degrades on broken NFSv3, concurrent writers never truncate each other's temp file.

---

## 4. Detecting "Pin Lost and Re-Created" vs. "Genuinely New Hub" (Question 9)

- **Should it be treated differently?** **Yes.** A first-use pin for an address that was previously pinned in the same identity directory without an intervening `tofu-clear` / `tofu-clear-all` is either store corruption/loss or tamper (`rm ~/.termlink/known_hubs`).
- **How to detect it:** `pin-events.jsonl` ([`tofu.rs:68-108`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/crates/termlink-session/src/tofu.rs#L68-L108)) is already an append-only ledger in the same identity directory. Before accepting a missing `host_port` in `accept()` (while holding `LOCK_EX`), scan `pin-events.jsonl` for the last event matching `address == host_port`:
  - **No prior event for `address`:** Genuinely new hub → allow TOFU and record `tofu-pin`.
  - **Last event is `tofu-clear` or `tofu-clear-all`:** Operator-authorized re-pin → allow TOFU and record `tofu-pin` (carrying `old_fingerprint` from the clear event or `repin_after_clear: true`).
  - **Last event is `tofu-pin` (no intervening clear):** The pin was **lost from `known_hubs`**!
    - If `last_event.new_fingerprint == fingerprint`: auto-heal the entry in `known_hubs` and emit a distinct `tofu-repin-same` (or `tofu-recovered`) event so `check-pin-events.sh` flags the store-loss bug separately from new hubs.
    - If `last_event.new_fingerprint != fingerprint`: **REFUSE connection (`TOFU VIOLATION`)**! Otherwise, deleting `known_hubs` silently downgrades a pinned hub back to TOFU and allows MITM cert substitution.

---

## 5. Regression Test (Question 10)

Add an integration test in `crates/termlink-session/src/tofu.rs` (and a multi-process test in `crates/termlink-cli/tests/cli_integration.rs`):
1. Create a shared `TERMLINK_IDENTITY_DIR` and `TERMLINK_PIN_EVENTS_FILE`.
2. Spawn $N = 32$ concurrent OS processes (or threads with independent `KnownHubStore::new(path)` instances), barrier-synchronized to start simultaneously:
   - 16 processes each accept a distinct hub `host-{0..15}:9100` (`sha256:fp-{0..15}`).
   - 16 processes concurrently accept the *same* 5 shared hubs `shared-{0..4}:9100` (`sha256:shared-{0..4}`) repeatedly (exercising both first-pin and `last_seen` paths).
3. Assert after all 32 exit:
   - Reloading `KnownHubStore::new(path)` contains **all 21 hubs** (`16 + 5`) with intact fingerprints.
   - Zero `.tmp*` files remain in the directory.
   - `pin-events.jsonl` contains **exactly 21 `tofu-pin` lines** (1 per unique `host_port`, zero duplicate `tofu-pin` events for `shared-{0..4}:9100`).

---

## 6. Confidence

**High (99%).** Every symptom in §3–4 (no `tofu-clear`, bursts during multi-process test/health runs, single-PID sub-second re-pinning of A–D, and repeated re-pinning 7 minutes later) is directly explained by the interaction of [`tofu.rs:285-291`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/crates/termlink-session/src/tofu.rs#L285-L291), [`tofu.rs:231-233`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/crates/termlink-session/src/tofu.rs#L231-L233), and [`tofu.rs:158-183`](file:///tmp/tl-dispatch/t3628-consult-antigravity-47c183c4b6e9/tree/crates/termlink-session/src/tofu.rs#L158-L183).
