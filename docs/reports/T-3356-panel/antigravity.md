## 5.1 Header

- **Seat:** `antigravity`
- **Model and version:** Antigravity / Gemini 4 Argon (Medium)
- **Date:** 2026-11-12
- **Checked hashes:** All match — all 10 `sha256sum` file hashes in sections 2.1 (`REQ`, `ARC`, `PC`, `PCa`, `PCb`) and 2.2 (`THR`, `FP`, `EV`, `TRC`, `COV`) match the table values exactly. (Note: the review workspace is a read-only exported directory tree without `.git`, so commit `c9aea3befd7ce44751ea7f9bf954da02a65c6845` could not be checked via `git rev-parse`, while all 10 design artefacts are bit-for-bit identical to the brief.)

---

## 5.2 Verdict

**Yes after the critical and high findings are fixed.**
The overall architecture (`ARC`) and requirements (`REQ`) are well-structured, traceable, and faithful to the operator's rulings `O-1..O-7`, but the step-6 pseudo-code (`PC`, `PCa`, `PCb`) contains concrete algorithm defects on the `PH-2` delivery path (`AL-1..AL-16`) and `PH-2` hardening (`AL-17..AL-40`) that will cause message loss, double hand-over, or origin/doorbell bypasses if built as written. The three things that matter most before starting `B-27`, `B-16`, `B-25`, and `B-20` are:
1. **Fix `AL-1` (`check_origin`), `AL-3` (`meter_tick`), `AL-6` (`ring_or_watchdog_type`), and `AL-7`/`AL-8` (`schedule` / `watchdog_tick`) (`ANTIGRAVITY-2`, `ANTIGRAVITY-4`, `ANTIGRAVITY-5`, `ANTIGRAVITY-6`, `ANTIGRAVITY-8`):** `AL-1` fails to override automated `basis: inferred` / terminal auto-reply prompts (bypassing `O-3` / `SI-20`), `AL-6` writes durable `pending` ring intents *before* the `stale-checks` hold (`FCW-33`) which permanently wedges the session in `UNSUBMITTED` on restart (`FCW-37`), `AL-8` fires the 300 s watchdog during `AL-7`'s 600 s normal backoff, and `AL-4` lets any unanswered outbound message starve all inbound doorbells and the watchdog for up to 1 hour.
2. **Fix `AL-14`/`AL-15` (`forward-once`) and `AL-26`/`AL-27` (`blob` encryption) (`ANTIGRAVITY-1`, `ANTIGRAVITY-3`):** `AL-26` seals blobs to `to`'s key and binds `to` in the AEAD `aad` while `AL-27` opens with `MY_AGENT`, breaking every role-addressed or `AL-14`-forwarded blob (`FCW-114`) after Y has already marked the message `FORWARDED` (final); and `AL-14` step 7's local `FORWARD_ABANDONED` fallback races with an in-flight hub post to cause double hand-over at both Y and X.
3. **Fix receiver ordering in `AL-9`/`AL-23` (`ANTIGRAVITY-10`), `AL-11` (`main_lease_act`) (`ANTIGRAVITY-7`), and `AL-18` (`hub_rpc_guard`) (`ANTIGRAVITY-12`):** `AL-9` hands over out-of-order `STORED` messages without checking `m.seq <= conv.up_to`, `AL-11` force-releases the incumbent's claim before acquiring offset 0 and lets a single `origin = unknown` call flip `OriginReporting` project-wide, and `AL-18` omits `channel.poll`/`read` from `consumer_bind`.

---

## 5.3 Findings

### ANTIGRAVITY-1
- Where: PCa 2.10 AL-26 steps 2–4, PCa 2.11 AL-27 step 4, PC 5.14 AL-14, PC 5.15 AL-15, ARC 6.4.4, ARC 7.2
- Severity: critical
- Class: algorithm
- What is wrong: Blob key wrapping and AEAD authentication (`AL-26` / `AL-27`) fail deterministically (`FCW-114`) whenever a message carrying a blob is forwarded from non-main copy Y to main copy X by `AL-14` (`forward-once`) or addressed to a role name (`to = "main"`), causing permanent message loss after Y marks its copy `FORWARDED`.
- Why: In `PCa 2.10 AL-26` steps 2 and 4 (`PCa` lines 379–384), the sender wraps the ephemeral key `k` to `rpub = encryption key of 'to'` (`wk = crypto_box_seal(rpub, k)`) and encrypts with `aad = utf8("termlink.blob/1\n" + to + "\n" + hex(ps))`. In `PCa 2.11 AL-27` step 4 (`PCa` lines 418–419), the receiver opens `wk` with `crypto_box_seal_open(my_x25519, m.blob.wrapped_key)` and decrypts with `aad(MY_AGENT, m.blob.plain_sha256)`. Two fatal breaks follow:
  1. Under `ARC 7.2.c` and `PC 5.14 AL-14` step 6 (`PC` lines 1084–1089), when non-main copy Y receives a role-addressed message `m` and forwards it to main copy X, Y wraps `m` inside `termlink.forward/1` *"carrying the original `Envelope` bytes unchanged"* (`m` is signed by the original sender, so Y cannot modify `m.blob.wrapped_key`) and marks `m` `FORWARDED` in Y's local store (`AL-14` step 8, final at Y per Invariant 3). When X receives the forward in `AL-15` step 5 (`PC` line 1138: *"run the normal `AL-23` checks on `m`"*), `AL-23` step 9 invokes `AL-27` (`fetch_blob`). If `m.blob.wrapped_key` was sealed to Y's X25519 key, X's `crypto_box_seal_open(my_x25519, ...)` fails with `FCW-114`, so X marks `m` `REFUSED("blob-unwrap")` while Y has already marked `m` `FORWARDED`. The message is never handed over at either Y or X.
  2. Even without `AL-14` forwarding, when the sender sets `m.to` to a role or project address (e.g. `"main"` or `<hub>/<project>/main`) in `AL-26` step 3 (`aad` binds `to`), `AL-27` step 4 reconstructs `aad(MY_AGENT, m.blob.plain_sha256)` using the session's concrete agent identity `MY_AGENT` rather than `m.to`. Because `MY_AGENT != m.to`, `aead_open` fails `FCW-114` on 100% of role-addressed blobs.
- Fix: Change `ARC 6.4.4`, `ARC 7.2`, `PC 5.14 AL-14`, `PC 5.15 AL-15`, and `PCa 2.10–2.11 AL-26/AL-27` so that: (a) `aad` is computed over immutable envelope fields `utf8("termlink.blob/1\n" + m.from + "\n" + m.to + "\n" + m.id + "\n" + hex(ps))` (using `m.to` verbatim in both `AL-26` and `AL-27`); and (b) when `AL-14` forwards a message with a blob from Y to X, Y unwraps `k` using Y's X25519 key and attaches an outer `forward_wrapped_key = crypto_box_seal(X.x25519_pub, k)` field on the `termlink.forward/1` envelope (signed by Y), which `AL-15` passes to `AL-27` in place of `m.blob.wrapped_key`.

### ANTIGRAVITY-2
- Where: PC 5.1 AL-1 (`check_origin`), PC 5.2 AL-2 (`record_pty_input`), ARC 17.5.b, ARC 6.13.1
- Severity: critical
- Class: bypass
- What is wrong: `AL-1` (`check_origin`) fails to override a false `origin: human` adapter report when the adapter reports `basis: inferred` (the required fallback in `ARC 17.5.b`), when the adapter omits `doorbell_ids`, or when a terminal auto-reply (`client_auto`) arrives after `inject` or `notify`, allowing automated PTY injections or doorbell rings to renew or fast-takeover the `main` lease (`O-3` / `SI-20` bypass) and set `conversation = human`.
- Why: In `PC 5.1 AL-1` (`PC` lines 293–305):
  1. Step 2 checks: `if o == human and ring_log.typed_within(sess, TICK) and obj.doorbell_ids intersects ring_log.ids(sess, TICK): return (automated, doorbell)`. Under `ARC 17.5.b` and `ARC 17.3 ZR-7`, populating `doorbell_ids` is only a `SHOULD`, and an adapter that cannot parse doorbell IDs or cannot determine origin from the line reports `origin: human, basis: inferred` with no `doorbell_ids`. When `obj.doorbell_ids` is empty, `obj.doorbell_ids intersects ring_log.ids(sess, TICK)` is false even though the sidecar itself typed a doorbell within `TICK`!
  2. Step 3 checks: `if o == human and b == pty-input: src = pty_last_source_before(sess, rx.mono); if src in {notify, inject}: return (automated, src)`. Because step 3 guards on `b == pty-input`, any adapter that emits `(human, inferred)`—which `ARC 17.5.b` explicitly requires for the Claude Code adapter whenever a prompt is not literally equal to the doorbell string (such as any prompt typed via `termlink pty inject` or a modified prompt)—skips step 3 completely and returns `(human, inferred)`!
  3. Even when an adapter reports `b == pty-input`, `AL-2` (`PC` line 375) stores only `{last_mono | null, bytes, writes}` per source `{client, client_auto, notify, inject}`. When `termlink pty inject` or `command.notify` writes text that triggers a terminal query (or focus/cursor report `ESC[<r>;<c>R` / `ESC[I`) from an attached terminal emulator, `AL-2` step 2 classifies that terminal response as `client_auto` with a timestamp slightly *after* `inject`/`notify` and *before* `rx.mono`. `pty_last_source_before(sess, rx.mono)` then returns `client_auto`, which is not in `{notify, inject}`, so step 3 returns `(human, pty-input)`. In all three cases, automated input is certified as `human`, renewing or stealing the `main` lease in `AL-11` (`O-3` violation) and setting `conversation = human` in `AL-4`.
- Fix: Change `PC 5.1 AL-1` steps 2–3 and `ARC 17.5`: (a) in step 2, if `o == human` and `ring_log.typed_within(sess, TICK)` is true and no `client` PTY input was recorded in that window, return `(automated, doorbell)` even if `obj.doorbell_ids` is empty; (b) in step 3, trigger on `o == human and b in {pty-input, inferred}`, ignore `client_auto` when computing `pty_last_source_before(sess, rx.mono)`, and override to `(automated, src)` whenever `notify` or `inject` has a `last_mono` more recent than `client.last_mono`.

### ANTIGRAVITY-3
- Where: PC 5.14 AL-14 step 7, PC 5.15 AL-15, ARC 7.2, REQ R-51.a, REQ R-67.a1 (4)
- Severity: critical
- Class: algorithm
- What is wrong: When forwarding a role-addressed message from non-main copy Y to main copy X in `AL-14` step 7 experiences an RPC timeout or ack loss at the 1-minute bound, Y writes `FORWARD_ABANDONED` and hands the message over locally at Y while X also receives the `FORWARD` envelope from `inbox:X` and hands it over at X, causing duplicate hand-over of the same message across two sessions.
- Why: `PC 5.14 AL-14` Invariant 3 (`PC` line 1068) states: *"a forwarded message is never handed over at Y (local state `FORWARDED` is final at Y)"*, and `REQ R-67.a1 (4)` / `THR H-24` require 0 messages processed twice. However, `AL-14` step 7 (`PC` lines 1090–1092) specifies: *"post `env` to `inbox:X` on the home hub; if not posted within the R-44.e 'not accepted' bound (1 min): append `FORWARD_ABANDONED m.id` to the local log; `m` is offered at Y again, as before; return `abandoned`."* If Y's RPC to post `env` to `inbox:X` succeeded on the hub (for example, the hub appended `env` to `inbox:X` just before the 1-minute timeout or the TCP connection dropped before Y received the RPC response), `env` is sitting in `inbox:X` while Y writes `FORWARD_ABANDONED` and offers `m` to Y's harness hook (`AL-9`). Because X and Y have separate local `store.db` instances, X's `AL-15` step 6 sees `m.id` is not in X's local store, stores `m` at X, and hands `m` over at X, while Y's `AL-9` hands `m` over at Y. Both X and Y process the message and both append `HANDED_OVER` to the hub record.
- Fix: Change `PC 5.14 AL-14`, `PC 5.15 AL-15`, and `ARC 7.2` so that forwarding and local hand-over arbitrate through an atomic stage transition (CAS) on the home hub's per-message stage record (e.g. `FORWARDED(Y -> X)` vs `ABANDONED_FORWARD(Y)`): X's `AL-15` must refuse a `termlink.forward/1` envelope unless the hub record holds `FORWARDED(Y -> X)`, and Y's `AL-14` step 7 may only transition to `FORWARD_ABANDONED` if a CAS on the hub record proves `FORWARDED(Y -> X)` was not committed (or while the hub is unreachable, X cannot receive `inbox:X` anyway).

### ANTIGRAVITY-4
- Where: PC 5.6 AL-6 steps 7–8, PC 5.8 AL-8 step 5, PC 5.7 AL-7, PC 6.1 FCW-33, FCW-37
- Severity: critical
- Class: algorithm
- What is wrong: In `AL-6` (`ring_or_watchdog_type`), step 7 durably appends a `pending` ring intent to the ring log (and for `via = watchdog`, to the admin chain) *before* step 8 checks `if mono() - t0 > TICK: return held("stale-checks")`. When step 8 holds the ring, no keystroke is typed, but the unclosed `pending` intent remains on disk, permanently wedging the session in `UNSUBMITTED` on sidecar restart (`FCW-37`) and corrupting the watchdog admin chain in `AL-8`.
- Why: In `PC 5.6 AL-6` (`PC` lines 579–588):
  - Step 7 executes: `durable append of rec (ring log; for via = watchdog the admin chain)` where `rec` has `outcome = pending`.
  - Step 8 then executes: `if mono() - t0 > TICK: return held("stale-checks") # FCW-33`.
  - Step 9 calls `command.notify(...)` and step 10 appends the closing `outcome` record (`typed` or `aborted(...)`).
  When step 8 returns `held("stale-checks")`, step 7 has already written the `pending` intent to disk, and step 10 never runs to close it. Two concrete failures result:
  1. Under `PC 6.1 FCW-37` (`PC` line 1255: *"Intent record in ring log with no outcome on restart -> Treat as `unknown` (typed, unsubmitted); hold normal rings"*), when the sidecar restarts or reconciles the ring log, the unclosed `pending` record from a held check is treated as an unsubmitted typed doorbell, locking the session in `UNSUBMITTED` (`AL-7`) and blocking all normal doorbell rings even though nothing was ever typed into the PTY.
  2. When `via == watchdog`, step 7 appends a row to the hash-chained admin log (`admin.jsonl`), and then `AL-8` step 5 (`PC` line 717: `if res is held(check): ... admin_row(row with outcome = held:<check>)`) appends a *second* row for the same watchdog attempt, leaving a dangling `pending` row in the hash-chained audit trail.
- Fix: In `PC 5.6 AL-6`, move step 8 (`if mono() - t0 > TICK: return held("stale-checks")`) *before* step 7 (`durable append of rec`), so that the durable `pending` intent is only written after all pre-type checks (including `FCW-33`) have passed.

### ANTIGRAVITY-5
- Where: PC 5.7 AL-7, PC 5.8 AL-8, PC 5.6 AL-6 step 11, ARC 6.9.4, ARC 6.9.7
- Severity: high
- Class: algorithm
- What is wrong: The 300-second watchdog (`AL-8`) fires without checking whether the normal doorbell is intentionally holding in `BACKOFF` (`AL-7` where backoff between ring 2 and ring 3 is 600 s), and `AL-7`'s state table has no transition for `watchdog typed` when `ring_state` is `BACKOFF` or `IDLE`, defeating the 10-minute backoff schedule (`O-2 D+` / `ARC 6.9.4`) and raising false `watchdog_fired_ready_busy` attention items.
- Why: In `PC 5.7 AL-7` (`PC` lines 632–640), the cumulative ring offsets are `O(0)=0`, `O(1)=4` (120 s), `O(2)=12` (360 s), `O(3)=32` (960 s): the gap between ring 2 (`k=2`) and ring 3 (`k=3`) is 20 ticks = **600 s** (10 minutes, `ARC 6.9.4` / `V-5`). Meanwhile, `PC 5.8 AL-8` (`watchdog_tick`, `PC` lines 698–707) fires whenever `sess.Flag == UP`, `s1 == KNOWN`, `s2_known`, `conversation == none`, and `mono() - since >= 300 s`, without inspecting `sess.ring_state` or whether `AL-5` is holding solely for `BACKOFF`:
  1. Suppose a session is idle (`sess.ready == READY`, `conversation == none`) with unread mail (`Flag == UP`) that was not cleared after ring 2 at tick 12 (`360 s`). `AL-7` intentionally holds normal rings in `BACKOFF` for 600 s until tick 32 (`960 s`). Halfway through that backoff (at `300 s` of silence, tick 22), `AL-8` triggers, types the watchdog doorbell into the `READY` session (breaking the 600 s backoff), and in `AL-8` step 6 (`PC` line 720) sees `sess.ready == READY` and raises `Attention(watchdog_fired_ready_busy, "watchdog typed into READY session; READY appears stale")`, falsely blaming the adapter!
  2. Furthermore, when `AL-8` calls `AL-6(..., via = watchdog)`, `AL-6` step 11 invokes `schedule.on_typed(sess, ring_id, ids, via)` (`PC` line 593). In `AL-7`'s state transition table (`PC` lines 653–662), `watchdog typed (AL-8)` is defined *only* for state `UNSUBMITTED` (`UNSUBMITTED -> AWAIT_SUBMIT, k unchanged`). When `ring_state` is `BACKOFF` (or `IDLE` when `READY` has been false/stale for 300 s since a message arrived), `on_typed(..., via = watchdog)` either matches the generic `BACKOFF | typed | -> AWAIT_SUBMIT | k = k + 1` row (corrupting the normal backoff counter `k`) or has no valid transition at all.
- Fix: Change `PC 5.8 AL-8`, `PC 5.7 AL-7`, and `ARC 6.9.7` so that: (a) `AL-8` does not fire when `sess.ready == READY` and `AL-7`'s `may_ring` is returning `(false, "backoff")` or `(false, "cap-4")` (the normal doorbell is working and intentionally in backoff/cap); and (b) `AL-7` explicitly defines `on_typed(..., via = watchdog)` from `IDLE` and `BACKOFF` to transition to `AWAIT_SUBMIT` with `k` unchanged (preserving `T0 = mono()` if `T0 == none`).

### ANTIGRAVITY-6
- Where: PC 5.3 AL-3 (`meter_tick`) steps 1–4, PC 5.8 AL-8 step 2, PC 6.1 FCW-13, FCW-16, ARC 6.14.1–6.14.3
- Severity: high
- Class: algorithm
- What is wrong: `AL-3` (`meter_tick`) sets `sess.s1 = KNOWN(at = r.sources.client.last_mono)` even when `last_mono` is `null` (before the first human keystroke), and sets `sess.s2_known = true` whenever `sess.transcript != none` even when `lstat(sess.transcript)` fails, causing `null` timestamp arithmetic in `AL-5`/`AL-8` and allowing the watchdog to fire blind when the transcript cannot be stat'd (violating `AFC-10` / `FCW-13`).
- Why: In `PC 5.3 AL-3` (`PC` lines 400–411):
  1. In `AL-2` (`PC` line 375), `sources.client.last_mono` is initialized to `null` until a human client writes to the PTY. In `AL-3` step 1 (`PC` line 405), whenever `session.input_activity` returns `Ok(r)`, `AL-3` sets `sess.s1 = KNOWN(at = r.sources.client.last_mono)`. For a newly started session where no human has typed yet, `sess.s1` is `KNOWN(at = null)`. Then `AL-5` step 5 evaluates `mono() - sess.s1.at < 30 s` (`PC` line 534) and `AL-3` step 4 evaluates `sess.last_activity = max(at of known)` (`PC` line 411) over `null`, which is undefined or type-errors.
  2. In `AL-3` step 3 (`PC` lines 406–408), `AL-3` runs:
     ```
     if sess.transcript != none:
         st = lstat(sess.transcript)
         if st is ok and (st.mtime_ns, st.size) != sess.s2_stat:
             sess.s2_stat = (st.mtime_ns, st.size); sess.s2_at = mono()
     sess.s2_known = (sess has had >= 1 valid report since boot) or (sess.transcript != none)
     ```
     Notice line 408: `sess.s2_known` is set to `true` whenever `sess.transcript != none`, **even if `lstat(sess.transcript)` returned an error** (e.g., `ENOENT` or `EACCES`) and no valid adapter report has ever been received! Under `ARC 6.14.3` (`AFC-10`) and `PC 6.1 FCW-13` / `FCW-16`, when `lstat` fails and no adapter report is valid, `S2` must be `unknown` and the watchdog must be held. Instead, `AL-3` sets `sess.s2_known = true` and `AL-8` fires the watchdog blind.
- Fix: Change `PC 5.3 AL-3` so that: (a) when `r.sources.client.last_mono == null`, `sess.s1` is `KNOWN(at = -infinity)` (or `0`, so `mono() - sess.s1.at >= 30 s` is well-defined and `last_activity` falls back to `sess.boot_mono`); and (b) `sess.s2_known` is `true` only if `sess` has received a valid adapter report in the current state OR `sess.transcript != none and st is ok` on the current `meter_tick` (setting `sess.s2_known = false` whenever `lstat` fails and adapter state is stale/unreported, per `FCW-16`).

### ANTIGRAVITY-7
- Where: PC 5.11 AL-11 (`main_lease_act`) step 3 and `FastTakeover` steps 2–4, ARC 6.13.1.a, ARC 6.13.3
- Severity: high
- Class: algorithm
- What is wrong: `AL-11` (`main_lease_act`) has two defects: (1) in `FastTakeover`, S2 calls `force_release(other.claim_id)` *before* acquiring offset 0 and posting `Acquire`, so if S2 fails or crashes at step 3 or 4, S1's live claim is destroyed while `main:<p>` still names S1 as holder; and (2) any caller passing an `origin` field (even `origin = unknown` or `automated` on an ignored claim) permanently flips `OriginReporting` on `main:<p>`, locking out all running sessions of that project that still use legacy hooks without `--origin` (violating `NID` / `ARC 6.13.1.a`).
- Why: Inspect `PC 5.11 AL-11` (`PC` lines 885–928):
  1. In `FastTakeover` (`PC` lines 925–928), step 2 executes `force_release(other.claim_id)` on `coord:claims`, step 3 executes `claim offset 0 (CAS); Err -> return unknown`, and step 4 executes `post Acquire (reason = fast-takeover, ...); Err -> release the claim, return unknown`. If S2 loses a CAS race at step 3, experiences a transient error at step 4, or crashes between steps 2 and 4, S1's coordination claim on offset 0 has already been deleted, while `main:<p>` still folds to S1 at generation `g`. That violates the lease invariant (`ARC 6.13.2`) that the current holder holds offset 0.
  2. In `AL-11` step 3 (`PC` lines 885–889): `if origin is present and not rep: post OriginReporting; rep = true`. This executes unconditionally *before* step 5 checks whether the caller is allowed to acquire, renew, or promote! Furthermore, `AL-11` line 871 notes: *"legacy callers omit `origin`; a wrapper maps omitted `--origin` to `unknown` once `OriginReporting` is on."* If a project is running on legacy hooks (where `rep == false` and legacy renewals are supposed to keep working under `ARC 6.13.1.a` / `AFC-17`), and a single test CLI invocation or automated sidecar call passes `origin = automated` or `origin = unknown`, step 3 posts `OriginReporting` to `main:<p>` even though step 5 immediately returns `ignored("non-human-origin")`! Once `OriginReporting` is on `main:<p>`, it is irreversible (`AL-10` line 816), so all existing legacy sessions of that project immediately stop renewing their main lease.
- Fix: Change `PC 5.11 AL-11` and `ARC 6.13.1.a`: (a) only post `OriginReporting` in `AL-11` when a caller with `origin == human` actually succeeds in posting an `Acquire`, `Promote`, or `Touch` transition (never before step 5 or on `unknown`/`automated`); and (b) in `FastTakeover`, replace `force_release` + `claim offset 0` with an atomic hub-side CAS claim-replacement or post the new `Acquire` record before releasing S1's coordination claim so `main:<p>` (which is authoritative in `AL-10`) and `coord:claims` cannot diverge on partial failure.

### ANTIGRAVITY-8
- Where: PC 5.4 AL-4 (`conversation_update`), PC 5.5 AL-5 step 6, PC 5.8 AL-8 step 1, PCa 2.7 AL-23, ARC 6.14.4, REQ R-55
- Severity: high
- Class: bypass
- What is wrong: Sending any message that expects a reply (`sent(m)`) keeps `sess.owed_to_us` non-empty until `m` is answered or becomes `STUCK`/`DEAD` (1 hour for a normal message under `R-44.e`), which sets `sess.conversation = agent` (`AL-4` step 3) even after the sender's own turn has finished (`turn_open == false`, `ready == READY`), completely blocking both the normal doorbell (`AL-5`) and the 300-second watchdog (`AL-8`) for all inbound mail (including the awaited reply itself!) for up to an hour.
- Why: Trace `PC 5.4 AL-4` (`PC` lines 476–491) alongside `AL-5` (`PC` line 535), `AL-8` (`PC` line 701), and `PCa 2.7 AL-23`:
  1. When session A sends a normal request `m1` to session B and finishes its turn to wait for B's answer (`REQ R-55` yield-and-wake, or normal asynchronous request/reply), `AL-4` step 2 adds `m1.id` to `A.owed_to_us`.
  2. Even though A's turn has ended (`A.turn_open == false` and `A.ready == READY`), `AL-4` step 3 (`PC` line 490) sets `A.conversation = agent` because `A.owed_to_us` is not empty.
  3. Both `AL-5` step 6 (`if sess.conversation != none: return held("conversation-active")`) and `AL-8` step 1 (`sess.conversation == none`) refuse to type any doorbell or watchdog line into A whenever `A.conversation != none`.
  4. Now B sends its reply `m2` (with `in_reply_to = m1.id`), or another session C sends an urgent message `m3` to A. When `m2` or `m3` arrives at A's sidecar in `AL-23` (`PCa` lines 295–326), `AL-23` stores the message as `STORED` and raises `A.Flag = UP`, **but `AL-23` never calls `AL-4` (`reply_or_no_action(m.in_reply_to)`) at `STORED`**—and for C's urgent message `m3`, `m3` is not a reply to `m1` at all! Because A is idle at its prompt, A's `UserPromptSubmit` hook (`AL-9`) will not fire until the doorbell wakes A; yet the doorbell (`AL-5`) and the watchdog (`AL-8`) both refuse to ring because `A.conversation == agent`! A is deadlocked for 1 hour (until `m1` becomes `STUCK` under `R-44.e`, or indefinitely if B was offline in `WAITING FOR RECIPIENT`).
- Fix: Change `PC 5.4 AL-4`, `PCa 2.7 AL-23`, and `ARC 6.14.4` so that: (a) `AL-23` invokes `AL-4.reply_or_no_action(m.in_reply_to)` as soon as a valid reply is accepted at `STORED` (so the reply itself clears `owed_to_us` before `AL-5` runs); and (b) when `sess.turn_open == false` and `sess.ready == READY` (the session is idle at its prompt waiting), `owed_to_us` does not block `AL-5` / `AL-8` from ringing for replies in the awaited conversation or for `urgent` (`priority >= 5`) messages from other conversations (or bound the `owed_to_us` doorbell hold while `turn_open == false` to `V-6 = 300 s` so the watchdog can still wake an idle waiting session).

### ANTIGRAVITY-9
- Where: ARC 6.10.1–6.10.2, PCa 2.15 AL-31, PCa 2.16 AL-32, REQ R-47.a (3), REQ R-79.a, FP 6.2.5.a (6), FP 11.3.5 (RA-5), Brief 2.3.d (O-4 A) and 2.4 (NID)
- Severity: high
- Class: ruling
- What is wrong: `AL-31` (`log_vendor_bypass`) and `AL-32` (`deny_sendmessage`) only match `VENDOR_TOOLS = {"SendMessage"}`, omitting vendor in-process sub-agent tools (`Task` / `Agent`) required by `REQ R-47.a (3)` and `R-79.a (2)–(3)`, and `ARC 6.10.2` omits the `R-79.a (5)` / `RA-5` requirement that lightweight TermLink sub-agent dispatch (`T-3386`) must be proven before disabling vendor sub-agents.
- Why: Two gaps exist between `REQ`/`FP` and `ARC`/`PCa`:
  1. `REQ R-47.a (3)` and `R-79.a (2)` require the adapter to log and surface in Watchtower needs-attention both vendor peer channels (`SendMessage`) *and* vendor in-process sub-agents (`Task` / `Agent` tool in Claude Code) from `PH-2` (`FX-1`, `R-79.e`). In `PCa 2.15 AL-31` (`PCa` lines 545, 552), `VENDOR_TOOLS` is hardcoded to `{"SendMessage"}`, so vendor in-process sub-agent calls are never logged to `vendor-bypass.jsonl` and never raise a needs-attention item in `PH-2`.
  2. Under `REQ R-79.a (3), (5)`, `FP 6.2.5.a (6)`, `FP 11.3.5 (RA-5)`, and the project's standing No-Premature-Disablement (`NID`) rule (Brief 2.4), vendor peer channels (`SendMessage`) are disabled after `B-26` (`session-message-selftest.sh` re-based on TermLink per `O-4 A`) is proven and approved by the operator, while vendor in-process sub-agents are disabled only after *both* `B-26` and lightweight TermLink sub-agent dispatch (`T-3386`, `PH-4`) are proven in the field and approved by the operator. `ARC 6.10.2` and `AL-32` collapse these into `SendMessage` only.
- Fix: Update `ARC 6.10.1–6.10.2`, `PCa 2.15 AL-31`, and `PCa 2.16 AL-32` to: (a) include Claude Code's sub-agent tool names (`"Task"`, `"Agent"`) in `AL-31`'s logged tool set (`VENDOR_BYPASS_TOOLS`) so in-process sub-agent invocations are logged and surfaced in Watchtower during `PH-2` per `R-79.a (2)` and `FX-1`; and (b) distinguish `VENDOR_PEER_TOOLS = {"SendMessage"}` (deny-gated on `B-26` + operator approval per `O-4 A`) from `VENDOR_SUBAGENT_TOOLS = {"Task", "Agent"}` (deny-gated on `T-3386` + `B-26` + operator approval in `PH-4` per `R-79.a (5)` and `RA-5`).

### ANTIGRAVITY-10
- Where: PCa 2.7 AL-23 steps 12–16, PCa 2.9 AL-25, PC 5.9 AL-9 step 2, ARC 6.3, REQ R-18, REQ R-46.e, REQ R-59
- Severity: high
- Class: algorithm
- What is wrong: Out-of-order messages in a conversation (`m.seq > conv.up_to + 1`) are stored with `stage = STORED` in `AL-23` step 12 and immediately handed over to the agent by `AL-9` step 2 (`hook_fetch`) because `AL-9` never checks `m.seq <= conv.up_to`; additionally, `AL-23` step 16 clears `gap_since = none` whenever `up_to` advances by even one sequence number, even when a second gap still exists in the reorder buffer.
- Why: Compare `PCa 2.7 AL-23` (`PCa` lines 318–325), `PCa 2.9 AL-25` (`PCa` lines 351–361), and `PC 5.9 AL-9` (`PC` lines 749–756):
  1. When message `seq = 1` and message `seq = 3` arrive (with `seq = 2` delayed in transit), `AL-23` step 12 inserts `seq = 3` into `store.db` with `stage = STORED`, step 14 records `gap_since = mono()` because `3 > up_to + 1`, and step 15 raises `Flag = UP`. Then on the next prompt/turn, `AL-9` step 2 selects:
     `cand = STORED messages for sess with no open attempt, ordered by priority DESC, stored_at ASC`.
     Because `AL-9` has no filter requiring `m.seq <= conv.up_to`, `AL-9` immediately hands `seq = 3` over to the agent (`HANDED_OVER`) before `seq = 2` has arrived! When `seq = 2` arrives 1 second later, `seq = 2` is handed over *after* `seq = 3`, violating `REQ R-59` (per-conversation sequence ordering), `REQ R-46.e` ("same message ids, stages and order"), and `ARC 6.3` ("A receiver holds out-of-order messages in a bounded reorder buffer").
  2. Suppose `seq = 1` is delivered (`up_to = 1`), and then `seq = 3` and `seq = 5` arrive (`seq = 2` and `seq = 4` are missing). `AL-23` step 14 sets `gap_since = mono()` for the first gap at `up_to + 1 = 2`. Next, `seq = 2` arrives. `AL-23` step 16 executes:
     `advance up_to to the highest contiguous stored seq; if up_to advanced: gap_since = none`.
     Here `up_to` advances from `1` to `3`, so `gap_since` is set to `none`—even though `seq = 5` is still sitting in the buffer behind a missing `seq = 4`! Because `gap_since` is now `none`, `AL-25` (`gap_check`, `PCa` line 354: `for each conv with gap_since != none`) never checks or escalates the remaining gap at `seq = 4` (`FCW-111` is bypassed).
- Fix: Change `PC 5.9 AL-9` step 2 and `PCa 2.7 AL-23` / `2.9 AL-25`: (a) in `AL-9` step 2, only select `STORED` messages where `m.seq <= conv.up_to` (unless `m` is an urgent message that `ARC 6.3` explicitly exempts from holding behind normal messages, or the conversation gap has been escalated/released); and (b) in `AL-23` step 16, when `up_to` advances, if there is still any stored message in `conv` with `seq > up_to + 1`, set `gap_since = mono()` (the time the new leading gap at `up_to + 1` became the head gap) instead of `none`.

### ANTIGRAVITY-11
- Where: PCb 2.11 AL-51 (`circuit_turn`), PCb 2.12 AL-52 (`reconcile_outage_log`), ARC 6.5, FP 10.5.4 (RC-4), FP 10.5.7 (RC-7), FP 10.5.8 (RC-8)
- Severity: high
- Class: algorithm
- What is wrong: Outage-log reconciliation (`AL-51` and `AL-52` in `PCb`) omits three binding rules from `FP 10.5` (`OP-9 A`): `RC-4` (fetching the hub's bounded encrypted copy on `lost-after-STORED` and accepting verified foreign `DEAD`/membership records into the local log), `RC-7` (flagging replayed stages whose timestamp is earlier than the hub's newest stage for that message), and `RC-8` (enforcing the local outage-log ceiling with visible refusal while unreconciled).
- Why: The operator ruled `OP-9 A` (`FP 13.2.9`), making `RC-1..RC-9` (`FP 10.5.1–10.5.9`) binding on `ARC 6.5` and `PCb`:
  1. `FP 10.5.4 (RC-4)` requires that when a record is in the hub record and not in the local log: (i) if it is `STORED` or later for this receiver, flag `lost-after-STORED` *and* request the bounded encrypted copy from the hub (`F-5`, `R-80`); (ii) if another party wrote it (a signed, sequenced `DEAD` statement or a membership event), check its signature and sequence and accept it into the local log. In `PCb 2.12 AL-52` step 3 (`PCb` lines 417–418), the code only does `for each record r in hub_records not in local_log: if r.stage >= STORED and r.to == MY_AGENT: flag lost_after_stored(r.msg_id)`. It never calls `AL-54` (`recover_from_hub_copy`) and never verifies/accepts foreign signed `DEAD` or membership records into `local_log`.
  2. `FP 10.5.7 (RC-7)` requires: *"A replayed stage is never written to the hub with a time earlier than the newest stage the hub already holds for that message without a flag."* `AL-52` step 2 (`PCb` lines 415–416) replays `r` to the hub with `reconciled = true` without comparing `r.ts` against the hub's newest stage timestamp for `r.msg_id`.
  3. `FP 10.5.8 (RC-8)` requires: *"When the local log reaches its ceiling during a long outage, the receiver refuses new STORED visibly (the F-3 behaviour). The log is never trimmed while it is unreconciled."* Neither `AL-51` (`PCb` lines 387–399) nor `AL-23` checks the unreconciled outage-log ceiling (`FCW` condition missing).
- Fix: Update `PCb 2.11 AL-51`, `PCb 2.12 AL-52`, and `PCb 3.1`: (a) in `AL-52` step 3, invoke `AL-54` to recover the bounded encrypted copy on `lost_after_stored` and verify + append foreign signed `DEAD`/membership records; (b) in `AL-52` step 2, compare `r.ts` against the hub's newest stage for `r.msg_id` and attach a `reconciled_older_ts` flag if `r.ts < hub_newest.ts` (`RC-7`); and (c) enforce the local outage-log ceiling in `AL-51`/`AL-23` while `circuit != reconciled`, refusing new `STORED` with a dedicated `FCW` code when full (`RC-8`).

### ANTIGRAVITY-12
- Where: PCa 2.2 AL-18 (`hub_rpc_guard`) step 3, ARC 6.6.1, ARC 6.6.2, REQ R-49, THR SI-16
- Severity: high
- Class: bypass
- What is wrong: `AL-18` (`hub_rpc_guard`) enforces `consumer_bind` and tracks `inbox_consumers[req.topic]` only when `op in {channel.subscribe, channel.ack}`, omitting `channel.poll` / `channel.read` / `event.poll` (the pull operations by which sidecars or legacy scripts read inbox messages), allowing a second consumer to read an inbox topic without triggering `SI-16` / `FCW-85`.
- Why: In `PCa 2.2 AL-18` step 3 (`PCa` lines 88–94):
  ```
  if op in {channel.subscribe, channel.ack} and req.topic starts with "inbox:":
      if client.agent_id != inbox_owner(req.topic):
          return Deny("FCW-84: inbox-owner-mismatch")
      if client.session_id not in inbox_consumers[req.topic]:
          if len(inbox_consumers[req.topic]) >= 1:
              ... return Deny("FCW-85: second-inbox-consumer")
  ```
  As documented in `REQ 2.4a.2` and `REQ 2.4a.5`, the 2026-10-06 three-consumer incident (`notify-sidecar`, `inbox-sidecar`, `journal-mirror.sh`) occurred because processes polled/read the channel using their own local offset state without calling `channel.ack` or `channel.subscribe`. If any process (or a second sidecar instance) reads `inbox:<hub_id>/<project>` via a pull RPC (`channel.poll` / `channel.read` / `event.poll`), `op` is not in `{channel.subscribe, channel.ack}`, so `AL-18` step 3 skips both the `FCW-84` owner check and the `FCW-85` single-consumer check!
- Fix: Change `PCa 2.2 AL-18` step 3 and `ARC 6.6.1–6.6.2` to include all inbox topic read/poll RPCs (`channel.poll`, `channel.read`, `event.poll` on `inbox:*`) in the `FCW-84` and `FCW-85` guard (while keeping sender stage-record queries on the separate per-message stage RPC of `T-3347`).

### ANTIGRAVITY-13
- Where: PC 5.12 AL-12 (`waiting_mail_tick`), PC 5.13 AL-13 (`oob_send_once`), ARC 6.13.5, REQ R-67.a
- Severity: medium
- Class: algorithm
- What is wrong: `AL-12` (`waiting_mail_tick`) suppresses all subsequent `mail_waiting_main_not_reading` notifications after the first notice until the project's unread queue drains completely to zero (`oldest_unread == none`), so a newly arriving **urgent** message (which has a 60 s threshold) or a new main lease generation (`g + 1`) never triggers an out-of-band notice if an older normal message is already sitting unread.
- Why: In `PC 5.12 AL-12` (`PC` lines 989–1005):
  - Step 1 resets `notified_episode = none` *only* when `oldest_unread == none`.
  - Step 6 sends the out-of-band notice *only* when `notified_episode == none`, setting `notified_episode = oldest_unread.id`.
  Suppose a low-priority normal message `m1` arrives while the operator is away, waits `180 s`, and triggers a single normal notice (`notified_episode = m1.id`). Thirty minutes later, while `m1` is still unread, a critical `urgent` (`priority >= 5`) message `m2` arrives (or another session takes over the `main` lease at generation `g + 1` and also stops reading). Because `oldest_unread` never became `none`, `notified_episode` remains `m1.id != none`, so step 6 never fires for the urgent message `m2` or for the new lease generation `g + 1`.
- Fix: In `PC 5.12 AL-12` and `ARC 6.13.5`, track `notified_episode` as a tuple `(lease.generation, highest_notified_urgency)` so that `AL-12` emits a new notice (under a distinct `episode_id` in `AL-13`) whenever (a) an `urgent` message crosses `T_wait = 60 s` during an episode that was previously notified only at `normal` urgency, or (b) the main lease changes holder/generation (`lease.generation` increments) and the new holder also fails to read within `T_wait`.

### ANTIGRAVITY-14
- Where: PCa 2.4 AL-20, PCa 2.5 AL-21 (`revocation_pull`) steps 2 and 5, ARC 6.2.3, ARC 6.2.4, THR SI-11
- Severity: medium
- Class: threat
- What is wrong: In `AL-21` step 2 (`PCa` line 197), when `hub.revocation.head().seq == revocation_seq_seen`, `AL-21` skips `hub.revocation.get()` (and therefore skips Ed25519 signature verification in step 4) and jumps directly to step 5 (`reconciled = true; last_revocation_ok_mono = mono()`), allowing an unsigned `revocation.head` response to mark a restarted sidecar `reconciled = true` and refresh `PN-13` indefinitely.
- Why: Under `THR SI-11` and `ARC 6.2.4`, a started or restored sidecar is `unreconciled` until its first **verified** hub contact, and after `PN-13` (5 min) without a verified pull it must refuse new `STORED` messages from senders whose grants are older than `PN-13` (`FCW-96`, `FCW-97`). However, in `PCa 2.5 AL-21` (`PCa` lines 196–205):
  - Step 2 calls `h = hub.revocation.head()`. If `h.seq == revocation_seq_seen`, it jumps to step 5.
  - Step 4 (`ed25519_verify(hub.signing.pub, ...)` on `list.sig`) only runs when `h.seq != revocation_seq_seen`!
  - In `AL-20` (`PCa` lines 155–169), only the full revocation list has `sig`; `revocation.head()` is not specified as signed or timestamped.
  Therefore, when a sidecar restarts with persisted `revocation_seq_seen = N` (or on every 30 s tick while `seq` is unchanged), an unauthenticated or replayed `revocation.head() -> {seq: N}` causes `AL-21` to set `reconciled = true` and `last_revocation_ok_mono = mono()` without verifying any signature from `hub.signing.pub`.
- Fix: Change `PCa 2.4 AL-20` and `PCa 2.5 AL-21` so that either (a) `revocation.head()` returns a signed statement `(seq, issued_at_ms, sig)` verified against `hub.signing.pub` (with `issued_at_ms` within `PN-13`), or (b) `AL-21` always calls `hub.revocation.get()` and verifies `list.sig` on startup (`reconciled == false`) and at least once every `PN-13 / 2`.

### ANTIGRAVITY-15
- Where: PC 5.5 AL-5 step 2, PC 5.6 AL-6 step 3, PCa 2.13 AL-29, EV 3.1 E-2, ARC 6.9.1, ARC 17.4
- Severity: medium
- Class: portability
- What is wrong: The foreground-process check (`fg_comm in HARNESS_NAMES`, `FCW-22` / `FCW-27`) fails closed on 100% of ticks when a harness runs under an interpreter (such as `node` for Claude Code CLI wrappers on Linux, where `/proc/<pid>/comm` is `"node"` rather than `"claude"`) and has no mechanism specified when `/proc` is absent on macOS (`R-8.g`, `T-14`).
- Why: In `PC 5.5 AL-5` step 2 (`PC` line 531) and `PC 5.6 AL-6` step 3 (`PC` line 570), the sidecar reads `tcgetpgrp(pty_master_fd)` and checks `fg_comm in HARNESS_NAMES` (`FCW-22`, `FCW-27`), where `EV 3.1 E-2` (`EV` line 198) defines `HARNESS_NAMES = {"claude", "opencode"}`. Two concrete portability problems arise:
  1. On Linux, when Claude Code or another harness is installed as an npm/Node.js script (`#!/usr/bin/env node`), `/proc/<fg_pgrp>/comm` is `"node"`, not `"claude"`. Checking `fg_comm in {"claude", "opencode"}` fails closed with `FCW-22`/`FCW-27` on every tick, so the doorbell never rings. Conversely, if `"node"` were added to `HARNESS_NAMES`, then any `node` subprocess spawned by the user inside a shell escape (`!node ...`) would falsely pass `FCW-27`!
  2. On macOS (`REQ R-8.g`, `THR T-14`), `/proc` does not exist. While `PCa 2.13 AL-29` step 1 (`PCa` line 488) explicitly handles missing `/proc` by returning `UNMEASURABLE(reason = "no-proc")`, `AL-5` step 2 and `AL-6` step 3 do not specify how `fg_comm` is obtained without `/proc` or how `FCW-22`/`FCW-27` behave on macOS.
- Fix: Change `PC 5.5 AL-5` step 2, `PC 5.6 AL-6` step 3, and `ARC 6.9.1` so that the sidecar records the harness process PID and process group `sess.harness_pgid` at `session_start` / launcher spawn and checks `tcgetpgrp(pty_master_fd) == sess.harness_pgid` (which is exact even when the harness runs under `node` or `python`, and immediately detects when a shell escape or editor like `vim` puts a child process group in the PTY foreground on both Linux and macOS via POSIX `tcgetpgrp`).

### ANTIGRAVITY-16
- Where: EV 14.1–14.3, COV 1.3, PC 6.1–6.2, PC 11.4, PCa 3.1, PCb 3.1
- Severity: medium
- Class: evidence
- What is wrong: `EV` (v0.1) and `COV` (v0.1) were not updated after step 6 (`PC`, `PCa`, `PCb`) defined `FCW-1..FCW-169` and answered the eight open parameters in `EV 14.3`, leaving `EV` section 14 marked *"Status: open. The pseudo-code (step 6, T-3355) does not exist"* and leaving step-6-only `FCW` conditions without `E-n` test cases or a `COV` table 2.9 (`FCW-n -> E-n`).
- Why: `EV 14.1–14.3` (`EV` lines 1336–1352) and `COV 1.3` (`COV` line 24) state that step 5 (`EV`/`COV`) must iterate with step 6 (`PC`/`PCa`/`PCb`) before step 7 to:
  1. Add `COV` table 2.9 mapping every `FCW-1..FCW-169` condition to an `E-n` evidence spec and negative control.
  2. Fold the concrete algorithm constants from `PC 11.4` (`PC` lines 1630–1647) into the `ASSERT` clauses of `E-5`, `E-23`, `E-24`, `E-32`, `E-39`, and `E-101`.
  3. Add test cases for the new fail-closed conditions added in step 6 (`PC 6.2` lines 1296–1306, `PCa 3.1`, `PCb 3.1`), such as `FCW-33` (`mono() - t0 > TICK` stale-checks hold in `AL-6`), `FCW-45` (clock backward jump in `AL-9`), `FCW-53` (`OriginReporting` active + omitted origin in `AL-11`), `FCW-68..FCW-71` (`FORWARD` envelope verification in `AL-14`/`AL-15`), and `FCW-78` (re-check failure after retry in `AL-16`).
- Fix: Update `EV` (to v0.2) and `COV` (to v0.2) before build planning (step 8) to incorporate the `PC 11.4` answers into `E-5`, `E-23`, `E-24`, `E-32`, `E-39`, `E-101`, add test cases and negative controls for the step-6-only `FCW` conditions (`FCW-33`, `FCW-45`, `FCW-53`, `FCW-68..71`, `FCW-78`), and populate `COV` table 2.9 (`FCW-1..FCW-169 -> E-n`).

### ANTIGRAVITY-17
- Where: PCb 2.2 AL-42 (`tier0_decide`) step 4, PCb 2.3 AL-43 (`evaluate_policy`), ARC 12.2, FP 10.3.4.c.2, FP 10.3.7.g, THR SI-13
- Severity: medium
- Class: threat
- What is wrong: In `AL-42` (`tier0_decide`), step 4 checks `ev.route_allowed("policy")` against the static event registry but never calls `AL-43` (`evaluate_policy`) to check whether an operator policy rule at the fleet, hub, or project level has set `effect: deny` on that event verb, violating `FP 10.3.4.c.2` and `10.3.7.g`.
- Why: `FP 10.3.4.c.2` and `FP 10.3.7.g` state normatively: *"A rule with an event verb may only narrow the built-in route. `effect: deny` switches the policy route off for the matching subject and `where`, so the event goes to a human route."* However, in `PCb 2.2 AL-42` step 4 (`PCb` lines 125–128):
  ```
  if d.route == "policy":
      if not ev.route_allowed("policy") or not policy_preconditions_hold(ev):
          return Err("human-route-only")
  ```
  `AL-42` never invokes `AL-43` (`evaluate_policy(ev.actor, ev.verb, self, ev.place)`)! Consequently, if the operator signs a policy rule setting `effect: deny` for `rotate_hub_key`, `replace_agent_key`, `renew_grant`, or `hand_over` to force human approval on a hub or project, `AL-42` ignores the `deny` rule and accepts the `policy` route anyway.
- Fix: In `PCb 2.2 AL-42` step 4, call `res = evaluate_policy(ev.actor, ev.verb, self, ev.place)` (`AL-43`) whenever `d.route == "policy"`; if `res.effect == "deny"`, reject `d.route == "policy"` with `Err("policy-route-denied-by-rule")` so the event must take a `human` route (`watchtower` or `oob`).

### ANTIGRAVITY-18
- Where: PC 5.10 AL-10 (`fold`), PC 5.11 AL-11 (`main_lease_act` / `FastTakeover`), ARC 6.13.3
- Severity: low
- Class: other
- What is wrong: In `AL-11` (`PC` lines 897–900, 918), `last_human(h, g)` can evaluate to `none` (or `-infinity`) when holder `h` acquired the lease with `reason = resume` or before `OriginReporting` was enabled and has not yet posted a `Touch` on `main-touch:<p>`, making `now - lease.last_human >= 60 s` immediately true 1 ms after acquisition.
- Why: In `PC 5.10 AL-10` (`PC` line 825), `Acquire` sets `last_human = r.ts_ms if not r.provisional else lease.last_human`, whereas `PC 5.11 AL-11` (`PC` lines 897–900) redefines `last_human(h, g)` as:
  `max ts_ms over (Acquire|Promote of h at g with origin == human) union (Touch on main-touch:<p> of h at g)`.
  If `h` acquired the lease before `OriginReporting` was posted (so `Acquire` had `origin` omitted rather than `origin == human`) and `OriginReporting` is then enabled, or if `main-touch:<p>` compacted/rolled over, `last_human(h, g)` under `AL-11`'s formula has an empty set, allowing immediate `FastTakeover` without waiting the 60 s silence window (`V-11`).
- Fix: In `PC 5.10 AL-10` and `PC 5.11 AL-11`, unify the definition of `last_human(h, g)` so that a non-provisional `Acquire` at generation `g` recorded when `OriginReporting == false` counts `Acquire.ts_ms` as the initial `last_human` baseline for generation `g`.

---

## 5.4 What you checked and found sound

1. **Operator Rulings `O-1..O-7` alignment across `REQ`, `ARC`, and `PC`/`PCa`/`PCb`:** Verified that `O-1 A` (`artifact put --hub` in `AL-28`, leaving git/rsync/scp untouched), `O-2 D+` (doorbell preconditions, 300 s external watchdog, S1/S2 activity and conversation signals in `AL-1..AL-8`), `O-3 A` (human-only main lease renewal, 60 s fast takeover, and "mail waiting, main not reading" signal in `AL-10..AL-13`), `O-4 A` (runme and `B-26` re-based on TermLink), `O-5 A` (`V-1..V-13` operational value register in `ARC 8.1`), `O-6` (Watchtower + OOB approvals with no new hub verb, reliability vs usage metrics, and `NID`), and `O-7 A` (`L1`/`L2`/`L3` per-project security levels in `ARC 8.4` and `AL-30`) are consistently reflected in the architecture and build items (subject to the specific algorithm fixes in Section 5.3).
2. **Pull-on-turn hand-over and crash recovery (`AL-9` `hook_fetch`):** Checked the two-step claim (`OPEN` attempt record before rendering output, transitioning to `HANDED_OVER` only after `stdout.write` + `flush` succeeds, and recovering interrupted attempts on the next hook call within the 10 s window). Aside from the `seq <= up_to` filter (`ANTIGRAVITY-10`), the crash-recovery and `HANDED_OVER` stage semantics are sound and prevent silent drop on hook crash.
3. **Out-of-band notification idempotency and single-writer fencing (`AL-13` `oob_send_once`):** Checked the `oob-sent:<project>` topic check, the `coord:claims` per-episode claim (`ttl = 120 s`), and the local ledger fallback when the hub topic is unreachable.
4. **Notification-flood coalescing and urgent isolation (`AL-16` and `AL-24`):** Checked that normal sender queues are capped (`Q_SENDER = 20`) with coalesced summary records while `urgent` (`priority >= 5`) messages bypass normal sender coalescing (`AL-24` step 1) and are capped separately (`FCW-113`).
5. **Sender-key binding, TOFU pinning, and key rotation (`AL-19`, `AL-22`, `AL-23` steps 5–8):** Checked that `AL-23` verifies canonical envelope fields, checks `known_senders` TOFU pin before accepting, and requires dual-signature (`old_sig` + `new_sig`) or an operator Tier-0 approval artifact for key replacement (`AL-22`).
6. **Display-framing / terminal-escape neutralisation (`AL-34` `neutralise_for_display`):** Checked that C0/C1 controls, CSI/OSC/DCS/APC sequences, directional overrides, and spoofed `[termlink-envelope ...]` markers are stripped/escaped before rendering untrusted message bodies (`SI-9`).
7. **Later-phase algorithms (`PCb` `AL-41..AL-55`):** Checked Tier-0 dual-route approval verification (`AL-41`..`AL-42`, noting `ANTIGRAVITY-17`), policy evaluation (`AL-43`..`AL-44`), remote grant verification (`AL-45`..`AL-48`), circuit breaker (`AL-50`..`AL-52`, noting `ANTIGRAVITY-11`), retention/redaction (`AL-53`), and encrypted hub recovery (`AL-54`..`AL-55`).

---

## 5.5 Not reached

None — all required documents in Section 2.1 (`REQ`, `ARC`, `PC`, `PCa`, `PCb`) and all context documents in Section 2.2 (`THR`, `FP`, `EV`, `TRC`, `COV`) as well as `CLAUDE.md` were read and reviewed in full.
