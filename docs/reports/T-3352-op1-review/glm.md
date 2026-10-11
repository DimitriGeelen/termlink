# Independent review of OP-1 (T-3352 step 3)

## 1 Recommendation

1a I recommend **option B**, with two amendments. The core reasoning of 10.1.8.e is sound and I adopt it on its merits: A3 coupled to "before PH-4 exits" is the right coupling, because PH-4 is the first phase where the floor claims a file exists "an agent cannot write" (SI-20, SI-21). Before PH-4 the only authority is a fixed built-in default (NK), so base strength plus detection genuinely covers what the record shows; from PH-4 the claim would be false without accounts.

1b The pilot placement is not really a choice: R-82.a(3) makes every account creation a Tier 0 act, and the C-15 route exists only from PH-3. v0.1's PH-2 placement was correctly withdrawn — it would have violated the ruling. So B is the earliest *legal* pilot, and the price gate (H-28 measured, operator confirms before migration) is the right control on an estimate marked **[A]**.

1c Rejections I agree with: A front-loads the largest unknown ahead of a first release whose value is honesty and visibility (INC-a), not same-host containment; C leaves the injection-to-root path open through every capability-adding phase; D delays a committed target.

1d Amendment 1: make A0's read-only inventory an explicit PH-0 exit item. The design only says it "may start earlier"; the inventory is what tells you the 86 root lines' *why*, and it gates nothing.

1e Amendment 2: add a standing rule that no *new* agent is brought up at root after the price gate. A window accepted for a fixed estate is defensible; one accepted for a growing estate is not.

1f An option not listed: pull the C-15/Tier 0 machinery slice earlier so the pilot can run in late PH-2. I would not — PH-2 already carries waves W1–W4 and slip risk, and buying weeks on the pilot by endangering the first operational release trades the wrong things.

## 2 Risks of B at base strength

2a The window PH-2→A3: a prompt-injected agent ends in a root shell (RR-1/RR-3 realized at full strength), reads sibling keys, signs as a sibling (SI-3 gives attribution/detection only), and consumes inboxes (INC-c class).

2b The *larger* injection surface in that window is RA-9, not the new chain: the pickup and other old rails stay unframed until PH-4, and they are where peer text actually arrives today. The mitigation is the project rule (G-020) plus agent judgement — real but soft. If anything tightens this window, it is framing the pickup rail earlier, not moving accounts earlier.

2c IC-2's interim approval is not a lock — a root agent can run the approve verb (BP-13). Tier 0 in PH-3 is discipline plus an audit trail, not enforcement. This is stated honestly; it is tolerable only because the audit trail survives and 3.1.e forbids overclaiming.

2d Shipping PH-2/PH-3 at base strength is defensible: the operator already accepted RR-1/RR-2; every incident in the record (INC-a..i) is an honesty/visibility failure, not a compromise, and base strength addresses exactly those; the kill switch and status honesty stand in. It becomes indefensible if the estate grows at root during the window, or if PH-4's "cannot write" claims are made while A3 has slipped (the re-ask of RA-1 is the right guard — keep it binding).

## 3 Price and stage cut

3a The table is plausible in shape; my read is 6–10 engineer-weeks total, and the binding constraint is operator attention, not code. Most likely underestimated, in order: (1) **the harness login per account** — N interactive authentications, and copying one credential into N accounts spreads it, the opposite of the aim; this could reshape the design; (2) **the operator's attach workflow** — tmux/PTY sockets per account, and `sudo`-per-attach breeds `sudo -i` habits that quietly defeat isolation; (3) **shared-checkout churn** — git mode noise, `safe.directory`, and the already-known index contention (T-3090/T-3231) multiplied by 19 accounts; (4) **cron/canary re-homing** — the canary layer assumes one `HOME`; (5) **TOFU re-pinning** — every account pins every hub again.

3b The stage cut is sane and H-28's thresholds (≤5 fixes; >15 distinct failures triggers redesign) are a good gate. One gap: a single pilot under-samples the estate's class diversity (systemd-supervised, tmux-managed, unmanaged — the T-3296 zombie classes). Pick a representative non-critical pilot and order A2 waves by agent class. A3's "no agent as root" check must count cron and systemd contexts, not only live interactive processes.

## 4 Unsafe elements

4a Helper creep: the sudoers entry must be NOPASSWD for one exact argv, no wildcards; helper output logs to an append-only store agents cannot write; every added helper action is a ruling change. Keep the R-82.d refusal fixture in the guard layer permanently.

4b Rollback must not delete accounts: the one-command return returns the *agent* to root and keeps the account and its HOME for the store's 14-day window — deleting an account destroys keys, pins and transcripts.

4c Half-migrated estate: extend status honesty so the status call names per-agent migration state; a mixed estate where some floor claims hold and others do not must never render green.

4d RA-7: A3 alone does not buy the promised isolation — the shared checkout stays a lateral path forever. The installed-code boundary and hook-hash checks must be A1 deliverables, not post-A3 cleanups; make the proposed SI-32 binding at A1.

4e The hub's own account move is a PL-021-class risk (secret/cert regeneration on restart); pre-seed the new runtime dir and verify persistence ground truth — this repo documents 14-day silent rotations.

4f Until A4 the helper and sudoers are administered by a root orchestrator, so "never general root for the orchestrator" is only fully true at PH-6; the status call should say so.

## 5 Verdict

5a **Option B** (with 1d/1e) — it is the only placement where the pilot is even legal, and it makes the platform reality (accounts) arrive at the same moment the floor first claims "an agent cannot write".
