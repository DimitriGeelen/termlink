**VERDICT: PASS** — the round-2 BLOCKER and all MAJORs are closed as dispositions state (or soundly rejected/deferred), and the changes introduce only MINOR defects (below).

## A findings

- **C-1 (BLOCKER, O-1) — CLOSED**: `put` calls `log(&card, old)?` before `save` (key_card.rs:~355), so a failed row writes nothing; proven by unit `r2_the_admin_row_is_written_first_and_a_failed_row_writes_nothing`, the real-hub unwritable-log test (card unchanged, get returns original key), mutant m18, and the sidecar consumer (`key-card-written` per (agent, seq) in pull.rs, asserted in t3704_impersonation.rs).
- **C-2 — CLOSED**: `KNOWN_DIGEST` baselined in `save` and at startup, compared in `load` ("changed outside hub.key_card_put"); unit `r2_an_outside_edit_with_the_same_seq_is_refused_until_restart` + m17; reconcile now compares `card_digest`, not seq.
- **C-3 — CLOSED**: replace check extended to `encryption_key` and `instances` (key_card.rs:~332); unit `r2_every_field_change_needs_replace_and_a_key_has_one_card` + m19.
- **C-4 — CLOSED (A part)**: `check_agent` returns `Ok(())` when the named map is empty (wire.rs:~317), so an upgraded receiver with a legacy roster refuses nothing new.
- **C-5, O-2 — CLOSED**: unlisted-type real-hub test + m20, unwritable-row unit+real-hub + m18, and Q's key claiming fx-a at the real hub (-32044 in t3704_key_card.rs); the receiver half stays at wire/unit level with real keys, exactly as the disposition records.
- **C-6 — CLOSED**: `real_sidecar_msg_and_stage_pass_enforce_with_their_card_and_an_impostor_does_not` stores a genuine msg and stage in enforce and refuses the impostor sidecar.
- **C-7 — CLOSED**: uncarded names share `_uncarded`, per-agent signer maps capped at 16, refusal rows capped at 60/min (`MAX_ROWS_PER_MINUTE`); unit `r2_refusals_for_uncarded_names_share_one_bucket_and_rows_are_capped`.
- **C-8, O-3 — CLOSED**: `hub.key_card_list` returns mode/mode_error/refusals/hub_start_ms; `handle_key_card_mode` requires `mode` and directs reads to the list; design amended.
- **C-9 — CLOSED**: alias semantics in the `bound` vector logic with a wire test (alice/alice2), and `roster_of` refuses a duplicate name via the `seen` set (unit `named_entries_bind_and_a_name_given_twice_is_refused`).
- **C-10 — CLOSED**: `set_mode` logs before writing, `write_mode` takes `STORE_LOCK`, `key_card_gate` calls `is_gated` before `effective_mode`, and startup reconcile compares the setting (unit `r2_mode_changes_are_logged_first_and_reconciled_at_start` + m21).
- **C-11 — CLOSED**: `hub_start_ms` in the list; sidecar keys state as `{start}:{total}` with base 0 on a new start, and sums only signers ≠ its own fingerprint; test asserts "(1 since".
- **C-12 — CLOSED (A parts)**: `by: "hub.key_card_put (local socket)"`, mode RPC is write-only, cross-card duplicate signing key refused (unit test "one agent"), CLI `hub key-card mode` exercised in the real-hub test.
- **O-4 — CLOSED**: two-project case at the real hub (proj-q card, enforce, Q's key as fx-a → -32044); hub-secret case mapped to the local-socket path as stated.

## R / D

- **C-4 R — SOUND**: the named roster is itself the roll-out switch (empty named map = PH-1 behavior, code-verified), so a receiver-side mode ladder adds nothing; deviation recorded.
- **C-5 D — SOUND**: a lock-free-read/concurrency mutant cannot be made deterministic; the functional read path is covered (digest check, m4/m17) and the limit is honestly recorded.
- **C-12 R (per-agent lock) — SOUND**: puts are rare local-socket admin acts on one small file; a single global lock is correct, simpler, and never nested.
- **O-5 R — SOUND**: same act profile — millisecond fsync on rare runme writes; gated posts read the store lock-free, so there is no post-path latency cost.

## New defects

1. **MINOR** — key_card.rs `startup_reconcile` — a card *deleted* by direct store edit is never flagged: the loop iterates only `store.cards`, then `set_known` re-baselines silently, though the C-2 disposition promises an attention row on re-baseline — put fx-a, stop the hub, remove fx-a's object from the store JSON, restart → no reconcile message despite the admin log showing its put — fix: also report admin-log agents whose last `card_digest` has no card in the store.
2. **MINOR** — key_card.rs mode file — the setting has no runtime integrity guard (unlike the store digest): writing `off` to (or deleting) `key-card-check` while the hub runs disables the gate for every post until the next restart, when reconcile flags it — `echo off > runtime/key-card-check` during enforce, then an impostor posts → stored — fix: baseline the mode file like `KNOWN_DIGEST` and refuse/attention on an unexplained runtime change (residual of C-10's start-up-only scope, not strictly new).
3. **MINOR** — key_card.rs `note_refusal` — once 16 distinct signer fingerprints are recorded for an agent, every further signer (including the agent's own key during a roll-out mismatch) lands in "other", defeating the sidecar's own-key exclusion of C-11 — 16 keys forge as fx-a, then fx-a's own mismatched-key post is refused → counted in "other" → its sidecar reports it as impersonation — fix: let the sidecar query an exact, uncapped count for its own fingerprint, or exempt it from the cap.
4. **MINOR** — pull.rs `observe_impersonation` — on its first pass every sidecar raises a `key-card-written` entry for every pre-existing card (current state, not writes) — a fresh sidecar against a hub holding N cards emits N attention entries — fix: persist the last-seen (agent, seq) per card and raise only on change.
