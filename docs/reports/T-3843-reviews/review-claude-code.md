# Design review: canary vs P-12 (action 37)

## 1. Diagnosis

**The canary is a recurring unnamed reader. It is not proven to be the only one.**

- **The registry merges every unsigned local reader into one row (high).** `record` keys an entry on (agent, instance, strength, key_fp). Every unsigned local read becomes `((unnamed), local, claimed, None)`. One row can therefore hide several readers:
  - `last_ms` shows only the newest of them.
  - `first_ms` shows only the oldest.
- **The first-seen time is not the canary (medium).** 2026-11-20T05:52:37Z is 06:52:37 CET, not HH:47. Some other unsigned local read happened then. Candidates the evidence does not rule out:
  - **Another all-topics script.** `check-artifact-writers.sh` and other T-3xxx canaries are the obvious suspects. If both inboxes also have first-seen times milliseconds apart, the reader was sweeping all topics.
  - **Any client that names itself the T-3309 way, with a `reader` string.** `parse_reader` reads only `read_proof`, so these are recorded as `(unnamed)`.
  - **W1 sidecars running the binary from before P-12, before step 4b restarted them.** They read without signing.
  - **Manual `termlink channel subscribe` runs,** including the agent's own earlier manual reads.
- **The log-mtime evidence is inconsistent with `--quiet` (medium).**
  - With `--quiet` and a healthy result, the script prints nothing.
  - Opening a file with `>>` and writing nothing does not change its mtime.
  - So a daily mtime of HH:47:08 means the canary writes output every day, which in quiet mode means it is firing.
  - Find out what it fires on. If the hits are in `inbox:*` topics, options A and B remove the canary's most valuable coverage.
- **Checks that would settle it:**
  - **Decisive:** poll `termlink hub inbox consumers <topic>` once a minute for 25 h. This call records nothing. Log the `(unnamed)` row's `last_ms`. Every advance must match a canary run, using the cron log and the heartbeat mtime. Any other advance gives the exact time of a second reader to hunt down.
  - **Cheap first step:** grep cron, systemd timers and `scripts/` for `channel subscribe` and `channel list`.

## 2. Options A–D

Before the options, a defect that affects all of them:

- **The canary already reports a tooling failure as clean (high).** `envelopes()` never checks `returncode` and silently drops lines that are not JSON. A refused `subscribe` therefore looks like an empty topic, and the canary exits 0.
- **Binding triggers it.** A bound inbox refuses unsigned reads, and binding is the planned next action. Once an inbox is bound, the canary goes blind on it while reporting "healthy".
- This breaks the rule that a tooling error is never reported clean. Fix it whatever option is chosen.

**A: skip every `inbox:` topic**
- **Weakness (high).** An agent sends `artifact-chunk` pieces straight to a peer's inbox, which is probably the main route for files between agents. Nothing sees it, from now on.
- **Weakness (medium).** Running `--ack` rebuilds `new_acked` from scanned topics only, so it erases the stored acks for skipped topics. If the skip is ever reverted, every old post fires again.
- **Rule:** violated. Coverage is narrowed with no replacement.

**B: skip inboxes that have a proven reader (agent's choice)**
- **Same loss as A where it matters (high).** The skipped set is exactly the two production inboxes, the busy ones. "Every other inbox stays covered" refers to inboxes that carry little mail.
- **The predicate is ambiguous (high).**
  - Read as "exactly one, proven", it never skips. Today's list is `(unnamed)` plus the proven sidecar, and the `(unnamed)` row is the canary's own read from yesterday. B never takes effect.
  - Read as "any proven reader present", it also skips inboxes that a real intruder is reading.
- **Coverage follows sidecar health (medium).** If a sidecar is down for more than 24 h, its proven row ages out. The canary reads the inbox again and adds an unnamed row. Action 37 then stays red for 24 h after the sidecar recovers.
- **Delay (low).** Even when B works, action 37 passes only 24 h after the first run that skips.
- **Rule:** violated, with no replacement.

**C: make action 37 tolerate unnamed readers**
- **Weakness (high).** Any local process reading without a signature passes the check. P-12 stops detecting the exact case it exists for.
- Tolerating only reads at HH:47 can be spoofed by timing a read.
- **Rule:** violated, since the P-12 guard is narrowed.
- The bundle treats this one bad version as if it were every way to "tolerate". Section 3 covers better versions.

**D: do nothing**
- **Weakness (high).** A real second reader produces the same "inbox consumed by 2" line that is already red every day, so nobody will notice it.
- **Weakness (high).** It permanently blocks the binding step that depends on action 37 passing.
- **Rule:** respected on paper. In practice a guard that is always red has been switched off.

**On the scores.** Compliance with the standing rule should be a gate, not a weighted criterion. A and B both fail that gate. B's +6 lead over A comes from inboxes that matter little, and it ignores the self-poisoning predicate.

## 3. Options the agent missed

**E. The canary signs its reads as a named audit reader, and P-12 classifies audit readers**
- **Cost:**
  - Give the canary a key card. That puts a signing key in a root cron job, so key custody has to be designed.
  - Add `read_proof` support to the CLI's paged `subscribe`, signing over `position` on each page.
  - Sidecar status and action 37 must learn the state "one (proven) + auditors (proven)".
  - The binding check (`passes`) must admit listed auditor keys. That is a hub change.
  - The text of P-12 changes, which needs an operator ruling.
- **Does 37 pass?** Yes, once the classifier changes and 24 h after the first signed run, provided §1 finds no other unnamed reader.
- **Coverage:** none is lost.
- **Interim step:** an unsigned `read_proof` naming the agent `file-shaped-canary` immediately splits the canary out of the `(unnamed)` row. That alone settles the diagnosis. It must not be tolerated by action 37, because the name is only claimed.

**F. Each sidecar detects file-shaped posts in its own inbox**
- **Cost:**
  - Port `file_shaped` to Rust.
  - Share one fixture set with the shell canary to prove both give the same results.
  - Report detections and acks through `sidecar status`.
  - The canary skips an inbox only when its sidecar reports the detector is live. That is the "proven replacement" the rule asks for.
- **Does 37 pass?** Yes, after a proving period. During that period the canary still reads, so 37 stays red unless E is combined with it.

**G. The canary reads inbox metadata only**
- No unrecorded read path exists today. A new one would let anyone read sender, `msg_type` and `filename` without being recorded.
- **Does 37 pass?** Yes, but only by creating an unrecorded second reader. That is C under another name (high).

**H. The hub detects file-shaped posts when they are posted, in `channel.post`**
- **How:** the hub already sees the plaintext at post time. It writes an admin row with topic, shape and sender. The canary reads admin rows for `inbox:*` topics instead of reading the inboxes.
- **Cost:**
  - A change to the hub's post path.
  - A proving period running alongside the canary to show parity.
  - The canary's inbox scanning is retired rather than a new guard added, so the guard cap is not hit.
- **Benefits:**
  - Nothing reads the inboxes at all.
  - Bound inboxes are covered too.
  - It is the natural first step toward the later "refuse such posts" decision.
- **Does 37 pass?** Yes, cleanly, once the canary stops scanning inboxes.

## 4. Option B: failure modes and tests

**Failure modes**
- **A binding appears between the consumers call and the read.** The read is refused and, through the defect in §2, reported clean (high).
- **A binding disappears.** The canary skips the inbox for one day (low).
- **The consumers call errors (FCW-125, table full, incomplete history) or times out.**
  - Failing toward a skip creates a silent gap (high).
  - Calling `tooling()` stops the whole scan, so every later topic goes unchecked (high).
  - **Correct behaviour:** read the topic anyway, note the error for that topic, finish the scan, and exit 2 at the end.
- **Inbox on another hub.** The canary scans only the local hub's topic list. The real risk is that root's cron environment sends `inbox consumers` and `channel subscribe` to different hubs (medium). Pin both to the same hub target.
- **Exit code and visibility when skipping.**
  - Skipping should exit 0.
  - But `--quiet` hides the list of skipped topics from the cron log, so the coverage loss is invisible (medium).
  - Always emit `skipped: [...]` in JSON output and one line in quiet mode.
- **`--ack` drops the stored acks of skipped topics (medium).**

**Tests.** Add a consumers fixture to `FILE_SHAPED_TEST_DIR`: one per-topic JSON file, plus an error marker.
1. Inbox with only a proven reader: skipped, its file-shaped fixture does not fire, and the topic is listed as skipped.
2. Inbox with `(unnamed)` plus proven: pins whichever predicate is chosen.
3. Consumers call errors: the topic is still read, and the run exits 2 after the full scan.
4. Consumers call times out: same as 3.
5. A non-inbox topic never triggers a consumers call.
6. A refused `subscribe` exits 2, not 0.
7. `--ack` keeps the stored offset of a skipped topic.
8. A skipped topic plus a firing topic elsewhere: exit 1.
9. Hub integration:
   - a canary run leaves the `(unnamed)` row's `last_ms` unchanged on a skipped inbox;
   - `inbox.consumers` does not change the registry state;
   - consumers and subscribe go to the same hub.

## 5. Recommendation

Do the prerequisites first:
- Run the §1 poll and grep.
- Fix the silent clean on refused reads.
- Name the canary (unsigned is enough for this) so its reads separate from the `(unnamed)` row.

Then build H: hub-side detection at post time, proven in parallel with the existing canary. Until parity is proven, accept D explicitly and on record. Once H is proven, retire the canary's inbox scanning with documented operator approval.

H is the only option that removes the second reader rather than hiding or excusing it. It covers bound inboxes, and it satisfies the replacement rule literally. B fails that rule just as A does, on the inboxes that matter, and its predicate either never fires or skips inboxes an intruder is reading.

**Steelman for E:** signing the canary keeps a working guard at full coverage from day one with no narrowing, so it meets the standing rule immediately. It makes the second reader cryptographically accountable, which is how audit access is normally granted. Its cost is a deliberate, ruled change to P-12's "exactly one" wording, plus key custody for a root cron job.
