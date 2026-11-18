**Verdict: FAIL** — the hub comparison itself is sound (decoded bytes, fail-closed on a bad store or setting, non-gated types untouched) and I found no forged arc-011 post that passes in enforce mode, but the card-write controls fall short of the operator ruling and several design-v2 commitments are missing or untested.

I could only judge the diff. The admin-intent dispatcher, the socket/peer plumbing, the receiver's existing type filter and any other post paths are not in the bundle.

1. **BLOCKER** — `router.rs: handle_key_card_put` — The card is persisted first, and the only row naming old and new key (`key-card-put`, `attention: true`) is written afterwards; if that write fails it is logged with `tracing::error!` and the put still returns `ok: true`. This contradicts the ruling's "admin-log row first (refused if unwritable)" and "needs-attention on every put/replace naming old and new key". The bundle also shows no needs-attention raise at all, only a JSON flag in the admin log with no visible consumer. — Sequence: the intent row succeeds, `rpc-admin.jsonl` becomes unwritable (disk full, chmod), `put --replace` swaps fx-a's key, the caller sees success, and no row names the old key. It also leaves card seq ahead of the log until the next restart. — Fix: write the full row (old and new key, project, seq) before `save`, refuse the put if it fails, and raise a real operator attention entry or show the consumer of `attention: true`.

2. **MAJOR (limit worse than stated)** — `key_card.rs: startup_reconcile`, `check_post` — Visibility controls sit only on the RPC. The reconcile compares `seq` alone and only at start-up, while the hub re-reads the file on every post. — Sequence: a root agent edits `signing_key` for fx-a in `key-cards.json` and leaves `seq` unchanged; forged posts as fx-a pass enforce immediately, with no admin row and no attention, and the reconcile stays silent after restart. The code comment claiming it catches "a card written outside `hub.key_card_put`" is false for this case. — Fix: log a digest of (agent, project, signing_key, encryption_key, seq) in the put row; compare digests at start-up and whenever the store's mtime or hash changes at load.

3. **MAJOR (limit worse than stated)** — `key_card.rs: put`; `router.rs` row — `replace` is required only when `signing_key` or `project` changes. `encryption_key` and `instances` can be changed or dropped without it, and the admin row records neither old nor new encryption key. — Input: `put --agent fx-a --project p --signing-key <same> --encryption-key x25519-pub:<attacker>` succeeds without `--replace`, and the row shows an unchanged signing key. Omitting `--encryption-key` silently deletes it. — Fix: require `replace` for any change to `encryption_key` (and `instances`), and log old and new values.

4. **MAJOR** — `wire.rs: Roster::check_agent`, `check_signer`, `verify_open` — Disposition O-11/C-11 says the receiver follows off/log/enforce and counts unnamed entries as "unbound" in status. Neither exists: `check_agent` always enforces, and `bound_count()` has no caller and counts bound entries. The plain-name rule also applies unconditionally, even with an empty roster and the hub check off. — Input: a legitimate sender whose `from_agent` is `Fx-A` or `fx_a@host` is refused `Malformed` by every upgraded receiver on deploy, with no log-mode window to detect it. This becomes a BLOCKER if any deployed agent name fails the regex. — Fix: thread the mode into the receiver (log = count, don't refuse), add the unbound counter to status, and add a test.

5. **MAJOR** — mutant table and tests — Several committed proofs are absent:
   - The receiver type-filter mutant (O-4/C-4, "a new mutant proves that admitting an unlisted type is caught") is missing, and there is no test. This is the only thing backing the claim that arc-011 mail cannot be forged under another type; the real-hub test itself shows a `chat` post claiming fx-a is stored.
   - The lock-free-read mutant (design §4.6) is missing.
   - O-M5's test "put refused when the admin row cannot be written" is missing. `t3704-m10` is killed only by an `is_admin_method` list assertion, which proves nothing about refusal.
   - The receiver E-69 half exists only as unit tests, not on the real hub with the check off.
   - The two-project case exists only as a unit test.
   
   — Fix: add these mutants and tests.

6. **MAJOR** — `t3704_key_card.rs`, `t3704_impersonation.rs` — No test sends a genuine sidecar-built `Msg` or `Stage` through the hub in enforce mode. Both files hand-build `{"v":1,"kind":"msg","from_agent":…}`. The impersonation test constructs a `Sender` and never sends with it. — If the real payload places `from_agent` differently (the stage shape, for instance), enforce refuses all legitimate mail with FCW-135 and no test goes red. — Fix: add a real-hub test with a card for fx-a where `Sender` sends a msg and a stage in enforce, both stored and both verified by the receiver.

7. **MAJOR** — `key_card.rs: note_refusal`, `first_in_minute`; `channel.rs: key_card_gate` — Counters and coalescing are keyed on the attacker-chosen `from_agent`. — Input: a hub-access holder posts `sidecar.msg.v1` with `from_agent` = `a1`, `a2`, … (valid names, no cards). Each post adds a permanent `REFUSALS` entry (unbounded memory, returned in every `hub.key_card_list`, which every sidecar polls each pass) and writes one admin row, defeating the C-12 flood guard. — Fix: count per agent only for names that have a card; put the rest in one bucket; cap rows per minute globally and per signer.

8. **MAJOR** — `router.rs` — Design §4.3/§4.5 and dispositions O-M4/C-12 commit to the mode and per-agent refusal counts being "shown in `hub.status`". The diff shows them only in `hub.key_card_list`, and the sidecar reads that instead. — Fix: add them to `hub.status`, or amend the design and dispositions.

9. **MINOR** — `wire.rs: check_agent`; `daemon.rs: roster_of` — Two roster defects:
   - The third rule refuses whenever any binding of the signer key has a different name. Input: roster `a=K, b=K` (an alias); every message signed by K is refused. Fix: if the key is bound, require that some binding matches `from_agent`.
   - `bind` silently overwrites. Input: `a=K1, a=K2` leaves K1 rostered and unbound, free to claim any unbound name. Fix: reject duplicate names in `roster_of`.

10. **MINOR** — `key_card.rs: read_mode`, `write_mode`, `atomic_write`; `channel.rs: key_card_gate` — Three issues around the setting:
    - Enforce exists only as a file. Deleting `key-card-check` turns the check off with no admin row, and start-up neither logs the effective mode nor compares it with the last logged `key-card-mode` row. Fix: reconcile the mode at start-up.
    - `write_mode` runs outside `STORE_LOCK` with a pid-only temp name, so two concurrent mode calls can interleave in the same temp file. That fails closed but breaks traffic. Fix: write under a lock or use a unique temp name.
    - `effective_mode` reads the file on every `channel.post` of any type. Fix: test topic and type first.

11. **MINOR** — `pull.rs: observe_impersonation` — Two issues:
    - After a hub restart, if the new total equals the stored `seen` before the next tick, the refusals are never reported (seen = 5, restart, 5 new refusals). Fix: store the hub start id alongside the count.
    - An agent's own refused posts (no card yet, or any log-mode run before cards are published) are reported as "posts claiming to be you". This will fire for every agent during P-31 roll-out. Fix: exclude or separately label refusals where the signer is the agent's own key.

12. **MINOR** — assorted:
    - `by: "runme"` is hard-coded, so a card written by a root agent is attributed to runme. Fix: record peer credentials.
    - `hub.key_card_mode` with no `mode` is a read but is scoped Execute and listed as admin, so each read writes an intent row.
    - `STORE_LOCK` is a blocking mutex held across file read, parse and fsync inside the async post handler, with one lock for all agents where the design says per-agent.
    - `put` allows the same signing key on two agents' cards.
    - The bundle claims 13 key_card unit tests; the diff contains 12.
    - The CLI is `key-card mode`; the design says `key-card-check`.
