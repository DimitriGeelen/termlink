**FAIL** — every round-2 BLOCKER and MAJOR is closed as its disposition says, except one piece of C-5. The C-7 fix introduces one new MAJOR: the global 60-row cap lets any poster suppress the admin row for a real impersonation refusal.

## Findings marked A

- **C-1 / O-1 (BLOCKER) — CLOSED** — `put` calls `log(&card, old.as_ref())?` before `save`, the router's closure returns the `admin_note` error, and the unit test asserts the store does not exist inside the callback and after a failed row. The consumer is the `key-card-written` loop in `pull.rs`, asserted on the real hub ("seq 1").
- **C-2 — CLOSED** — `load` refuses when the file digest differs from `KNOWN_DIGEST`, `save` and `startup_reconcile` re-baseline, and the reconcile compares `card_digest`. The unit test shows the refusal for a post and a put, then a report and pass after reconcile (see new defect 2 for a gap).
- **C-3 — CLOSED** — the replace condition now covers `encryption_key` and `instances`, and `put` refuses a signing key already on another agent's card. `r2_every_field_change_needs_replace_and_a_key_has_one_card` covers swap, drop, instances and the second card.
- **C-4 (A part) — CLOSED** — `check_agent` returns `Ok` when `self.1.is_empty()`. No test or mutant in the bundle pins this; its only effect is that a non-plain `from_agent` passes an unnamed roster.
- **C-5 / O-2 — CLOSED in part** — the type-filter test with m20, the unwritable-row case (unit, real hub, m18) and Q-claims-P (`-32044`) are present. "Receiver E-69 on a real hub" was not delivered: the disposition keeps it at unit level under an A label, with no reason why a named-roster `Receiver` cannot run in the existing harness.
- **C-6 — CLOSED** — `real_sidecar_msg_and_stage_pass_enforce_with_their_card_and_an_impostor_does_not` stores a real `Sender` msg and stage in enforce (count +2). The impostor half is weak (new defect 6).
- **C-7 — CLOSED as stated, but the fix is defective** — the `_uncarded` bucket, `MAX_SIGNERS = 16` and `MAX_ROWS_PER_MINUTE = 60` are in the code and unit-tested. The cap is new defect 1.
- **C-8 / O-3 — CLOSED (code side)** — `handle_key_card_list` returns mode, refusals and `hub_start_ms`. The amended design text is not in the bundle, so that half is unverified.
- **C-9 — CLOSED** — `check_agent` collects every name bound to the key (alias case added to the wire test), and `roster_of` refuses a repeated name (`named_entries_bind_and_a_name_given_twice_is_refused`).
- **C-10 — CLOSED as stated** — `set_mode` logs first, the reconcile compares against the last `key-card-mode` row, `write_mode` takes `STORE_LOCK`, and `is_gated` runs before `effective_mode`. The residual is not closed and not claimed: deleting the setting file outside the RPC still turns the check off at once, and restoring it before the next start leaves no trace.
- **C-11 — CLOSED** — state is `"{hub_start_ms}:{total}"` with base 0 on a new start, and the total sums only signers other than the agent's own fingerprint. Only the basic path is tested; the restart and own-key paths have no test in the bundle.
- **C-12 (A parts) — CLOSED** — `by` is `"hub.key_card_put (local socket)"`, `handle_key_card_mode` checks local-only first and refuses a missing `mode`, and a key on two cards is refused.
- **O-4 — CLOSED** — the real-hub test expects `-32044` for project Q's key claiming `fx-a`. The fresh-key local-socket post (`-32043`) stands in for the hub-secret-only case; that equivalence is asserted, not shown.

## Findings marked R or D

- **C-4 R (no receiver off/log/enforce, no unbound counter) — SOUND** — an unnamed roster changes nothing and naming is a per-receiver operator act, so roll-out can be staged. The cost is no observe phase before enforcement at a receiver; the recorded deviation from design v2 item 11 needs the design owner's sign-off.
- **C-5 D (lock-free-read mutant) — SOUND** — a concurrency mutant under one global lock has no deterministic red test, and it is recorded as a test limit.
- **C-12 R (per-agent locking) — SOUND** — one small file is rewritten whole, so a single lock is correct and has no nesting.
- **O-5 R (global lock latency) — SOUND** — puts are rare local-socket acts. The lock now also spans the admin-row write, which is still milliseconds.

## New defects

1. **MAJOR** — `key_card.rs` `first_in_minute` (+ unchanged call in `channel.rs`) — the 60-row cap is global, keyed on the raw attacker-chosen name, and writes no "suppressed" marker. Any poster can therefore blind the admin log for a chosen minute.
   - **Sequence:** at the start of a minute, key K1 posts 60 `sidecar.msg.v1` records with `from_agent` = `x0`…`x59` (no cards). Key K2 then posts as carded `fx-a`. That FCW-133 refusal gets no `key-card-refused` row; K2 survives only in the in-memory signer map, lost on restart or shown as "other" once 16 signers exist.
   - **Fix:** coalesce uncarded refusals under `UNCARDED` (one row per signer per minute, with a count), give carded mismatches their own budget, and write one "n rows suppressed" row at minute rollover. Add a test that a carded refusal is still logged after 60 uncarded ones.

2. **MINOR** — `startup_reconcile` — the `Err` arm of `load_raw` never calls `set_known`, so a hub that started on an unreadable store has no baseline and accepts any later outside write.
   - **Sequence:** write `{x` to the store, restart the hub (report says only "corrupt"), write a forged store with `fx-a` on key 7. `check_post` now passes key 7 with no "changed outside" refusal.
   - **Fix:** on `Err`, baseline a sentinel that no file digest can equal.

3. **MINOR** — `pull.rs` `observe_impersonation` (2) — the consumer cannot see three cases.
   - **Same-seq edit:** the entry is keyed `agent:seq`, so an outside edit accepted after a restart raises nothing at the sidecar, though the router comment says "seq/digest change".
   - **Silent failures:** a failed `attention_coalesced` is dropped with `let _ =`, and a failing `hub.key_card_list` (the "changed outside" state) returns silently.
   - **Mode changes:** `key-card-mode` rows, including a switch to `off`, have no consumer at all.
   - **Fix:** key on `agent:seq:signing_key` or a digest returned by the list; report write failures as part (1) does; raise attention on a list error and on a mode change.

4. **MINOR** — `set_mode` — the `before` read and the `log` call run outside `STORE_LOCK`; only the file write is locked.
   - **Sequence:** two concurrent calls log `enforce` then `off` but write `off` then `enforce`. The file and last row disagree until the next start, and the `from` values are wrong.
   - **Fix:** hold the lock across read, log and write, using an unlocked inner writer.

5. **MINOR** — `startup_reconcile` — it walks only the cards in the store, and it needs `card_digest` in the row.
   - **Missing cards:** a logged card that is deleted outside the RPC, or a deleted store file, is re-baselined with no report.
   - **Pre-change rows:** rows written before this change have no `card_digest`, so every existing card is reported at every start until it is re-put.
   - **Fix:** report logged agents missing from the store; fall back to `seq` for rows without a digest, or document the one-time re-put.

6. **MINOR** — tests — three assertions are weaker than their names.
   - **Impostor:** refusal is shown only as "count unchanged after 500 ms", while the genuine path needs a wait loop of up to 10 s. Assert the `-32044` or the `fx-a` refusal count instead.
   - **Unwritable row (real hub):** asserts only `!status.success()`. Assert the "admin-log row not written" text.
   - **Unlisted type:** asserts only that the `inbox-unknown-type` entry exists. Also assert that no message was delivered.
