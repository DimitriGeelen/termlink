# T-3628 consultation: comparison (codex, antigravity, claude)

Brief: `.context/working/t3628/consult-brief.md` (host details withheld; hubs A–E). Answers in this folder. codex's
result.md was overwritten by its closing line; its review was recovered from its transcript (`result.jsonl`).

## Agreed by all three
1. The suspected cause is real: `save_result` writes through a FIXED `known_hubs.tmp` and replaces the whole file
   from a process-local snapshot, with no inter-process lock (`tofu.rs:205-233`).
2. An amplifier: every matching connection rewrites the whole file just to update `last_seen` (`tofu.rs:285-291`),
   so parallel health checks are the worst case.
3. A load error of any kind is silently treated as an empty store, which means a fresh TOFU (`tofu.rs:157-162`).
4. A second, unlogged writer: `scripts/deploy-remote.sh:124-129` filters and replaces the store through the same
   `known_hubs.tmp`, with a loose `grep -v` that can drop other hubs, and writes no `tofu-clear` event.
5. Fix: an exclusive advisory lock on a separate lock file; under it, re-read, apply only this process's change, and
   save via a uniquely named temp file; make the first-pin decision under the lock; stop rewriting on every match;
   fail closed when the store cannot be read; route deploy-remote.sh through the same API.
6. Keep pin history and tombstones: a first pin for an address seen before, without a deliberate clear, is a
   distinct event ("pin lost"), and a different fingerprint there is refused until the operator clears it.
7. Test: N separate processes against a temporary store; every pin survives, one first-pin event per new address,
   none for existing pins; it must fail on today's code first.

## Where they differ: acknowledge now?
8. claude: YES, after two quick checks: hash each hub's certificate ON the hub host (out of band), and confirm the
   store's five entries match the events. Do not wait for the fix; note that more events will come until it lands.
9. antigravity: YES, now, with the same two conditions (an out-of-band or pre-incident baseline match, and the store
   complete with those fingerprints). Leaving the check red blinds the operator to a 22nd event.
10. codex: NO on the evidence alone. Recover pre-incident fingerprints, verify through an independent channel,
    correlate times with process records, and preserve the store and trail first. After that, acknowledging is fine
    and need not wait for the fix. It also warns that `check-pin-events.sh --ack` acknowledges the count at
    invocation, which can include events that arrived after the review (`check-pin-events.sh:36,52-54`).

## Found by one seat, verified here
11. codex: the TLS verifier never checks the handshake signature (`tofu.rs:371-388` return "valid"
    unconditionally). A matching pinned fingerprint therefore does not prove possession of the hub's private key.
    VERIFIED by reading the code, 2026-10-29. Filed as T-3629 (security).
12. claude and codex: `check-pin-events.sh:20` ignores `XDG_STATE_HOME` while the Rust writer honours it, so the
    check and the writer could read different trails. To verify under T-3628.

## Bearing on the operator's decision
13. All three accept acknowledging once out-of-band checks pass; they differ only on how much checking comes first.
    The out-of-band check matters more now: because of finding 11, the live-wire match proves less than it seemed.
