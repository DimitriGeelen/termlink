I've written the review, but the harness never gave me a path for result.md, so it's at `/tmp/t3580-review-claude/result.md`. Move it if the harness expects it somewhere else. I read the local checkout only and changed nothing in the repo.

**Q6, the cause:** the suspected cause is right.
- **The race:** `save_result` writes every save to the fixed temp file `known_hubs.tmp` (`tofu.rs:231-233`). That temp file can be cleared by a second writer and then renamed into place by the first. A process that loads at that moment sees an empty store and "first-pins" every hub again, one after another, saving a smaller store each time. That matches the bursts: one process re-pins A–D within a second.
- **Why it's so frequent:** every connection builds a fresh store, reading the file (`tofu.rs:412`, `client.rs:88,142`). Every matching connection then rewrites the whole file just to update `last_seen` (`tofu.rs:286-291`). So parallel health checks are the worst case.
- **Reads fail open:** `load` treats any read error (permissions, I/O, bad UTF-8) as "no pins" (`tofu.rs:159-162`).
- **A second writer:** `scripts/deploy-remote.sh:125-129` removes a hub through the same `known_hubs.tmp` name. Its loose `grep -v` match can drop other hubs too. It writes no `tofu-clear` event, so the operator never sees that removal.
- **A path mismatch:** `check-pin-events.sh:20` ignores `XDG_STATE_HOME`, while the Rust code honours it. If XDG is set, the check and the writer use different trails.
- **No wrong pins:** this race can't put a wrong fingerprint in the store, because cut-off lines are dropped whole. So all 21 events came from real handshakes. The race leaves gaps with no pin, not wrong pins.

**Q7, acknowledge now: yes, with two quick checks first.**
1. Confirm each hub's fingerprint out-of-band, by hashing the certificate file on the hub host itself. A probe over the same network proves less.
2. Confirm the five stored entries match the events, and that no address in them is unfamiliar.

Don't wait for the fix to acknowledge, but note in the acknowledgement that more such events will come until it lands.

**Q8, fix:**
- Take an exclusive advisory lock on a separate `known_hubs.lock` file.
- Inside the lock, re-read the store, apply only this process's change, and save through a uniquely named temp file. Make the first-pin decision inside the lock too.
- Stop rewriting the file on every match, and refuse to connect if the store can't be read.
- Pitfalls: on NFS the lock may not work across machines, Windows rename over an open file may need a retry, and waiting for the lock needs a time limit. A crash can't leave the lock stuck, because the system releases it when the process dies.

**Q9:** keep tombstones and pin history. A first pin for an address seen before without a deliberate clear gets its own `tofu-pin-lost` event. If its fingerprint differs from the last known one, it is refused until the operator clears it.

**Q10:** the test runs 32 separate processes against a temporary store, plus a variant that kills workers mid-run. It asserts that:
- every pin survives;
- there is exactly one `tofu-pin` event per new address;
- no worker ever loads a short store;
- no temp files are left behind.

Before trusting the test, confirm it fails on today's code.

Confidence is about 85% on the cause.
