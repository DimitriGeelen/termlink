# Review: wc/0.3.4 item 2 (optional dispatch closure)

## 1. Weaknesses

**W1. Close-vs-T1 race: the losing branch of the *close* is undefined — high.**
D's last attempt is `lost`; ring20's auto-closer and a supervisor's T1(`attempt_of=D`) fire together. If T1 commits first, the last attempt is now `dispatched` — the draft specifies only T1-after-close (`dispatch_closed`), never close-after-T1: no typed refusal, no requirement that close re-check terminality inside its own transaction. Backend A closes a dispatch with a live attempt, B silently retries the close, C returns `cas_mismatch` — one race, three outcomes, zero vectors.

**W2. Closure vs late attempt resolution: "closing changes no attempt" is ambiguous, and `unknown` is a hole — high.**
Close commits on last attempt `lost`; the worker's legitimate late lost→done (CC-07/CC-24) then arrives. A backend reading the clause as "closed dispatches reject lifecycle writes" refuses it with an undefined reason — breaking CC-07/CC-24 *on closing backends only*. If the late outcome is done(paused), "never counts as finished" is pinned only for pause-*before*-close, not pause-after (snapshot vs live evaluation). And `unknown` is terminal per the context yet outside closure's {done, lost}: nothing forbids closing on unknown, so D-last-unknown is closed on one backend and merely stuck on another.

**W3. Closure is the only unfenced terminal write; "actor" is a claim, not a principal — high.**
Every attempt transition requires nonce + supervisor token + sgen (`transition_tx` refuses `bad_nonce`/`bad_credential`); closure — the one write that permanently ends a retry chain — specifies no credential, no authorization class, and a free-text actor. Any client that can reach a close route records `actor="ops", reason="done"` and the audit proves nothing; no `bad_credential` path exists for close.

**W4. Terminal but undetectable and unrecoverable — high.**
An operator closes D meaning D′; the owner's next T1 gets `dispatch_closed` forever. The draft defines no reopen, names no §4.7 fields (the excerpted `get` / `wc_dispatches` schema have nowhere to carry `closed/closed_by/closed_reason/closed_ts`), and is silent on whether `allocate(dispatch_id=new, retry_of_dispatch_id=closed-D)` is the sanctioned rebirth. Note also: `allocate`'s fences append no history, so T1 `dispatch_closed` refusals can be unrecorded and unattributable.

**W5. MAY + "accept either" disarms CC-15/CC-19 and legalizes cross-backend disagreement — high.**
Ids are client-chosen (`allocate` accepts `dispatch_id`; `IdTaken` checks only locally), so the same id can exist on hub (never closes) and ring20 (auto-closes); a failover T1(`attempt_of`) succeeds on hub and gets `dispatch_closed` on ring20 — both conformant, no vector fires, the client cannot attribute the divergence. Meanwhile a backend that closes on `running`/`unknown` last attempts still passes CC-15/CC-19 ("accept either"), and the new wire-visible reason code under wc/0.3 breaks closed-enum clients with no capability declaring it.

**W6. Repeated close: idempotency and audit mutability unspecified — medium.**
`close(D,"lost")` then `close(D,"done")` — does the second error (with which reason?), no-op, or overwrite actor/reason/ts, making the mandated close record rewritable evidence? Auto-closers diverge on all three answers.

## 2. Amendments and conformance vectors

**W1.** Amend: *"A close executes in one IMMEDIATE transaction that CASes on (last attempt = n, state ∈ {done, lost}) — exactly the set that admits T1(`attempt_of`) — and if a T1 committed first it MUST refuse with fence `dispatch_not_terminal(detail: attempt, state)`; commit order alone decides the race."*
- CC-30: arrange D last attempt lost / act close, then T1 / assert close commits one record; T1 → fence `dispatch_closed`.
- CC-31: arrange D last attempt lost / act T1 (n+1 dispatched), then close / assert fence `dispatch_not_terminal` naming attempt n+1 `dispatched`; closed=false, no record.

**W2.** Amend: *"Closure never gates attempt transitions: any transition legal before close — including late lost→done and done(paused) (CC-07, CC-24) — MUST succeed after it; a closed dispatch's finished-ness is computed from the current last-attempt state, not the close-time snapshot; close MUST be refused while the last attempt is unknown or non-terminal."*
- CC-32: arrange closed D, last attempt lost, valid nonce/token / act late lost→done / assert transition succeeds, close record unchanged, both events in §8.4.
- CC-33: arrange closed D / act late lost→done(paused) / assert finished=false on current state.
- CC-34: arrange D last attempt unknown / act close / assert typed refusal, closed=false (T1 already refuses unknown in `allocate`, so D stays uniformly open).

**W3.** Amend: *"A close MUST authenticate a dispatcher/supervisor credential (refusal `bad_credential`, recorded); actor MUST be the authenticated principal, or `backend:<id>` for policy closes — never client-supplied text."*
- CC-35: arrange D last attempt done / act close with missing/wrong credential / assert fence `bad_credential`, closed=false, refusal row appended.

**W4.** Amend: *"§4.7 get MUST expose `closed, closed_by, closed_reason, closed_ts` on every backend (false where the feature is off); closure is terminal (no reopen) and the sanctioned recovery is `allocate(dispatch_id=new, retry_of_dispatch_id=<closed id>)`, which MUST succeed and be recorded as linked; every close and every `dispatch_closed` refusal appends a dispatch-level history row."*
- CC-36: arrange closed D on a closing backend, open D′ on hub / act get both / assert closed fields present: true+actor/reason/ts vs false.
- CC-37: arrange closed D (last lost) / act allocate(new, retry_of=D), then T1(attempt_of=D) / assert allocate ok + link recorded; T1 still `dispatch_closed`.

**W5.** Amend: *"A backend MUST declare a `closes_dispatches` capability; when declared, CC-15/CC-19 pin the single correct outcome per arrange (drop 'accept either'); when undeclared, emitting `dispatch_closed` is a conformance failure; clients MUST treat unrecognized fence reasons as typed refusals with passthrough detail, so new codes evolve without a version bump."*
- CC-38: arrange undeclared backend, D last attempt done / act T1 repeatedly / assert T1 always succeeds; `dispatch_closed` never emitted.
- CC-39: arrange declared backend, D last attempt running / act close, then T1 / assert close refused, T1 unaffected — pins terminal-only precondition.
- CC-40: act parse fence_reject with an unrecognized reason / assert client surfaces it verbatim (no crash, no remap to `cas_mismatch`).

**W6.** Amend: *"Repeated close by the same principal is an idempotent no-op returning the original record; by a different principal it refuses with fence `already_closed`; close records and their history rows are append-only."*
- CC-41: arrange closed D / act close again (same principal, then different) / assert no-op then `already_closed`; original actor/reason/ts unchanged; history append-only.

## 3. Recommendation

Adopt **with the amendments**, not as drafted. The core permission is sound and already mirrors an existing invariant — closure's {done, lost} is exactly T1's retry precondition in `allocate`, so "close iff a retry would be allowed" makes the close/T1 race commit-order-decided for free — and rejecting it would only push ring20's closure into undocumented, unpinnable behavior. As drafted, however, it exports the permission without any of the discipline the rest of the contract enforces (transactional precondition, typed refusal, authenticated actor, pinned read surface, append-only audit) and deliberately weakens CC-15/CC-19 into "accept either," converting every failure above — closed-live-dispatch races, unauthenticated terminal writes, unrecoverable wrong closures, cross-backend disagreement — into conformant behavior the suite can never catch. Steelman for adopting as-is: it is a pure MAY that changes no attempt semantics and costs non-closing backends nothing, so shipping now and hardening in wc/0.4 from real incidents beats speculating about races nobody has observed. One can equally argue closure is backend-local garbage collection that never crosses a backend boundary, in which case the less the shared contract says about it, the better.
