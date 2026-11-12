# T-3356 panel answer — seat `claude-code`

## 5.1 Header

- **Seat:** claude-code
- **Model:** Claude Opus 5.5 (`claude-opus-5-5`), running in Claude Code
- **Date:** 2026-11-11
- **Brief:** version 1, material revision `c9aea3befd7ce44751ea7f9bf954da02a65c6845`
- **Hashes:** all match. I checked all ten files of brief section 2 with `sha256sum`: REQ, ARC, PC, PCa, PCb, THR,
  FP, EV, TRC and COV.
- **What I read:**
  - PC in full: sections 1–5 and the FCW register, up to FCW-113.
  - ARC 6.9, 6.13, 6.14, 7, 17, 19 and 20.
  - PCa AL-22, AL-23, AL-24, AL-29, AL-30, AL-36, AL-37, AL-38, AL-39 and AL-40.
  - PCb AL-41 to AL-46 and AL-50 to AL-53.
  - EV E-5, E-7, E-13, E-27, E-85 and 15–17.
  - A mechanical check that every R-n of REQ and every SI-n and TH-n of THR is cited somewhere in ARC, TRC or PC (no
    misses).
  - To check one [B] claim, I also read the built `pull`/`withhold` code in `crates/termlink-sidecar/src/api.rs` and
    `lease.rs`.

## 5.2 Verdict

**Yes, after the critical and high findings are fixed.** The PH-2 delivery path has a sound shape: one writer of
HANDED_OVER, a durable intent before every keystroke, the FAILS CLOSED WHEN register, and the lease's fail directions.
But B-27, B-16 and B-25 would build in several fail-open or wrong-session defects. The three that matter most:

1. **The watchdog does not re-read typing just before it types (CLAUDE-CODE-1).** It can type the line, plus Enter,
   into a person's half-written prompt at exactly the moment a person comes back to an idle session. That is the case
   O-2 D+ ruled out.
2. **The receiver-side prompt-origin checks never reach the lease (CLAUDE-CODE-3, -4, -5).** Combined with the
   provisional take at any SessionStart, the O-3 tightening can be bypassed by `inferred` origins, by headless worker
   sessions and by mixed-harness projects. In mixed-harness projects it also switches off main for harnesses that
   do not report origin (NID).
3. **The watchdog can be held off or made to repeat without bound (CLAUDE-CODE-2, -6).** Any adapter report, including
   a repeated `ready` report, counts as S2 activity, so a stuck adapter suppresses the watchdog forever. In the other
   direction, an unsubmitted line gets one more watchdog line every 300 s with no limit. Separately, forward-once has
   an ambiguous-post path that delivers one message at two agents (CLAUDE-CODE-7).

## 5.3 Findings

### CLAUDE-CODE-1
- **Where:** PC 5.8 AL-8 step 4 → PC 5.6 AL-6 steps 3–5 (`via = watchdog`); ARC 6.9.7.b; EV E-7.
- **Severity:** critical
- **Class:** algorithm / ruling
- **What is wrong:** The watchdog decides "300 s of silence" from `last_activity`, which `meter_tick` computed at the
  start of the tick. It then types the line and Enter without re-reading S1. AL-6 step 5, the typing re-read, runs only
  for `via == doorbell`.
- **Why:**
  - Between the meter read and `command.notify`, the time is unbounded by any typing check. It covers up to one tick
    (30 s) of loop scheduling, plus the 2 s RPC, the 1.5 s veto and the durable append.
  - The most likely moment for a person to come back to an idle session with pending mail is right around the 300 s
    mark. That is exactly when the watchdog is due.
  - A person who starts typing in that window gets `sidecar: N peer messages waiting…` appended to their partial text.
    The second write (Enter) then submits both.
  - ARC 6.9.0 rejects option B precisely because it "types … into a person's half-written prompt". AL-8 invariant (1)
    ("every fire is preceded by at least 300 s in which no S1 … event happened") is false as written.
  - Re-reading S1 does not defeat the watchdog's purpose. S1 is measured by the TermLink session itself and cannot be
    "stuck" the way READY can.
  - AL-6 step 3 already fetches `session.input_activity` for the foreground check, so the client stamp is in hand at
    no cost.
  - E-7 freezes S1, so no spec catches this.
- **Fix:**
  - **ARC 6.9.7.b:** add "no client input in the last 30 s (re-read)" to the watchdog's checks. Leave out READY and
    the conversation signal.
  - **PC AL-6:** run the step 5 S1 re-read for both `via` values, using the step 3 reply. Correct AL-8 invariant (1).
  - **EV E-7:** add a case where a client write lands after the meter tick and before the fire, with the expected
    result `held:typing`. Its mutant drops the re-read.

### CLAUDE-CODE-2
- **Where:** PC 5.1 AL-1 step 7 (last line, `sess.s2_at = rx.mono # S2 (a), every valid event`); PC 5.3 AL-3; ARC 6.14.1
  and 17.4.
- **Severity:** high
- **Class:** algorithm / threat
- **What is wrong:** S2 moves on every valid adapter report, including `ready`, `evidence` and `complete`. ARC 6.14.1
  and 17.4 define S2 (a) as the turn events only: session_start, prompt_submit, tool_start, tool_end, turn_stop and
  session_end.
- **Why:**
  - The watchdog exists for a stuck or false READY flag. An adapter that is stuck re-sending `ready {BUSY}` keeps S2
    fresh. Examples are a hook firing on a status-line refresh, a looping hook, or any harness whose adapter re-reports
    state on a timer.
  - Each such report has a new `at_ms`, so it is applied (FCW-3 drops only older ones) and refreshes `s2_at`.
  - `last_activity` then never ages past 300 s, and the watchdog never fires. That is the exact failure O-2 item 2
    requires it to survive.
  - Nothing in 17.3 forbids repeating a READY report.
- **Fix:**
  - **PC AL-1:** move S2 only on the 17.4 turn events, and on a `ready` report only when its state changes.
  - **PC AL-3 invariant (1):** state this.
  - **ARC 17.3:** say a repeated READY state is not activity.
  - **EV:** add a fixture "adapter repeats BUSY every 10 s; watchdog still fires at 300 s + 1 tick".

### CLAUDE-CODE-3
- **Where:** PC 5.1 AL-1 `check_origin` step 3; PC 5.11 AL-11 hooks block ("the adapter's report, AL-1 rules");
  QR-3; ARC 17.5.b, 17.5.d, 19.7 AR-6; EV E-27, E-85.
- **Severity:** high
- **Class:** bypass / ruling (O-3)
- **What is wrong:** There are two gaps, and together they mean the receiver's origin downgrade does not protect the
  lease.
  - **Gap 1, the check is skipped for `inferred`.** The receiver checks its own PTY input counters only when the
    adapter says basis `pty-input`. A `human` claim with basis `inferred` is accepted without looking at the counters.
  - **Gap 2, the downgrade never reaches the lease.** AL-11's `claim()` runs in the `termlink main` process, with the
    origin the hook passes in. No step consults the sidecar's downgraded `last_prompt`.
- **Why:**
  - 17.5.d makes `inferred` the Claude Code adapter's normal `human` basis until T-5054 verifies otherwise.
  - So for the main harness, FCW-5 (step 3) never runs. Step 2 depends on the adapter echoing `doorbell_ids`, which an
    adapter that failed to recognise the line will not do.
  - Any harness-generated prompt the adapter cannot mark is a human renewal: a `/loop` wake-up, a restored stale line
    (T-3591), a doorbell line appended to text already in the buffer. This includes a session under a TermLink PTY,
    where the receiver *could* see that no `client` bytes arrived since the previous prompt.
  - AR-6 says it "closes when … the session runs under a TermLink PTY (basis `pty-input`)". As written, it does not
    close, because the receiver never applies its counters to `inferred`.
  - Gap 2 means that even when FCW-5 fires, it changes only the conversation signal, not the lease. The "forgotten
    session renewing forever through automated prompts" that O-3 closes stays open.
  - E-27 and E-85 test only the adapter's output, not the receiver's override.
- **Fix:**
  - **PC AL-1:** for any `human` origin from a session under a TermLink PTY, downgrade to `automated` when no `client`
    bytes, other than `client_auto`, arrived since the previous `prompt_submit`.
  - **PC AL-11:** a `prompt` claim from a session served by a sidecar MUST carry the sidecar's checked origin. The hook
    asks the sidecar, or `termlink main claim` queries the sidecar and takes the lower of the two origins. If the
    sidecar is unreachable, the origin is `unknown`.
  - **ARC 19.7:** update AR-6.
  - **EV:** add a receiver-side negative control: an adapter fixture that reports `human/inferred` for a doorbell line
    and for a `/loop` prompt, with no client bytes, must not renew main.

### CLAUDE-CODE-4
- **Where:** PC 5.11 AL-11 plan table, row `VACANT | session-start | — | TakeProvisional`; ARC 6.13.2.
- **Severity:** high
- **Class:** algorithm / ruling (O-3)
- **What is wrong:** ARC 6.13.2 allows the provisional take only "at SessionStart of an *interactive* session". The PC
  table has no guard, and nothing feeds `interactive` (AL-1 `session_start.interactive`) into `claim()`.
- **Why:**
  - Headless and dispatched sessions fire SessionStart too: `claude -p`, fleet workers, review workers like this one,
    and scheduled agents.
  - On a vacant lease such a session becomes the provisional main for 5 min, and 6.13.2 says a provisional holder is
    main for `am-i`.
  - Role mail is then surfaced to an unattended worker doing unrelated work, and HANDED_OVER is legitimately recorded
    there. The operator's session is withheld (AL-12). So the message is delivered to the wrong session with no error
    anywhere.
  - In a project that dispatches many workers, every worker start raises the generation. Each one turns any in-flight
    FORWARD into FORWARD-STALE (AL-15 step 6), and the lease flaps between provisional workers and vacant.
- **Fix:**
  - **PC AL-11:** add the guard `interactive == true` (from the adapter's `session_start`, or `non-interactive` basis
    absent) to the TakeProvisional row. A SessionStart from a non-interactive session is "Answer only".
  - **EV:** add an E-23 case with a print-mode SessionStart on a vacant lease, expected result "no take".

### CLAUDE-CODE-5
- **Where:** ARC 6.13.1.a; PC 5.11 AL-11 step 3 (`if origin is absent and rep: origin = unknown`, FCW-53); ARC 17.10.
- **Severity:** high
- **Class:** ruling (NID) / portability
- **What is wrong:** "Origin-reporting" is a per-project switch, and it flips the first time any session of the project
  sends an origin. From then on, claims without an origin are ignored.
- **Why:**
  - A project is run by Claude Code (adapter updated by T-5054) and by codex, opencode or antigravity sessions whose
    adapters do not yet report origin. The panel's own seat list shows such projects exist.
  - The first Claude Code prompt flips the project. From then on, the other harnesses' sessions can never acquire,
    renew or take main.
  - Role mail gated by `am-i` is no longer surfaced to them. Their sessions are withheld at a TermLink sidecar
    (AL-12), so a working capability is switched off as a side effect.
  - There is no proven replacement and no operator approval. CLAUDE.md's NID rule and O-6 forbid this.
  - It also makes the "neutral" lease (17.10) depend on which harness shipped its adapter first.
- **Fix:**
  - **ARC 6.13.1.a and PC AL-11:** decide reporting **per harness** (or per adapter `harness` field), not per project.
    A claim with no origin from a harness that has never reported origin counts as before.
  - Flip a harness only through a recorded operator act (runme) once its adapter passes the 17.9 fixtures.

### CLAUDE-CODE-6
- **Where:** PC 5.7 AL-7 (row `UNSUBMITTED | watchdog typed → AWAIT_SUBMIT`) with PC 5.8 AL-8 step 3; ARC 6.9.5,
  6.9.7.d, 19.6 AR-5.
- **Severity:** high
- **Class:** algorithm
- **What is wrong:** When a typed line is never submitted, the watchdog types another line every 300 s, forever, into
  the same input buffer.
- **Why:**
  - A line is typed but Enter does not take. This is the operator's observed hazard 6.9.5 (b). The session moves to
    UNSUBMITTED.
  - Nothing then moves S1, S2 or S3: `notify` bytes are not S1, and the transcript does not grow.
  - So every 300 s, `since = max(…, wd_last_fire)` expires and the watchdog appends another line. Each fire returns the
    state to AWAIT_SUBMIT, and then to UNSUBMITTED one tick later. The mail stays pending, so this never ends.
  - That is 12 lines an hour, 288 a day, accumulating in one prompt buffer. If Enter later takes, that is one enormous
    prompt; if not, the person returns to hundreds of lines.
  - AR-5's "bounded to one line per 300 s" bounds the rate, not the total. H-11 (e) counts it but nothing stops it.
- **Fix:**
  - **PC AL-7/AL-8:** after a watchdog fire into a session already UNSUBMITTED with no submit, set a terminal state,
    for example `STALLED`. Allow no further typing of any kind until a submit or a session restart, and raise a
    needs-attention entry once.
  - At most one watchdog line per unsubmitted episode, written into ARC 6.9.7.d.
  - **EV E-5/E-7:** add the case with 60 simulated minutes and no submit, expecting exactly one watchdog fire after
    the first unsubmitted ring.

### CLAUDE-CODE-7
- **Where:** PC 5.14 AL-14 step 7 and FCW-71; crash table row "post and FORWARDED".
- **Severity:** high
- **Class:** algorithm (message processed twice)
- **What is wrong:** When the post of a FORWARD fails ambiguously, the forward is abandoned and the message is offered
  at Y. That gives two hand-overs, one at X and one at Y.
- **Why:**
  - The ambiguous cases are a timeout, or a lost response after the hub stored the envelope.
  - Y retries for 1 min, gets errors, writes FORWARD_ABANDONED and offers `m` at Y again.
  - X has already pulled the first copy, verified it (AL-15) and handed it over.
  - The identity triple dedupes copies *at one receiver*, so it cannot prevent this.
  - H-24 ("0 processed twice in 1000 forwards") and AL-14 invariant (2) do not hold.
  - The design treats "post failed" as "not posted", which is the exact at-most-once/at-least-once confusion the card
    warns about (5.14.g even says the target can commit while Y's record says intent only).
- **Fix:**
  - **PC AL-14:** before abandoning, query the hub for the idempotency id `(Y, "forward", m.id)` on `inbox:X`. If it is
    present, write FORWARDED. If the query fails, the outcome is UNCERTAIN for the operator (as AL-9 does), never
    "offer at Y". Abandon only on a definite "not present".
  - **FCW-71 and EV E-35:** amend them, and add the lost-reply fault.

### CLAUDE-CODE-8
- **Where:** PCa 2.7 AL-23 2.7.f (3) ("steps 8..11 under the store lock"); step 9 (blob fetch) and step 11
  (`after_stored(m)`) → PC 5.14 AL-14 step 3 (synchronous lease read, 10 s), step 5 (`with store.lock`) and step 7
  (hub post).
- **Severity:** high
- **Class:** algorithm (left ambiguous enough to be built two ways, and one of the two is a deadlock)
- **What is wrong:** Admission holds `store.lock` across network calls, and the nested `store.lock` in AL-14 is either
  a self-deadlock or a silent assumption that the lock is re-entrant.
- **Why:**
  - The network calls under the lock are a blob fetch of up to 16 MiB, then `after_stored`, which can do a 10 s
    synchronous lease read and a hub post.
  - Meanwhile the hook pull and every other admission wait on the lock, and so does every `store.lock` user. A slow or
    hostile hub stalls hand-over for the whole agent.
  - If `store.lock` is not re-entrant, AL-14 step 5 deadlocks the receiver on the first role message.
- **Fix:**
  - **PCa AL-23:** fetch and verify the blob before taking the lock.
  - Call `after_stored` after releasing the lock, as a separate step that is crash-safe through the start-up scan
    AL-14 already defines.
  - **PC 3.2.a:** state that `store.lock` is not re-entrant and never held across an RPC.

### CLAUDE-CODE-9
- **Where:** PC 5.4 AL-4 step 1 and 5.4.f (5); FCW-16; QR-2.
- **Severity:** medium
- **Class:** algorithm (fail-open after a restart)
- **What is wrong:** After a sidecar restart, `last_prompt` is lost and AL-4 returns `none`. The doorbell may then ring
  into a live conversation.
- **Why:**
  - The code reads `if p and p.origin in {human, unknown, absent}`.
  - FCW-16 says an `absent` origin means `operator`, but a missing `p` skips the clause.
  - Within one tick of a restart, a person who is reading the agent's answer but not typing is protected only by the
    30 s typing veto.
  - This is the opposite of the design's own direction ("unknown fails toward a person"). The data is tiny and easy to
    persist.
- **Fix:**
  - **PC AL-4:** persist `last_prompt` with `sess.ready` (atomic_write). Or treat "no prompt seen in this
    `sidecar_epoch` and an S2 event within 120 s" as `operator`.
  - Close QR-2 accordingly.

### CLAUDE-CODE-10
- **Where:** PC 5.1 AL-1 step 7 `ready` ordering rule (FCW-3) and 5.1.f.3; ARC 17.2.
- **Severity:** medium (I am unsure how `at_ms` is produced; see Why)
- **Class:** algorithm (clock)
- **What is wrong:** READY reports are ordered by the adapter's `at_ms`. A backward wall-clock step freezes the READY
  flag until the clock catches up.
- **Why:**
  - Hooks are separate processes per event (Claude Code), so "monotonic-anchored wall time" (17.2) has no shared
    monotonic anchor. In practice `at_ms` is the wall clock.
  - After an NTP step back of Δ, every report has `at_ms` < the applied one and is dropped as reordered, for Δ.
  - 5.1.f.3's "the next report repairs it" is false for that period.
  - If the frozen state is READY, the gate keeps believing the agent is idle while it works. Only the re-checks hold
    it.
- **Fix:**
  - **ARC 17.2 and PC AL-1:** have the adapter send `boot_id` plus a monotonic `CLOCK_MONOTONIC` value, which every
    process on the host shares, and order on that.
  - Or accept a report whose `at_ms` is more than a threshold (for example 5 s) behind the applied one as a clock step:
    apply it, re-base, and count it.

### CLAUDE-CODE-11
- **Where:** PC 5.6 AL-6 step 3 (`sess.harness_exes`, `sess.harness_pid`); PC 5.5 AL-5 step 4 (`sess.pty_launched`,
  `sess.harness_exe`); ARC 17.4.
- **Severity:** medium
- **Class:** coverage / portability
- **What is wrong:** The foreground binding, which is SI-2's core check, compares against `harness_pid` and a
  registered executable list. No algorithm says where either comes from or when they change.
- **Why:**
  - AL-1 `session_start` carries `transcript_path` and `interactive`, not a pid.
  - The harness may be restarted inside the same TermLink PTY: a user exits `claude` and restarts it. If `harness_pid`
    is stale, every ring is held. If it is re-learned from the foreground itself, the check is circular.
  - The executable registry is unspecified for harnesses run through an interpreter (`node`, `bun`). A registry entry
    of `node` matches any node program in the foreground.
  - This will be built two ways.
- **Fix:**
  - **PC AL-1/AL-2:** the TermLink session records the pid and `exe_realpath` of the process it launched, and the pid
    of every later process-group leader the user starts. `session_start` carries the adapter's own `ppid` chain, and
    the sidecar accepts a harness pid only when it matches a descendant of the PTY's session leader.
  - State that the registry matches on `(exe_realpath, argv[1] script path)` for interpreted harnesses.
  - **ARC 17.4:** add `harness_pid` to `session_start`.

### CLAUDE-CODE-12
- **Where:** PC 5.15 AL-15 step 6; ARC 7.3.
- **Severity:** medium
- **Class:** algorithm / internal inconsistency
- **What is wrong:** X sends FORWARD-STALE whenever `L.generation != env.generation`, even when the lease moved
  between two sessions of the same agent X. ARC 7.3 defines mismatch as "X is no longer main".
- **Why:**
  - Fast takeover between the operator's own two windows (6.13.3) raises the generation. So does a provisional take,
    including every worker start, as in CLAUDE-CODE-4.
  - Every forward in flight across such a change bounces to the sender, which re-resolves and re-sends. The agent is
    unchanged, and within an agent AL-12's withhold already routes to the right session.
  - Under frequent generation changes, a forward may never be accepted.
- **Fix:**
  - **PC AL-15:** compare `L.holder.agent == MY_AGENT` only. Keep the generation in the record for audit, and compare
    it only when the holder agent differs.
  - Or state in ARC 7.3 that a same-agent generation change is accepted.

### CLAUDE-CODE-13
- **Where:** PC 5.14 AL-14 (runs only from `after_stored`); PC 5.13 AL-13 step 3 and 5.13.f (1).
- **Severity:** medium
- **Class:** algorithm / coverage
- **What is wrong:** The forward decision is made once, at STORED. Role mail stored at Y while the lease was unknown,
  stale or vacant is never re-evaluated. The waiting-mail signal is blind to it, because the lease holder X *is*
  reading.
- **Why:**
  - Y's hub link blips, the lease read fails (FCW-67) and the mail is "delivered as before" at Y.
  - Y may have no live session, or only non-interactive ones.
  - AL-13 computes `reading` from X's `last_human_at`, so the signal never rises. The sender gets STUCK only at the
    R-44.e deadline, and nobody on the receiving side is told.
- **Fix:**
  - **PC AL-14:** add a per-tick re-evaluation of stored, unforwarded and un-handed-over role mail at Y. This uses the
    same predicate as the start-up scan.
  - **PC AL-13 / ARC 6.13.5:** count role mail held by a receiver that is not the holder's agent as "not reading" for
    that receiver.

### CLAUDE-CODE-14
- **Where:** PC 5.12 AL-12 `withhold` ([B]; built `api.rs` `pull` applies it to every message); ARC 7.1 ("withholds
  role mail"); PC 5.13 FCW-64.
- **Severity:** medium
- **Class:** coverage / internal inconsistency
- **What is wrong:** ARC 7.1 and the `lease.rs` module doc say only role mail is withheld from a non-main session. The
  built `pull`, and AL-12 as written, withhold *all* mail, including exact-instance conversation mail (R-62.a).
- **Why:**
  - Main is a forgotten session of agent A, and the operator works in another session of A.
  - Exact-instance mail to A is withheld from the operator's session. AL-13 excludes it by FCW-64, so it sits with no
    signal until R-44.e STUCK.
  - AL-5 step 1 uses the same predicate, so it is never rung there either.
- **Fix:**
  - **PC AL-12:** take a decision. Either make `withhold(m, …)` per message and apply it to role mail only, which
    matches ARC 7.1, or state in ARC that all mail is withheld and include exact-instance mail in AL-13's waiting
    signal.
  - Change the [B] note or the code to match.

### CLAUDE-CODE-15
- **Where:** PCa 2.20 AL-36 step 3 ("store") versus PCa 2.7 AL-23 steps 7–10.
- **Severity:** medium
- **Class:** algorithm (left ambiguous enough to be built two ways, and one of the two loses messages)
- **What is wrong:** AL-36 records `seen(sender, conv, n, digest)` and "stores" at AL-23 step 7. AL-23 can still refuse
  the message after that, with QUOTA at step 8, a bad blob at step 9 or a failed fsync at step 10.
- **Why:**
  - If `seen` or the dedupe set is updated before the refusal, the sender's re-send of the same id and digest is
    treated as a duplicate and dropped.
  - Yet the message was never STORED. That is a silent loss, against AL-23 invariant (4) and the "nothing dropped"
    promise.
- **Fix:**
  - **PCa AL-36/AL-23:** split the step. Step 7 *checks* the window and equivocation only. The `seen` and `held_since`
    updates commit together with step 10's STORED write, in one locked step.

### CLAUDE-CODE-16
- **Where:** PCa 2.24 AL-40 `gate(w3)`; ARC 6.9.6, 6.11.1; EV 15.4 and 17.1 YR-1.
- **Severity:** medium
- **Class:** evidence
- **What is wrong:** The W3 gate checks H-11 (a)–(d) only. H-11 (e), unsubmitted typed lines, and (f), watchdog fires
  and the false-busy share, have no bound (YR-1), and no gate reads them.
- **Why:**
  - (e) is the operator's own observed failure (6.9.5 (b)), and (f) measures whether READY is reliable.
  - Under RNU, both are reliability measures, failed attempts per ring.
  - W3, and then the W4 decommission evidence, can pass while lines are piling up unsent (see CLAUDE-CODE-6).
- **Fix:**
  - **FP/EV (YR-1):** set starting bounds for (e) and (f), per ring, under FP-4.
  - **PCa AL-40:** have `gate(w3)` check (a)–(f).

### CLAUDE-CODE-17
- **Where:** PC 5.6 AL-6 step 4 / FCW-41; ARC 6.9.7; FP 6.2.3.d (5).
- **Severity:** medium
- **Class:** portability / ruling (O-2)
- **What is wrong:** Where the veto is Unmeasurable (no `/proc`, no `ss`), the watchdog is always held. On macOS this
  means always, so O-2's guarantee that a stuck READY "can never hold a message forever" holds only on Linux.
- **Why:**
  - The design presents the watchdog as a harness-neutral guarantee.
  - FCW-41 makes it a permanent hold, recorded once per silence episode.
  - FP only asks the platform-lock entry to "say how the veto behaves".
- **Fix:**
  - **ARC 6.9.7 / 19:** state the platform limit as a residual risk the operator accepts, with status
    `watchdog: unavailable (veto unmeasurable)` raised as needs-attention, not a routine row.
  - Or give a macOS veto (`proc_pidinfo`, `nettop`) as a build item.

### CLAUDE-CODE-18
- **Where:** PCb 2.1 AL-41 `create_action` step 1; PCb 2.2 AL-42.
- **Severity:** medium (PH-3)
- **Class:** bypass (operator approves something they did not mean to)
- **What is wrong:** `expires_at` is `wall() + PN-15` of the *requester*, and `requested_by` is a field of the action.
  AL-41 requires only that they are present.
- **Why:**
  - A compromised requester can create an action with `expires_at` years ahead.
  - The operator approves what is displayed, believing PN-15's 10-minute window applies (SI-31 "visible expiry").
  - The approval then stays APPROVED and consumable long after the context the operator judged has changed.
  - `requested_by` can name another agent.
  - Both fields are inside the digest, so the operator "approved" them. But nothing checks them against the hub's
    clock or the authenticated caller.
- **Fix:**
  - **PCb AL-41/AL-42:** the hub sets `requested_by` from the authenticated caller.
  - The hub refuses `expires_at > wall_hub() + PN-15` (with a small skew allowance) at create. FCW-153 gains both
    conditions.

### CLAUDE-CODE-19
- **Where:** PC 5.15 AL-15 steps 2–3.
- **Severity:** low (doubt: depends on who can read other members' inboxes)
- **Class:** bypass
- **What is wrong:** X checks that the poster is a roster member and that `inner_topic` is *an* inbox of an agent of
  X's project. It does not check that `inner_topic` is the poster's own inbox.
- **Why:**
  - A roster member Y that obtains role mail originally posted to agent Z's inbox can "forward" it to X as if Y had
    received it. Y might have it through a shared hub view or by being the sender.
  - Dedupe and the sequence window (step 5) limit replays, but the provenance in the VIA notice is false.
- **Fix:**
  - **PC AL-15 step 3:** require `env.inner_topic == inbox(poster)`, and add it to FCW-73.

### CLAUDE-CODE-20
- **Where:** PC 5.13 AL-13 step 7 / FCW-63.
- **Severity:** low
- **Class:** algorithm (reports "sent" without having looked)
- **What is wrong:** If another receiver holds the `oob:<project>` claim, the status reads "sent-by-other within 1 h".
  But the claim holder's own send may have produced `not confirmed`.
- **Why:** Invariant (1), "never says sent without the channel's receipt", is broken on the second receiver.
- **Fix:**
  - **PC AL-13:** the claim holder writes its outcome into the claim record.
  - Other receivers show "claimed by <receiver> at T; outcome <sent at T | not confirmed | pending>".

### CLAUDE-CODE-21
- **Where:** PC 5.11 AL-11 `last_human` (Touch on `main-touch:<p>`).
- **Severity:** low
- **Class:** bypass (bounded)
- **What is wrong:** `last_human` counts any Touch whose `session` and `generation` match the holder. It does not
  require the record to be signed by the holder's key.
- **Why:**
  - Any project member can post Touch records naming the holder. That keeps `last_human_at` fresh, which blocks fast
    takeover and suppresses the AL-13 "not reading" signal.
  - It is bounded by the 15 min claim TTL, because only the holder renews.
- **Fix:**
  - **PC AL-11:** count a Touch only if its signer is the holder's agent key and its `claim_id` equals the live claim.

### CLAUDE-CODE-22
- **Where:** PC 5.6 AL-6 step 8 versus step 7; PC 5.7 AL-7 `on_start`.
- **Severity:** low
- **Class:** algorithm
- **What is wrong:** A ring held at step 8 (`stale-checks`) after its intent was written leaves an intent with no
  outcome.
- **Why:** After the next restart, `on_start` marks the session UNSUBMITTED for a line that was never typed. That
  suppresses normal rings until the watchdog fires.
- **Fix:**
  - **PC AL-6:** move the step 8 check before step 7, or append outcome `held:stale-checks` at step 8.

### CLAUDE-CODE-23
- **Where:** PC 5.4 AL-4 step 1 (`p.at_ms`); PC 5.1 AL-1 (`sess.last_prompt` has no `at_ms`).
- **Severity:** low
- **Class:** other (left ambiguous enough to be built two ways)
- **What is wrong:** AL-4 compares `turn_stop` or READY `at_ms > p.at_ms`, but `last_prompt` does not store `at_ms`.
  "READY report" is also ambiguous: does a `ready {BUSY}` report end the operator window?
- **Fix:**
  - **PC AL-1:** store `at_ms` in `last_prompt`.
  - **PC AL-4:** say "a `turn_stop`, or a `ready` report with state READY".

### CLAUDE-CODE-24
- **Where:** THR 9.2.b SI-2; ARC 20 (no change request).
- **Severity:** low
- **Class:** ruling / consistency
- **What is wrong:** SI-2 still says that before typing, the adapter must say READY and that a missing flag reads not
  ready. The watchdog, as ruled by O-2, types with READY busy or absent. XR-9 updates R-19 and R-20 but no change
  request updates SI-2.
- **Fix:** Add an XR to step 2 rewording SI-2: "the normal ring requires READY; the watchdog requires measured silence
  instead (DR-15)".

### CLAUDE-CODE-25
- **Where:** PC 5.8 AL-8 step 1 (per-session watchdog, no per-agent choice).
- **Severity:** low
- **Class:** algorithm
- **What is wrong:** AL-5 rings at most one session per agent per tick (FCW-26). The watchdog runs per session, so
  several silent sessions of one agent can each be typed into for the same ids in one tick.
- **Why:** Only one pull wins the claim, and the rest count as stray rings. The extra lines are noise in idle
  terminals.
- **Fix:**
  - **PC AL-8:** at most one watchdog fire per agent per tick, chosen like AL-5 step 2. Or exclude sessions the lease
    withholds; AL-5 step 1 already does this, so state that it applies.

## 5.4 What I checked and found sound

1. **AL-1 intake boundaries.** The owner-uid check, the 64 KiB limit, strict JSON, the exact schema and the agent-id
   check come before any state change. Refused reports never move READY, origin or evidence, and are counted.
2. **AL-2.** Every PTY write is attributed to exactly one source, and unclassifiable writes count as `client` (the safe
   side). Terminal auto-replies are kept apart from typing.
3. **AL-3 and FCW-12/-13.** An unknown source never reads as silence, and the status freshness bound (FCW-15) is right.
4. **AL-5 gate order and the decision record.** A ring decision is taken on one snapshot, and every hold is recorded
   with its inputs.
5. **AL-6 line construction (SI-1).**
   - `compose` is validated on both sides of the RPC.
   - I recomputed the test vector at 150 bytes from the stated string, which agrees with v0.2.
   - The alphabet has no shell metacharacters.
   - The intent is written durably before the keystrokes.
   - An `unknown` outcome is treated as typed, never as handed over.
6. **AL-9.** It has one writer of HANDED_OVER (the finalizer), a claim before content leaves the store, nonce evidence
   only from harness-written records, HANDOVER_UNCONFIRMED as a label rather than a state, and UNCERTAIN to the
   operator without the complete point. The crash table matches the built kill points.
7. **AL-10.** An unreadable switch reads as off, and the switch is read at every typed path.
8. **AL-11 holder-change mechanics.** It has the CAS on offset 0, a generation on every holder change, the pin never
   fast-taken, "never hold an unrecorded lease", Touch before renew, and the NID default for projects that have never
   reported origin. The flaws are in which claims count (CLAUDE-CODE-3, -4, -5), not in these mechanics.
9. **AL-12.** The fail directions are right: open for delivery, closed for lease-based decisions, and the 60 s versus
   15 s asymmetry is justified.
10. **AL-14/AL-15 cryptography.** The envelope's domain-separated signature, the inner bytes carried unchanged and
    verified against the original topic, the hop limit, never forwarding a FORWARD, and roster membership for the
    outer signer are all correct.
11. **AL-16.** One gate decides every seam, and production refuses loudly. `TERMLINK_MAIN_TEST_*`, which the built code
    honours in production today, is closed (B-25).
12. **AL-22 revocation.** Every outcome except the two verified ones is a failed pull, `unverified_since` is never moved
    by a failure, rollback and fork are refused with a tamper entry, and the unverified-since-start narrowing is in
    place.
13. **AL-23.** Its check order is cheap first, then authenticity, then limits, then durability. Quotas leave the hub
    copy unacked rather than dropping it, and the reserved shares are separated from general traffic. The exception is
    the lock scope (CLAUDE-CODE-8) and the `seen` timing (CLAUDE-CODE-15).
14. **AL-29.** CLEAR needs real surfacing evidence. Heartbeats and ready reports never count, and silence reads
    UNKNOWN, never DEAD.
15. **AL-40 W4 gate (NID).** It reads only the operator's recorded words in the task's Decisions, never the status
    field, and the SendMessage deny is gated on B-26's prover with its control. This satisfies O-4 and O-6.
16. **PCb AL-42/AL-43/AL-44 after r1.**
    - The consume is one compare-and-set under `event.lock` with insert-if-absent.
    - Cancel, revoke and expiry take the same lock.
    - `recover()` follows the crash table with at most one re-apply.
    - The display is rendered from the canonical object.
    - Apart from CLAUDE-CODE-18, I found no interleaving that consumes a final event.
17. **PCb AL-46.** Check and use are one reserve under `grant.lock`, and AL-45's install takes the same lock.
18. **Rule checks.**
    - **RNU:** thresholds and acceptances measure failure per attempt or per ring, never usage. H-21 is removed by
      XR-7, AL-13 episode counts are reported and uncapped, and lease renewal shares by basis are measured, not gated.
    - **Rulings:** O-1 is reflected in AL-28, and O-5 through the V-n starting values under FP-4.
19. **Coverage.** Every R-n of REQ and every SI-n and TH-n of THR is cited at least once in ARC, TRC or PC (a mechanical
    check). Citation is not implementation, and the gaps found are listed above.

## 5.5 Not reached

- **PC:** section 7 onward (the drawings D-1 and D-2, residual risks QR-n, change requests ZR-n), and the FCW register
  past FCW-113. I relied on the per-algorithm FCW text instead.
- **PCa:** AL-17 to AL-21, AL-25 to AL-28 and AL-31 to AL-35 (hub id and key, mail-hub declaration, canonical form and
  vectors, revocation issue, encryption register, blobs, artifact writer, admin log, founding verbs, key cards, dead
  letters, reconciliation), and the drawings in section 3.
- **PCb:** AL-47 to AL-49 (roster, circuit credential, membership) and the drawings.
- **REQ, THR, FP and EV:** not read in full. I read REQ and THR only through the mechanical id check and targeted
  greps, FP only 6.2.3.d, and EV only the specs named in 5.1. TRC and COV I did not read.
- **Code:** I did not review the B-27 code already committed (`8cf07e7a3`) against these findings. CLAUDE-CODE-2
  (S2 on every report) and CLAUDE-CODE-1 (the watchdog's S1 re-read) are worth checking there first.
